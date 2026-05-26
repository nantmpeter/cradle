use axum::extract::{Path, State};
use axum::Json;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::department::{CreateDepartmentRequest, DepartmentResponse, UpdateDepartmentRequest};
use crate::AppState;

/// GET /api/departments
pub async fn list_departments(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let departments = crate::services::department_service::list_departments(&state.db).await?;
    let tree = crate::services::department_service::build_tree(departments.clone());
    Ok(Json(serde_json::json!({
        "departments": departments,
        "tree": tree,
    })))
}

/// POST /api/departments
pub async fn create_department(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateDepartmentRequest>,
) -> Result<Json<DepartmentResponse>, AppError> {
    let dept = crate::services::department_service::create_department(&state.db, &auth, &req).await?;
    Ok(Json(dept))
}

/// PUT /api/departments/:id
pub async fn update_department(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateDepartmentRequest>,
) -> Result<Json<DepartmentResponse>, AppError> {
    let dept = crate::services::department_service::update_department(&state.db, &auth, id, &req).await?;
    Ok(Json(dept))
}

/// DELETE /api/departments/:id
pub async fn delete_department(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    crate::services::department_service::delete_department(&state.db, &auth, id).await?;
    Ok(Json(serde_json::json!({"message": "Department deleted"})))
}
