use serde::{Deserialize, Serialize};

/// POST /api/auth/2fa/setup — no body required (user extracted from auth)
/// Response for 2FA setup: QR code + secret + recovery codes
#[derive(Debug, Serialize)]
pub struct Setup2FaResponse {
    pub qr_code_base64: String,
    pub secret: String,
    pub recovery_codes: Vec<String>,
}

/// POST /api/auth/2fa/enable — confirm 2FA with TOTP code
#[derive(Debug, Deserialize)]
pub struct Enable2FaRequest {
    pub code: String,
}

/// POST /api/auth/2fa/disable — disable 2FA (requires password or TOTP code)
#[derive(Debug, Deserialize)]
pub struct Disable2FaRequest {
    pub password: Option<String>,
    pub code: Option<String>,
}

/// Verify 2FA during login (with temp token)
#[derive(Debug, Deserialize)]
pub struct Verify2FaRequest {
    pub temp_token: String,
    pub code: Option<String>,
    pub recovery_code: Option<String>,
}
