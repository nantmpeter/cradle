import { useQuery } from "@tanstack/react-query"
import { api } from "@/lib/api"
import type { LoginLog, LoginLogListParams } from "@/types/loginLog"
import type { PaginatedResponse } from "@/types/api"

const loginLogKeys = {
  all: ["loginLogs"] as const,
  list: (params: LoginLogListParams) => ["loginLogs", "list", params] as const,
  detail: (id: string) => ["loginLogs", "detail", id] as const,
}

export function useLoginLogs(params: LoginLogListParams = {}) {
  return useQuery({
    queryKey: loginLogKeys.list(params),
    queryFn: async () => {
      const sp = new URLSearchParams()
      if (params.email) sp.set("email", params.email)
      if (params.event) sp.set("event", params.event)
      if (params.ip_address) sp.set("ip_address", params.ip_address)
      if (params.start_date) sp.set("start_date", params.start_date)
      if (params.end_date) sp.set("end_date", params.end_date)
      if (params.page) sp.set("page", String(params.page))
      if (params.per_page) sp.set("per_page", String(params.per_page))
      const res = await api.get<PaginatedResponse<LoginLog>>(`/login-logs?${sp.toString()}`)
      return res.data
    },
  })
}

export function useLoginLogDetail(id: string | null) {
  return useQuery({
    queryKey: loginLogKeys.detail(id || ""),
    queryFn: async () => {
      const res = await api.get<LoginLog>(`/login-logs/${id}`)
      return res.data
    },
    enabled: !!id,
  })
}
