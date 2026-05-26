use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::login_log::{CreateLoginLogParams, LoginLogListParams, LoginLogResponse};
use crate::repository::login_log_repo;

/// Record a login log entry (best-effort: errors are logged but not propagated)
pub async fn record_login_log(pool: &PgPool, params: &CreateLoginLogParams) {
    if let Err(e) = login_log_repo::insert_log(pool, params).await {
        tracing::error!("Failed to record login log: {:?}", e);
    }
}

/// List login logs with pagination and filtering
pub async fn list_logs(
    pool: &PgPool,
    auth: &AuthUser,
    params: &LoginLogListParams,
) -> Result<(Vec<LoginLogResponse>, i64), AppError> {
    auth.require_permission(pool, "login-logs:read").await?;
    let (logs, total) = login_log_repo::list_logs_paginated(pool, params).await?;
    let responses: Vec<LoginLogResponse> = logs.into_iter().map(|l| l.into()).collect();
    Ok((responses, total))
}

/// Get a single login log detail
pub async fn get_log_detail(
    pool: &PgPool,
    auth: &AuthUser,
    id: Uuid,
) -> Result<LoginLogResponse, AppError> {
    auth.require_permission(pool, "login-logs:read").await?;
    let log = login_log_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Login log {} not found", id)))?;
    Ok(log.into())
}
