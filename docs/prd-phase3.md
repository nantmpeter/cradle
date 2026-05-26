# PRD — cradle Phase 3: Dashboard & 系统设置

## 1. 项目信息

| 字段 | 值 |
|---|---|
| Language | 中文 |
| Programming Language | Rust (Axum + SQLx + PostgreSQL + JWT) + React 19 (Vite + shadcn/ui + Zustand + TanStack Query) |
| Project Name | `cradle` |
| Phase | Phase 3 — Dashboard 统计 & 系统设置 |

### 原始需求复述

当前 DashboardPage.tsx 是硬编码假数据的占位页面。Phase 3 需要：
1. 后端新增 Dashboard 统计 API，返回实时统计数据（用户数/角色数/今日登录/最近审计日志等）
2. 前端替换硬编码数据，展示真实统计数据，使用图表组件
3. 新增系统设置页面，展示基本系统配置（当前用户信息、系统版本等）

---

## 2. 产品定义

### Product Goals

1. **数据可视化** — 管理员登录后首屏即可看到系统核心运营指标（用户活跃度、安全事件、角色分布），无需逐页翻查
2. **权限安全** — 统计数据按权限分层展示，普通用户仅看到自身摘要，管理员看到全局统计，敏感操作日志仅 superadmin 可见
3. **系统透明** — 提供系统设置页面让管理员查看系统元信息（版本、配置摘要），便于运维排查与系统状态确认

### User Stories

| # | 角色 | 故事 |
|---|---|---|
| US-1 | 管理员 | 作为管理员，我想在 Dashboard 看到用户总数、活跃/禁用分布，以便快速了解系统用户状态 |
| US-2 | 管理员 | 作为管理员，我想在 Dashboard 看到最近 5 条审计日志，以便及时发现异常操作 |
| US-3 | 管理员 | 作为管理员，我想在 Dashboard 看到角色分布图表，以便了解权限分配概况 |
| US-4 | 超级管理员 | 作为超级管理员，我想在 Dashboard 看到今日登录次数和登录失败次数，以便监控安全状况 |
| US-5 | 任何登录用户 | 作为登录用户，我想在系统设置页面看到系统版本和我的账户基本信息，以便确认环境 |
| US-6 | 超级管理员 | 作为超级管理员，我想在系统设置页面看到服务配置摘要（数据库连接状态、JWT 过期策略等），以便运维排查 |

---

## 3. 技术规范

### Requirements Pool

#### P0 — Must Have

| ID | 需求 | 说明 |
|---|---|---|
| P0-1 | **Dashboard 统计 API** | `GET /api/dashboard/stats`，返回聚合统计数据 |
| P0-2 | **统计数据字段** | 返回内容至少包含：用户总数、活跃用户数、禁用用户数、角色总数、今日登录次数、最近 N 条审计日志摘要、角色分布（每个角色名→用户数） |
| P0-3 | **API 权限控制** | 所有登录用户可调用基础统计（自身信息）；全局统计需 `users:read` 权限；审计日志摘要需 `audit:read` 权限；API 根据调用者权限返回不同层级的数据（字段级过滤） |
| P0-4 | **Dashboard 前端页面重构** | 替换 DashboardPage.tsx 中的硬编码数据，从 API 获取真实数据 |
| P0-5 | **统计卡片组件** | 展示核心数字指标：用户总数、活跃用户、角色总数、今日登录，使用 Card 组件 |
| P0-6 | **最近审计日志列表** | Dashboard 内展示最近 5 条审计日志（时间、操作、操作者），可点击跳转至审计日志详情页 |
| P0-7 | **角色分布图表** | 使用图表展示各角色的用户数量分布（饼图或条形图） |

#### P1 — Should Have

| ID | 需求 | 说明 |
|---|---|---|
| P1-1 | **系统设置页面** | `GET /api/dashboard/settings` 返回系统元信息；前端新增 `/dashboard/settings` 页面 |
| P1-2 | **系统信息展示** | 展示：系统名称、后端版本号（从 Cargo.toml 读取或环境变量注入）、运行时长（uptime）、数据库连接状态 |
| P1-3 | **当前用户信息卡片** | 设置页面展示当前登录用户的邮箱、角色、权限列表、最后登录时间 |
| P1-4 | **用户状态分布图表** | Dashboard 新增用户状态（active/disabled）分布图 |
| P1-5 | **新增权限种子数据** | 在 Phase 2 migration 基础上，为 Phase 3 新增 `dashboard:read` 权限（属 `dashboard` module），并分配给 superadmin 和 admin 角色 |

#### P2 — Nice to Have

| ID | 需求 | 说明 |
|---|---|---|
| P2-1 | **数据刷新按钮** | Dashboard 页面手动刷新统计数据 |
| P2-2 | **自动定时刷新** | Dashboard 每 60 秒自动刷新（可配置） |
| P2-3 | **审计日志按日趋势图** | 展示近 7 天审计日志数量趋势折线图 |
| P2-4 | **今日登录用户列表** | 展示今日登录过的用户列表 |

---

### API 设计草案

#### `GET /api/dashboard/stats`

**权限**: 需认证（Bearer Token），根据用户权限返回不同层级数据

**Response（管理员视角）**:
```json
{
  "users": {
    "total": 150,
    "active": 142,
    "disabled": 8
  },
  "roles": {
    "total": 5,
    "distribution": [
      { "role_name": "superadmin", "count": 2 },
      { "role_name": "admin", "count": 10 },
      { "role_name": "user", "count": 138 }
    ]
  },
  "today_logins": 23,
  "recent_audit_logs": [
    {
      "id": "uuid",
      "action": "login",
      "user_email": "admin@example.com",
      "resource_type": "auth",
      "created_at": "2026-05-24T10:30:00Z"
    }
  ]
}
```

**字段级权限规则**:
| 字段 | 所需权限 |
|---|---|
| `users.*` | `users:read`（无权限则返回 `null`） |
| `roles.distribution` | `roles:read`（无权限则返回 `null`） |
| `today_logins` | `users:read` |
| `recent_audit_logs` | `audit:read`（无权限则返回空数组） |
| `roles.total` | 所有认证用户可见 |

#### `GET /api/dashboard/settings`

**权限**: 需认证，`dashboard:read` 权限

**Response**:
```json
{
  "system": {
    "name": "Cradle",
    "version": "0.1.0",
    "uptime_seconds": 86400
  },
  "database": {
    "status": "connected",
    "max_connections": 10
  },
  "current_user": {
    "id": "uuid",
    "email": "admin@example.com",
    "name": "Admin",
    "role": "admin",
    "permissions": ["users:read", "users:create", "..."],
    "created_at": "2026-05-20T00:00:00Z"
  }
}
```

**字段级权限规则**:
| 字段 | 所需权限 |
|---|---|
| `database.*` | `system:manage`（无权限则返回 `null`） |
| `current_user.permissions` | 所有认证用户可见 |
| `system.*` | 所有认证用户可见 |

---

### UI 设计稿描述

#### Dashboard 页面布局

```
┌─────────────────────────────────────────────────────┐
│  Dashboard                                          │
│  系统概览与运营数据                                    │
├─────────────┬─────────────┬─────────────┬───────────┤
│  📊 用户总数  │  ✅ 活跃用户  │  🛡️ 角色总数  │ 🔐 今日登录 │
│    150      │    142      │     5       │    23     │
├─────────────┴─────────────┴─────────────┴───────────┤
│                                                      │
│  ┌─── 角色分布 ────────┐  ┌─── 用户状态 ────────────┐ │
│  │                     │  │                         │ │
│  │   [饼图/条形图]      │  │   [环形图]               │ │
│  │   superadmin: 2     │  │   active: 142           │ │
│  │   admin: 10         │  │   disabled: 8           │ │
│  │   user: 138         │  │                         │ │
│  └─────────────────────┘  └─────────────────────────┘ │
│                                                      │
│  ┌─── 最近审计日志 ─────────────────────────────────┐ │
│  │ 时间              │ 操作      │ 操作者            │ │
│  │ 2026-05-24 10:30  │ login     │ admin@ex.com     │ │
│  │ 2026-05-24 10:15  │ update    │ super@ex.com     │ │
│  │ ...               │ ...       │ ...              │ │
│  │          [查看全部 →]                             │ │
│  └──────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────┘
```

#### 系统设置页面布局

```
┌─────────────────────────────────────────────────────┐
│  系统设置                                            │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌─── 系统信息 ────────────────────────────────────┐ │
│  │  系统名称:  Cradle                           │ │
│  │  版本:      0.1.0                               │ │
│  │  运行时长:   1天 2小时 30分钟                     │ │
│  └─────────────────────────────────────────────────┘ │
│                                                      │
│  ┌─── 数据库状态 ──────────────────────────────────┐ │
│  │  连接状态:  ✅ 已连接  (需 system:manage 权限)    │ │
│  │  最大连接数: 10                                  │ │
│  └─────────────────────────────────────────────────┘ │
│                                                      │
│  ┌─── 当前用户 ────────────────────────────────────┐ │
│  │  邮箱:    admin@example.com                      │ │
│  │  姓名:    Admin                                  │ │
│  │  角色:    admin                                  │ │
│  │  权限:    [users:read] [users:create] [...]      │ │
│  │  注册时间: 2026-05-20                            │ │
│  └─────────────────────────────────────────────────┘ │
│                                                      │
└──────────────────────────────────────────────────────┘
```

---

### 技术实现要点

#### 后端

1. **新增 `dashboard_handler.rs`** — 处理 `/api/dashboard/stats` 和 `/api/dashboard/settings`
2. **新增 `dashboard_repo.rs`** — 聚合查询逻辑：
   - `count_users_by_status()` — `SELECT status, COUNT(*) FROM users GROUP BY status`
   - `count_users_by_role()` — `SELECT r.name, COUNT(u.id) FROM roles r LEFT JOIN users u ON u.role_id = r.id GROUP BY r.name`
   - `count_today_logins()` — `SELECT COUNT(*) FROM audit_logs WHERE action = 'login' AND created_at >= TODAY()`
   - `recent_audit_logs(limit)` — 复用现有 `audit_log_repo::list_paginated` 或新增简化查询
3. **权限字段过滤** — handler 中根据 `AuthUser` 权限动态构建 response，无权限字段返回 `null`
4. **系统信息** — version 从编译时环境变量 `env!("CARGO_PKG_VERSION")` 注入，uptime 从 app 启动时间计算
5. **数据库状态** — 执行 `SELECT 1` 简单查询判断连接状态

#### 前端

1. **图表库选型** — 推荐 `recharts`（React 生态最成熟，shadcn/ui 生态兼容好，轻量）
2. **新增 API hooks** — `useDashboardStats()`、`useDashboardSettings()` (TanStack Query)
3. **组件拆分**：
   - `StatsCards.tsx` — 4 个统计数字卡片
   - `RoleDistributionChart.tsx` — 角色分布图表
   - `UserStatusChart.tsx` — 用户状态图表
   - `RecentAuditLogs.tsx` — 最近审计日志表格
   - `SystemInfoCard.tsx` — 系统信息卡片
   - `DatabaseStatusCard.tsx` — 数据库状态卡片
   - `CurrentUserCard.tsx` — 当前用户信息卡片
4. **DashboardPage.tsx** — 重构为组合以上组件
5. **新增 `SettingsPage.tsx`** — 系统设置页面
6. **路由更新** — `/dashboard/settings` 替换现有占位组件
7. **Sidebar 更新** — 添加 "Settings" 导航项（已在 navItems 中但指向占位路由）

---

### 待确认问题 (Open Questions)

| # | 问题 | 建议 |
|---|---|---|
| 1 | "今日登录次数"的统计口径 — 是按 `action = 'login'` 还是按 distinct user 计？ | 建议按 action='login' 记录数统计（含重复登录），简单直接 |
| 2 | `dashboard:read` 权限是否需要新增，还是复用现有权限？ | 建议新增 `dashboard:read` 权限（P1），统一控制 Dashboard 数据访问 |
| 3 | 图表库选择 — `recharts` 还是 `chart.js` (react-chartjs-2)？ | 建议 `recharts`，声明式 API 更契合 React |
| 4 | 系统设置页面是否需要可编辑功能（如修改系统名称）？ | Phase 3 仅做只读展示，编辑功能留到后续 Phase |
| 5 | Dashboard 是否需要时间范围筛选（如最近 7 天/30 天）？ | P0 不做，作为 P2 后续迭代 |
| 6 | 超级管理员是否能在 Dashboard 看到当前在线用户（活跃 session）？ | 当前架构无 session 追踪，需要额外设计，建议留到后续 Phase |
