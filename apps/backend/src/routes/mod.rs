use axum::routing::{delete, get, post, put};
use axum::Router;
use std::sync::Arc;

use crate::handlers::{
    api_key_handler, app_auth_handler, app_user_handler, audit_handler, auth_handler,
    dashboard_handler, department_handler, dict_handler, export_handler, file_handler,
    i18n_handler, import_handler, login_log_handler, menu_handler, notification_handler,
    open_api_handler, role_handler, search_handler, session_handler, sse_handler,
    system_config_handler, two_factor_handler, user_handler, webhook_handler,
};
use crate::AppState;

// ─── Auth Routes ───────────────────────────────────────────────

/// Public auth routes (login, register, refresh, 2fa-verify).
/// These are NOT versioned — the frontend already uses these paths.
pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/api/auth",
        Router::new()
            .route("/register", post(auth_handler::register))
            .route("/login", post(auth_handler::login))
            .route("/refresh", post(auth_handler::refresh))
            .route("/2fa/verify", post(auth_handler::verify_2fa)),
    )
}

/// Auth routes that require authentication (e.g. logout needs JWT blacklist)
pub fn auth_protected_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/auth",
        Router::new()
            .route("/logout", post(auth_handler::logout))
            .route("/2fa/setup", post(two_factor_handler::setup))
            .route("/2fa/enable", post(two_factor_handler::enable))
            .route("/2fa/disable", post(two_factor_handler::disable)),
    )
}

// ─── Admin Routes (prefix-free; mounted under /api or /api/v1/admin) ──

pub fn user_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/users",
        Router::new()
            .route("/me", get(user_handler::get_me))
            .route("/me/password", put(user_handler::change_my_password))
            .route(
                "/",
                get(user_handler::list_users).post(user_handler::create_user),
            )
            .route(
                "/{id}",
                get(user_handler::get_user)
                    .put(user_handler::update_user)
                    .delete(user_handler::delete_user),
            )
            .route("/{id}/status", put(user_handler::update_status))
            .route("/{id}/password", put(user_handler::reset_password))
            .route("/{id}/2fa/reset", post(two_factor_handler::reset_user_2fa))
            .route("/me/avatar", post(user_handler::upload_avatar)),
    )
}

pub fn role_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/roles",
        Router::new()
            .route(
                "/",
                get(role_handler::list_roles).post(role_handler::create_role),
            )
            .route("/permissions", get(role_handler::list_permissions))
            .route(
                "/{id}",
                get(role_handler::get_role)
                    .put(role_handler::update_role)
                    .delete(role_handler::delete_role),
            )
            .route("/{id}/permissions", put(role_handler::update_permissions)),
    )
}

pub fn audit_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/audit-logs",
        Router::new()
            .route("/", get(audit_handler::list_logs))
            .route("/export", get(audit_handler::export_logs))
            .route("/{id}", get(audit_handler::get_log_detail)),
    )
}

pub fn dashboard_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/dashboard",
        Router::new()
            .route("/stats", get(dashboard_handler::stats))
            .route("/settings", get(dashboard_handler::settings)),
    )
}

pub fn file_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/files",
        Router::new()
            .route("/", get(file_handler::list_files).post(file_handler::upload_file))
            .route("/{id}", delete(file_handler::delete_file)),
    )
}

pub fn export_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/export",
        Router::new().route("/users", get(export_handler::export_users)),
    )
}

pub fn session_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/sessions",
        Router::new()
            .route("/", get(session_handler::list_sessions))
            .route("/me", get(session_handler::list_my_sessions))
            .route("/user/{user_id}", delete(session_handler::force_logout_user))
            .route("/{id}", delete(session_handler::terminate_session)),
    )
}

pub fn menu_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/menus",
        Router::new()
            .route(
                "/",
                get(menu_handler::list_menus).post(menu_handler::create_menu),
            )
            .route("/tree", get(menu_handler::get_menu_tree))
            .route(
                "/{id}",
                put(menu_handler::update_menu).delete(menu_handler::delete_menu),
            ),
    )
}

pub fn notification_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/notifications",
        Router::new()
            .route("/", get(notification_handler::list))
            .route("/unread-count", get(notification_handler::unread_count))
            .route("/read-all", post(notification_handler::mark_all_read))
            .route("/broadcast", post(notification_handler::broadcast))
            .route("/{id}/read", post(notification_handler::mark_read))
            .route("/{id}", delete(notification_handler::delete_notification)),
    )
}

pub fn config_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/configs",
        Router::new()
            .route("/", get(system_config_handler::list_all))
            .route("/{group}", get(system_config_handler::list_by_group))
            .route("/{group}/{key}", put(system_config_handler::update_config)),
    )
}

pub fn config_public_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/api/configs",
        Router::new().route("/public", get(system_config_handler::get_public)),
    )
}

pub fn department_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/departments",
        Router::new()
            .route(
                "/",
                get(department_handler::list_departments).post(department_handler::create_department),
            )
            .route(
                "/{id}",
                put(department_handler::update_department).delete(department_handler::delete_department),
            ),
    )
}

pub fn dict_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/dict-types",
        Router::new()
            .route(
                "/",
                get(dict_handler::list_dict_types).post(dict_handler::create_dict_type),
            )
            .route(
                "/{id}",
                put(dict_handler::update_dict_type).delete(dict_handler::delete_dict_type),
            )
            .route("/{id}/items", get(dict_handler::list_dict_items).post(dict_handler::create_dict_item)),
    )
}

pub fn dict_item_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/dict-items",
        Router::new()
            .route(
                "/{id}",
                put(dict_handler::update_dict_item).delete(dict_handler::delete_dict_item),
            ),
    )
}

pub fn dict_public_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/dicts",
        Router::new().route("/{code}", get(dict_handler::get_dict_by_code)),
    )
}

pub fn login_log_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/login-logs",
        Router::new()
            .route("/", get(login_log_handler::list_login_logs))
            .route("/{id}", get(login_log_handler::get_login_log)),
    )
}

pub fn import_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/users/import",
        Router::new()
            .route("/", post(import_handler::import_users))
            .route("/template", get(import_handler::download_import_template)),
    )
}

pub fn search_routes() -> Router<Arc<AppState>> {
    Router::new().route("/search", get(search_handler::search))
}

pub fn i18n_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/api/i18n",
        Router::new().route("/messages", get(i18n_handler::get_messages)),
    )
}

pub fn sse_routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/sse/notifications", get(sse_handler::notification_stream))
}

// ─── API Key Management Routes (Phase 8B — mounted under /api/v1/admin) ──

pub fn api_key_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/api-keys",
        Router::new()
            .route("/", post(api_key_handler::create_api_key).get(api_key_handler::list_api_keys))
            .route("/{id}", delete(api_key_handler::revoke_api_key)),
    )
}

// ─── App Auth Routes (Phase 8C — public, no auth middleware) ─────

pub fn app_auth_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/api/v1/app/auth",
        Router::new()
            .route("/authorize", get(app_auth_handler::authorize))
            .route("/callback", get(app_auth_handler::callback))
            .route("/providers", get(app_auth_handler::list_providers))
            .route("/sms/send", post(app_auth_handler::send_sms))
            .route("/sms/verify", post(app_auth_handler::verify_sms)),
    )
}

// ─── App API Routes (Phase 8 — mounted under /api/v1/app, requires app auth) ─────

pub fn app_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/me", get(user_handler::get_me))
        .route("/me/password", put(user_handler::change_my_password))
        .route("/notifications", get(notification_handler::list))
        .route("/notifications/{id}/read", post(notification_handler::mark_read))
        // Phase 8C: Social bindings and bind/unbind (require app auth)
        .route("/auth/bindings", get(app_user_handler::list_bindings))
        .route("/auth/social-bind", post(app_auth_handler::social_bind))
        .route("/auth/social-unbind", post(app_auth_handler::social_unbind))
}

// ─── Open API Routes (Phase 8 — mounted under /api/v1/open) ───

/// Public Open API routes (no authentication required)
pub fn open_public_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(open_api_handler::health))
}

/// Protected Open API routes (require API key authentication)
pub fn open_protected_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users", get(open_api_handler::list_users))
        .route("/users/{id}", get(open_api_handler::get_user))
        .route("/departments", get(open_api_handler::list_departments))
}

// ─── Webhook Routes (Phase 8D — mounted under /api/v1/admin) ──

pub fn webhook_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/webhooks",
        Router::new()
            .route(
                "/",
                post(webhook_handler::create_webhook).get(webhook_handler::list_webhooks),
            )
            .route(
                "/{id}",
                put(webhook_handler::update_webhook).delete(webhook_handler::delete_webhook),
            )
            .route("/{id}/deliveries", get(webhook_handler::list_deliveries)),
    )
}
