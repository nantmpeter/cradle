use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::audit_log::{AuditLogListQuery, AuditLogResponse};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>,
}

/// GET /api/audit-logs
pub async fn list_logs(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<AuditLogListQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);

    let (logs, total) = crate::services::audit_service::list_logs(&state.db, &auth, &query).await?;

    Ok(Json(serde_json::json!({
        "data": logs,
        "pagination": {
            "page": page,
            "per_page": per_page,
            "total": total,
        }
    })))
}

/// GET /api/audit-logs/:id
pub async fn get_log_detail(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AuditLogResponse>, AppError> {
    let log = crate::services::audit_service::get_log_detail(&state.db, &auth, id).await?;
    Ok(Json(log))
}

/// GET /api/audit-logs/export — Export audit logs as CSV or XLSX
pub async fn export_logs(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<AuditLogListQuery>,
    Query(export_q): Query<ExportQuery>,
) -> Result<impl IntoResponse, AppError> {
    let fmt = export_q.format.as_deref().unwrap_or("csv");

    let logs = crate::services::audit_service::export_logs(&state.db, &auth, &query).await?;

    match fmt {
        "xlsx" => {
            let bytes = export_audit_xlsx(&logs)?;
            Ok((
                [
                    (
                        header::CONTENT_TYPE,
                        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                            .to_string(),
                    ),
                    (
                        header::CONTENT_DISPOSITION,
                        "attachment; filename=\"audit_logs.xlsx\"".to_string(),
                    ),
                ],
                bytes,
            )
                .into_response())
        }
        _ => {
            let bytes = export_audit_csv(&logs)?;
            Ok((
                [
                    (
                        header::CONTENT_TYPE,
                        "text/csv; charset=utf-8".to_string(),
                    ),
                    (
                        header::CONTENT_DISPOSITION,
                        "attachment; filename=\"audit_logs.csv\"".to_string(),
                    ),
                ],
                bytes,
            )
                .into_response())
        }
    }
}

/// Export audit logs to CSV bytes
fn export_audit_csv(logs: &[AuditLogResponse]) -> Result<Vec<u8>, AppError> {
    let mut wtr = csv::Writer::from_writer(Vec::new());
    wtr.write_record(["id", "user_email", "action", "resource_type", "resource_id", "ip_address", "user_agent", "created_at"])
        .map_err(|e| AppError::Internal(anyhow::anyhow!("CSV write error: {}", e)))?;

    for log in logs {
        wtr.write_record([
            &log.id.to_string(),
            log.user_email.as_deref().unwrap_or(""),
            &log.action,
            log.resource_type.as_deref().unwrap_or(""),
            log.resource_id.map(|id| id.to_string()).as_deref().unwrap_or(""),
            log.ip_address.as_deref().unwrap_or(""),
            log.user_agent.as_deref().unwrap_or(""),
            &log.created_at.to_rfc3339(),
        ])
        .map_err(|e| AppError::Internal(anyhow::anyhow!("CSV write error: {}", e)))?;
    }

    let bytes = wtr
        .into_inner()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("CSV flush error: {}", e)))?;
    Ok(bytes)
}

/// Export audit logs to XLSX bytes
fn export_audit_xlsx(logs: &[AuditLogResponse]) -> Result<Vec<u8>, AppError> {
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let worksheet = workbook.add_worksheet();
    let header_format = rust_xlsxwriter::Format::new().set_bold();

    let headers = ["ID", "User Email", "Action", "Resource Type", "Resource ID", "IP Address", "User Agent", "Created At"];
    for (col, h) in headers.iter().enumerate() {
        worksheet
            .write_with_format(0, col as u16, *h, &header_format)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("XLSX error: {}", e)))?;
    }

    for (i, log) in logs.iter().enumerate() {
        let row = (i + 1) as u32;
        worksheet.write(row, 0, log.id.to_string())
            .map_err(|e| AppError::Internal(anyhow::anyhow!("XLSX error: {}", e)))?;
        worksheet.write(row, 1, log.user_email.as_deref().unwrap_or(""))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("XLSX error: {}", e)))?;
        worksheet.write(row, 2, &log.action)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("XLSX error: {}", e)))?;
        worksheet.write(row, 3, log.resource_type.as_deref().unwrap_or(""))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("XLSX error: {}", e)))?;
        worksheet.write(row, 4, log.resource_id.map(|id| id.to_string()).as_deref().unwrap_or(""))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("XLSX error: {}", e)))?;
        worksheet.write(row, 5, log.ip_address.as_deref().unwrap_or(""))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("XLSX error: {}", e)))?;
        worksheet.write(row, 6, log.user_agent.as_deref().unwrap_or(""))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("XLSX error: {}", e)))?;
        worksheet.write(row, 7, log.created_at.to_rfc3339())
            .map_err(|e| AppError::Internal(anyhow::anyhow!("XLSX error: {}", e)))?;
    }

    let bytes = workbook
        .save_to_buffer()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("XLSX save error: {}", e)))?;
    Ok(bytes.to_vec())
}
