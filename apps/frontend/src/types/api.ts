export type Role = "user" | "admin" | "superadmin"
export type UserStatus = "active" | "disabled"

// ─── User Types ──────────────────────────────────────────────────────────────

export interface User {
  id: string
  email: string
  name: string | null
  avatar_url: string | null
  role: Role
  role_id: string | null
  department_id: string | null
  permissions: string[]
  status: UserStatus
  must_change_password: boolean
  two_factor_enabled: boolean
  created_at: string | null
  updated_at: string | null
}

export interface TokenResponse {
  access_token: string
  refresh_token: string
  token_type: string
  must_change_password: boolean
  requires_2fa?: boolean
  temp_token?: string
}

export interface LoginRequest {
  email: string
  password: string
}

export interface RegisterRequest {
  email: string
  password: string
  name?: string
}

export interface CreateUserRequest {
  email: string
  password: string
  name?: string
  role_id: string
  department_id?: string | null
}

export interface UpdateUserRequest {
  name?: string
  role_id?: string
  department_id?: string | null
}

export interface UpdateStatusRequest {
  status: UserStatus
}

export interface ChangeMyPasswordRequest {
  current_password: string
  new_password: string
}

export interface ResetPasswordRequest {
  new_password: string
}

export interface PaginatedResponse<T> {
  data: T[]
  pagination: {
    page: number
    per_page: number
    total: number
  }
}

export interface ApiError {
  error: string
  status: number
}

export interface ListUsersParams {
  page?: number
  per_page?: number
  search?: string
  role?: string
  role_id?: string
  status?: string
  department_id?: string
  sort_by?: string
  sort_order?: string
}

// ─── Role Types ──────────────────────────────────────────────────────────────

export interface RoleResponse {
  id: string
  name: string
  description: string | null
  is_system: boolean
  data_scope: string
  permissions: string[]
  created_at: string | null
  updated_at: string | null
}

export interface CreateRoleRequest {
  name: string
  description?: string
  permission_ids: string[]
  data_scope?: string
}

export interface UpdateRoleRequest {
  name?: string
  description?: string
  data_scope?: string
}

export interface UpdateRolePermissionsRequest {
  permission_ids: string[]
}

// ─── Permission Types ────────────────────────────────────────────────────────

export interface Permission {
  id: string
  name: string
  description: string | null
  module: string
  created_at: string | null
}

// ─── Audit Log Types ─────────────────────────────────────────────────────────

export interface AuditLog {
  id: string
  user_id: string | null
  user_email: string | null
  action: string
  resource_type: string | null
  resource_id: string | null
  details: Record<string, unknown> | null
  ip_address: string | null
  user_agent: string | null
  created_at: string
}

export interface AuditLogListParams {
  page?: number
  per_page?: number
  user_id?: string
  action?: string
  from?: string
  to?: string
}

/** Validate password strength on the frontend */
export function validatePasswordStrength(password: string): string | null {
  if (password.length < 8) {
    return "Password must be at least 8 characters"
  }

  const hasUppercase = /[A-Z]/.test(password)
  const hasLowercase = /[a-z]/.test(password)
  const hasDigit = /[0-9]/.test(password)
  const hasSpecial = /[^A-Za-z0-9]/.test(password)

  const categoryCount = [hasUppercase, hasLowercase, hasDigit, hasSpecial].filter(Boolean).length

  if (categoryCount < 3) {
    return "Password must contain at least 3 of: uppercase letter, lowercase letter, digit, special character"
  }

  return null
}
