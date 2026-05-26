import { useAuditLogDetail } from "@/hooks/useAuditLogs"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Badge } from "@/components/ui/badge"
import { Separator } from "@/components/ui/separator"
import { useTranslation } from "react-i18next"

interface AuditLogDetailDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  logId: string | null
}

export function AuditLogDetailDialog({
  open,
  onOpenChange,
  logId,
}: AuditLogDetailDialogProps) {
  const { t } = useTranslation()
  const { data: log, isLoading } = useAuditLogDetail(logId || "")

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>{t("auditLogs.detailTitle")}</DialogTitle>
          <DialogDescription>
            {log?.created_at
              ? new Date(log.created_at).toLocaleString()
              : t("common.loading")}
          </DialogDescription>
        </DialogHeader>

        {isLoading ? (
          <div className="py-8 text-center text-muted-foreground">
            {t("common.loading")}
          </div>
        ) : log ? (
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-3 text-sm">
              <div>
                <p className="font-medium text-muted-foreground">{t("auditLogs.action")}</p>
                <Badge variant="outline">{log.action}</Badge>
              </div>
              <div>
                <p className="font-medium text-muted-foreground">{t("auditLogs.user")}</p>
                <p>{log.user_email || t("auditLogs.system")}</p>
              </div>
              <div>
                <p className="font-medium text-muted-foreground">{t("auditLogs.resourceType")}</p>
                <p>{log.resource_type || "-"}</p>
              </div>
              <div>
                <p className="font-medium text-muted-foreground">{t("auditLogs.resourceId")}</p>
                <p className="font-mono text-xs">{log.resource_id || "-"}</p>
              </div>
              <div>
                <p className="font-medium text-muted-foreground">{t("auditLogs.ipAddress")}</p>
                <p>{log.ip_address || "-"}</p>
              </div>
              <div>
                <p className="font-medium text-muted-foreground">{t("auditLogs.userAgent")}</p>
                <p className="truncate text-xs" title={log.user_agent || undefined}>
                  {log.user_agent || "-"}
                </p>
              </div>
            </div>

            {log.details && (
              <>
                <Separator />
                <div>
                  <p className="font-medium text-muted-foreground mb-2">{t("auditLogs.details")}</p>
                  <pre className="rounded-md bg-muted p-3 text-xs overflow-x-auto">
                    {JSON.stringify(log.details, null, 2)}
                  </pre>
                </div>
              </>
            )}
          </div>
        ) : (
          <div className="py-8 text-center text-muted-foreground">
            {t("auditLogs.notFound")}
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
