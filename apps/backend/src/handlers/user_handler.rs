use axum::extract::{Multipart, Path, Query, State};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::user::{
    ChangeMyPasswordRequest, CreateUserRequest, MeResponse, ResetPasswordRequest, UpdateStatusRequest,
    UpdateUserRequest, UserResponse,
};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub role: Option<String>,
    pub role_id: Option<Uuid>,
    pub status: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// GET /api/users
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);

    let (users, total) = crate::services::user_service::list_users(
        &state.db,
        &auth,
        query.search,
        query.role,
        query.role_id,
        query.status,
        query.sort_by,
        query.sort_order,
        page,
        per_page,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "data": users,
        "pagination": {
            "page": page,
            "per_page": per_page,
            "total": total,
        }
    })))
}

/// GET /api/users/me
pub async fn get_me(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<MeResponse>, AppError> {
    let me = crate::services::user_service::get_me(&state.db, auth.user_id).await?;
    Ok(Json(me))
}

/// GET /api/users/:id
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    let user = crate::services::user_service::get_user(&state.db, id).await?;
    Ok(Json(user))
}

/// POST /api/users — Create user (admin+)
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let user = crate::services::user_service::create_user(&state.db, &auth, &req).await?;

    // Fire webhook event: user.created
    let _ = state.webhook_tx.send(crate::models::webhook::WebhookEvent {
        event: "user.created".into(),
        payload: serde_json::json!({
            "id": user.id.to_string(),
            "email": user.email,
            "name": user.name,
            "role": user.role.to_string(),
        }),
    }).await;

    Ok(Json(user))
}

/// PUT /api/users/:id
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let user = crate::services::user_service::update_user(&state.db, &auth, id, &req).await?;

    // Fire webhook event: user.updated
    let _ = state.webhook_tx.send(crate::models::webhook::WebhookEvent {
        event: "user.updated".into(),
        payload: serde_json::json!({
            "id": user.id.to_string(),
            "email": user.email,
            "name": user.name,
            "role": user.role.to_string(),
        }),
    }).await;

    Ok(Json(user))
}

/// DELETE /api/users/:id
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    crate::services::user_service::delete_user(&state.db, &auth, id).await?;

    // Fire webhook event: user.deleted
    let _ = state.webhook_tx.send(crate::models::webhook::WebhookEvent {
        event: "user.deleted".into(),
        payload: serde_json::json!({ "id": id.to_string() }),
    }).await;

    Ok(Json(serde_json::json!({"message": "User deleted"})))
}

/// PUT /api/users/:id/status — Update status (admin+)
pub async fn update_status(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateStatusRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let user =
        crate::services::user_service::update_status(&state.db, &auth, id, &req).await?;
    Ok(Json(user))
}

/// PUT /api/users/:id/password — Reset password (admin+)
pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    crate::services::user_service::reset_password(&state.db, &auth, id, &req).await?;
    Ok(Json(serde_json::json!({"message": "Password reset successfully"})))
}

/// PUT /api/users/me/password — Change own password (authenticated)
pub async fn change_my_password(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<ChangeMyPasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    crate::services::user_service::change_my_password(&state.db, &auth, &req).await?;
    Ok(Json(serde_json::json!({"message": "Password changed successfully"})))
}

/// POST /api/users/me/avatar — Upload avatar
pub async fn upload_avatar(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut avatar_url: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("Multipart error: {}", e))
    })? {
        let filename = field.file_name().unwrap_or("avatar").to_string();
        let data = field.bytes().await.map_err(|e| {
            AppError::BadRequest(format!("Failed to read file: {}", e))
        })?;

        // Use file_service to save the file
        let mime_type = "image/png".to_string(); // Default; could detect from content
        let file_resp = crate::services::file_service::upload_file(
            &state.db,
            auth.user_id,
            &filename,
            &mime_type,
            &data,
            "uploads/avatars",
            2 * 1024 * 1024, // 2MB max for avatar
        )
        .await?;

        avatar_url = Some(format!("/api/files/{}/{}", file_resp.id, file_resp.filename));
    }

    let url = avatar_url.ok_or_else(|| AppError::BadRequest("No file uploaded".into()))?;

    // Update user's avatar_url
    crate::repository::user_repo::update_avatar_url(&state.db, auth.user_id, Some(&url)).await?;

    Ok(Json(serde_json::json!({ "avatar_url": url })))
}
