import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { useTranslation } from "react-i18next"

interface RoleDistributionChartProps {
  distribution: { role_name: string; count: number }[] | null
}

/**
 * 角色分布条形图
 * 仅当 distribution 不为 null 时渲染（需要 roles:read 权限）
 */
export function RoleDistributionChart({ distribution }: RoleDistributionChartProps) {
  const { t } = useTranslation()
  // 无权限时不渲染
  if (distribution === null || distribution.length === 0) {
    return null
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">{t("dashboard.roleDistribution")}</CardTitle>
      </CardHeader>
      <CardContent>
        <ResponsiveContainer width="100%" height={280}>
          <BarChart
            data={distribution}
            margin={{ top: 5, right: 20, left: 0, bottom: 5 }}
          >
            <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
            <XAxis
              dataKey="role_name"
              tick={{ fontSize: 12 }}
              className="text-muted-foreground"
            />
            <YAxis
              allowDecimals={false}
              tick={{ fontSize: 12 }}
              className="text-muted-foreground"
            />
            <Tooltip
              contentStyle={{
                backgroundColor: "hsl(var(--popover))",
                border: "1px solid hsl(var(--border))",
                borderRadius: "6px",
                color: "hsl(var(--popover-foreground))",
              }}
            />
            <Bar
              dataKey="count"
              name={t("dashboard.users")}
              fill="hsl(var(--chart-1))"
              radius={[4, 4, 0, 0]}
            />
          </BarChart>
        </ResponsiveContainer>
      </CardContent>
    </Card>
  )
}
