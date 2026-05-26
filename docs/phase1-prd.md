# Phase 1 PRD: 用户管理

## 项目信息

- **Language**: 中文
- **Backend**: Rust (Axum 0.8 + SQLx + JWT + Argon2)
- **Frontend**: React 19 + Vite 6 + shadcn/ui + Zustand + TanStack Query + React Router + React Hook Form + Zod
- **Database**: PostgreSQL
- **Project Name**: cradle

## 原始需求

在 Phase 0 已有的认证体系与用户 CRUD 基础上，补全用户管理的核心功能：角色权限控制、用户状态管理、管理员操作界面、个人资料页，使系统可作为后台管理系统的基础框架交付。

---

## 产品目标

1. **完整的用户生命周期管理**：管理员可通过界面完成用户的创建、编辑、禁用、删除等全流程操作，无需直接操作数据库
2. **基于角色的访问控制**：引入 user / admin / superadmin 三级角色，不同角色拥有不同的操作权限，确保系统安全
3. **自助服务能力**：普通用户可独立修改个人信息和密码，减少管理员日常运维负担

---

## 用户故事

1. **作为超级管理员**，我想在用户列表中按角色筛选和搜索用户，以便快速定位特定用户
2. **作为超级管理员**，我想创建新用户并指定其角色，以便为新成员开通系统权限
3. **作为超级管理员**，我想禁用某个用户账号而不是删除它，以便保留数据的同时阻止其登录
4. **作为管理员/超级管理员**，我想为用户重置密码，以便帮助忘记密码的用户恢复访问
5. **作为普通用户**，我想在个人资料页修改自己的姓名和密码，而不需要联系管理员

---

## 需求池

### P0 — 必须有

| # | 需求 | 说明 |
|---|------|------|
| P0-1 | 用户列表页 | 表格展示所有用户，含分页、搜索（姓名/邮箱）、角色筛选、排序（按创建时间/姓名） |
| P0-2 | 新增用户 | 管理员创建用户，指定姓名、邮箱、初始密码、角色；邮箱不可重复 |
| P0-3 | 编辑用户 | 管理员修改用户姓名、角色；不能修改邮箱（邮箱为身份标识） |
| P0-4 | 删除用户 | 确认对话框，不能删除自己，superadmin 不能被 admin 删除 |
| P0-5 | 用户状态字段 | 数据库 users 表新增 `status` 字段（active / disabled），disabled 用户无法登录 |
| P0-6 | 禁用/启用用户 | 管理员可切换用户状态，不能禁用自己 |
| P0-7 | 角色枚举约束 | role 字段从 String 改为枚举（user / admin / superadmin），数据库加 CHECK 约束 |
| P0-8 | 权限守卫 | 后端 API 按角色鉴权：用户管理接口仅 admin+ 可访问；角色变更仅 superadmin 可操作；superadmin 账号仅 superadmin 可管理 |
| P0-9 | 个人资料页 | 用户查看和修改自己的姓名 |
| P0-10 | 修改密码 | 用户输入旧密码 + 新密码修改自己密码；管理员可重置其他用户密码（无需旧密码） |

### P1 — 重要但非阻塞

| # | 需求 | 说明 |
|---|------|------|
| P1-1 | 用户列表多列排序 | 支持按姓名、邮箱、角色、创建时间、状态排序 |
| P1-2 | 批量操作 | 批量禁用/启用用户 |
| P1-3 | 操作审计日志 | 记录用户的创建、修改、删除、状态变更操作及操作人 |
| P1-4 | 前端权限控制 | 根据当前用户角色动态显示/隐藏菜单项和操作按钮 |

### P2 — 锦上添花

| # | 需求 | 说明 |
|---|------|------|
| P2-1 | 用户头像上传 | 支持上传头像，使用对象存储 |
| P2-2 | 用户导入/导出 | CSV 格式批量导入/导出用户 |
| P2-3 | 登录历史 | 展示用户最近登录 IP、时间、设备 |

---

## API 设计

### 新增端点

| Method | Path | 说明 | 权限 |
|--------|------|------|------|
| POST | /api/users | 管理员创建用户 | admin+ |
| PUT | /api/users/:id/password | 重置用户密码（管理员） | admin+ |
| PUT | /api/users/:id/status | 切换用户状态（启用/禁用） | admin+ |
| PUT | /api/users/me/password | 修改自己密码 | 认证用户 |

### 修改端点

| Method | Path | 变更说明 |
|--------|------|----------|
| GET | /api/users | 增加 query 参数：`search`（姓名/邮箱模糊搜索）、`role`（角色筛选）、`status`（状态筛选）、`sort_by` + `sort_order`（排序） |
| PUT | /api/users/:id | 增加权限校验：仅 admin+ 可调用；角色变更仅 superadmin；superadmin 账号仅 superadmin 可修改 |
| DELETE | /api/users/:id | 增加权限校验：不能删自己；superadmin 仅 superadmin 可删 |

### 端点详细规格

#### POST /api/users — 管理员创建用户

```json
// Request
{
  "email": "string (required, valid email)",
  "password": "string (required, min 6)",
  "name": "string | null (max 100)",
  "role": "user | admin | superadmin (default: user)"
}

// Response 201
{
  "id": "uuid",
  "email": "string",
  "name": "string | null",
  "role": "string",
  "status": "active",
  "created_at": "datetime"
}
```

#### GET /api/users — 列表（增强）

```json
// Query Params
?search=张&role=admin&status=active&sort_by=created_at&sort_order=desc&page=1&per_page=20

// Response (不变)
{
  "data": [UserResponse],
  "pagination": { "page": 1, "per_page": 20, "total": 100 }
}
```

#### PUT /api/users/:id/status — 切换状态

```json
// Request
{
  "status": "active | disabled"
}

// Response
{
  "id": "uuid",
  "email": "string",
  "name": "string | null",
  "role": "string",
  "status": "string",
  "created_at": "datetime"
}
```

#### PUT /api/users/:id/password — 管理员重置密码

```json
// Request
{
  "new_password": "string (min 6)"
}

// Response 200
{ "message": "Password reset successfully" }
```

#### PUT /api/users/me/password — 修改自己密码

```json
// Request
{
  "current_password": "string",
  "new_password": "string (min 6)"
}

// Response 200
{ "message": "Password changed successfully" }
```

### 数据模型变更

```sql
-- users 表新增 status 字段
ALTER TABLE users ADD COLUMN status VARCHAR(20) NOT NULL DEFAULT 'active'
  CHECK (status IN ('active', 'disabled'));

-- users 表 role 字段加 CHECK 约束
ALTER TABLE users ADD CONSTRAINT users_role_check
  CHECK (role IN ('user', 'admin', 'superadmin'));
```

### UserResponse 变更

```json
{
  "id": "uuid",
  "email": "string",
  "name": "string | null",
  "role": "user | admin | superadmin",
  "status": "active | disabled",
  "created_at": "datetime"
}
```

---

## UI 规划

### 页面清单

| 页面 | 路由 | 说明 |
|------|------|------|
| 用户列表 | /dashboard/users | 替换当前占位页，核心管理页面 |
| 个人资料 | /dashboard/profile | 查看和编辑自己的信息，修改密码 |

### 组件清单

| 组件 | 位置 | 说明 |
|------|------|------|
| UserListPage | pages/users/ | 用户列表页主组件，组合搜索栏+表格+分页 |
| UserSearchBar | pages/users/ | 搜索输入 + 角色下拉筛选 + 状态下拉筛选 |
| UserTable | pages/users/ | 用户数据表格，支持排序列头 |
| UserCreateDialog | pages/users/ | 新增用户对话框（表单：邮箱、密码、姓名、角色） |
| UserEditDialog | pages/users/ | 编辑用户对话框（表单：姓名、角色） |
| UserDeleteDialog | pages/users/ | 删除确认对话框 |
| UserStatusToggle | pages/users/ | 启用/禁用切换 |
| UserResetPasswordDialog | pages/users/ | 管理员重置密码对话框 |
| ProfilePage | pages/profile/ | 个人资料页 |
| ChangePasswordForm | pages/profile/ | 修改密码表单（旧密码+新密码+确认） |
| RoleBadge | components/shared/ | 角色标签（不同颜色区分 user/admin/superadmin） |
| StatusBadge | components/shared/ | 状态标签（绿色 active / 红色 disabled） |
| PermissionGuard | components/shared/ | 权限守卫组件，根据角色条件渲染子组件 |

### UI 交互说明

**用户列表页**：
- 顶部搜索栏：搜索框（placeholder: "搜索姓名或邮箱"）+ 角色下拉 + 状态下拉 + "新增用户"按钮
- 表格列：姓名、邮箱、角色（Badge）、状态（Badge）、创建时间、操作（编辑/重置密码/禁用/删除）
- 操作列：按权限显示 — admin 可看编辑/重置密码；superadmin 额外可看禁用/删除
- 不能对自己显示禁用/删除操作
- 分页：底部分页组件

**个人资料页**：
- 左侧：用户信息卡片（姓名、邮箱、角色、状态、创建时间）
- 右侧：修改姓名表单 + 修改密码表单

---

## 待确认问题

1. **superadmin 数量限制**：是否限制系统只能有一个 superadmin？还是允许多个？建议允许多个，但至少保留一个不可删除
2. **disabled 用户登录行为**：disabled 用户尝试登录时返回 403 Forbidden（而非 401），提示"账号已禁用，请联系管理员"
3. **管理员创建用户时的默认密码策略**：是否强制要求首次登录修改密码？P0 建议暂不实现，P1 考虑
4. **角色降级保护**：superadmin 能否将自己的角色降级为 admin？建议不能，防止误操作导致无 superadmin
5. **密码复杂度策略**：当前仅要求 6 位，是否需要更强的策略（大小写+数字+特殊字符）？P0 保持 6 位，P1 考虑增强
6. **刷新令牌实现**：Phase 0 的 refresh/logout 接口仍为 TODO，是否在 Phase 1 一起实现？建议 P1 处理
