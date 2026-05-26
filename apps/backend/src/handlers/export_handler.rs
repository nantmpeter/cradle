use axum::extract::{Query, State};
use axum::http::header;
use axum::response::IntoResponse;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>, // "csv" or "xlsx", default "csv"
    pub search: Option<String>,
    pub role: Option<String>,
    pub role_id: Option<Uuid>,
    pub status: Option<String>,
}

/// GET /api/export/users — Export user list with optional filters
pub async fn export_users(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<ExportQuery>,
) -> Result<impl IntoResponse, AppError> {
    auth.require_permission(&state.db, "export:users").await?;

    let fmt = query.format.as_deref().unwrap_or("csv");

    // Data scope filtering
    let department_ids = auth.get_visible_department_ids(&state.db).await;

    let params = crate::repository::user_repo::ListParams {
        search: query.search,
        role: query.role,
        role_id: query.role_id,
        status: query.status,
        sort_by: "created_at".into(),
        sort_order: "desc".into(),
        page: 1,
        per_page: 10_000,
        department_ids,
    };

    match fmt {
        "xlsx" => {
            let bytes = crate::services::export_service::export_users_xlsx(&state.db, &params).await?;
            Ok((
                [
                    (
                        header::CONTENT_TYPE,
                        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                            .to_string(),
                    ),
                    (
                        header::CONTENT_DISPOSITION,
                        "attachment; filename=\"users.xlsx\"".to_string(),
                    ),
                ],
                bytes,
            )
                .into_response())
        }
        _ => {
            let bytes = crate::services::export_service::export_users_csv(&state.db, &params).await?;
            Ok((
                [
                    (
                        header::CONTENT_TYPE,
                        "text/csv; charset=utf-8".to_string(),
                    ),
                    (
                        header::CONTENT_DISPOSITION,
                        "attachment; filename=\"users.csv\"".to_string(),
                    ),
                ],
                bytes,
            )
                .into_response())
        }
    }
}
