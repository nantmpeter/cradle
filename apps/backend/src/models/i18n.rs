use std::collections::HashMap;
use std::sync::LazyLock;

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    En,
    Zh,
}

impl Lang {
    pub fn from_header(header: &str) -> Self {
        if header.to_lowercase().starts_with("zh") {
            Lang::Zh
        } else {
            Lang::En
        }
    }
}

/// Backend error message translations (only ~11 messages)
static TRANSLATIONS: LazyLock<HashMap<&'static str, HashMap<Lang, &'static str>>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        let mut insert = |key: &'static str, en: &'static str, zh: &'static str| {
            let mut inner = HashMap::new();
            inner.insert(Lang::En, en);
            inner.insert(Lang::Zh, zh);
            map.insert(key, inner);
        };

        insert("auth.invalid_credentials", "Invalid email or password", "邮箱或密码错误");
        insert("auth.account_disabled", "Account is disabled", "账号已禁用");
        insert("auth.account_locked", "Account is temporarily locked", "账号已被临时锁定");
        insert("auth.token_expired", "Token has expired", "令牌已过期");
        insert("auth.unauthorized", "Authentication required", "需要登录");
        insert("auth.forbidden", "Access denied", "权限不足");
        insert("file.not_found", "File not found", "文件不存在");
        insert("file.too_large", "File size exceeds limit", "文件大小超出限制");
        insert("file.upload_failed", "File upload failed", "文件上传失败");
        insert("session.not_found", "Session not found", "会话不存在");
        insert("menu.not_found", "Menu item not found", "菜单项不存在");

        map
    });

/// Translate a backend message key to the user's language.
/// Returns the key itself as fallback if no translation found.
pub fn t(key: &'static str, lang: Lang) -> &'static str {
    TRANSLATIONS
        .get(key)
        .and_then(|translations| translations.get(&lang).copied())
        .unwrap_or(key)
}
