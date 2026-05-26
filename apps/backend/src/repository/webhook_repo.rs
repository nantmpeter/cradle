use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::webhook::{Webhook, WebhookDelivery};

/// Helper struct for count query result
#[derive(sqlx::FromRow)]
struct CountResult {
    count: i64,
}

/// Create a new webhook
pub async fn create(
    pool: &PgPool,
    name: &str,
    url: &str,
    secret: &str,
    events: &[String],
) -> Result<Webhook, AppError> {
    let events_json = serde_json::json!(events);
    let webhook = sqlx::query_as::<_, Webhook>(
        r#"INSERT INTO webhooks (name, url, secret, events)
           VALUES ($1, $2, $3, $4)
           RETURNING *"#,
    )
    .bind(name)
    .bind(url)
    .bind(secret)
    .bind(events_json)
    .fetch_one(pool)
    .await?;

    Ok(webhook)
}

/// Find a webhook by ID
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Webhook>, AppError> {
    let webhook = sqlx::query_as::<_, Webhook>("SELECT * FROM webhooks WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(webhook)
}

/// List all webhooks with pagination
pub async fn list(
    pool: &PgPool,
    page: i64,
    per_page: i64,
) -> Result<(Vec<Webhook>, i64), AppError> {
    let offset = (page - 1).max(0) * per_page;

    let webhooks = sqlx::query_as::<_, Webhook>(
        r#"SELECT * FROM webhooks
           ORDER BY created_at DESC
           LIMIT $1 OFFSET $2"#,
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let count_result: CountResult =
        sqlx::query_as::<_, CountResult>("SELECT COUNT(*) as count FROM webhooks")
            .fetch_one(pool)
            .await?;

    Ok((webhooks, count_result.count))
}

/// Update a webhook
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    url: Option<&str>,
    events: Option<&[String]>,
    status: Option<&str>,
) -> Result<Webhook, AppError> {
    // Build dynamic UPDATE query
    let mut set_clauses = Vec::new();
    let mut param_idx = 2u32; // $1 is id
    let mut query = String::from("UPDATE webhooks SET updated_at = NOW()");

    if name.is_some() {
        set_clauses.push(format!("name = ${}", param_idx));
        param_idx += 1;
    }
    if url.is_some() {
        set_clauses.push(format!("url = ${}", param_idx));
        param_idx += 1;
    }
    if events.is_some() {
        set_clauses.push(format!("events = ${}", param_idx));
        param_idx += 1;
    }
    if status.is_some() {
        set_clauses.push(format!("status = ${}", param_idx));
        let _ = param_idx + 1; // param_idx not needed after last clause
    }

    if !set_clauses.is_empty() {
        query.push_str(", ");
        query.push_str(&set_clauses.join(", "));
    }

    query.push_str(" WHERE id = $1 RETURNING *");

    // Build and bind query dynamically
    let q = sqlx::query_as::<_, Webhook>(&query);

    let q = q.bind(id);
    let mut q = q;
    if let Some(n) = name {
        q = q.bind(n);
    }
    if let Some(u) = url {
        q = q.bind(u);
    }
    if let Some(e) = events {
        q = q.bind(serde_json::json!(e));
    }
    if let Some(s) = status {
        q = q.bind(s);
    }

    let webhook = q.fetch_one(pool).await?;
    Ok(webhook)
}

/// Delete a webhook
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM webhooks WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Find all active webhooks that subscribe to a given event.
/// Uses PostgreSQL JSONB `@>` operator.
pub async fn find_active_by_event(
    pool: &PgPool,
    event: &str,
) -> Result<Vec<Webhook>, AppError> {
    let event_json = serde_json::json!([event]);
    let webhooks = sqlx::query_as::<_, Webhook>(
        "SELECT * FROM webhooks WHERE status = 'active' AND events @> $1::jsonb",
    )
    .bind(event_json)
    .fetch_all(pool)
    .await?;

    Ok(webhooks)
}

/// Create a delivery record
pub async fn create_delivery(
    pool: &PgPool,
    webhook_id: Uuid,
    event: &str,
    payload: &serde_json::Value,
    response_status: Option<i32>,
    response_body: Option<&str>,
    next_retry_at: Option<DateTime<Utc>>,
) -> Result<WebhookDelivery, AppError> {
    let delivery = sqlx::query_as::<_, WebhookDelivery>(
        r#"INSERT INTO webhook_deliveries (webhook_id, event, payload, response_status, response_body, next_retry_at)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING *"#,
    )
    .bind(webhook_id)
    .bind(event)
    .bind(payload)
    .bind(response_status)
    .bind(response_body)
    .bind(next_retry_at)
    .fetch_one(pool)
    .await?;

    Ok(delivery)
}

/// List deliveries for a webhook with pagination
pub async fn find_deliveries_by_webhook_id(
    pool: &PgPool,
    webhook_id: Uuid,
    page: i64,
    per_page: i64,
) -> Result<(Vec<WebhookDelivery>, i64), AppError> {
    let offset = (page - 1).max(0) * per_page;

    let deliveries = sqlx::query_as::<_, WebhookDelivery>(
        r#"SELECT * FROM webhook_deliveries
           WHERE webhook_id = $1
           ORDER BY delivered_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(webhook_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let count_result: CountResult = sqlx::query_as::<_, CountResult>(
        "SELECT COUNT(*) as count FROM webhook_deliveries WHERE webhook_id = $1",
    )
    .bind(webhook_id)
    .fetch_one(pool)
    .await?;

    Ok((deliveries, count_result.count))
}

/// Update a delivery record after retry
pub async fn update_delivery_retry(
    pool: &PgPool,
    id: Uuid,
    response_status: Option<i32>,
    response_body: Option<&str>,
    next_retry_at: Option<DateTime<Utc>>,
) -> Result<(), AppError> {
    sqlx::query(
        r#"UPDATE webhook_deliveries
           SET response_status = $1, response_body = $2, next_retry_at = $3, retry_count = retry_count + 1
           WHERE id = $4"#,
    )
    .bind(response_status)
    .bind(response_body)
    .bind(next_retry_at)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Find deliveries that are pending retry (next_retry_at <= NOW() and retry_count < max)
pub async fn find_pending_retries(
    pool: &PgPool,
    max_retries: i32,
) -> Result<Vec<WebhookDelivery>, AppError> {
    let deliveries = sqlx::query_as::<_, WebhookDelivery>(
        r#"SELECT * FROM webhook_deliveries
           WHERE next_retry_at IS NOT NULL
             AND next_retry_at <= NOW()
             AND retry_count < $1
           ORDER BY next_retry_at ASC
           LIMIT 50"#,
    )
    .bind(max_retries)
    .fetch_all(pool)
    .await?;

    Ok(deliveries)
}
