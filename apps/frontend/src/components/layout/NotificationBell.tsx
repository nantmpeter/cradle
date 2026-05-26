import { useNavigate } from "react-router-dom"
import { useUnreadCount, useNotifications, useMarkNotificationRead } from "@/hooks/useNotifications"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Bell, Check } from "lucide-react"
import { cn } from "@/lib/utils"
import { useTranslation } from "react-i18next"

export function NotificationBell() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const { data: unreadData } = useUnreadCount()
  const { data: notifData } = useNotifications(1, 5)
  const markRead = useMarkNotificationRead()

  const unreadCount = unreadData?.count ?? 0
  const notifications = notifData?.items ?? []

  const handleMarkRead = async (id: string) => {
    try {
      await markRead.mutateAsync(id)
    } catch {
      // ignore
    }
  }

  const typeColor: Record<string, string> = {
    info: "bg-blue-500",
    warning: "bg-yellow-500",
    error: "bg-red-500",
    success: "bg-green-500",
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" className="relative">
          <Bell className="h-4 w-4" />
          {unreadCount > 0 && (
            <Badge className="absolute -top-1 -right-1 h-5 w-5 rounded-full p-0 flex items-center justify-center text-[10px]">
              {unreadCount > 99 ? "99+" : unreadCount}
            </Badge>
          )}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-80">
        <DropdownMenuLabel className="flex items-center justify-between">
          <span>{t("notifications.title")}</span>
          {unreadCount > 0 && (
            <Badge variant="secondary" className="text-xs">{unreadCount} {t("notifications.unread")}</Badge>
          )}
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        {notifications.length === 0 ? (
          <div className="p-4 text-center text-sm text-muted-foreground">{t("notifications.noUnread")}</div>
        ) : (
          notifications.map((n) => (
            <DropdownMenuItem
              key={n.id}
              className="flex items-start gap-2 p-3 cursor-pointer"
              onClick={() => {
                if (!n.read) handleMarkRead(n.id)
                navigate("/dashboard/notifications")
              }}
            >
              <div className={cn("mt-1 h-2 w-2 rounded-full shrink-0", typeColor[n.type] || "bg-gray-400")} />
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium truncate">{n.title}</p>
                <p className="text-xs text-muted-foreground line-clamp-1">{n.content}</p>
              </div>
              {!n.read && (
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-6 w-6 shrink-0"
                  onClick={(e) => { e.stopPropagation(); handleMarkRead(n.id) }}
                >
                  <Check className="h-3 w-3" />
                </Button>
              )}
            </DropdownMenuItem>
          ))
        )}
        <DropdownMenuSeparator />
        <DropdownMenuItem
          className="justify-center text-sm font-medium cursor-pointer"
          onClick={() => navigate("/dashboard/notifications")}
        >
          {t("notifications.viewAll")}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
