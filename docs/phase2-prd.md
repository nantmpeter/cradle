# Phase 2 PRD: RBAC + 安全

## 项目信息

- **Language**: 中文
- **Backend**: Rust (Axum 0.8 + SQLx + JWT + Argon2)
- **Frontend**: React 19 + Vite 6 + shadcn/ui + Zustand + TanStack Query + React Router + React Hook Form + Zod
- **Database**: PostgreSQL
- **Project Name**: cradle

## 原始需求

Phase 1 已完成用户管理（三级硬编码角色、用户 CRUD、状态管理、密码管理、Refresh Token 轮换）。Phase 2 将安全体系从"基于角色"升级为"基于权限"的 RBAC 体系，并补充审计日志、API 限流、登录安全防护等企业级安全能力，使系统可满足多租户、合规审计等生产场景需求。

---

## 产品目标

1. **细粒度权限控制**：从硬编码角色枚举升级为 RBAC 权限体系，支持自定义角色和细粒度权限（如 `users:create`、`roles:manage`），超级管理员自动拥有全部权限
2. **完整审计追踪**：所有敏感操作（登录/登出、用户 CRUD、角色/权限变更、密码重置）均记录不可篡改的审计日志，支持按用户、操作类型、时间范围查询
3. **接口级安全防护**：API 限流防暴力破解、登录失败锁定、JWT 黑名单确保登出即时生效，形成纵深防御
4. **向后兼容**：Phase 1 的用户数据、Token、前端功能平滑迁移，不破坏现有行为

---

## 用户故事

### 超级管理员

1. 作为超级管理员，我想创建自定义角色并为角色分配细粒度权限，以便根据组织架构灵活控制系统访问
2. 作为超级管理员，我想查看审计日志以了解系统中所有关键操作的历史记录，以便进行安全审查和问题追溯
3. 作为超级管理员，我想按用户、操作类型、时间范围筛选审计日志，以便快速定位特定事件

### 管理员

4. 作为管理员，我想在用户管理界面看到当前用户的权限列表，以便了解每个用户的实际能力范围
5. 作为管理员，我想分配自定义角色给用户，以便按最小权限原则控制用户访问

### 系统层面

6. 作为系统，我需要对所有 API 端点进行限流，以防止暴力破解和 DoS 攻击
7. 作为系统，我需要在用户连续登录失败后临时锁定账号，以防止密码被暴力破解
8. 作为系统，我需要在用户登出时立即使 JWT 失效，以防止 Token 被盗用后继续使用

---

## 需求池

### P0 — 必须有

| # | 需求 | 说明 |
|---|------|------|
| P0-1 | permissions 表 | 创建 permissions 表，预置系统权限（users:create, users:read, users:update, users:delete, users:manage_status, users:reset_password, roles:create, roles:read, roles:update, roles:delete, roles:manage_permissions, audit:read, system:manage） |
| P0-2 | role_permissions 关联表 | 角色-权限多对多关联；超级管理员（superadmin 角色标记为 `is_system = true`）自动拥有全部权限，无需显式关联 |
| P0-3 | roles 表扩展 | 新建 roles 表（id, name, description, is_system, created_at, updated_at），将现有硬编码角色迁移为系统内置记录；users 表 role 字段改为 role_id 外键 |
| P0-4 | 自定义角色 CRUD | 超级管理员可创建/编辑/删除自定义角色（is_system = false），并为角色分配/撤销权限；内置角色不可删除、名称不可修改 |
| P0-5 | 权限守卫重构 | 后端权限检查从 `AuthUser::require_admin()` 改为 `AuthUser::require_permission("users:create")` 模式；新增权限检查中间件/提取器 |
| P0-6 | 前端权限升级 | PermissionGuard 组件从角色检查改为权限检查；前端维护权限列表，按钮/菜单按权限显示隐藏 |
| P0-7 | 审计日志记录 | 后端自动记录关键操作到 audit_logs 表；记录字段：id, user_id, action, resource_type, resource_id, details(JSONB), ip_address, user_agent, created_at |
| P0-8 | 审计日志查询 API | GET /api/audit-logs，支持分页 + 筛选（按用户/操作类型/时间范围），仅超级管理员可访问 |
| P0-9 | 审计日志管理界面 | 前端审计日志列表页，支持分页浏览和筛选 |
| P0-10 | 审计日志不可篡改 | audit_logs 表仅允许 INSERT，禁止 UPDATE 和 DELETE（数据库级约束 + 应用层保障） |
| P0-11 | API 通用限流 | 基于 IP 的全局限流（100 req/min），超出返回 429 Too Many Requests + Retry-After 头 |
| P0-12 | 登录端点限流 | 登录端点独立限流（5 次/min/IP），超出返回 429 |
| P0-13 | 登录失败锁定 | 连续登录失败 5 次后锁定账号 15 分钟；users 表增加 `login_failures` 计数和 `locked_until` 时间戳 |
| P0-14 | JWT Token 黑名单 | 登出时将 access token 的 jti 加入黑名单（Redis 或数据库表），直到 token 自然过期；auth 中间件检查黑名单 |

### P1 — 应该有

| # | 需求 | 说明 |
|---|------|------|
| P1-1 | CORS 白名单配置 | 从 `CorsLayer::new().allow_origin(Any)` 改为可配置的白名单域名列表，通过环境变量或配置文件指定 |
| P1-2 | 角色管理界面 | 前端角色列表页，展示所有角色及其权限，支持创建/编辑自定义角色、分配权限 |
| P1-3 | 用户详情增加角色/权限展示 | 用户列表和编辑弹窗中展示当前角色名称和关联的权限标签 |
| P1-4 | 审计日志详情 | 点击审计日志条目可查看详情（完整 JSONB details 字段） |
| P1-5 | 限流响应头 | 所有 API 响应中包含 `X-RateLimit-Limit`、`X-RateLimit-Remaining`、`X-RateLimit-Reset` 标准头 |
| P1-6 | 登录失败提示优化 | 返回不同错误信息："密码错误（还剩 N 次机会）" vs "账号已锁定，请 N 分钟后重试" |

### P2 — 可以有

| # | 需求 | 说明 |
|---|------|------|
| P2-1 | 审计日志导出 | 支持将审计日志导出为 CSV |
| P2-2 | 权限批量操作 | 批量给多个用户分配同一角色 |
| P2-3 | 限流配置管理 | 超级管理员可通过界面调整限流参数（无需重启服务） |
| P2-4 | 可疑登录通知 | 检测到异常 IP 登录时发送通知（邮件/WebSocket） |

---

## 数据模型设计建议

### 新增表

#### roles 表

```sql
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL UNIQUE,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT false,  -- 内置角色不可删除
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 迁移内置角色
INSERT INTO roles (name, description, is_system) VALUES
    ('user', '普通用户', true),
    ('admin', '管理员', true),
    ('superadmin', '超级管理员', true);
```

#### permissions 表

```sql
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,   -- 如 'users:create'
    description TEXT,
    module VARCHAR(50) NOT NULL,          -- 模块分组：users / roles / audit / system
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 预置权限
INSERT INTO permissions (name, description, module) VALUES
    ('users:create',  '创建用户',       'users'),
    ('users:read',    '查看用户',       'users'),
    ('users:update',  '编辑用户',       'users'),
    ('users:delete',  '删除用户',       'users'),
    ('users:manage_status', '启用/禁用用户', 'users'),
    ('users:reset_password', '重置用户密码', 'users'),
    ('roles:create',  '创建角色',       'roles'),
    ('roles:read',    '查看角色',       'roles'),
    ('roles:update',  '编辑角色',       'roles'),
    ('roles:delete',  '删除角色',       'roles'),
    ('roles:manage_permissions', '分配角色权限', 'roles'),
    ('audit:read',    '查看审计日志',   'audit'),
    ('system:manage', '系统管理',       'system');
```

#### role_permissions 表

```sql
CREATE TABLE role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- admin 角色默认权限（除角色管理和审计外的用户管理权限）
-- superadmin 不需要显式关联，代码层面 is_system + name='superadmin' 自动放行
```

#### audit_logs 表

```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID,                          -- 可为空（未认证操作如登录失败）
    action VARCHAR(100) NOT NULL,          -- 如 'user.login', 'user.create', 'role.update'
    resource_type VARCHAR(50),             -- 如 'user', 'role', 'permission'
    resource_id UUID,                      -- 操作对象的 ID
    details JSONB,                         -- 操作详情（变更前后值等）
    ip_address INET,                       -- 客户端 IP
    user_agent TEXT,                       -- 客户端 User-Agent
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);

-- 防止修改和删除（通过数据库触发器）
CREATE OR REPLACE FUNCTION prevent_audit_modification()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'Audit logs cannot be modified or deleted';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_prevent_audit_update
    BEFORE UPDATE ON audit_logs
    FOR EACH ROW EXECUTE FUNCTION prevent_audit_modification();

CREATE TRIGGER trg_prevent_audit_delete
    BEFORE DELETE ON audit_logs
    FOR EACH ROW EXECUTE FUNCTION prevent_audit_modification();
```

#### token_blacklist 表（如果不用 Redis）

```sql
CREATE TABLE token_blacklist (
    jti UUID PRIMARY KEY,                  -- JWT ID
    expires_at TIMESTAMPTZ NOT NULL,       -- token 原始过期时间，用于清理
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_token_blacklist_expires ON token_blacklist(expires_at);
```

### 修改表

#### users 表变更

```sql
-- 新增字段
ALTER TABLE users ADD COLUMN role_id UUID REFERENCES roles(id);
ALTER TABLE users ADD COLUMN login_failures INTEGER NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN locked_until TIMESTAMPTZ;

-- 数据迁移：将现有 role 字符串映射到 role_id
UPDATE users u SET role_id = r.id FROM roles r WHERE u.role = r.name;

-- 迁移完成后，考虑保留原 role 列做兼容或创建新列
-- 建议保留 role 列用于 JWT claims 短期兼容，逐步迁移到 role_id
```

---

## API 设计建议

### 新增端点

#### 角色管理

| Method | Path | 说明 | 权限 |
|--------|------|------|------|
| GET | /api/roles | 角色列表（含权限） | roles:read |
| POST | /api/roles | 创建自定义角色 | roles:create |
| GET | /api/roles/:id | 角色详情（含权限列表） | roles:read |
| PUT | /api/roles/:id | 编辑角色（名称/描述/权限） | roles:update |
| DELETE | /api/roles/:id | 删除自定义角色（内置角色不可删） | roles:delete |
| PUT | /api/roles/:id/permissions | 更新角色的权限列表 | roles:manage_permissions |

#### 权限管理

| Method | Path | 说明 | 权限 |
|--------|------|------|------|
| GET | /api/permissions | 权限列表（全部预置权限） | roles:read |

#### 审计日志

| Method | Path | 说明 | 权限 |
|--------|------|------|------|
| GET | /api/audit-logs | 审计日志列表（分页+筛选） | audit:read |
| GET | /api/audit-logs/:id | 审计日志详情 | audit:read |

### 修改端点

| Method | Path | 变更说明 |
|--------|------|----------|
| POST | /api/auth/login | 新增登录失败计数和锁定逻辑；返回更详细的错误信息；触发审计日志 |
| POST | /api/auth/logout | 登出时将 access token jti 写入黑名单 |
| GET | /api/auth/me | 响应中增加 `permissions` 数组和 `role_id` |
| POST | /api/users | 权限从 `admin+` 改为 `users:create`；触发审计日志 |
| GET | /api/users | 权限从 `admin+` 改为 `users:read`；请求参数增加 `role_id` 筛选 |
| PUT | /api/users/:id | 权限从 `admin+` 改为 `users:update`；role 字段改为 role_id；触发审计日志 |
| DELETE | /api/users/:id | 权限从 `admin+` 改为 `users:delete`；触发审计日志 |
| PUT | /api/users/:id/status | 权限改为 `users:manage_status`；触发审计日志 |
| PUT | /api/users/:id/password | 权限改为 `users:reset_password`；触发审计日志 |

### 端点详细规格

#### GET /api/roles — 角色列表

```json
// Response 200
{
  "data": [
    {
      "id": "uuid",
      "name": "editor",
      "description": "内容编辑",
      "is_system": false,
      "permissions": ["users:read", "users:update"],
      "created_at": "datetime",
      "updated_at": "datetime"
    }
  ]
}
```

#### POST /api/roles — 创建角色

```json
// Request
{
  "name": "editor (required, unique, 2-50 chars)",
  "description": "string | null",
  "permission_ids": ["uuid", "uuid"]
}

// Response 201
{
  "id": "uuid",
  "name": "editor",
  "description": "内容编辑",
  "is_system": false,
  "permissions": ["users:read", "users:update"],
  "created_at": "datetime",
  "updated_at": "datetime"
}
```

#### PUT /api/roles/:id/permissions — 更新角色权限

```json
// Request
{
  "permission_ids": ["uuid", "uuid", "uuid"]
}

// Response 200
{
  "id": "uuid",
  "name": "editor",
  "description": "...",
  "is_system": false,
  "permissions": ["users:read", "users:update", "users:create"],
  "created_at": "datetime",
  "updated_at": "datetime"
}
```

#### GET /api/audit-logs — 审计日志列表

```json
// Query Params
?page=1&per_page=20&user_id=uuid&action=user.login&from=2026-01-01&to=2026-06-01

// Response 200
{
  "data": [
    {
      "id": "uuid",
      "user_id": "uuid | null",
      "user_email": "string | null",
      "action": "user.login",
      "resource_type": "user",
      "resource_id": "uuid | null",
      "details": { "key": "value" },
      "ip_address": "192.168.1.1",
      "user_agent": "Mozilla/5.0...",
      "created_at": "datetime"
    }
  ],
  "pagination": { "page": 1, "per_page": 20, "total": 1000 }
}
```

#### GET /api/auth/me 增强

```json
// Response 200 (Phase 2 新增 permissions 和 role_id)
{
  "id": "uuid",
  "email": "string",
  "name": "string | null",
  "role": "admin",
  "role_id": "uuid",
  "status": "active",
  "must_change_password": false,
  "permissions": ["users:create", "users:read", "users:update", "users:delete", "users:manage_status", "users:reset_password"],
  "created_at": "datetime",
  "updated_at": "datetime"
}
```

---

## UI 页面规划

### 新增页面

| 页面 | 路由 | 说明 |
|------|------|------|
| 角色列表 | /dashboard/roles | 展示所有角色、权限数量、操作按钮（P1） |
| 角色创建/编辑 | /dashboard/roles/new, /dashboard/roles/:id/edit | 表单：名称、描述、权限多选 |
| 审计日志 | /dashboard/audit-logs | 日志列表，含筛选栏（用户/操作类型/时间范围） |

### 修改页面

| 页面 | 变更说明 |
|------|----------|
| 用户列表 | 创建/编辑用户弹窗：角色选择改为下拉（含自定义角色）；表格增加角色名列 |
| 个人资料 | 展示当前用户的权限列表（只读） |
| 侧边栏导航 | 新增"角色管理"和"审计日志"菜单项，按权限显示/隐藏 |

### 新增组件

| 组件 | 位置 | 说明 |
|------|------|------|
| RoleListPage | pages/roles/ | 角色列表页 |
| RoleCreateDialog | pages/roles/ | 创建角色弹窗（名称 + 描述 + 权限勾选） |
| RoleEditDialog | pages/roles/ | 编辑角色弹窗 |
| RoleDeleteDialog | pages/roles/ | 删除角色确认弹窗 |
| PermissionSelector | components/shared/ | 权限多选组件，按模块分组显示 |
| PermissionBadge | components/shared/ | 权限标签组件 |
| AuditLogListPage | pages/audit/ | 审计日志列表页 |
| AuditLogFilterBar | pages/audit/ | 筛选栏（用户选择 + 操作类型下拉 + 时间范围） |
| AuditLogDetailDialog | pages/audit/ | 日志详情弹窗（展示完整 JSONB） |

### PermissionGuard 组件变更

```tsx
// Phase 1：基于角色
<PermissionGuard adminOnly>
  <Button>管理用户</Button>
</PermissionGuard>

// Phase 2：基于权限
<PermissionGuard permission="users:create">
  <Button>新增用户</Button>
</PermissionGuard>
```

### 侧边栏菜单（按权限控制）

```
仪表盘        → 无特殊权限
用户管理      → users:read
角色管理      → roles:read
审计日志      → audit:read
```

---

## 向后兼容策略

1. **数据库迁移**：users 表保留 `role` 列（VARCHAR），新增 `role_id` 列；数据迁移脚本将 role 字符串映射到 roles 表记录
2. **JWT Claims**：Token 中保留 `role` 字符串字段用于快速判断，新增 `permissions` 数组字段
3. **前端 authStore**：User 类型新增 `permissions: string[]` 和 `role_id: string`，保留 `role: Role` 字段兼容
4. **渐进式迁移**：Phase 2 初期，后端权限检查同时支持旧角色检查和新权限检查，确保零停机

---

## 审计日志 action 清单

| action | 说明 | resource_type | details 示例 |
|--------|------|---------------|-------------|
| auth.login | 用户登录 | user | `{"success": true}` |
| auth.login_failed | 登录失败 | user | `{"reason": "wrong_password", "failures": 3}` |
| auth.logout | 用户登出 | user | `{}` |
| auth.account_locked | 账号锁定 | user | `{"locked_until": "..."}` |
| user.create | 创建用户 | user | `{"email": "...", "role": "admin"}` |
| user.update | 编辑用户 | user | `{"changes": {"name": ["old", "new"]}}` |
| user.delete | 删除用户 | user | `{"email": "..."}` |
| user.status_change | 状态变更 | user | `{"from": "active", "to": "disabled"}` |
| user.password_reset | 管理员重置密码 | user | `{}` |
| user.password_change | 用户修改密码 | user | `{}` |
| role.create | 创建角色 | role | `{"name": "...", "permissions": [...]}` |
| role.update | 编辑角色 | role | `{"changes": {...}}` |
| role.delete | 删除角色 | role | `{"name": "..."}` |
| role.permissions_change | 角色权限变更 | role | `{"added": [...], "removed": [...]}` |

---

## 技术建议

### 限流实现

- 使用 `tower-governor` crate，基于 IP 的滑动窗口限流
- 全局限流：100 req/min/IP，作为全局中间件
- 登录限流：5 req/min/IP，仅作用于 `POST /api/auth/login`
- 超出限流返回 429 + `Retry-After` 头 + JSON 错误体

### JWT 黑名单实现

- 方案 A（推荐，简单）：新增 `token_blacklist` 数据库表，记录 jti + expires_at
- 方案 B（高性能）：使用 Redis SET + TTL
- auth 中间件在验证 JWT 后额外检查 jti 是否在黑名单中
- 定期清理过期的黑名单记录（expires_at < NOW()）

### 登录失败锁定

- users 表增加 `login_failures` (INTEGER) 和 `locked_until` (TIMESTAMPTZ)
- 登录失败时 `login_failures += 1`；达到阈值（5 次）时设置 `locked_until = NOW() + 15min`
- 登录成功时重置 `login_failures = 0`，清除 `locked_until`
- 登录前检查 `locked_until`，未过期则拒绝登录

---

## 待确认问题

1. **自定义角色数量上限**：是否限制每个系统最多创建的角色数量？建议不限制
2. **角色删除时用户处理**：删除自定义角色时，该角色下的用户如何处理？建议：不允许删除仍有用户关联的角色（需先迁移用户到其他角色）
3. **JWT 黑名单存储**：使用数据库表还是 Redis？建议 Phase 2 先用数据库表（简单），后续可迁移到 Redis
4. **审计日志保留策略**：审计日志是否需要自动清理（如保留 90 天）？建议 Phase 2 不清理，后续按需实现
5. **登录锁定阈值**：连续失败 5 次锁定 15 分钟是否合适？是否需要可配置？
6. **限流实现 crate**：`tower-governor` vs `governor` vs 自定义中间件？建议 `tower-governor`（Axum 原生集成）
7. **前端路由权限**：前端路由守卫是仅基于权限隐藏菜单，还是在路由层面拦截（显示 403）？建议两者都做
8. **JWT Token 结构变更**：是否在 Phase 2 的 JWT 中新增 `permissions` 数组（会导致 Token 体积增大）？建议不在 JWT 中放权限列表，而是在每次请求时从数据库/缓存查询，或登录时加载到前端 store
