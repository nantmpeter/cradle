use axum::extract::{Path, State};
use axum::Json;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::permission::PermissionResponse;
use crate::models::role::{CreateRoleRequest, RoleResponse, UpdateRolePermissionsRequest, UpdateRoleRequest};
use crate::AppState;

/// GET /api/roles
pub async fn list_roles(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<RoleResponse>>, AppError> {
    auth.require_permission(&state.db, "roles:read").await?;
    let roles = crate::services::role_service::list_roles(&state.db).await?;
    Ok(Json(roles))
}

/// POST /api/roles
pub async fn create_role(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateRoleRequest>,
) -> Result<Json<RoleResponse>, AppError> {
    let role = crate::services::role_service::create_role(&state.db, &auth, &req).await?;
    Ok(Json(role))
}

/// GET /api/roles/:id
pub async fn get_role(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<RoleResponse>, AppError> {
    auth.require_permission(&state.db, "roles:read").await?;
    let role = crate::services::role_service::get_role(&state.db, id).await?;
    Ok(Json(role))
}

/// PUT /api/roles/:id
pub async fn update_role(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<Json<RoleResponse>, AppError> {
    let role = crate::services::role_service::update_role(&state.db, &auth, id, &req).await?;
    Ok(Json(role))
}

/// DELETE /api/roles/:id
pub async fn delete_role(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    crate::services::role_service::delete_role(&state.db, &auth, id).await?;
    Ok(Json(serde_json::json!({"message": "Role deleted"})))
}

/// PUT /api/roles/:id/permissions
pub async fn update_permissions(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRolePermissionsRequest>,
) -> Result<Json<RoleResponse>, AppError> {
    let role = crate::services::role_service::update_permissions(&state.db, &auth, id, &req).await?;
    Ok(Json(role))
}

/// GET /api/permissions
pub async fn list_permissions(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<PermissionResponse>>, AppError> {
    auth.require_permission(&state.db, "roles:read").await?;
    let permissions = crate::repository::permission_repo::list_all(&state.db).await?;
    let responses: Vec<PermissionResponse> = permissions
        .into_iter()
        .map(|p| PermissionResponse {
            id: p.id,
            name: p.name,
            description: p.description,
            module: p.module,
            created_at: p.created_at,
        })
        .collect();
    Ok(Json(responses))
}
