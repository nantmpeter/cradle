export interface LoginLog {
  id: string
  user_id: string | null
  email: string
  event: string
  ip_address: string | null
  user_agent: string | null
  os: string | null
  browser: string | null
  success: boolean
  fail_reason: string | null
  login_at: string
}

export interface LoginLogListParams {
  email?: string
  event?: string
  ip_address?: string
  start_date?: string
  end_date?: string
  page?: number
  per_page?: number
}
