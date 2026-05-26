import { useState } from "react"
import { useLoginLogs } from "@/hooks/useLoginLogs"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from "@/components/ui/table"
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select"
import {
  Dialog, DialogContent, DialogHeader, DialogTitle,
} from "@/components/ui/dialog"
import { Pagination } from "@/components/ui/pagination"
import { Search, Eye } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { LoginLog } from "@/types/loginLog"

export function LoginLogListPage() {
  const { t } = useTranslation()
  const [page, setPage] = useState(1)
  const perPage = 20
  const [emailFilter, setEmailFilter] = useState("")
  const [eventFilter, setEventFilter] = useState("")
  const [ipFilter, setIpFilter] = useState("")
  const [detailLog, setDetailLog] = useState<LoginLog | null>(null)

  const { data, isLoading } = useLoginLogs({
    email: emailFilter || undefined,
    event: eventFilter || undefined,
    ip_address: ipFilter || undefined,
    page,
    per_page: perPage,
  })

  const logs = data?.data || []
  const total = (data as any)?.total || 0

  const events = ["login_success", "login_failed", "logout"]

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">{t("loginLogs.title")}</h1>
        <p className="text-muted-foreground">{t("loginLogs.subtitle")}</p>
      </div>

      {/* Filters */}
      <div className="flex flex-wrap items-center gap-4">
        <div className="relative w-60">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder={t("loginLogs.searchEmail")}
            value={emailFilter}
            onChange={(e) => { setEmailFilter(e.target.value); setPage(1) }}
            className="pl-9"
          />
        </div>
        <Select value={eventFilter} onValueChange={(v) => { setEventFilter(v === "all" ? "" : v); setPage(1) }}>
          <SelectTrigger className="w-40">
            <SelectValue placeholder={t("loginLogs.allEvents")} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t("loginLogs.allEvents")}</SelectItem>
            {events.map((e) => (
              <SelectItem key={e} value={e}>{e}</SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Input
          placeholder={t("loginLogs.filterIp")}
          value={ipFilter}
          onChange={(e) => { setIpFilter(e.target.value); setPage(1) }}
          className="w-40"
        />
      </div>

      {/* Table */}
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t("loginLogs.email")}</TableHead>
              <TableHead>{t("loginLogs.event")}</TableHead>
              <TableHead>{t("loginLogs.ip")}</TableHead>
              <TableHead>{t("loginLogs.browser")}</TableHead>
              <TableHead>{t("loginLogs.os")}</TableHead>
              <TableHead>{t("loginLogs.success")}</TableHead>
              <TableHead>{t("loginLogs.loginAt")}</TableHead>
              <TableHead className="w-12">{t("common.details")}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow><TableCell colSpan={8} className="text-center py-8">{t("common.loading")}</TableCell></TableRow>
            ) : logs.length === 0 ? (
              <TableRow><TableCell colSpan={8} className="text-center py-8 text-muted-foreground">{t("common.noData")}</TableCell></TableRow>
            ) : (
              logs.map((log) => (
                <TableRow key={log.id}>
                  <TableCell className="font-medium">{log.email}</TableCell>
                  <TableCell><Badge variant="outline">{log.event}</Badge></TableCell>
                  <TableCell className="text-muted-foreground text-sm">{log.ip_address || "-"}</TableCell>
                  <TableCell className="text-sm">{log.browser || "-"}</TableCell>
                  <TableCell className="text-sm">{log.os || "-"}</TableCell>
                  <TableCell>
                    <Badge variant={log.success ? "default" : "destructive"}>
                      {log.success ? t("common.yes") : t("common.no")}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-sm text-muted-foreground">
                    {log.login_at ? new Date(log.login_at).toLocaleString() : "-"}
                  </TableCell>
                  <TableCell>
                    <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => setDetailLog(log)}>
                      <Eye className="h-3.5 w-3.5" />
                    </Button>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      <Pagination page={page} total={total} perPage={perPage} onPageChange={setPage} />

      {/* Detail Dialog */}
      <Dialog open={!!detailLog} onOpenChange={() => setDetailLog(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("loginLogs.detailTitle")}</DialogTitle>
          </DialogHeader>
          {detailLog && (
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span className="text-muted-foreground">{t("loginLogs.email")}</span>
                <p className="font-medium">{detailLog.email}</p>
              </div>
              <div>
                <span className="text-muted-foreground">{t("loginLogs.event")}</span>
                <p className="font-medium"><Badge variant="outline">{detailLog.event}</Badge></p>
              </div>
              <div>
                <span className="text-muted-foreground">{t("loginLogs.ip")}</span>
                <p className="font-medium">{detailLog.ip_address || "-"}</p>
              </div>
              <div>
                <span className="text-muted-foreground">{t("loginLogs.browser")}</span>
                <p className="font-medium">{detailLog.browser || "-"}</p>
              </div>
              <div>
                <span className="text-muted-foreground">{t("loginLogs.os")}</span>
                <p className="font-medium">{detailLog.os || "-"}</p>
              </div>
              <div>
                <span className="text-muted-foreground">{t("loginLogs.success")}</span>
                <p>
                  <Badge variant={detailLog.success ? "default" : "destructive"}>
                    {detailLog.success ? t("common.yes") : t("common.no")}
                  </Badge>
                </p>
              </div>
              {detailLog.fail_reason && (
                <div className="col-span-2">
                  <span className="text-muted-foreground">{t("loginLogs.failReason")}</span>
                  <p className="font-medium text-destructive">{detailLog.fail_reason}</p>
                </div>
              )}
              <div>
                <span className="text-muted-foreground">{t("loginLogs.userAgent")}</span>
                <p className="font-medium text-xs break-all">{detailLog.user_agent || "-"}</p>
              </div>
              <div>
                <span className="text-muted-foreground">{t("loginLogs.loginAt")}</span>
                <p className="font-medium">{detailLog.login_at ? new Date(detailLog.login_at).toLocaleString() : "-"}</p>
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </div>
  )
}
