use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::dict::{
    CreateDictItemRequest, CreateDictTypeRequest, DictItemResponse, DictPublicItem,
    DictTypeListParams, DictTypeResponse, UpdateDictItemRequest, UpdateDictTypeRequest,
};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ListDictTypesQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub status: Option<String>,
}

/// GET /api/dict-types
pub async fn list_dict_types(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(q): Query<ListDictTypesQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let params = DictTypeListParams {
        page: q.page.unwrap_or(1).max(1),
        per_page: q.per_page.unwrap_or(20).min(100),
        search: q.search,
        status: q.status,
    };
    let (items, total) = crate::services::dict_service::list_dict_types(&state.db, &auth, &params).await?;
    Ok(Json(serde_json::json!({
        "data": items,
        "total": total,
        "page": params.page,
        "per_page": params.per_page,
    })))
}

/// POST /api/dict-types
pub async fn create_dict_type(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateDictTypeRequest>,
) -> Result<Json<DictTypeResponse>, AppError> {
    let dt = crate::services::dict_service::create_dict_type(&state.db, &auth, &req).await?;
    Ok(Json(dt))
}

/// PUT /api/dict-types/:id
pub async fn update_dict_type(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateDictTypeRequest>,
) -> Result<Json<DictTypeResponse>, AppError> {
    let dt = crate::services::dict_service::update_dict_type(&state.db, &auth, id, &req).await?;
    Ok(Json(dt))
}

/// DELETE /api/dict-types/:id
pub async fn delete_dict_type(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    crate::services::dict_service::delete_dict_type(&state.db, &auth, id).await?;
    Ok(Json(serde_json::json!({"message": "Dict type deleted"})))
}

/// GET /api/dict-types/:id/items
pub async fn list_dict_items(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(dict_type_id): Path<Uuid>,
) -> Result<Json<Vec<DictItemResponse>>, AppError> {
    let items = crate::services::dict_service::list_dict_items(&state.db, &auth, dict_type_id).await?;
    Ok(Json(items))
}

/// POST /api/dict-types/:id/items
pub async fn create_dict_item(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(dict_type_id): Path<Uuid>,
    Json(req): Json<CreateDictItemRequest>,
) -> Result<Json<DictItemResponse>, AppError> {
    let item = crate::services::dict_service::create_dict_item(&state.db, &auth, dict_type_id, &req).await?;
    Ok(Json(item))
}

/// PUT /api/dict-items/:id
pub async fn update_dict_item(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateDictItemRequest>,
) -> Result<Json<DictItemResponse>, AppError> {
    let item = crate::services::dict_service::update_dict_item(&state.db, &auth, id, &req).await?;
    Ok(Json(item))
}

/// DELETE /api/dict-items/:id
pub async fn delete_dict_item(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    crate::services::dict_service::delete_dict_item(&state.db, &auth, id).await?;
    Ok(Json(serde_json::json!({"message": "Dict item deleted"})))
}

/// GET /api/dicts/:code — public lookup (auth required, no permission check)
pub async fn get_dict_by_code(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(code): Path<String>,
) -> Result<Json<Vec<DictPublicItem>>, AppError> {
    let items = crate::services::dict_service::get_dict_by_code(&state.db, &code).await?;
    Ok(Json(items))
}
