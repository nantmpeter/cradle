-- Phase 2: RBAC + Security tables
-- Creates: roles, permissions, role_permissions, audit_logs, token_blacklist
-- Modifies: users (adds role_id, login_failures, locked_until)

-- ============================================================
-- 1. Roles table
-- ============================================================
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================
-- 2. Permissions table
-- ============================================================
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    module VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================
-- 3. Role-Permissions junction table
-- ============================================================
CREATE TABLE role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- ============================================================
-- 4. Seed system roles
-- ============================================================
INSERT INTO roles (id, name, description, is_system) VALUES
    ('00000000-0000-0000-0000-000000000001', 'superadmin', 'Super Administrator with full system access', true),
    ('00000000-0000-0000-0000-000000000002', 'admin', 'Administrator with management access', true),
    ('00000000-0000-0000-0000-000000000003', 'user', 'Standard user with basic access', true);

-- ============================================================
-- 5. Seed permissions
-- ============================================================
INSERT INTO permissions (name, description, module) VALUES
    ('users:create', 'Create new users', 'users'),
    ('users:read', 'View user information', 'users'),
    ('users:update', 'Update user information', 'users'),
    ('users:delete', 'Delete users', 'users'),
    ('users:manage_status', 'Enable/disable user accounts', 'users'),
    ('users:reset_password', 'Reset user passwords', 'users'),
    ('roles:create', 'Create new roles', 'roles'),
    ('roles:read', 'View role information', 'roles'),
    ('roles:update', 'Update role information', 'roles'),
    ('roles:delete', 'Delete roles', 'roles'),
    ('roles:manage_permissions', 'Assign/revoke permissions from roles', 'roles'),
    ('audit:read', 'View audit logs', 'audit'),
    ('system:manage', 'Manage system settings', 'system');

-- ============================================================
-- 6. Assign all permissions to superadmin
-- ============================================================
INSERT INTO role_permissions (role_id, permission_id)
SELECT
    (SELECT id FROM roles WHERE name = 'superadmin'),
    id
FROM permissions;

-- Assign limited permissions to admin
INSERT INTO role_permissions (role_id, permission_id)
SELECT
    (SELECT id FROM roles WHERE name = 'admin'),
    id
FROM permissions
WHERE name IN (
    'users:create', 'users:read', 'users:update', 'users:delete',
    'users:manage_status', 'users:reset_password',
    'roles:read',
    'audit:read'
);

-- Assign basic permissions to user
INSERT INTO role_permissions (role_id, permission_id)
SELECT
    (SELECT id FROM roles WHERE name = 'user'),
    id
FROM permissions
WHERE name IN ('users:read');

-- ============================================================
-- 7. Add role_id column to users table
-- ============================================================
ALTER TABLE users ADD COLUMN IF NOT EXISTS role_id UUID REFERENCES roles(id);

-- Migrate existing data: set role_id based on current role value
UPDATE users SET role_id = (SELECT id FROM roles WHERE name = 'superadmin') WHERE role = 'superadmin';
UPDATE users SET role_id = (SELECT id FROM roles WHERE name = 'admin') WHERE role = 'admin';
UPDATE users SET role_id = (SELECT id FROM roles WHERE name = 'user') WHERE role = 'user';

-- Make role_id NOT NULL (all existing rows should now have a value)
ALTER TABLE users ALTER COLUMN role_id SET NOT NULL;

CREATE INDEX IF NOT EXISTS idx_users_role_id ON users(role_id);

-- ============================================================
-- 8. Add login failure tracking columns to users table
-- ============================================================
ALTER TABLE users ADD COLUMN IF NOT EXISTS login_failures INTEGER NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN IF NOT EXISTS locked_until TIMESTAMPTZ;

-- ============================================================
-- 9. Token blacklist table
-- ============================================================
CREATE TABLE token_blacklist (
    jti UUID PRIMARY KEY,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_token_blacklist_expires ON token_blacklist(expires_at);

-- ============================================================
-- 10. Audit logs table
-- ============================================================
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50),
    resource_id UUID,
    details JSONB,
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource ON audit_logs(resource_type, resource_id);

-- ============================================================
-- 11. Trigger to prevent audit log UPDATE/DELETE
-- ============================================================
CREATE OR REPLACE FUNCTION prevent_audit_modification()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'Audit logs cannot be modified';
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_prevent_audit_update
    BEFORE UPDATE ON audit_logs
    FOR EACH ROW EXECUTE FUNCTION prevent_audit_modification();

CREATE TRIGGER trg_prevent_audit_delete
    BEFORE DELETE ON audit_logs
    FOR EACH ROW EXECUTE FUNCTION prevent_audit_modification();

-- ============================================================
-- 12. Drop old role constraint and add broader one
-- ============================================================
ALTER TABLE users DROP CONSTRAINT IF EXISTS chk_users_role;
ALTER TABLE users ADD CONSTRAINT chk_users_role CHECK (role IN ('user', 'admin', 'superadmin'));
