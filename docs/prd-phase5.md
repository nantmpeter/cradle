# PRD — cradle Phase 5: 双因素认证 · 通知系统 · 审计日志增强 · 系统配置管理

## 1. 项目信息

| 字段 | 值 |
|---|---|
| Language | 中文 |
| Programming Language | Rust (Axum + SQLx + PostgreSQL + JWT) + React 19 (Vite + shadcn/ui + Zustand + TanStack Query + TanStack Router) |
| Project Name | `cradle` |
| Phase | Phase 5 — 双因素认证、通知系统、审计日志增强、系统配置管理 |

### 原始需求复述

在 Phase 1-4（用户管理、RBAC 安全、Dashboard 统计、文件上传/导出/会话/动态菜单/i18n）基础上，Phase 5 新增 4 个模块：
1. **双因素认证 (2FA)** — TOTP 绑定/解绑、QR 码、登录二次验证、恢复码、管理员强制启用策略
2. **通知系统** — 站内通知 CRUD、分类、已读管理、批量操作、角标、分页筛选
3. **审计日志增强** — 变更 diff 记录、多维度筛选、导出、详情查看
4. **系统配置管理** — 可配置参数、分组管理、多类型配置项、变更审计、动态表单渲染

---

## 2. 产品定义

### Product Goals

1. **安全纵深防御** — 通过 2FA（TOTP）为高风险操作增加第二层验证，支持管理员强制策略和恢复码，显著提升账号安全水位
2. **信息触达与可观测性** — 建立站内通知体系，让用户和管理员及时获取系统公告、安全告警和操作提醒，结合审计日志的变更 diff 实现「谁在什么时候改了什么」的完整可追溯性
3. **系统可配置性** — 将站点名称、Logo、邮件、存储、安全策略等参数从代码/环境变量迁移到后台可视化管理，降低运维门槛，配置变更全程可审计

### User Stories

#### 模块一：双因素认证 (2FA)

| # | 角色 | 故事 |
|---|---|---|
| 2FA-1 | 登录用户 | 作为登录用户，我想在安全设置页绑定 TOTP 二步验证（扫描 QR 码），以便提升我的账号安全性 |
| 2FA-2 | 登录用户 | 作为登录用户，我想在绑定时获得一组恢复码，以便在丢失认证设备时仍能登录 |
| 2FA-3 | 登录用户 | 作为登录用户，我想在登录时输入 TOTP 验证码完成二次验证，以便系统确认是我的操作 |
| 2FA-4 | 登录用户 | 作为登录用户，我想解绑我的 2FA 设备（需验证当前 TOTP 码），以便更换认证器或不再使用 |
| 2FA-5 | 管理员 | 作为管理员，我想强制要求特定角色启用 2FA，以便对高权限账号实施安全策略 |
| 2FA-6 | 管理员 | 作为管理员，我想为丢失设备的用户重置 2FA 绑定，以便帮用户恢复访问 |

#### 模块二：通知系统

| # | 角色 | 故事 |
|---|---|---|
| NT-1 | 登录用户 | 作为登录用户，我想在 Header 看到未读通知数量角标，以便第一时间获知新消息 |
| NT-2 | 登录用户 | 作为登录用户，我想在通知列表页查看所有通知（按时间倒序），并按分类/已读状态筛选，以便高效管理消息 |
| NT-3 | 登录用户 | 作为登录用户，我想批量标记通知已读或一键全部已读，以便快速清理通知 |
| NT-4 | 管理员 | 作为管理员，我想发送系统公告（广播给所有用户或指定角色），以便传达重要信息 |
| NT-5 | 系统 | 作为系统，我想在安全事件（登录锁定、2FA 绑定/解绑、密码修改）发生时自动发送通知，以便用户及时知情 |

#### 模块三：审计日志增强

| # | 角色 | 故事 |
|---|---|---|
| AL-1 | 管理员 | 作为管理员，我想在审计日志中看到修改前后的值（变更 diff），以便清楚知道具体改了什么 |
| AL-2 | 管理员 | 作为管理员，我想按操作类型、用户、日期范围、目标对象筛选审计日志，以便快速定位特定操作 |
| AL-3 | 管理员 | 作为管理员，我想将筛选后的审计日志导出为 CSV/XLSX 文件，以便离线审计或归档 |
| AL-4 | 管理员 | 作为管理员，我想点击某条审计日志查看详情（含变更 diff 的可视化展示），以便深入分析操作上下文 |

#### 模块四：系统配置管理

| # | 角色 | 故事 |
|---|---|---|
| CF-1 | 超级管理员 | 作为超级管理员，我想在后台管理页面修改系统参数（站点名称、Logo 等），而不需要修改代码或环境变量 |
| CF-2 | 超级管理员 | 作为超级管理员，我想配置项按分组展示（基础/邮件/安全/存储），以便快速定位要修改的参数 |
| CF-3 | 超级管理员 | 作为超级管理员，我想配置项支持字符串、数字、布尔、JSON 等多种类型，以便适配不同参数的数据格式 |
| CF-4 | 超级管理员 | 作为超级管理员，我想看到配置变更的历史审计记录，以便追溯谁改了什么 |
| CF-5 | 登录用户 | 作为登录用户，前端页面标题、Logo 等随系统配置动态变化，以便保持品牌一致性 |

---

## 3. 技术规范

### Requirements Pool

#### P0 — Must Have

| ID | 需求 | 说明 |
|---|---|---|
| **双因素认证** | | |
| P0-2FA-1 | **用户 2FA 绑定 API** | `POST /api/auth/2fa/enable` — 生成 TOTP secret + QR 码（base64 PNG），前端展示供用户扫描 |
| P0-2FA-2 | **2FA 绑定确认** | `POST /api/auth/2fa/confirm` — 用户提交 TOTP 码验证绑定，成功后生成恢复码（10 个一次性码） |
| P0-2FA-3 | **2FA 解绑** | `POST /api/auth/2fa/disable` — 验证当前 TOTP 码后解绑，删除 secret |
| P0-2FA-4 | **登录二次验证** | `POST /api/auth/login` 返回 `requires_2fa: true` + 临时 token；`POST /api/auth/2fa/verify` 提交 TOTP 码换正式 token |
| P0-2FA-5 | **恢复码登录** | `POST /api/auth/2fa/verify` 支持 `recovery_code` 字段，验证通过后该码作废，提示用户重新生成 |
| P0-2FA-6 | **users 表扩展** | 新增 `totp_secret`（加密存储）、`two_factor_enabled`（bool）、`recovery_codes`（JSON 数组，bcrypt 哈希） |
| P0-2FA-7 | **2FA 设置页面** | 前端安全设置 Tab：绑定流程（QR 码 + 验证码确认）、恢复码展示/重新生成、解绑 |
| P0-2FA-8 | **登录 2FA 验证页** | 登录后若用户启用了 2FA，跳转到验证页面，输入 TOTP 码或恢复码 |
| **通知系统** | | |
| P0-NT-1 | **通知数据表** | 新增 `notifications` 表（id、user_id、type、title、content、is_read、metadata、created_at）和 `notification_reads` 表（或直接在 notifications 表存 is_read） |
| P0-NT-2 | **通知 CRUD API** | `GET /api/notifications`（分页 + 筛选 type/is_read）、`GET /api/notifications/unread-count`、`PUT /api/notifications/{id}/read`、`PUT /api/notifications/read-all`、`DELETE /api/notifications/{id}` |
| P0-NT-3 | **管理员发送通知** | `POST /api/notifications`（管理员发送系统公告，支持 broadcast 或指定 role） |
| P0-NT-4 | **通知角标** | Header 新增通知铃铛图标 + 未读数量 Badge，点击弹出通知面板（最近 5 条） |
| P0-NT-5 | **通知列表页面** | 独立通知列表页，支持分类筛选（系统/安全/操作）、已读/未读筛选、批量标记已读、全部已读 |
| P0-NT-6 | **自动通知触发** | 后端 service 层在关键事件（登录锁定、2FA 绑定/解绑、密码修改、账号禁用）自动创建通知 |
| **审计日志增强** | | |
| P0-AL-1 | **变更 diff 记录** | 扩展 `audit_logs.details` 字段，记录 `old_value` 和 `new_value`（JSON），用于对比展示 |
| P0-AL-2 | **变更记录中间件/工具** | 封装 `AuditChangeBuilder`，在 service 层更新操作时记录变更前后值，自动写入 audit_logs |
| P0-AL-3 | **增强筛选** | `GET /api/audit-logs` 支持 `resource_type`、`resource_id`、`action`、`user_id`、`from`、`to` 多维度筛选 |
| P0-AL-4 | **审计日志详情 API** | `GET /api/audit-logs/{id}` 返回完整详情含 `old_value`/`new_value` diff |
| P0-AL-5 | **审计日志详情前端** | 点击日志条目查看详情，diff 以「删除行（红）/新增行（绿）」方式可视化展示 |
| P0-AL-6 | **审计日志导出** | `GET /api/audit-logs/export?format=csv|xlsx`，支持筛选参数，返回文件流 |
| **系统配置管理** | | |
| P0-CF-1 | **系统配置表** | 新增 `system_configs` 表（id、group_key、config_key、value、value_type、description、created_at、updated_at） |
| P0-CF-2 | **配置 CRUD API** | `GET /api/configs`（按 group 返回）、`PUT /api/configs/{key}`（更新配置值）、`GET /api/configs/public`（公开配置如站点名称，无需认证） |
| P0-CF-3 | **配置分组** | 预置分组：general（基础设置）、email（邮件设置）、security（安全策略）、storage（存储设置） |
| P0-CF-4 | **多类型配置值** | value_type 支持：string、number、boolean、json；后端根据类型解析/校验，前端根据类型渲染不同表单控件 |
| P0-CF-5 | **配置变更审计** | 配置更新时自动写入 audit_logs，记录 old_value → new_value |
| P0-CF-6 | **系统设置页面** | 前端设置页 Tabs 改为按分组渲染动态表单（Text/Number/Switch/JSON Editor），保存时调用 PUT API |
| P0-CF-7 | **配置种子数据** | 迁移脚本插入默认配置项（站点名称、Logo URL、会话超时、最大登录尝试、文件大小限制等） |

#### P1 — Should Have

| ID | 需求 | 说明 |
|---|---|---|
| P1-2FA-1 | **角色级 2FA 强制策略** | 角色表新增 `require_2fa` 字段；登录时检查用户角色是否要求 2FA，若未绑定则引导绑定 |
| P1-2FA-2 | **管理员重置用户 2FA** | `DELETE /api/users/{id}/2fa` — 管理员清除用户 TOTP secret 和恢复码 |
| P1-2FA-3 | **恢复码重新生成** | `POST /api/auth/2fa/recovery-codes` — 验证 TOTP 后重新生成恢复码（旧码全部作废） |
| P1-NT-1 | **通知删除** | 用户可删除自己的通知（软删除或硬删除） |
| P1-NT-2 | **通知分页优化** | 游标分页（cursor-based）替代 offset 分页，提升大数据量性能 |
| P1-AL-1 | **操作类型图标** | 不同操作类型（create/update/delete/login）在列表中显示不同颜色/图标 |
| P1-CF-1 | **配置缓存** | 配置值内存缓存，更新时刷新，避免每次请求查库 |
| P1-CF-2 | **配置项描述** | 每个配置项有 `description` 字段，前端在表单字段下方展示说明文字 |

#### P2 — Nice to Have

| ID | 需求 | 说明 |
|---|---|---|
| P2-2FA-1 | **WebAuthn/FIDO2** | 支持硬件安全密钥作为第二因子 |
| P2-2FA-2 | **2FA 统计** | 管理员查看 2FA 启用率统计 |
| P2-NT-1 | **邮件推送** | 通知同时发送邮件（需邮件服务配置） |
| P2-NT-2 | **WebSocket 实时推送** | 通知实时推送到前端，无需轮询 |
| P2-NT-3 | **通知模板** | 可配置的通知消息模板 |
| P2-AL-1 | **审计日志归档** | 自动归档 90 天以上的日志到归档表 |
| P2-CF-1 | **配置版本历史** | 保留配置历史版本，支持回滚 |

---

### API 设计草案

#### 双因素认证

| Method | Path | 说明 | 权限 |
|--------|------|------|------|
| POST | `/api/auth/2fa/enable` | 生成 TOTP secret + QR 码 | 认证用户 |
| POST | `/api/auth/2fa/confirm` | 确认绑定（提交 TOTP 码，获得恢复码） | 认证用户 |
| POST | `/api/auth/2fa/disable` | 解绑 2FA（需验证 TOTP 码） | 认证用户 |
| POST | `/api/auth/2fa/verify` | 登录二次验证（TOTP 码或恢复码） | 临时 token |
| POST | `/api/auth/2fa/recovery-codes` | 重新生成恢复码 | 认证用户 |
| DELETE | `/api/users/{id}/2fa` | 管理员重置用户 2FA | `users:manage` |

#### 通知系统

| Method | Path | 说明 | 权限 |
|--------|------|------|------|
| GET | `/api/notifications` | 通知列表（分页 + 筛选） | 认证用户 |
| GET | `/api/notifications/unread-count` | 未读数量 | 认证用户 |
| POST | `/api/notifications` | 发送通知/公告 | `notifications:send` |
| PUT | `/api/notifications/{id}/read` | 标记已读 | 认证用户（自己） |
| PUT | `/api/notifications/read-all` | 全部标记已读 | 认证用户 |
| DELETE | `/api/notifications/{id}` | 删除通知 | 认证用户（自己） |

#### 审计日志增强

| Method | Path | 说明 | 权限 |
|--------|------|------|------|
| GET | `/api/audit-logs` | 审计日志列表（增强筛选） | `audit:read` |
| GET | `/api/audit-logs/{id}` | 审计日志详情（含 diff） | `audit:read` |
| GET | `/api/audit-logs/export` | 导出审计日志 | `audit:read` |

#### 系统配置管理

| Method | Path | 说明 | 权限 |
|--------|------|------|------|
| GET | `/api/configs` | 获取所有配置（按分组） | `configs:read` |
| GET | `/api/configs/public` | 获取公开配置（无需认证） | 公开 |
| PUT | `/api/configs/{key}` | 更新配置项 | `configs:update` |

---

### 数据模型变更

```sql
-- ========= 双因素认证 =========

-- users 表扩展字段
ALTER TABLE users ADD COLUMN totp_secret VARCHAR(255);         -- AES 加密的 TOTP secret
ALTER TABLE users ADD COLUMN two_factor_enabled BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE users ADD COLUMN recovery_codes JSONB DEFAULT '[]'; -- bcrypt 哈希后的恢复码数组

-- ========= 通知系统 =========

CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),                         -- NULL 表示广播通知
    target_role VARCHAR(50),                                    -- 指定角色（广播时使用）
    type VARCHAR(20) NOT NULL DEFAULT 'system',                -- system / security / operation
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB,                                             -- 额外数据（如关联资源 ID）
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_is_read ON notifications(user_id, is_read);
CREATE INDEX idx_notifications_type ON notifications(type);
CREATE INDEX idx_notifications_created_at ON notifications(created_at DESC);

-- 用户读取广播通知的记录表
CREATE TABLE notification_reads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_id UUID NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    read_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(notification_id, user_id)
);
CREATE INDEX idx_notification_reads_user ON notification_reads(user_id);

-- ========= 审计日志增强 =========

-- audit_logs 表无需新增列，利用现有 details JSONB 字段存储 diff
-- details 结构扩展：
-- {
--   "old_value": { "name": "John", "role": "user" },
--   "new_value": { "name": "Jane", "role": "admin" },
--   "changes": { "name": ["John", "Jane"], "role": ["user", "admin"] }
-- }

-- 新增 resource_type 索引加速筛选
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);

-- ========= 系统配置管理 =========

CREATE TABLE system_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_key VARCHAR(50) NOT NULL,                -- general / email / security / storage
    config_key VARCHAR(100) NOT NULL UNIQUE,
    value TEXT NOT NULL DEFAULT '',
    value_type VARCHAR(20) NOT NULL DEFAULT 'string', -- string / number / boolean / json
    description VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_system_configs_group ON system_configs(group_key);
CREATE INDEX idx_system_configs_key ON system_configs(config_key);
```

---

### UI / 交互要点

#### 2FA 安全设置（个人资料页 → 安全 Tab）

```
┌──────────────────────────────────────────────────────────┐
│  安全设置                                                 │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  双因素认证（TOTP）          [未启用] / [已启用 ✅]        │
│                                                          │
│  ┌─ 绑定流程（未启用时显示）────────────────────────────┐ │
│  │  步骤 1：使用 Google Authenticator 等应用扫描二维码    │ │
│  │  ┌─────────────┐                                     │ │
│  │  │   QR Code    │                                     │ │
│  │  │  (base64)    │     或手动输入密钥: XXXX-XXXX-XXXX  │ │
│  │  └─────────────┘                                     │ │
│  │                                                     │ │
│  │  步骤 2：输入验证码确认绑定                            │ │
│  │  [______] [确认绑定]                                  │ │
│  └─────────────────────────────────────────────────────┘ │
│                                                          │
│  ┌─ 恢复码（绑定成功后显示一次）─────────────────────────┐ │
│  │  ⚠️ 请妥善保存以下恢复码，关闭后无法再次查看           │ │
│  │  xxxx-xxxx    xxxx-xxxx    xxxx-xxxx                 │ │
│  │  xxxx-xxxx    xxxx-xxxx    xxxx-xxxx                 │ │
│  │  xxxx-xxxx    xxxx-xxxx    xxxx-xxxx                 │ │
│  │  xxxx-xxxx    xxxx-xxxx                               │ │
│  │                        [已保存，确认] [重新生成]       │ │
│  └─────────────────────────────────────────────────────┘ │
│                                                          │
│  [关闭 2FA] [重新生成恢复码]  （已启用时显示）              │
└──────────────────────────────────────────────────────────┘
```

#### 登录 2FA 验证页

```
┌─────────────────────────────┐
│  双因素认证验证              │
│                             │
│  请输入验证器中的 6 位数字码  │
│  [ _ _ _ _ _ _ ]            │
│                             │
│  [验证]                     │
│                             │
│  ── 或使用恢复码 ──         │
│  [输入恢复码]  [验证]        │
└─────────────────────────────┘
```

#### 通知面板（Header 弹出）

```
┌──────────────────────────────────────┐
│  通知                    [全部已读]   │
├──────────────────────────────────────┤
│  🔒 安全 | 您的账号已绑定 2FA    2分钟前│
│  📢 系统 | 系统维护通知          1小时前│
│  ⚙️ 操作 | 密码修改成功          3小时前│
│  🔒 安全 | 异地登录提醒          昨天  │
│  📢 系统 | 新功能上线通知        2天前  │
├──────────────────────────────────────┤
│               [查看全部 →]           │
└──────────────────────────────────────┘
```

#### 通知列表页

```
┌──────────────────────────────────────────────────────┐
│  通知中心                                              │
├──────────────────────────────────────────────────────┤
│  [全部] [系统] [安全] [操作]    [未读 ▼]  [全部已读]   │
├──────────────────────────────────────────────────────┤
│  🔵 🔒 安全 | 异地登录提醒                             │
│  │  您的账号在新设备上登录...         昨天 10:30  [删除] │
│  ├─────────────────────────────────────────────────── │
│  🔵 📢 系统 | 系统维护通知                             │
│  │  系统将于周六凌晨...              前天 09:00  [删除] │
│  ├─────────────────────────────────────────────────── │
│     ⚙️ 操作 | 密码修改成功                             │
│  │  您的密码已成功修改...             3天前        [删除] │
│  ...                                                 │
├──────────────────────────────────────────────────────┤
│                    ◀ 1 2 3 ▶                          │
└──────────────────────────────────────────────────────┘
```

#### 审计日志详情（Diff 展示）

```
┌──────────────────────────────────────────────────────────┐
│  审计日志详情                                    [关闭]   │
├──────────────────────────────────────────────────────────┤
│  操作：更新用户    操作人：admin@ex.com    时间：2025-01-15 │
│  目标：用户 user@ex.com                                  │
├──────────────────────────────────────────────────────────┤
│  变更内容：                                               │
│                                                          │
│  字段        │ 修改前          │ 修改后                   │
│  ───────────┼────────────────┼────────────────           │
│  name       │ John           │ Jane                     │
│  role       │ user           │ admin                    │
│  status     │ active         │ active                   │
│                                                          │
│  IP：192.168.1.100    UA：Chrome/macOS                   │
└──────────────────────────────────────────────────────────┘
```

#### 系统配置页面（动态表单）

```
┌──────────────────────────────────────────────────────────┐
│  系统设置                                                 │
│  [基础设置] [邮件设置] [安全策略] [存储设置]               │
├──────────────────────────────────────────────────────────┤
│  ▸ 基础设置                                              │
│                                                          │
│  站点名称    [Cradle                    ]             │
│  站点描述    [现代化的后台管理系统            ]             │
│  Logo URL    [https://example.com/logo.png   ]           │
│                                                          │
│  ▸ 安全策略                                              │
│                                                          │
│  最大登录尝试 [5    ]                                     │
│  锁定时长(分) [30   ]                                     │
│  会话超时(分) [120  ]                                     │
│  强制 2FA    [Toggle OFF]                                 │
│                                                          │
│                                    [保存设置]             │
└──────────────────────────────────────────────────────────┘
```

---

### 技术实现要点

#### 后端新增模块

| 模块 | Handler | Service | Repository |
|------|---------|---------|------------|
| 双因素认证 | `auth_handler.rs` 扩展 | `totp_service.rs`（新增） | `user_repo.rs` 扩展 |
| 通知系统 | `notification_handler.rs`（新增） | `notification_service.rs`（新增） | `notification_repo.rs`（新增） |
| 审计日志增强 | `audit_handler.rs` 扩展 | `audit_service.rs` 扩展 | `audit_log_repo.rs` 扩展 |
| 系统配置 | `config_handler.rs`（新增） | `config_service.rs`（新增） | `config_repo.rs`（新增） |

#### 后端依赖新增

- `totp-rs` — TOTP 生成与验证（QR 码生成 + 验证码校验）
- `qrcode` — QR 码生成（base64 PNG）
- `base64` — QR 码 base64 编码（可能已引入）

#### 前端新增

- 新增页面：通知中心、2FA 验证页
- 新增组件：NotificationBell（Header 角标）、NotificationPanel（弹出面板）、DiffViewer（变更对比）
- 修改页面：Profile（安全设置 Tab）、Settings（动态表单）、AuditLogListPage（详情 diff 展示、导出按钮）
- 新增路由：`/notifications`、`/2fa/verify`

#### 新增权限种子数据

| 权限名 | 模块 | 说明 |
|--------|------|------|
| `2fa:manage` | 2FA | 管理员重置用户 2FA |
| `notifications:read` | notifications | 查看自己的通知 |
| `notifications:send` | notifications | 发送通知/公告 |
| `configs:read` | configs | 查看系统配置 |
| `configs:update` | configs | 修改系统配置 |

---

## 4. 待确认问题

| # | 问题 | 建议 |
|---|---|---|
| 1 | **TOTP secret 存储加密** — 是否需要 AES 加密存储 TOTP secret，还是直接存明文？ | 建议使用 AES-256-GCM 加密，密钥从环境变量读取，避免数据库泄露后 secret 被直接使用 |
| 2 | **恢复码存储方式** — 恢复码是明文存储还是哈希存储？ | 建议类似密码处理，使用 bcrypt 单向哈希存储；用户绑定时仅展示一次明文，之后只做哈希比对 |
| 3 | **2FA 强制策略粒度** — 是角色级（整个角色强制）还是用户级（管理员手动指定）？ | 建议 Phase 5 P1 实现角色级策略（`roles` 表新增 `require_2fa`），P2 再考虑更细粒度 |
| 4 | **广播通知的实现方式** — 发送给「所有用户」的通知是插入 N 条记录还是用 NULL user_id + 判断逻辑？ | 建议通知表 `user_id` 允许 NULL 表示广播；另建 `notification_reads` 表记录用户已读状态，避免为每个用户插入一条记录 |
| 5 | **未读通知角标更新策略** — 前端轮询还是 WebSocket 推送？ | 建议 P0 使用轮询（Header 组件挂载时 + 定时 30s），P2 升级为 WebSocket |
| 6 | **审计日志 diff 记录范围** — 所有更新操作都记录 diff，还是仅关键操作？ | 建议仅关键资源（用户/角色/权限/配置）的 update 操作记录 diff，login/logout 等操作不记录 |
| 7 | **系统配置缓存策略** — 配置是否需要缓存？ | 建议 P1 实现应用启动时加载到内存 + 更新时刷新，避免每次请求查库 |
| 8 | **通知保留策略** — 通知是否有保留期限？ | 建议 P0 不限制，P2 添加自动清理（如 90 天自动删除已读通知） |
