use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Account disabled: {0}")]
    AccountDisabled(String),

    #[error("Account locked: {0}")]
    AccountLocked(String),

    #[error("Too many requests: {0}")]
    TooManyRequests(String),

    #[error("Two-factor authentication required: {0}")]
    TwoFactorRequired(String),

    #[error("Payload too large: {0}")]
    PayloadTooLarge(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    /// Return the UPPER_SNAKE_CASE error code for this error variant.
    pub fn error_code(&self) -> &str {
        match self {
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Unauthorized(_) => "UNAUTHORIZED",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::Conflict(_) => "CONFLICT",
            AppError::AccountDisabled(_) => "ACCOUNT_DISABLED",
            AppError::AccountLocked(_) => "ACCOUNT_LOCKED",
            AppError::TooManyRequests(_) => "TOO_MANY_REQUESTS",
            AppError::TwoFactorRequired(_) => "TWO_FACTOR_REQUIRED",
            AppError::PayloadTooLarge(_) => "PAYLOAD_TOO_LARGE",
            AppError::Database(_) => "INTERNAL_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message, code) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone(), "NOT_FOUND"),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone(), "UNAUTHORIZED"),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone(), "FORBIDDEN"),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone(), "BAD_REQUEST"),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone(), "CONFLICT"),
            AppError::AccountDisabled(msg) => {
                (StatusCode::FORBIDDEN, msg.clone(), "ACCOUNT_DISABLED")
            }
            AppError::AccountLocked(msg) => {
                (StatusCode::FORBIDDEN, msg.clone(), "ACCOUNT_LOCKED")
            }
            AppError::TooManyRequests(msg) => {
                (StatusCode::TOO_MANY_REQUESTS, msg.clone(), "TOO_MANY_REQUESTS")
            }
            AppError::TwoFactorRequired(msg) => {
                (StatusCode::UNAUTHORIZED, msg.clone(), "TWO_FACTOR_REQUIRED")
            }
            AppError::PayloadTooLarge(msg) => {
                (StatusCode::PAYLOAD_TOO_LARGE, msg.clone(), "PAYLOAD_TOO_LARGE")
            }
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".into(),
                    "INTERNAL_ERROR",
                )
            }
            AppError::Internal(e) => {
                tracing::error!("Internal error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".into(),
                    "INTERNAL_ERROR",
                )
            }
        };

        let body = Json(json!({
            "error": {
                "code": code,
                "message": message,
                "request_id": null,
                "details": null
            },
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}
