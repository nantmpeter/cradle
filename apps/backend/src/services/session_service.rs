use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::session::{CreateSessionRequest, SessionResponse};
use crate::repository::session_repo;

/// Create a new session record
pub async fn create_session(
    pool: &PgPool,
    user_id: Uuid,
    refresh_token_hash: String,
    ip_address: Option<String>,
    user_agent: Option<String>,
    expires_at: chrono::DateTime<Utc>,
) -> Result<SessionResponse, AppError> {
    let req = CreateSessionRequest {
        user_id,
        refresh_token_hash,
        ip_address,
        user_agent,
        expires_at,
    };
    let session = session_repo::create(pool, &req).await?;
    Ok(SessionResponse::from(session))
}

/// List all active sessions (admin)
pub async fn list_all_sessions(pool: &PgPool) -> Result<Vec<SessionResponse>, AppError> {
    let sessions = session_repo::list_all_active(pool).await?;
    Ok(sessions.into_iter().map(SessionResponse::from).collect())
}

/// List sessions for a specific user
pub async fn list_user_sessions(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<SessionResponse>, AppError> {
    let sessions = session_repo::list_by_user(pool, user_id).await?;
    Ok(sessions.into_iter().map(SessionResponse::from).collect())
}

/// Terminate a session (force logout)
pub async fn terminate_session(
    pool: &PgPool,
    session_id: Uuid,
    requester_is_admin: bool,
    requester_id: Uuid,
) -> Result<(), AppError> {
    let session = session_repo::find_by_id(pool, session_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Session not found".into()))?;

    // Non-admin can only terminate their own sessions
    if !requester_is_admin && session.user_id != requester_id {
        return Err(AppError::Forbidden(
            "Cannot terminate other user's session".into(),
        ));
    }

    session_repo::delete(pool, session_id).await?;
    Ok(())
}

/// Clean up expired sessions
pub async fn cleanup_sessions(pool: &PgPool) -> Result<u64, AppError> {
    session_repo::cleanup_expired(pool).await
}

/// Force logout all sessions for a specific user (admin only)
pub async fn force_logout_user(pool: &PgPool, user_id: Uuid) -> Result<u64, AppError> {
    session_repo::delete_by_user(pool, user_id).await
}
