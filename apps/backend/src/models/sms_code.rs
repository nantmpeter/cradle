use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// SMS verification code record
#[derive(Debug, FromRow)]
pub struct SmsVerificationCode {
    pub id: Uuid,
    pub phone: String,
    pub code: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
}
