use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

/// Database model for the permissions table
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PermissionRecord {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub module: String,
    pub created_at: DateTime<Utc>,
}

/// Public permission response
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PermissionResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub module: String,
    pub created_at: DateTime<Utc>,
}
