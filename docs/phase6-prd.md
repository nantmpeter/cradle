# Phase 6 PRD — Cradle 功能补齐

## 项目信息

- **Language**: 中文
- **Programming Language**: Rust (Axum + SQLx + SQLite) + React 19 (Vite + TypeScript + shadcn/ui + Tailwind)
- **Project Name**: cradle
- **原始需求**: 对标 ContiNew Admin 和 gin-vue-admin，补齐部门管理、字典管理、登录日志、用户导入/导出 Excel、系统设置在线编辑 UI 五大功能模块。

---

## 1. 产品目标

| # | 目标 | 说明 |
|---|------|------|
| G1 | **完善组织架构能力** | 通过部门树形管理和用户归属，支持企业级组织模型的落地，为后续数据权限打下基础。 |
| G2 | **提升系统可配置性** | 通过字典管理和系统设置在线编辑，让运维人员无需改代码即可调整系统行为（下拉选项、安全策略等）。 |
| G3 | **增强安全审计与数据操作效率** | 通过独立的登录日志和用户 Excel 批量导入/导出，提升安全追溯能力和批量操作效率。 |

---

## 2. 用户故事

| # | 角色 | 故事 |
|---|------|------|
| US1 | 系统管理员 | 我希望能以树形结构管理部门，并将用户分配到对应部门，以便清晰表达组织架构。 |
| US2 | 系统管理员 | 我希望能通过字典类型和字典项管理下拉选项（如性别、状态、类型等），以便前端组件直接从字典加载选项，无需硬编码。 |
| US3 | 安全审计员 | 我希望有独立的登录日志页面，记录每次登录/登出的 IP、User-Agent 和结果，以便快速定位异常登录行为。 |
| US4 | 系统管理员 | 我希望能将用户列表导出为 Excel 文件、也能从 Excel 批量导入用户，以便高效完成大批量用户数据操作。 |
| US5 | 运维人员 | 我希望系统设置页面不只是只读展示，而是可以在一个结构化的编辑面板中修改网站配置、安全配置、登录配置，并实时生效。 |

---

## 3. 需求池

### P0 — Must Have

#### F1: 部门/组织架构管理

| 需求ID | 需求 | 说明 |
|--------|------|------|
| F1-01 | 部门表 `departments` | `id`, `parent_id`(自引用), `name`, `code`(唯一编码), `sort_order`, `status(active/disabled)`, `leader`(负责人名称), `created_at`, `updated_at` |
| F1-02 | 用户表增加 `department_id` | `ALTER TABLE users ADD COLUMN department_id UUID REFERENCES departments(id)` |
| F1-03 | 部门 CRUD API | `GET /api/departments`（列表+树形）, `POST`, `PUT /{id}`, `DELETE /{id}` |
| F1-04 | 部门树形展示页 | 前端：左侧树形导航 + 右侧部门详情/编辑面板，复用 Menu 管理页的树形模式 |
| F1-05 | 用户创建/编辑支持选择部门 | 在用户创建和编辑弹窗中增加「所属部门」下拉（树形选择器或级联选择） |
| F1-06 | 权限种子 | `departments:read`, `departments:create`, `departments:update`, `departments:delete` |

#### F2: 字典管理

| 需求ID | 需求 | 说明 |
|--------|------|------|
| F2-01 | 字典类型表 `dict_types` | `id`, `name`(字典名称), `code`(唯一编码), `status`, `remark`, `created_at`, `updated_at` |
| F2-02 | 字典项表 `dict_items` | `id`, `dict_type_id`, `label`(显示文本), `value`(实际值), `sort_order`, `status`, `remark`, `created_at`, `updated_at` |
| F2-03 | 字典类型 CRUD API | `GET /api/dict-types`（分页列表）, `POST`, `PUT /{id}`, `DELETE /{id}` |
| F2-04 | 字典项 CRUD API | `GET /api/dict-types/{id}/items`（列表）, `POST`, `PUT /{id}`, `DELETE /{id}` |
| F2-05 | 字典公共查询 API | `GET /api/dicts/{code}` — 根据 code 返回该字典所有启用项（前端下拉组件使用，缓存友好） |
| F2-06 | 字典管理前端页 | 字典类型列表页 → 点击某类型进入字典项管理（Tab 页或嵌套表格） |
| F2-07 | 权限种子 | `dicts:read`, `dicts:create`, `dicts:update`, `dicts:delete` |

#### F3: 登录日志

| 需求ID | 需求 | 说明 |
|--------|------|------|
| F3-01 | 登录日志表 `login_logs` | `id`, `user_id`(可空，登录失败时无用户), `email`(登录邮箱), `event`(login/logout/login_failed), `ip_address`, `location`(IP 归属地，P1), `user_agent`, `os`, `browser`(UA 解析), `success`(bool), `fail_reason`, `login_at` |
| F3-02 | 登录/登出时自动写入 | 在 `auth_handler::login` 和 `auth_handler::logout` 中插入日志记录 |
| F3-03 | 登录日志列表 API | `GET /api/login-logs` — 支持分页、按用户/email/事件类型/时间范围/IP 筛选 |
| F3-04 | 登录日志详情 API | `GET /api/login-logs/{id}` |
| F3-05 | 登录日志前端页 | 列表页 + 筛选栏 + 详情弹窗，布局与 AuditLogListPage 类似 |
| F3-06 | 权限种子 | `login-logs:read` |

#### F4: 用户导入/导出 Excel

| 需求ID | 需求 | 说明 |
|--------|------|------|
| F4-01 | 用户导出已有 | 当前 `GET /api/export/users?format=xlsx` 已实现 xlsx 导出，保留 |
| F4-02 | 用户导入 API | `POST /api/users/import` — 接收 multipart xlsx 文件，解析并批量创建用户 |
| F4-03 | 导入校验逻辑 | 邮箱重复校验、密码强度校验、角色有效性校验；行级错误收集，返回导入结果报告 |
| F4-04 | 导入模板下载 | `GET /api/users/import/template` — 下载空白导入模板 xlsx |
| F4-05 | 前端导入功能 | UserListPage 增加「导入」按钮，弹窗上传 xlsx，展示导入结果（成功 N 条、失败 N 条 + 失败原因列表） |
| F4-06 | 权限种子 | `users:import`（已有 `export:users`） |

#### F5: 系统设置在线编辑 UI

| 需求ID | 需求 | 说明 |
|--------|------|------|
| F5-01 | 系统设置编辑页面 | 改造当前 SettingsPage（只读系统信息页），新增独立的「系统设置」编辑页面，与已有 SystemConfigPage（通用 key-value 编辑）并存但更友好 |
| F5-02 | 按分组展示配置卡片 | 分为「网站配置」「安全配置」「登录配置」三个 Tab 或 Card 分组 |
| F5-03 | 网站配置组 | 站点名称(`site_name`)、站点描述(`site_description`)、Logo URL（新增配置项） |
| F5-04 | 安全配置组 | 会话超时(`session_timeout`)、最大登录尝试(`max_login_attempts`)、密码最小长度(`password_min_length`)、是否开启注册（新增 `allow_register`） |
| F5-05 | 登录配置组 | 是否开启验证码（新增 `captcha_enabled`）、是否开启 TOTP 2FA 强制（新增 `force_2fa`）、登录失败锁定时长（新增 `lockout_duration`） |
| F5-06 | 表单化编辑 | 每个 Card 内使用表单控件（Input/Switch/Select），点击「保存」批量提交当前分组的修改，调用已有 `PUT /api/configs/{group}/{key}` |
| F5-07 | 新增配置项种子 | 在 migration 中添加上述新增配置项的 INSERT |

### P1 — Should Have

| 需求ID | 需求 | 说明 |
|--------|------|------|
| P1-01 | 部门数据权限基础 | 支持按部门过滤用户列表（`GET /api/users?department_id=xxx`），为后续角色级数据权限做前置准备 |
| P1-02 | IP 归属地查询 | 登录日志中解析 IP 的地理位置（可使用离线库如 ip2region 或跳过） |
| P1-03 | 登录日志 Dashboard 卡片 | Dashboard 页增加「最近登录」卡片，展示最近 10 条登录记录 |
| P1-04 | 字典前端 Hook | 封装 `useDict(code)` Hook，前端组件一行代码即可获取字典选项列表 |
| P1-05 | 部门树形选择组件 | 封装 `<DepartmentSelect />` 组件供用户表单复用 |
| P1-06 | 导入预览 | 用户导入上传后先展示预览表格，确认无误后再执行导入 |

### P2 — Nice to Have

| 需求ID | 需求 | 说明 |
|--------|------|------|
| P2-01 | 部门拖拽排序 | 前端支持拖拽调整部门层级和排序 |
| P2-02 | 登录日志导出 | 支持导出登录日志为 xlsx/csv |
| P2-03 | 字典缓存刷新 | 后台字典数据变更后，前端自动刷新（通过 WebSocket 或版本号机制） |
| P2-04 | 批量操作 | 用户列表支持勾选多行进行批量删除、批量修改部门/角色 |
| P2-05 | 配置变更审计 | 系统设置每次修改自动写入审计日志，记录新旧值 |

---

## 4. UI 设计说明

### 4.1 部门管理页 `/dashboard/departments`

**布局**：左右分栏（复用 MenusPage 的树形管理模式）

- **左侧面板（~40% 宽度）**：
  - 树形组件展示部门层级
  - 顶部有「新增顶级部门」按钮
  - 点击节点可展开/折叠，选中后右侧显示该部门详情
  - 每个节点旁有操作按钮（编辑、删除、新增子部门）

- **右侧面板（~60% 宽度）**：
  - 选中部门时展示：部门名称、编码、负责人、排序、状态、创建时间
  - 编辑模式：表单 Input（名称、编码、负责人、排序、状态 Switch）
  - 该部门下的用户列表（简要表格，链接到用户管理页按部门筛选）

### 4.2 字典管理页 `/dashboard/dicts`

**布局**：单页双层结构

- **第一层 — 字典类型列表**：
  - 顶部筛选栏（名称搜索、状态筛选）
  - 表格列：字典名称、字典编码、状态 Badge、备注、创建时间、操作（编辑/删除）
  - 「新增字典类型」按钮打开弹窗

- **第二层 — 字典项管理**（点击某字典类型行展开或进入子页面）：
  - 表格列：显示文本(label)、实际值(value)、排序、状态、操作
  - 「新增字典项」按钮
  - 支持拖拽排序调整 sort_order

### 4.3 登录日志页 `/dashboard/login-logs`

**布局**：与 AuditLogListPage 一致

- **筛选栏**：Email 输入框、事件类型下拉（登录/登出/登录失败）、时间范围选择器、IP 搜索
- **表格列**：登录邮箱、事件类型(Badge)、IP 地址、浏览器、操作系统、登录结果(成功✓/失败✗)、失败原因、登录时间
- **行操作**：点击展开详情弹窗（完整 User-Agent、详细信息）
- **分页**：复用 Pagination 组件

### 4.4 用户导入/导出（增强 UserListPage）

**导出**（已有，保持不变）：
- 顶部工具栏「导出 CSV」「导出 Xlsx」按钮

**导入**（新增）：
- 工具栏新增「导入」按钮（Upload 图标）
- 点击打开导入弹窗：
  1. 「下载模板」链接
  2. 文件拖拽/选择上传区域
  3. 上传后展示导入结果：
     - 成功条数、失败条数
     - 失败列表（行号 + 原因）以表格展示
  4. 确认关闭

### 4.5 系统设置编辑页（改造 SettingsPage 或新建路由）

**方案**：在现有 `/dashboard/settings` 系统信息展示页的下方，增加「系统配置」编辑区域。或者新建独立路由 `/dashboard/system-settings`。

推荐在 SettingsPage 中增加编辑 Tab：

- **Tab 1 — 系统信息**（现有内容保留）
- **Tab 2 — 网站配置**：站点名称 Input、站点描述 Textarea、Logo URL Input
- **Tab 3 — 安全配置**：会话超时 Input[number]、最大登录尝试 Input[number]、密码最小长度 Input[number]、允许注册 Switch
- **Tab 4 — 登录配置**：验证码 Switch、强制 2FA Switch、锁定时长 Input[number]

每个 Tab 底部有「保存」按钮，调用 `PUT /api/configs/{group}/{key}` 批量更新该组配置。

---

## 5. 数据库变更概要

### 新增表

```sql
-- 部门表
CREATE TABLE departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID REFERENCES departments(id) ON DELETE SET NULL,
    name VARCHAR(100) NOT NULL,
    code VARCHAR(100) UNIQUE NOT NULL,
    sort_order INT NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    leader VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 字典类型表
CREATE TABLE dict_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    code VARCHAR(100) UNIQUE NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    remark TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 字典项表
CREATE TABLE dict_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dict_type_id UUID NOT NULL REFERENCES dict_types(id) ON DELETE CASCADE,
    label VARCHAR(200) NOT NULL,
    value VARCHAR(200) NOT NULL,
    sort_order INT NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    remark TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 登录日志表
CREATE TABLE login_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    email VARCHAR(255),
    event VARCHAR(50) NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    os VARCHAR(100),
    browser VARCHAR(100),
    success BOOLEAN NOT NULL DEFAULT true,
    fail_reason TEXT,
    login_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 修改表

```sql
-- 用户表增加部门字段
ALTER TABLE users ADD COLUMN department_id UUID REFERENCES departments(id);

-- 新增系统配置项
INSERT INTO system_configs (group_key, config_key, value, value_type, description) VALUES
    ('general', 'logo_url', '', 'string', '站点 Logo URL'),
    ('security', 'allow_register', 'true', 'boolean', '是否允许自助注册'),
    ('login', 'captcha_enabled', 'false', 'boolean', '是否开启登录验证码'),
    ('login', 'force_2fa', 'false', 'boolean', '是否强制开启两步验证'),
    ('login', 'lockout_duration', '30', 'number', '登录失败锁定时长(分钟)');
```

### 新增权限种子

```sql
INSERT INTO permissions (name, description, module) VALUES
    ('departments:read', 'View department information', 'departments'),
    ('departments:create', 'Create departments', 'departments'),
    ('departments:update', 'Update departments', 'departments'),
    ('departments:delete', 'Delete departments', 'departments'),
    ('dicts:read', 'View dictionary data', 'dicts'),
    ('dicts:create', 'Create dictionary types and items', 'dicts'),
    ('dicts:update', 'Update dictionary types and items', 'dicts'),
    ('dicts:delete', 'Delete dictionary types and items', 'dicts'),
    ('login-logs:read', 'View login logs', 'audit'),
    ('users:import', 'Import users from Excel', 'users');
```

---

## 6. API 路由设计

| Method | Path | 说明 |
|--------|------|------|
| GET | `/api/departments` | 获取部门列表（支持树形 `?format=tree`） |
| POST | `/api/departments` | 创建部门 |
| PUT | `/api/departments/{id}` | 更新部门 |
| DELETE | `/api/departments/{id}` | 删除部门 |
| GET | `/api/dict-types` | 字典类型列表（分页） |
| POST | `/api/dict-types` | 创建字典类型 |
| PUT | `/api/dict-types/{id}` | 更新字典类型 |
| DELETE | `/api/dict-types/{id}` | 删除字典类型 |
| GET | `/api/dict-types/{id}/items` | 获取某字典类型下的字典项 |
| POST | `/api/dict-types/{id}/items` | 创建字典项 |
| PUT | `/api/dict-items/{id}` | 更新字典项 |
| DELETE | `/api/dict-items/{id}` | 删除字典项 |
| GET | `/api/dicts/{code}` | 根据 code 获取字典项（公开/缓存接口） |
| GET | `/api/login-logs` | 登录日志列表（分页+筛选） |
| GET | `/api/login-logs/{id}` | 登录日志详情 |
| POST | `/api/users/import` | 导入用户（multipart xlsx） |
| GET | `/api/users/import/template` | 下载导入模板 |

---

## 7. 待确认问题

| # | 问题 | 影响 |
|---|------|------|
| Q1 | 部门删除策略：当部门下有用户时，是禁止删除还是自动将用户部门置空？ | 影响 F1 删除逻辑和数据库约束 |
| Q2 | 字典数据是否需要 i18n 支持？当前字典 label 是单语言还是多语言？ | 影响字典表设计（是否需要 locale 字段） |
| Q3 | 用户导入时，密码是必填还是系统自动生成（然后通过邮件通知或 must_change_password 机制）？ | 影响导入逻辑和用户首次体验 |
| Q4 | 系统设置编辑页是替换现有 SettingsPage 还是在 SystemConfigPage 上做增强？ | 影响前端路由和页面结构 |
| Q5 | 登录日志是否需要保留策略（如自动清理 90 天前的数据）？ | 影响是否需要定时任务或手动清理功能 |
| Q6 | 字典公共接口 `/api/dicts/{code}` 是否需要认证？建议不需要，方便前端全局使用 | 影响路由注册位置（public vs protected） |
