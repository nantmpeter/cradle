import { useAuthStore } from "@/stores/authStore"
import { Moon, Sun, UserCircle, Languages, ChevronRight, Home, Search, Users, Shield, BookOpen, Wrench } from "lucide-react"
import { Button } from "@/components/ui/button"
import { useEffect, useState, useRef } from "react"
import { useNavigate, useLocation } from "react-router-dom"
import { useTranslation } from "react-i18next"
import { useQuery } from "@tanstack/react-query"
import { api } from "@/lib/api"
import { reapplyAccent } from "@/lib/theme"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { NotificationBell } from "@/components/layout/NotificationBell"

interface SearchItem {
  type: string
  id: string
  title: string
  subtitle: string | null
  path: string
}

/** Map route path to i18n key for breadcrumb */
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

export function Header() {
  const user = useAuthStore((s) => s.user)
  const [dark, setDark] = useState(false)
  const navigate = useNavigate()
  const location = useLocation()
  const { t, i18n } = useTranslation()

  useEffect(() => {
    const saved = localStorage.getItem("theme")
    if (saved === "dark") {
      setDark(true)
      document.documentElement.classList.add("dark")
    }
    reapplyAccent()
  }, [])

  const toggleTheme = () => {
    setDark(!dark)
    if (!dark) {
      document.documentElement.classList.add("dark")
      localStorage.setItem("theme", "dark")
    } else {
      document.documentElement.classList.remove("dark")
      localStorage.setItem("theme", "light")
    }
    // Re-apply accent color for new mode
    reapplyAccent()
  }

  // Build breadcrumb from current path
  const pathSegments = location.pathname.split("/").filter(Boolean)
  const breadcrumbs = pathSegments.map((_, index) => {
    const path = "/" + pathSegments.slice(0, index + 1).join("/")
    const labelKey = routeLabelMap[path]
    return { path, label: labelKey ? t(labelKey) : pathSegments[index] }
  })

  // Global search
  const [searchQuery, setSearchQuery] = useState("")
  const [searchOpen, setSearchOpen] = useState(false)
  const searchRef = useRef<HTMLDivElement>(null)

  const { data: searchResults } = useQuery({
    queryKey: ["global-search", searchQuery],
    queryFn: async () => {
      const res = await api.get<{ results: SearchItem[] }>(`/search?q=${encodeURIComponent(searchQuery)}`)
      return res.data.results
    },
    enabled: searchQuery.length >= 2,
  })

  // Close search dropdown on outside click
  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (searchRef.current && !searchRef.current.contains(e.target as Node)) {
        setSearchOpen(false)
      }
    }
    document.addEventListener("mousedown", handler)
    return () => document.removeEventListener("mousedown", handler)
  }, [])

  const typeIcons: Record<string, React.ReactNode> = {
    user: <Users className="h-4 w-4" />,
    role: <Shield className="h-4 w-4" />,
    menu: <BookOpen className="h-4 w-4" />,
    config: <Wrench className="h-4 w-4" />,
  }

  const handleSelectResult = (item: SearchItem) => {
    setSearchOpen(false)
    setSearchQuery("")
    navigate(item.path)
  }

  return (
    <header className="flex h-14 items-center justify-between border-b bg-background px-6">
      <nav className="flex items-center gap-1 text-sm">
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7"
          onClick={() => navigate("/dashboard")}
        >
          <Home className="h-4 w-4 text-muted-foreground" />
        </Button>
        {breadcrumbs.map((crumb, index) => (
          <span key={crumb.path} className="flex items-center gap-1">
            <ChevronRight className="h-3 w-3 text-muted-foreground" />
            {index === breadcrumbs.length - 1 ? (
              <span className="font-medium">{crumb.label}</span>
            ) : (
              <Button
                variant="link"
                className="h-auto p-0 text-muted-foreground hover:text-foreground"
                onClick={() => navigate(crumb.path)}
              >
                {crumb.label}
              </Button>
            )}
          </span>
        ))}
      </nav>

      {/* Global Search */}
      <div ref={searchRef} className="relative mx-4 flex-1 max-w-md">
        <div className="relative">
          <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
          <input
            type="text"
            placeholder={t("common.search") || "Search..."}
            className="h-9 w-full rounded-md border border-input bg-background pl-9 pr-3 text-sm outline-none focus:ring-1 focus:ring-ring placeholder:text-muted-foreground"
            value={searchQuery}
            onChange={(e) => {
              setSearchQuery(e.target.value)
              setSearchOpen(true)
            }}
            onFocus={() => searchQuery.length >= 2 && setSearchOpen(true)}
          />
        </div>
        {searchOpen && searchQuery.length >= 2 && (
          <div className="absolute top-full left-0 z-50 mt-1 w-full rounded-md border bg-popover shadow-md">
            {searchResults && searchResults.length > 0 ? (
              <ul className="py-1">
                {searchResults.map((item) => (
                  <li key={`${item.type}-${item.id}`}>
                    <button
                      className="flex w-full items-center gap-3 px-3 py-2 text-sm hover:bg-accent text-left"
                      onClick={() => handleSelectResult(item)}
                    >
                      <span className="text-muted-foreground">{typeIcons[item.type] || <Search className="h-4 w-4" />}</span>
                      <div className="flex-1 min-w-0">
                        <div className="truncate font-medium">{item.title}</div>
                        {item.subtitle && (
                          <div className="truncate text-xs text-muted-foreground">{item.subtitle}</div>
                        )}
                      </div>
                      <span className="text-xs text-muted-foreground capitalize">{item.type}</span>
                    </button>
                  </li>
                ))}
              </ul>
            ) : searchResults && searchResults.length === 0 ? (
              <div className="px-3 py-6 text-center text-sm text-muted-foreground">
                {t("common.noResults") || "No results found"}
              </div>
            ) : (
              <div className="px-3 py-6 text-center text-sm text-muted-foreground">
                ...
              </div>
            )}
          </div>
        )}
      </div>

      <div className="flex items-center gap-2">
        <NotificationBell />
        <Button variant="ghost" size="icon" onClick={() => navigate("/dashboard/profile")}>
          <UserCircle className="h-4 w-4" />
        </Button>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon">
              <Languages className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={() => i18n.changeLanguage("zh-CN")}>
              中文
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => i18n.changeLanguage("en-US")}>
              English
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
        <Button variant="ghost" size="icon" onClick={toggleTheme}>
          {dark ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
        </Button>
      </div>
    </header>
  )
}
