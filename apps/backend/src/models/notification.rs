use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Database model for notifications table
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub title: String,
    pub content: String,
    pub category: String,
    pub is_pinned: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Notification response with read status
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub title: String,
    pub content: String,
    pub category: String,
    pub is_pinned: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub is_read: bool,
}

/// Query parameters for listing notifications
#[derive(Debug, Deserialize)]
pub struct NotificationListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub category: Option<String>,
    pub is_read: Option<bool>,
}

/// Request to create a notification for a specific user
#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    #[serde(default = "default_category")]
    pub category: String,
}

fn default_category() -> String {
    "system".to_string()
}

/// Request to broadcast a notification to all users
#[derive(Debug, Deserialize)]
pub struct BroadcastNotificationRequest {
    pub title: String,
    pub content: String,
    #[serde(default = "default_category")]
    pub category: String,
}

/// SSE notification event sent via broadcast channel
#[derive(Debug, Clone, Serialize)]
pub struct SseNotification {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub title: String,
    pub content: String,
    pub category: String,
}
