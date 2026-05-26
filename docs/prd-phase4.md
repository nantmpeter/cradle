# PRD — cradle Phase 4: 文件管理 · 数据导出 · 会话管理 · 动态菜单 · 国际化

## 1. 项目信息

| 字段 | 值 |
|---|---|
| Language | 中文 |
| Programming Language | Rust (Axum + SQLx + PostgreSQL + JWT) + React 19 (Vite + shadcn/ui + Zustand + TanStack Query + Recharts) |
| Project Name | `cradle` |
| Phase | Phase 4 — 文件上传/管理、数据导出、会话管理、动态菜单、i18n 国际化 |

### 原始需求复述

在 Phase 1-3（用户管理、RBAC 安全、Dashboard 统计，86 tests passed）基础上，Phase 4 新增 5 个模块：
1. 文件上传/管理 — 头像上传、通用附件管理、本地+S3 双后端
2. 数据导出 — 用户/角色/审计日志列表导出 CSV/XLSX
3. 会话管理 — 服务端 session、在线会话查看、强制下线
4. 动态菜单 — 后端菜单 CRUD、权限关联、前端动态侧边栏
5. i18n 国际化 — react-i18next 中英双语、后端错误消息国际化

---

## 2. 产品定义

### Product Goals

1. **完善的附件管理能力** — 用户可上传头像，管理员可管理通用附件，系统支持本地存储和 S3/MinIO 双后端切换，满足不同部署环境需求
2. **数据可移植性** — 管理员可将用户列表、角色列表、审计日志等数据导出为 CSV/XLSX 格式，支持按当前筛选条件导出，便于数据分析与合规审计
3. **安全可观测的会话管理** — 引入服务端 session 存储，管理员可查看在线用户、强制下线可疑会话，普通用户可管理自己的登录设备，提升系统安全性
4. **灵活的导航架构** — 菜单由后端动态管理，根据角色/权限动态返回，前端根据菜单数据渲染侧边栏，取代硬编码导航，使系统可扩展性大幅提升
5. **国际化基础能力** — 提供中英双语支持，覆盖侧边栏、表单、表格、提示信息等全部 UI 文案，为多语言用户和国际部署奠定基础

---

### User Stories

#### 模块一：文件上传/管理

| # | 角色 | 故事 |
|---|---|---|
| FM-1 | 登录用户 | 作为登录用户，我想上传自己的头像，以便在系统中展示个人形象 |
| FM-2 | 管理员 | 作为管理员，我想上传和管理通用附件（图片、文档等），以便在系统中共享资源 |
| FM-3 | 管理员 | 作为管理员，我想查看所有已上传文件的列表并删除不需要的文件，以便管理存储空间 |
| FM-4 | 系统运维 | 作为运维人员，我想通过配置切换本地存储和 S3/MinIO 后端，以便适配不同部署环境 |

#### 模块二：数据导出

| # | 角色 | 故事 |
|---|---|---|
| EX-1 | 管理员 | 作为管理员，我想将用户列表导出为 CSV 或 XLSX 文件，以便在 Excel 中进行离线分析 |
| EX-2 | 管理员 | 作为管理员，我想将审计日志导出为 CSV 或 XLSX 文件，以便进行合规审计和安全审查 |
| EX-3 | 管理员 | 作为管理员，我想在筛选条件下导出数据（如仅导出禁用状态的用户），以便只获取我关心的数据 |
| EX-4 | 管理员 | 作为管理员，我想将角色列表导出为 CSV 或 XLSX 文件，以便进行权限审计 |

#### 模块三：会话管理

| # | 角色 | 故事 |
|---|---|---|
| SM-1 | 管理员 | 作为管理员，我想查看当前所有在线会话列表（IP、设备、最后活跃时间），以便监控系统使用状况 |
| SM-2 | 管理员 | 作为管理员，我想强制某个会话下线（踢人），以便及时阻止可疑活动 |
| SM-3 | 登录用户 | 作为登录用户，我想查看自己所有活跃会话，以便了解我的登录设备情况 |
| SM-4 | 登录用户 | 作为登录用户，我想远程注销某个设备上的登录会话，以便在更换设备后保护账号安全 |

#### 模块四：动态菜单

| # | 角色 | 故事 |
|---|---|---|
| DM-1 | 超级管理员 | 作为超级管理员，我想在后端管理菜单项（增删改排序），以便灵活配置系统导航结构 |
| DM-2 | 超级管理员 | 作为超级管理员，我想为菜单项关联权限，以便控制不同角色看到的导航菜单 |
| DM-3 | 登录用户 | 作为登录用户，我想看到根据我角色权限动态生成的侧边栏菜单，以便只看到我有权访问的功能 |
| DM-4 | 超级管理员 | 作为超级管理员，我想为菜单配置图标和排序，以便侧边栏美观且符合使用习惯 |

#### 模块五：i18n 国际化

| # | 角色 | 故事 |
|---|---|---|
| I18-1 | 登录用户 | 作为登录用户，我想在系统中切换中文/英文界面语言，以便使用我偏好的语言操作 |
| I18-2 | 登录用户 | 作为登录用户，我想所有界面文案（侧边栏、表单、表格、提示信息）都随语言切换而变化，以便完整的多语言体验 |
| I18-3 | 开发者 | 作为开发者，我想后端错误消息也支持国际化，以便前端能根据用户语言展示正确的错误提示 |

---

## 3. 技术规范

### Requirements Pool

#### P0 — Must Have

| ID | 需求 | 说明 |
|---|---|---|
| **文件上传/管理** | | |
| P0-F1 | **文件上传 API** | `POST /api/files/upload`，支持 multipart/form-data，限制文件类型（白名单）和大小（默认 10MB） |
| P0-F2 | **文件列表 API** | `GET /api/files`，分页返回已上传文件列表（文件名、大小、类型、上传者、上传时间） |
| P0-F3 | **文件删除 API** | `DELETE /api/files/{id}`，删除文件记录和实际文件 |
| P0-F4 | **本地存储后端** | 默认使用本地文件系统存储，文件保存在配置目录下，数据库记录元信息 |
| P0-F5 | **头像上传** | 用户可在个人资料页上传/更换头像，调用文件上传 API，更新用户 avatar_url 字段 |
| P0-F6 | **文件上传前端组件** | 通用 FileUploader 组件（拖拽上传、进度条、文件类型校验） |
| P0-F7 | **文件管理页面** | 管理员查看文件列表、删除文件的管理界面 |
| **数据导出** | | |
| P0-E1 | **用户列表导出** | `GET /api/users/export?format=csv\|xlsx`，支持现有筛选参数（search/role/status），返回文件流 |
| P0-E2 | **审计日志导出** | `GET /api/audit-logs/export?format=csv\|xlsx`，支持现有筛选参数，返回文件流 |
| P0-E3 | **角色列表导出** | `GET /api/roles/export?format=csv\|xlsx`，返回文件流 |
| P0-E4 | **导出前端按钮** | 用户/角色/审计日志列表页新增"导出"按钮，支持选择 CSV 或 XLSX 格式 |
| **会话管理** | | |
| P0-S1 | **服务端 session 存储** | 新增 `sessions` 表，记录 session_id、user_id、ip、user_agent、created_at、last_active_at、expires_at |
| P0-S2 | **登录创建 session** | 用户登录时创建 session 记录，返回 session_id（可嵌入 JWT claims 或作为独立标识） |
| P0-S3 | **在线会话列表 API** | `GET /api/sessions`，管理员查看所有在线会话（IP、设备、最后活跃时间） |
| P0-S4 | **强制下线 API** | `DELETE /api/sessions/{id}`，管理员强制终止指定会话（将 refresh token 加入黑名单 + 删除 session 记录） |
| P0-S5 | **我的会话 API** | `GET /api/sessions/me`，普通用户查看自己的活跃会话 |
| P0-S6 | **会话管理前端页面** | 管理员会话管理页面（在线会话列表 + 强制下线按钮），个人资料页展示"我的设备" |
| **动态菜单** | | |
| P0-M1 | **菜单数据表** | 新增 `menus` 表（id、parent_id、title、path、icon、sort_order、permission_id、status、created_at、updated_at），最多 3 级 |
| P0-M2 | **菜单 CRUD API** | `POST/GET/PUT/DELETE /api/menus`，超级管理员管理菜单项 |
| P0-M3 | **动态菜单 API** | `GET /api/menus/tree`，根据当前用户权限返回可见菜单树 |
| P0-M4 | **前端动态侧边栏** | 替换 Sidebar.tsx 中硬编码的 `navItems`，从 `/api/menus/tree` 获取菜单数据动态渲染 |
| P0-M5 | **菜单种子数据** | 迁移脚本插入现有菜单项（Dashboard、Users、Roles、Audit Logs、Profile、Settings）及其权限关联 |
| **i18n 国际化** | | |
| P0-I1 | **前端 i18n 框架** | 集成 `react-i18next` + `i18next`，创建中英文翻译文件 |
| P0-I2 | **语言切换组件** | Header 新增语言切换按钮（中/英），选择存入 localStorage |
| P0-I3 | **侧边栏国际化** | 所有菜单文案使用 i18n key |
| P0-I4 | **表单/表格国际化** | 所有表单 label、placeholder、表格列头、空状态提示使用 i18n key |
| P0-I5 | **后端错误消息国际化** | 后端 API 支持通过 `Accept-Language` header 返回对应语言的错误消息（中文默认） |

#### P1 — Should Have

| ID | 需求 | 说明 |
|---|---|---|
| P1-F1 | **S3/MinIO 存储后端** | 支持 S3 兼容存储后端（MinIO），通过环境变量配置切换存储策略 |
| P1-F2 | **图片缩略图/预览** | 图片文件上传时自动生成缩略图，前端支持图片预览（lightbox 或 modal） |
| P1-F3 | **文件类型白名单配置** | 允许通过配置文件/环境变量自定义允许上传的文件类型列表 |
| P1-E1 | **导出进度提示** | 导出大数据量时显示加载状态/进度提示 |
| P1-S1 | **session 自动清理** | 定时任务清理过期 session 记录 |
| P1-S2 | **request 中间件更新 last_active** | 每次请求更新 session 的 last_active_at 字段（节流，如每 5 分钟更新一次） |
| P1-M1 | **菜单排序拖拽** | 菜单管理页面支持拖拽排序 |
| P1-M2 | **菜单管理页面** | 超级管理员可视化管理菜单的完整页面（树形展示、新增/编辑/删除） |
| P1-I1 | **提示信息国际化** | toast/notification 提示信息使用 i18n key |
| P1-I2 | **分页/筛选组件国际化** | 分页器、筛选器等通用组件文案国际化 |

#### P2 — Nice to Have

| ID | 需求 | 说明 |
|---|---|---|
| P2-F1 | **文件批量上传** | 支持一次选择多个文件上传 |
| P2-F2 | **文件分类/标签** | 文件支持分类和标签管理 |
| P2-F3 | **存储配额** | 用户/系统存储配额限制 |
| P2-E1 | **导出模板** | 支持自定义导出列选择 |
| P2-S1 | **设备识别优化** | 解析 User-Agent 显示友好设备名称（如"Chrome on macOS"） |
| P2-S2 | **登录地理信息** | 通过 IP 解析地理位置 |
| P2-I1 | **更多语言** | 支持日文、韩文等更多语言 |
| P2-I2 | **用户偏好持久化** | 语言偏好存入后端用户 profile |

---

### API 设计草案

#### 文件上传/管理

| Method | Path | 说明 | 权限 |
|--------|------|------|------|
| POST | `/api/files/upload` | 上传文件（multipart/form-data） | 认证用户 |
| GET | `/api/files` | 文件列表（分页） | `files:read` |
| GET | `/api/files/{id}` | 获取文件详情/下载 | 认证用户（自己上传）或 `files:read` |
| DELETE | `/api/files/{id}` | 删除文件 | `files:delete` 或文件所有者 |

#### 数据导出

| Method | Path | 说明 | 权限 |
|--------|------|------|------|
| GET | `/api/users/export` | 导出用户列表（`?format=csv|xlsx&search=&role=&status=`） | `users:read` |
| GET | `/api/audit-logs/export` | 导出审计日志（`?format=csv|xlsx&action=&user_id=`） | `audit:read` |
| GET | `/api/roles/export` | 导出角色列表（`?format=csv|xlsx`） | `roles:read` |

#### 会话管理

| Method | Path | 说明 | 权限 |
|--------|------|------|------|
| GET | `/api/sessions` | 所有在线会话列表 | `sessions:read`（管理员） |
| GET | `/api/sessions/me` | 当前用户的活跃会话 | 认证用户 |
| DELETE | `/api/sessions/{id}` | 强制下线 | `sessions:manage`（管理员） |
| DELETE | `/api/sessions/me/{id}` | 注销自己的某个会话 | 认证用户（仅自己的） |

#### 动态菜单

| Method | Path | 说明 | 权限 |
|--------|------|------|------|
| GET | `/api/menus/tree` | 获取当前用户可见菜单树 | 认证用户 |
| GET | `/api/menus` | 菜单列表（扁平，管理用） | `menus:read` |
| POST | `/api/menus` | 创建菜单项 | `menus:create` |
| PUT | `/api/menus/{id}` | 更新菜单项 | `menus:update` |
| DELETE | `/api/menus/{id}` | 删除菜单项（含子菜单） | `menus:delete` |

#### i18n

| Method | Path | 说明 | 权限 |
|--------|------|------|------|
| GET | `/api/i18n/{locale}` | 获取后端错误消息翻译 | 公开 |

---

### 数据模型变更

```sql
-- 文件管理
CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    filename VARCHAR(255) NOT NULL,
    original_name VARCHAR(255) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    size BIGINT NOT NULL,
    storage_path VARCHAR(500) NOT NULL,
    storage_backend VARCHAR(20) NOT NULL DEFAULT 'local',  -- 'local' | 's3'
    thumbnail_path VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 会话管理
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    refresh_token_hash VARCHAR(255) NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);

-- 动态菜单
CREATE TABLE menus (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID REFERENCES menus(id),
    title_key VARCHAR(100) NOT NULL,       -- i18n key, e.g. "menu.dashboard"
    title_label VARCHAR(100) NOT NULL,      -- 默认显示文本（中文）
    path VARCHAR(200) NOT NULL,
    icon VARCHAR(50),                        -- lucide icon name
    sort_order INT NOT NULL DEFAULT 0,
    permission_id UUID REFERENCES permissions(id),
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT menus_max_depth CHECK (
        parent_id IS NULL
        OR NOT EXISTS (
            SELECT 1 FROM menus p WHERE p.id = menus.parent_id AND p.parent_id IS NOT NULL
            AND EXISTS (SELECT 1 FROM menus gp WHERE gp.id = p.parent_id AND gp.parent_id IS NOT NULL)
        )
    )
);
CREATE INDEX idx_menus_parent_id ON menus(parent_id);
CREATE INDEX idx_menus_sort_order ON menus(sort_order);

-- 用户表新增头像字段
ALTER TABLE users ADD COLUMN avatar_url VARCHAR(500);
```

---

### UI 设计稿描述

#### 文件管理页面（管理员）

```
┌─────────────────────────────────────────────────────┐
│  文件管理                                            │
├─────────────────────────────────────────────────────┤
│  [搜索文件名...]  [文件类型 ▼]  [上传文件]            │
├─────────────────────────────────────────────────────┤
│  文件名          │ 大小   │ 类型    │ 上传者  │ 时间  │ 操作 │
│  avatar.png     │ 2.1MB  │ image   │ admin  │ 5/24  │ 🗑️ 👁️│
│  report.pdf     │ 156KB  │ pdf     │ admin  │ 5/23  │ 🗑️ ⬇️│
│  ...            │ ...    │ ...     │ ...    │ ...   │ ...  │
├─────────────────────────────────────────────────────┤
│                    ◀ 1 2 3 ▶                         │
└─────────────────────────────────────────────────────┘
```

#### 数据导出（列表页集成）

```
┌─────────────────────────────────────────────┐
│  用户列表                                    │
│  [搜索...] [角色▼] [状态▼]    [导出 ▼] [新增]│
│                                    │        │
│                                    ├ CSV    │
│                                    └ XLSX   │
└─────────────────────────────────────────────┘
```

#### 会话管理页面

```
┌─────────────────────────────────────────────────────────┐
│  会话管理                                                │
├─────────────────────────────────────────────────────────┤
│  用户         │ IP            │ 设备          │ 最后活跃   │ 操作    │
│  admin@ex.com │ 192.168.1.100 │ Chrome/macOS  │ 2 分钟前   │ [踢下线] │
│  user@ex.com  │ 10.0.0.55     │ Safari/iOS    │ 15 分钟前  │ [踢下线] │
│  ...          │ ...           │ ...           │ ...       │ ...     │
└─────────────────────────────────────────────────────────┘

个人资料页 - "我的设备" 区域：
┌─────────────────────────────────────────────────────────┐
│  我的设备                                                │
│  Chrome / macOS  │ 192.168.1.100 │ 当前设备 ✅           │
│  Safari / iOS    │ 10.0.0.55     │ 3 小时前  [注销]     │
└─────────────────────────────────────────────────────────┘
```

#### 菜单管理页面

```
┌─────────────────────────────────────────────────────┐
│  菜单管理                            [+ 新增菜单]   │
├─────────────────────────────────────────────────────┤
│  ☰ Dashboard         /dashboard        [编辑] [删除] │
│  ☰ Users             /dashboard/users   [编辑] [删除] │
│  ☰ Roles             /dashboard/roles   [编辑] [删除] │
│  ☰ Audit Logs        /dashboard/audit   [编辑] [删除] │
│  ☰ Profile           /dashboard/profile [编辑] [删除] │
│  ☰ Settings          /dashboard/settings[编辑] [删除] │
└─────────────────────────────────────────────────────┘
```

---

### 技术实现要点

#### 后端新增模块

| 模块 | Handler | Service | Repository |
|------|---------|---------|------------|
| 文件管理 | `file_handler.rs` | `file_service.rs` | `file_repo.rs` |
| 数据导出 | `export_handler.rs` | `export_service.rs` | 复用现有 repo |
| 会话管理 | `session_handler.rs` | `session_service.rs` | `session_repo.rs` |
| 动态菜单 | `menu_handler.rs` | `menu_service.rs` | `menu_repo.rs` |
| i18n | `i18n_handler.rs` | — | — |

#### 后端依赖新增

- `rust-s3` 或 `aws-sdk-s3` — S3/MinIO 支持（P1）
- `csv` — CSV 导出
- `xlsxwriter` 或 `rust_xlsxwriter` — XLSX 导出
- `tower-http` 已有的 `limit::RequestBodyLimit` — 文件大小限制

#### 前端新增

- `react-i18next` + `i18next` — i18n 框架
- 翻译文件结构：`src/locales/zh-CN.json`、`src/locales/en-US.json`
- 新增页面：文件管理、会话管理、菜单管理
- 修改组件：Sidebar（动态菜单）、Header（语言切换）、ProfilePage（头像上传、我的设备）
- 列表页统一添加导出按钮

#### 新增权限种子数据

| 权限名 | 模块 | 说明 |
|--------|------|------|
| `files:read` | files | 查看文件列表 |
| `files:upload` | files | 上传文件 |
| `files:delete` | files | 删除文件 |
| `export:users` | export | 导出用户列表 |
| `export:audit` | export | 导出审计日志 |
| `export:roles` | export | 导出角色列表 |
| `sessions:read` | sessions | 查看所有会话 |
| `sessions:manage` | sessions | 强制下线 |
| `menus:read` | menus | 查看菜单列表 |
| `menus:create` | menus | 创建菜单 |
| `menus:update` | menus | 更新菜单 |
| `menus:delete` | menus | 删除菜单 |

---

## 4. 待确认问题

| # | 问题 | 建议 |
|---|---|---|
| 1 | **文件存储后端优先级** — Phase 4 P0 是否只做本地存储，S3 后端放 P1？ | 建议 P0 仅本地存储，S3/MinIO 作为 P1，降低初始复杂度 |
| 2 | **session 与 JWT 的关系** — 引入 session 后，JWT 是否仍为主要认证机制？session 是辅助还是替代？ | 建议 JWT 仍为主认证，session 仅用于追踪在线状态。登录时同时创建 session 记录，JWT claims 中嵌入 session_id |
| 3 | **导出数据量限制** — 大数据量导出是否需要异步/后台任务？ | P0 同步导出（限制最大行数如 10000），P2 考虑异步导出+下载链接 |
| 4 | **菜单层级深度限制** — 是否最多 3 级？ | 建议最多 3 级（parent → child → grandchild），数据库约束 + 后端校验 |
| 5 | **i18n 翻译文件管理** — 翻译文件是 JSON 嵌套结构还是扁平 key？ | 建议扁平 key（如 `menu.dashboard`、`users.table.name`），便于查找和维护 |
| 6 | **菜单 i18n key 与翻译文件的同步** — 菜单的 `title_key` 如何与前端翻译文件对应？ | 建议：数据库菜单记录存 `title_key`（如 `menu.dashboard`），前端用 `t(menu.title_key)` 渲染；菜单管理界面同时展示默认中文标签便于管理 |
| 7 | **会话清理策略** — 过期 session 清理是后台定时任务还是惰性清理？ | 建议 P0 惰性清理（查询时过滤 + 登录时顺便清理），P1 添加定时任务 |
| 8 | **文件上传大小限制** — 默认限制多大？ | 建议默认 10MB，头像限制 2MB，可通过配置调整 |
| 9 | **导出权限是否需要独立权限** — 还是复用列表查看权限（如 `users:read`）？ | 建议导出需要额外 `export:xxx` 权限，与列表查看权限分离，便于精细控制 |
