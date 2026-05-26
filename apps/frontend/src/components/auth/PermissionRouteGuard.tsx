import { Navigate } from "react-router-dom"
import { useAuthStore } from "@/stores/authStore"

interface PermissionRouteGuardProps {
  /** Permission string required to access this route */
  permission: string
  children: React.ReactNode
}

/**
 * Route-level permission guard.
 * If the user lacks the required permission, redirects to /dashboard/forbidden (403 page).
 * superadmin always passes.
 */
export function PermissionRouteGuard({ permission, children }: PermissionRouteGuardProps) {
  const hasPermission = useAuthStore((s) => s.hasPermission)
  const user = useAuthStore((s) => s.user)

  if (!user) {
    return <Navigate to="/login" replace />
  }

  if (!hasPermission(permission)) {
    return <Navigate to="/dashboard/forbidden" replace />
  }

  return <>{children}</>
}
