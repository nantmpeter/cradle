use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

// ─── Role Enum ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Admin,
    SuperAdmin,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::User => "user",
            Role::Admin => "admin",
            Role::SuperAdmin => "superadmin",
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Role {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(Role::User),
            "admin" => Ok(Role::Admin),
            "superadmin" => Ok(Role::SuperAdmin),
            _ => Err(format!("Unknown role: {}", s)),
        }
    }
}

// ─── UserStatus Enum ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Disabled,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserStatus::Active => "active",
            UserStatus::Disabled => "disabled",
        }
    }
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for UserStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(UserStatus::Active),
            "disabled" => Ok(UserStatus::Disabled),
            _ => Err(format!("Unknown status: {}", s)),
        }
    }
}

// ─── Database Models ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub name: Option<String>,
    pub role: String,
    pub role_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub status: String,
    pub must_change_password: bool,
    pub totp_secret: Option<String>,
    pub two_factor_enabled: bool,
    pub recovery_codes: Option<Value>,
    pub login_failures: Option<i32>,
    pub locked_until: Option<DateTime<Utc>>,
    pub avatar_url: Option<String>,
    pub phone: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Public user response (no password_hash)
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub role: Role,
    pub role_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub avatar_url: Option<String>,
    pub status: UserStatus,
    pub must_change_password: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            name: u.name,
            role: u.role.parse().unwrap_or(Role::User),
            role_id: u.role_id,
            department_id: u.department_id,
            avatar_url: u.avatar_url,
            status: u.status.parse().unwrap_or(UserStatus::Active),
            must_change_password: u.must_change_password,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

/// Me response with permissions included (for GET /api/users/me)
#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub role: Role,
    pub role_id: Uuid,
    pub avatar_url: Option<String>,
    pub status: UserStatus,
    pub must_change_password: bool,
    pub two_factor_enabled: bool,
    pub department_id: Option<Uuid>,
    pub permissions: Vec<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

// ─── Refresh Token Model ────────────────────────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub token_hash: String,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ─── Request / Response Types ───────────────────────────────────────────────

/// Login request
#[derive(Debug, Deserialize, validator::Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}

/// Register request
#[derive(Debug, Deserialize, validator::Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
}

/// Token response
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub must_change_password: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_2fa: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_token: Option<String>,
}

/// Refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Create user request (admin+)
#[derive(Debug, Deserialize, validator::Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub role: Role,
    pub role_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
}

/// Update user request
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub role: Option<Role>,
    pub role_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
}

/// Update status request (admin+)
#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: UserStatus,
}

/// Change my password request (authenticated)
#[derive(Debug, Deserialize, validator::Validate)]
pub struct ChangeMyPasswordRequest {
    #[validate(length(min = 1))]
    pub current_password: String,
    #[validate(length(min = 8))]
    pub new_password: String,
}

/// Reset password request (admin+)
#[derive(Debug, Deserialize, validator::Validate)]
pub struct ResetPasswordRequest {
    #[validate(length(min = 8))]
    pub new_password: String,
}

// ─── Password Validation ────────────────────────────────────────────────────

/// Validate password strength:
/// - At least 8 characters
/// - Must contain at least 3 of: uppercase, lowercase, digit, special character
pub fn validate_password_strength(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters".into());
    }

    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    let category_count = [has_uppercase, has_lowercase, has_digit, has_special]
        .iter()
        .filter(|&&x| x)
        .count();

    if category_count < 3 {
        return Err(
            "Password must contain at least 3 of: uppercase letter, lowercase letter, digit, special character"
                .into(),
        );
    }

    Ok(())
}
