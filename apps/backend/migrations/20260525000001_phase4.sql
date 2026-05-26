-- Phase 4 迁移文件

-- 1. 用户表新增头像字段
ALTER TABLE users ADD COLUMN IF NOT EXISTS avatar_url VARCHAR(500);

-- 2. 文件管理表
CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    original_name VARCHAR(255) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    size BIGINT NOT NULL,
    storage_path VARCHAR(500) NOT NULL,
    storage_backend VARCHAR(20) NOT NULL DEFAULT 'local',
    thumbnail_path VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_files_user_id ON files(user_id);
CREATE INDEX idx_files_mime_type ON files(mime_type);

-- 3. 会话表
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token_hash VARCHAR(255) NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);

-- 4. 菜单表
CREATE TABLE menus (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID REFERENCES menus(id) ON DELETE CASCADE,
    title_key VARCHAR(100) NOT NULL,
    title_label VARCHAR(100) NOT NULL,
    path VARCHAR(200) NOT NULL,
    icon VARCHAR(50),
    sort_order INT NOT NULL DEFAULT 0,
    permission_id UUID REFERENCES permissions(id) ON DELETE SET NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_menus_title_key UNIQUE (title_key),
    CONSTRAINT chk_menus_status CHECK (status IN ('active', 'disabled'))
);
CREATE INDEX idx_menus_parent_id ON menus(parent_id);
CREATE INDEX idx_menus_sort_order ON menus(sort_order);

-- 5. 新增权限种子数据
INSERT INTO permissions (name, description, module) VALUES
    ('files:read', 'View file list', 'files'),
    ('files:upload', 'Upload files', 'files'),
    ('files:delete', 'Delete files', 'files'),
    ('export:users', 'Export user list', 'export'),
    ('export:audit', 'Export audit logs', 'export'),
    ('export:roles', 'Export role list', 'export'),
    ('sessions:read', 'View all sessions', 'sessions'),
    ('sessions:manage', 'Terminate sessions', 'sessions'),
    ('menus:read', 'View menu list', 'menus'),
    ('menus:create', 'Create menu items', 'menus'),
    ('menus:update', 'Update menu items', 'menus'),
    ('menus:delete', 'Delete menu items', 'menus')
ON CONFLICT (name) DO NOTHING;

-- 6. 分配新权限给 superadmin
INSERT INTO role_permissions (role_id, permission_id)
SELECT (SELECT id FROM roles WHERE name = 'superadmin'), id
FROM permissions WHERE module IN ('files', 'export', 'sessions', 'menus')
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- 7. 分配部分新权限给 admin
INSERT INTO role_permissions (role_id, permission_id)
SELECT (SELECT id FROM roles WHERE name = 'admin'), id
FROM permissions WHERE name IN (
    'files:read', 'files:upload', 'files:delete',
    'export:users', 'export:audit', 'export:roles',
    'sessions:read', 'sessions:manage',
    'menus:read'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- 8. 分配基础权限给 user
INSERT INTO role_permissions (role_id, permission_id)
SELECT (SELECT id FROM roles WHERE name = 'user'), id
FROM permissions WHERE name IN ('files:upload')
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- 9. 菜单种子数据
INSERT INTO menus (title_key, title_label, path, icon, sort_order, permission_id) VALUES
    ('menu.dashboard', 'Dashboard', '/dashboard', 'LayoutDashboard', 1, NULL),
    ('menu.users', 'Users', '/dashboard/users', 'Users', 2, (SELECT id FROM permissions WHERE name = 'users:read')),
    ('menu.roles', 'Roles', '/dashboard/roles', 'Shield', 3, (SELECT id FROM permissions WHERE name = 'roles:read')),
    ('menu.auditLogs', 'Audit Logs', '/dashboard/audit-logs', 'ScrollText', 4, (SELECT id FROM permissions WHERE name = 'audit:read')),
    ('menu.files', 'Files', '/dashboard/files', 'FolderOpen', 5, (SELECT id FROM permissions WHERE name = 'files:read')),
    ('menu.sessions', 'Sessions', '/dashboard/sessions', 'Monitor', 6, (SELECT id FROM permissions WHERE name = 'sessions:read')),
    ('menu.profile', 'Profile', '/dashboard/profile', 'UserCircle', 99, NULL),
    ('menu.settings', 'Settings', '/dashboard/settings', 'Settings', 100, NULL);
