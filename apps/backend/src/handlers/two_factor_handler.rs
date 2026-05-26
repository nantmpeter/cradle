use axum::extract::{Path, State};
use axum::Json;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::two_factor::{Disable2FaRequest, Enable2FaRequest, Setup2FaResponse};
use crate::AppState;

/// POST /api/auth/2fa/setup — Initialize 2FA setup
pub async fn setup(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Setup2FaResponse>, AppError> {
    let response = crate::services::two_factor_service::setup_2fa(
        &state.db,
        auth.user_id,
        &auth.email,
        &state.totp_encryption_key,
    )
    .await?;

    Ok(Json(response))
}

/// POST /api/auth/2fa/enable — Confirm 2FA with TOTP code
pub async fn enable(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<Enable2FaRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    crate::services::two_factor_service::enable_2fa(
        &state.db,
        auth.user_id,
        &req.code,
        &state.totp_encryption_key,
    )
    .await?;

    Ok(Json(serde_json::json!({"message": "2FA enabled successfully"})))
}

/// POST /api/auth/2fa/disable — Disable 2FA
pub async fn disable(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<Disable2FaRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    crate::services::two_factor_service::disable_2fa(
        &state.db,
        auth.user_id,
        req.password.as_deref(),
        req.code.as_deref(),
        &state.totp_encryption_key,
    )
    .await?;

    Ok(Json(serde_json::json!({"message": "2FA disabled successfully"})))
}

/// POST /api/users/{id}/2fa/reset — Admin reset user's 2FA
pub async fn reset_user_2fa(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "2fa:manage").await?;

    crate::services::two_factor_service::reset_user_2fa(
        &state.db,
        auth.user_id,
        user_id,
    )
    .await?;

    Ok(Json(serde_json::json!({"message": "2FA reset successfully"})))
}
