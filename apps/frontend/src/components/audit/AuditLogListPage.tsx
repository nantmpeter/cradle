import { useState } from "react"
import { useAuditLogs } from "@/hooks/useAuditLogs"
import { AuditLogFilterBar } from "./AuditLogFilterBar"
import { AuditLogDetailDialog } from "./AuditLogDetailDialog"
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
import { Pagination } from "@/components/ui/pagination"
import { Badge } from "@/components/ui/badge"
import { Eye, Download } from "lucide-react"
import { toast } from "sonner"
import { api } from "@/lib/api"
import { useTranslation } from "react-i18next"
import type { AuditLog, AuditLogListParams } from "@/types/api"

/** Color mapping for audit log action groups */
const actionColors: Record<string, string> = {
  auth: "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
  user: "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
  role: "bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200",
}

function getActionColor(action: string): string {
  const prefix = action.split(".")[0] || ""
  return actionColors[prefix] || "bg-secondary text-secondary-foreground"
}

export function AuditLogListPage() {
  const { t } = useTranslation()
  const [filters, setFilters] = useState<AuditLogListParams>({
    page: 1,
    per_page: 20,
  })
  const [selectedLogId, setSelectedLogId] = useState<string | null>(null)
  const [detailOpen, setDetailOpen] = useState(false)

  const { data, isLoading } = useAuditLogs(filters)

  const logs = data?.data || []
  const total = data?.pagination.total || 0
  const page = filters.page || 1
  const perPage = filters.per_page || 20

  const handleFilterChange = (newFilters: Omit<AuditLogListParams, "page" | "per_page">) => {
    setFilters((prev) => ({
      ...prev,
      ...newFilters,
      page: 1,
    }))
  }

  const handlePageChange = (newPage: number) => {
    setFilters((prev) => ({ ...prev, page: newPage }))
  }

  const openDetail = (log: AuditLog) => {
    setSelectedLogId(log.id)
    setDetailOpen(true)
  }

  const handleExport = async (format: "csv" | "xlsx") => {
    try {
      const params = new URLSearchParams()
      if (filters.action) params.set("action", filters.action)
      if (filters.user_id) params.set("user_id", filters.user_id)
      if (filters.start_date) params.set("start_date", filters.start_date)
      if (filters.end_date) params.set("end_date", filters.end_date)
      params.set("format", format)
      const res = await api.get(`/audit-logs/export?${params.toString()}`, {
        responseType: "blob",
      })
      const blob = new Blob([res.data])
      const url = window.URL.createObjectURL(blob)
      const a = document.createElement("a")
      a.href = url
      a.download = `audit-logs.${format === "csv" ? "csv" : "xlsx"}`
      document.body.appendChild(a)
      a.click()
      a.remove()
      window.URL.revokeObjectURL(url)
      toast.success(t("auditLogs.exportSuccess", { format: format.toUpperCase() }))
    } catch (err: any) {
      toast.error(err.response?.data?.error || t("auditLogs.exportFailed"))
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t("auditLogs.title")}</h1>
          <p className="text-muted-foreground">{t("auditLogs.subtitle")}</p>
        </div>
        <PermissionGuard permission="audit:export">
          <div className="flex gap-2">
            <Button variant="outline" size="sm" onClick={() => handleExport("csv")}>
              <Download className="mr-2 h-4 w-4" />
              {t("common.exportCsv")}
            </Button>
            <Button variant="outline" size="sm" onClick={() => handleExport("xlsx")}>
              <Download className="mr-2 h-4 w-4" />
              {t("common.exportXlsx")}
            </Button>
          </div>
        </PermissionGuard>
      </div>

      <AuditLogFilterBar onFilterChange={handleFilterChange} />

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t("auditLogs.time")}</TableHead>
              <TableHead>{t("auditLogs.user")}</TableHead>
              <TableHead>{t("auditLogs.action")}</TableHead>
              <TableHead>{t("auditLogs.resource")}</TableHead>
              <TableHead>{t("auditLogs.ipAddress")}</TableHead>
              <TableHead className="w-12">{t("auditLogs.details")}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={6} className="text-center text-muted-foreground py-8">
                  {t("common.loading")}
                </TableCell>
              </TableRow>
            ) : logs.length === 0 ? (
              <TableRow>
                <TableCell colSpan={6} className="text-center text-muted-foreground py-8">
                  {t("common.noData")}
                </TableCell>
              </TableRow>
            ) : (
              logs.map((log) => (
                <TableRow key={log.id}>
                  <TableCell className="text-sm text-muted-foreground whitespace-nowrap">
                    {new Date(log.created_at).toLocaleString()}
                  </TableCell>
                  <TableCell className="text-sm">
                    {log.user_email || (
                      <span className="text-muted-foreground">{t("auditLogs.system")}</span>
                    )}
                  </TableCell>
                  <TableCell>
                    <Badge variant="outline" className={getActionColor(log.action)}>
                      {log.action}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-sm">
                    {log.resource_type ? (
                      <span>
                        {log.resource_type}
                        {log.resource_id && (
                          <span className="text-muted-foreground ml-1 text-xs">
                            ({log.resource_id.slice(0, 8)}...)
                          </span>
                        )}
                      </span>
                    ) : (
                      <span className="text-muted-foreground">-</span>
                    )}
                  </TableCell>
                  <TableCell className="text-sm text-muted-foreground">
                    {log.ip_address || "-"}
                  </TableCell>
                  <TableCell>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8"
                      onClick={() => openDetail(log)}
                    >
                      <Eye className="h-4 w-4" />
                    </Button>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      <Pagination page={page} total={total} perPage={perPage} onPageChange={handlePageChange} />

      <AuditLogDetailDialog
        open={detailOpen}
        onOpenChange={setDetailOpen}
        logId={selectedLogId}
      />
    </div>
  )
}
