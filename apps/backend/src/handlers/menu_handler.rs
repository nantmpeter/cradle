use axum::extract::{Path, State};
use axum::Json;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::menu::{CreateMenuRequest, UpdateMenuRequest};
use crate::AppState;

/// GET /api/menus — List all menus (admin)
pub async fn list_menus(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "menus:read").await?;

    let menus = crate::services::menu_service::list_menus(&state.db).await?;
    Ok(Json(serde_json::json!({ "data": menus })))
}

/// GET /api/menus/tree — Get menu tree for current user
pub async fn get_menu_tree(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let permissions =
        crate::services::auth_service::get_user_permissions(&state.db, auth.user_id).await?;
    let tree = crate::services::menu_service::get_menu_tree(&state.db, &permissions).await?;
    Ok(Json(serde_json::json!({ "data": tree })))
}

/// POST /api/menus — Create menu
pub async fn create_menu(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateMenuRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "menus:create").await?;
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let menu = crate::services::menu_service::create_menu(&state.db, &req).await?;
    Ok(Json(serde_json::json!({ "data": menu })))
}

/// PUT /api/menus/:id — Update menu
pub async fn update_menu(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateMenuRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "menus:update").await?;
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let menu = crate::services::menu_service::update_menu(&state.db, id, &req).await?;
    Ok(Json(serde_json::json!({ "data": menu })))
}

/// DELETE /api/menus/:id — Delete menu
pub async fn delete_menu(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "menus:delete").await?;

    crate::services::menu_service::delete_menu(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Menu deleted" })))
}
