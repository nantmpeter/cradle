import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { api } from "@/lib/api"
import type { FileListResponse, FileRecord } from "@/types/file"

// ─── Query Keys ──────────────────────────────────────────────────────────────

const fileKeys = {
  all: ["files"] as const,
  list: (params: { page?: number; per_page?: number }) => ["files", "list", params] as const,
  detail: (id: string) => ["files", "detail", id] as const,
}

// ─── List Files ──────────────────────────────────────────────────────────────

export function useFiles(params: { page?: number; per_page?: number } = {}) {
  return useQuery({
    queryKey: fileKeys.list(params),
    queryFn: async () => {
      const searchParams = new URLSearchParams()
      if (params.page) searchParams.set("page", String(params.page))
      if (params.per_page) searchParams.set("per_page", String(params.per_page))

      const res = await api.get<FileListResponse>(`/files?${searchParams.toString()}`)
      return res.data
    },
  })
}

// ─── Upload File ─────────────────────────────────────────────────────────────

export function useUploadFile() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (file: File) => {
      const formData = new FormData()
      formData.append("file", file)
      const res = await api.post<FileRecord>("/files", formData, {
        headers: { "Content-Type": "multipart/form-data" },
      })
      return res.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: fileKeys.all })
    },
  })
}

// ─── Delete File ─────────────────────────────────────────────────────────────

export function useDeleteFile() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/files/${id}`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: fileKeys.all })
    },
  })
}
