use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Serialize;
use sqlx::PgPool;

use crate::config::ApiSettings;
use crate::error::AppError;
use crate::models::oauth_account::OAuthAccount;
use crate::models::user::{TokenResponse, User, UserStatus};
use crate::repository::{oauth_account_repo, refresh_token_repo, user_repo};

/// JWT claims for app tokens
#[derive(Debug, Serialize)]
struct AppClaims {
    sub: String,
    email: String,
    role: String,
    role_id: Option<String>,
    department_id: Option<String>,
    jti: String,
    exp: i64,
    two_factor_pending: Option<bool>,
    client_type: Option<String>,
}

/// Hash a refresh token with SHA-256 for secure storage
fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

// ─── OAuth Provider Configuration ───────────────────────────────────────────

/// GitHub OAuth endpoints
mod github {
    pub const AUTHORIZE_URL: &str = "https://github.com/login/oauth/authorize";
    pub const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
    pub const USER_URL: &str = "https://api.github.com/user";
}

/// Google OAuth endpoints
mod google {
    pub const AUTHORIZE_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
    pub const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
    pub const USER_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";
}

// ─── Authorization URL Generation ───────────────────────────────────────────

/// Generate the authorization URL for a given OAuth provider.
/// Returns the URL to redirect the user to for authentication.
pub fn get_authorization_url(
    provider: &str,
    redirect_uri: &str,
    state: &str,
    settings: &ApiSettings,
) -> Result<String, AppError> {
    match provider {
        "github" => {
            let (client_id, _) = get_github_settings(settings)?;
            let url = format!(
                "{}?client_id={}&redirect_uri={}&state={}&scope=user:email",
                github::AUTHORIZE_URL,
                urlencoding::encode(&client_id),
                urlencoding::encode(redirect_uri),
                urlencoding::encode(state),
            );
            Ok(url)
        }
        "google" => {
            let (client_id, _) = get_google_settings(settings)?;
            let url = format!(
                "{}?client_id={}&redirect_uri={}&state={}&scope=openid+email+profile&response_type=code",
                google::AUTHORIZE_URL,
                urlencoding::encode(&client_id),
                urlencoding::encode(redirect_uri),
                urlencoding::encode(state),
            );
            Ok(url)
        }
        "wechat" => Err(AppError::BadRequest(
            "WeChat OAuth is not yet supported".into(),
        )),
        _ => Err(AppError::BadRequest(format!(
            "Unsupported OAuth provider: {}",
            provider
        ))),
    }
}

// ─── OAuth Callback Handling ────────────────────────────────────────────────

/// Handle the OAuth callback: exchange code for token, get user info,
/// find or create a local user, and return JWT tokens.
pub async fn handle_callback(
    pool: &PgPool,
    provider: &str,
    code: &str,
    _state: &str,
    settings: &ApiSettings,
    http_client: &reqwest::Client,
    jwt_secret: &str,
    access_exp_secs: i64,
    refresh_exp_secs: i64,
) -> Result<TokenResponse, AppError> {
    // 1. Exchange code for access token
    let provider_access_token = exchange_code(provider, code, settings, http_client).await?;

    // 2. Get user info from provider
    let (provider_user_id, email, name) =
        get_user_info(provider, &provider_access_token, http_client).await?;

    // 3. Check if OAuth account already exists
    if let Some(oauth_account) =
        oauth_account_repo::find_by_provider_user(pool, provider, &provider_user_id).await?
    {
        // Existing link — load user and return tokens
        let user = user_repo::find_by_id(pool, oauth_account.user_id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("OAuth user not found")))?;

        return create_app_token_response(pool, &user, jwt_secret, access_exp_secs, refresh_exp_secs)
            .await;
    }

    // 4. Try to find user by email and link
    if let Some(user) = user_repo::find_by_email(pool, &email).await? {
        // Link the OAuth account to existing user
        oauth_account_repo::create(pool, user.id, provider, &provider_user_id, None).await?;

        return create_app_token_response(pool, &user, jwt_secret, access_exp_secs, refresh_exp_secs)
            .await;
    }

    // 5. Create new user with random password and link OAuth account
    let random_password = generate_random_password();
    let password_hash = crate::services::auth_service::hash_password(&random_password)?;

    let user_role = crate::repository::role_repo::find_by_name(pool, "user").await?;
    let role_id = user_role.as_ref().map(|r| r.id);

    let user = user_repo::create_user(
        pool,
        &email,
        &password_hash,
        Some(&name),
        "user",
        UserStatus::Active.as_str(),
        false,
        role_id,
        None,
    )
    .await?;

    // Link OAuth account to the new user
    oauth_account_repo::create(pool, user.id, provider, &provider_user_id, None).await?;

    create_app_token_response(pool, &user, jwt_secret, access_exp_secs, refresh_exp_secs).await
}

// ─── Social Bind / Unbind ───────────────────────────────────────────────────

/// Bind a social account to an existing user.
pub async fn bind_social_account(
    pool: &PgPool,
    user_id: uuid::Uuid,
    provider: &str,
    code: &str,
    _redirect_uri: &str,
    settings: &ApiSettings,
    http_client: &reqwest::Client,
) -> Result<OAuthAccount, AppError> {
    // Exchange code for access token
    let provider_access_token = exchange_code(provider, code, settings, http_client).await?;

    // Get user info from provider
    let (provider_user_id, _email, _name) =
        get_user_info(provider, &provider_access_token, http_client).await?;

    // Check if already bound to another user
    if let Some(existing) =
        oauth_account_repo::find_by_provider_user(pool, provider, &provider_user_id).await?
    {
        if existing.user_id != user_id {
            return Err(AppError::Conflict(
                "This social account is already bound to another user".into(),
            ));
        }
        return Err(AppError::Conflict(
            "This social account is already bound to your account".into(),
        ));
    }

    // Create the binding
    let account =
        oauth_account_repo::create(pool, user_id, provider, &provider_user_id, None).await?;

    Ok(account)
}

/// Unbind a social account from a user.
pub async fn unbind_social_account(
    pool: &PgPool,
    user_id: uuid::Uuid,
    provider: &str,
) -> Result<(), AppError> {
    let account = oauth_account_repo::find_by_user_and_provider(pool, user_id, provider)
        .await?
        .ok_or_else(|| AppError::NotFound("Social account binding not found".into()))?;

    oauth_account_repo::delete(pool, account.id).await?;
    Ok(())
}

// ─── Token Exchange Helpers ─────────────────────────────────────────────────

/// Exchange an authorization code for an access token
async fn exchange_code(
    provider: &str,
    code: &str,
    settings: &ApiSettings,
    http_client: &reqwest::Client,
) -> Result<String, AppError> {
    match provider {
        "github" => {
            let (client_id, client_secret) = get_github_settings(settings)?;
            let resp = http_client
                .post(github::TOKEN_URL)
                .header("Accept", "application/json")
                .json(&serde_json::json!({
                    "client_id": client_id,
                    "client_secret": client_secret,
                    "code": code,
                }))
                .send()
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("GitHub token exchange failed: {}", e)))?;

            let body: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to parse GitHub token response: {}", e)))?;

            body.get("access_token")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| AppError::Unauthorized("Failed to get GitHub access token".into()))
        }
        "google" => {
            let (client_id, client_secret) = get_google_settings(settings)?;
            let resp = http_client
                .post(google::TOKEN_URL)
                .json(&serde_json::json!({
                    "client_id": client_id,
                    "client_secret": client_secret,
                    "code": code,
                    "grant_type": "authorization_code",
                    "redirect_uri": "",  // Not needed for server-side
                }))
                .send()
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Google token exchange failed: {}", e)))?;

            let body: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to parse Google token response: {}", e)))?;

            body.get("access_token")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| AppError::Unauthorized("Failed to get Google access token".into()))
        }
        _ => Err(AppError::BadRequest(format!(
            "Unsupported provider: {}",
            provider
        ))),
    }
}

/// Get user info from an OAuth provider using an access token
async fn get_user_info(
    provider: &str,
    access_token: &str,
    http_client: &reqwest::Client,
) -> Result<(String, String, String), AppError> {
    match provider {
        "github" => {
            let resp = http_client
                .get(github::USER_URL)
                .header("Authorization", format!("Bearer {}", access_token))
                .header("User-Agent", "cradle")
                .send()
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("GitHub user info request failed: {}", e)))?;

            let body: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to parse GitHub user info: {}", e)))?;

            let provider_user_id = body
                .get("id")
                .and_then(|v| v.as_i64())
                .map(|id| id.to_string())
                .ok_or_else(|| AppError::Unauthorized("Failed to get GitHub user ID".into()))?;

            let login = body
                .get("login")
                .and_then(|v| v.as_str())
                .unwrap_or("github_user");

            let email = body
                .get("email")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // If email is private, try to fetch emails endpoint
            let email = if email.is_empty() {
                fetch_github_email(access_token, http_client).await?
            } else {
                email
            };

            let name = body
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(login)
                .to_string();

            Ok((provider_user_id, email, name))
        }
        "google" => {
            let resp = http_client
                .get(google::USER_URL)
                .header("Authorization", format!("Bearer {}", access_token))
                .send()
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Google user info request failed: {}", e)))?;

            let body: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to parse Google user info: {}", e)))?;

            let provider_user_id = body
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| AppError::Unauthorized("Failed to get Google user ID".into()))?;

            let email = body
                .get("email")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let name = body
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Google User")
                .to_string();

            Ok((provider_user_id, email, name))
        }
        _ => Err(AppError::BadRequest(format!(
            "Unsupported provider: {}",
            provider
        ))),
    }
}

/// Fetch the primary email from GitHub's /user/emails endpoint
async fn fetch_github_email(
    access_token: &str,
    http_client: &reqwest::Client,
) -> Result<String, AppError> {
    let resp = http_client
        .get("https://api.github.com/user/emails")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "cradle")
        .send()
        .await
        .map_err(|e| {
            AppError::Internal(anyhow::anyhow!("GitHub emails request failed: {}", e))
        })?;

    let emails: Vec<serde_json::Value> = resp.json().await.map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Failed to parse GitHub emails: {}", e))
    })?;

    // Prefer the primary verified email
    for email_obj in &emails {
        let is_primary = email_obj.get("primary").and_then(|v| v.as_bool()).unwrap_or(false);
        let is_verified = email_obj.get("verified").and_then(|v| v.as_bool()).unwrap_or(false);
        if is_primary && is_verified {
            if let Some(email) = email_obj.get("email").and_then(|v| v.as_str()) {
                return Ok(email.to_string());
            }
        }
    }

    // Fall back to any email
    for email_obj in &emails {
        if let Some(email) = email_obj.get("email").and_then(|v| v.as_str()) {
            return Ok(email.to_string());
        }
    }

    Ok(format!("github_{}@no-email.local", chrono::Utc::now().timestamp()))
}

// ─── Config Helpers ─────────────────────────────────────────────────────────

fn get_github_settings(settings: &ApiSettings) -> Result<(String, String), AppError> {
    let client_id = settings.github.client_id.clone();
    let client_secret = settings.github.client_secret.clone();
    if client_id.is_empty() || client_secret.is_empty() {
        return Err(AppError::BadRequest(
            "GitHub OAuth is not configured. Set api.github.client_id and api.github.client_secret".into(),
        ));
    }
    Ok((client_id, client_secret))
}

fn get_google_settings(settings: &ApiSettings) -> Result<(String, String), AppError> {
    let client_id = settings.google.client_id.clone();
    let client_secret = settings.google.client_secret.clone();
    if client_id.is_empty() || client_secret.is_empty() {
        return Err(AppError::BadRequest(
            "Google OAuth is not configured. Set api.google.client_id and api.google.client_secret".into(),
        ));
    }
    Ok((client_id, client_secret))
}

// ─── Token Creation ─────────────────────────────────────────────────────────

/// Create app JWT tokens and store refresh token.
/// Public wrapper for use by app_auth_handler.
pub async fn create_app_token_response_public(
    pool: &PgPool,
    user: &User,
    jwt_secret: &str,
    access_exp_secs: i64,
    refresh_exp_secs: i64,
) -> Result<TokenResponse, AppError> {
    create_app_token_response(pool, user, jwt_secret, access_exp_secs, refresh_exp_secs).await
}

/// Create app JWT tokens and store refresh token
async fn create_app_token_response(
    pool: &PgPool,
    user: &User,
    jwt_secret: &str,
    access_exp_secs: i64,
    refresh_exp_secs: i64,
) -> Result<TokenResponse, AppError> {
    let access_token = create_app_jwt(user, jwt_secret, access_exp_secs)?;
    let refresh_token = create_app_jwt(user, jwt_secret, refresh_exp_secs)?;

    // Store refresh token hash
    let token_hash = hash_token(&refresh_token);
    let expires_at = Utc::now() + chrono::Duration::seconds(refresh_exp_secs);
    refresh_token_repo::create(pool, &token_hash, user.id, expires_at).await?;

    Ok(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".into(),
        must_change_password: user.must_change_password,
        requires_2fa: Some(false),
        temp_token: None,
    })
}

/// Create a JWT with client_type = "app"
fn create_app_jwt(user: &User, secret: &str, exp_secs: i64) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = now.timestamp() + exp_secs;
    let jti = uuid::Uuid::new_v4().to_string();

    let claims = AppClaims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role: user.role.clone(),
        role_id: user.role_id.map(|id| id.to_string()),
        department_id: user.department_id.map(|id| id.to_string()),
        jti,
        exp,
        two_factor_pending: None,
        client_type: Some("app".to_string()),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create app token: {}", e)))
}

/// Generate a random password for OAuth-created users.
/// Public for use by other services.
pub fn generate_random_password_display() -> String {
    generate_random_password()
}

/// Generate a random password for OAuth-created users
fn generate_random_password() -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    let bytes: [u8; 32] = rand::random();
    URL_SAFE_NO_PAD.encode(bytes)
}
