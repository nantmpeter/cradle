import { Badge } from "@/components/ui/badge"

/** Module-based color mapping for permission badges */
const moduleColors: Record<string, string> = {
  users: "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
  roles: "bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200",
  audit: "bg-amber-100 text-amber-800 dark:bg-amber-900 dark:text-amber-200",
  system: "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
}

const defaultColor = "bg-secondary text-secondary-foreground hover:bg-secondary/80"

/** Extract the module prefix from a permission string (e.g. "users:create" → "users") */
function getModule(permissionName: string): string {
  const parts = permissionName.split(":")
  return parts[0] || "system"
}

interface PermissionBadgeProps {
  permission: string
}

export function PermissionBadge({ permission }: PermissionBadgeProps) {
  const module = getModule(permission)
  const colorClass = moduleColors[module] || defaultColor

  return (
    <Badge variant="outline" className={colorClass}>
      {permission}
    </Badge>
  )
}
