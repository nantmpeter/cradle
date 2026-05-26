import { useDashboardStats } from "@/hooks/useDashboard"
import { StatsCards } from "./StatsCards"
import { RoleDistributionChart } from "./RoleDistributionChart"
import { UserStatusChart } from "./UserStatusChart"
import { RecentAuditLogs } from "./RecentAuditLogs"
import { Card, CardContent } from "@/components/ui/card"
import { AlertCircle } from "lucide-react"
import { useTranslation } from "react-i18next"

export function DashboardPage() {
  const { data, isLoading, error } = useDashboardStats()
  const { t } = useTranslation()

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t("dashboard.title")}</h1>
          <p className="text-muted-foreground">{t("dashboard.description")}</p>
        </div>
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          {[1, 2, 3, 4].map((i) => (
            <Card key={i}>
              <CardContent className="p-6">
                <div className="h-20 animate-pulse rounded bg-muted" />
              </CardContent>
            </Card>
          ))}
        </div>
        <div className="grid gap-4 md:grid-cols-2">
          {[1, 2].map((i) => (
            <Card key={i}>
              <CardContent className="p-6">
                <div className="h-64 animate-pulse rounded bg-muted" />
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t("dashboard.title")}</h1>
          <p className="text-muted-foreground">{t("dashboard.description")}</p>
        </div>
        <Card>
          <CardContent className="flex items-center gap-3 p-6">
            <AlertCircle className="h-5 w-5 text-destructive" />
            <p className="text-destructive">
              {error instanceof Error ? error.message : t("common.error")}
            </p>
          </CardContent>
        </Card>
      </div>
    )
  }

  if (!data) return null

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">{t("dashboard.title")}</h1>
        <p className="text-muted-foreground">{t("dashboard.description")}</p>
      </div>
      <StatsCards data={data} />
      <div className="grid gap-4 md:grid-cols-2">
        <RoleDistributionChart distribution={data.roles.distribution} />
        <UserStatusChart users={data.users} />
      </div>
      <RecentAuditLogs logs={data.recent_audit_logs} />
    </div>
  )
}
