import { useEffect, useRef, useCallback } from "react"
import { useQueryClient } from "@tanstack/react-query"
import { useAuthStore } from "@/stores/authStore"

interface SseNotification {
  id: string
  user_id: string | null
  title: string
  content: string
  category: string
}

export function useSseNotifications() {
  const token = useAuthStore((s) => s.token)
  const queryClient = useQueryClient()
  const eventSourceRef = useRef<EventSource | null>(null)
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const connect = useCallback(() => {
    if (!token) return

    // Close existing connection
    if (eventSourceRef.current) {
      eventSourceRef.current.close()
    }

    const url = new URL("/api/sse/notifications", window.location.origin)
    // EventSource doesn't support custom headers, so we pass token as query param
    // The backend should accept both header and query param for SSE
    url.searchParams.set("token", token)

    const es = new EventSource(url.toString())

    es.onmessage = (event) => {
      try {
        const notif: SseNotification = JSON.parse(event.data)
        // Invalidate notification queries to trigger refetch
        queryClient.invalidateQueries({ queryKey: ["notifications"] })
        queryClient.invalidateQueries({ queryKey: ["unread-count"] })

        // Show browser notification if permitted
        if (Notification.permission === "granted") {
          new Notification(notif.title, { body: notif.content })
        }
      } catch {
        // ignore parse errors (e.g. keepalive pings)
      }
    }

    es.onerror = () => {
      es.close()
      eventSourceRef.current = null
      // Reconnect after 5 seconds
      reconnectTimeoutRef.current = setTimeout(() => {
        connect()
      }, 5000)
    }

    eventSourceRef.current = es
  }, [token, queryClient])

  useEffect(() => {
    connect()

    return () => {
      if (eventSourceRef.current) {
        eventSourceRef.current.close()
        eventSourceRef.current = null
      }
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current)
      }
    }
  }, [connect])
}
