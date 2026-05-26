-- Phase 3: Dashboard + System Settings
-- Adds dashboard:read permission and assigns to superadmin + admin roles

-- ============================================================
-- 1. Insert dashboard:read permission
-- ============================================================
INSERT INTO permissions (name, description, module)
VALUES ('dashboard:read', 'Access dashboard statistics and system settings', 'dashboard')
ON CONFLICT (name) DO NOTHING;

-- ============================================================
-- 2. Assign dashboard:read to superadmin role
-- ============================================================
INSERT INTO role_permissions (role_id, permission_id)
SELECT
    (SELECT id FROM roles WHERE name = 'superadmin'),
    (SELECT id FROM permissions WHERE name = 'dashboard:read')
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- ============================================================
-- 3. Assign dashboard:read to admin role
-- ============================================================
INSERT INTO role_permissions (role_id, permission_id)
SELECT
    (SELECT id FROM roles WHERE name = 'admin'),
    (SELECT id FROM permissions WHERE name = 'dashboard:read')
ON CONFLICT (role_id, permission_id) DO NOTHING;
