use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::config::WebhookSettings;
use crate::models::webhook::WebhookEvent;
use crate::repository::webhook_repo;

type HmacSha256 = Hmac<Sha256>;

/// Dispatch a webhook event to all matching active webhooks.
/// Looks up webhooks subscribed to this event and delivers payloads asynchronously.
pub async fn dispatch_event(
    pool: &sqlx::PgPool,
    http_client: &reqwest::Client,
    webhook_settings: &WebhookSettings,
    event: &str,
    payload: &serde_json::Value,
) {
    // Find matching active webhooks
    let webhooks = match webhook_repo::find_active_by_event(pool, event).await {
        Ok(w) => w,
        Err(e) => {
            tracing::error!("Failed to find webhooks for event '{}': {:?}", event, e);
            return;
        }
    };

    if webhooks.is_empty() {
        return;
    }

    let payload_bytes = match serde_json::to_vec(payload) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("Failed to serialize webhook payload: {:?}", e);
            return;
        }
    };

    // Deliver each webhook asynchronously
    for webhook in webhooks {
        let pool_clone = pool.clone();
        let http_client_clone = http_client.clone();
        let event_str = event.to_string();
        let payload_clone = payload.clone();
        let payload_bytes_clone = payload_bytes.clone();
        let timeout_secs = webhook_settings.timeout_secs;

        tokio::spawn(async move {
            deliver_single_webhook(
                &pool_clone,
                &http_client_clone,
                &webhook,
                &event_str,
                &payload_clone,
                &payload_bytes_clone,
                timeout_secs,
            )
            .await;
        });
    }
}

/// Deliver a webhook payload to a single endpoint.
/// Records the delivery result (success or failure) in the database.
async fn deliver_single_webhook(
    pool: &sqlx::PgPool,
    http_client: &reqwest::Client,
    webhook: &crate::models::webhook::Webhook,
    event: &str,
    _payload: &serde_json::Value,
    payload_bytes: &[u8],
    timeout_secs: u64,
) {
    // Compute HMAC-SHA256 signature
    let signature = compute_signature(&webhook.secret, payload_bytes);

    // Build the webhook payload envelope
    let body = serde_json::json!({
        "event": event,
        "timestamp": Utc::now().to_rfc3339(),
        "data": serde_json::from_slice::<serde_json::Value>(payload_bytes).unwrap_or(serde_json::json!({})),
    });

    // Send HTTP POST with timeout
    let result = http_client
        .post(&webhook.url)
        .header("Content-Type", "application/json")
        .header("X-Webhook-Signature", &signature)
        .header("X-Webhook-Event", event)
        .header("X-Webhook-ID", webhook.id.to_string())
        .timeout(std::time::Duration::from_secs(timeout_secs.max(1)))
        .json(&body)
        .send()
        .await;

    let (response_status, response_body) = match result {
        Ok(resp) => {
            let status = resp.status().as_u16() as i32;
            let body_text = resp.text().await.unwrap_or_default();
            (Some(status), Some(body_text))
        }
        Err(e) => {
            tracing::warn!(
                webhook_id = %webhook.id,
                error = %e,
                "Webhook delivery failed"
            );
            (None, Some(e.to_string()))
        }
    };

    // Determine if we need a retry
    let success = response_status.map_or(false, |s| (200..300).contains(&s));
    let next_retry_at = if success {
        None
    } else {
        Some(Utc::now() + Duration::minutes(1))
    };

    // Record delivery
    if let Err(e) = webhook_repo::create_delivery(
        pool,
        webhook.id,
        event,
        &serde_json::json!({
            "event": event,
            "signature": signature,
        }),
        response_status,
        response_body.as_deref(),
        next_retry_at,
    )
    .await
    {
        tracing::error!("Failed to record webhook delivery: {:?}", e);
    }
}

/// Compute HMAC-SHA256 signature for a webhook payload
fn compute_signature(secret: &str, payload: &[u8]) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC key length is always valid");
    mac.update(payload);
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    URL_SAFE_NO_PAD.encode(code_bytes)
}

/// Retry failed webhook deliveries that are due.
pub async fn retry_failed_deliveries(
    pool: &sqlx::PgPool,
    http_client: &reqwest::Client,
    webhook_settings: &WebhookSettings,
) {
    let pending = match webhook_repo::find_pending_retries(pool, webhook_settings.max_retries as i32)
        .await
    {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("Failed to find pending webhook retries: {:?}", e);
            return;
        }
    };

    for delivery in pending {
        // Look up the webhook
        let webhook = match webhook_repo::find_by_id(pool, delivery.webhook_id).await {
            Ok(Some(w)) => w,
            Ok(None) => continue,
            Err(_) => continue,
        };

        if webhook.status != "active" {
            continue;
        }

        // Re-deliver
        let payload_bytes = serde_json::to_vec(&delivery.payload).unwrap_or_default();
        let signature = compute_signature(&webhook.secret, &payload_bytes);

        let body = serde_json::json!({
            "event": delivery.event,
            "timestamp": Utc::now().to_rfc3339(),
            "retry_count": delivery.retry_count + 1,
            "data": delivery.payload,
        });

        let result = http_client
            .post(&webhook.url)
            .header("Content-Type", "application/json")
            .header("X-Webhook-Signature", &signature)
            .header("X-Webhook-Event", &delivery.event)
            .header("X-Webhook-ID", webhook.id.to_string())
            .timeout(std::time::Duration::from_secs(webhook_settings.timeout_secs.max(1)))
            .json(&body)
            .send()
            .await;

        let (response_status, response_body) = match result {
            Ok(resp) => {
                let status = resp.status().as_u16() as i32;
                let body_text = resp.text().await.unwrap_or_default();
                (Some(status), Some(body_text))
            }
            Err(e) => (None, Some(e.to_string())),
        };

        let success = response_status.map_or(false, |s| (200..300).contains(&s));
        let next_retry_at = if success {
            None
        } else if delivery.retry_count + 1 >= webhook_settings.max_retries as i32 {
            // Max retries reached, no more retries
            None
        } else {
            // Calculate next retry with exponential backoff
            let retry_idx = (delivery.retry_count as usize).min(webhook_settings.retry_intervals_secs.len().saturating_sub(1));
            let delay_secs = webhook_settings.retry_intervals_secs.get(retry_idx).copied().unwrap_or(1800);
            Some(Utc::now() + Duration::seconds(delay_secs as i64))
        };

        if let Err(e) = webhook_repo::update_delivery_retry(
            pool,
            delivery.id,
            response_status,
            response_body.as_deref(),
            next_retry_at,
        )
        .await
        {
            tracing::error!("Failed to update webhook delivery retry: {:?}", e);
        }
    }
}

/// Start a background worker that processes the webhook event channel.
/// Receives events from the mpsc channel and dispatches them.
pub fn start_event_worker(
    pool: sqlx::PgPool,
    http_client: reqwest::Client,
    webhook_settings: WebhookSettings,
    mut rx: tokio::sync::mpsc::Receiver<WebhookEvent>,
) {
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            dispatch_event(
                &pool,
                &http_client,
                &webhook_settings,
                &event.event,
                &event.payload,
            )
            .await;
        }
        tracing::info!("Webhook event worker stopped");
    });
}

/// Start the retry background worker that periodically checks for failed deliveries.
pub fn start_retry_worker(
    pool: sqlx::PgPool,
    http_client: reqwest::Client,
    webhook_settings: WebhookSettings,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            retry_failed_deliveries(&pool, &http_client, &webhook_settings).await;
        }
    });
}
