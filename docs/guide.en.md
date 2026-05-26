# Cradle — Usage Guide

> Version: 0.1.0 · Last updated: 2026-05-25

## Table of Contents

- [1. Quick Start](#1-quick-start)
- [2. Configuration](#2-configuration)
- [3. Authentication](#3-authentication)
- [4. API Routing Architecture](#4-api-routing-architecture)
- [5. Admin API Reference](#5-admin-api-reference)
- [6. App API Reference](#6-app-api-reference)
- [7. Open API Reference](#7-open-api-reference)
- [8. API Key Management](#8-api-key-management)
- [9. OAuth2 Third-Party Login](#9-oauth2-third-party-login)
- [10. Webhook Outbound Notifications](#10-webhook-outbound-notifications)
- [11. Error Format](#11-error-format)
- [12. Permission System](#12-permission-system)
- [13. Data Permissions](#13-data-permissions)
- [14. Testing](#14-testing)
- [15. Deployment](#15-deployment)
- [16. FAQ](#16-faq)

---

## 1. Quick Start

### 1.1 Prerequisites

| Dependency | Version | Notes |
|------------|---------|-------|
| Rust | 1.85+ (edition 2024) | Backend compilation |
| Node.js | 20+ | Frontend build |
| PostgreSQL | 16 | Database |
| pnpm | Recommended | Frontend package manager |

### 1.2 Setup

```bash
# 1. Clone
git clone <repo-url> && cd cradle

# 2. Start database (pick one)
docker compose up db -d              # Docker
# createdb cradle                # Local PostgreSQL

# 3. Backend
cd apps/backend
cp config/default.toml config/local.toml
cargo run
# → http://localhost:8080

# 4. Frontend
cd apps/frontend
pnpm install
pnpm dev
# → http://localhost:5173
```

### 1.3 Default Credentials

| Email | Password | Role |
|-------|----------|------|
| `admin@example.com` | `Admin@1234` | superadmin |

---

## 2. Configuration

Backend config lives in `apps/backend/config/`. Load order: `default.toml` → `local.toml` (override) → environment variables (`APP__` prefix).

### 2.1 Full Configuration

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgres://dev:dev@localhost:5432/cradle"
max_connections = 10

[jwt]
secret = "change-me-in-production-use-strong-random-key"
access_exp_secs = 3600       # 1 hour
refresh_exp_secs = 604800    # 7 days

[rate_limit]
global_rpm = 100
login_rpm = 5

[storage]
upload_dir = "./uploads"
max_upload_size = 10485760   # 10 MB
max_avatar_size = 2097152    # 2 MB

[totp]
encryption_key = "jBrgUIZ+TRxlYByzq83lkK0pR9lLQQP/ZmRWxDK6r8A="

[api]
version = "v1"
legacy_routes_enabled = true
docs_enabled = true

[api.wechat]
enabled = false
app_id = ""
app_secret = ""

[api.sms]
enabled = false
provider = ""
template_id = ""

[api.github]                 # GitHub OAuth2
client_id = ""
client_secret = ""

[api.google]                 # Google OAuth2
client_id = ""
client_secret = ""

[webhook]
max_retries = 3
retry_intervals_secs = [60, 300, 1800]
timeout_secs = 10
```

### 2.2 Environment Variable Override

Use `APP__` prefix with `__` separator:

```bash
APP__DATABASE__URL=postgres://user:pass@host:5432/db
APP__JWT__SECRET=my-production-secret
APP__SERVER__PORT=9090
```

---

## 3. Authentication

Three authentication methods correspond to three API tiers:

| Method | Use Case | Header |
|--------|----------|--------|
| JWT + RBAC | Admin dashboard | `Authorization: Bearer <access_token>` |
| JWT (client_type=app) | Mobile / Mini-program | `Authorization: Bearer <access_token>` |
| API Key + Scope | Third-party integration | `Authorization: Bearer rk_xxx` or `X-API-Key: rk_xxx` |

### 3.1 Login

**POST** `/api/auth/login`

Request:
```json
{
  "email": "admin@example.com",
  "password": "Admin@1234"
}
```

Success (200):
```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_type": "bearer",
  "must_change_password": false
}
```

2FA required (401):
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

The 2FA response also includes `requires_2fa: true` and `temp_token` for the next step.

**POST** `/api/auth/2fa/verify`

Request:
```json
{
  "temp_token": "temp_xxx...",
  "code": "123456"
}
```

### 3.2 Token Refresh

**POST** `/api/auth/refresh`

Request:
```json
{ "refresh_token": "eyJ..." }
```

### 3.3 Registration

**POST** `/api/auth/register`

Request:
```json
{
  "email": "new@example.com",
  "password": "StrongPass@123",
  "name": "New User"
}
```

Password rules: minimum 8 characters, at least 3 of: uppercase, lowercase, digit, special character. New users get the `user` role automatically.

---

## 4. API Routing Architecture

### 4.1 Three-Tier Routing

| Tier | Prefix | Auth | Description |
|------|--------|------|-------------|
| **Admin** | `/api/v1/admin/*` | JWT + RBAC + permission check | Management dashboard |
| **App** | `/api/v1/app/*` | JWT (client_type=app) | Mobile / Mini-program |
| **Open** | `/api/v1/open/*` | API Key + Scope | Third-party integration |
| **Legacy** | `/api/*` | JWT + RBAC | Backward compatibility (with Deprecation header) |

### 4.2 Legacy Route Compatibility

Legacy routes remain fully functional with added response headers:

```
Deprecation: true
X-Legacy-Route: true
```

Set `api.legacy_routes_enabled = false` to disable.

### 4.3 Non-Versioned Routes

| Path | Description |
|------|-------------|
| `/api/auth/*` | Auth (login, register, refresh, 2FA) |
| `/api/sse/notifications` | SSE real-time notification stream |
| `/api/i18n/messages` | i18n messages |
| `/api/configs/public` | Public configuration |

---

## 5. Admin API Reference

All Admin API endpoints are mounted under `/api/v1/admin/` (legacy `/api/` also works). Requires JWT authentication.

### 5.1 User Management

| Method | Path | Description | Permission |
|--------|------|-------------|------------|
| GET | `/users` | User list (paginated) | users:read |
| POST | `/users` | Create user | users:create |
| GET | `/users/me` | Current user info (with permissions) | — |
| GET | `/users/{id}` | User detail | users:read |
| PUT | `/users/{id}` | Update user | users:update |
| DELETE | `/users/{id}` | Delete user | users:delete |
| PUT | `/users/{id}/status` | Toggle status | users:update |
| PUT | `/users/{id}/password` | Reset password | users:update |
| POST | `/users/{id}/2fa/reset` | Reset user 2FA | users:update |
| PUT | `/users/me/password` | Change own password | — |
| POST | `/users/me/avatar` | Upload avatar | — |
| POST | `/users/import/` | Batch import users | users:create |
| GET | `/users/import/template` | Download import template | users:create |

**User List** `GET /users`

Query parameters:
| Param | Type | Description |
|-------|------|-------------|
| page | integer | Page number (default 1) |
| per_page | integer | Per page (default 20) |
| search | string | Search keyword |
| role | string | Role filter |
| status | string | Status filter |
| department_id | uuid | Department filter |

Response:
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

**Create User** `POST /users`

Request:
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

**GET /users/me Response** (includes permission list):
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
  "permissions": ["users:read", "users:create", "*"],
  "created_at": "...",
  "updated_at": "..."
}
```

### 5.2 Roles & Permissions

| Method | Path | Description |
|--------|------|-------------|
| GET | `/roles` | Role list |
| POST | `/roles` | Create role |
| GET | `/roles/{id}` | Role detail |
| PUT | `/roles/{id}` | Update role |
| DELETE | `/roles/{id}` | Delete role |
| GET | `/roles/permissions` | List all available permissions |
| PUT | `/roles/{id}/permissions` | Update role permissions |

**Update Permissions** `PUT /roles/{id}/permissions`

Request:
```json
{
  "permissions": ["users:read", "users:create", "roles:read"]
}
```

### 5.3 Department Management

| Method | Path | Description |
|--------|------|-------------|
| GET | `/departments` | Department list (tree support) |
| POST | `/departments` | Create department |
| PUT | `/departments/{id}` | Update department |
| DELETE | `/departments/{id}` | Delete department |

### 5.4 Dictionary Management

| Method | Path | Description |
|--------|------|-------------|
| GET | `/dict-types` | Dictionary type list |
| POST | `/dict-types` | Create type |
| PUT | `/dict-types/{id}` | Update type |
| DELETE | `/dict-types/{id}` | Delete type |
| GET | `/dict-types/{id}/items` | Dictionary item list |
| POST | `/dict-types/{id}/items` | Create item |
| PUT | `/dict-items/{id}` | Update item |
| DELETE | `/dict-items/{id}` | Delete item |
| GET | `/dicts/{code}` | Lookup by code (public) |

### 5.5 File Management

| Method | Path | Description |
|--------|------|-------------|
| GET | `/files` | File list |
| POST | `/files/` | Upload file (multipart/form-data) |
| DELETE | `/files/{id}` | Delete file |

Upload limits: 10 MB for files, 2 MB for avatars.

### 5.6 Data Export

| Method | Path | Description |
|--------|------|-------------|
| GET | `/export/users` | Export users (CSV/XLSX) |

Query parameters:
| Param | Type | Description |
|-------|------|-------------|
| format | string | `csv` or `xlsx` |
| columns | string | Column selection (comma-separated) |

### 5.7 Audit Logs

| Method | Path | Description |
|--------|------|-------------|
| GET | `/audit-logs` | Log list (paginated) |
| GET | `/audit-logs/{id}` | Log detail |
| GET | `/audit-logs/export` | Export logs |

### 5.8 Login Logs

| Method | Path | Description |
|--------|------|-------------|
| GET | `/login-logs` | Log list (paginated) |
| GET | `/login-logs/{id}` | Log detail |

### 5.9 Session Management

| Method | Path | Description |
|--------|------|-------------|
| GET | `/sessions` | All active sessions |
| GET | `/sessions/me` | Current user sessions |
| DELETE | `/sessions/{id}` | Terminate session |
| DELETE | `/sessions/user/{user_id}` | Force logout all sessions for user |

### 5.10 Menu Management

| Method | Path | Description |
|--------|------|-------------|
| GET | `/menus` | Menu list |
| POST | `/menus` | Create menu |
| GET | `/menus/tree` | Menu tree structure |
| PUT | `/menus/{id}` | Update menu |
| DELETE | `/menus/{id}` | Delete menu |

### 5.11 Notifications

| Method | Path | Description |
|--------|------|-------------|
| GET | `/notifications` | Notification list |
| GET | `/notifications/unread-count` | Unread count |
| POST | `/notifications/read-all` | Mark all read |
| POST | `/notifications/{id}/read` | Mark as read |
| DELETE | `/notifications/{id}` | Delete notification |
| POST | `/notifications/broadcast` | Broadcast notification |

**SSE Stream** `GET /api/sse/notifications?token=<access_token>`

EventSource connection for real-time notification push.

### 5.12 System Config

| Method | Path | Description |
|--------|------|-------------|
| GET | `/configs` | All configs |
| GET | `/configs/{group}` | Config by group |
| PUT | `/configs/{group}/{key}` | Update config |
| GET | `/api/configs/public` | Public config (no auth) |

### 5.13 Global Search

**GET** `/search?q=keyword`

Response:
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

| Method | Path | Description |
|--------|------|-------------|
| GET | `/dashboard/stats` | System statistics |
| GET | `/dashboard/settings` | Dashboard settings |

### 5.15 2FA Management

| Method | Path | Description |
|--------|------|-------------|
| POST | `/auth/2fa/setup` | Generate TOTP secret + QR code |
| POST | `/auth/2fa/enable` | Enable 2FA |
| POST | `/auth/2fa/disable` | Disable 2FA |

---

## 6. App API Reference

App API is mounted under `/api/v1/app/`, requires JWT with `client_type=app`.

### 6.1 App Auth (public, no JWT required)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/app/auth/providers` | List available OAuth providers |
| GET | `/api/v1/app/auth/authorize?provider=github` | Get OAuth authorization URL |
| GET | `/api/v1/app/auth/callback?provider=github&code=xxx` | OAuth callback, exchange for App JWT |
| POST | `/api/v1/app/auth/sms/send` | Send SMS verification code |
| POST | `/api/v1/app/auth/sms/verify` | Verify SMS code, exchange for App JWT |

**Get Authorization URL** `GET /api/v1/app/auth/authorize?provider=github`

Response:
```json
{
  "url": "https://github.com/login/oauth/authorize?client_id=xxx&..."
}
```

**OAuth Callback** `GET /api/v1/app/auth/callback?provider=github&code=xxx`

Response (auto-register or link existing account):
```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_type": "bearer",
  "must_change_password": false
}
```

**SMS Login:**

Send code `POST /api/v1/app/auth/sms/send`
```json
{ "phone": "13800138000" }
```

Verify `POST /api/v1/app/auth/sms/verify`
```json
{ "phone": "13800138000", "code": "123456" }
```

### 6.2 App User Operations (App JWT required)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/me` | Current user info |
| PUT | `/me/password` | Change password |
| GET | `/notifications` | Notification list |
| POST | `/notifications/{id}/read` | Mark as read |
| GET | `/auth/bindings` | List social account bindings |
| POST | `/auth/social-bind` | Bind social account |
| POST | `/auth/social-unbind` | Unbind social account |

---

## 7. Open API Reference

Open API is mounted under `/api/v1/open/`, designed for third-party integration.

### 7.1 Public Endpoint (no auth required)

**GET** `/api/v1/open/health`

Response:
```json
{
  "status": "ok",
  "version": "0.1.0",
  "timestamp": "2026-05-25T10:00:00Z"
}
```

### 7.2 Protected Endpoints (API Key required)

| Method | Path | Description | Required Scope |
|--------|------|-------------|----------------|
| GET | `/users` | User list | `users:read` |
| GET | `/users/{id}` | User detail | `users:read` |
| GET | `/departments` | Department list | `departments:read` |

Usage:
```bash
curl -H "Authorization: Bearer rk_xxxxx" \
  http://localhost:8080/api/v1/open/users

# or
curl -H "X-API-Key: rk_xxxxx" \
  http://localhost:8080/api/v1/open/users
```

---

## 8. API Key Management

### 8.1 API Key Format

- Prefix: `rk_` + base64url(32 random bytes)
- Storage: SHA-256 hashed; plaintext shown only once at creation
- Display: first 8 characters for identification (e.g., `rk_abcDEF`)

### 8.2 CRUD Endpoints

| Method | Path | Description | Permission |
|--------|------|-------------|------------|
| POST | `/api-keys` | Create API Key | api-keys:manage |
| GET | `/api-keys` | List keys | — |
| DELETE | `/api-keys/{id}` | Revoke key | api-keys:manage |

**Create API Key** `POST /api/v1/admin/api-keys`

Request:
```json
{
  "name": "My Integration",
  "scopes": ["users:read", "departments:read"],
  "expires_at": "2027-01-01T00:00:00Z"
}
```

Response (**plaintext key returned only once**):
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

### 8.3 Scopes

| Scope | Description |
|-------|-------------|
| `users:read` | Read user information |
| `departments:read` | Read department information |
| `*` | Wildcard — all permissions |

Custom scopes can be added as needed.

---

## 9. OAuth2 Third-Party Login

### 9.1 Supported Providers

| Provider | Config | Status |
|----------|--------|--------|
| GitHub | `[api.github]` | ✅ Implemented |
| Google | `[api.google]` | ✅ Implemented |
| WeChat | `[api.wechat]` | 📋 Reserved |

### 9.2 Configuring OAuth

Example for GitHub:

```toml
[api.github]
client_id = "your-github-client-id"
client_secret = "your-github-client-secret"
```

GitHub OAuth App setup:
1. GitHub → Settings → Developer settings → OAuth Apps → New OAuth App
2. Authorization callback URL: `http://your-domain/api/v1/app/auth/callback?provider=github`
3. Copy Client ID and Client Secret into config

### 9.3 OAuth Flow

```
┌──────────┐   1. GET /auth/authorize?provider=github   ┌──────────┐
│  App     │ ─────────────────────────────────────────→ │  Server  │
│          │ ←──── { url: "github.com/oauth/..." } ──── │          │
│          │                                              │          │
│  Browser │   2. User redirected to GitHub auth page    │          │
│          │   3. GitHub redirects back to callback      │          │
│          │                                              │          │
│          │   4. GET /auth/callback?provider=github&code=xxx      │
│          │ ─────────────────────────────────────────→ │          │
│          │ ←──── { access_token, refresh_token } ──────│          │
│          │                                              │          │
│          │   (Token with client_type=app)              │          │
└──────────┘                                              └──────────┘
```

### 9.4 Auto-Registration/Linking Logic

1. **Existing binding**: `provider` + `provider_user_id` match → login directly
2. **Same email**: OAuth email matches existing user → auto-link and login
3. **New user**: Create new user (random password) + link → login

### 9.5 Bind/Unbind

Logged-in users can bind/unbind third-party accounts:

**Bind** `POST /api/v1/app/auth/social-bind`
```json
{
  "provider": "github",
  "code": "oauth-authorization-code"
}
```

**Unbind** `POST /api/v1/app/auth/social-unbind`
```json
{
  "provider": "github"
}
```

---

## 10. Webhook Outbound Notifications

### 10.1 Overview

Webhooks let you subscribe to system events. When triggered, the server sends HTTP POST to your URL.

Features:
- HMAC-SHA256 signature verification
- Automatic retry with exponential backoff (60s → 5min → 30min, max 3 retries)
- Delivery log queries

### 10.2 CRUD Endpoints

| Method | Path | Description | Permission |
|--------|------|-------------|------------|
| POST | `/webhooks` | Create webhook | webhooks:manage |
| GET | `/webhooks` | Webhook list | — |
| PUT | `/webhooks/{id}` | Update webhook | webhooks:manage |
| DELETE | `/webhooks/{id}` | Delete webhook | webhooks:manage |
| GET | `/webhooks/{id}/deliveries` | Delivery logs | webhooks:manage |

**Create Webhook** `POST /api/v1/admin/webhooks`

Request:
```json
{
  "name": "User Sync",
  "url": "https://example.com/webhook",
  "events": ["user.created", "user.updated", "user.deleted"]
}
```

### 10.3 Available Events

| Event | Trigger |
|-------|---------|
| `user.created` | User created |
| `user.updated` | User updated |
| `user.deleted` | User deleted |

### 10.4 Delivery Format

POST request to your URL with headers:

| Header | Description |
|--------|-------------|
| `X-Webhook-Signature` | HMAC-SHA256 signature (base64url) |
| `X-Webhook-Event` | Event name |
| `X-Webhook-ID` | Delivery record ID |

Body example:
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

### 10.5 Signature Verification

```python
import hmac, hashlib, base64

def verify_signature(secret: str, payload: bytes, signature: str) -> bool:
    expected = base64.urlsafe_b64encode(
        hmac.new(secret.encode(), payload, hashlib.sha256).digest()
    ).decode().rstrip('=')
    return hmac.compare_digest(expected, signature)
```

### 10.6 Retry Mechanism

- Failed deliveries enter the retry queue
- Retry intervals: 60s → 5min → 30min (configurable)
- Max retries: 3 (configurable)
- Non-2xx status codes are treated as failures

---

## 11. Error Format

### 11.1 Unified Error Response

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

### 11.2 Error Codes

| HTTP Status | Code | Description |
|-------------|------|-------------|
| 400 | `BAD_REQUEST` | Invalid request parameters |
| 401 | `UNAUTHORIZED` | Not authenticated |
| 401 | `TWO_FACTOR_REQUIRED` | 2FA verification required |
| 403 | `FORBIDDEN` | Insufficient permissions |
| 403 | `ACCOUNT_DISABLED` | Account disabled |
| 403 | `ACCOUNT_LOCKED` | Account locked |
| 404 | `NOT_FOUND` | Resource not found |
| 409 | `CONFLICT` | Resource conflict (e.g., email taken) |
| 413 | `PAYLOAD_TOO_LARGE` | Request body too large |
| 429 | `TOO_MANY_REQUESTS` | Rate limit exceeded |
| 500 | `INTERNAL_ERROR` | Internal server error |

### 11.3 Request ID

Every response includes an `X-Request-Id` header. Clients may also provide a custom value for tracing.

---

## 12. Permission System

### 12.1 Role Hierarchy

| Role | Description |
|------|-------------|
| `superadmin` | Super admin — all permissions |
| `admin` | Admin — assigned permissions |
| `user` | Regular user — basic permissions |

### 12.2 Permission List

| Permission | Description |
|------------|-------------|
| `users:read` | View users |
| `users:create` | Create users |
| `users:update` | Update users |
| `users:delete` | Delete users |
| `roles:read` | View roles |
| `roles:create` | Create roles |
| `roles:update` | Update roles |
| `roles:delete` | Delete roles |
| `departments:read` | View departments |
| `departments:create` | Create departments |
| `departments:update` | Update departments |
| `departments:delete` | Delete departments |
| `dicts:read` | View dictionaries |
| `dicts:create` | Create dictionaries |
| `dicts:update` | Update dictionaries |
| `dicts:delete` | Delete dictionaries |
| `menus:read` | View menus |
| `menus:create` | Create menus |
| `menus:update` | Update menus |
| `menus:delete` | Delete menus |
| `audit:read` | View audit logs |
| `sessions:read` | View sessions |
| `sessions:delete` | Terminate sessions |
| `files:read` | View files |
| `files:upload` | Upload files |
| `files:delete` | Delete files |
| `configs:read` | View system config |
| `configs:update` | Update system config |
| `notifications:read` | View notifications |
| `notifications:broadcast` | Broadcast notifications |
| `api-keys:manage` | Manage API keys |
| `webhooks:manage` | Manage webhooks |
| `login-logs:read` | View login logs |
| `import:execute` | Execute data import |
| `export:execute` | Execute data export |
| `search:execute` | Execute global search |
| `*` | Wildcard — all permissions |

---

## 13. Data Permissions

### 13.1 Data Scope

Roles can be configured with data visibility:

| Data Scope | Description |
|------------|-------------|
| `all` | All data |
| `department` | Own department only |
| `department_and_sub` | Own department + sub-departments |
| `self` | Own data only |

### 13.2 Affected Endpoints

- User list (`GET /users`)
- Audit logs (`GET /audit-logs`)
- Login logs (`GET /login-logs`)
- Session list (`GET /sessions`)

---

## 14. Testing

### 14.1 Running Tests

```bash
cd apps/backend

# All tests (requires running PostgreSQL)
cargo test

# Phase 1-7 integration tests
cargo test --test integration_test

# Phase 8 API tests
cargo test --test phase8_api_tests

# Single test
cargo test test_login_success

# Show output
cargo test -- --nocapture
```

### 14.2 Test Statistics

| Suite | Count | Coverage |
|-------|-------|----------|
| integration_test | 35 | Auth, user CRUD, roles, departments, dicts, import/export |
| phase8_api_tests | 26 | Request ID, route versioning, API Key, App Auth, Webhook |

---

## 15. Deployment

### 15.1 Docker Compose (Recommended)

```bash
docker compose up -d
docker compose logs -f backend
docker compose down
```

Services:
- Backend: http://localhost:8080
- Frontend: http://localhost:3000
- PostgreSQL: localhost:5432

### 15.2 Manual Deployment

**Backend:**
```bash
cd apps/backend
cargo build --release
./target/release/cradle-backend
```

**Frontend:**
```bash
cd apps/frontend
pnpm build
# Output in dist/ — serve with Nginx
```

### 15.3 Production Checklist

- [ ] Change `jwt.secret` to a strong random key
- [ ] Change `totp.encryption_key` to a random Base64 key
- [ ] Use strong password for PostgreSQL
- [ ] Set `storage.upload_dir` to a persistent directory
- [ ] Configure OAuth2 client_id / client_secret
- [ ] Optionally disable `api.legacy_routes_enabled`
- [ ] Configure reverse proxy (Nginx) with CORS and HTTPS
- [ ] Set log level (`RUST_LOG=info`)

### 15.4 Environment Variables

| Variable | Description |
|----------|-------------|
| `APP__DATABASE__URL` | Database connection string |
| `APP__JWT__SECRET` | JWT signing secret |
| `APP__SERVER__PORT` | Listen port |
| `RUST_LOG` | Log level (`debug`, `info`, `warn`, `error`) |

---

## 16. FAQ

### Q: Login returns TWO_FACTOR_REQUIRED but I disabled 2FA?

Check the `users` table — `two_factor_enabled` might still be `true` from manual testing.

### Q: Can I still use the old routes?

Yes. All `/api/*` routes are fully compatible. They include a `Deprecation: true` header. Set `api.legacy_routes_enabled = false` to disable.

### Q: Can I see the plaintext API Key after creation?

No. The plaintext key is returned only in the creation response. The database stores only the SHA-256 hash. If lost, revoke and create a new one.

### Q: What happens when webhook delivery fails?

The system retries 3 times (60s/5min/30min intervals). Check `GET /webhooks/{id}/deliveries` for delivery logs and failure reasons.

### Q: How do I create an API Key for a third party?

1. Login as admin
2. `POST /api/v1/admin/api-keys` (requires `api-keys:manage`)
3. Securely share the `rk_xxx` plaintext key with the integrator
4. They use `Authorization: Bearer rk_xxx` or `X-API-Key: rk_xxx` for Open API calls

### Q: How do database migrations work?

Migrations run automatically on backend startup. Manual commands:

```bash
cd apps/backend
sqlx migrate run          # Run pending migrations
sqlx migrate revert       # Revert last migration
sqlx migrate info         # Show migration status
```

### Q: Where is Swagger UI?

Visit `http://localhost:8080/docs` after starting the backend. Disable with `api.docs_enabled = false`.
