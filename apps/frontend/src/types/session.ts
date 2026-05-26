export interface Session {
  id: string
  user_id: string
  ip_address: string | null
  user_agent: string | null
  last_active_at: string | null
  created_at: string | null
  expires_at: string
}

export interface SessionListResponse {
  data: Session[]
}
