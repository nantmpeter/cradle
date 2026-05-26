// ─── Dashboard 统计数据类型 ──────────────────────────────────────────────────

/** Dashboard Stats API 响应类型 — 字段可能因权限为 null */
export interface DashboardStats {
  users: {
    total: number
    active: number
    disabled: number
  } | null
  roles: {
    total: number
    distribution: { role_name: string; count: number }[] | null
  }
  today_logins: number | null
  recent_audit_logs: {
    id: string
    action: string
    user_email: string | null
    resource_type: string | null
    created_at: string
  }[]
}

// ─── Dashboard 系统设置类型 ──────────────────────────────────────────────────

/** Dashboard Settings API 响应类型 — database 可能因权限为 null */
export interface DashboardSettings {
  system: {
    name: string
    version: string
    uptime_seconds: number
  }
  database: {
    status: string
    max_connections: number
  } | null
  current_user: {
    id: string
    email: string
    name: string | null
    role: string
    two_factor_enabled: boolean
    permissions: string[]
    created_at: string
  }
}
