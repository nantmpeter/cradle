use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::AppState;
use crate::error::AppError;

/// Context injected by the API key authentication middleware.
/// Contains the authenticated API key's identity and authorized scopes.
#[derive(Debug, Clone)]
pub struct ApiKeyContext {
    pub key_id: Uuid,
    pub name: String,
    pub scopes: Vec<String>,
}

impl ApiKeyContext {
    /// Check that the API key has a specific scope.
    /// Returns 403 if the scope is not present.
    pub fn require_scope(&self, scope: &str) -> Result<(), AppError> {
        if self.scopes.contains(&scope.to_string()) || self.scopes.contains(&"*".to_string()) {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "API key requires scope: {}",
                scope
            )))
        }
    }

    /// Check that the API key has all of the specified scopes.
    /// Returns 403 if any scope is missing.
    pub fn require_scopes(&self, scopes: &[&str]) -> Result<(), AppError> {
        // Wildcard scope grants all
        if self.scopes.contains(&"*".to_string()) {
            return Ok(());
        }

        for scope in scopes {
            if !self.scopes.contains(&scope.to_string()) {
                return Err(AppError::Forbidden(format!(
                    "API key requires scope: {}",
                    scope
                )));
            }
        }
        Ok(())
    }
}

impl FromRequestParts<std::sync::Arc<AppState>> for ApiKeyContext {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &std::sync::Arc<AppState>,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let result = parts
            .extensions
            .get::<ApiKeyContext>()
            .cloned()
            .ok_or_else(|| {
                AppError::Unauthorized("API key authentication required".into())
            });

        std::future::ready(result)
    }
}
