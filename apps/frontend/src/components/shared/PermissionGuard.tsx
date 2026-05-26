import { useAuthStore } from "@/stores/authStore"

interface PermissionGuardProps {
  /**
   * Permission string or array of permission strings to check.
   * If an array is given, the user must have ALL permissions.
   * superadmin always passes.
   */
  permission?: string | string[]
  children: React.ReactNode
  /** Optional fallback to render when permission is denied */
  fallback?: React.ReactNode
}

/**
 * Conditionally renders children based on the current user's permissions.
 * superadmin always passes regardless of the permission prop.
 */
export function PermissionGuard({
  permission,
  children,
  fallback = null,
}: PermissionGuardProps) {
  const hasPermission = useAuthStore((s) => s.hasPermission)
  const user = useAuthStore((s) => s.user)

  // Not authenticated → no permission
  if (!user) return <>{fallback}</>

  // No permission specified → just check authentication
  if (!permission) return <>{children}</>

  // Check permission(s)
  const permissions = Array.isArray(permission) ? permission : [permission]
  const hasAll = permissions.every((p) => hasPermission(p))

  return hasAll ? <>{children}</> : <>{fallback}</>
}
