use axum::extract::State;
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::Deserialize;

use crate::AppState;
use crate::error::AppError;
use crate::extractors::auth::{AuthUser, ClientType};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Claims {
    sub: String,
    email: String,
    role: String,
    role_id: Option<String>,
    department_id: Option<String>,
    jti: Option<String>,
    exp: i64,
    client_type: Option<String>,
}

/// JWT authentication middleware with blacklist check
pub async fn require_auth(
    State(state): State<std::sync::Arc<AppState>>,
    request: axum::extract::Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".into()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid authorization format".into()))?;

    let decoding_key = DecodingKey::from_secret(state.jwt_secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

    let user_id = uuid::Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user id in token".into()))?;

    // Parse jti from claims
    let jti = match &token_data.claims.jti {
        Some(jti_str) => uuid::Uuid::parse_str(jti_str)
            .map_err(|_| AppError::Unauthorized("Invalid jti in token".into()))?,
        None => uuid::Uuid::nil(), // Legacy tokens without jti
    };

    // Parse role_id from claims
    let role_id = match &token_data.claims.role_id {
        Some(rid_str) => uuid::Uuid::parse_str(rid_str).ok(),
        None => None,
    };

    // Parse department_id from claims
    let department_id = match &token_data.claims.department_id {
        Some(did_str) => uuid::Uuid::parse_str(did_str).ok(),
        None => None,
    };

    // Parse client_type from claims, default to Admin
    let client_type = match token_data
        .claims
        .client_type
        .as_deref()
        .unwrap_or("admin")
    {
        "app" => ClientType::App,
        _ => ClientType::Admin,
    };

    // Check JWT blacklist
    if !jti.is_nil() {
        let blacklisted = crate::repository::token_blacklist_repo::is_blacklisted(&state.db, jti)
            .await
            .unwrap_or(false);

        if blacklisted {
            return Err(AppError::Unauthorized("Token has been revoked".into()));
        }

        // 1% probability cleanup of expired blacklist entries
        let should_cleanup = rand::random::<u32>() % 100 == 0;
        if should_cleanup {
            if let Err(e) = crate::repository::token_blacklist_repo::cleanup_expired(&state.db).await {
                tracing::warn!("Failed to cleanup expired token blacklist entries: {:?}", e);
            }
        }
    }

    let auth_user = AuthUser {
        user_id,
        email: token_data.claims.email,
        role: token_data.claims.role,
        role_id,
        department_id,
        jti,
        client_type,
    };

    let mut request = request;
    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}
