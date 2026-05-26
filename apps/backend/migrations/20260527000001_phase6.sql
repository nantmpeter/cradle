-- ═══════════════════════════════════════════════════════════════
-- Phase 6: 部门管理、字典管理、登录日志、用户导入
-- ═══════════════════════════════════════════════════════════════

-- 1. departments 表
CREATE TABLE departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID REFERENCES departments(id) ON DELETE SET NULL,
    name VARCHAR(100) NOT NULL,
    code VARCHAR(100) UNIQUE NOT NULL,
    sort_order INT NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    leader VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_departments_parent_id ON departments(parent_id);
CREATE INDEX idx_departments_code ON departments(code);

-- 2. dict_types 表
CREATE TABLE dict_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    code VARCHAR(100) UNIQUE NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    remark TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 3. dict_items 表
CREATE TABLE dict_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dict_type_id UUID NOT NULL REFERENCES dict_types(id) ON DELETE CASCADE,
    label VARCHAR(200) NOT NULL,
    value VARCHAR(200) NOT NULL,
    sort_order INT NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    remark TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_dict_items_type_id ON dict_items(dict_type_id);

-- 4. login_logs 表
CREATE TABLE login_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    email VARCHAR(255),
    event VARCHAR(50) NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    os VARCHAR(100),
    browser VARCHAR(100),
    success BOOLEAN NOT NULL DEFAULT true,
    fail_reason TEXT,
    login_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_login_logs_user_id ON login_logs(user_id);
CREATE INDEX idx_login_logs_email ON login_logs(email);
CREATE INDEX idx_login_logs_event ON login_logs(event);
CREATE INDEX idx_login_logs_login_at ON login_logs(login_at DESC);

-- 5. users 表增加 department_id
ALTER TABLE users ADD COLUMN IF NOT EXISTS department_id UUID REFERENCES departments(id);

CREATE INDEX idx_users_department_id ON users(department_id);

-- ═══════════════════════════════════════════════════════════════
-- 6. 新增权限种子
-- ═══════════════════════════════════════════════════════════════
INSERT INTO permissions (name, description, module) VALUES
    ('departments:read', 'View department information', 'departments'),
    ('departments:create', 'Create departments', 'departments'),
    ('departments:update', 'Update departments', 'departments'),
    ('departments:delete', 'Delete departments', 'departments'),
    ('dicts:read', 'View dictionary data', 'dicts'),
    ('dicts:create', 'Create dictionary types and items', 'dicts'),
    ('dicts:update', 'Update dictionary types and items', 'dicts'),
    ('dicts:delete', 'Delete dictionary types and items', 'dicts'),
    ('login-logs:read', 'View login logs', 'audit'),
    ('users:import', 'Import users from Excel', 'users');

-- superadmin 授予所有新权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT (SELECT id FROM roles WHERE name = 'superadmin'), id
FROM permissions WHERE name IN (
    'departments:read', 'departments:create', 'departments:update', 'departments:delete',
    'dicts:read', 'dicts:create', 'dicts:update', 'dicts:delete',
    'login-logs:read', 'users:import'
);

-- admin 授予部分新权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT (SELECT id FROM roles WHERE name = 'admin'), id
FROM permissions WHERE name IN (
    'departments:read', 'departments:create', 'departments:update',
    'dicts:read', 'dicts:create', 'dicts:update',
    'login-logs:read', 'users:import'
);

-- ═══════════════════════════════════════════════════════════════
-- 7. 新增系统配置项种子
-- ═══════════════════════════════════════════════════════════════
INSERT INTO system_configs (group_key, config_key, value, value_type, description) VALUES
    ('general', 'logo_url', '', 'string', '站点 Logo URL'),
    ('security', 'allow_register', 'true', 'boolean', '是否允许自助注册'),
    ('login', 'captcha_enabled', 'false', 'boolean', '是否开启登录验证码'),
    ('login', 'force_2fa', 'false', 'boolean', '是否强制开启两步验证'),
    ('login', 'lockout_duration', '30', 'number', '登录失败锁定时长(分钟)')
ON CONFLICT (group_key, config_key) DO NOTHING;
