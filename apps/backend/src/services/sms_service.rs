use chrono::{Duration, Utc};
use sqlx::PgPool;

use crate::error::AppError;
use crate::repository::sms_code_repo;

/// SMS code length
const CODE_LENGTH: usize = 6;
/// SMS code expiry in minutes
const CODE_EXPIRY_MINS: i64 = 5;

/// Send a verification code to a phone number.
/// In development mode, the code is logged instead of actually being sent via SMS.
pub async fn send_code(pool: &PgPool, phone: &str) -> Result<String, AppError> {
    // Validate phone format (basic check)
    if phone.is_empty() || phone.len() < 10 {
        return Err(AppError::BadRequest("Invalid phone number".into()));
    }

    // Generate a 6-digit code
    let code = generate_numeric_code(CODE_LENGTH);
    let expires_at = Utc::now() + Duration::minutes(CODE_EXPIRY_MINS);

    // Store in database
    sms_code_repo::create(pool, phone, &code, expires_at).await?;

    // Development: log the code instead of sending SMS
    tracing::info!(
        phone = %phone,
        code = %code,
        "SMS verification code generated (dev mode — not actually sent)"
    );

    Ok(code)
}

/// Verify a SMS code for a phone number.
/// Returns true if the code is valid, marks it as used.
pub async fn verify_code(pool: &PgPool, phone: &str, code: &str) -> Result<bool, AppError> {
    let record = sms_code_repo::find_valid_code(pool, phone, code).await?;

    match record {
        Some(record) => {
            // Mark as used
            sms_code_repo::mark_used(pool, record.id).await?;
            Ok(true)
        }
        None => Ok(false),
    }
}

/// Generate a random numeric code of the given length
fn generate_numeric_code(length: usize) -> String {
    let mut code = String::with_capacity(length);
    for _ in 0..length {
        let digit: u8 = rand::random::<u8>() % 10;
        code.push((b'0' + digit) as char);
    }
    code
}
