use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;
use uuid::Uuid;

// ─── ApiKeyStatus Enum ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyStatus {
    Active,
    Revoked,
}

impl ApiKeyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ApiKeyStatus::Active => "active",
            ApiKeyStatus::Revoked => "revoked",
        }
    }
}

impl std::fmt::Display for ApiKeyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ApiKeyStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(ApiKeyStatus::Active),
            "revoked" => Ok(ApiKeyStatus::Revoked),
            _ => Err(format!("Unknown API key status: {}", s)),
        }
    }
}

// ─── Database Model ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub key_prefix: String,
    pub user_id: Uuid,
    pub scopes: JsonValue,
    pub rate_limit: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl ApiKey {
    /// Parse the scopes JSON value into a Vec<String>.
    pub fn get_scopes(&self) -> Vec<String> {
        self.scopes
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

// ─── Request / Response Types ───────────────────────────────────────────────

/// Create API key request
#[derive(Debug, Deserialize, validator::Validate)]
pub struct CreateApiKeyRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// API key response (no secret)
#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub scopes: Vec<String>,
    pub status: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<ApiKey> for ApiKeyResponse {
    fn from(key: ApiKey) -> Self {
        let scopes = key.get_scopes();
        Self {
            id: key.id,
            name: key.name,
            key_prefix: key.key_prefix,
            scopes,
            status: key.status,
            expires_at: key.expires_at,
            last_used_at: key.last_used_at,
            created_at: key.created_at,
        }
    }
}

/// Create API key response (includes plaintext key, shown only once)
#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    /// Plaintext key — only available at creation time
    pub key: String,
    pub key_prefix: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Paginated list response
#[derive(Debug, Serialize)]
pub struct ApiKeyListResponse {
    pub items: Vec<ApiKeyResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}
