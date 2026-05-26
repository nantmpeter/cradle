import { Badge } from "@/components/ui/badge"
import type { UserStatus } from "@/types/api"

const statusConfig: Record<UserStatus, { label: string; className: string }> = {
  active: {
    label: "Active",
    className: "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
  },
  disabled: {
    label: "Disabled",
    className: "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
  },
}

interface StatusBadgeProps {
  status: UserStatus
}

export function StatusBadge({ status }: StatusBadgeProps) {
  const config = statusConfig[status] || statusConfig.active
  return (
    <Badge variant="outline" className={config.className}>
      {config.label}
    </Badge>
  )
}
