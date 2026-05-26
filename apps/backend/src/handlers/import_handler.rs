use axum::extract::{Multipart, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use std::sync::Arc;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::AppState;

/// POST /api/users/import
pub async fn import_users(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut xlsx_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        crate::error::AppError::BadRequest(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            xlsx_bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| {
                        crate::error::AppError::BadRequest(format!("Failed to read file bytes: {}", e))
                    })?
                    .to_vec(),
            );
            break;
        }
    }

    let bytes = xlsx_bytes.ok_or_else(|| {
        crate::error::AppError::BadRequest("No file uploaded. Please upload an xlsx file with field name 'file'.".to_string())
    })?;

    let result = crate::services::import_service::import_users(&state.db, &auth, &bytes).await?;
    Ok((StatusCode::OK, axum::Json(result)))
}

/// GET /api/users/import/template
pub async fn download_import_template(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    auth.require_permission(&state.db, "users:import").await?;
    let bytes = crate::services::import_service::generate_import_template();
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string()),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"user_import_template.xlsx\"".to_string()),
        ],
        bytes,
    ))
}
