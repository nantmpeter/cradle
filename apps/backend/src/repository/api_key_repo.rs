use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::api_key::ApiKey;

/// Helper struct for count query result
#[derive(sqlx::FromRow)]
struct CountResult {
    count: i64,
}

/// Create a new API key record
pub async fn create(
    pool: &PgPool,
    key_hash: &str,
    key_prefix: &str,
    user_id: Uuid,
    name: &str,
    scopes: &[String],
    expires_at: Option<DateTime<Utc>>,
) -> Result<ApiKey, AppError> {
    let scopes_json = serde_json::json!(scopes);
    let key = sqlx::query_as::<_, ApiKey>(
        r#"INSERT INTO api_keys (name, key_hash, key_prefix, user_id, scopes, expires_at)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING *"#,
    )
    .bind(name)
    .bind(key_hash)
    .bind(key_prefix)
    .bind(user_id)
    .bind(scopes_json)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(key)
}

/// Find an API key by its key hash
pub async fn find_by_key_hash(pool: &PgPool, key_hash: &str) -> Result<Option<ApiKey>, AppError> {
    let key = sqlx::query_as::<_, ApiKey>(
        "SELECT * FROM api_keys WHERE key_hash = $1",
    )
    .bind(key_hash)
    .fetch_optional(pool)
    .await?;

    Ok(key)
}

/// Find an API key by ID
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ApiKey>, AppError> {
    let key = sqlx::query_as::<_, ApiKey>("SELECT * FROM api_keys WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(key)
}

/// List API keys for a user with pagination
pub async fn list_by_user(
    pool: &PgPool,
    user_id: Uuid,
    page: i64,
    per_page: i64,
) -> Result<(Vec<ApiKey>, i64), AppError> {
    let offset = (page - 1).max(0) * per_page;

    let keys = sqlx::query_as::<_, ApiKey>(
        r#"SELECT * FROM api_keys
           WHERE user_id = $1
           ORDER BY created_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(user_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let count_result: CountResult =
        sqlx::query_as::<_, CountResult>("SELECT COUNT(*) as count FROM api_keys WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;

    Ok((keys, count_result.count))
}

/// Update API key status (e.g. revoke)
pub async fn update_status(pool: &PgPool, id: Uuid, status: &str) -> Result<(), AppError> {
    let now = Utc::now();
    sqlx::query("UPDATE api_keys SET status = $1, updated_at = $2 WHERE id = $3")
        .bind(status)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Update last_used_at timestamp for an API key
pub async fn update_last_used_at(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let now = Utc::now();
    sqlx::query("UPDATE api_keys SET last_used_at = $1 WHERE id = $2")
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}
