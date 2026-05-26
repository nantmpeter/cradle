use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::AppError;
use crate::models::user::{
    validate_password_strength, LoginRequest, RegisterRequest, Role, TokenResponse, User,
    UserResponse, UserStatus,
};
use crate::repository::{refresh_token_repo, token_blacklist_repo, user_repo};
use sqlx::PgPool;

/// Max login failures before lockout
const MAX_LOGIN_FAILURES: i32 = 5;
/// Lockout duration in minutes
const LOCK_DURATION_MINS: i64 = 15;
/// 2FA temp token TTL in seconds
const TWO_FA_TEMP_TOKEN_SECS: i64 = 300; // 5 minutes

#[derive(Debug, Serialize)]
struct Claims {
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
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Hash a password using Argon2
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to hash password: {}", e)))
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid password hash: {}", e)))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Create a JWT token for the given user with jti and role_id
fn create_token(user: &User, secret: &str, exp_secs: i64) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = now.timestamp() + exp_secs;
    let jti = uuid::Uuid::new_v4().to_string();

    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role: user.role.clone(),
        role_id: user.role_id.map(|id| id.to_string()),
        department_id: user.department_id.map(|id| id.to_string()),
        jti,
        exp,
        two_factor_pending: None,
        client_type: Some("admin".to_string()),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create token: {}", e)))
}

/// Create a temporary 2FA token (5min TTL, with two_factor_pending claim)
fn create_temp_2fa_token(user: &User, secret: &str) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = now.timestamp() + TWO_FA_TEMP_TOKEN_SECS;
    let jti = uuid::Uuid::new_v4().to_string();

    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role: user.role.clone(),
        role_id: user.role_id.map(|id| id.to_string()),
        department_id: user.department_id.map(|id| id.to_string()),
        jti,
        exp,
        two_factor_pending: Some(true),
        client_type: Some("admin".to_string()),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create temp token: {}", e)))
}

/// Get user permissions by user_id (helper for handlers)
pub async fn get_user_permissions(pool: &PgPool, user_id: uuid::Uuid) -> Result<Vec<String>, AppError> {
    crate::repository::permission_repo::find_by_user_id(pool, user_id).await
}

/// Register a new user (public). Role is forced to "user".
pub async fn register(
    pool: &PgPool,
    req: &RegisterRequest,
) -> Result<UserResponse, AppError> {
    // Validate password strength
    validate_password_strength(&req.password)
        .map_err(|e| AppError::BadRequest(e))?;

    // Check if email already exists
    if user_repo::find_by_email(pool, &req.email).await?.is_some() {
        return Err(AppError::Conflict("Email already registered".into()));
    }

    // Hash password
    let password_hash = hash_password(&req.password)?;

    // Look up the "user" role_id
    let user_role = crate::repository::role_repo::find_by_name(pool, "user").await?;
    let role_id = user_role.as_ref().map(|r| r.id);

    // Insert user with forced role=user
    let user = user_repo::create_user(
        pool,
        &req.email,
        &password_hash,
        req.name.as_deref(),
        Role::User.as_str(),
        UserStatus::Active.as_str(),
        false,
        role_id,
        None, // department_id: not set on registration
    )
    .await?;

    Ok(UserResponse::from(user))
}

/// Login: verify credentials, check lockout, check status, create and store refresh token
pub async fn login(
    pool: &PgPool,
    jwt_secret: &str,
    access_exp_secs: i64,
    refresh_exp_secs: i64,
    req: &LoginRequest,
) -> Result<TokenResponse, AppError> {
    let user = user_repo::find_by_email(pool, &req.email)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".into()))?;

    // Check lockout status
    if let Some(locked_until) = user.locked_until {
        if locked_until > Utc::now() {
            let remaining_mins = (locked_until.timestamp() - Utc::now().timestamp()) / 60;
            return Err(AppError::AccountLocked(format!(
                "Account locked due to too many failed attempts, retry after {} minutes",
                remaining_mins.max(1)
            )));
        }
        // Lock expired, allow attempt
    }

    // Check account status
    if user.status == UserStatus::Disabled.as_str() {
        return Err(AppError::AccountDisabled(
            "Account has been disabled".into(),
        ));
    }

    // Verify password
    if !verify_password(&req.password, &user.password_hash)? {
        // Increment login failures
        let failures = user_repo::increment_login_failures(pool, user.id).await?;

        if failures >= MAX_LOGIN_FAILURES {
            // Lock the account
            let locked_until = Utc::now() + chrono::Duration::minutes(LOCK_DURATION_MINS);
            user_repo::set_locked_until(pool, user.id, locked_until).await?;

            // Audit log for account lockout
            if let Err(e) = crate::repository::audit_log_repo::create(
                pool,
                Some(user.id),
                "auth.account_locked",
                Some("user"),
                Some(user.id),
                Some(serde_json::json!({
                    "locked_until": locked_until.to_rfc3339(),
                })),
                None,
                None,
            )
            .await
            {
                tracing::error!("Failed to write audit log for account lockout: {:?}", e);
            }

            return Err(AppError::AccountLocked(format!(
                "Account locked due to too many failed attempts, retry after {} minutes",
                LOCK_DURATION_MINS
            )));
        }

        let remaining = MAX_LOGIN_FAILURES - failures;

        // Audit log for failed login
        if let Err(e) = crate::repository::audit_log_repo::create(
            pool,
            Some(user.id),
            "auth.login_failed",
            Some("user"),
            Some(user.id),
            Some(serde_json::json!({
                "reason": "wrong_password",
                "failures": failures,
            })),
            None,
            None,
        )
        .await
        {
            tracing::error!("Failed to write audit log for login failure: {:?}", e);
        }

        return Err(AppError::Unauthorized(format!(
            "Invalid email or password. {} attempts remaining",
            remaining
        )));
    }

    // Reset login failures on success
    user_repo::reset_login_failures(pool, user.id).await?;

    // Check if user has 2FA enabled
    if user.two_factor_enabled {
        // Generate temp token instead of real tokens
        let temp_token = create_temp_2fa_token(&user, jwt_secret)?;

        // Audit log for 2FA pending login
        if let Err(e) = crate::repository::audit_log_repo::create(
            pool,
            Some(user.id),
            "auth.login_2fa_pending",
            Some("user"),
            Some(user.id),
            Some(serde_json::json!({"success": true, "2fa_required": true})),
            None,
            None,
        )
        .await
        {
            tracing::error!("Failed to write audit log for 2FA pending: {:?}", e);
        }

        return Ok(TokenResponse {
            access_token: String::new(),
            refresh_token: String::new(),
            token_type: "Bearer".into(),
            must_change_password: user.must_change_password,
            requires_2fa: Some(true),
            temp_token: Some(temp_token),
        });
    }

    // Generate access token
    let access_token = create_token(&user, jwt_secret, access_exp_secs)?;

    // Generate refresh token
    let refresh_token = create_token(&user, jwt_secret, refresh_exp_secs)?;

    // Hash and store refresh token
    let token_hash = hash_token(&refresh_token);
    let expires_at = Utc::now() + chrono::Duration::seconds(refresh_exp_secs);
    refresh_token_repo::create(pool, &token_hash, user.id, expires_at).await?;

    // Audit log for successful login
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(user.id),
        "auth.login",
        Some("user"),
        Some(user.id),
        Some(serde_json::json!({"success": true})),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for login success: {:?}", e);
    }

    Ok(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".into(),
        must_change_password: user.must_change_password,
        requires_2fa: Some(false),
        temp_token: None,
    })
}

/// Refresh: validate refresh token, check user status, rotate tokens
pub async fn refresh(
    pool: &PgPool,
    jwt_secret: &str,
    access_exp_secs: i64,
    refresh_exp_secs: i64,
    refresh_token_str: &str,
) -> Result<TokenResponse, AppError> {
    // Hash the incoming token and look it up
    let token_hash = hash_token(refresh_token_str);
    let stored_token = refresh_token_repo::find_by_hash(pool, &token_hash)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid refresh token".into()))?;

    // Check if already revoked
    if stored_token.revoked_at.is_some() {
        return Err(AppError::Unauthorized("Refresh token has been revoked".into()));
    }

    // Check if expired
    if stored_token.expires_at < Utc::now() {
        return Err(AppError::Unauthorized("Refresh token has expired".into()));
    }

    // Look up the user
    let user = user_repo::find_by_id(pool, stored_token.user_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

    // Check account status
    if user.status == UserStatus::Disabled.as_str() {
        return Err(AppError::AccountDisabled(
            "Account has been disabled".into(),
        ));
    }

    // Revoke the old refresh token
    refresh_token_repo::revoke(pool, stored_token.id).await?;

    // Generate new token pair
    let access_token = create_token(&user, jwt_secret, access_exp_secs)?;
    let new_refresh_token = create_token(&user, jwt_secret, refresh_exp_secs)?;

    // Store new refresh token hash
    let new_token_hash = hash_token(&new_refresh_token);
    let expires_at = Utc::now() + chrono::Duration::seconds(refresh_exp_secs);
    refresh_token_repo::create(pool, &new_token_hash, user.id, expires_at).await?;

    Ok(TokenResponse {
        access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".into(),
        must_change_password: user.must_change_password,
        requires_2fa: Some(false),
        temp_token: None,
    })
}

/// Logout: revoke refresh token and add access token jti to blacklist
pub async fn logout(
    pool: &PgPool,
    refresh_token_str: &str,
    access_jti: uuid::Uuid,
    access_expires_at: chrono::DateTime<chrono::Utc>,
) -> Result<(), AppError> {
    // Revoke the refresh token (ignore if not found or already revoked)
    let token_hash = hash_token(refresh_token_str);
    if let Ok(Some(stored_token)) = refresh_token_repo::find_by_hash(pool, &token_hash).await {
        if stored_token.revoked_at.is_none() {
            let _ = refresh_token_repo::revoke(pool, stored_token.id).await;
        }
    }

    // Add access token jti to blacklist
    if !access_jti.is_nil() {
        token_blacklist_repo::add(pool, access_jti, access_expires_at).await?;
    }

    Ok(())
}

/// Verify 2FA login: validate temp token, verify TOTP or recovery code, issue real tokens
pub async fn verify_2fa_login(
    pool: &PgPool,
    jwt_secret: &str,
    access_exp_secs: i64,
    refresh_exp_secs: i64,
    totp_encryption_key: &str,
    temp_token: &str,
    code: Option<&str>,
    recovery_code: Option<&str>,
) -> Result<TokenResponse, AppError> {
    // Decode and validate the temp token
    let decoding_key = jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes());
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);

    let token_data = jsonwebtoken::decode::<serde_json::Value>(temp_token, &decoding_key, &validation)
        .map_err(|e| AppError::Unauthorized(format!("Invalid temp token: {}", e)))?;

    // Check two_factor_pending claim
    let is_2fa_pending = token_data
        .claims
        .get("two_factor_pending")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !is_2fa_pending {
        return Err(AppError::BadRequest("Not a 2FA pending token".into()));
    }

    let user_id_str = token_data
        .claims
        .get("sub")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Unauthorized("Invalid token claims".into()))?;

    let user_id = uuid::Uuid::parse_str(user_id_str)
        .map_err(|_| AppError::Unauthorized("Invalid user id in token".into()))?;

    // Load user
    let user = user_repo::find_by_id(pool, user_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

    if !user.two_factor_enabled {
        return Err(AppError::BadRequest("2FA is not enabled for this user".into()));
    }

    // Verify either TOTP code or recovery code
    let mut verified = false;
    let mut used_recovery = false;

    if let Some(totp_code) = code {
        if let Some(ref encrypted_secret) = user.totp_secret {
            let secret = crate::services::two_factor_service::decrypt_secret(
                encrypted_secret,
                totp_encryption_key,
            )?;
            verified = crate::services::two_factor_service::verify_totp(&secret, totp_code)?;
        }
    }

    if !verified {
        if let Some(rc) = recovery_code {
            verified = crate::services::two_factor_service::verify_recovery_code(
                pool, user_id, rc,
            )
            .await?;
            used_recovery = true;
        }
    }

    if !verified {
        // Audit failed 2FA attempt
        if let Err(e) = crate::repository::audit_log_repo::create(
            pool,
            Some(user_id),
            "auth.2fa_verify_failed",
            Some("user"),
            Some(user_id),
            Some(serde_json::json!({"method": if code.is_some() { "totp" } else { "recovery_code" }})),
            None,
            None,
        )
        .await
        {
            tracing::error!("Failed to write audit log: {:?}", e);
        }

        return Err(AppError::Unauthorized(
            "Invalid TOTP code or recovery code".into(),
        ));
    }

    // Generate real tokens
    let access_token = create_token(&user, jwt_secret, access_exp_secs)?;
    let refresh_token = create_token(&user, jwt_secret, refresh_exp_secs)?;

    // Hash and store refresh token
    let token_hash = hash_token(&refresh_token);
    let expires_at = Utc::now() + chrono::Duration::seconds(refresh_exp_secs);
    refresh_token_repo::create(pool, &token_hash, user.id, expires_at).await?;

    // Audit successful 2FA login
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(user_id),
        "auth.2fa_verified",
        Some("user"),
        Some(user_id),
        Some(serde_json::json!({
            "method": if used_recovery { "recovery_code" } else { "totp" }
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for 2FA verify: {:?}", e);
    }

    Ok(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".into(),
        must_change_password: user.must_change_password,
        requires_2fa: Some(false),
        temp_token: None,
    })
}
