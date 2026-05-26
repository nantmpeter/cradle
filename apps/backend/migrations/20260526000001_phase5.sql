-- ═══════════════════════════════════════════════════════════════
-- 1. users 表扩展字段
-- ═══════════════════════════════════════════════════════════════
ALTER TABLE users ADD COLUMN IF NOT EXISTS totp_secret TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS two_factor_enabled BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE users ADD COLUMN IF NOT EXISTS recovery_codes JSONB DEFAULT '[]'::jsonb;

-- ═══════════════════════════════════════════════════════════════
-- 2. notifications 表
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    category VARCHAR(50) NOT NULL DEFAULT 'system',
    is_pinned BOOLEAN NOT NULL DEFAULT false,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notifications_user_id ON notifications(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_notifications_created_at ON notifications(created_at DESC);
CREATE INDEX idx_notifications_category ON notifications(category);

-- ═══════════════════════════════════════════════════════════════
-- 3. notification_reads 表（已读记录）
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE notification_reads (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notification_id UUID NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
    read_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, notification_id)
);

CREATE INDEX idx_notification_reads_user ON notification_reads(user_id);

-- ═══════════════════════════════════════════════════════════════
-- 4. system_configs 表
-- ═══════════════════════════════════════════════════════════════
CREATE TABLE system_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_key VARCHAR(100) NOT NULL,
    config_key VARCHAR(100) NOT NULL,
    value TEXT NOT NULL,
    value_type VARCHAR(20) NOT NULL DEFAULT 'string',
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(group_key, config_key)
);

CREATE INDEX idx_system_configs_group ON system_configs(group_key);

-- 预置配置
INSERT INTO system_configs (group_key, config_key, value, value_type, description) VALUES
    ('general', 'site_name', 'Cradle', 'string', '站点名称'),
    ('general', 'site_description', 'Administration Panel', 'string', '站点描述'),
    ('email', 'smtp_host', '', 'string', 'SMTP 服务器地址'),
    ('email', 'smtp_port', '587', 'number', 'SMTP 端口'),
    ('email', 'smtp_user', '', 'string', 'SMTP 用户名'),
    ('email', 'smtp_from', '', 'string', '发件人地址'),
    ('security', 'session_timeout', '30', 'number', '会话超时时间(分钟)'),
    ('security', 'max_login_attempts', '5', 'number', '最大登录尝试次数'),
    ('security', 'password_min_length', '8', 'number', '密码最小长度'),
    ('storage', 'max_upload_size', '10485760', 'number', '最大上传大小(字节)'),
    ('storage', 'allowed_extensions', 'jpg,jpeg,png,gif,pdf,doc,docx,xlsx,csv', 'string', '允许的文件扩展名');

-- ═══════════════════════════════════════════════════════════════
-- 5. 新增权限种子
-- ═══════════════════════════════════════════════════════════════
INSERT INTO permissions (name, description, module) VALUES
    ('2fa:manage', 'Manage two-factor authentication settings', 'auth'),
    ('notifications:read', 'View notifications', 'notifications'),
    ('notifications:send', 'Send announcements', 'notifications'),
    ('configs:read', 'View system configuration', 'system'),
    ('configs:update', 'Update system configuration', 'system');

-- superadmin 授予所有新权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT (SELECT id FROM roles WHERE name = 'superadmin'), id
FROM permissions WHERE name IN ('2fa:manage', 'notifications:read', 'notifications:send', 'configs:read', 'configs:update');

-- admin 授予部分新权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT (SELECT id FROM roles WHERE name = 'admin'), id
FROM permissions WHERE name IN ('2fa:manage', 'notifications:read', 'notifications:send', 'configs:read');

-- user 基础权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT (SELECT id FROM roles WHERE name = 'user'), id
FROM permissions WHERE name IN ('notifications:read');
