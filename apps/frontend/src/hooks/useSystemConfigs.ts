import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { api } from "@/lib/api"
import type { SystemConfig } from "@/types/systemConfig"

const configKeys = {
  all: ["configs"] as const,
  group: (group: string) => ["configs", group] as const,
}

export function useSystemConfigs() {
  return useQuery({
    queryKey: configKeys.all,
    queryFn: async (): Promise<SystemConfig[]> => {
      const res = await api.get("/configs")
      // API returns { data: SystemConfig[] }
      const payload = res.data
      return Array.isArray(payload) ? payload : payload?.data ?? []
    },
  })
}

export function useConfigGroup(group: string) {
  return useQuery({
    queryKey: configKeys.group(group),
    queryFn: async (): Promise<SystemConfig[]> => {
      const res = await api.get(`/configs/${group}`)
      const payload = res.data
      return Array.isArray(payload) ? payload : payload?.data ?? []
    },
    enabled: !!group,
  })
}

export function useUpdateConfig() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ group, key, value, description }: { group: string; key: string; value: string; description?: string }) => {
      const res = await api.put(`/configs/${group}/${key}`, { value, description })
      return res.data
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: configKeys.all })
    },
  })
}
