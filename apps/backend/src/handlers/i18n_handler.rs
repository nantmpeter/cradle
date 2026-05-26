use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use std::sync::Arc;

use crate::AppState;

/// GET /api/i18n/messages — Get backend error messages in user's language
pub async fn get_messages(
    State(_state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Json<serde_json::Value> {
    let lang_header = headers
        .get("accept-language")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("en");

    let lang = crate::models::i18n::Lang::from_header(lang_header);

    // Return all error messages for the detected language
    let messages = serde_json::json!({
        "auth.invalid_credentials": crate::models::i18n::t("auth.invalid_credentials", lang),
        "auth.account_disabled": crate::models::i18n::t("auth.account_disabled", lang),
        "auth.account_locked": crate::models::i18n::t("auth.account_locked", lang),
        "auth.token_expired": crate::models::i18n::t("auth.token_expired", lang),
        "auth.unauthorized": crate::models::i18n::t("auth.unauthorized", lang),
        "auth.forbidden": crate::models::i18n::t("auth.forbidden", lang),
        "file.not_found": crate::models::i18n::t("file.not_found", lang),
        "file.too_large": crate::models::i18n::t("file.too_large", lang),
        "file.upload_failed": crate::models::i18n::t("file.upload_failed", lang),
        "session.not_found": crate::models::i18n::t("session.not_found", lang),
        "menu.not_found": crate::models::i18n::t("menu.not_found", lang),
    });

    Json(
        serde_json::json!({ "data": messages, "lang": format!("{:?}", lang).to_lowercase() }),
    )
}
