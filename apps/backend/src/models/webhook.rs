use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

// ─── Database Models ─────────────────────────────────────────────────────────

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct Webhook {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    /// HMAC signing secret
    pub secret: String,
    /// JSONB array of event names: ["user.created", "user.updated"]
    pub events: JsonValue,
    pub api_key_id: Option<Uuid>,
    /// "active" or "disabled"
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Webhook {
    /// Parse the events JSONB value into a Vec<String>.
    pub fn get_events(&self) -> Vec<String> {
        self.events
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event: String,
    pub payload: JsonValue,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub delivered_at: DateTime<Utc>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub retry_count: i32,
}

// ─── Request / Response Types ───────────────────────────────────────────────

#[derive(Debug, Deserialize, validator::Validate)]
pub struct CreateWebhookRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1))]
    pub url: String,
    pub events: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWebhookRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub status: Option<String>,
}

/// Webhook response (excludes secret)
#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Webhook> for WebhookResponse {
    fn from(w: Webhook) -> Self {
        let events = w.get_events();
        Self {
            id: w.id,
            name: w.name,
            url: w.url,
            events,
            status: w.status,
            created_at: w.created_at,
            updated_at: w.updated_at,
        }
    }
}

/// Webhook delivery response
#[derive(Debug, Serialize)]
pub struct WebhookDeliveryResponse {
    pub id: Uuid,
    pub event: String,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub delivered_at: DateTime<Utc>,
    pub retry_count: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
}

impl From<WebhookDelivery> for WebhookDeliveryResponse {
    fn from(d: WebhookDelivery) -> Self {
        Self {
            id: d.id,
            event: d.event,
            response_status: d.response_status,
            response_body: d.response_body,
            delivered_at: d.delivered_at,
            retry_count: d.retry_count,
            next_retry_at: d.next_retry_at,
        }
    }
}

/// Paginated webhook list response
#[derive(Debug, Serialize)]
pub struct WebhookListResponse {
    pub items: Vec<WebhookResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

/// Paginated delivery list response
#[derive(Debug, Serialize)]
pub struct DeliveryListResponse {
    pub items: Vec<WebhookDeliveryResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

/// Internal event type for the webhook channel
#[derive(Debug, Clone)]
pub struct WebhookEvent {
    pub event: String,
    pub payload: serde_json::Value,
}
