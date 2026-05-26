import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom"
import { AppShell } from "@/components/layout/AppShell"
import { ProtectedRoute } from "@/components/auth/ProtectedRoute"
import { ForbiddenPage } from "@/components/auth/ForbiddenPage"
import { LoginForm } from "@/components/auth/LoginForm"
import { TwoFactorVerifyForm } from "@/components/auth/TwoFactorVerifyForm"
import { DashboardPage } from "@/components/dashboard/DashboardPage"
import { UserListPage } from "@/components/users/UserListPage"
import { ProfilePage } from "@/components/profile/ProfilePage"
import { RoleListPage } from "@/components/roles/RoleListPage"
import { AuditLogListPage } from "@/components/audit/AuditLogListPage"
import { SettingsPage } from "@/components/dashboard/SettingsPage"
import { FilesPage } from "@/components/files/FilesPage"
import { SessionsPage } from "@/components/sessions/SessionsPage"
import { MenusPage } from "@/components/menus/MenusPage"
import { NotificationsPage } from "@/components/notifications/NotificationsPage"
import { SystemConfigPage } from "@/components/systemConfig/SystemConfigPage"
import { DepartmentsPage } from "@/components/departments/DepartmentsPage"
import { DictsPage } from "@/components/dicts/DictsPage"
import { LoginLogListPage } from "@/components/loginLogs/LoginLogListPage"
import { PermissionRouteGuard } from "@/components/auth/PermissionRouteGuard"

export function AppRoutes() {
  return (
    <BrowserRouter>
      <Routes>
        {/* Public routes */}
        <Route path="/login" element={<LoginForm />} />
        <Route path="/login/2fa" element={<TwoFactorVerifyForm />} />

        {/* Protected routes */}
        <Route
          path="/dashboard"
          element={
            <ProtectedRoute>
              <AppShell />
            </ProtectedRoute>
          }
        >
          <Route index element={<DashboardPage />} />

          {/* User management — requires users:read */}
          <Route
            path="users"
            element={
              <PermissionRouteGuard permission="users:read">
                <UserListPage />
              </PermissionRouteGuard>
            }
          />

          {/* Role management — requires roles:read */}
          <Route
            path="roles"
            element={
              <PermissionRouteGuard permission="roles:read">
                <RoleListPage />
              </PermissionRouteGuard>
            }
          />

          {/* Audit logs — requires audit:read */}
          <Route
            path="audit-logs"
            element={
              <PermissionRouteGuard permission="audit:read">
                <AuditLogListPage />
              </PermissionRouteGuard>
            }
          />

          {/* File management — requires files:read */}
          <Route
            path="files"
            element={
              <PermissionRouteGuard permission="files:read">
                <FilesPage />
              </PermissionRouteGuard>
            }
          />

          {/* Session management — requires sessions:read */}
          <Route
            path="sessions"
            element={
              <PermissionRouteGuard permission="sessions:read">
                <SessionsPage />
              </PermissionRouteGuard>
            }
          />

          {/* Menu management — requires menus:read */}
          <Route
            path="menus"
            element={
              <PermissionRouteGuard permission="menus:read">
                <MenusPage />
              </PermissionRouteGuard>
            }
          />

          {/* Notifications — requires notifications:read */}
          <Route
            path="notifications"
            element={
              <PermissionRouteGuard permission="notifications:read">
                <NotificationsPage />
              </PermissionRouteGuard>
            }
          />

          {/* System Config — requires configs:read */}
          <Route
            path="configs"
            element={
              <PermissionRouteGuard permission="configs:read">
                <SystemConfigPage />
              </PermissionRouteGuard>
            }
          />

          {/* Departments — requires departments:read */}
          <Route
            path="departments"
            element={
              <PermissionRouteGuard permission="departments:read">
                <DepartmentsPage />
              </PermissionRouteGuard>
            }
          />

          {/* Dictionary — requires dicts:read */}
          <Route
            path="dicts"
            element={
              <PermissionRouteGuard permission="dicts:read">
                <DictsPage />
              </PermissionRouteGuard>
            }
          />

          {/* Login Logs — requires login-logs:read */}
          <Route
            path="login-logs"
            element={
              <PermissionRouteGuard permission="login-logs:read">
                <LoginLogListPage />
              </PermissionRouteGuard>
            }
          />

          <Route path="profile" element={<ProfilePage />} />
          <Route path="settings" element={<SettingsPage />} />

          {/* 403 catch-all within dashboard */}
          <Route path="forbidden" element={<ForbiddenPage />} />
        </Route>

        {/* Redirect */}
        <Route path="*" element={<Navigate to="/dashboard" replace />} />
      </Routes>
    </BrowserRouter>
  )
}
