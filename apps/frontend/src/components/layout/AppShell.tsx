import { Outlet, useLocation, useNavigate } from "react-router-dom"
import { useState, useEffect, useCallback } from "react"
import { X, Home } from "lucide-react"
import { Sidebar } from "./Sidebar"
import { Header } from "./Header"
import { Button } from "@/components/ui/button"
import { useTranslation } from "react-i18next"
import { ScrollArea, ScrollBar } from "@/components/ui/scroll-area"
import { useSseNotifications } from "@/hooks/useSseNotifications"

/** Map route path to i18n key for tab label */
const routeLabelMap: Record<string, string> = {
  "/dashboard": "nav.dashboard",
  "/dashboard/users": "nav.users",
  "/dashboard/roles": "nav.roles",
  "/dashboard/departments": "nav.departments",
  "/dashboard/dicts": "nav.dicts",
  "/dashboard/audit-logs": "nav.auditLogs",
  "/dashboard/login-logs": "nav.loginLogs",
  "/dashboard/files": "nav.files",
  "/dashboard/sessions": "nav.sessions",
  "/dashboard/menus": "nav.menus",
  "/dashboard/notifications": "nav.notifications",
  "/dashboard/configs": "nav.configs",
  "/dashboard/profile": "nav.profile",
  "/dashboard/settings": "nav.settings",
}

export interface Tab {
  path: string
  labelKey: string
}

const STORAGE_KEY = "app-tabs"

function loadTabs(): Tab[] {
  try {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved) return JSON.parse(saved)
  } catch {}
  return [{ path: "/dashboard", labelKey: "nav.dashboard" }]
}

function saveTabs(tabs: Tab[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(tabs))
}

export function AppShell() {
  const location = useLocation()
  const navigate = useNavigate()
  const { t } = useTranslation()
  const [tabs, setTabs] = useState<Tab[]>(loadTabs)

  // Connect to SSE for real-time notifications
  useSseNotifications()

  // Add current route to tabs when navigating
  useEffect(() => {
    const currentPath = location.pathname
    setTabs((prev) => {
      const labelKey = routeLabelMap[currentPath]
      if (!labelKey) return prev // Skip unknown routes (e.g. forbidden)
      const exists = prev.some((tab) => tab.path === currentPath)
      if (exists) return prev
      const newTabs = [...prev, { path: currentPath, labelKey }]
      saveTabs(newTabs)
      return newTabs
    })
  }, [location.pathname])

  const closeTab = useCallback(
    (path: string) => {
      setTabs((prev) => {
        const idx = prev.findIndex((tab) => tab.path === path)
        const newTabs = prev.filter((tab) => tab.path !== path)
        saveTabs(newTabs)
        // If closing active tab, navigate to neighbor
        if (path === location.pathname && newTabs.length > 0) {
          const nextIdx = Math.min(idx, newTabs.length - 1)
          navigate(newTabs[nextIdx].path)
        }
        return newTabs
      })
    },
    [location.pathname, navigate],
  )

  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar />
      <div className="flex flex-1 flex-col overflow-hidden">
        <Header />
        {/* Tab Bar */}
        <div className="flex h-9 items-center border-b bg-muted/40 px-2">
          <ScrollArea className="flex-1">
            <div className="flex items-center gap-0.5">
              {tabs.map((tab) => {
                const isActive = tab.path === location.pathname
                const isDashboard = tab.path === "/dashboard"
                return (
                  <button
                    key={tab.path}
                    onClick={() => navigate(tab.path)}
                    className={`flex items-center gap-1 rounded-md px-2.5 py-1 text-xs transition-colors ${
                      isActive
                        ? "bg-background text-foreground shadow-sm"
                        : "text-muted-foreground hover:bg-background/50 hover:text-foreground"
                    }`}
                  >
                    {isDashboard && <Home className="h-3 w-3" />}
                    <span>{t(tab.labelKey)}</span>
                    {!isDashboard && (
                      <span
                        role="button"
                        tabIndex={0}
                        onClick={(e) => {
                          e.stopPropagation()
                          closeTab(tab.path)
                        }}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") {
                            e.stopPropagation()
                            closeTab(tab.path)
                          }
                        }}
                        className="ml-0.5 rounded-sm p-0.5 opacity-60 hover:opacity-100 hover:bg-muted"
                      >
                        <X className="h-3 w-3" />
                      </span>
                    )}
                  </button>
                )
              })}
            </div>
            <ScrollBar orientation="horizontal" />
          </ScrollArea>
        </div>
        <main className="flex-1 overflow-y-auto p-6">
          <Outlet />
        </main>
      </div>
    </div>
  )
}
