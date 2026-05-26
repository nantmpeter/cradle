import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip, Legend } from "recharts"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { useTranslation } from "react-i18next"

interface UserStatusChartProps {
  users: {
    total: number
    active: number
    disabled: number
  } | null
}

/** 图表配色 — 兼容 dark mode */
const COLORS = {
  active: "hsl(var(--chart-2))",
  disabled: "hsl(var(--chart-5))",
}

/**
 * 用户状态饼图
 * 仅当 users 不为 null 时渲染（需要 users:read 权限）
 */
export function UserStatusChart({ users }: UserStatusChartProps) {
  const { t } = useTranslation()
  // 无权限时不渲染
  if (users === null) {
    return null
  }

  const chartData = [
    { name: t("users.active"), value: users.active, color: COLORS.active },
    { name: t("users.disabled"), value: users.disabled, color: COLORS.disabled },
  ]

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">{t("dashboard.userStatus")}</CardTitle>
      </CardHeader>
      <CardContent>
        <ResponsiveContainer width="100%" height={280}>
          <PieChart>
            <Pie
              data={chartData}
              cx="50%"
              cy="50%"
              innerRadius={60}
              outerRadius={100}
              paddingAngle={4}
              dataKey="value"
              nameKey="name"
              label={({ name, percent }) =>
                `${name} ${(percent * 100).toFixed(0)}%`
              }
            >
              {chartData.map((entry, index) => (
                <Cell key={`cell-${index}`} fill={entry.color} />
              ))}
            </Pie>
            <Tooltip
              contentStyle={{
                backgroundColor: "hsl(var(--popover))",
                border: "1px solid hsl(var(--border))",
                borderRadius: "6px",
                color: "hsl(var(--popover-foreground))",
              }}
            />
            <Legend />
          </PieChart>
        </ResponsiveContainer>
      </CardContent>
    </Card>
  )
}
