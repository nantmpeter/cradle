import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { api } from "@/lib/api"
import type { NotificationListResponse, BroadcastNotificationRequest } from "@/types/notification"

const notificationKeys = {
  all: ["notifications"] as const,
  list: (page: number, limit: number) => ["notifications", "list", page, limit] as const,
  unreadCount: ["notifications", "unread-count"] as const,
}

export function useNotifications(page = 1, limit = 20) {
  return useQuery({
    queryKey: notificationKeys.list(page, limit),
    queryFn: async (): Promise<NotificationListResponse> => {
      const res = await api.get("/notifications", { params: { page, limit } })
      return res.data
    },
  })
}

export function useUnreadCount() {
  return useQuery({
    queryKey: notificationKeys.unreadCount,
    queryFn: async (): Promise<{ count: number }> => {
      const res = await api.get("/notifications/unread-count")
      return res.data
    },
    refetchInterval: 30000,
  })
}

export function useMarkNotificationRead() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => {
      const res = await api.post(`/notifications/${id}/read`)
      return res.data
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: notificationKeys.all })
    },
  })
}

export function useMarkAllRead() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async () => {
      const res = await api.post("/notifications/read-all")
      return res.data
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: notificationKeys.all })
    },
  })
}

export function useDeleteNotification() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => {
      const res = await api.delete(`/notifications/${id}`)
      return res.data
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: notificationKeys.all })
    },
  })
}

export function useBroadcastNotification() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (data: BroadcastNotificationRequest) => {
      const res = await api.post("/notifications/broadcast", data)
      return res.data
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: notificationKeys.all })
    },
  })
}
