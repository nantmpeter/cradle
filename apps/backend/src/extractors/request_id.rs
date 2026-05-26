use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::error::AppError;
use crate::middleware::request_id::RequestId;

/// Extractor that pulls the request ID from request extensions.
pub struct ExtractRequestId(pub String);

impl FromRequestParts<std::sync::Arc<crate::AppState>> for ExtractRequestId {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &std::sync::Arc<crate::AppState>,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let result = parts
            .extensions
            .get::<RequestId>()
            .map(|rid| ExtractRequestId(rid.0.clone()))
            .ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!(
                    "Request ID not found in extensions"
                ))
            });

        std::future::ready(result)
    }
}
