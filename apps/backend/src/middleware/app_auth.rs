use axum::extract::State;
use axum::middleware::Next;
use axum::response::Response;

use crate::AppState;
use crate::error::AppError;
use crate::extractors::auth::ClientType;

/// App-only auth middleware that validates the JWT token, checks the JWT
/// blacklist, and additionally enforces that `client_type` claim is `"app"`.
/// Returns 403 if a non-App client (e.g. Admin) tries to access an App-only
/// endpoint.
pub async fn require_app_auth_check(
    State(state): State<std::sync::Arc<AppState>>,
    mut request: axum::extract::Request,
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

    // Decode JWT to get claims including client_type
    let decoding_key = jsonwebtoken::DecodingKey::from_secret(state.jwt_secret.as_bytes());
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);

    let token_data = jsonwebtoken::decode::<serde_json::Value>(token, &decoding_key, &validation)
        .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

    // Check client_type claim
    let client_type_str = token_data
        .claims
        .get("client_type")
        .and_then(|v| v.as_str())
        .unwrap_or("admin");

    if client_type_str != "app" {
        return Err(AppError::Forbidden(
            "App client type required".into(),
        ));
    }

    // Extract user info from claims
    let user_id = token_data
        .claims
        .get("sub")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Unauthorized("Invalid token claims".into()))?;

    let user_id = uuid::Uuid::parse_str(user_id)
        .map_err(|_| AppError::Unauthorized("Invalid user id in token".into()))?;

    let email = token_data
        .claims
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let role = token_data
        .claims
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("user")
        .to_string();

    let role_id = token_data
        .claims
        .get("role_id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok());

    let department_id = token_data
        .claims
        .get("department_id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok());

    let jti = token_data
        .claims
        .get("jti")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .unwrap_or(uuid::Uuid::nil());

    // Check JWT blacklist
    if !jti.is_nil() {
        let blacklisted = crate::repository::token_blacklist_repo::is_blacklisted(&state.db, jti)
            .await
            .unwrap_or(false);

        if blacklisted {
            return Err(AppError::Unauthorized("Token has been revoked".into()));
        }

        let should_cleanup = rand::random::<u32>() % 100 == 0;
        if should_cleanup {
            if let Err(e) =
                crate::repository::token_blacklist_repo::cleanup_expired(&state.db).await
            {
                tracing::warn!("Failed to cleanup expired token blacklist entries: {:?}", e);
            }
        }
    }

    let auth_user = crate::extractors::auth::AuthUser {
        user_id,
        email,
        role,
        role_id,
        department_id,
        jti,
        client_type: ClientType::App,
    };

    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}
