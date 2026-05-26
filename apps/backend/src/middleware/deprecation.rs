use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

/// Middleware that adds `Deprecation: true` and `X-Legacy-Route: true` headers
/// to responses served through legacy (non-versioned) route paths.
pub async fn deprecation_header(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        "Deprecation",
        HeaderValue::from_static("true"),
    );
    response.headers_mut().insert(
        "X-Legacy-Route",
        HeaderValue::from_static("true"),
    );
    response
}
