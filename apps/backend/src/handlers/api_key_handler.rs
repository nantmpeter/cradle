use axum::extract::{Path, Query, State};
use axum::response::Json;
use axum::Extension;
use serde::Deserialize;
use std::sync::Arc;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::api_key::CreateApiKeyRequest;
use crate::services::api_key_service;
use crate::AppState;

/// Query parameters for listing API keys
#[derive(Debug, Deserialize)]
pub struct ListApiKeysQuery {
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

/// Create a new API key.
/// Requires `api-keys:manage` permission.
pub async fn create_api_key(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<Json<crate::models::api_key::CreateApiKeyResponse>, AppError> {
    // Require permission
    auth_user
        .require_permission(&state.db, "api-keys:manage")
        .await?;

    let response = api_key_service::create(&state.db, auth_user.user_id, &req).await?;
    Ok(Json(response))
}

/// List API keys for the current user.
pub async fn list_api_keys(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListApiKeysQuery>,
) -> Result<Json<crate::models::api_key::ApiKeyListResponse>, AppError> {
    let response =
        api_key_service::list(&state.db, auth_user.user_id, query.page, query.per_page).await?;
    Ok(Json(response))
}

/// Revoke an API key.
/// Requires `api-keys:manage` permission. The key must belong to the current user.
pub async fn revoke_api_key(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(key_id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Require permission
    auth_user
        .require_permission(&state.db, "api-keys:manage")
        .await?;

    // Check that the key belongs to the current user
    let key = crate::repository::api_key_repo::find_by_id(&state.db, key_id)
        .await?
        .ok_or_else(|| AppError::NotFound("API key not found".into()))?;

    if key.user_id != auth_user.user_id {
        return Err(AppError::NotFound("API key not found".into()));
    }

    api_key_service::revoke(&state.db, key_id).await?;

    Ok(Json(serde_json::json!({
        "message": "API key revoked successfully"
    })))
}
