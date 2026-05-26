import { useNavigate } from "react-router-dom"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { ArrowRight } from "lucide-react"
import type { DashboardStats } from "@/types/dashboard"
import { useTranslation } from "react-i18next"

interface RecentAuditLogsProps {
  logs: DashboardStats["recent_audit_logs"]
}

/** 操作类型对应的 Badge 颜色 */
const actionColorMap: Record<string, string> = {
  auth: "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
  user: "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
  role: "bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200",
  permission: "bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200",
  dashboard: "bg-cyan-100 text-cyan-800 dark:bg-cyan-900 dark:text-cyan-200",
}

/** 根据 action 前缀获取 Badge 样式 */
function getActionColor(action: string): string {
  const prefix = action.split(".")[0] || ""
  return actionColorMap[prefix] || "bg-secondary text-secondary-foreground"
}

/**
 * 最近审计日志表格
 * 展示最近 5 条审计日志，底部提供"查看全部"链接
 */
export function RecentAuditLogs({ logs }: RecentAuditLogsProps) {
  const navigate = useNavigate()
  const { t } = useTranslation()

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">{t("dashboard.recentActivity")}</CardTitle>
      </CardHeader>
      <CardContent>
        {logs.length === 0 ? (
          <p className="py-8 text-center text-sm text-muted-foreground">
            {t("common.noData")}
          </p>
        ) : (
          <>
            <div className="rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>{t("auditLogs.time")}</TableHead>
                    <TableHead>{t("auditLogs.action")}</TableHead>
                    <TableHead>{t("auditLogs.user")}</TableHead>
                    <TableHead>{t("auditLogs.resource")}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {logs.map((log) => (
                    <TableRow key={log.id}>
                      <TableCell className="text-sm text-muted-foreground whitespace-nowrap">
                        {new Date(log.created_at).toLocaleString()}
                      </TableCell>
                      <TableCell>
                        <Badge variant="outline" className={getActionColor(log.action)}>
                          {log.action}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-sm">
                        {log.user_email || (
                          <span className="text-muted-foreground">{t("auditLogs.system")}</span>
                        )}
                      </TableCell>
                      <TableCell className="text-sm text-muted-foreground">
                        {log.resource_type || "-"}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
            <div className="mt-3 flex justify-end">
              <Button
                variant="ghost"
                size="sm"
                className="text-sm text-muted-foreground"
                onClick={() => navigate("/dashboard/audit-logs")}
              >
                {t("dashboard.viewAll")}
                <ArrowRight className="ml-1 h-4 w-4" />
              </Button>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  )
}
