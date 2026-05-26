import { useState } from "react"
import { useNavigate, useLocation } from "react-router-dom"
import { LayoutDashboard, Users, LogOut, ChevronLeft, ChevronRight, UserCircle, Shield, ScrollText, Settings, FileText, Monitor, Menu as MenuIcon, Bell, Wrench, Building2, BookOpen, LogIn } from "lucide-react"
import { Button } from "@/components/ui/button"
import { useAuthStore } from "@/stores/authStore"
import { PermissionGuard } from "@/components/shared/PermissionGuard"
import { cn } from "@/lib/utils"
import { useTranslation } from "react-i18next"

interface NavItem {
  labelKey: string
  icon: React.ComponentType<{ className?: string }>
  path: string
  /** Permission required to see this nav item. If omitted, visible to all authenticated users. */
  permission?: string
}

const navItems: NavItem[] = [
  { labelKey: "nav.dashboard", icon: LayoutDashboard, path: "/dashboard" },
  { labelKey: "nav.users", icon: Users, path: "/dashboard/users", permission: "users:read" },
  { labelKey: "nav.roles", icon: Shield, path: "/dashboard/roles", permission: "roles:read" },
  { labelKey: "nav.departments", icon: Building2, path: "/dashboard/departments", permission: "departments:read" },
  { labelKey: "nav.dicts", icon: BookOpen, path: "/dashboard/dicts", permission: "dicts:read" },
  { labelKey: "nav.auditLogs", icon: ScrollText, path: "/dashboard/audit-logs", permission: "audit:read" },
  { labelKey: "nav.loginLogs", icon: LogIn, path: "/dashboard/login-logs", permission: "login-logs:read" },
  { labelKey: "nav.files", icon: FileText, path: "/dashboard/files", permission: "files:read" },
  { labelKey: "nav.sessions", icon: Monitor, path: "/dashboard/sessions", permission: "sessions:read" },
  { labelKey: "nav.menus", icon: MenuIcon, path: "/dashboard/menus", permission: "menus:read" },
  { labelKey: "nav.notifications", icon: Bell, path: "/dashboard/notifications", permission: "notifications:read" },
  { labelKey: "nav.configs", icon: Wrench, path: "/dashboard/configs", permission: "configs:read" },
  { labelKey: "nav.profile", icon: UserCircle, path: "/dashboard/profile" },
  { labelKey: "nav.settings", icon: Settings, path: "/dashboard/settings" },
]

export function Sidebar() {
  const [collapsed, setCollapsed] = useState(false)
  const navigate = useNavigate()
  const location = useLocation()
  const logout = useAuthStore((s) => s.logout)
  const { t } = useTranslation()

  const handleLogout = async () => {
    await logout()
    navigate("/login")
  }

  return (
    <aside
      className={cn(
        "flex h-screen flex-col border-r bg-sidebar-background transition-all duration-300",
        collapsed ? "w-16" : "w-64",
      )}
    >
      {/* Logo */}
      <div className="flex h-14 items-center border-b px-4">
        {!collapsed && <h1 className="text-lg font-bold">Cradle</h1>}
        <Button
          variant="ghost"
          size="icon"
          className={cn("ml-auto", collapsed && "mx-auto")}
          onClick={() => setCollapsed(!collapsed)}
        >
          {collapsed ? <ChevronRight className="h-4 w-4" /> : <ChevronLeft className="h-4 w-4" />}
        </Button>
      </div>

      {/* Navigation */}
      <nav className="flex-1 space-y-1 p-2">
        {navItems.map((item) => {
          const Icon = item.icon
          const isActive = location.pathname === item.path
          const navButton = (
            <Button
              key={item.path}
              variant={isActive ? "secondary" : "ghost"}
              className={cn("w-full justify-start", collapsed && "justify-center px-2")}
              onClick={() => navigate(item.path)}
            >
              <Icon className="h-4 w-4" />
              {!collapsed && <span className="ml-2">{t(item.labelKey)}</span>}
            </Button>
          )

          // Wrap with PermissionGuard if permission is required
          if (item.permission) {
            return (
              <PermissionGuard key={item.path} permission={item.permission}>
                {navButton}
              </PermissionGuard>
            )
          }

          return navButton
        })}
      </nav>

      {/* Footer */}
      <div className="border-t p-2">
        <Button
          variant="ghost"
          className={cn("w-full justify-start text-destructive", collapsed && "justify-center px-2")}
          onClick={handleLogout}
        >
          <LogOut className="h-4 w-4" />
          {!collapsed && <span className="ml-2">{t("auth.signOut")}</span>}
        </Button>
      </div>
    </aside>
  )
}
