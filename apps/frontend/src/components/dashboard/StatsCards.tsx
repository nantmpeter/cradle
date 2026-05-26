import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Users, UserCheck, Shield, LogIn } from "lucide-react"
import type { DashboardStats } from "@/types/dashboard"
import { useTranslation } from "react-i18next"

interface StatsCardsProps {
  data: DashboardStats
}

interface StatCard {
  title: string
  value: number | null
  icon: React.ComponentType<{ className?: string }>
  description: string
}

export function StatsCards({ data }: StatsCardsProps) {
  const { t } = useTranslation()

  const cards: StatCard[] = [
    ...(data.users !== null
      ? [
          {
            title: t("dashboard.totalUsers"),
            value: data.users.total,
            icon: Users,
            description: t("dashboard.totalUsers"),
          },
          {
            title: t("dashboard.activeUsers"),
            value: data.users.active,
            icon: UserCheck,
            description: t("dashboard.activeUsers"),
          },
        ]
      : []),
    {
      title: t("dashboard.totalRoles"),
      value: data.roles.total,
      icon: Shield,
      description: t("dashboard.totalRoles"),
    },
    ...(data.today_logins !== null
      ? [
          {
            title: t("dashboard.activeSessions"),
            value: data.today_logins,
            icon: LogIn,
            description: t("dashboard.activeSessions"),
          },
        ]
      : []),
  ]

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      {cards.map((card, idx) => {
        const Icon = card.icon
        return (
          <Card key={idx}>
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-sm font-medium">{card.title}</CardTitle>
              <Icon className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{card.value}</div>
              <p className="text-xs text-muted-foreground">{card.description}</p>
            </CardContent>
          </Card>
        )
      })}
    </div>
  )
}
