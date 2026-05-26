# Cradle 使用文档

> 版本：0.1.0 · 最后更新：2026-05-25

## 目录

- [1. 快速开始](#1-快速开始)
- [2. 配置说明](#2-配置说明)
- [3. 认证体系](#3-认证体系)
- [4. API 路由架构](#4-api-路由架构)
- [5. Admin API 参考](#5-admin-api-参考)
- [6. App API 参考](#6-app-api-参考)
- [7. Open API 参考](#7-open-api-参考)
- [8. API Key 管理](#8-api-key-管理)
- [9. OAuth2 第三方登录](#9-oauth2-第三方登录)
- [10. Webhook 出站通知](#10-webhook-出站通知)
- [11. 错误格式](#11-错误格式)
- [12. 权限体系](#12-权限体系)
- [13. 数据权限](#13-数据权限)
- [14. 测试](#14-测试)
- [15. 部署](#15-部署)
- [16. FAQ](#16-faq)

---

## 1. 快速开始

### 1.1 环境要求

| 依赖 | 版本 | 说明 |
|------|------|------|
| Rust | 1.85+ (edition 2024) | 后端编译 |
| Node.js | 20+ | 前端构建 |
| PostgreSQL | 16 | 数据库 |
| pnpm | 推荐 | 前端包管理 |

### 1.2 启动步骤

```bash
# 1. 克隆项目
git clone <repo-url> && cd cradle

# 2. 启动数据库（二选一）
docker compose up db -d              # Docker 方式
# createdb cradle                # 本地 PostgreSQL

# 3. 启动后端
cd apps/backend
cp config/default.toml config/local.toml   # 复制配置
cargo run                                   # 启动（自动迁移）
# → http://localhost:8080

# 4. 启动前端
cd apps/frontend
pnpm install
pnpm dev
# → http://localhost:5173
```

### 1.3 默认账号

| 邮箱 | 密码 | 角色 |
|------|------|------|
| `admin@example.com` | `Admin@1234` | superadmin |

---

## 2. 配置说明

后端配置文件位于 `apps/backend/config/`，加载顺序：`default.toml` → `local.toml`（覆盖） → 环境变量（`APP__` 前缀）。

### 2.1 完整配置项

```toml
[server]
host = "0.0.0.0"           # 监听地址
port = 8080                 # 监听端口

[database]
url = "postgres://dev:dev@localhost:5432/cradle"
max_connections = 10        # 连接池大小

[jwt]
secret = "change-me-in-production-use-strong-random-key"  # JWT 签名密钥（生产环境必改）
access_exp_secs = 3600      # Access Token 有效期（秒）
refresh_exp_secs = 604800   # Refresh Token 有效期（秒）

[rate_limit]
global_rpm = 100            # 全局每分钟请求数上限
login_rpm = 5               # 登录接口每分钟请求数上限

[storage]
upload_dir = "./uploads"    # 文件上传目录
max_upload_size = 10485760  # 文件大小上限（10 MB）
max_avatar_size = 2097152   # 头像大小上限（2 MB）

[totp]
encryption_key = "jBrgUIZ+TRxlYByzq83lkK0pR9lLQQP/ZmRWxDK6r8A="  # TOTP 密钥加密 key

[api]
version = "v1"                    # API 版本号
legacy_routes_enabled = true      # 是否启用旧路由兼容（/api/*）
docs_enabled = true               # 是否启用 Swagger UI

[api.wechat]
enabled = false
app_id = ""
app_secret = ""

[api.sms]
enabled = false
provider = ""
template_id = ""

[api.github]                       # GitHub OAuth2
client_id = ""
client_secret = ""

[api.google]                       # Google OAuth2
client_id = ""
client_secret = ""

[webhook]
max_retries = 3                          # 最大重试次数
retry_intervals_secs = [60, 300, 1800]   # 重试间隔（秒），指数退避
timeout_secs = 10                        # HTTP 请求超时（秒）
```

### 2.2 环境变量覆盖

环境变量使用 `APP__` 前缀 + `__` 分隔符，例如：

```bash
APP__DATABASE__URL=postgres://user:pass@host:5432/db
APP__JWT__SECRET=my-production-secret
APP__SERVER__PORT=9090
```

---

## 3. 认证体系

系统支持三种认证方式，分别对应三个 API 层：

| 认证方式 | 使用场景 | 请求头 |
|---------|---------|--------|
| JWT + RBAC | 管理后台 (Admin) | `Authorization: Bearer <access_token>` |
| JWT (client_type=app) | 移动端/小程序 (App) | `Authorization: Bearer <access_token>` |
| API Key + Scope | 第三方接入 (Open) | `Authorization: Bearer rk_xxx` 或 `X-API-Key: rk_xxx` |

### 3.1 JWT 认证流程

```
┌──────────┐    POST /api/auth/login     ┌──────────┐
│  Client  │ ──────────────────────────→ │  Server  │
│          │ ←────────────────────────── │          │
│          │  { access_token, refresh_token }       │
│          │                              │          │
│          │  GET /api/v1/admin/users     │          │
│          │  Authorization: Bearer xxx   │          │
│          │ ──────────────────────────→ │          │
│          │ ←────────────────────────── │          │
│          │  { data: [...] }            │          │
└──────────┘                              └──────────┘
```

### 3.2 登录接口

**POST** `/api/auth/login`

请求：
```json
{
  "email": "admin@example.com",
  "password": "Admin@1234"
}
```

成功响应 (200)：
```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_type": "bearer",
  "must_change_password": false
}
```

需要 2FA 时响应 (401)：
```json
{
  "error": {
    "code": "TWO_FACTOR_REQUIRED",
    "message": "Two-factor authentication required",
    "request_id": null,
    "details": null
  },
  "status": 401
}
```

此时需用 `temp_token` 继续验证：
```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_type": "bearer",
  "must_change_password": false,
  "requires_2fa": true,
  "temp_token": "temp_xxx..."
}
```

**POST** `/api/auth/2fa/verify`

请求：
```json
{
  "temp_token": "temp_xxx...",
  "code": "123456"
}
```

### 3.3 Token 刷新

**POST** `/api/auth/refresh`

请求：
```json
{
  "refresh_token": "eyJ..."
}
```

响应同登录成功响应。

### 3.4 注册

**POST** `/api/auth/register`

请求：
```json
{
  "email": "new@example.com",
  "password": "StrongPass@123",
  "name": "New User"
}
```

密码规则：至少 8 位，且包含以下 4 类中至少 3 类：大写字母、小写字母、数字、特殊字符。

注册用户自动获得 `user` 角色。

---

## 4. API 路由架构

### 4.1 三层路由体系

| 层 | 前缀 | 认证 | 说明 |
|----|------|------|------|
| **Admin** | `/api/v1/admin/*` | JWT + RBAC + 权限校验 | 管理后台接口 |
| **App** | `/api/v1/app/*` | JWT (client_type=app) | 移动端 / 小程序接口 |
| **Open** | `/api/v1/open/*` | API Key + Scope | 第三方开放接口 |
| **Legacy** | `/api/*` | JWT + RBAC | 旧版路由兼容（带 Deprecation 响应头） |

### 4.2 旧路由兼容

旧路由 (`/api/users`, `/api/roles` 等) 保持完全可用，响应头自动附加：

```
Deprecation: true
X-Legacy-Route: true
```

可通过配置 `api.legacy_routes_enabled = false` 关闭。

### 4.3 不受版本化影响的路由

以下路由保持原有路径，不受版本化影响：

| 路径 | 说明 |
|------|------|
| `/api/auth/*` | 认证（登录/注册/刷新/2FA） |
| `/api/sse/notifications` | SSE 实时通知推送 |
| `/api/i18n/messages` | 国际化消息 |
| `/api/configs/public` | 公开配置 |

---

## 5. Admin API 参考

所有 Admin API 挂载在 `/api/v1/admin/` 下（旧版 `/api/` 同样可用），需要 JWT 认证。

### 5.1 用户管理

| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| GET | `/users` | 用户列表（分页） | users:read |
| POST | `/users` | 创建用户 | users:create |
| GET | `/users/me` | 当前用户信息（含权限列表） | - |
| GET | `/users/{id}` | 用户详情 | users:read |
| PUT | `/users/{id}` | 更新用户 | users:update |
| DELETE | `/users/{id}` | 删除用户 | users:delete |
| PUT | `/users/{id}/status` | 切换用户状态 | users:update |
| PUT | `/users/{id}/password` | 重置密码 | users:update |
| POST | `/users/{id}/2fa/reset` | 重置用户 2FA | users:update |
| PUT | `/users/me/password` | 修改自己密码 | - |
| POST | `/users/me/avatar` | 上传头像 | - |
| POST | `/users/import/` | 批量导入用户 | users:create |
| GET | `/users/import/template` | 下载导入模板 | users:create |

**用户列表** `GET /users`

查询参数：
| 参数 | 类型 | 说明 |
|------|------|------|
| page | integer | 页码（默认 1） |
| per_page | integer | 每页数量（默认 20） |
| search | string | 搜索关键词 |
| role | string | 角色过滤 |
| status | string | 状态过滤 |
| department_id | uuid | 部门过滤 |

响应：
```json
{
  "data": [
    {
      "id": "uuid",
      "email": "user@example.com",
      "name": "User",
      "role": "admin",
      "role_id": "uuid",
      "department_id": null,
      "avatar_url": null,
      "status": "active",
      "must_change_password": false,
      "created_at": "2026-05-20T00:00:00Z",
      "updated_at": "2026-05-20T00:00:00Z"
    }
  ],
  "total": 100,
  "page": 1,
  "per_page": 20
}
```

**创建用户** `POST /users`

请求：
```json
{
  "email": "new@example.com",
  "password": "StrongPass@123",
  "name": "New User",
  "role": "admin",
  "role_id": "uuid",
  "department_id": "uuid"
}
```

**GET /users/me 响应**（额外包含权限列表）：
```json
{
  "id": "uuid",
  "email": "admin@example.com",
  "name": "Admin",
  "role": "superadmin",
  "role_id": "uuid",
  "avatar_url": null,
  "status": "active",
  "must_change_password": false,
  "two_factor_enabled": false,
  "department_id": null,
  "permissions": ["users:read", "users:create", "users:update", "users:delete", "*"],
  "created_at": "...",
  "updated_at": "..."
}
```

### 5.2 角色与权限

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/roles` | 角色列表 |
| POST | `/roles` | 创建角色 |
| GET | `/roles/{id}` | 角色详情 |
| PUT | `/roles/{id}` | 更新角色 |
| DELETE | `/roles/{id}` | 删除角色 |
| GET | `/roles/permissions` | 获取所有可用权限 |
| PUT | `/roles/{id}/permissions` | 更新角色权限 |

**更新角色权限** `PUT /roles/{id}/permissions`

请求：
```json
{
  "permissions": ["users:read", "users:create", "roles:read"]
}
```

### 5.3 部门管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/departments` | 部门列表（支持树形） |
| POST | `/departments` | 创建部门 |
| PUT | `/departments/{id}` | 更新部门 |
| DELETE | `/departments/{id}` | 删除部门 |

### 5.4 字典管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/dict-types` | 字典类型列表 |
| POST | `/dict-types` | 创建字典类型 |
| PUT | `/dict-types/{id}` | 更新字典类型 |
| DELETE | `/dict-types/{id}` | 删除字典类型 |
| GET | `/dict-types/{id}/items` | 字典项列表 |
| POST | `/dict-types/{id}/items` | 创建字典项 |
| PUT | `/dict-items/{id}` | 更新字典项 |
| DELETE | `/dict-items/{id}` | 删除字典项 |
| GET | `/dicts/{code}` | 按 code 查询字典（公开） |

### 5.5 文件管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/files` | 文件列表 |
| POST | `/files/` | 上传文件（multipart/form-data） |
| DELETE | `/files/{id}` | 删除文件 |

上传限制：普通文件 10 MB，头像 2 MB。

### 5.6 数据导出

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/export/users` | 导出用户数据（CSV/XLSX） |

查询参数：
| 参数 | 类型 | 说明 |
|------|------|------|
| format | string | `csv` 或 `xlsx` |
| columns | string | 列选择（逗号分隔，如 `email,name,role`） |

### 5.7 审计日志

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/audit-logs` | 日志列表（分页） |
| GET | `/audit-logs/{id}` | 日志详情 |
| GET | `/audit-logs/export` | 导出日志 |

### 5.8 登录日志

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/login-logs` | 日志列表（分页） |
| GET | `/login-logs/{id}` | 日志详情 |

### 5.9 会话管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/sessions` | 所有活跃会话 |
| GET | `/sessions/me` | 当前用户会话 |
| DELETE | `/sessions/{id}` | 终止会话 |
| DELETE | `/sessions/user/{user_id}` | 强制登出用户所有会话 |

### 5.10 菜单管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/menus` | 菜单列表 |
| POST | `/menus` | 创建菜单 |
| GET | `/menus/tree` | 菜单树形结构 |
| PUT | `/menus/{id}` | 更新菜单 |
| DELETE | `/menus/{id}` | 删除菜单 |

### 5.11 通知

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/notifications` | 通知列表 |
| GET | `/notifications/unread-count` | 未读数量 |
| POST | `/notifications/read-all` | 全部已读 |
| POST | `/notifications/{id}/read` | 标记已读 |
| DELETE | `/notifications/{id}` | 删除通知 |
| POST | `/notifications/broadcast` | 广播通知 |

**SSE 实时推送** `GET /api/sse/notifications?token=<access_token>`

EventSource 连接，实时推送通知事件。

### 5.12 系统配置

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/configs` | 所有配置 |
| GET | `/configs/{group}` | 按分组查询 |
| PUT | `/configs/{group}/{key}` | 更新配置项 |
| GET | `/api/configs/public` | 公开配置（无需认证） |

### 5.13 全局搜索

**GET** `/search?q=keyword`

响应：
```json
{
  "data": {
    "users": [...],
    "roles": [...],
    "departments": [...],
    "dicts": [...]
  }
}
```

### 5.14 Dashboard

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/dashboard/stats` | 系统统计（用户数、角色数、在线数等） |
| GET | `/dashboard/settings` | Dashboard 设置 |

### 5.15 2FA 管理

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/auth/2fa/setup` | 生成 TOTP secret + QR 码 |
| POST | `/auth/2fa/enable` | 启用 2FA |
| POST | `/auth/2fa/disable` | 禁用 2FA |

---

## 6. App API 参考

App API 挂载在 `/api/v1/app/` 下，需要 `client_type=app` 的 JWT Token。

### 6.1 App 认证（公开，无需 JWT）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/app/auth/providers` | 获取可用第三方登录列表 |
| GET | `/api/v1/app/auth/authorize?provider=github` | 获取 OAuth 授权 URL |
| GET | `/api/v1/app/auth/callback?provider=github&code=xxx` | OAuth 回调，换取 App JWT |
| POST | `/api/v1/app/auth/sms/send` | 发送短信验证码 |
| POST | `/api/v1/app/auth/sms/verify` | 验证短信码，换取 App JWT |

**获取授权 URL** `GET /api/v1/app/auth/authorize?provider=github`

响应：
```json
{
  "url": "https://github.com/login/oauth/authorize?client_id=xxx&..."
}
```

**OAuth 回调** `GET /api/v1/app/auth/callback?provider=github&code=xxx`

响应（自动注册或关联已有账号）：
```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_type": "bearer",
  "must_change_password": false
}
```

**短信验证码登录**：

发送验证码 `POST /api/v1/app/auth/sms/send`
```json
{ "phone": "13800138000" }
```

验证 `POST /api/v1/app/auth/sms/verify`
```json
{ "phone": "13800138000", "code": "123456" }
```

### 6.2 App 用户操作（需 App JWT）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/me` | 当前用户信息 |
| PUT | `/me/password` | 修改密码 |
| GET | `/notifications` | 通知列表 |
| POST | `/notifications/{id}/read` | 标记已读 |
| GET | `/auth/bindings` | 第三方账号绑定列表 |
| POST | `/auth/social-bind` | 绑定第三方账号 |
| POST | `/auth/social-unbind` | 解绑第三方账号 |

---

## 7. Open API 参考

Open API 挂载在 `/api/v1/open/` 下，用于第三方接入。

### 7.1 公开端点（无需认证）

**GET** `/api/v1/open/health`

响应：
```json
{
  "status": "ok",
  "version": "0.1.0",
  "timestamp": "2026-05-25T10:00:00Z"
}
```

### 7.2 受保护端点（需要 API Key）

| 方法 | 路径 | 说明 | 所需 Scope |
|------|------|------|-----------|
| GET | `/users` | 用户列表 | `users:read` |
| GET | `/users/{id}` | 用户详情 | `users:read` |
| GET | `/departments` | 部门列表 | `departments:read` |

请求方式：
```bash
curl -H "Authorization: Bearer rk_xxxxx" \
  http://localhost:8080/api/v1/open/users

# 或
curl -H "X-API-Key: rk_xxxxx" \
  http://localhost:8080/api/v1/open/users
```

响应格式与 Admin API 一致（分页、字段过滤）。

---

## 8. API Key 管理

### 8.1 API Key 格式

- 前缀：`rk_` + base64url 编码的 32 字节随机数
- 存储：SHA-256 哈希后存储，原始 Key 仅在创建时返回一次
- 前缀展示：前 8 个字符用于识别（如 `rk_abcDEF`）

### 8.2 CRUD 接口

| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| POST | `/api-keys` | 创建 API Key | api-keys:manage |
| GET | `/api-keys` | 列表 | - |
| DELETE | `/api-keys/{id}` | 吊销 | api-keys:manage |

**创建 API Key** `POST /api/v1/admin/api-keys`

请求：
```json
{
  "name": "My Integration",
  "scopes": ["users:read", "departments:read"],
  "expires_at": "2027-01-01T00:00:00Z"
}
```

响应（**明文 Key 仅此一次返回**）：
```json
{
  "id": "uuid",
  "name": "My Integration",
  "key": "rk_dGhpcyBpcyBhIHRlc3Qg...",
  "key_prefix": "rk_dGhpcy",
  "scopes": ["users:read", "departments:read"],
  "expires_at": "2027-01-01T00:00:00Z"
}
```

### 8.3 Scope 权限

| Scope | 说明 |
|-------|------|
| `users:read` | 读取用户信息 |
| `departments:read` | 读取部门信息 |
| `*` | 通配符，允许所有权限 |

可自定义扩展更多 Scope。

### 8.4 安全机制

- API Key 认证支持 `Authorization: Bearer` 和 `X-API-Key` 两种方式
- 过期的 Key 自动失效（401）
- 被吊销的 Key 立即失效（401）
- 每次使用自动更新 `last_used_at`（异步，不影响响应时间）

---

## 9. OAuth2 第三方登录

### 9.1 支持的提供商

| 提供商 | 配置项 | 状态 |
|--------|--------|------|
| GitHub | `[api.github]` | ✅ 已实现 |
| Google | `[api.google]` | ✅ 已实现 |
| WeChat | `[api.wechat]` | 📋 预留 |

### 9.2 配置 OAuth

以 GitHub 为例：

```toml
[api.github]
client_id = "your-github-client-id"
client_secret = "your-github-client-secret"
```

GitHub OAuth App 配置：
1. 前往 GitHub → Settings → Developer settings → OAuth Apps → New OAuth App
2. Authorization callback URL 填写：`http://your-domain/api/v1/app/auth/callback?provider=github`
3. 将 Client ID 和 Client Secret 填入配置

### 9.3 OAuth 流程

```
┌──────────┐   1. GET /auth/authorize?provider=github   ┌──────────┐
│  App     │ ──────────────────────────────────────────→ │  Server  │
│          │ ←──── { url: "github.com/oauth/..." } ──── │          │
│          │                                              │          │
│  Browser │   2. 用户跳转到 GitHub 授权页面              │          │
│          │   3. GitHub 重定向回 callback                 │          │
│          │                                              │          │
│          │   4. GET /auth/callback?provider=github&code=xxx      │
│          │ ──────────────────────────────────────────→ │          │
│          │ ←──── { access_token, refresh_token } ──────│          │
│          │                                              │          │
│          │   (Token 的 client_type=app)                 │          │
└──────────┘                                              └──────────┘
```

### 9.4 自动注册/关联逻辑

OAuth 回调时的用户匹配策略：

1. **已有绑定**：`provider` + `provider_user_id` 匹配 → 直接登录
2. **同邮箱关联**：OAuth 邮箱与已有用户邮箱一致 → 自动关联并登录
3. **新用户**：创建新用户（随机密码）+ 关联 → 登录

### 9.5 绑定/解绑

已登录用户可以绑定/解绑第三方账号：

**绑定** `POST /api/v1/app/auth/social-bind`
```json
{
  "provider": "github",
  "code": "oauth-authorization-code"
}
```

**解绑** `POST /api/v1/app/auth/social-unbind`
```json
{
  "provider": "github"
}
```

---

## 10. Webhook 出站通知

### 10.1 概述

Webhook 允许你订阅系统事件，当事件触发时，系统会向指定 URL 发送 HTTP POST 请求。

特性：
- HMAC-SHA256 签名验证
- 自动重试（指数退避：60s → 5min → 30min，最多 3 次）
- 投递日志查询

### 10.2 CRUD 接口

| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| POST | `/webhooks` | 创建 Webhook | webhooks:manage |
| GET | `/webhooks` | Webhook 列表 | - |
| PUT | `/webhooks/{id}` | 更新 Webhook | webhooks:manage |
| DELETE | `/webhooks/{id}` | 删除 Webhook | webhooks:manage |
| GET | `/webhooks/{id}/deliveries` | 投递日志 | webhooks:manage |

**创建 Webhook** `POST /api/v1/admin/webhooks`

请求：
```json
{
  "name": "User Sync",
  "url": "https://example.com/webhook",
  "events": ["user.created", "user.updated", "user.deleted"]
}
```

响应：
```json
{
  "id": "uuid",
  "name": "User Sync",
  "url": "https://example.com/webhook",
  "events": ["user.created", "user.updated", "user.deleted"],
  "status": "active",
  "created_at": "2026-05-25T10:00:00Z",
  "updated_at": "2026-05-25T10:00:00Z"
}
```

### 10.3 可订阅事件

| 事件 | 触发时机 |
|------|---------|
| `user.created` | 创建用户 |
| `user.updated` | 更新用户 |
| `user.deleted` | 删除用户 |

### 10.4 投递格式

系统向 Webhook URL 发送 POST 请求，包含以下 Headers：

| Header | 说明 |
|--------|------|
| `X-Webhook-Signature` | HMAC-SHA256 签名（base64url） |
| `X-Webhook-Event` | 事件名称 |
| `X-Webhook-ID` | 投递记录 ID |

Body 示例：
```json
{
  "event": "user.created",
  "timestamp": "2026-05-25T10:00:00Z",
  "data": {
    "id": "uuid",
    "email": "new@example.com"
  }
}
```

### 10.5 签名验证

接收方可以用签名验证请求真实性：

```python
import hmac, hashlib, base64

def verify_signature(secret: str, payload: bytes, signature: str) -> bool:
    expected = base64.urlsafe_b64encode(
        hmac.new(secret.encode(), payload, hashlib.sha256).digest()
    ).decode().rstrip('=')
    return hmac.compare_digest(expected, signature)

# 示例
verify_signature("webhook-secret", request.body, request.headers["X-Webhook-Signature"])
```

### 10.6 重试机制

- 首次投递失败后进入重试队列
- 重试间隔：60s → 5min → 30min（可配置）
- 最大重试次数：3 次（可配置）
- 响应状态码非 2xx 视为失败

---

## 11. 错误格式

### 11.1 统一错误响应

所有错误使用统一格式：

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable message",
    "request_id": "req_xxx",
    "details": null
  },
  "status": 400
}
```

### 11.2 错误码对照表

| HTTP 状态 | 错误码 | 说明 |
|-----------|--------|------|
| 400 | `BAD_REQUEST` | 请求参数错误 |
| 401 | `UNAUTHORIZED` | 未认证 |
| 401 | `TWO_FACTOR_REQUIRED` | 需要两步验证 |
| 403 | `FORBIDDEN` | 无权限 |
| 403 | `ACCOUNT_DISABLED` | 账号已禁用 |
| 403 | `ACCOUNT_LOCKED` | 账号已锁定 |
| 404 | `NOT_FOUND` | 资源不存在 |
| 409 | `CONFLICT` | 资源冲突（如邮箱已注册） |
| 413 | `PAYLOAD_TOO_LARGE` | 请求体过大 |
| 429 | `TOO_MANY_REQUESTS` | 请求频率过高 |
| 500 | `INTERNAL_ERROR` | 服务器内部错误 |

### 11.3 Request ID

每个请求都会在响应头中附带 `X-Request-Id`，客户端也可在请求中传入自定义值用于链路追踪。

---

## 12. 权限体系

### 12.1 角色层级

| 角色 | 说明 |
|------|------|
| `superadmin` | 超级管理员，拥有所有权限 |
| `admin` | 管理员，按分配的权限操作 |
| `user` | 普通用户，基本权限 |

### 12.2 权限列表

| 权限 | 说明 |
|------|------|
| `users:read` | 查看用户 |
| `users:create` | 创建用户 |
| `users:update` | 更新用户 |
| `users:delete` | 删除用户 |
| `roles:read` | 查看角色 |
| `roles:create` | 创建角色 |
| `roles:update` | 更新角色 |
| `roles:delete` | 删除角色 |
| `departments:read` | 查看部门 |
| `departments:create` | 创建部门 |
| `departments:update` | 更新部门 |
| `departments:delete` | 删除部门 |
| `dicts:read` | 查看字典 |
| `dicts:create` | 创建字典 |
| `dicts:update` | 更新字典 |
| `dicts:delete` | 删除字典 |
| `menus:read` | 查看菜单 |
| `menus:create` | 创建菜单 |
| `menus:update` | 更新菜单 |
| `menus:delete` | 删除菜单 |
| `audit:read` | 查看审计日志 |
| `sessions:read` | 查看会话 |
| `sessions:delete` | 终止会话 |
| `files:read` | 查看文件 |
| `files:upload` | 上传文件 |
| `files:delete` | 删除文件 |
| `configs:read` | 查看系统配置 |
| `configs:update` | 更新系统配置 |
| `notifications:read` | 查看通知 |
| `notifications:broadcast` | 广播通知 |
| `api-keys:manage` | 管理 API Key |
| `webhooks:manage` | 管理 Webhook |
| `login-logs:read` | 查看登录日志 |
| `import:execute` | 执行数据导入 |
| `export:execute` | 执行数据导出 |
| `search:execute` | 执行全局搜索 |
| `*` | 通配符（所有权限） |

---

## 13. 数据权限

### 13.1 数据范围（Data Scope）

角色可配置数据可见范围：

| Data Scope | 说明 |
|------------|------|
| `all` | 查看所有数据 |
| `department` | 仅本部门 |
| `department_and_sub` | 本部门及下级部门 |
| `self` | 仅本人数据 |

### 13.2 影响范围

数据权限影响以下接口的返回结果：
- 用户列表（`GET /users`）
- 审计日志（`GET /audit-logs`）
- 登录日志（`GET /login-logs`）
- 会话列表（`GET /sessions`）

---

## 14. 测试

### 14.1 运行测试

```bash
cd apps/backend

# 全部测试（需要运行中的 PostgreSQL）
cargo test

# Phase 1-7 集成测试
cargo test --test integration_test

# Phase 8 API 测试
cargo test --test phase8_api_tests

# 单个测试
cargo test test_login_success

# 显示输出
cargo test -- --nocapture
```

### 14.2 测试统计

| 测试套件 | 数量 | 覆盖范围 |
|----------|------|---------|
| integration_test | 35 | 认证、用户 CRUD、角色、部门、字典、导入导出 |
| phase8_api_tests | 26 | Request ID、路由版本化、API Key、App Auth、Webhook |

### 14.3 注意事项

- 测试使用共享数据库，非隔离的
- 登录限流 5RPM 会影响并发测试，需要间隔
- 并发执行可能导致 admin 账号被错误锁定
- 测试前确保 `admin` 账号的 2FA 处于关闭状态

---

## 15. 部署

### 15.1 Docker Compose（推荐）

```bash
# 构建并启动
docker compose up -d

# 查看日志
docker compose logs -f backend

# 停止
docker compose down
```

服务地址：
- 后端：http://localhost:8080
- 前端：http://localhost:3000
- PostgreSQL：localhost:5432

### 15.2 手动部署

**后端：**
```bash
cd apps/backend

# 构建 release
cargo build --release

# 运行
./target/release/cradle-backend
```

**前端：**
```bash
cd apps/frontend

# 构建
pnpm build

# 产物在 dist/ 目录，用 Nginx 托管
```

### 15.3 生产环境检查清单

- [ ] 修改 `jwt.secret` 为强随机密钥
- [ ] 修改 `totp.encryption_key` 为随机 Base64 密钥
- [ ] 配置 PostgreSQL 连接使用强密码
- [ ] 设置 `storage.upload_dir` 为持久化目录
- [ ] 配置 OAuth2 的 client_id / client_secret
- [ ] 按需关闭 `api.legacy_routes_enabled`
- [ ] 配置反向代理（Nginx）的 CORS 和 HTTPS
- [ ] 设置日志级别（`RUST_LOG=info`）

### 15.4 环境变量

| 变量 | 说明 |
|------|------|
| `APP__DATABASE__URL` | 数据库连接字符串 |
| `APP__JWT__SECRET` | JWT 签名密钥 |
| `APP__SERVER__PORT` | 监听端口 |
| `RUST_LOG` | 日志级别（`debug`, `info`, `warn`, `error`） |

---

## 16. FAQ

### Q: 登录返回 TWO_FACTOR_REQUIRED 但我已禁用 2FA？

检查 `users` 表中 `two_factor_enabled` 是否为 `false`。可能手动测试时开启了 2FA 未关闭。测试框架会自动重置。

### Q: 旧路由还能用吗？

可以。所有 `/api/*` 路由完全兼容，只是响应头会带 `Deprecation: true`。设置 `api.legacy_routes_enabled = false` 可关闭。

### Q: API Key 创建后能看到明文吗？

不能。明文 Key 仅在创建响应中返回一次。数据库只存储 SHA-256 哈希。如果丢失需要吊销后重新创建。

### Q: Webhook 投递失败怎么办？

系统自动重试 3 次（间隔 60s/5min/30min）。可以通过 `GET /webhooks/{id}/deliveries` 查看投递日志和失败原因。

### Q: 如何给第三方接入方创建 API Key？

1. 使用管理员账号登录
2. 调用 `POST /api/v1/admin/api-keys` 创建 Key（需要 `api-keys:manage` 权限）
3. 将返回的 `rk_xxx` 明文 Key 安全地交给接入方
4. 接入方使用 `Authorization: Bearer rk_xxx` 或 `X-API-Key: rk_xxx` 请求 Open API

### Q: 数据库迁移怎么执行？

后端启动时自动执行迁移。也可以手动执行：

```bash
cd apps/backend
sqlx migrate run          # 执行迁移
sqlx migrate revert       # 回退最后一次迁移
sqlx migrate info         # 查看迁移状态
```

### Q: 前端如何切换中英文？

前端使用 `i18next`，通过 Header 右上角的语言切换按钮切换，或调用 `GET /api/i18n/messages` 获取翻译文件。

### Q: Swagger UI 在哪里？

启动后端后访问 `http://localhost:8080/docs`。可通过 `api.docs_enabled = false` 关闭。
