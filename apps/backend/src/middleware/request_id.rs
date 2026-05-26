use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

/// Request ID extracted from the X-Request-Id header or generated as a new UUID.
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

/// Middleware layer that ensures every request has a unique X-Request-Id.
/// If the client provides one via the `X-Request-Id` header, it is preserved.
/// Otherwise a new UUID v4 is generated. The ID is stored in request extensions
/// and echoed back in the response header.
pub async fn request_id_layer(request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("X-Request-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let mut request = request;
    request.extensions_mut().insert(RequestId(request_id.clone()));

    let mut response = next.run(request).await;

    if let Ok(val) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("X-Request-Id", val);
    }

    response
}
