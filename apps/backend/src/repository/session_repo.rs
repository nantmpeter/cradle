use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::session::{CreateSessionRequest, Session};

pub async fn create(pool: &PgPool, req: &CreateSessionRequest) -> Result<Session, AppError> {
    let session = sqlx::query_as::<_, Session>(
        "INSERT INTO sessions (user_id, refresh_token_hash, ip_address, user_agent, expires_at)
         VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(req.user_id)
    .bind(&req.refresh_token_hash)
    .bind(&req.ip_address)
    .bind(&req.user_agent)
    .bind(req.expires_at)
    .fetch_one(pool)
    .await?;
    Ok(session)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Session>, AppError> {
    let session = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(session)
}

pub async fn list_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<Session>, AppError> {
    let sessions = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE user_id = $1 AND expires_at > NOW() ORDER BY last_active_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(sessions)
}

pub async fn list_all_active(pool: &PgPool) -> Result<Vec<Session>, AppError> {
    let sessions = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE expires_at > NOW() ORDER BY last_active_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(sessions)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Session {} not found", id)));
    }
    Ok(())
}

pub async fn delete_by_user(pool: &PgPool, user_id: Uuid) -> Result<u64, AppError> {
    let result = sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

pub async fn update_last_active(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    sqlx::query("UPDATE sessions SET last_active_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Clean up expired sessions (can be called periodically)
pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, AppError> {
    let result = sqlx::query("DELETE FROM sessions WHERE expires_at < NOW()")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}
