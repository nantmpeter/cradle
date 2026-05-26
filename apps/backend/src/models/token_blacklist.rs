use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Database model for the token_blacklist table
#[derive(Debug, sqlx::FromRow)]
pub struct TokenBlacklist {
    pub jti: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
