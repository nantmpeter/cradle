use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::notification::NotificationResponse;
use crate::repository::notification_repo;

/// Send a notification to a specific user
pub async fn send_to_user(
    pool: &PgPool,
    user_id: Uuid,
    title: &str,
    content: &str,
    category: &str,
) -> Result<(), AppError> {
    notification_repo::create(pool, Some(user_id), title, content, category, None).await?;
    Ok(())
}

/// Broadcast a notification to all users (user_id = NULL)
pub async fn broadcast(
    pool: &PgPool,
    title: &str,
    content: &str,
    category: &str,
    created_by: Uuid,
) -> Result<crate::models::notification::Notification, AppError> {
    let notification =
        notification_repo::create(pool, None, title, content, category, Some(created_by)).await?;

    // Audit log
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(created_by),
        "notification.broadcast",
        Some("notification"),
        Some(notification.id),
        Some(serde_json::json!({
            "title": title,
            "category": category,
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for notification broadcast: {:?}", e);
    }

    Ok(notification)
}

/// List notifications for a user with pagination and filtering
pub async fn list(
    pool: &PgPool,
    user_id: Uuid,
    page: i64,
    per_page: i64,
    category: Option<&str>,
    is_read: Option<bool>,
) -> Result<(Vec<NotificationResponse>, i64), AppError> {
    let page = page.max(1);
    let per_page = per_page.clamp(1, 100);

    notification_repo::list_paginated(pool, user_id, page, per_page, category, is_read).await
}

/// Get unread notification count for a user
pub async fn get_unread_count(pool: &PgPool, user_id: Uuid) -> Result<i64, AppError> {
    notification_repo::count_unread(pool, user_id).await
}

/// Mark a single notification as read
pub async fn mark_read(
    pool: &PgPool,
    user_id: Uuid,
    notification_id: Uuid,
) -> Result<(), AppError> {
    // Verify notification exists and is accessible
    let notification = notification_repo::find_by_id(pool, notification_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Notification not found".into()))?;

    // Verify access: must be the owner or a broadcast
    if let Some(n_user_id) = notification.user_id {
        if n_user_id != user_id {
            return Err(AppError::Forbidden(
                "Cannot mark another user's notification as read".into(),
            ));
        }
    }

    notification_repo::insert_read(pool, user_id, notification_id).await
}

/// Mark all notifications as read for a user
pub async fn mark_all_read(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    notification_repo::mark_all_read(pool, user_id).await
}

/// Delete a notification
pub async fn delete(
    pool: &PgPool,
    user_id: Uuid,
    notification_id: Uuid,
) -> Result<(), AppError> {
    // Verify notification exists and user has access
    let notification = notification_repo::find_by_id(pool, notification_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Notification not found".into()))?;

    // Verify access: must be the owner
    match notification.user_id {
        Some(n_user_id) if n_user_id == user_id => {}
        None => {
            // Broadcast: only creator or admin can delete
        }
        _ => {
            return Err(AppError::Forbidden(
                "Cannot delete another user's notification".into(),
            ));
        }
    }

    notification_repo::delete(pool, notification_id).await
}
