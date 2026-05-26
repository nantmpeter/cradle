use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

/// Get the encrypted TOTP secret for a user
pub async fn get_totp_secret(pool: &PgPool, user_id: Uuid) -> Result<Option<String>, AppError> {
    let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT totp_secret FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    Ok(row.and_then(|r| r.0))
}

/// Save the encrypted TOTP secret for a user
pub async fn save_totp_secret(pool: &PgPool, user_id: Uuid, encrypted_secret: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET totp_secret = $2, updated_at = NOW() WHERE id = $1")
        .bind(user_id)
        .bind(encrypted_secret)
        .execute(pool)
        .await?;

    Ok(())
}

/// Get recovery codes for a user (stored as JSONB array of objects)
pub async fn get_recovery_codes(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<serde_json::Value>, AppError> {
    let row: Option<(Option<serde_json::Value>,)> =
        sqlx::query_as("SELECT recovery_codes FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    Ok(row.and_then(|r| r.0))
}

/// Save recovery codes (hashed) for a user
pub async fn save_recovery_codes(
    pool: &PgPool,
    user_id: Uuid,
    codes: serde_json::Value,
) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET recovery_codes = $2, updated_at = NOW() WHERE id = $1")
        .bind(user_id)
        .bind(codes)
        .execute(pool)
        .await?;

    Ok(())
}

/// Enable or disable 2FA for a user
pub async fn set_2fa_enabled(pool: &PgPool, user_id: Uuid, enabled: bool) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET two_factor_enabled = $2, updated_at = NOW() WHERE id = $1")
        .bind(user_id)
        .bind(enabled)
        .execute(pool)
        .await?;

    Ok(())
}

/// Reset 2FA completely: clear secret, codes, and disable flag
pub async fn reset_2fa(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE users SET totp_secret = NULL, two_factor_enabled = false, recovery_codes = '[]'::jsonb, updated_at = NOW() WHERE id = $1",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}
