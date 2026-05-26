use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use qrcode::QrCode;
use sqlx::PgPool;
use totp_rs::{Algorithm, TOTP, Secret};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::two_factor::Setup2FaResponse;
use crate::repository::{two_factor_repo, user_repo};

/// AES-256-GCM nonce size in bytes
const NONCE_SIZE: usize = 12;

/// Encrypt a TOTP secret using AES-256-GCM.
/// Returns base64( nonce || ciphertext ).
pub fn encrypt_secret(plaintext: &str, key_base64: &str) -> Result<String, AppError> {
    let key_bytes = BASE64
        .decode(key_base64)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid TOTP encryption key: {}", e)))?;

    if key_bytes.len() != 32 {
        return Err(AppError::Internal(anyhow::anyhow!(
            "TOTP encryption key must be 32 bytes, got {}",
            key_bytes.len()
        )));
    }

    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create cipher: {}", e)))?;

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Encryption failed: {}", e)))?;

    let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&combined))
}

/// Decrypt a TOTP secret encrypted by encrypt_secret.
pub fn decrypt_secret(encrypted: &str, key_base64: &str) -> Result<String, AppError> {
    let key_bytes = BASE64
        .decode(key_base64)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid TOTP encryption key: {}", e)))?;

    if key_bytes.len() != 32 {
        return Err(AppError::Internal(anyhow::anyhow!(
            "TOTP encryption key must be 32 bytes, got {}",
            key_bytes.len()
        )));
    }

    let combined = BASE64
        .decode(encrypted)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid encrypted secret: {}", e)))?;

    if combined.len() < NONCE_SIZE {
        return Err(AppError::Internal(anyhow::anyhow!(
            "Encrypted secret too short"
        )));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create cipher: {}", e)))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Decryption failed: {}", e)))?;

    String::from_utf8(plaintext)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid UTF-8 in secret: {}", e)))
}

/// Generate a new random TOTP secret (base32 encoded)
fn generate_secret() -> String {
    let secret = Secret::generate_secret();
    secret.to_encoded().to_string()
}

/// Generate QR code as SVG base64 data URI
pub fn generate_qr_base64(email: &str, secret: &str, issuer: &str) -> Result<String, AppError> {
    let secret_bytes = Secret::Encoded(secret.to_string())
        .to_bytes()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid secret: {}", e)))?;

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some(issuer.to_string()),
        email.to_string(),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create TOTP: {}", e)))?;

    let url = totp.get_url();
    let code = QrCode::new(url.as_bytes())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to generate QR: {}", e)))?;

    let svg = code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(200, 200)
        .build();

    let b64 = BASE64.encode(svg.as_bytes());
    Ok(format!("data:image/svg+xml;base64,{}", b64))
}

/// Verify a TOTP code against a secret
pub fn verify_totp(secret: &str, code: &str) -> Result<bool, AppError> {
    let secret_bytes = Secret::Encoded(secret.to_string())
        .to_bytes()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid secret: {}", e)))?;

    let totp = TOTP::new_unchecked(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        None,
        String::new(),
    );

    Ok(totp.check_current(code).unwrap_or(false))
}

/// Generate 10 random recovery codes (hex strings)
pub fn generate_recovery_codes() -> Vec<String> {
    let mut rng = rand::rng();
    (0..10)
        .map(|_| {
            let mut bytes = [0u8; 6];
            rand::Rng::fill(&mut rng, &mut bytes);
            bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>()
        })
        .collect()
}

/// Hash a recovery code using Argon2
fn hash_recovery_code(code: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(code.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to hash recovery code: {}", e)))
}

/// Verify a recovery code against stored hashes.
/// Returns the index of the matched code if found.
fn verify_recovery_code_hash(
    codes: &[serde_json::Value],
    input: &str,
) -> Result<Option<usize>, AppError> {
    for (i, entry) in codes.iter().enumerate() {
        let used = entry.get("used").and_then(|v| v.as_bool()).unwrap_or(false);
        if used {
            continue;
        }
        if let Some(hash) = entry.get("hash").and_then(|v| v.as_str()) {
            let parsed = PasswordHash::new(hash)
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid hash: {}", e)))?;
            if Argon2::default()
                .verify_password(input.as_bytes(), &parsed)
                .is_ok()
            {
                return Ok(Some(i));
            }
        }
    }
    Ok(None)
}

/// Setup 2FA for a user: generate secret, encrypt & store, generate QR and recovery codes
pub async fn setup_2fa(
    pool: &PgPool,
    user_id: Uuid,
    _email: &str,
    encryption_key: &str,
) -> Result<Setup2FaResponse, AppError> {
    // Check user exists
    let user = user_repo::find_by_id(pool, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    if user.two_factor_enabled {
        return Err(AppError::BadRequest(
            "2FA is already enabled. Disable it first.".into(),
        ));
    }

    // Generate secret
    let secret = generate_secret();

    // Encrypt and store secret
    let encrypted = encrypt_secret(&secret, encryption_key)?;
    two_factor_repo::save_totp_secret(pool, user_id, &encrypted).await?;

    // Generate QR code
    let qr_base64 = generate_qr_base64(&user.email, &secret, "Cradle")?;

    // Generate recovery codes and hash them
    let plain_codes = generate_recovery_codes();
    let hashed_codes: Vec<serde_json::Value> = plain_codes
        .iter()
        .map(|code| {
            let hash = hash_recovery_code(code).expect("hash should not fail");
            serde_json::json!({
                "hash": hash,
                "used": false
            })
        })
        .collect();

    // Store hashed recovery codes
    two_factor_repo::save_recovery_codes(
        pool,
        user_id,
        serde_json::Value::Array(hashed_codes),
    )
    .await?;

    Ok(Setup2FaResponse {
        qr_code_base64: qr_base64,
        secret,
        recovery_codes: plain_codes,
    })
}

/// Enable 2FA: verify TOTP code to confirm setup
pub async fn enable_2fa(
    pool: &PgPool,
    user_id: Uuid,
    code: &str,
    encryption_key: &str,
) -> Result<(), AppError> {
    let encrypted_secret = two_factor_repo::get_totp_secret(pool, user_id)
        .await?
        .ok_or_else(|| AppError::BadRequest("2FA not set up. Run setup first.".into()))?;

    let secret = decrypt_secret(&encrypted_secret, encryption_key)?;

    if !verify_totp(&secret, code)? {
        return Err(AppError::BadRequest("Invalid TOTP code".into()));
    }

    two_factor_repo::set_2fa_enabled(pool, user_id, true).await?;

    // Audit log
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(user_id),
        "auth.2fa_enabled",
        Some("user"),
        Some(user_id),
        None,
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for 2FA enable: {:?}", e);
    }

    Ok(())
}

/// Disable 2FA for the current user
pub async fn disable_2fa(
    pool: &PgPool,
    user_id: Uuid,
    password: Option<&str>,
    code: Option<&str>,
    encryption_key: &str,
) -> Result<(), AppError> {
    let user = user_repo::find_by_id(pool, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    // Verify either password or TOTP code
    let mut verified = false;

    if let Some(pwd) = password {
        let hash = &user.password_hash;
        let parsed = argon2::PasswordHash::new(hash)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid hash: {}", e)))?;
        verified = argon2::Argon2::default()
            .verify_password(pwd.as_bytes(), &parsed)
            .is_ok();
    }

    if !verified {
        if let Some(totp_code) = code {
            if let Some(ref encrypted_secret) = user.totp_secret {
                let secret = decrypt_secret(encrypted_secret, encryption_key)?;
                verified = verify_totp(&secret, totp_code)?;
            }
        }
    }

    if !verified {
        return Err(AppError::BadRequest(
            "Password or TOTP code verification failed".into(),
        ));
    }

    two_factor_repo::reset_2fa(pool, user_id).await?;

    // Audit log
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(user_id),
        "auth.2fa_disabled",
        Some("user"),
        Some(user_id),
        None,
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for 2FA disable: {:?}", e);
    }

    Ok(())
}

/// Verify recovery code for a user. Returns true if valid, marks code as used.
pub async fn verify_recovery_code(
    pool: &PgPool,
    user_id: Uuid,
    input_code: &str,
) -> Result<bool, AppError> {
    let codes_json = two_factor_repo::get_recovery_codes(pool, user_id)
        .await?
        .ok_or_else(|| AppError::BadRequest("No recovery codes found".into()))?;

    let mut codes: Vec<serde_json::Value> = codes_json
        .as_array()
        .cloned()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Invalid recovery codes format")))?;

    match verify_recovery_code_hash(&codes, input_code)? {
        Some(idx) => {
            // Mark as used
            codes[idx] = serde_json::json!({
                "hash": codes[idx].get("hash").and_then(|v| v.as_str()).unwrap_or(""),
                "used": true
            });
            two_factor_repo::save_recovery_codes(
                pool,
                user_id,
                serde_json::Value::Array(codes),
            )
            .await?;
            Ok(true)
        }
        None => Ok(false),
    }
}

/// Admin reset a user's 2FA
pub async fn reset_user_2fa(
    pool: &PgPool,
    admin_id: Uuid,
    target_user_id: Uuid,
) -> Result<(), AppError> {
    // Verify target user exists
    let _user = user_repo::find_by_id(pool, target_user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    two_factor_repo::reset_2fa(pool, target_user_id).await?;

    // Audit log
    if let Err(e) = crate::repository::audit_log_repo::create(
        pool,
        Some(admin_id),
        "auth.2fa_reset",
        Some("user"),
        Some(target_user_id),
        Some(serde_json::json!({"reset_by": "admin"})),
        None,
        None,
    )
    .await
    {
        tracing::error!("Failed to write audit log for 2FA reset: {:?}", e);
    }

    Ok(())
}
