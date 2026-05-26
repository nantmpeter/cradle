use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::login_log::LoginLogListParams;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ListLoginLogsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub email: Option<String>,
    pub event: Option<String>,
    pub ip_address: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// GET /api/login-logs
pub async fn list_login_logs(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(q): Query<ListLoginLogsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let params = LoginLogListParams {
        page: q.page.unwrap_or(1).max(1),
        per_page: q.per_page.unwrap_or(20).min(100),
        email: q.email,
        event: q.event,
        ip_address: q.ip_address,
        start_date: q.start_date,
        end_date: q.end_date,
    };
    let (logs, total) = crate::services::login_log_service::list_logs(&state.db, &auth, &params).await?;
    Ok(Json(serde_json::json!({
        "data": logs,
        "total": total,
        "page": params.page,
        "per_page": params.per_page,
    })))
}

/// GET /api/login-logs/:id
pub async fn get_login_log(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let log = crate::services::login_log_service::get_log_detail(&state.db, &auth, id).await?;
    Ok(Json(serde_json::json!({"data": log})))
}
