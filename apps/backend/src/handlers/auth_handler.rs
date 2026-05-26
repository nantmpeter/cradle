use axum::extract::State;
use axum::Json;
use std::sync::Arc;
use validator::Validate;

use crate::error::AppError;
use crate::models::user::{LoginRequest, RefreshRequest, RegisterRequest, TokenResponse, UserResponse};
use crate::models::two_factor::Verify2FaRequest;
use crate::AppState;

/// POST /api/auth/register
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<UserResponse>, AppError> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let user = crate::services::auth_service::register(&state.db, &req).await?;
    Ok(Json(user))
}

/// POST /api/auth/login
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let tokens = crate::services::auth_service::login(
        &state.db,
        &state.jwt_secret,
        state.access_exp_secs,
        state.refresh_exp_secs,
        &req,
    )
    .await?;
    Ok(Json(tokens))
}

/// POST /api/auth/refresh
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let tokens = crate::services::auth_service::refresh(
        &state.db,
        &state.jwt_secret,
        state.access_exp_secs,
        state.refresh_exp_secs,
        &req.refresh_token,
    )
    .await?;
    Ok(Json(tokens))
}

/// POST /api/auth/logout
pub async fn logout(
    State(state): State<Arc<AppState>>,
    auth: crate::extractors::auth::AuthUser,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Decode the access token to get exp for blacklist expiration
    let access_expires_at = chrono::Utc::now() + chrono::Duration::seconds(state.access_exp_secs);

    crate::services::auth_service::logout(
        &state.db,
        &req.refresh_token,
        auth.jti,
        access_expires_at,
    )
    .await?;

    Ok(Json(serde_json::json!({"message": "Logged out"})))
}

/// POST /api/auth/2fa/verify — Verify 2FA during login
pub async fn verify_2fa(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Verify2FaRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let tokens = crate::services::auth_service::verify_2fa_login(
        &state.db,
        &state.jwt_secret,
        state.access_exp_secs,
        state.refresh_exp_secs,
        &state.totp_encryption_key,
        &req.temp_token,
        req.code.as_deref(),
        req.recovery_code.as_deref(),
    )
    .await?;
    Ok(Json(tokens))
}
