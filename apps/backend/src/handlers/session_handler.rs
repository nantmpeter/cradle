use axum::extract::{Path, State};
use axum::Json;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::AppState;

/// GET /api/sessions — List all active sessions (admin)
pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "sessions:read").await?;

    let sessions = crate::services::session_service::list_all_sessions(&state.db).await?;

    Ok(Json(serde_json::json!({ "data": sessions })))
}

/// GET /api/sessions/me — List my sessions
pub async fn list_my_sessions(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let sessions = crate::services::session_service::list_user_sessions(
        &state.db,
        auth.user_id,
    )
    .await?;

    Ok(Json(serde_json::json!({ "data": sessions })))
}

/// DELETE /api/sessions/:id — Terminate a session
pub async fn terminate_session(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "sessions:manage").await?;

    crate::services::session_service::terminate_session(
        &state.db,
        id,
        auth.is_admin(),
        auth.user_id,
    )
    .await?;

    Ok(Json(serde_json::json!({ "message": "Session terminated" })))
}

/// DELETE /api/sessions/user/:user_id — Force logout all sessions for a user (admin)
pub async fn force_logout_user(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth.require_permission(&state.db, "sessions:manage").await?;

    let count = crate::services::session_service::force_logout_user(&state.db, user_id).await?;

    Ok(Json(serde_json::json!({
        "message": format!("Terminated {} session(s) for user {}", count, user_id)
    })))
}
