# Phase 1 用户管理 - 交付完成

## 交付概览

| 项目 | 状态 |
|------|------|
| 后端 API | ✅ 全部通过 |
| 前端页面 | ✅ 全部通过 |
| 集成测试 | ✅ 24/24 通过 |
| 数据库迁移 | ✅ 已运行 |

## 后端 API 清单

| 端点 | 方法 | 功能 | 权限 |
|------|------|------|------|
| `/api/auth/register` | POST | 公开注册（强制 role=user） | 公开 |
| `/api/auth/login` | POST | 登录（返回 access+refresh token） | 公开 |
| `/api/auth/refresh` | POST | 刷新 token（旧 token 自动失效） | 公开 |
| `/api/auth/logout` | POST | 登出（吊销 refresh token） | 公开 |
| `/api/users` | GET | 分页列表（搜索/筛选/排序） | admin+ |
| `/api/users` | POST | 创建用户 | admin+ |
| `/api/users/me` | GET | 获取当前用户信息 | 认证 |
| `/api/users/me/password` | PUT | 修改自己的密码 | 认证 |
| `/api/users/{id}` | GET | 获取指定用户 | 认证 |
| `/api/users/{id}` | PUT | 更新用户（名字/角色） | admin+ |
| `/api/users/{id}` | DELETE | 删除用户 | admin+ |
| `/api/users/{id}/status` | PUT | 启用/禁用用户 | admin+ |
| `/api/users/{id}/password` | PUT | 重置用户密码 | admin+ |

## 前端页面

| 页面 | 路由 | 功能 |
|------|------|------|
| 登录 | `/login` | 邮箱+密码登录 |
| Dashboard | `/dashboard` | 总览页 |
| 用户管理 | `/dashboard/users` | 列表+搜索+创建+编辑+删除+启禁+重置密码 |
| 个人资料 | `/dashboard/profile` | 查看信息+改名字+改密码+强制改密提示 |

## 测试结果

24 个集成测试全部通过，覆盖：
- 注册（成功/重复邮箱/弱密码/强制角色）
- 登录（成功/错误密码/不存在用户/被禁用用户）
- Token 刷新（成功轮换/旧 token 失效）
- 登出（吊销 refresh token）
- 用户 CRUD（列表权限/创建/更新/删除/自我保护）
- 状态切换（启用/禁用）
- 密码管理（重置设 must_change_password/自己改密/旧密码错误）
- 认证检查（无 token/无效 token）

## 启动方式

```bash
# 后端（需要 PostgreSQL 运行）
cd apps/backend && cargo run

# 前端
cd apps/frontend && npx vite --host
```

- 后端: http://localhost:8080
- 前端: http://localhost:5173
- 登录凭据: `admin@example.com` / `Admin@1234`

## 数据库

```bash
# 运行迁移
cd apps/backend && sqlx migrate run --database-url "postgres://dev:dev@localhost:5432/cradle"
```

## 运行测试

```bash
cd apps/backend && cargo test
```
