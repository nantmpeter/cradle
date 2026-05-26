# Cradle

[中文文档](./README.zh-CN.md)

> **Where great systems begin.**

Every product starts somewhere. Most begin with a fragile admin panel that works — until it doesn't. Teams outgrow it, patch around it, and eventually rebuild from scratch. **Cradle** is the starting point that doesn't need replacing.

The name says it all: a cradle nurtures what you place in it. Cradle gives your product a memory-safe, layered API architecture from day one — and scales with you through every growth spurt. Three-tier routing, scope-based access control, event-driven webhooks, and real-time notifications aren't add-ons you bolt on later. They're already there, waiting to be activated.

**Born different:**
- **Memory-safe by default** — Rust + Axum eliminates entire classes of runtime bugs. No GC pauses, no null dereferences in production. Your cradle doesn't crack under pressure.
- **Three APIs, one codebase** — Admin dashboard, mobile app, and third-party integrations each get their own authenticated API tier. One foundation, every direction.
- **Convention over configuration** — Every module follows the same 7-layer pattern (migration → model → repo → service → handler → route → test). New developers ship features on day one — the cradle teaches its own conventions.
- **Real-time ready** — SSE push notifications, webhook delivery with HMAC signatures, and OpenAPI docs are built in, not bolted on.

Built with **Rust** (Axum) and **React** (Vite + shadcn/ui).

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust · Axum 0.8 · SQLx · PostgreSQL 16 |
| Frontend | React 19 · TypeScript · Vite · shadcn/ui · Tailwind CSS v4 |
| State | Zustand · TanStack Query v5 |
| Auth | JWT (access + refresh) · Argon2 · TOTP 2FA · API Key · OAuth2 |
| Build | Cargo Workspace · Turborepo |

## Features

### Core (Phase 1-2)
- JWT authentication with access/refresh token rotation
- RBAC (Role-Based Access Control) with permission granularity
- User CRUD, role management, status toggle

### Dashboard & Settings (Phase 3)
- System overview (version, uptime, DB status)
- User profile with display name editing
- Password change with strength validation

### File & Session Management (Phase 4)
- File upload/download with size limits
- User export (CSV/XLSX) with column selection
- Active session management
- Dynamic sidebar menu from DB
- i18n (Chinese/English)

### Security (Phase 5)
- TOTP two-factor authentication (setup/enable/disable)
- In-app notification system
- Enhanced audit logging (resource/action detail)
- System config key-value store

### Organization (Phase 6)
- Department management (tree structure)
- Dictionary management (types + items)
- Login log tracking with UA parsing
- User import from Excel

### UX Enhancement (Phase 7)
- Global search (users/roles/departments/dicts)
- Data permission (department-level isolation)
- SSE real-time notifications
- User avatar upload
- Breadcrumb navigation
- Tab-based page navigation
- Theme customizer (8 accent colors)
- Force logout for online users

### API Infrastructure (Phase 8)
- **Three-tier API routing**: Admin (JWT+RBAC) · App (JWT+client_type) · Open (API Key+Scope)
- **Route versioning**: `/api/v1/admin/*`, `/api/v1/app/*`, `/api/v1/open/*` with legacy `/api/*` compatibility
- **API Key management**: create, list, revoke with SHA-256 hashed storage and scope-based access control
- **OAuth2 integration**: GitHub/Google authorization code flow with auto user registration/linking
- **SMS verification login**: phone number + 6-digit code authentication
- **Webhook outbound framework**: event-driven delivery with HMAC-SHA256 signatures, retry with exponential backoff
- **OpenAPI documentation**: Swagger UI at `/docs` (configurable toggle)
- **Request ID middleware**: X-Request-Id header tracing
- **Unified error format**: `{ error: { code, message, request_id, details }, status }`
- 61 integration tests passing

## Project Structure

```
cradle/
├── apps/
│   ├── backend/               # Rust Axum API server
│   │   ├── config/            # TOML config files
│   │   ├── migrations/        # SQLx database migrations
│   │   └── src/
│   │       ├── extractors/    # Custom Axum extractors (AuthUser, ApiKeyContext, RequestId)
│   │       ├── handlers/      # Route handlers (25 modules)
│   │       ├── middleware/     # Auth, rate limiting, API key, request ID, deprecation
│   │       ├── models/        # Data models + DTOs
│   │       ├── repository/    # Database access layer
│   │       ├── routes/        # Router registration
│   │       └── services/      # Business logic
│   └── frontend/              # React SPA
│       └── src/
│           ├── components/    # UI components (18 feature modules)
│           ├── hooks/         # React Query hooks
│           ├── lib/           # Utilities, API client, theme
│           ├── locales/       # i18n JSON (en-US, zh-CN)
│           ├── stores/        # Zustand stores
│           └── types/         # TypeScript type definitions
├── docs/                      # Design documents (PRD, architecture, diagrams)
├── docker-compose.yml
├── turbo.json
└── Cargo.toml                 # Workspace root
```

## Getting Started

### Prerequisites

- Rust 1.85+ (edition 2024)
- Node.js 20+
- PostgreSQL 16
- pnpm (recommended)

### 1. Database

```bash
# Option A: Docker
docker compose up db -d

# Option B: Local PostgreSQL
createdb cradle
```

### 2. Backend

```bash
cd apps/backend

# Copy and edit config (adjust DB URL if needed)
cp config/default.toml config/local.toml

# Run migrations (auto on startup, or manually)
sqlx migrate run

# Start dev server
cargo run
# → http://localhost:8080
```

### 3. Frontend

```bash
cd apps/frontend

# Install dependencies
pnpm install

# Start dev server
pnpm dev
# → http://localhost:5173
```

### 4. Login

| Email | Password |
|-------|----------|
| `admin@example.com` | `Admin@1234` |

## API Architecture

### Three-Tier Routing

All API routes follow a versioned structure. Legacy `/api/*` routes remain functional with a `Deprecation` header.

| Tier | Prefix | Authentication | Use Case |
|------|--------|---------------|----------|
| Admin | `/api/v1/admin/*` | JWT + RBAC | Management dashboard |
| App | `/api/v1/app/*` | JWT (client_type=app) | Mobile app / Mini program |
| Open | `/api/v1/open/*` | API Key + Scope | Third-party integration |
| Legacy | `/api/*` | JWT + RBAC | Backward compatibility |

### API Overview

| Module | Endpoints |
|--------|-----------|
| Auth | `POST /auth/login`, `/logout`, `/refresh`, `/2fa/*` |
| Users | `GET/POST /users`, `GET/PUT/DELETE /users/:id`, `GET /users/me` |
| Roles | `GET/POST /roles`, `GET/PUT/DELETE /roles/:id` |
| Departments | `GET/POST /departments`, `GET/PUT/DELETE /departments/:id` |
| Dicts | `GET/POST /dict-types`, `/dict-items`, `GET /dicts/:code` |
| Files | `POST /files/upload`, `GET /files/:id/download` |
| Audit | `GET /audit-logs`, `/audit-logs/stats` |
| Sessions | `GET /sessions`, `DELETE /sessions/:id` |
| Search | `GET /search?q=` |
| Dashboard | `GET /dashboard/stats`, `/dashboard/settings` |
| Notifications | `GET /notifications`, `POST /notifications/broadcast` |
| SSE | `GET /sse/notifications?token=` |
| Export | `POST /export/users` |
| Import | `POST /users/import`, `GET /users/import/template` |
| Menus | `GET/POST /menus`, `GET/PUT/DELETE /menus/:id` |
| Configs | `GET/POST/PUT/DELETE /configs` |
| API Keys | `POST/GET /api-keys`, `DELETE /api-keys/:id` |
| App Auth | `GET /auth/authorize`, `/callback`, `/providers`, `POST /auth/sms/*` |
| App User | `GET /auth/bindings`, `POST /auth/social-bind`, `/social-unbind` |
| Open API | `GET /health`, `/users`, `/departments` |
| Webhooks | `POST/GET /webhooks`, `PUT/DELETE /webhooks/:id`, `GET /webhooks/:id/deliveries` |
| Docs | `GET /docs` (Swagger UI), `/docs/openapi.json` |

## Configuration

Backend config lives in `apps/backend/config/default.toml`:

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgres://dev:dev@localhost:5432/cradle"

[jwt]
access_exp_secs = 3600       # 1 hour
refresh_exp_secs = 604800    # 7 days

[rate_limit]
global_rpm = 100
login_rpm = 5

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

## Testing

```bash
cd apps/backend

# Run all integration tests (requires running PostgreSQL)
cargo test

# Run Phase 1-7 tests
cargo test --test integration_test

# Run Phase 8 tests
cargo test --test phase8_api_tests
```

## Docker Deployment

```bash
# Build and run everything
docker compose up -d

# Backend: http://localhost:8080
# Frontend: http://localhost:3000
```

## License

Private project. All rights reserved.

## Roadmap

### Completed
- [x] **Phase 1** — User management (CRUD, status, pagination)
- [x] **Phase 2** — RBAC (roles, permissions, middleware)
- [x] **Phase 3** — Dashboard & settings (system info, profile)
- [x] **Phase 4** — File upload, export, session management, dynamic menu, i18n
- [x] **Phase 5** — 2FA, notifications, audit log enhancement, system config
- [x] **Phase 6** — Departments, dictionaries, login logs, user import
- [x] **Phase 7** — Global search, data permission, SSE, avatar, breadcrumb, tabs, theme
- [x] **Phase 8** — API infrastructure (three-tier routing, API Key, OAuth2, Webhook, OpenAPI docs)

### Planned
- [ ] **Phase 9** — Data visualization (user growth charts, login heatmaps, activity dashboards)
- [ ] **Phase 10** — Workflow engine (approval flows, status machines)
- [ ] **Phase 11** — Plugin system (dynamic route/module registration)
- [ ] **Phase 12** — Multi-tenancy support
- [ ] **Phase 13** — Mobile-responsive layout overhaul
- [ ] System resource monitor (CPU/Memory/Disk)
- [ ] Scheduled task management (cron UI)
- [ ] Data backup & restore utility
