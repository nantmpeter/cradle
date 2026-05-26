use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Database model for the audit_logs table
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub details: Option<Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Public audit log response with user email joined
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AuditLogResponse {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_email: Option<String>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub details: Option<Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Query parameters for audit log listing
#[derive(Debug, Deserialize)]
pub struct AuditLogListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub ip_address: Option<String>,
}
