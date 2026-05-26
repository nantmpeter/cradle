-- Phase 7: Data Permission (Department Isolation)
-- Add data_scope column to roles table

-- data_scope values:
-- 'all'                - Can see all data (default for superadmin)
-- 'department'         - Can only see data from own department
-- 'department_and_sub' - Can see data from own department and sub-departments
-- 'self'               - Can only see own data

ALTER TABLE roles ADD COLUMN IF NOT EXISTS data_scope VARCHAR(20) NOT NULL DEFAULT 'all';

-- Set sensible defaults for existing roles
UPDATE roles SET data_scope = 'all' WHERE name ILIKE '%super%admin%' OR is_system = true;
