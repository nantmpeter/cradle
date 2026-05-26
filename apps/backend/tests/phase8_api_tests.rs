//! Phase 8 Integration Tests: API Infrastructure
//!
//! Covers:
//! - 8A: Request ID middleware, route versioning, deprecation header, error format
//! - 8B: API Key CRUD, API Key auth, Open API, scope enforcement
//! - 8C: App auth middleware (client_type check)
//! - 8D: Webhook CRUD, delivery logs

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tower::ServiceExt;

use cradle_backend::config::Settings;

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Build a test app that includes all Phase 8 routes.
/// This mirrors `lib.rs::app()` but without rate limiting (to avoid flaky tests)
/// and with the full v1 + legacy route tree.
async fn create_test_app() -> (axum::Router, PgPool) {
    dotenvy::dotenv().ok();
    let settings = Settings::new().expect("Failed to load settings");
    let db = sqlx::PgPool::connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Migration failed");

    // Reset admin account lockout and 2FA to prevent cross-test interference
    sqlx::query(
        "UPDATE users SET locked_until = NULL, login_failures = 0, two_factor_enabled = false, totp_secret = NULL WHERE email = 'admin@example.com'",
    )
    .execute(&db)
    .await
    .ok();

    let jwt_secret = settings.jwt.secret.clone();
    let access_exp_secs = settings.jwt.access_exp_secs;
    let refresh_exp_secs = settings.jwt.refresh_exp_secs;
    let max_connections = settings.database.max_connections;

    // Notification channel
    let (notification_tx, _) = tokio::sync::broadcast::channel(256);

    // Webhook channel
    let (webhook_tx, webhook_rx) =
        tokio::sync::mpsc::channel::<cradle_backend::models::webhook::WebhookEvent>(256);

    let http_client = reqwest::Client::new();

    let state = Arc::new(cradle_backend::AppState {
        db: db.clone(),
        jwt_secret,
        access_exp_secs,
        refresh_exp_secs,
        started_at: std::time::Instant::now(),
        max_connections,
        settings: settings.clone(),
        totp_encryption_key: settings.totp.encryption_key.clone(),
        config_cache: Arc::new(RwLock::new(HashMap::new())),
        notification_tx,
        http_client,
        webhook_tx,
    });

    // Start webhook workers (consume webhook_rx)
    {
        let pool = state.db.clone();
        let client = reqwest::Client::new();
        let wh_settings = settings.webhook.clone();
        cradle_backend::services::webhook_service::start_event_worker(
            pool,
            client,
            wh_settings,
            webhook_rx,
        );
    }
    {
        let pool = state.db.clone();
        let client = reqwest::Client::new();
        let wh_settings = settings.webhook.clone();
        cradle_backend::services::webhook_service::start_retry_worker(pool, client, wh_settings);
    }

    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let auth_mw = axum::middleware::from_fn_with_state(
        state.clone(),
        cradle_backend::middleware::auth::require_auth,
    );

    let api_key_mw = axum::middleware::from_fn_with_state(
        state.clone(),
        cradle_backend::middleware::api_key_auth::require_api_key_check,
    );

    let deprecation_mw = axum::middleware::from_fn(
        cradle_backend::middleware::deprecation::deprecation_header,
    );

    // NOTE: No rate limiting layers (GovernorLayer) to avoid flaky test failures
    let app = axum::Router::new()
        // Auth routes (public, not versioned)
        .merge(cradle_backend::routes::auth_routes())
        // App auth routes (public)
        .merge(cradle_backend::routes::app_auth_routes())
        // New v1 Admin API
        .nest(
            "/api/v1/admin",
            axum::Router::new()
                .merge(cradle_backend::routes::auth_protected_routes())
                .merge(cradle_backend::routes::user_routes())
                .merge(cradle_backend::routes::role_routes())
                .merge(cradle_backend::routes::audit_routes())
                .merge(cradle_backend::routes::dashboard_routes())
                .merge(cradle_backend::routes::file_routes())
                .merge(cradle_backend::routes::export_routes())
                .merge(cradle_backend::routes::session_routes())
                .merge(cradle_backend::routes::menu_routes())
                .merge(cradle_backend::routes::notification_routes())
                .merge(cradle_backend::routes::config_routes())
                .merge(cradle_backend::routes::department_routes())
                .merge(cradle_backend::routes::dict_routes())
                .merge(cradle_backend::routes::dict_item_routes())
                .merge(cradle_backend::routes::login_log_routes())
                .merge(cradle_backend::routes::import_routes())
                .merge(cradle_backend::routes::search_routes())
                .merge(cradle_backend::routes::dict_public_routes())
                .merge(cradle_backend::routes::api_key_routes())
                .merge(cradle_backend::routes::webhook_routes())
                .layer(auth_mw.clone()),
        )
        // App API (v1)
        .nest(
            "/api/v1/app",
            cradle_backend::routes::app_routes().layer(axum::middleware::from_fn_with_state(
                state.clone(),
                cradle_backend::middleware::app_auth::require_app_auth_check,
            )),
        )
        // Open API (v1) — public
        .nest(
            "/api/v1/open",
            cradle_backend::routes::open_public_routes(),
        )
        // Open API (v1) — protected (API key)
        .nest(
            "/api/v1/open",
            cradle_backend::routes::open_protected_routes().layer(api_key_mw),
        )
        // Legacy routes (with deprecation headers)
        .nest(
            "/api",
            axum::Router::new()
                .merge(cradle_backend::routes::auth_protected_routes())
                .merge(cradle_backend::routes::user_routes())
                .merge(cradle_backend::routes::department_routes())
                .merge(cradle_backend::routes::dict_routes())
                .merge(cradle_backend::routes::dict_item_routes())
                .merge(cradle_backend::routes::dict_public_routes())
                .merge(cradle_backend::routes::login_log_routes())
                .merge(cradle_backend::routes::import_routes())
                .merge(cradle_backend::routes::search_routes())
                .layer(auth_mw.clone())
                .layer(deprecation_mw),
        )
        .merge(cradle_backend::routes::i18n_routes())
        .merge(cradle_backend::routes::config_public_routes())
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(cors)
        .layer(axum::middleware::from_fn(
            cradle_backend::middleware::request_id::request_id_layer,
        ));

    (app, db)
}

/// Send a request and get (status, response_headers, body_json)
async fn send_request_full(
    app: &mut axum::Router,
    req: Request<Body>,
) -> (StatusCode, axum::http::HeaderMap, Value) {
    let response = app.oneshot(req).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));
    (status, headers, value)
}

/// Send a request and get (status, body_json) — convenience wrapper
async fn send_request(app: &mut axum::Router, req: Request<Body>) -> (StatusCode, Value) {
    let (status, _, value) = send_request_full(app, req).await;
    (status, value)
}

/// Login as admin and return the access token
async fn admin_token(app: &mut axum::Router) -> String {
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({"email": "admin@example.com", "password": "Admin@1234"}).to_string(),
        ))
        .unwrap();
    let (_, body) = send_request(app, req).await;
    body["access_token"].as_str().unwrap().to_string()
}

/// Register a user and return the access token
async fn register_and_login(
    app: &mut axum::Router,
    email: &str,
    password: &str,
    name: &str,
) -> String {
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({"email": email, "password": password, "name": name}).to_string(),
        ))
        .unwrap();
    let _ = send_request(app, req).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("Content-Type", "application/json")
        .body(Body::from(json!({"email": email, "password": password}).to_string()))
        .unwrap();
    let (_, body) = send_request(app, req).await;
    body["access_token"].as_str().unwrap().to_string()
}

/// Build an authenticated request
fn auth_request(method: &str, uri: &str, token: &str, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("Authorization", format!("Bearer {}", token));

    if body.is_some() {
        builder = builder.header("Content-Type", "application/json");
    }

    let body = body
        .map(|b| Body::from(b.to_string()))
        .unwrap_or(Body::empty());
    builder.body(body).unwrap()
}

/// Build an API-key-authenticated request
fn api_key_request(method: &str, uri: &str, api_key: &str, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("X-API-Key", api_key);

    if body.is_some() {
        builder = builder.header("Content-Type", "application/json");
    }

    let body = body
        .map(|b| Body::from(b.to_string()))
        .unwrap_or(Body::empty());
    builder.body(body).unwrap()
}

fn unique_email(prefix: &str) -> String {
    format!(
        "{}_{}@example.com",
        prefix,
        chrono::Utc::now().timestamp_millis()
    )
}

async fn cleanup_user(db: &PgPool, email: &str) {
    sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(email)
        .execute(db)
        .await
        .ok();
}

async fn cleanup_webhooks(db: &PgPool, name: &str) {
    sqlx::query("DELETE FROM webhook_deliveries WHERE webhook_id IN (SELECT id FROM webhooks WHERE name LIKE $1)")
        .bind(format!("{}%", name))
        .execute(db)
        .await
        .ok();
    sqlx::query("DELETE FROM webhooks WHERE name LIKE $1")
        .bind(format!("{}%", name))
        .execute(db)
        .await
        .ok();
}

// ─── 8A: Request ID Middleware Tests ──────────────────────────────────────────

#[tokio::test]
async fn test_request_id_auto_generated() {
    let (mut app, _) = create_test_app().await;

    // Open API health endpoint requires no auth — good for testing request ID
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/open/health")
        .body(Body::empty())
        .unwrap();

    let (status, headers, _body) = send_request_full(&mut app, req).await;
    assert_eq!(status, StatusCode::OK);

    // X-Request-Id should be present
    let request_id = headers
        .get("X-Request-Id")
        .expect("X-Request-Id header should be present")
        .to_str()
        .unwrap();

    // Should be a UUID v4 format
    assert!(
        uuid::Uuid::parse_str(request_id).is_ok(),
        "Auto-generated request ID should be a valid UUID, got: {}",
        request_id
    );
}

#[tokio::test]
async fn test_request_id_passthrough() {
    let (mut app, _) = create_test_app().await;

    let custom_id = "my-custom-request-id-12345";
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/open/health")
        .header("X-Request-Id", custom_id)
        .body(Body::empty())
        .unwrap();

    let (status, headers, _body) = send_request_full(&mut app, req).await;
    assert_eq!(status, StatusCode::OK);

    let request_id = headers
        .get("X-Request-Id")
        .expect("X-Request-Id header should be present")
        .to_str()
        .unwrap();

    assert_eq!(
        request_id, custom_id,
        "Custom request ID should be passed through"
    );
}

// ─── 8A: Route Versioning Tests ───────────────────────────────────────────────

#[tokio::test]
async fn test_v1_admin_route_accessible() {
    let (mut app, _) = create_test_app().await;
    let token = admin_token(&mut app).await;

    // v1 route should work
    let req = auth_request("GET", "/api/v1/admin/users/me", &token, None);
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK, "v1 route failed: {:?}", body);
    assert_eq!(body["email"], "admin@example.com");
}

#[tokio::test]
async fn test_legacy_route_has_deprecation_header() {
    let (mut app, _) = create_test_app().await;
    let token = admin_token(&mut app).await;

    // Legacy /api route should return Deprecation header
    let req = auth_request("GET", "/api/users/me", &token, None);
    let (status, headers, body) = send_request_full(&mut app, req).await;
    assert_eq!(status, StatusCode::OK, "Legacy route failed: {:?}", body);

    let deprecation = headers
        .get("Deprecation")
        .expect("Deprecation header should be present on legacy routes")
        .to_str()
        .unwrap();
    assert_eq!(deprecation, "true");

    let legacy = headers
        .get("X-Legacy-Route")
        .expect("X-Legacy-Route header should be present")
        .to_str()
        .unwrap();
    assert_eq!(legacy, "true");
}

#[tokio::test]
async fn test_v1_route_no_deprecation_header() {
    let (mut app, _) = create_test_app().await;
    let token = admin_token(&mut app).await;

    // v1 route should NOT have Deprecation header
    let req = auth_request("GET", "/api/v1/admin/users/me", &token, None);
    let (status, headers, _body) = send_request_full(&mut app, req).await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        headers.get("Deprecation").is_none(),
        "v1 routes should not have Deprecation header"
    );
}

#[tokio::test]
async fn test_open_health_no_auth_required() {
    let (mut app, _) = create_test_app().await;

    // Health endpoint should be accessible without any authentication
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/open/health")
        .body(Body::empty())
        .unwrap();

    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK, "Open health failed: {:?}", body);
    assert_eq!(body["status"], "ok");
    assert!(body["version"].is_string());
}

#[tokio::test]
async fn test_open_protected_requires_api_key() {
    let (mut app, _) = create_test_app().await;

    // Accessing protected Open API without API key should return 401
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/open/users")
        .body(Body::empty())
        .unwrap();

    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "Expected 401 without API key, got: {:?}", body);
}

// ─── 8A: Error Format Tests ───────────────────────────────────────────────────

#[tokio::test]
async fn test_401_error_format() {
    let (mut app, _) = create_test_app().await;

    // Access protected route without auth → 401
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/admin/users/me")
        .body(Body::empty())
        .unwrap();

    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Verify unified error format: { error: { code, message, ... }, status }
    assert!(body["error"].is_object(), "Response should have 'error' object, got: {:?}", body);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
    assert!(body["error"]["message"].is_string());
    assert!(body["status"].is_number());
    assert_eq!(body["status"], 401);
}

#[tokio::test]
async fn test_404_error_format() {
    let (mut app, _) = create_test_app().await;
    let token = admin_token(&mut app).await;

    // Access non-existent user → 404
    let fake_id = uuid::Uuid::new_v4();
    let req = auth_request("GET", &format!("/api/v1/admin/users/{}", fake_id), &token, None);
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    assert!(body["error"].is_object(), "Response should have 'error' object, got: {:?}", body);
    assert_eq!(body["error"]["code"], "NOT_FOUND");
    assert!(body["error"]["message"].is_string());
    assert_eq!(body["status"], 404);
}

#[tokio::test]
async fn test_403_error_format() {
    let (mut app, db) = create_test_app().await;
    let email = unique_email("forbidden");

    // Create a normal user who does NOT have api-keys:manage permission
    let token = register_and_login(&mut app, &email, "TestPass@123", "Normal User").await;

    // Try to create an API key → should be 403
    let req = auth_request(
        "POST",
        "/api/v1/admin/api-keys",
        &token,
        Some(json!({
            "name": "Test Key",
            "scopes": ["users:read"]
        })),
    );
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "Expected 403, got: {:?}", body);

    assert!(body["error"].is_object(), "Response should have 'error' object");
    assert_eq!(body["error"]["code"], "FORBIDDEN");
    assert_eq!(body["status"], 403);

    cleanup_user(&db, &email).await;
}

// ─── 8B: API Key CRUD Tests ───────────────────────────────────────────────────

#[tokio::test]
async fn test_create_api_key_success() {
    let (mut app, _db) = create_test_app().await;
    let token = admin_token(&mut app).await;

    let req = auth_request(
        "POST",
        "/api/v1/admin/api-keys",
        &token,
        Some(json!({
            "name": "Test API Key",
            "scopes": ["users:read", "departments:read"]
        })),
    );
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK, "Create API key failed: {:?}", body);

    // Should return the plaintext key (shown only once)
    assert!(body["key"].is_string(), "Response should include plaintext key");
    assert!(body["key"].as_str().unwrap().starts_with("rk_"), "Key should start with rk_");
    assert_eq!(body["name"], "Test API Key");
    assert!(body["id"].is_string());

    // Clean up
    let key_id = body["id"].as_str().unwrap();
    let req = auth_request("DELETE", &format!("/api/v1/admin/api-keys/{}", key_id), &token, None);
    let _ = send_request(&mut app, req).await;
}

#[tokio::test]
async fn test_create_api_key_no_permission() {
    let (mut app, db) = create_test_app().await;
    let email = unique_email("no_perm");

    // Normal user should not be able to create API keys
    let token = register_and_login(&mut app, &email, "TestPass@123", "No Perm User").await;

    let req = auth_request(
        "POST",
        "/api/v1/admin/api-keys",
        &token,
        Some(json!({
            "name": "Forbidden Key",
            "scopes": ["users:read"]
        })),
    );
    let (status, _body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "Normal user should not be able to create API keys");

    cleanup_user(&db, &email).await;
}

#[tokio::test]
async fn test_list_api_keys() {
    let (mut app, _db) = create_test_app().await;
    let token = admin_token(&mut app).await;

    // Create an API key first
    let req = auth_request(
        "POST",
        "/api/v1/admin/api-keys",
        &token,
        Some(json!({
            "name": "List Test Key",
            "scopes": ["users:read"]
        })),
    );
    let (status, create_body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK);
    let key_id = create_body["id"].as_str().unwrap();

    // List API keys
    let req = auth_request("GET", "/api/v1/admin/api-keys", &token, None);
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK, "List API keys failed: {:?}", body);
    assert!(body["items"].is_array());
    assert!(body["total"].is_number());
    assert!(body["total"].as_i64().unwrap() >= 1);

    // Verify the created key is in the list
    let items = body["items"].as_array().unwrap();
    assert!(items.iter().any(|k| k["id"] == key_id), "Created key should appear in list");

    // Clean up
    let req = auth_request("DELETE", &format!("/api/v1/admin/api-keys/{}", key_id), &token, None);
    let _ = send_request(&mut app, req).await;
}

#[tokio::test]
async fn test_api_key_auth_and_scope_check() {
    let (mut app, _db) = create_test_app().await;
    let token = admin_token(&mut app).await;

    // Create API key with users:read scope
    let req = auth_request(
        "POST",
        "/api/v1/admin/api-keys",
        &token,
        Some(json!({
            "name": "Scope Test Key",
            "scopes": ["users:read"]
        })),
    );
    let (status, create_body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK);
    let api_key = create_body["key"].as_str().unwrap().to_string();
    let key_id = create_body["id"].as_str().unwrap();

    // Use API key to access Open API /users (should succeed with users:read scope)
    let req = api_key_request("GET", "/api/v1/open/users", &api_key, None);
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK, "API key should access /open/users: {:?}", body);
    assert!(body["items"].is_array());

    // Use API key to access /open/departments (should FAIL — no departments:read scope)
    let req = api_key_request("GET", "/api/v1/open/departments", &api_key, None);
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(
        status, StatusCode::FORBIDDEN,
        "API key without departments:read scope should be forbidden: {:?}",
        body
    );

    // Clean up
    let req = auth_request("DELETE", &format!("/api/v1/admin/api-keys/{}", key_id), &token, None);
    let _ = send_request(&mut app, req).await;
}

#[tokio::test]
async fn test_revoke_api_key_then_use_fails() {
    let (mut app, _db) = create_test_app().await;
    let token = admin_token(&mut app).await;

    // Create API key
    let req = auth_request(
        "POST",
        "/api/v1/admin/api-keys",
        &token,
        Some(json!({
            "name": "Revoke Test Key",
            "scopes": ["users:read"]
        })),
    );
    let (_, create_body) = send_request(&mut app, req).await;
    let api_key = create_body["key"].as_str().unwrap().to_string();
    let key_id = create_body["id"].as_str().unwrap();

    // Verify it works before revocation
    let req = api_key_request("GET", "/api/v1/open/users", &api_key, None);
    let (status, _) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK, "API key should work before revocation");

    // Revoke the key
    let req = auth_request("DELETE", &format!("/api/v1/admin/api-keys/{}", key_id), &token, None);
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK, "Revoke failed: {:?}", body);
    assert!(body["message"].as_str().unwrap().contains("revoked"));

    // Use revoked key → should be 401
    let req = api_key_request("GET", "/api/v1/open/users", &api_key, None);
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "Revoked key should return 401: {:?}", body);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
}

#[tokio::test]
async fn test_api_key_expired_returns_401() {
    let (mut app, db) = create_test_app().await;
    let token = admin_token(&mut app).await;

    // Create API key with expiry in the past
    let past_time = chrono::Utc::now() - chrono::Duration::hours(1);
    let req = auth_request(
        "POST",
        "/api/v1/admin/api-keys",
        &token,
        Some(json!({
            "name": "Expired Key",
            "scopes": ["users:read"],
            "expires_at": past_time.to_rfc3339()
        })),
    );
    let (_, create_body) = send_request(&mut app, req).await;
    let api_key = create_body["key"].as_str().unwrap().to_string();
    let key_id = create_body["id"].as_str().unwrap();

    // Use expired key → should be 401
    let req = api_key_request("GET", "/api/v1/open/users", &api_key, None);
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "Expired key should return 401: {:?}", body);

    // Clean up
    let key_uuid: uuid::Uuid = key_id.parse().unwrap();
    sqlx::query("DELETE FROM api_keys WHERE id = $1")
        .bind(key_uuid)
        .execute(&db)
        .await
        .ok();
}

#[tokio::test]
async fn test_invalid_api_key_format() {
    let (mut app, _) = create_test_app().await;

    // API key that doesn't start with rk_ should fail
    let req = api_key_request("GET", "/api/v1/open/users", "invalid_key_format", None);
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "Invalid key format should return 401: {:?}", body);
}

// ─── 8C: App Auth Middleware Tests ─────────────────────────────────────────────

#[tokio::test]
async fn test_admin_token_cannot_access_app_api() {
    let (mut app, _) = create_test_app().await;

    // Admin login gives client_type=admin by default
    let token = admin_token(&mut app).await;

    // Try to access App API → should be 403
    let req = auth_request("GET", "/api/v1/app/me", &token, None);
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "Admin token should not access App API: {:?}", body);
    assert_eq!(body["error"]["code"], "FORBIDDEN");
}

#[tokio::test]
async fn test_app_token_can_access_app_api() {
    let (mut app, _db) = create_test_app().await;

    // Create an App Token manually: generate a JWT with client_type=app
    let token = create_app_token(&app).await;

    // Access App API → should succeed
    let req = auth_request("GET", "/api/v1/app/me", &token, None);
    let (status, body) = send_request(&mut app, req).await;
    // Note: may fail with 404 or other if user doesn't exist, but should NOT be 403
    assert_ne!(
        status,
        StatusCode::FORBIDDEN,
        "App token should not be forbidden from App API: {:?}",
        body
    );
}

#[tokio::test]
async fn test_app_api_without_token_returns_401() {
    let (mut app, _) = create_test_app().await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/app/me")
        .body(Body::empty())
        .unwrap();
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "App API without token should return 401: {:?}", body);
}

/// Helper to create an App-type JWT token.
/// We manually craft a JWT with client_type=app by encoding it ourselves.
async fn create_app_token(_app: &axum::Router) -> String {
    // We need to get the JWT secret. Since the state is inside the Router,
    // we can't easily extract it. Instead, use the existing test infrastructure.
    // Create a token by reading the settings directly.
    dotenvy::dotenv().ok();
    let settings = Settings::new().expect("Failed to load settings");

    use jsonwebtoken::{encode, EncodingKey, Header};

    #[derive(serde::Serialize)]
    struct Claims {
        sub: String,
        email: String,
        role: String,
        role_id: Option<String>,
        department_id: Option<String>,
        jti: String,
        exp: i64,
        client_type: String,
    }

    // Get admin user id from DB
    let db = sqlx::PgPool::connect(&settings.database.url)
        .await
        .expect("Failed to connect");
    let admin_id: (uuid::Uuid, Option<uuid::Uuid>, Option<uuid::Uuid>) = sqlx::query_as(
        "SELECT id, role_id, department_id FROM users WHERE email = 'admin@example.com'",
    )
    .fetch_one(&db)
    .await
    .expect("Admin user should exist");

    let now = chrono::Utc::now();
    let exp = now + chrono::Duration::seconds(settings.jwt.access_exp_secs);

    let claims = Claims {
        sub: admin_id.0.to_string(),
        email: "admin@example.com".to_string(),
        role: "superadmin".to_string(),
        role_id: admin_id.1.map(|id| id.to_string()),
        department_id: admin_id.2.map(|id| id.to_string()),
        jti: uuid::Uuid::new_v4().to_string(),
        exp: exp.timestamp(),
        client_type: "app".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(settings.jwt.secret.as_bytes()),
    )
    .expect("Failed to encode app token")
}

// ─── 8D: Webhook CRUD Tests ───────────────────────────────────────────────────

#[tokio::test]
async fn test_webhook_crud_full_lifecycle() {
    let (mut app, db) = create_test_app().await;
    let token = admin_token(&mut app).await;
    let ts = chrono::Utc::now().timestamp_millis();
    let wh_name = format!("Test WH {}", ts);

    // ── Create Webhook ──
    let req = auth_request(
        "POST",
        "/api/v1/admin/webhooks",
        &token,
        Some(json!({
            "name": &wh_name,
            "url": "https://example.com/webhook",
            "events": ["user.created", "user.updated"]
        })),
    );
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK, "Create webhook failed: {:?}", body);

    let webhook_id = body["id"].as_str().unwrap();
    assert_eq!(body["name"], wh_name);
    assert_eq!(body["url"], "https://example.com/webhook");
    assert_eq!(body["status"], "active");
    assert!(body["events"].is_array());
    let events = body["events"].as_array().unwrap();
    assert!(events.contains(&json!("user.created")));
    assert!(events.contains(&json!("user.updated")));

    // ── List Webhooks ──
    let req = auth_request("GET", "/api/v1/admin/webhooks", &token, None);
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK, "List webhooks failed: {:?}", body);
    assert!(body["items"].is_array());
    assert!(body["total"].is_number());
    assert!(body["total"].as_i64().unwrap() >= 1);
    let items = body["items"].as_array().unwrap();
    assert!(items.iter().any(|w| w["id"] == webhook_id));

    // ── Update Webhook ──
    let req = auth_request(
        "PUT",
        &format!("/api/v1/admin/webhooks/{}", webhook_id),
        &token,
        Some(json!({
            "name": "Updated Webhook",
            "url": "https://example.com/new-webhook",
            "events": ["user.deleted"]
        })),
    );
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK, "Update webhook failed: {:?}", body);
    assert_eq!(body["name"], "Updated Webhook");
    assert_eq!(body["url"], "https://example.com/new-webhook");
    let updated_events = body["events"].as_array().unwrap();
    assert!(updated_events.contains(&json!("user.deleted")));
    assert!(!updated_events.contains(&json!("user.created")));

    // ── List Deliveries (empty for new webhook) ──
    let req = auth_request(
        "GET",
        &format!("/api/v1/admin/webhooks/{}/deliveries", webhook_id),
        &token,
        None,
    );
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK, "List deliveries failed: {:?}", body);
    assert!(body["items"].is_array());
    assert_eq!(body["total"], 0);

    // ── Delete Webhook ──
    let req = auth_request(
        "DELETE",
        &format!("/api/v1/admin/webhooks/{}", webhook_id),
        &token,
        None,
    );
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK, "Delete webhook failed: {:?}", body);
    assert!(body["message"].as_str().unwrap().contains("deleted"));

    // Verify deletion: listing should not contain the webhook
    let req = auth_request("GET", "/api/v1/admin/webhooks", &token, None);
    let (_, list_body) = send_request(&mut app, req).await;
    let items = list_body["items"].as_array().unwrap();
    assert!(
        !items.iter().any(|w| w["id"] == webhook_id),
        "Deleted webhook should not appear in list"
    );

    cleanup_webhooks(&db, "Test WH").await;
}

#[tokio::test]
async fn test_webhook_update_status() {
    let (mut app, db) = create_test_app().await;
    let token = admin_token(&mut app).await;
    let ts = chrono::Utc::now().timestamp_millis();
    let wh_name = format!("Status WH {}", ts);

    // Create
    let req = auth_request(
        "POST",
        "/api/v1/admin/webhooks",
        &token,
        Some(json!({
            "name": &wh_name,
            "url": "https://example.com/status-test",
            "events": ["user.created"]
        })),
    );
    let (_, create_body) = send_request(&mut app, req).await;
    let webhook_id = create_body["id"].as_str().unwrap();

    // Update status to disabled
    let req = auth_request(
        "PUT",
        &format!("/api/v1/admin/webhooks/{}", webhook_id),
        &token,
        Some(json!({"status": "disabled"})),
    );
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK, "Disable webhook failed: {:?}", body);
    assert_eq!(body["status"], "disabled");

    // Re-enable
    let req = auth_request(
        "PUT",
        &format!("/api/v1/admin/webhooks/{}", webhook_id),
        &token,
        Some(json!({"status": "active"})),
    );
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK, "Re-enable webhook failed: {:?}", body);
    assert_eq!(body["status"], "active");

    // Invalid status
    let req = auth_request(
        "PUT",
        &format!("/api/v1/admin/webhooks/{}", webhook_id),
        &token,
        Some(json!({"status": "invalid"})),
    );
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "Invalid status should return 400: {:?}", body);

    // Clean up
    let req = auth_request(
        "DELETE",
        &format!("/api/v1/admin/webhooks/{}", webhook_id),
        &token,
        None,
    );
    let _ = send_request(&mut app, req).await;

    cleanup_webhooks(&db, "Status WH").await;
}

#[tokio::test]
async fn test_webhook_no_permission() {
    let (mut app, db) = create_test_app().await;
    let email = unique_email("wh_noperm");

    // Normal user should not be able to manage webhooks
    let token = register_and_login(&mut app, &email, "TestPass@123", "No Perm").await;

    let req = auth_request(
        "POST",
        "/api/v1/admin/webhooks",
        &token,
        Some(json!({
            "name": "Forbidden WH",
            "url": "https://example.com/forbidden",
            "events": ["user.created"]
        })),
    );
    let (status, _body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "Normal user should not create webhooks");

    cleanup_user(&db, &email).await;
}

#[tokio::test]
async fn test_webhook_deliveries_not_found() {
    let (mut app, _) = create_test_app().await;
    let token = admin_token(&mut app).await;

    // Query deliveries for a non-existent webhook
    let fake_id = uuid::Uuid::new_v4();
    let req = auth_request(
        "GET",
        &format!("/api/v1/admin/webhooks/{}/deliveries", fake_id),
        &token,
        None,
    );
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "Non-existent webhook deliveries should 404: {:?}", body);
}

// ─── API Key: Wildcard Scope Test ─────────────────────────────────────────────

#[tokio::test]
async fn test_api_key_wildcard_scope() {
    let (mut app, _db) = create_test_app().await;
    let token = admin_token(&mut app).await;

    // Create API key with wildcard scope
    let req = auth_request(
        "POST",
        "/api/v1/admin/api-keys",
        &token,
        Some(json!({
            "name": "Wildcard Key",
            "scopes": ["*"]
        })),
    );
    let (status, create_body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK);
    let api_key = create_body["key"].as_str().unwrap().to_string();
    let key_id = create_body["id"].as_str().unwrap();

    // Wildcard scope should allow access to all Open API endpoints
    let req = api_key_request("GET", "/api/v1/open/users", &api_key, None);
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK, "Wildcard should access /users: {:?}", body);

    let req = api_key_request("GET", "/api/v1/open/departments", &api_key, None);
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK, "Wildcard should access /departments: {:?}", body);

    // Clean up
    let req = auth_request("DELETE", &format!("/api/v1/admin/api-keys/{}", key_id), &token, None);
    let _ = send_request(&mut app, req).await;
}

// ─── API Key Auth via Bearer Header ────────────────────────────────────────────

#[tokio::test]
async fn test_api_key_via_bearer_header() {
    let (mut app, _db) = create_test_app().await;
    let token = admin_token(&mut app).await;

    // Create API key
    let req = auth_request(
        "POST",
        "/api/v1/admin/api-keys",
        &token,
        Some(json!({
            "name": "Bearer Key Test",
            "scopes": ["users:read"]
        })),
    );
    let (_, create_body) = send_request(&mut app, req).await;
    let api_key = create_body["key"].as_str().unwrap().to_string();
    let key_id = create_body["id"].as_str().unwrap();

    // Use API key via Authorization: Bearer header
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/open/users")
        .header("Authorization", format!("Bearer {}", api_key))
        .body(Body::empty())
        .unwrap();
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK, "API key via Bearer header should work: {:?}", body);

    // Clean up
    let req = auth_request("DELETE", &format!("/api/v1/admin/api-keys/{}", key_id), &token, None);
    let _ = send_request(&mut app, req).await;
}
