use axum::extract::State;
use axum::middleware::Next;
use axum::response::Response;
use sha2::{Digest, Sha256};

use crate::AppState;
use crate::error::AppError;

/// API Key authentication middleware.
/// Extracts the API key from `Authorization: Bearer rk_xxx` or `X-API-Key: rk_xxx`.
/// Validates the key against the database, checks status and expiry.
/// Injects `ApiKeyContext` into request extensions on success.
pub async fn require_api_key_check(
    State(state): State<std::sync::Arc<AppState>>,
    mut request: axum::extract::Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract API key from Authorization header or X-API-Key header
    let api_key = extract_api_key(&request)?;

    // Key must start with "rk_"
    if !api_key.starts_with("rk_") {
        return Err(AppError::Unauthorized(
            "Invalid API key format".into(),
        ));
    }

    // Hash the key for database lookup
    let key_hash = hash_api_key(&api_key);

    // Look up the key in the database
    let api_key_record = crate::repository::api_key_repo::find_by_key_hash(&state.db, &key_hash)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid API key".into()))?;

    // Check status
    if api_key_record.status != "active" {
        return Err(AppError::Unauthorized(
            "API key has been revoked".into(),
        ));
    }

    // Check expiry
    if let Some(expires_at) = api_key_record.expires_at {
        if expires_at < chrono::Utc::now() {
            return Err(AppError::Unauthorized(
                "API key has expired".into(),
            ));
        }
    }

    // Update last_used_at (fire and forget, don't block on failure)
    let db = state.db.clone();
    let key_id = api_key_record.id;
    tokio::spawn(async move {
        let _ = crate::repository::api_key_repo::update_last_used_at(&db, key_id).await;
    });

    // Build ApiKeyContext and inject into extensions
    let scopes = api_key_record.get_scopes();
    let context = crate::extractors::api_key::ApiKeyContext {
        key_id: api_key_record.id,
        name: api_key_record.name,
        scopes,
    };

    request.extensions_mut().insert(context);

    Ok(next.run(request).await)
}

/// Extract API key from request headers.
/// Tries `X-API-Key` first, then falls back to `Authorization: Bearer xxx`.
fn extract_api_key(request: &axum::extract::Request) -> Result<String, AppError> {
    // Try X-API-Key header first
    if let Some(val) = request.headers().get("X-API-Key") {
        let key = val
            .to_str()
            .map_err(|_| AppError::Unauthorized("Invalid X-API-Key header".into()))?;
        return Ok(key.to_string());
    }

    // Fall back to Authorization: Bearer xxx
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing API key or Authorization header".into()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid authorization format".into()))?;

    Ok(token.to_string())
}

/// Hash an API key using SHA-256
fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}
