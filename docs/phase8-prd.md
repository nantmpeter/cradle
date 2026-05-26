# Phase 8 PRD — 多端 API 基础设施

## 项目信息

- **Language**: 中文
- **Programming Language**: Rust (Axum + SQLx + PostgreSQL) + React 19 (Vite + TypeScript + shadcn/ui + Tailwind)
- **Project Name**: cradle
- **原始需求**: 将 cradle 从单一 Web 管理后台后端升级为面向多客户端的 API 基础设施，支持 Admin API（Web 管理后台）、App API（移动端/小程序）、Open API（第三方调用）三类端点，引入路由版本化、API Key 认证、OAuth2 第三方登录、OpenAPI 文档和 Webhook 出站能力。

---

## 1. 产品目标

| # | 目标 | 说明 |
|---|------|------|
| G1 | **多端 API 分层** | 将现有扁平的 `/api/*` 路由体系重构为 `/api/v1/admin/*`、`/api/v1/app/*`、`/api/v1/open/*` 三层架构，实现客户端隔离与差异化认证。 |
| G2 | **开放生态接入** | 通过 API Key 认证 + Open API 端点 + Webhook 出站机制，使第三方系统能安全、规范地集成 cradle 的用户/权限/数据能力。 |
| G3 | **移动端就绪** | 通过 OAuth2 标准协议和微信/手机登录支持 App API 子集，让 App、小程序、H5 能以最小权限访问系统核心功能。 |

---

## 2. 用户故事

### Admin API（Web 管理后台）

| # | 角色 | 故事 |
|---|------|------|
| US-A1 | 系统管理员 | 我希望管理后台的 API 路径升级到 `/api/v1/admin/*` 但旧路径 `/api/*` 仍能兼容访问，以便平滑迁移不中断线上服务。 |
| US-A2 | 系统管理员 | 我希望每个 API 响应都包含 `request_id` 字段和统一错误格式，以便排查问题时能快速定位到具体请求。 |
| US-A3 | 系统管理员 | 我希望在管理后台能创建、查看、吊销 API Key，以便控制第三方系统的访问权限。 |

### App API（移动端/小程序）

| # | 角色 | 故事 |
|---|------|------|
| US-B1 | App 用户 | 我希望能通过微信扫码或手机号验证码登录 App，以便不需要记忆额外的用户名密码。 |
| US-B2 | App 用户 | 我希望 App 端只能访问受限的功能接口（个人信息、通知、文件），以便我的管理权限不会在移动端泄露。 |

### Open API（第三方系统）

| # | 角色 | 故事 |
|---|------|------|
| US-C1 | 第三方开发者 | 我希望通过 API Key 调用用户查询、部门列表等只读接口，以便在我的业务系统中展示组织架构数据。 |
| US-C2 | 第三方开发者 | 我希望能订阅 Webhook 事件（用户创建、角色变更等），以便实时同步数据而无需轮询。 |
| US-C3 | 第三方开发者 | 我希望有 OpenAPI 3.0 规范的交互式文档，以便快速了解接口参数和返回格式。 |

---

## 3. 需求池

### P0 — Must Have

#### 8A: 路由版本化 + 统一错误格式 + Request ID

| 需求ID | 需求 | 说明 |
|--------|------|------|
| 8A-01 | 路由版本化重构 | 将现有 `/api/*` 路由迁移到 `/api/v1/admin/*`，保留完整的管理后台能力。路由注册在 `routes/mod.rs` 中改为嵌套结构。 |
| 8A-02 | 旧路由兼容映射 | 旧路径 `/api/*` 通过 301/302 重定向或内部 fallback 路由映射到 `/api/v1/admin/*`，确保前端无需立即修改。兼容期默认 2 个版本。 |
| 8A-03 | 统一错误响应格式 | 标准化错误响应为 `{ "error": { "code": "ERROR_CODE", "message": "...", "request_id": "uuid", "details": null } }`，替代当前的 `{ "error": "...", "status": 401 }` 格式。保持向后兼容：旧字段 `error`（string）仍保留，新增 `code` 和 `request_id`。 |
| 8A-04 | Request ID 中间件 | 新增 `RequestIdLayer` 中间件，为每个入站请求生成 UUID v4 的 `request_id`，写入响应头 `X-Request-Id`，并注入到请求扩展（extension）中供 handler 和错误处理使用。 |
| 8A-05 | App API 路由骨架 | 建立 `/api/v1/app/*` 路由骨架，复用 JWT 认证中间件，限定仅暴露安全子集：`/me`、`/me/password`、`/notifications/*`、`/files/*`。无管理类操作（无 CRUD 用户/角色/部门等）。 |
| 8A-06 | Open API 路由骨架 | 建立 `/api/v1/open/*` 路由骨架，使用独立的 API Key 认证中间件（与 JWT 中间件并列，互不干扰）。 |

**验收标准**：
- AC-8A-01: `GET /api/v1/admin/users` 返回与原 `GET /api/users` 相同的数据
- AC-8A-02: `GET /api/users` 返回 301 重定向到 `/api/v1/admin/users`（或直接代理返回相同数据）
- AC-8A-03: 所有错误响应包含 `request_id` 和 `code` 字段
- AC-8A-04: 响应头包含 `X-Request-Id`
- AC-8A-05: `GET /api/v1/app/me` 使用 JWT 认证成功返回当前用户信息
- AC-8A-06: `GET /api/v1/open/health` 返回 200（无需认证的健康检查）

---

#### 8B: API Key 认证 + Open API 基础端点

| 需求ID | 需求 | 说明 |
|--------|------|------|
| 8B-01 | `api_keys` 数据表 | `id`, `name`(名称), `key_hash`(SHA-256 哈希), `key_prefix`(前8字符用于展示), `user_id`(创建者), `permissions`(JSON 权限列表), `scopes`(JSON 作用域，如 `["users:read","departments:read"]`), `rate_limit`(每分钟限制, nullable), `expires_at`(过期时间, nullable), `last_used_at`, `status`(active/revoked), `created_at`, `updated_at` |
| 8B-02 | API Key 认证中间件 | 新增 `require_api_key` 中间件：从 `Authorization: Bearer rk_xxx` 或 `X-API-Key: rk_xxx` 提取 Key，SHA-256 哈希后查库，验证状态和过期时间，将 `ApiKey` 结构体注入请求扩展。 |
| 8B-03 | `ApiKey` Extractor | 类似 `AuthUser`，实现 `FromRequestParts`，提供 `key_id`、`scopes`、`name` 字段，以及 `require_scope(&self, scope: &str)` 权限检查方法。 |
| 8B-04 | API Key CRUD（Admin API） | `POST /api/v1/admin/api-keys`（创建，返回明文 key 仅一次）、`GET /api/v1/admin/api-keys`（列表）、`DELETE /api/v1/admin/api-keys/{id}`（吊销）。创建时需要 `api-keys:manage` 权限。 |
| 8B-05 | Open API 健康检查 | `GET /api/v1/open/health`（无需认证），返回系统版本、启动时间等基本信息。 |
| 8B-06 | Open API 用户只读接口 | `GET /api/v1/open/users`（列表，需 `users:read` scope）、`GET /api/v1/open/users/{id}`（详情，需 `users:read` scope）。仅返回非敏感字段（排除 password_hash、totp_secret 等）。 |
| 8B-07 | Open API 部门只读接口 | `GET /api/v1/open/departments`（列表，需 `departments:read` scope）。 |
| 8B-08 | API Key 使用审计 | 每次 Open API 请求自动写入审计日志（action: `open_api.request`，resource_type: `api_key`，resource_id: key_id）。 |
| 8B-09 | API Key 管理前端页 | Admin 管理后台新增 API Key 管理页 `/dashboard/api-keys`：列表（名称、前缀、状态、创建时间、最后使用）、创建弹窗（名称、权限勾选、过期时间）、吊销确认。 |

**验收标准**：
- AC-8B-01: 创建 API Key 后，使用该 Key 调用 `GET /api/v1/open/users` 返回 200
- AC-8B-02: 不带 Key 调用 Open API 端点返回 401
- AC-8B-03: 使用被吊销的 Key 调用返回 401 + "API key has been revoked"
- AC-8B-04: Key 缺少 `users:read` scope 调用用户接口返回 403
- AC-8B-05: 创建 Key 时响应体包含完整明文 Key，列表接口仅展示前缀
- AC-8B-06: 每个 Open API 请求在 `audit_logs` 表中有对应记录
- AC-8B-07: Admin 前端 API Key 管理页能完成创建、查看、吊销完整流程

---

### P1 — Should Have

#### 8C: OAuth2 + 第三方登录 + App API

| 需求ID | 需求 | 说明 |
|--------|------|------|
| 8C-01 | `oauth_accounts` 数据表 | `id`, `user_id`(关联用户), `provider`(wechat/phone/github 等), `provider_user_id`(第三方用户标识，如微信 openid), `union_id`(可选), `created_at`。支持一个用户绑定多个第三方账号。 |
| 8C-02 | OAuth2 授权码流程 | 实现 `/api/v1/app/auth/authorize`（重定向到第三方授权页）和 `/api/v1/app/auth/callback`（回调，换取 token 并创建/关联本地用户），遵循 RFC 6749。 |
| 8C-03 | 微信登录集成 | 支持微信开放平台（PC 网页扫码）和微信小程序（`wx.login` code 换 session）两种模式。配置项：`wechat_app_id`、`wechat_app_secret`。 |
| 8C-04 | 手机号验证码登录 | `POST /api/v1/app/auth/sms/send`（发送验证码）、`POST /api/v1/app/auth/sms/verify`（验证码校验+登录/注册）。验证码存储在 Redis 或数据库，5 分钟过期。 |
| 8C-05 | App Token 区分 | JWT Claims 新增 `client_type: "admin" | "app"` 字段，App API 中间件校验 `client_type == "app"`，防止管理端 Token 被用于 App API。 |
| 8C-06 | App API 扩展端点 | 除 8A 骨架外，增加：`GET /api/v1/app/auth/providers`（获取可用的第三方登录方式列表）、`POST /api/v1/app/auth/social-bind`（绑定已有账号）、`POST /api/v1/app/auth/social-unbind`（解绑）。 |

**验收标准**：
- AC-8C-01: 微信授权回调后，首次用户自动创建账号并返回 JWT
- AC-8C-02: 已绑定微信的用户再次扫码直接登录，返回 JWT
- AC-8C-03: 手机验证码登录成功返回 `client_type: "app"` 的 JWT
- AC-8C-04: 使用 Admin 端 JWT 调用 `GET /api/v1/app/me` 返回 403
- AC-8C-05: `GET /api/v1/app/auth/providers` 返回已配置的登录方式列表

---

#### 8D: OpenAPI 文档 + Webhook 出站

| 需求ID | 需求 | 说明 |
|--------|------|------|
| 8D-01 | OpenAPI 3.0 Spec 生成 | 使用 `utoipa` crate 通过代码注解自动生成 OpenAPI 3.0 JSON/YAML spec，覆盖 Admin/App/Open 三端路由。 |
| 8D-02 | Swagger UI | 集成 `utoipa-swagger-ui`，提供 `/docs` 路径的交互式 API 文档页面（仅 Admin 可访问或通过配置开放）。 |
| 8D-03 | `webhooks` 配置表 | `id`, `name`, `url`(回调地址), `secret`(签名密钥), `events`(JSON 事件列表，如 `["user.created","user.updated"]`), `api_key_id`(关联的 API Key, nullable), `status`(active/disabled), `created_at`, `updated_at` |
| 8D-04 | Webhook 出站投递 | 当匹配事件发生时（如用户创建、角色变更），异步向配置的 URL 发送 HTTP POST，body 为事件 payload，header 含 `X-Webhook-Signature`(HMAC-SHA256) 和 `X-Webhook-Event`。 |
| 8D-05 | `webhook_deliveries` 日志表 | `id`, `webhook_id`, `event`, `payload`(JSON), `response_status`, `response_body`, `delivered_at`, `next_retry_at`(可空), `retry_count`。记录每次投递结果。 |
| 8D-06 | Webhook 重试机制 | 失败的投递（非 2xx）按指数退避重试，最多 3 次（1min → 5min → 30min）。 |
| 8D-07 | Webhook 管理接口（Admin API） | `POST /api/v1/admin/webhooks`（创建）、`GET`（列表）、`PUT /{id}`（更新）、`DELETE /{id}`（删除）、`GET /{id}/deliveries`（查看投递日志）。需要 `webhooks:manage` 权限。 |

**验收标准**：
- AC-8D-01: `/docs` 页面正常展示所有已注册路由的 API 文档
- AC-8D-02: Swagger UI 能直接测试 Admin API 端点（需输入 JWT）
- AC-8D-03: 创建 Webhook 订阅 `user.created` 事件后，通过 Admin API 创建用户，目标 URL 收到 POST 请求
- AC-8D-04: POST 请求的 `X-Webhook-Signature` 能用 secret 验证通过
- AC-8D-05: 目标 URL 返回 500 时，投递记录 retry_count 递增且有 next_retry_at
- AC-8D-06: 重试 3 次后仍然失败，不再继续重试

---

### P2 — Nice to Have

| 需求ID | 需求 | 说明 |
|--------|------|------|
| P2-01 | API Key 批量操作 | 支持批量吊销、批量更新 scope |
| P2-02 | Open API 字典只读接口 | `GET /api/v1/open/dicts/{code}`，供第三方获取字典数据 |
| P2-03 | Webhook 事件过滤表达式 | 支持 JQ 或简易表达式过滤事件内容（如仅在 `role == "admin"` 时触发） |
| P2-04 | API 版本协商 | 支持 `Accept: application/vnd.cradle.v1+json` 头进行版本协商 |
| P2-05 | API Rate Limit 按端独立配置 | Admin/App/Open 各自配置不同的速率限制策略 |
| P2-06 | OpenAPI 文档按端分组 | Swagger UI 中可通过 tag 切换查看 Admin/App/Open API |
| P2-07 | API Key IP 白名单 | 创建 Key 时可选绑定 IP 白名单 |

---

## 4. 技术约束与向后兼容

### 4.1 当前架构现状

基于代码分析，当前系统：

- **路由结构**：所有路由挂载在 `/api/*` 下，在 `lib.rs` 的 `app()` 函数中通过 `Router::merge()` 逐个合并
- **认证中间件**：`middleware/auth.rs` 中的 `require_auth` 解析 JWT Bearer Token，构建 `AuthUser` extension
- **AuthUser Extractor**：`extractors/auth.rs` 中从请求扩展获取已认证用户，支持 `is_admin()`、`require_permission()`、`get_visible_department_ids()` 等方法
- **错误处理**：`AppError` 枚举通过 `IntoResponse` 转为 `{ "error": "...", "status": N }` 格式
- **配置**：`config/default.toml` 包含 server、database、jwt、rate_limit、storage、totp 六组配置

### 4.2 兼容性策略

```
迁移期（Phase 8 发布后 2 个 minor 版本内）：

/api/auth/login          → 内部代理到 /api/v1/admin/auth/login（无重定向，直接处理）
/api/users               → 内部代理到 /api/v1/admin/users
/api/v1/admin/users      → 新路径，正常处理
/api/v1/app/me           → 新路径，App 专用
/api/v1/open/users       → 新路径，API Key 认证

废弃期（2 个 minor 版本后）：

/api/users               → 301 永久重定向到 /api/v1/admin/users
响应头增加 Deprecation: true
```

### 4.3 认证中间件路由策略

```
/api/v1/admin/*  → require_auth (JWT) + RBAC permission check
/api/v1/app/*    → require_auth (JWT, client_type=app)
/api/v1/open/*   → require_api_key (API Key) + scope check
/api/auth/*      → 旧路由兼容，使用 require_auth（迁移期）
```

### 4.4 配置扩展

在 `config/default.toml` 中新增：

```toml
[api]
version = "v1"
legacy_routes_enabled = true     # 是否启用旧路由兼容
docs_enabled = true              # 是否启用 Swagger UI

[api.wechat]
enabled = false
app_id = ""
app_secret = ""

[api.sms]
enabled = false
provider = ""                    # sms provider 名称
template_id = ""

[webhook]
max_retries = 3
retry_intervals_secs = [60, 300, 1800]
timeout_secs = 10
```

---

## 5. 数据库变更概要

### 新增表

```sql
-- API Keys 表
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    key_hash VARCHAR(64) NOT NULL UNIQUE,      -- SHA-256 hex
    key_prefix VARCHAR(8) NOT NULL,             -- 前8字符，用于展示识别
    user_id UUID NOT NULL REFERENCES users(id),
    scopes JSONB NOT NULL DEFAULT '[]',         -- ["users:read", "departments:read"]
    rate_limit INT,                             -- 每分钟请求限制（null 使用全局）
    expires_at TIMESTAMPTZ,                     -- 过期时间（null 表示永不过期）
    last_used_at TIMESTAMPTZ,
    status VARCHAR(20) NOT NULL DEFAULT 'active',  -- active / revoked
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_api_keys_key_hash ON api_keys(key_hash);
CREATE INDEX idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX idx_api_keys_status ON api_keys(status);

-- OAuth 第三方账号表
CREATE TABLE oauth_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,              -- wechat / phone / github
    provider_user_id VARCHAR(255) NOT NULL,     -- openid / phone number
    union_id VARCHAR(255),                      -- 微信 unionid
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_oauth_accounts_provider_user ON oauth_accounts(provider, provider_user_id);
CREATE INDEX idx_oauth_accounts_user_id ON oauth_accounts(user_id);

-- Webhook 配置表
CREATE TABLE webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    url TEXT NOT NULL,
    secret VARCHAR(64) NOT NULL,
    events JSONB NOT NULL DEFAULT '[]',         -- ["user.created", "user.updated"]
    api_key_id UUID REFERENCES api_keys(id),
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Webhook 投递日志表
CREATE TABLE webhook_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    webhook_id UUID NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,
    event VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    response_status INT,
    response_body TEXT,
    delivered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    next_retry_at TIMESTAMPTZ,
    retry_count INT NOT NULL DEFAULT 0
);

CREATE INDEX idx_webhook_deliveries_webhook_id ON webhook_deliveries(webhook_id);
CREATE INDEX idx_webhook_deliveries_next_retry ON webhook_deliveries(next_retry_at) WHERE next_retry_at IS NOT NULL;
```

### 新增权限种子

```sql
INSERT INTO permissions (name, description, module) VALUES
    ('api-keys:manage', 'Create, view and revoke API keys', 'api_keys'),
    ('webhooks:manage', 'Manage webhook configurations', 'webhooks');
```

---

## 6. API 路由设计（新增）

### Admin API（JWT 认证）

| Method | Path | 说明 |
|--------|------|------|
| POST | `/api/v1/admin/api-keys` | 创建 API Key |
| GET | `/api/v1/admin/api-keys` | API Key 列表 |
| DELETE | `/api/v1/admin/api-keys/{id}` | 吊销 API Key |
| POST | `/api/v1/admin/webhooks` | 创建 Webhook |
| GET | `/api/v1/admin/webhooks` | Webhook 列表 |
| PUT | `/api/v1/admin/webhooks/{id}` | 更新 Webhook |
| DELETE | `/api/v1/admin/webhooks/{id}` | 删除 Webhook |
| GET | `/api/v1/admin/webhooks/{id}/deliveries` | Webhook 投递日志 |

> 注：现有 Admin 路由（users/roles/departments 等）迁移到 `/api/v1/admin/*` 下保持不变。

### App API（JWT 认证，client_type=app）

| Method | Path | 说明 |
|--------|------|------|
| GET | `/api/v1/app/auth/providers` | 获取可用第三方登录方式 |
| POST | `/api/v1/app/auth/sms/send` | 发送短信验证码 |
| POST | `/api/v1/app/auth/sms/verify` | 短信验证码登录 |
| GET | `/api/v1/app/auth/authorize` | OAuth2 授权重定向 |
| GET | `/api/v1/app/auth/callback` | OAuth2 回调 |
| POST | `/api/v1/app/auth/social-bind` | 绑定第三方账号 |
| POST | `/api/v1/app/auth/social-unbind` | 解绑第三方账号 |
| GET | `/api/v1/app/me` | 当前用户信息 |
| PUT | `/api/v1/app/me/password` | 修改密码 |
| POST | `/api/v1/app/me/avatar` | 上传头像 |
| GET | `/api/v1/app/notifications` | 通知列表 |
| POST | `/api/v1/app/notifications/{id}/read` | 标记已读 |

### Open API（API Key 认证）

| Method | Path | 说明 |
|--------|------|------|
| GET | `/api/v1/open/health` | 健康检查（无需认证） |
| GET | `/api/v1/open/users` | 用户列表（需 `users:read` scope） |
| GET | `/api/v1/open/users/{id}` | 用户详情（需 `users:read` scope） |
| GET | `/api/v1/open/departments` | 部门列表（需 `departments:read` scope） |

### 文档

| Method | Path | 说明 |
|--------|------|------|
| GET | `/docs` | Swagger UI |
| GET | `/docs/openapi.json` | OpenAPI 3.0 JSON Spec |

---

## 7. 前端变更

仅 Admin 管理后台需要前端改动：

### 7.1 API Key 管理页 `/dashboard/api-keys`

- **列表视图**：表格展示 API Key 名称、前缀、Scopes 标签、状态 Badge、创建时间、最后使用时间
- **创建弹窗**：名称输入、Scopes 多选（users:read / departments:read / roles:read 等）、过期时间选择（可选）
- **创建结果弹窗**：完整 Key 明文展示 + 警告提示"请立即复制，此后不再展示"
- **吊销操作**：行内按钮 + 确认弹窗

### 7.2 Webhook 管理页 `/dashboard/webhooks`（P1）

- **列表视图**：名称、URL、订阅事件标签、状态、操作
- **创建/编辑弹窗**：名称、URL、事件多选（user.created / user.updated / role.updated 等）
- **投递日志弹窗**：点击某 Webhook 展开投递历史表格（时间、状态码、重试次数）

---

## 8. 实现依赖与建议顺序

```
Phase 8A（基础层，所有后续模块依赖）
  ├── 路由版本化重构
  ├── Request ID 中间件
  ├── 统一错误格式
  └── App/Open 路由骨架
       │
       ▼
Phase 8B（开放能力）
  ├── api_keys 表 + 中间件 + Extractor
  ├── Open API 端点
  ├── 审计集成
  └── Admin 前端 API Key 管理页
       │
       ▼
Phase 8C（移动端）— 可与 8D 并行
  ├── oauth_accounts 表
  ├── OAuth2 授权码流程
  ├── 微信/手机登录
  └── App Token client_type 区分
       │
       ▼
Phase 8D（开发者体验）
  ├── utoipa 集成 + Swagger UI
  ├── webhooks 表 + 投递 + 重试
  └── Admin 前端 Webhook 管理页
```

---

## 9. 待确认问题

| # | 问题 | 影响 |
|---|------|------|
| Q1 | 旧路由兼容期多长？建议 2 个 minor 版本后切换为 301 重定向。 | 影响兼容映射的实现复杂度 |
| Q2 | 微信登录需要开放平台还是仅小程序？是否需要支持微信支付场景？ | 影响配置项和 OAuth2 流程分支 |
| Q3 | 短信验证码服务选型：使用哪个 SMS Provider（阿里云/腾讯云/Twilio）？ | 影响配置项和 Provider 抽象层设计 |
| Q4 | OpenAPI 文档是否需要登录才能访问？建议默认仅 Admin JWT 可访问，生产环境可配置关闭。 | 影响路由注册和权限设计 |
| Q5 | Webhook 出站使用什么 HTTP Client？建议使用 `reqwest`，是否需要支持 HTTPS 证书验证配置？ | 影响依赖和配置 |
| Q6 | App API 的 `/me` 是否需要返回与 Admin 不同的字段子集？例如是否需要隐藏 `role_id`、`department_id` 等？ | 影响响应序列化逻辑 |
| Q7 | API Key 的 scopes 是否需要与现有 RBAC permissions 表打通，还是独立维护？ | 影响权限模型设计 |
