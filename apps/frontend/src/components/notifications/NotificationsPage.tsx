import { useState } from "react"
import { useNotifications, useMarkNotificationRead, useMarkAllRead, useDeleteNotification, useBroadcastNotification } from "@/hooks/useNotifications"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent } from "@/components/ui/card"
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Pagination } from "@/components/ui/pagination"
import { PermissionGuard } from "@/components/shared/PermissionGuard"
import { Bell, CheckCheck, Megaphone, Trash2, ExternalLink } from "lucide-react"
import { toast } from "sonner"
import { useTranslation } from "react-i18next"

const typeColors: Record<string, string> = {
  info: "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
  warning: "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200",
  error: "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
  success: "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
}

export function NotificationsPage() {
  const { t } = useTranslation()
  const [page, setPage] = useState(1)
  const limit = 20

  const { data, isLoading } = useNotifications(page, limit)
  const markRead = useMarkNotificationRead()
  const markAllRead = useMarkAllRead()
  const deleteNotif = useDeleteNotification()
  const broadcast = useBroadcastNotification()

  const [broadcastOpen, setBroadcastOpen] = useState(false)
  const [bTitle, setBTitle] = useState("")
  const [bContent, setBContent] = useState("")
  const [bType, setBType] = useState<"info" | "warning" | "error" | "success">("info")

  const notifications = data?.items ?? []
  const total = data?.total ?? 0
  const totalPages = Math.ceil(total / limit)

  const handleMarkAllRead = async () => {
    try {
      await markAllRead.mutateAsync()
      toast.success(t("notifications.markedAllRead"))
    } catch {
      toast.error(t("notifications.markAllReadFailed"))
    }
  }

  const handleDelete = async (id: string) => {
    try {
      await deleteNotif.mutateAsync(id)
      toast.success(t("notifications.deleted"))
    } catch {
      toast.error(t("notifications.deleteFailed"))
    }
  }

  const handleBroadcast = async () => {
    try {
      await broadcast.mutateAsync({ title: bTitle, content: bContent, type: bType })
      toast.success(t("notifications.broadcastSent"))
      setBroadcastOpen(false)
      setBTitle("")
      setBContent("")
      setBType("info")
    } catch (err: any) {
      toast.error(err.response?.data?.error || t("notifications.broadcastFailed"))
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold flex items-center gap-2">
            <Bell className="h-6 w-6" /> {t("notifications.title")}
          </h1>
          <p className="text-muted-foreground">{t("notifications.subtitle")}</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={handleMarkAllRead} disabled={markAllRead.isPending}>
            <CheckCheck className="mr-2 h-4 w-4" /> {t("notifications.markAllRead")}
          </Button>
          <PermissionGuard permission="notifications:send">
            <Dialog open={broadcastOpen} onOpenChange={setBroadcastOpen}>
              <DialogTrigger asChild>
                <Button size="sm">
                  <Megaphone className="mr-2 h-4 w-4" /> {t("notifications.broadcast")}
                </Button>
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>{t("notifications.sendBroadcast")}</DialogTitle>
                  <DialogDescription>{t("notifications.sendBroadcastDesc")}</DialogDescription>
                </DialogHeader>
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label>{t("notifications.broadcastTitle")}</Label>
                    <Input value={bTitle} onChange={(e) => setBTitle(e.target.value)} placeholder={t("notifications.broadcastTitle")} />
                  </div>
                  <div className="space-y-2">
                    <Label>{t("notifications.broadcastContent")}</Label>
                    <textarea
                      className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                      value={bContent}
                      onChange={(e) => setBContent(e.target.value)}
                      placeholder={t("notifications.broadcastContent")}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>{t("notifications.broadcastType")}</Label>
                    <Select value={bType} onValueChange={(v: any) => setBType(v)}>
                      <SelectTrigger><SelectValue /></SelectTrigger>
                      <SelectContent>
                        <SelectItem value="info">{t("notifications.type.info")}</SelectItem>
                        <SelectItem value="warning">{t("notifications.type.warning")}</SelectItem>
                        <SelectItem value="error">{t("notifications.type.error")}</SelectItem>
                        <SelectItem value="success">{t("notifications.type.success")}</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>
                <DialogFooter>
                  <Button variant="outline" onClick={() => setBroadcastOpen(false)}>{t("common.cancel")}</Button>
                  <Button onClick={handleBroadcast} disabled={!bTitle || !bContent || broadcast.isPending}>
                    {broadcast.isPending ? t("notifications.sending") : t("notifications.send")}
                  </Button>
                </DialogFooter>
              </DialogContent>
            </Dialog>
          </PermissionGuard>
        </div>
      </div>

      <Card>
        <CardContent className="p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-12">{t("notifications.type.label")}</TableHead>
                <TableHead>{t("notifications.broadcastTitle")}</TableHead>
                <TableHead>{t("notifications.broadcastContent")}</TableHead>
                <TableHead className="w-24">{t("common.status")}</TableHead>
                <TableHead className="w-40">{t("auditLogs.time")}</TableHead>
                <TableHead className="w-24">{t("common.actions")}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow><TableCell colSpan={6} className="text-center py-8 text-muted-foreground">{t("common.loading")}</TableCell></TableRow>
              ) : notifications.length === 0 ? (
                <TableRow><TableCell colSpan={6} className="text-center py-8 text-muted-foreground">{t("notifications.noNotifications")}</TableCell></TableRow>
              ) : (
                notifications.map((n) => (
                  <TableRow key={n.id} className={!n.read ? "bg-muted/50" : ""}>
                    <TableCell>
                      <Badge variant="secondary" className={typeColors[n.type] || ""}>{n.type}</Badge>
                    </TableCell>
                    <TableCell className="font-medium">{n.title}</TableCell>
                    <TableCell className="text-muted-foreground max-w-xs truncate">{n.content}</TableCell>
                    <TableCell>
                      {n.read ? (
                        <Badge variant="outline">{t("notifications.read")}</Badge>
                      ) : (
                        <Badge>{t("notifications.unread")}</Badge>
                      )}
                    </TableCell>
                    <TableCell className="text-sm text-muted-foreground">
                      {new Date(n.created_at).toLocaleString()}
                    </TableCell>
                    <TableCell>
                      <div className="flex gap-1">
                        {!n.read && (
                          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => markRead.mutate(n.id)}>
                            <ExternalLink className="h-3 w-3" />
                          </Button>
                        )}
                        <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive" onClick={() => handleDelete(n.id)}>
                          <Trash2 className="h-3 w-3" />
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      {totalPages > 1 && (
        <Pagination
          currentPage={page}
          totalPages={totalPages}
          onPageChange={setPage}
        />
      )}
    </div>
  )
}
