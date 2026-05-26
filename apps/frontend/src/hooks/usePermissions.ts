import { useQuery } from "@tanstack/react-query"
import { api } from "@/lib/api"
import type { Permission } from "@/types/api"

// ─── Query Keys ──────────────────────────────────────────────────────────────

const permissionKeys = {
  all: ["permissions"] as const,
}

// ─── List Permissions ────────────────────────────────────────────────────────

export function usePermissions() {
  return useQuery({
    queryKey: permissionKeys.all,
    queryFn: async () => {
      const res = await api.get<Permission[]>("/roles/permissions")
      return res.data
    },
  })
}
