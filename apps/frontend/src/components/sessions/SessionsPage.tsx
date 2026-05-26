import { useState } from "react"
import { useTranslation } from "react-i18next"
import { useSessions, useMySessions, useTerminateSession, useForceLogoutUser } from "@/hooks/useSessions"
import { PermissionGuard } from "@/components/shared/PermissionGuard"
import { Button } from "@/components/ui/button"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Trash2, Monitor, User, MoreHorizontal, LogOut } from "lucide-react"
import { toast } from "sonner"
import { cn } from "@/lib/utils"
import type { Session } from "@/types/session"

/** Truncate a string to maxLen characters */
function truncate(str: string | null, maxLen: number): string {
  if (!str) return "-"
  return str.length > maxLen ? str.slice(0, maxLen) + "..." : str
}

export function SessionsPage() {
  const { t } = useTranslation()

  const [activeTab, setActiveTab] = useState<"all" | "me">("all")

  const allSessions = useSessions()
  const mySessions = useMySessions()
  const terminateSession = useTerminateSession()
  const forceLogoutUser = useForceLogoutUser()

  // Delete dialog state
  const [terminateOpen, setTerminateOpen] = useState(false)
  const [selectedSession, setSelectedSession] = useState<Session | null>(null)

  // Force logout dialog state
  const [forceLogoutOpen, setForceLogoutOpen] = useState(false)
  const [forceLogoutUserId, setForceLogoutUserId] = useState<string>("")
  const [forceLogoutSessionCount, setForceLogoutSessionCount] = useState(0)

  const handleTerminate = async () => {
    if (!selectedSession) return
    try {
      await terminateSession.mutateAsync(selectedSession.id)
      toast.success(t("common.success"))
      setTerminateOpen(false)
      setSelectedSession(null)
    } catch (err: any) {
      toast.error(err.response?.data?.error || t("common.error"))
    }
  }

  const openTerminateDialog = (session: Session) => {
    setSelectedSession(session)
    setTerminateOpen(true)
  }

  const openForceLogoutDialog = (session: Session) => {
    const userSessions = sessions.filter(s => s.user_id === session.user_id)
    setForceLogoutUserId(session.user_id)
    setForceLogoutSessionCount(userSessions.length)
    setForceLogoutOpen(true)
  }

  const handleForceLogout = async () => {
    try {
      await forceLogoutUser.mutateAsync(forceLogoutUserId)
      toast.success(t("sessions.forceLogoutSuccess", { count: forceLogoutSessionCount }))
      setForceLogoutOpen(false)
    } catch (err: any) {
      toast.error(err.response?.data?.error || t("common.error"))
    }
  }

  const sessions = activeTab === "all" ? (allSessions.data?.data || []) : (mySessions.data?.data || [])
  const isLoading = activeTab === "all" ? allSessions.isLoading : mySessions.isLoading

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t("sessions.title")}</h1>
          <p className="text-muted-foreground">{t("sessions.myDevices")}</p>
        </div>
      </div>

      {/* Tab Switch */}
      <div className="flex gap-2">
        <Button
          variant={activeTab === "all" ? "default" : "outline"}
          size="sm"
          onClick={() => setActiveTab("all")}
        >
          <Monitor className="mr-2 h-4 w-4" />
          All Sessions
        </Button>
        <Button
          variant={activeTab === "me" ? "default" : "outline"}
          size="sm"
          onClick={() => setActiveTab("me")}
        >
          <User className="mr-2 h-4 w-4" />
          {t("sessions.myDevices")}
        </Button>
      </div>

      {/* Sessions Table */}
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t("sessions.ipAddress")}</TableHead>
              <TableHead>{t("sessions.userAgent")}</TableHead>
              <TableHead>{t("sessions.lastActive")}</TableHead>
              <TableHead>{t("sessions.expiresAt")}</TableHead>
              {activeTab === "all" && (
                <PermissionGuard permission="sessions:manage">
                  <TableHead className="w-12">{t("common.actions")}</TableHead>
                </PermissionGuard>
              )}
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={5} className="text-center text-muted-foreground py-8">
                  {t("common.loading")}
                </TableCell>
              </TableRow>
            ) : sessions.length === 0 ? (
              <TableRow>
                <TableCell colSpan={5} className="text-center text-muted-foreground py-8">
                  {t("common.noData")}
                </TableCell>
              </TableRow>
            ) : (
              sessions.map((session) => (
                <TableRow key={session.id}>
                  <TableCell className="font-medium font-mono text-sm">
                    {session.ip_address || "-"}
                  </TableCell>
                  <TableCell className="text-muted-foreground text-sm max-w-xs">
                    <span title={session.user_agent || undefined}>
                      {truncate(session.user_agent, 50)}
                    </span>
                  </TableCell>
                  <TableCell className="text-muted-foreground text-sm">
                    {session.last_active_at
                      ? new Date(session.last_active_at).toLocaleString()
                      : "-"}
                  </TableCell>
                  <TableCell className="text-muted-foreground text-sm">
                    {session.expires_at
                      ? new Date(session.expires_at).toLocaleString()
                      : "-"}
                  </TableCell>
                  {activeTab === "all" && (
                    <PermissionGuard permission="sessions:manage">
                      <TableCell>
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button variant="ghost" size="icon" className="h-8 w-8">
                              <MoreHorizontal className="h-4 w-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuItem onClick={() => openTerminateDialog(session)}>
                              <Trash2 className="mr-2 h-4 w-4" />
                              {t("sessions.terminate")}
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem
                              className="text-destructive focus:text-destructive"
                              onClick={() => openForceLogoutDialog(session)}
                            >
                              <LogOut className="mr-2 h-4 w-4" />
                              {t("sessions.forceLogout")}
                            </DropdownMenuItem>
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </TableCell>
                    </PermissionGuard>
                  )}
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      {/* Terminate Session Dialog */}
      <Dialog open={terminateOpen} onOpenChange={setTerminateOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("sessions.terminate")}</DialogTitle>
            <DialogDescription>{t("sessions.terminateConfirm")}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setTerminateOpen(false)}>
              {t("common.cancel")}
            </Button>
            <Button variant="destructive" onClick={handleTerminate} disabled={terminateSession.isPending}>
              {terminateSession.isPending ? t("common.loading") : t("sessions.terminate")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Force Logout User Dialog */}
      <Dialog open={forceLogoutOpen} onOpenChange={setForceLogoutOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("sessions.forceLogout")}</DialogTitle>
            <DialogDescription>
              {t("sessions.forceLogoutConfirm", { count: forceLogoutSessionCount })}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setForceLogoutOpen(false)}>
              {t("common.cancel")}
            </Button>
            <Button variant="destructive" onClick={handleForceLogout} disabled={forceLogoutUser.isPending}>
              {forceLogoutUser.isPending ? t("common.loading") : t("sessions.forceLogout")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
