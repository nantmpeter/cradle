import { useQuery } from "@tanstack/react-query"
import { api } from "@/lib/api"
import type { DashboardStats, DashboardSettings } from "@/types/dashboard"

// ─── Query Keys ──────────────────────────────────────────────────────────────

const dashboardKeys = {
  stats: ["dashboard", "stats"] as const,
  settings: ["dashboard", "settings"] as const,
}

// ─── Dashboard Stats ─────────────────────────────────────────────────────────

/** 获取 Dashboard 统计数据（用户/角色/登录/审计日志） */
export function useDashboardStats() {
  return useQuery({
    queryKey: dashboardKeys.stats,
    queryFn: async () => {
      const res = await api.get<DashboardStats>("/dashboard/stats")
      return res.data
    },
  })
}

// ─── Dashboard Settings ──────────────────────────────────────────────────────

/** 获取系统设置信息（系统/数据库/当前用户） */
export function useDashboardSettings() {
  return useQuery({
    queryKey: dashboardKeys.settings,
    queryFn: async () => {
      const res = await api.get<DashboardSettings>("/dashboard/settings")
      return res.data
    },
  })
}
