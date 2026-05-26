# Cradle

[English](./README.md)

> **每一个伟大系统的起点。**

每个产品都有一个起点。大多数从一个勉强能用的后台开始——能用，直到不能用的那天。团队越做越大，补丁越打越多，最终推倒重来。**Cradle** 就是那个不需要被替换的起点。

名字即使命：摇篮孕育其中的一切。Cradle 从第一天起就为你的产品提供内存安全的分层 API 架构——并且陪你度过每一次业务爆发。三层路由、Scope 级访问控制、事件驱动 Webhook、实时通知推送，不是上线后才补的功能，而是从摇篮里就准备好的能力。

**与生俱来的不同：**
- **内存安全，抗压不裂** — Rust + Axum 从根上消灭一整类运行时 bug。无 GC 停顿，无生产环境空指针。摇篮不会在压力下碎裂。
- **一套代码，三路出击** — 管理后台、移动端 App、第三方接入，各自拥有独立认证的 API 层。一个底座，所有方向。
- **约定优于配置** — 每个模块遵循相同的 7 层模式（迁移 → Model → Repo → Service → Handler → 路由 → 测试），新成员第一天就能交付功能——摇篮自带使用说明书。
- **实时能力内置** — SSE 推送通知、HMAC 签名的 Webhook 投递、OpenAPI 文档——全部内置，不是后补的补丁。

基于 **Rust**（Axum）和 **React**（Vite + shadcn/ui）构建。

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端 | Rust · Axum 0.8 · SQLx · PostgreSQL 16 |
| 前端 | React 19 · TypeScript · Vite · shadcn/ui · Tailwind CSS v4 |
| 状态管理 | Zustand · TanStack Query v5 |
| 认证 | JWT（access + refresh 双令牌）· Argon2 · TOTP 两步验证 · API Key · OAuth2 |
| 构建 | Cargo Workspace · Turborepo |

## 功能特性

### 基础功能（Phase 1-2）
- JWT 认证，access/refresh 令牌轮换
- RBAC（基于角色的访问控制），权限粒度到接口级别
- 用户增删改查、角色管理、状态切换

### 仪表盘与设置（Phase 3）
- 系统概览（版本号、运行时间、数据库状态）
- 用户资料页，支持显示名修改
- 密码修改，含强度校验

### 文件与会话管理（Phase 4）
- 文件上传/下载，支持大小限制
- 用户数据导出（CSV/XLSX），支持列选择
- 在线会话管理
- 数据库驱动的动态侧边栏菜单
- 国际化（中文/英文）

### 安全增强（Phase 5）
- TOTP 两步验证（设置/启用/禁用）
- 站内通知系统
- 增强审计日志（资源类型/操作详情）
- 系统配置键值对管理

### 组织架构（Phase 6）
- 部门管理（树形结构）
- 字典管理（字典类型 + 字典项）
- 登录日志追踪，含 UA 解析
- Excel 批量导入用户

### 体验优化（Phase 7）
- 全局搜索（用户/角色/部门/字典）
- 数据权限（部门级隔离）
- SSE 实时通知推送
- 用户头像上传
- 面包屑导航
- Tab 标签页导航
- 主题定制（8 种主题色）
- 强制下线在线用户

### API 基础设施（Phase 8）
- **三层 API 路由**：Admin（JWT+RBAC）· App（JWT+client_type）· Open（API Key+Scope）
- **路由版本化**：`/api/v1/admin/*`、`/api/v1/app/*`、`/api/v1/open/*`，旧路由 `/api/*` 兼容
- **API Key 管理**：创建、列表、吊销，SHA-256 哈希存储，Scope 权限控制
- **OAuth2 集成**：GitHub/Google 授权码流程，自动注册/关联账号
- **短信验证码登录**：手机号 + 6 位验证码认证
- **Webhook 出站框架**：事件驱动的消息投递，HMAC-SHA256 签名，指数退避重试
- **OpenAPI 文档**：Swagger UI（`/docs`），可配置开关
- **Request ID 中间件**：X-Request-Id 请求链路追踪
- **统一错误格式**：`{ error: { code, message, request_id, details }, status }`
- 61 个集成测试全部通过

## 项目结构

```
cradle/
├── apps/
│   ├── backend/               # Rust Axum API 服务
│   │   ├── config/            # TOML 配置文件
│   │   ├── migrations/        # SQLx 数据库迁移
│   │   └── src/
│   │       ├── extractors/    # 自定义 Axum 提取器（AuthUser、ApiKeyContext、RequestId）
│   │       ├── handlers/      # 路由处理器（25 个模块）
│   │       ├── middleware/     # 认证、限流、API Key、Request ID、Deprecation
│   │       ├── models/        # 数据模型 + DTO
│   │       ├── repository/    # 数据库访问层
│   │       ├── routes/        # 路由注册
│   │       └── services/      # 业务逻辑层
│   └── frontend/              # React 单页应用
│       └── src/
│           ├── components/    # UI 组件（18 个功能模块）
│           ├── hooks/         # React Query hooks
│           ├── lib/           # 工具函数、API 客户端、主题
│           ├── locales/       # 国际化 JSON（en-US, zh-CN）
│           ├── stores/        # Zustand 状态管理
│           └── types/         # TypeScript 类型定义
├── docs/                      # 设计文档（PRD、架构设计、类图、时序图）
├── docker-compose.yml
├── turbo.json
└── Cargo.toml                 # Workspace 根配置
```

## 快速开始

### 环境要求

- Rust 1.85+（edition 2024）
- Node.js 20+
- PostgreSQL 16
- pnpm（推荐）

### 1. 数据库

```bash
# 方式 A：Docker
docker compose up db -d

# 方式 B：本地 PostgreSQL
createdb cradle
```

### 2. 启动后端

```bash
cd apps/backend

# 复制并编辑配置（按需调整数据库连接地址）
cp config/default.toml config/local.toml

# 运行数据库迁移（启动时自动执行，也可手动运行）
sqlx migrate run

# 启动开发服务器
cargo run
# → http://localhost:8080
```

### 3. 启动前端

```bash
cd apps/frontend

# 安装依赖
pnpm install

# 启动开发服务器
pnpm dev
# → http://localhost:5173
```

### 4. 登录

| 邮箱 | 密码 |
|------|------|
| `admin@example.com` | `Admin@1234` |

## API 架构

### 三层路由体系

所有 API 路由遵循版本化结构。旧版 `/api/*` 路由保持兼容，响应头附带 `Deprecation` 标记。

| 层级 | 前缀 | 认证方式 | 使用场景 |
|------|------|---------|---------|
| Admin | `/api/v1/admin/*` | JWT + RBAC | 管理后台 |
| App | `/api/v1/app/*` | JWT（client_type=app） | 移动端 / 小程序 |
| Open | `/api/v1/open/*` | API Key + Scope | 第三方接入 |
| 旧版兼容 | `/api/*` | JWT + RBAC | 向下兼容 |

### API 总览

| 模块 | 端点 |
|------|------|
| 认证 | `POST /auth/login`, `/logout`, `/refresh`, `/2fa/*` |
| 用户 | `GET/POST /users`, `GET/PUT/DELETE /users/:id`, `GET /users/me` |
| 角色 | `GET/POST /roles`, `GET/PUT/DELETE /roles/:id` |
| 部门 | `GET/POST /departments`, `GET/PUT/DELETE /departments/:id` |
| 字典 | `GET/POST /dict-types`, `/dict-items`, `GET /dicts/:code` |
| 文件 | `POST /files/upload`, `GET /files/:id/download` |
| 审计 | `GET /audit-logs`, `/audit-logs/stats` |
| 会话 | `GET /sessions`, `DELETE /sessions/:id` |
| 搜索 | `GET /search?q=` |
| 仪表盘 | `GET /dashboard/stats`, `/dashboard/settings` |
| 通知 | `GET /notifications`, `POST /notifications/broadcast` |
| SSE | `GET /sse/notifications?token=` |
| 导出 | `POST /export/users` |
| 导入 | `POST /users/import`, `GET /users/import/template` |
| 菜单 | `GET/POST /menus`, `GET/PUT/DELETE /menus/:id` |
| 配置 | `GET/POST/PUT/DELETE /configs` |
| API Key | `POST/GET /api-keys`, `DELETE /api-keys/:id` |
| App 认证 | `GET /auth/authorize`, `/callback`, `/providers`, `POST /auth/sms/*` |
| App 用户 | `GET /auth/bindings`, `POST /auth/social-bind`, `/social-unbind` |
| Open API | `GET /health`, `/users`, `/departments` |
| Webhook | `POST/GET /webhooks`, `PUT/DELETE /webhooks/:id`, `GET /webhooks/:id/deliveries` |
| 文档 | `GET /docs`（Swagger UI）, `/docs/openapi.json` |

## 配置说明

后端配置文件位于 `apps/backend/config/default.toml`：

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgres://dev:dev@localhost:5432/cradle"

[jwt]
access_exp_secs = 3600       # 1 小时
refresh_exp_secs = 604800    # 7 天

[rate_limit]
global_rpm = 100             # 全局每分钟请求数
login_rpm = 5                # 登录每分钟请求数

[storage]
upload_dir = "./uploads"
max_upload_size = 10485760   # 10 MB
max_avatar_size = 2097152    # 2 MB

[api]
version = "v1"
legacy_routes_enabled = true
docs_enabled = true

[api.github]
client_id = ""
client_secret = ""

[api.google]
client_id = ""
client_secret = ""

[webhook]
max_retries = 3
retry_intervals_secs = [60, 300, 1800]
timeout_secs = 10
```

## 测试

```bash
cd apps/backend

# 运行全部集成测试（需要运行中的 PostgreSQL）
cargo test

# 运行 Phase 1-7 测试
cargo test --test integration_test

# 运行 Phase 8 测试
cargo test --test phase8_api_tests
```

## Docker 部署

```bash
# 构建并启动所有服务
docker compose up -d

# 后端：http://localhost:8080
# 前端：http://localhost:3000
```

## 路线图

### 已完成
- [x] **Phase 1** — 用户管理（增删改查、状态、分页）
- [x] **Phase 2** — RBAC（角色、权限、中间件）
- [x] **Phase 3** — 仪表盘与设置（系统信息、个人资料）
- [x] **Phase 4** — 文件上传、数据导出、会话管理、动态菜单、国际化
- [x] **Phase 5** — 两步验证、通知系统、审计日志增强、系统配置
- [x] **Phase 6** — 部门管理、字典管理、登录日志、用户导入
- [x] **Phase 7** — 全局搜索、数据权限、SSE 实时通知、头像上传、面包屑、标签页、主题定制
- [x] **Phase 8** — API 基础设施（三层路由、API Key、OAuth2、Webhook、OpenAPI 文档）

### 计划中
- [ ] **Phase 9** — 数据可视化（用户增长图表、登录热力图、活动仪表盘）
- [ ] **Phase 10** — 工作流引擎（审批流、状态机）
- [ ] **Phase 11** — 插件系统（动态路由/模块注册）
- [ ] **Phase 12** — 多租户支持
- [ ] **Phase 13** — 移动端响应式布局适配
- [ ] 系统资源监控（CPU / 内存 / 磁盘）
- [ ] 定时任务管理（Cron 可视化）
- [ ] 数据备份与恢复工具

## 许可证

私有项目，保留所有权利。
