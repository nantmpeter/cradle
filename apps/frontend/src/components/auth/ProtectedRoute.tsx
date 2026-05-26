import { Navigate } from "react-router-dom"
import { useAuthStore } from "@/stores/authStore"

export function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated)
  const mustChangePassword = useAuthStore((s) => s.mustChangePassword)

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />
  }

  // If user must change password, redirect to profile change password section
  if (mustChangePassword) {
    return <Navigate to="/dashboard/profile" replace />
  }

  return <>{children}</>
}
