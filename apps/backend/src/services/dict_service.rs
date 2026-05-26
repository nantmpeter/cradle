use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::dict::{
    CreateDictItemRequest, CreateDictTypeRequest, DictItemResponse, DictPublicItem,
    DictTypeListParams, DictTypeResponse, UpdateDictItemRequest, UpdateDictTypeRequest,
};
use crate::repository::dict_repo;

/// List dict types with pagination
pub async fn list_dict_types(
    pool: &PgPool,
    auth: &AuthUser,
    params: &DictTypeListParams,
) -> Result<(Vec<DictTypeResponse>, i64), AppError> {
    auth.require_permission(pool, "dicts:read").await?;
    let (items, total) = dict_repo::list_dict_types_paginated(pool, params).await?;
    let responses: Vec<DictTypeResponse> = items.into_iter().map(|dt| dt.into()).collect();
    Ok((responses, total))
}

/// Create a new dict type
pub async fn create_dict_type(
    pool: &PgPool,
    auth: &AuthUser,
    req: &CreateDictTypeRequest,
) -> Result<DictTypeResponse, AppError> {
    auth.require_permission(pool, "dicts:create").await?;

    // Check code uniqueness
    if let Some(existing) = dict_repo::find_dict_type_by_code(pool, &req.code).await? {
        return Err(AppError::Conflict(format!(
            "Dict type with code '{}' already exists",
            existing.code
        )));
    }

    let status = req.status.as_deref().unwrap_or("active");
    let dt = dict_repo::create_dict_type(pool, &req.name, &req.code, status, req.remark.as_deref()).await?;

    // Audit log (best-effort)
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "dict_type.create",
        Some("dict"),
        Some(dt.id),
        Some(serde_json::json!({
            "name": dt.name,
            "code": dt.code,
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for dict type creation: {:?}", e);
    }

    Ok(dt.into())
}

/// Update a dict type
pub async fn update_dict_type(
    pool: &PgPool,
    auth: &AuthUser,
    id: Uuid,
    req: &UpdateDictTypeRequest,
) -> Result<DictTypeResponse, AppError> {
    auth.require_permission(pool, "dicts:update").await?;

    // Check exists
    if dict_repo::find_dict_type_by_id(pool, id).await?.is_none() {
        return Err(AppError::NotFound(format!("Dict type {} not found", id)));
    }

    let dt = dict_repo::update_dict_type(
        pool,
        id,
        req.name.as_deref(),
        req.status.as_deref(),
        req.remark.as_deref(),
    )
    .await?;

    // Audit log (best-effort)
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "dict_type.update",
        Some("dict"),
        Some(id),
        Some(serde_json::json!({
            "name": req.name,
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for dict type update: {:?}", e);
    }

    Ok(dt.into())
}

/// Delete a dict type (cascade deletes items)
pub async fn delete_dict_type(
    pool: &PgPool,
    auth: &AuthUser,
    id: Uuid,
) -> Result<(), AppError> {
    auth.require_permission(pool, "dicts:delete").await?;

    let existing = dict_repo::find_dict_type_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Dict type {} not found", id)))?;

    dict_repo::delete_dict_type(pool, id).await?;

    // Audit log (best-effort)
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "dict_type.delete",
        Some("dict"),
        Some(id),
        Some(serde_json::json!({
            "name": existing.name,
            "code": existing.code,
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for dict type deletion: {:?}", e);
    }

    Ok(())
}

/// List dict items for a given type
pub async fn list_dict_items(
    pool: &PgPool,
    auth: &AuthUser,
    dict_type_id: Uuid,
) -> Result<Vec<DictItemResponse>, AppError> {
    auth.require_permission(pool, "dicts:read").await?;

    // Check type exists
    if dict_repo::find_dict_type_by_id(pool, dict_type_id)
        .await?
        .is_none()
    {
        return Err(AppError::NotFound(format!(
            "Dict type {} not found",
            dict_type_id
        )));
    }

    let items = dict_repo::list_items_by_type(pool, dict_type_id).await?;
    Ok(items.into_iter().map(|di| di.into()).collect())
}

/// Create a new dict item
pub async fn create_dict_item(
    pool: &PgPool,
    auth: &AuthUser,
    dict_type_id: Uuid,
    req: &CreateDictItemRequest,
) -> Result<DictItemResponse, AppError> {
    auth.require_permission(pool, "dicts:create").await?;

    // Check type exists
    if dict_repo::find_dict_type_by_id(pool, dict_type_id)
        .await?
        .is_none()
    {
        return Err(AppError::NotFound(format!(
            "Dict type {} not found",
            dict_type_id
        )));
    }

    let item = dict_repo::create_item(pool, dict_type_id, req).await?;

    // Audit log (best-effort)
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "dict_item.create",
        Some("dict"),
        Some(item.id),
        Some(serde_json::json!({
            "label": item.label,
            "value": item.value,
            "dict_type_id": dict_type_id,
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for dict item creation: {:?}", e);
    }

    Ok(item.into())
}

/// Update a dict item
pub async fn update_dict_item(
    pool: &PgPool,
    auth: &AuthUser,
    id: Uuid,
    req: &UpdateDictItemRequest,
) -> Result<DictItemResponse, AppError> {
    auth.require_permission(pool, "dicts:update").await?;

    // Check exists
    if dict_repo::find_item_by_id(pool, id).await?.is_none() {
        return Err(AppError::NotFound(format!("Dict item {} not found", id)));
    }

    let item = dict_repo::update_item(pool, id, req).await?;

    // Audit log (best-effort)
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "dict_item.update",
        Some("dict"),
        Some(id),
        Some(serde_json::json!({
            "label": req.label,
            "value": req.value,
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for dict item update: {:?}", e);
    }

    Ok(item.into())
}

/// Delete a dict item
pub async fn delete_dict_item(
    pool: &PgPool,
    auth: &AuthUser,
    id: Uuid,
) -> Result<(), AppError> {
    auth.require_permission(pool, "dicts:delete").await?;

    let existing = dict_repo::find_item_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Dict item {} not found", id)))?;

    dict_repo::delete_item(pool, id).await?;

    // Audit log (best-effort)
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(auth.user_id),
        "dict_item.delete",
        Some("dict"),
        Some(id),
        Some(serde_json::json!({
            "label": existing.label,
            "value": existing.value,
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for dict item deletion: {:?}", e);
    }

    Ok(())
}

/// Public dict query by code (auth required, no permission check)
pub async fn get_dict_by_code(
    pool: &PgPool,
    code: &str,
) -> Result<Vec<DictPublicItem>, AppError> {
    dict_repo::find_active_items_by_code(pool, code).await
}
