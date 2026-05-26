pub mod config;
pub mod db;
pub mod error;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod repository;
pub mod routes;
pub mod services;

use axum::Router;
use axum::middleware as axum_mw;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Instant;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tower_governor::GovernorLayer;

use crate::config::Settings;
use crate::middleware::request_id::request_id_layer;

pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
    pub access_exp_secs: i64,
    pub refresh_exp_secs: i64,
    pub started_at: Instant,
    pub max_connections: u32,
    pub settings: crate::config::Settings,
    pub totp_encryption_key: String,
    pub config_cache: std::sync::Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
    pub notification_tx: tokio::sync::broadcast::Sender<crate::models::notification::SseNotification>,
    pub http_client: reqwest::Client,
    pub webhook_tx: tokio::sync::mpsc::Sender<crate::models::webhook::WebhookEvent>,
}

/// Build the Axum application
pub async fn app(settings: &Settings) -> anyhow::Result<Router> {
    // Database
    let db = db::init_pool(&settings.database).await?;
    sqlx::migrate!("./migrations").run(&db).await?;

    let jwt_secret = settings.jwt.secret.clone();
    let access_exp_secs = settings.jwt.access_exp_secs;
    let refresh_exp_secs = settings.jwt.refresh_exp_secs;
    let max_connections = settings.database.max_connections;
    let totp_encryption_key = settings.totp.encryption_key.clone();

    // Initialize config cache
    let config_cache_map: HashMap<String, HashMap<String, String>> =
        match crate::repository::system_config_repo::find_all(&db).await {
            Ok(configs) => {
                let mut map: HashMap<String, HashMap<String, String>> = HashMap::new();
                for c in &configs {
                    map.entry(c.group_key.clone())
                        .or_default()
                        .insert(c.config_key.clone(), c.value.clone());
                }
                map
            }
            Err(e) => {
                tracing::warn!("Failed to load config cache on startup: {:?}", e);
                HashMap::new()
            }
        };
    let config_cache = std::sync::Arc::new(RwLock::new(config_cache_map));

    // Notification broadcast channel (capacity 256)
    let (notification_tx, _) = tokio::sync::broadcast::channel(256);

    // HTTP client for OAuth and webhook calls
    let http_client = reqwest::Client::new();

    // Webhook event channel (mpsc, capacity 256)
    let (webhook_tx, webhook_rx) =
        tokio::sync::mpsc::channel::<crate::models::webhook::WebhookEvent>(256);

    let state = std::sync::Arc::new(AppState {
        db,
        jwt_secret,
        access_exp_secs,
        refresh_exp_secs,
        started_at: Instant::now(),
        max_connections,
        settings: settings.clone(),
        totp_encryption_key,
        config_cache,
        notification_tx,
        http_client: http_client.clone(),
        webhook_tx,
    });

    // Start webhook event worker (consumes events from mpsc channel)
    {
        let pool = state.db.clone();
        let client = http_client.clone();
        let wh_settings = settings.webhook.clone();
        crate::services::webhook_service::start_event_worker(pool, client, wh_settings, webhook_rx);
    }

    // Start webhook retry worker (periodic retry of failed deliveries)
    {
        let pool = state.db.clone();
        let client = http_client.clone();
        let wh_settings = settings.webhook.clone();
        crate::services::webhook_service::start_retry_worker(pool, client, wh_settings);
    }

    // CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Global rate limiting: X requests per minute per IP
    let global_rps = (settings.rate_limit.global_rpm / 60).max(1);
    let global_governor_config = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(global_rps)
        .burst_size(settings.rate_limit.global_rpm as u32)
        .finish()
        .expect("Global governor config should be valid");

    let global_governor = GovernorLayer::new(global_governor_config);

    // Login-specific rate limiting: Y requests per minute per IP
    let login_rps = (settings.rate_limit.login_rpm / 60).max(1);
    let login_governor_config = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(login_rps)
        .burst_size(settings.rate_limit.login_rpm as u32)
        .finish()
        .expect("Login governor config should be valid");

    let login_governor = GovernorLayer::new(login_governor_config);

    // Auth middleware layer
    let auth_mw = axum_mw::from_fn_with_state(
        state.clone(),
        middleware::auth::require_auth,
    );

    // API Key auth middleware layer
    let api_key_mw = axum_mw::from_fn_with_state(
        state.clone(),
        middleware::api_key_auth::require_api_key_check,
    );

    // Deprecation middleware for legacy routes
    let deprecation_mw = axum_mw::from_fn(middleware::deprecation::deprecation_header);

    // Routes
    let mut app = Router::new()
        // ─── Auth routes (public, with login-specific rate limiting, NOT versioned) ───
        .merge(
            routes::auth_routes().layer(login_governor)
        )
        // ─── App Auth routes (public OAuth/SMS endpoints, no auth middleware) ───
        .merge(routes::app_auth_routes())
        // ─── New v1 Admin API ───
        .nest(
            "/api/v1/admin",
            Router::new()
                .merge(routes::auth_protected_routes())
                .merge(routes::user_routes())
                .merge(routes::role_routes())
                .merge(routes::audit_routes())
                .merge(routes::dashboard_routes())
                .merge(routes::file_routes())
                .merge(routes::export_routes())
                .merge(routes::session_routes())
                .merge(routes::menu_routes())
                .merge(routes::notification_routes())
                .merge(routes::config_routes())
                .merge(routes::department_routes())
                .merge(routes::dict_routes())
                .merge(routes::dict_item_routes())
                .merge(routes::login_log_routes())
                .merge(routes::import_routes())
                .merge(routes::search_routes())
                .merge(routes::dict_public_routes())
                .merge(routes::api_key_routes())
                .merge(routes::webhook_routes())
                .layer(auth_mw.clone()),
        )
        // ─── App API (v1) ───
        .nest(
            "/api/v1/app",
            routes::app_routes().layer(axum_mw::from_fn_with_state(
                state.clone(),
                middleware::app_auth::require_app_auth_check,
            )),
        )
        // ─── Open API (v1) — public ───
        .nest(
            "/api/v1/open",
            routes::open_public_routes(),
        )
        // ─── Open API (v1) — protected (API key required) ───
        .nest(
            "/api/v1/open",
            routes::open_protected_routes().layer(api_key_mw),
        )
        // ─── Legacy routes (with deprecation headers) ───
        .nest(
            "/api",
            Router::new()
                .merge(routes::auth_protected_routes())
                .merge(routes::user_routes())
                .merge(routes::role_routes())
                .merge(routes::audit_routes())
                .merge(routes::dashboard_routes())
                .merge(routes::file_routes())
                .merge(routes::export_routes())
                .merge(routes::session_routes())
                .merge(routes::menu_routes())
                .merge(routes::notification_routes())
                .merge(routes::config_routes())
                .merge(routes::department_routes())
                .merge(routes::dict_routes())
                .merge(routes::dict_item_routes())
                .merge(routes::login_log_routes())
                .merge(routes::import_routes())
                .merge(routes::search_routes())
                .merge(routes::dict_public_routes())
                .layer(auth_mw.clone())
                .layer(deprecation_mw),
        )
        // SSE route (not versioned)
        .merge(routes::sse_routes())
        .merge(routes::i18n_routes())
        // Public routes (no auth required)
        .merge(routes::config_public_routes())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(global_governor)
        .layer(axum_mw::from_fn(request_id_layer));

    // Swagger UI + OpenAPI spec (if docs_enabled)
    if settings.api.docs_enabled {
        use utoipa::OpenApi as _;
        let openapi_spec = handlers::docs_handler::ApiDoc::openapi();
        let docs_router = Router::new()
            .route(
                "/docs/openapi.json",
                axum::routing::get(move || async move {
                    axum::Json(openapi_spec)
                }),
            )
            .route(
                "/docs",
                axum::routing::get(|| async {
                    axum::response::Html(handlers::docs_handler::swagger_ui_html())
                }),
            );
        app = app.merge(docs_router);
    }

    Ok(app)
}
