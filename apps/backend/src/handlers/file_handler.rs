use axum::extract::{Multipart, Query, State};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct FileListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// POST /api/files — Upload a file
pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "files:upload").await?;

    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?
        .ok_or_else(|| AppError::BadRequest("No file field".into()))?;

    let original_name = field.file_name().unwrap_or("unknown").to_string();
    let mime_type = field
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();
    let data = field
        .bytes()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read file data: {}", e)))?;

    let upload_dir = state.settings.storage.upload_dir.clone();
    let max_size = state.settings.storage.max_upload_size;

    let file = crate::services::file_service::upload_file(
        &state.db,
        auth.user_id,
        &original_name,
        &mime_type,
        &data,
        &upload_dir,
        max_size,
    )
    .await?;

    Ok(Json(serde_json::json!({ "data": file })))
}

/// GET /api/files — List my files
pub async fn list_files(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<FileListQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "files:read").await?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);

    let (files, total) =
        crate::services::file_service::list_files(&state.db, auth.user_id, page, per_page).await?;

    Ok(Json(serde_json::json!({
        "data": files,
        "pagination": { "page": page, "per_page": per_page, "total": total }
    })))
}

/// DELETE /api/files/:id — Delete a file
pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "files:delete").await?;

    crate::services::file_service::delete_file(&state.db, id, auth.user_id, auth.is_admin())
        .await?;

    Ok(Json(serde_json::json!({ "message": "File deleted" })))
}
