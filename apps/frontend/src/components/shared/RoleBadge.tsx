import { Badge } from "@/components/ui/badge"
import type { Role } from "@/types/api"

const roleConfig: Record<Role, { label: string; className: string }> = {
  user: {
    label: "User",
    className: "bg-secondary text-secondary-foreground hover:bg-secondary/80",
  },
  admin: {
    label: "Admin",
    className: "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
  },
  superadmin: {
    label: "SuperAdmin",
    className: "bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200",
  },
}

interface RoleBadgeProps {
  role: Role
}

export function RoleBadge({ role }: RoleBadgeProps) {
  const config = roleConfig[role] || roleConfig.user
  return (
    <Badge variant="outline" className={config.className}>
      {config.label}
    </Badge>
  )
}
