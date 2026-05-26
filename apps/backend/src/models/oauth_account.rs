use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ─── Database Model ─────────────────────────────────────────────────────────

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct OAuthAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    /// OAuth provider name: "github", "google", "wechat"
    pub provider: String,
    /// Third-party user ID
    pub provider_user_id: String,
    /// WeChat union_id (optional)
    pub union_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl OAuthAccount {
    /// List of valid OAuth providers
    pub fn valid_providers() -> &'static [&'static str] {
        &["github", "google", "wechat"]
    }

    /// Check if a provider string is valid
    pub fn is_valid_provider(p: &str) -> bool {
        Self::valid_providers().contains(&p)
    }
}

// ─── Request / Response Types ───────────────────────────────────────────────

/// Request to bind a social account (requires authorization code)
#[derive(Debug, Deserialize)]
pub struct SocialBindRequest {
    pub provider: String,
    pub code: String,
    pub redirect_uri: String,
}

/// Request to unbind a social account
#[derive(Debug, Deserialize)]
pub struct SocialUnbindRequest {
    pub provider: String,
}

/// Response for listing bound social accounts
#[derive(Debug, Serialize)]
pub struct OAuthAccountResponse {
    pub id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub created_at: DateTime<Utc>,
}

impl From<OAuthAccount> for OAuthAccountResponse {
    fn from(account: OAuthAccount) -> Self {
        Self {
            id: account.id,
            provider: account.provider,
            provider_user_id: account.provider_user_id,
            created_at: account.created_at,
        }
    }
}
