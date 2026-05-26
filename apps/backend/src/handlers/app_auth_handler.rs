use axum::extract::{Query, State};
use axum::response::Json;
use axum::Extension;
use serde::Deserialize;
use std::sync::Arc;

use crate::error::AppError;
use crate::extractors::auth::AuthUser;
use crate::models::oauth_account::{SocialBindRequest, SocialUnbindRequest};
use crate::models::user::TokenResponse;
use crate::AppState;

// ─── Query / Request Types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AuthorizeQuery {
    pub provider: String,
    pub redirect_uri: String,
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub provider: String,
    pub code: String,
    #[allow(dead_code)]
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct SendSmsRequest {
    pub phone: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifySmsRequest {
    pub phone: String,
    pub code: String,
}

// ─── Handlers ───────────────────────────────────────────────────────────────

/// GET /api/v1/app/auth/authorize?provider=github&redirect_uri=...
/// Returns the authorization URL for the specified OAuth provider.
pub async fn authorize(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuthorizeQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Validate provider
    if !crate::models::oauth_account::OAuthAccount::is_valid_provider(&query.provider) {
        return Err(AppError::BadRequest(format!(
            "Unsupported provider: {}",
            query.provider
        )));
    }

    let state_param = uuid::Uuid::new_v4().to_string();
    let url = crate::services::oauth_service::get_authorization_url(
        &query.provider,
        &query.redirect_uri,
        &state_param,
        &state.settings.api,
    )?;

    Ok(Json(serde_json::json!({
        "authorization_url": url,
        "state": state_param,
    })))
}

/// GET /api/v1/app/auth/callback?provider=github&code=xxx&state=xxx
/// Exchanges the authorization code for JWT tokens.
pub async fn callback(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CallbackQuery>,
) -> Result<Json<TokenResponse>, AppError> {
    let token_response = crate::services::oauth_service::handle_callback(
        &state.db,
        &query.provider,
        &query.code,
        &query.state,
        &state.settings.api,
        &state.http_client,
        &state.jwt_secret,
        state.access_exp_secs,
        state.refresh_exp_secs,
    )
    .await?;

    Ok(Json(token_response))
}

/// GET /api/v1/app/auth/providers
/// Returns a list of available OAuth providers.
pub async fn list_providers(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut providers = vec![];

    // GitHub: available if client_id is configured
    let github_configured = !state.settings.api.github.client_id.is_empty();
    providers.push(serde_json::json!({
        "provider": "github",
        "name": "GitHub",
        "enabled": github_configured,
    }));

    // Google: available if client_id is configured
    let google_configured = !state.settings.api.google.client_id.is_empty();
    providers.push(serde_json::json!({
        "provider": "google",
        "name": "Google",
        "enabled": google_configured,
    }));

    // WeChat: available if enabled and configured
    providers.push(serde_json::json!({
        "provider": "wechat",
        "name": "WeChat",
        "enabled": state.settings.api.wechat.enabled && !state.settings.api.wechat.app_id.is_empty(),
    }));

    Ok(Json(serde_json::json!({ "providers": providers })))
}

/// POST /api/v1/app/auth/sms/send
/// Send a verification code to the specified phone number.
pub async fn send_sms(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SendSmsRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let code = crate::services::sms_service::send_code(&state.db, &req.phone).await?;

    Ok(Json(serde_json::json!({
        "ok": true,
        // In development mode, include the code for easy testing
        "code": code,
        "message": "Verification code sent (check server logs in production)",
    })))
}

/// POST /api/v1/app/auth/sms/verify
/// Verify a phone number + code, find or create a user, return JWT tokens.
pub async fn verify_sms(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VerifySmsRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    // 1. Verify the code
    let valid = crate::services::sms_service::verify_code(&state.db, &req.phone, &req.code)
        .await?;

    if !valid {
        return Err(AppError::Unauthorized(
            "Invalid or expired verification code".into(),
        ));
    }

    // 2. Find user by phone number
    let user = find_or_create_user_by_phone(&state.db, &req.phone).await?;

    // 3. Create app tokens
    let token_response = crate::services::oauth_service::create_app_token_response_public(
        &state.db,
        &user,
        &state.jwt_secret,
        state.access_exp_secs,
        state.refresh_exp_secs,
    )
    .await?;

    Ok(Json(token_response))
}

/// POST /api/v1/app/auth/social-bind
/// Bind a social account to the authenticated user.
pub async fn social_bind(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<SocialBindRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let account = crate::services::oauth_service::bind_social_account(
        &state.db,
        auth_user.user_id,
        &req.provider,
        &req.code,
        &req.redirect_uri,
        &state.settings.api,
        &state.http_client,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "ok": true,
        "provider": account.provider,
        "provider_user_id": account.provider_user_id,
    })))
}

/// POST /api/v1/app/auth/social-unbind
/// Unbind a social account from the authenticated user.
pub async fn social_unbind(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<SocialUnbindRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    crate::services::oauth_service::unbind_social_account(
        &state.db,
        auth_user.user_id,
        &req.provider,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "ok": true,
        "message": format!("{} account unbound successfully", req.provider),
    })))
}

// ─── Helper Functions ───────────────────────────────────────────────────────

/// Find or create a user by phone number.
/// Uses the phone number as a pseudo-email (phone@app.local) if no email is available.
async fn find_or_create_user_by_phone(
    pool: &sqlx::PgPool,
    phone: &str,
) -> Result<crate::models::user::User, AppError> {
    use crate::models::user::UserStatus;

    // Check if a user with this phone already exists
    let user = sqlx::query_as::<_, crate::models::user::User>(
        "SELECT * FROM users WHERE phone = $1",
    )
    .bind(phone)
    .fetch_optional(pool)
    .await?;

    if let Some(user) = user {
        return Ok(user);
    }

    // Create a new user with phone as identifier
    let pseudo_email = format!("{}@sms.local", phone);
    let random_password = crate::services::oauth_service::generate_random_password_display();
    let password_hash = crate::services::auth_service::hash_password(&random_password)?;

    let user_role = crate::repository::role_repo::find_by_name(pool, "user").await?;
    let role_id = user_role.as_ref().map(|r| r.id);

    // Create user with phone number
    let user = sqlx::query_as::<_, crate::models::user::User>(
        r#"INSERT INTO users (email, password_hash, name, role, status, must_change_password, role_id, phone)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
           RETURNING *"#,
    )
    .bind(&pseudo_email)
    .bind(&password_hash)
    .bind(format!("User {}", phone))
    .bind(UserStatus::Active.as_str())
    .bind(false)
    .bind(role_id)
    .bind(phone)
    .fetch_one(pool)
    .await?;

    Ok(user)
}
