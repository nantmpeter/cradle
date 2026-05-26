use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::user::RefreshToken;

/// Create a new refresh token record
pub async fn create(
    pool: &PgPool,
    token_hash: &str,
    user_id: Uuid,
    expires_at: DateTime<Utc>,
) -> Result<RefreshToken, AppError> {
    let token = sqlx::query_as::<_, RefreshToken>(
        "INSERT INTO refresh_tokens (token_hash, user_id, expires_at) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(token_hash)
    .bind(user_id)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(token)
}

/// Find a refresh token by its hash
pub async fn find_by_hash(pool: &PgPool, token_hash: &str) -> Result<Option<RefreshToken>, AppError> {
    let token = sqlx::query_as::<_, RefreshToken>(
        "SELECT * FROM refresh_tokens WHERE token_hash = $1",
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await?;

    Ok(token)
}

/// Revoke a specific refresh token
pub async fn revoke(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = NOW() WHERE id = $1 AND revoked_at IS NULL",
    )
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Refresh token {} not found or already revoked",
            id
        )));
    }

    Ok(())
}

/// Revoke all refresh tokens for a given user
pub async fn revoke_all_for_user(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}
