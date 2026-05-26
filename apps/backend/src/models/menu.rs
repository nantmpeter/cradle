use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Database menu record
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Menu {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub title_key: String,
    pub title_label: String,
    pub path: String,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub permission_id: Option<Uuid>,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Public menu response
#[derive(Debug, Serialize)]
pub struct MenuResponse {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub title_key: String,
    pub title_label: String,
    pub path: String,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub permission_id: Option<Uuid>,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<Menu> for MenuResponse {
    fn from(m: Menu) -> Self {
        Self {
            id: m.id,
            parent_id: m.parent_id,
            title_key: m.title_key,
            title_label: m.title_label,
            path: m.path,
            icon: m.icon,
            sort_order: m.sort_order,
            permission_id: m.permission_id,
            status: m.status,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

/// Menu tree node for the frontend dynamic menu
#[derive(Debug, Serialize, Clone)]
pub struct MenuTreeNode {
    pub id: Uuid,
    pub title_key: String,
    pub title_label: String,
    pub path: String,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub children: Vec<MenuTreeNode>,
}

/// Create menu request
#[derive(Debug, Deserialize, Validate)]
pub struct CreateMenuRequest {
    pub parent_id: Option<Uuid>,
    #[validate(length(min = 1, max = 100))]
    pub title_key: String,
    #[validate(length(min = 1, max = 100))]
    pub title_label: String,
    #[validate(length(min = 1, max = 200))]
    pub path: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub sort_order: i32,
    pub permission_id: Option<Uuid>,
}

/// Update menu request
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMenuRequest {
    pub parent_id: Option<Uuid>,
    #[validate(length(min = 1, max = 100))]
    pub title_key: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub title_label: Option<String>,
    #[validate(length(min = 1, max = 200))]
    pub path: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
    pub permission_id: Option<Uuid>,
    pub status: Option<String>,
}
