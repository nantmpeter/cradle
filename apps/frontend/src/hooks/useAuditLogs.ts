import { useQuery } from "@tanstack/react-query"
import { api } from "@/lib/api"
import type { AuditLog, AuditLogListParams, PaginatedResponse } from "@/types/api"

// ─── Query Keys ──────────────────────────────────────────────────────────────

const auditLogKeys = {
  all: ["audit-logs"] as const,
  list: (params: AuditLogListParams) => ["audit-logs", "list", params] as const,
  detail: (id: string) => ["audit-logs", "detail", id] as const,
}

// ─── List Audit Logs ─────────────────────────────────────────────────────────

export function useAuditLogs(params: AuditLogListParams = {}) {
  return useQuery({
    queryKey: auditLogKeys.list(params),
    queryFn: async () => {
      const searchParams = new URLSearchParams()
      if (params.page) searchParams.set("page", String(params.page))
      if (params.per_page) searchParams.set("per_page", String(params.per_page))
      if (params.user_id) searchParams.set("user_id", params.user_id)
      if (params.action) searchParams.set("action", params.action)
      if (params.from) searchParams.set("from", params.from)
      if (params.to) searchParams.set("to", params.to)

      const res = await api.get<PaginatedResponse<AuditLog>>(`/audit-logs?${searchParams.toString()}`)
      return res.data
    },
  })
}

// ─── Get Audit Log Detail ────────────────────────────────────────────────────

export function useAuditLogDetail(id: string) {
  return useQuery({
    queryKey: auditLogKeys.detail(id),
    queryFn: async () => {
      const res = await api.get<AuditLog>(`/audit-logs/${id}`)
      return res.data
    },
    enabled: !!id,
  })
}
