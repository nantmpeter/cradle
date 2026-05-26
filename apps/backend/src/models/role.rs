use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Database model for the roles table
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RoleRecord {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub data_scope: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Public role response with permissions included
#[derive(Debug, Serialize)]
pub struct RoleResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub data_scope: String,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create role request
#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub permission_ids: Vec<Uuid>,
    pub data_scope: Option<String>,
}

/// Update role request
#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub data_scope: Option<String>,
}

/// Update role permissions request
#[derive(Debug, Deserialize)]
pub struct UpdateRolePermissionsRequest {
    pub permission_ids: Vec<Uuid>,
}
