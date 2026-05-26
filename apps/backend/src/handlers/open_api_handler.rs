use axum::extract::{Path, Query, State};
use axum::response::Json;
use axum::Extension;
use serde::Deserialize;
use std::sync::Arc;

use crate::error::AppError;
use crate::extractors::api_key::ApiKeyContext;
use crate::services::open_api_service;
use crate::AppState;

/// Health check endpoint for the Open API.
/// Returns current version, uptime, and status. No authentication required.
pub async fn health(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_secs": state.started_at.elapsed().as_secs(),
        "status": "ok"
    }))
}

/// Query parameters for listing users via Open API
#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    20
}

/// List users via Open API.
/// Requires API key with `users:read` scope.
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    Extension(api_key): Extension<ApiKeyContext>,
    Query(query): Query<ListUsersQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    api_key.require_scope("users:read")?;

    let (users, total) = open_api_service::list_users(&state.db, query.page, query.per_page).await?;

    Ok(Json(serde_json::json!({
        "items": users,
        "total": total,
        "page": query.page,
        "per_page": query.per_page,
    })))
}

/// Get a single user via Open API.
/// Requires API key with `users:read` scope.
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Extension(api_key): Extension<ApiKeyContext>,
    Path(user_id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    api_key.require_scope("users:read")?;

    let user = open_api_service::get_user(&state.db, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    Ok(Json(serde_json::json!(user)))
}

/// List departments via Open API.
/// Requires API key with `departments:read` scope.
pub async fn list_departments(
    State(state): State<Arc<AppState>>,
    Extension(api_key): Extension<ApiKeyContext>,
) -> Result<Json<serde_json::Value>, AppError> {
    api_key.require_scope("departments:read")?;

    let departments = open_api_service::list_departments(&state.db).await?;

    Ok(Json(serde_json::json!({
        "items": departments,
    })))
}
