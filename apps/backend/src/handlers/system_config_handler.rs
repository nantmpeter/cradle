use axum::extract::{Path, Query, State};
use axum::Json;
use std::sync::Arc;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::system_config::{ConfigGroupQuery, UpsertConfigRequest};
use crate::AppState;

/// GET /api/configs/public — Get public configs (no auth required)
pub async fn get_public(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let configs = crate::services::system_config_service::get_public_configs(&state.db).await?;

    Ok(Json(serde_json::json!({
        "data": configs,
    })))
}

/// GET /api/configs — List all configs (optionally by group)
pub async fn list_all(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<ConfigGroupQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "configs:read").await?;

    let configs = crate::services::system_config_service::list_by_group(
        &state.db,
        query.group_key.as_deref(),
    )
    .await?;

    Ok(Json(serde_json::json!({
        "data": configs,
    })))
}

/// GET /api/configs/{group} — List configs by group
pub async fn list_by_group(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(group): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "configs:read").await?;

    let configs =
        crate::services::system_config_service::list_by_group(&state.db, Some(&group)).await?;

    Ok(Json(serde_json::json!({
        "data": configs,
    })))
}

/// PUT /api/configs/{group}/{key} — Update a config
pub async fn update_config(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((group, key)): Path<(String, String)>,
    Json(req): Json<UpsertConfigRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "configs:update").await?;

    let config = crate::services::system_config_service::upsert(
        &state.db,
        &group,
        &key,
        &req.value,
        &req.value_type,
        req.description.as_deref(),
        auth.user_id,
        &state.config_cache,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "data": config,
        "message": "Config updated successfully",
    })))
}
