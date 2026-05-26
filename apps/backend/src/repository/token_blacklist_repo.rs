use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

/// Add a JWT ID to the blacklist
pub async fn add(pool: &PgPool, jti: Uuid, expires_at: DateTime<Utc>) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO token_blacklist (jti, expires_at) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(jti)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Check if a JWT ID is blacklisted
pub async fn is_blacklisted(pool: &PgPool, jti: Uuid) -> Result<bool, AppError> {
    #[derive(sqlx::FromRow)]
    struct ExistsResult {
        exists: bool,
    }

    let result: ExistsResult = sqlx::query_as::<_, ExistsResult>(
        "SELECT EXISTS(SELECT 1 FROM token_blacklist WHERE jti = $1) AS exists",
    )
    .bind(jti)
    .fetch_one(pool)
    .await?;

    Ok(result.exists)
}

/// Clean up expired blacklist entries
pub async fn cleanup_expired(pool: &PgPool) -> Result<(), AppError> {
    sqlx::query("DELETE FROM token_blacklist WHERE expires_at < NOW()")
        .execute(pool)
        .await?;
    Ok(())
}
