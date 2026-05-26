export interface Notification {
  id: string
  user_id: string | null
  title: string
  content: string
  type: "info" | "warning" | "error" | "success"
  created_at: string
  read?: boolean
}

export interface NotificationListResponse {
  items: Notification[]
  total: number
  page: number
  limit: number
}

export interface BroadcastNotificationRequest {
  title: string
  content: string
  type: "info" | "warning" | "error" | "success"
}
