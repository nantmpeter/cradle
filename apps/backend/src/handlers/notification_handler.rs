use axum::extract::{Path, Query, State};
use axum::Json;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::notification::{BroadcastNotificationRequest, NotificationListQuery};
use crate::AppState;

/// GET /api/notifications — List notifications for current user
pub async fn list(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<NotificationListQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "notifications:read").await?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);

    let (notifications, total) = crate::services::notification_service::list(
        &state.db,
        auth.user_id,
        page,
        per_page,
        query.category.as_deref(),
        query.is_read,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "data": notifications,
        "pagination": {
            "page": page,
            "per_page": per_page,
            "total": total,
        }
    })))
}

/// GET /api/notifications/unread-count — Get unread notification count
pub async fn unread_count(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "notifications:read").await?;

    let count =
        crate::services::notification_service::get_unread_count(&state.db, auth.user_id).await?;

    Ok(Json(serde_json::json!({
        "unread_count": count,
    })))
}

/// POST /api/notifications/{id}/read — Mark notification as read
pub async fn mark_read(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "notifications:read").await?;

    crate::services::notification_service::mark_read(&state.db, auth.user_id, id).await?;

    Ok(Json(serde_json::json!({"message": "Notification marked as read"})))
}

/// POST /api/notifications/read-all — Mark all notifications as read
pub async fn mark_all_read(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "notifications:read").await?;

    crate::services::notification_service::mark_all_read(&state.db, auth.user_id).await?;

    Ok(Json(serde_json::json!({"message": "All notifications marked as read"})))
}

/// DELETE /api/notifications/{id} — Delete a notification
pub async fn delete_notification(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "notifications:read").await?;

    crate::services::notification_service::delete(&state.db, auth.user_id, id).await?;

    Ok(Json(serde_json::json!({"message": "Notification deleted"})))
}

/// POST /api/notifications/broadcast — Broadcast a notification to all users
pub async fn broadcast(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<BroadcastNotificationRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "notifications:send").await?;

    let notification = crate::services::notification_service::broadcast(
        &state.db,
        &req.title,
        &req.content,
        &req.category,
        auth.user_id,
    )
    .await?;

    // Push to SSE broadcast channel
    let sse_notif = crate::models::notification::SseNotification {
        id: notification.id,
        user_id: None, // broadcast
        title: notification.title.clone(),
        content: notification.content.clone(),
        category: notification.category.clone(),
    };
    let _ = state.notification_tx.send(sse_notif);

    Ok(Json(serde_json::json!({
        "id": notification.id,
        "message": "Notification broadcast successfully",
    })))
}
