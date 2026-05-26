---
name: cradle-dev
version: 1.0.0
description: "cradle 项目二次开发指南：架构、约定、技术栈、新增功能的标准流程与踩坑笔记"
author: agent
trigger:
  - "cradle"
  - "新增功能"
  - "添加接口"
  - "新模块"
  - "二次开发"
  - "加个页面"
  - "加个API"
  - "写个接口"
  - "新CRUD"
---

# cradle 项目二次开发指南

> 项目路径：`/Users/wangjiayi/Projects/cradle/`
> 本 Skill 是该项目的**唯一开发知识库**，所有新增功能都应参照此文档执行。

---

## 1. 项目概览

| 维度 | 说明 |
|------|------|
| 定位 | 业务系统底座 / 后台管理系统基础框架 |
| 后端 | Rust (Axum 0.8 + SQLx 0.8 + PostgreSQL 16) |
| 前端 | React 19 + Vite 6 + shadcn/ui + Tailwind CSS 4 + TypeScript 6 |
| 状态管理 | Zustand (auth) + TanStack Query (数据) |
| 路由 | react-router-dom v7 |
| 国际化 | i18next (zh-CN / en-US) |
| 认证 | JWT (access + refresh) + 2FA (TOTP) |
| API 文档 | utoipa@5 + swagger-ui@8 |
| 单仓结构 | Cargo workspace (后端) + Turborepo (前端) |

### 端口与凭证

| 项目 | 值 |
|------|------|
| 后端端口 | 8080 |
| 前端端口 | 5173 (Vite dev proxy → 8080) |
| 数据库 | `postgres://dev:dev@localhost:5432/cradle` |
| 默认管理员 | `admin@example.com` / `Admin@1234` |
| 管理员角色 | `superadmin` |

---

## 2. 目录结构速查

```
cradle/
├── apps/
│   ├── backend/                    # Rust 后端
│   │   ├── src/
│   │   │   ├── main.rs             # 入口
│   │   │   ├── lib.rs              # AppState + app() 路由组装
│   │   │   ├── config.rs           # Settings (config crate)
│   │   │   ├── db.rs               # 数据库连接池
│   │   │   ├── error.rs            # AppError 统一错误
│   │   │   ├── extractors/         # AuthUser, ApiKeyContext, RequestId
│   │   │   ├── handlers/           # HTTP handler 函数
│   │   │   ├── middleware/         # auth, api_key_auth, app_auth, request_id, deprecation
│   │   │   ├── models/             # 数据结构 (serde Serialize/Deserialize)
│   │   │   ├── repository/         # 数据库查询 (sqlx)
│   │   │   ├── routes/             # 路由定义 (mod.rs)
│   │   │   └── services/           # 业务逻辑层
│   │   ├── config/default.toml     # 运行时配置
│   │   ├── migrations/             # SQL 迁移文件
│   │   └── tests/                  # 集成测试
│   └── frontend/                   # React 前端
│       └── src/
│           ├── components/         # UI 组件 (按模块分目录)
│           │   ├── ui/             # shadcn/ui 基础组件
│           │   ├── layout/         # AppShell, Header, Sidebar
│           │   ├── auth/           # 登录/权限守卫
│           │   └── <module>/       # 业务模块组件
│           ├── hooks/              # TanStack Query hooks (use<Module>.ts)
│           ├── types/              # TypeScript 类型定义
│           ├── lib/                # api.ts (axios), theme.ts, utils.ts
│           ├── stores/             # Zustand stores
│           ├── routes/             # 路由配置 (index.tsx)
│           └── locales/            # i18n JSON (zh-CN.json, en-US.json)
├── docs/                           # 设计文档 (PRD, 架构, 图表)
├── site/                           # 官网静态页面
└── docker-compose.yml              # PostgreSQL + backend + frontend
```

---

## 3. 三层路由架构 (Phase 8)

所有路由函数定义在 `routes/mod.rs` 中，**不含 `/api` 前缀**，在 `lib.rs` 的 `app()` 函数中通过 `.nest()` 挂载到不同层级：

| 层级 | 前缀 | 认证方式 | 用途 |
|------|------|----------|------|
| **Admin** | `/api/v1/admin/*` | JWT + RBAC (auth 中间件) | 后台管理 |
| **App** | `/api/v1/app/*` | JWT (client_type=app 中间件) | 移动端/小程序 |
| **Open** | `/api/v1/open/*` | API Key (api_key_auth 中间件) | 第三方集成 |
| **Legacy** | `/api/*` | JWT + deprecation header | 向后兼容 |

### 关键原则

- **路由函数内部不写 `/api` 前缀**，只写模块路径如 `/users`, `/roles`
- **中间件在 `.nest()` 时通过 `.layer()` 挂载**，不在 handler 中检查
- `auth_protected_routes()` (如 `/auth/logout`) 需要 auth 中间件，测试 Router 必须显式注册并加中间件

---

## 4. 后端新增功能标准流程 (CRUD 模块)

以添加「标签管理 (tags)」模块为例：

### Step 1: 数据库迁移

文件命名: `migrations/YYYYMMDD000001_phaseN_description.sql`

```sql
-- migrations/20260530000001_create_tags_table.sql
CREATE TABLE tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 权限种子数据
INSERT INTO permissions (name, code, module) VALUES
('查看标签', 'tags:read', 'tags'),
('创建标签', 'tags:create', 'tags'),
('编辑标签', 'tags:update', 'tags'),
('删除标签', 'tags:delete', 'tags');
```

### Step 2: Model (`models/tag.rs`)

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTagRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 100))]
    pub slug: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTagRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub slug: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TagListResponse {
    pub items: Vec<Tag>,
    pub total: i64,
}
```

在 `models/mod.rs` 中添加: `pub mod tag;`

### Step 3: Repository (`repository/tag_repo.rs`)

```rust
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::tag::*;

pub async fn find_all(pool: &PgPool, page: i64, page_size: i64) -> Result<(Vec<Tag>, i64), sqlx::Error> {
    let offset = (page - 1) * page_size;
    let items = sqlx::query_as::<_, Tag>("SELECT * FROM tags ORDER BY created_at DESC LIMIT $1 OFFSET $2")
        .bind(page_size).bind(offset).fetch_all(pool).await?;
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tags")
        .fetch_one(pool).await?;
    Ok((items, count.0))
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Tag>, sqlx::Error> {
    sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE id = $1")
        .bind(id).fetch_optional(pool).await
}

pub async fn create(pool: &PgPool, req: &CreateTagRequest) -> Result<Tag, sqlx::Error> {
    sqlx::query_as::<_, Tag>(
        "INSERT INTO tags (name, slug) VALUES ($1, $2) RETURNING *"
    ).bind(&req.name).bind(&req.slug).fetch_one(pool).await
}

pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateTagRequest) -> Result<Option<Tag>, sqlx::Error> {
    // 动态构建 UPDATE，只更新非 None 字段
    let tag = match find_by_id(pool, id).await? {
        Some(t) => t,
        None => return Ok(None),
    };
    let name = req.name.as_deref().unwrap_or(&tag.name);
    let slug = req.slug.as_deref().unwrap_or(&tag.slug);
    sqlx::query_as::<_, Tag>(
        "UPDATE tags SET name = $1, slug = $2, updated_at = NOW() WHERE id = $3 RETURNING *"
    ).bind(name).bind(slug).bind(id).fetch_optional(pool).await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(id).execute(pool).await?;
    Ok(result.rows_affected() > 0)
}
```

在 `repository/mod.rs` 中添加: `pub mod tag_repo;`

### Step 4: Service (`services/tag_service.rs`)

Service 层处理业务逻辑，调用 repo，返回 `Result<T, AppError>`：

```rust
use crate::error::AppError;
use crate::models::tag::*;
use crate::repository::tag_repo;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list_tags(pool: &PgPool, page: i64, page_size: i64) -> Result<TagListResponse, AppError> {
    let (items, total) = tag_repo::find_all(pool, page, page_size).await?;
    Ok(TagListResponse { items, total })
}

pub async fn get_tag(pool: &PgPool, id: Uuid) -> Result<Tag, AppError> {
    tag_repo::find_by_id(pool, id).await?
        .ok_or_else(|| AppError::NotFound(format!("Tag {} not found", id)))
}

pub async fn create_tag(pool: &PgPool, req: CreateTagRequest) -> Result<Tag, AppError> {
    tag_repo::create(pool, &req).await.map_err(AppError::Database)
}

pub async fn update_tag(pool: &PgPool, id: Uuid, req: UpdateTagRequest) -> Result<Tag, AppError> {
    tag_repo::update(pool, id, &req).await?
        .ok_or_else(|| AppError::NotFound(format!("Tag {} not found", id)))
}

pub async fn delete_tag(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    if !tag_repo::delete(pool, id).await? {
        return Err(AppError::NotFound(format!("Tag {} not found", id)));
    }
    Ok(())
}
```

在 `services/mod.rs` 中添加: `pub mod tag_service;`

### Step 5: Handler (`handlers/tag_handler.rs`)

```rust
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

use crate::extractors::auth::AuthUser;
use crate::models::tag::*;
use crate::services::tag_service;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn list_tags(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,          // JWT 自动注入
    Query(query): Query<ListQuery>,
) -> Result<Json<TagListResponse>, crate::error::AppError> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let result = tag_service::list_tags(&state.db, page, page_size).await?;
    Ok(Json(result))
}

pub async fn get_tag(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<Tag>, crate::error::AppError> {
    let tag = tag_service::get_tag(&state.db, id).await?;
    Ok(Json(tag))
}

pub async fn create_tag(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Json(req): Json<CreateTagRequest>,
) -> Result<(StatusCode, Json<Tag>), crate::error::AppError> {
    req.validate().map_err(|e| crate::error::AppError::BadRequest(e.to_string()))?;
    let tag = tag_service::create_tag(&state.db, req).await?;
    Ok((StatusCode::CREATED, Json(tag)))
}

pub async fn update_tag(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<UpdateTagRequest>,
) -> Result<Json<Tag>, crate::error::AppError> {
    req.validate().map_err(|e| crate::error::AppError::BadRequest(e.to_string()))?;
    let tag = tag_service::update_tag(&state.db, id, req).await?;
    Ok(Json(tag))
}

pub async fn delete_tag(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, crate::error::AppError> {
    tag_service::delete_tag(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

在 `handlers/mod.rs` 中添加: `pub mod tag_handler;`

### Step 6: 路由注册 (`routes/mod.rs`)

```rust
// 在 routes/mod.rs 中添加
pub fn tag_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/tags",
        Router::new()
            .route("/", get(tag_handler::list_tags).post(tag_handler::create_tag))
            .route("/{id}", get(tag_handler::get_tag).put(tag_handler::update_tag).delete(tag_handler::delete_tag)),
    )
}
```

在 `lib.rs` 的 `app()` 函数中，在 admin 和 legacy 路由组中各加一行：
```rust
.merge(routes::tag_routes())
```

### Step 7: 集成测试

在 `tests/` 下新增测试文件或追加到已有文件：

```rust
// 测试模板
#[tokio::test]
async fn test_tag_crud() {
    let (mut app, _pool) = create_test_app().await;
    let token = login_as_admin(&mut app).await;

    // Create
    let req = Request::builder()
        .method("POST")
        .uri("/api/tags")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"name":"Test","slug":"test"}"#))
        .unwrap();
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::CREATED);
    let tag_id = body["id"].as_str().unwrap();

    // List
    let req = Request::builder()
        .uri("/api/tags")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let (status, body) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["items"].as_array().unwrap().len() >= 1);

    // Delete
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/tags/{}", tag_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let (status, _) = send_request(&mut app, req).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}
```

---

## 5. 前端新增功能标准流程

### Step 1: TypeScript 类型 (`types/tag.ts`)

```typescript
export interface Tag {
  id: string
  name: string
  slug: string
  created_at: string
  updated_at: string
}

export interface TagListResponse {
  items: Tag[]
  total: number
}

export interface CreateTagRequest {
  name: string
  slug: string
}
```

### Step 2: API Hook (`hooks/useTags.ts`)

```typescript
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { api } from "@/lib/api"
import type { TagListResponse, CreateTagRequest, Tag } from "@/types/tag"

export function useTags(page = 1, pageSize = 20) {
  return useQuery<TagListResponse>({
    queryKey: ["tags", page, pageSize],
    queryFn: async () => {
      const { data } = await api.get("/v1/admin/tags", { params: { page, page_size: pageSize } })
      return data
    },
  })
}

export function useTag(id: string) {
  return useQuery<Tag>({
    queryKey: ["tags", id],
    queryFn: async () => {
      const { data } = await api.get(`/v1/admin/tags/${id}`)
      return data
    },
    enabled: !!id,
  })
}

export function useCreateTag() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (req: CreateTagRequest) => {
      const { data } = await api.post("/v1/admin/tags", req)
      return data
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ["tags"] }),
  })
}

export function useDeleteTag() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/v1/admin/tags/${id}`)
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ["tags"] }),
  })
}
```

### Step 3: 页面组件 (`components/tags/TagsPage.tsx`)

```tsx
import { useState } from "react"
import { useTags, useCreateTag, useDeleteTag } from "@/hooks/useTags"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from "@/components/ui/table"
import { Pagination } from "@/components/ui/pagination"
import { Plus, Trash2 } from "lucide-react"

export function TagsPage() {
  const [page, setPage] = useState(1)
  const { data, isLoading } = useTags(page)
  const createMutation = useCreateTag()
  const deleteMutation = useDeleteTag()
  // ... 标准 CRUD 页面模式，参考 UserListPage / RoleListPage
}
```

### Step 4: 路由注册 (`routes/index.tsx`)

```tsx
import { TagsPage } from "@/components/tags/TagsPage"

// 在 AppRoutes 的 <Route path="dashboard"> 内添加:
<Route
  path="tags"
  element={
    <PermissionRouteGuard permission="tags:read">
      <TagsPage />
    </PermissionRouteGuard>
  }
/>
```

### Step 5: 国际化 (`locales/zh-CN.json` / `en-US.json`)

```json
// zh-CN.json
{
  "tags": {
    "title": "标签管理",
    "name": "名称",
    "slug": "标识",
    "create": "创建标签",
    "delete": "删除标签"
  }
}

// en-US.json
{
  "tags": {
    "title": "Tag Management",
    "name": "Name",
    "slug": "Slug",
    "create": "Create Tag",
    "delete": "Delete Tag"
  }
}
```

---

## 6. 统一错误格式

后端所有错误统一返回：

```json
{
  "error": {
    "code": "UPPER_SNAKE_CASE",
    "message": "人类可读描述",
    "request_id": null,
    "details": null
  },
  "status": 404
}
```

错误码列表 (`error.rs`):
- `NOT_FOUND` (404)
- `UNAUTHORIZED` (401)
- `FORBIDDEN` (403)
- `BAD_REQUEST` (400)
- `CONFLICT` (409)
- `ACCOUNT_DISABLED` (403)
- `ACCOUNT_LOCKED` (403)
- `TOO_MANY_REQUESTS` (429)
- `TWO_FACTOR_REQUIRED` (401)
- `PAYLOAD_TOO_LARGE` (413)
- `INTERNAL_ERROR` (500)

**集成测试断言写法**: `body["error"]["message"]` (不是 `body["error"]`)

---

## 7. AppState 结构

```rust
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
    pub access_exp_secs: i64,
    pub refresh_exp_secs: i64,
    pub started_at: Instant,
    pub max_connections: u32,
    pub settings: Settings,
    pub totp_encryption_key: String,
    pub config_cache: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
    pub notification_tx: broadcast::Sender<SseNotification>,
    pub http_client: reqwest::Client,
    pub webhook_tx: mpsc::Sender<WebhookEvent>,
}
```

**测试中构建 AppState 时必须包含所有字段**，缺一不可，否则编译失败。

---

## 8. 测试基础设施

### 测试文件

| 文件 | 内容 |
|------|------|
| `tests/integration_test.rs` | Phase 1-7 功能测试 (35 tests) |
| `tests/phase8_api_tests.rs` | Phase 8 API 基础设施测试 (26 tests) |

### create_test_app() 模板

```rust
async fn create_test_app() -> (axum::Router, PgPool) {
    dotenvy::dotenv().ok();
    let settings = Settings::new().expect("Failed to load settings");
    let db = sqlx::PgPool::connect(&settings.database.url).await.expect("...");
    sqlx::migrate!("./migrations").run(&db).await.expect("...");

    // 重置 admin 账号状态（防跨测试干扰）
    sqlx::query("UPDATE users SET locked_until = NULL, login_failures = 0 WHERE email = 'admin@example.com'")
        .execute(&db).await.ok();
    sqlx::query("UPDATE users SET two_factor_enabled = false, totp_secret = NULL WHERE email = 'admin@example.com'")
        .execute(&db).await.ok();

    // 构建 AppState（必须包含全部字段）
    let (notification_tx, _) = tokio::sync::broadcast::channel(100);
    let (webhook_tx, _) = tokio::sync::mpsc::channel(100);
    let state = Arc::new(AppState {
        db: db.clone(), jwt_secret, access_exp_secs, refresh_exp_secs,
        started_at: Instant::now(), max_connections, settings: settings.clone(),
        totp_encryption_key: settings.totp.encryption_key.clone(),
        config_cache: Arc::new(RwLock::new(HashMap::new())),
        notification_tx, http_client: reqwest::Client::new(), webhook_tx,
    });

    // 路由组装：所有路由 nest 在 /api 下
    let auth_mw = axum::middleware::from_fn_with_state(state.clone(), require_auth);
    let app = Router::new()
        .merge(auth_routes())
        .nest("/api", Router::new()
            .merge(auth_protected_routes().layer(auth_mw.clone()))
            .merge(user_routes().layer(auth_mw.clone()))
            // ... 所有需要认证的路由
            .merge(search_routes().layer(auth_mw))
        )
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors);
    (app, db)
}
```

### 关键注意事项

- **测试共享数据库**，非隔离的，密码哈希变化会导致 `test_login_success` 失败
- **登录限流 5RPM** 会影响锁定测试，需要 sleep 间隔
- **并发执行** 会导致 admin 账号被错误锁定
- **admin 账号可能在手动测试时开启 2FA**，需在 `create_test_app` 中重置
- **Cargo workspace** 下编辑文件后可能需要 `touch` 强制重编译

---

## 9. 技术栈关键笔记

### Rust 后端

| 主题 | 注意事项 |
|------|----------|
| **中间件** | 必须用 `axum_mw::from_fn_with_state()` 显式应用到 Router group。handler 中的 `AuthUser` extractor 只是读取中间件注入的数据 |
| **QueryBuilder** | data query 和 count query 必须使用相同的表别名 |
| **审计日志** | action 过滤用 `ILIKE` 模糊匹配，非精确匹配 |
| **Dashboard** | 用 `tokio::join!` 并行多个独立 SQL，字段级权限过滤在 handler 层用 `serde_json::json!` 动态构建 |
| **系统版本** | 用 `env!("CARGO_PKG_VERSION")` 编译时注入 |
| **TOTP** | `totp-rs` v5: `Secret::generate_secret()` 需 `gen_secret` feature; `TOTP::new()` 在 `otpauth` feature 下需 7 参数 |
| **2FA 登录** | login 返回 `requires_2fa`+`temp_token` → 前端跳转 `/login/2fa` → 用户输入 TOTP code → POST `/api/auth/2fa/verify` 换取真实 token |
| **数据导入** | `calamine` 0.26: 用 `open_workbook_auto_from_rs(cursor)` 替代旧 API |
| **SSE** | `AppState` 中 `broadcast::Sender<SseNotification>` → sse_handler 订阅 → 前端 EventSource + query param token 认证 |
| **Webhook** | mpsc channel → event worker → HMAC-SHA256 签名 → tokio::spawn 异步 POST，事件在 handler 层触发不侵入 service 签名 |
| **API Key** | 格式 `rk_` + base64url(32 random bytes)，SHA-256 哈希存储，前 8 字符用于展示识别 |
| **数据权限** | roles.data_scope (all/department/department_and_sub/self) → `AuthUser::get_visible_department_ids()` → ListParams.department_ids 过滤 |

### React 前端

| 主题 | 注意事项 |
|------|----------|
| **API 基础路径** | `api.ts` 中 `API_BASE_URL = import.meta.env.VITE_API_BASE_URL || "/api"`；Vite 代理 `/api` → `localhost:8080` |
| **Token 存储** | localStorage: `access_token`, `refresh_token`；自动 401 刷新 + 并发请求队列 |
| **主题** | shadcn CSS 使用 `oklch` 格式；颜色对必须包含 `--primary` + `--primary-foreground` |
| **图表** | recharts，配色用 CSS 变量 `hsl(var(--chart-N))` 兼容 dark mode |
| **i18n** | `const { t, i18n } = useTranslation()` — 不要忘记解构 `i18n` |
| **shadcn 组件** | 安装到 `src/components/ui/`，不要用 CLI 的 `@` 字面路径 |
| **权限守卫** | `<PermissionRouteGuard permission="module:action">` 包裹页面组件 |
| **Query Key** | 格式 `["模块名", ...params]`，如 `["users", page, pageSize]` |

---

## 10. 迁移文件命名规范

```
migrations/YYYYMMDD000001_description.sql          # 建表/加字段
migrations/YYYYMMDD000002_phaseN_seed_perms.sql     # 权限种子数据
```

权限 code 格式: `模块:操作`，如 `tags:read`, `tags:create`, `tags:update`, `tags:delete`

---

## 11. 已完成功能清单

| Phase | 功能 | 测试 |
|-------|------|------|
| 1 | 用户管理 (CRUD + JWT + 注册/登录) | 24 tests |
| 2 | RBAC + Security (角色/权限/审计日志) | 45 tests |
| 3 | Dashboard + Settings (统计/系统设置) | 17 tests |
| 4 | 文件上传/数据导出/会话管理/动态菜单/i18n | 24 tests |
| 5 | 2FA/通知系统/审计日志增强/系统配置 | 24 tests |
| 6 | 部门管理/字典管理/登录日志/用户导入 | 35 tests |
| 7 | 全局搜索/数据权限/强制登出/头像上传/面包屑/Tab导航/SSE通知/主题定制 | — |
| 8 | 三层路由(Admin/App/Open) + API Key + OAuth2 + Webhook + OpenAPI 文档 | 26 tests |

**总计: 61 tests passed**

---

## 12. Roadmap (待开发)

- OpenAPI 文档完善 (更多 endpoint 覆盖)
- 数据可视化增强 (更多图表/报表)
- 运营工具 (消息推送/任务调度)
- 工作流引擎
- 多租户支持
- 移动端适配

---

## 13. 常用命令

```bash
# 后端开发
cd apps/backend
cargo test                        # 运行全部测试
cargo test test_name              # 运行单个测试
cargo run                         # 启动后端 (端口 8080)
cargo build                       # 编译检查

# 前端开发
cd apps/frontend
npm run dev                       # 启动前端 (端口 5173)
npm run build                     # 生产构建
npm run lint                      # ESLint 检查

# Docker
docker-compose up -d db           # 只启动数据库
docker-compose up                 # 全部启动

# 数据库迁移 (自动在 cargo run 时执行)
# 新增迁移文件放入 apps/backend/migrations/ 即可
```

---

## 14. 踩坑记录

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| scroll-area 组件找不到 | shadcn CLI 安装到字面 `@` 目录 | 手动复制到 `src/components/ui/` |
| i18n is not defined | `useTranslation()` 只解构了 `t` | 改为 `const { t, i18n } = useTranslation()` |
| 白色按钮白字 | 缺少 `--primary-foreground` CSS 变量 | 创建 `theme.ts`，提供 oklch 颜色对 |
| 2FA 刷新丢失状态 | useState 初始化早于数据加载 | 用 `useMe()` + `useEffect` 同步 |
| 测试 AppState 编译失败 | Phase 8 新增了 3 个字段 | 添加 notification_tx, http_client, webhook_tx |
| 测试路由 404 | Phase 8 移除了路由函数的 /api 前缀 | 在测试中 nest 在 /api 下 |
| 测试断言失败 | 错误格式从 `{ error: "msg" }` 改为嵌套 | 改为 `body["error"]["message"]` |
| EventSource 无法自定义 header | 浏览器限制 | SSE 端点用 query param 传 token |
| BroadcastStream 编译失败 | 缺少 feature | `tokio-stream` 添加 `features = ["sync"]` |

---

## 15. 配置文件参考

### 后端配置 (`config/default.toml`)

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgres://dev:dev@localhost:5432/cradle"
max_connections = 10

[jwt]
secret = "change-me-in-production-use-strong-random-key"
access_exp_secs = 3600
refresh_exp_secs = 604800

[rate_limit]
global_rpm = 100
login_rpm = 5

[storage]
upload_dir = "./uploads"
max_upload_size = 10485760
max_avatar_size = 2097152

[totp]
encryption_key = "jBrgUIZ+TRxlYByzq83lkK0pR9lLQQP/ZmRWxDK6r8A="

[api]
version = "v1"
legacy_routes_enabled = true
docs_enabled = true

[webhook]
max_retries = 3
retry_intervals_secs = [60, 300, 1800]
timeout_secs = 10
```

### 环境变量覆盖

支持 `APP__` 前缀的环境变量覆盖，如：
- `APP__DATABASE__URL` = 数据库连接串
- `APP__JWT__SECRET` = JWT 密钥
- `APP__SERVER__PORT` = 服务端口

---

_本 Skill 随项目迭代持续更新。新增模块后请同步更新本文档。_
