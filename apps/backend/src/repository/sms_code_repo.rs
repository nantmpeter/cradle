use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::error::AppError;
use crate::models::sms_code::SmsVerificationCode;

/// Create a new SMS verification code
pub async fn create(
    pool: &PgPool,
    phone: &str,
    code: &str,
    expires_at: DateTime<Utc>,
) -> Result<SmsVerificationCode, AppError> {
    let record = sqlx::query_as::<_, SmsVerificationCode>(
        r#"INSERT INTO sms_verification_codes (phone, code, expires_at)
           VALUES ($1, $2, $3)
           RETURNING *"#,
    )
    .bind(phone)
    .bind(code)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(record)
}

/// Find the latest unused, non-expired code for a phone number
pub async fn find_valid_code(
    pool: &PgPool,
    phone: &str,
    code: &str,
) -> Result<Option<SmsVerificationCode>, AppError> {
    let record = sqlx::query_as::<_, SmsVerificationCode>(
        r#"SELECT * FROM sms_verification_codes
           WHERE phone = $1 AND code = $2 AND used = FALSE AND expires_at > $3
           ORDER BY created_at DESC
           LIMIT 1"#,
    )
    .bind(phone)
    .bind(code)
    .bind(Utc::now())
    .fetch_optional(pool)
    .await?;

    Ok(record)
}

/// Mark a verification code as used
pub async fn mark_used(pool: &PgPool, id: uuid::Uuid) -> Result<(), AppError> {
    sqlx::query("UPDATE sms_verification_codes SET used = TRUE WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Clean up expired codes (older than 24 hours)
pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, AppError> {
    let cutoff = Utc::now() - chrono::Duration::hours(24);
    let result = sqlx::query("DELETE FROM sms_verification_codes WHERE expires_at < $1")
        .bind(cutoff)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}
