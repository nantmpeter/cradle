use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::audit_log::{AuditLogListQuery, AuditLogResponse};
use crate::repository::audit_log_repo;

/// Record an audit log entry. Does not block on errors (logs error and continues).
pub async fn log(
    pool: &PgPool,
    user_id: Option<Uuid>,
    action: &str,
    resource_type: Option<&str>,
    resource_id: Option<Uuid>,
    details: Option<serde_json::Value>,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) {
    if let Err(e) = audit_log_repo::create(
        pool,
        user_id,
        action,
        resource_type,
        resource_id,
        details,
        ip_address,
        user_agent,
    )
    .await
    {
        tracing::error!("Failed to write audit log: {:?}", e);
    }
}

/// List audit logs with pagination and filtering. Requires audit:read permission.
pub async fn list_logs(
    pool: &PgPool,
    auth: &AuthUser,
    query: &AuditLogListQuery,
) -> Result<(Vec<AuditLogResponse>, i64), AppError> {
    // Permission check
    auth.require_permission(pool, "audit:read").await?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);

    // Validate date filters
    if let Some(ref from) = query.from {
        if from.parse::<chrono::NaiveDateTime>().is_err()
            && from.parse::<chrono::DateTime<chrono::Utc>>().is_err()
        {
            return Err(AppError::BadRequest(
                "Invalid 'from' date format. Expected ISO 8601.".into(),
            ));
        }
    }
    if let Some(ref to) = query.to {
        if to.parse::<chrono::NaiveDateTime>().is_err()
            && to.parse::<chrono::DateTime<chrono::Utc>>().is_err()
        {
            return Err(AppError::BadRequest(
                "Invalid 'to' date format. Expected ISO 8601.".into(),
            ));
        }
    }

    let params = audit_log_repo::AuditLogQueryParams {
        page,
        per_page,
        user_id: query.user_id,
        action: query.action.clone(),
        from: query.from.clone(),
        to: query.to.clone(),
        resource_type: query.resource_type.clone(),
        resource_id: query.resource_id,
        ip_address: query.ip_address.clone(),
    };

    audit_log_repo::list_paginated(pool, &params).await
}

/// Get a single audit log detail. Requires audit:read permission.
pub async fn get_log_detail(
    pool: &PgPool,
    auth: &AuthUser,
    id: Uuid,
) -> Result<AuditLogResponse, AppError> {
    // Permission check
    auth.require_permission(pool, "audit:read").await?;

    audit_log_repo::find_response_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Audit log {} not found", id)))
}

/// Export audit logs with filtering (no pagination). Requires audit:read permission.
pub async fn export_logs(
    pool: &PgPool,
    auth: &AuthUser,
    query: &AuditLogListQuery,
) -> Result<Vec<AuditLogResponse>, AppError> {
    // Permission check
    auth.require_permission(pool, "audit:read").await?;

    // Validate date filters
    if let Some(ref from) = query.from {
        if from.parse::<chrono::NaiveDateTime>().is_err()
            && from.parse::<chrono::DateTime<chrono::Utc>>().is_err()
        {
            return Err(AppError::BadRequest(
                "Invalid 'from' date format. Expected ISO 8601.".into(),
            ));
        }
    }
    if let Some(ref to) = query.to {
        if to.parse::<chrono::NaiveDateTime>().is_err()
            && to.parse::<chrono::DateTime<chrono::Utc>>().is_err()
        {
            return Err(AppError::BadRequest(
                "Invalid 'to' date format. Expected ISO 8601.".into(),
            ));
        }
    }

    let params = audit_log_repo::AuditLogQueryParams {
        page: 1,
        per_page: i64::MAX,
        user_id: query.user_id,
        action: query.action.clone(),
        from: query.from.clone(),
        to: query.to.clone(),
        resource_type: query.resource_type.clone(),
        resource_id: query.resource_id,
        ip_address: query.ip_address.clone(),
    };

    audit_log_repo::export_filtered(pool, &params).await
}
