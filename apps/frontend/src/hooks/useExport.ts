import { useMutation } from "@tanstack/react-query"
import { api } from "@/lib/api"

export interface ExportFilters {
  search?: string
  role?: string
  role_id?: string
  status?: string
}

// ─── Export Users ────────────────────────────────────────────────────────────

export function useExportUsers() {
  return useMutation({
    mutationFn: async ({ format, filters }: { format: "csv" | "xlsx"; filters?: ExportFilters }) => {
      const params = new URLSearchParams()
      params.set("format", format)
      if (filters?.search) params.set("search", filters.search)
      if (filters?.role) params.set("role", filters.role)
      if (filters?.role_id) params.set("role_id", filters.role_id)
      if (filters?.status) params.set("status", filters.status)

      const res = await api.get(`/export/users?${params.toString()}`, {
        responseType: "blob",
      })
      // Trigger file download
      const blob = new Blob([res.data])
      const url = window.URL.createObjectURL(blob)
      const link = document.createElement("a")
      link.href = url
      link.download = `users.${format}`
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)
      window.URL.revokeObjectURL(url)
    },
  })
}
