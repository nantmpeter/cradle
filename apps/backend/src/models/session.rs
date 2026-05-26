use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

/// Database session record
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token_hash: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub last_active_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
}

/// Public session response (no refresh_token_hash exposure)
#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub last_active_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
}

impl From<Session> for SessionResponse {
    fn from(s: Session) -> Self {
        Self {
            id: s.id,
            user_id: s.user_id,
            ip_address: s.ip_address,
            user_agent: s.user_agent,
            last_active_at: s.last_active_at,
            created_at: s.created_at,
            expires_at: s.expires_at,
        }
    }
}

/// Request to create a new session
#[derive(Debug)]
pub struct CreateSessionRequest {
    pub user_id: Uuid,
    pub refresh_token_hash: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub expires_at: DateTime<Utc>,
}
