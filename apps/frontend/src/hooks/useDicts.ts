import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { api } from "@/lib/api"
import type {
  DictType, DictItem, DictPublicItem,
  CreateDictTypeRequest, UpdateDictTypeRequest,
  CreateDictItemRequest, UpdateDictItemRequest,
  DictTypeListParams,
} from "@/types/dict"
import type { PaginatedResponse } from "@/types/api"

const dictKeys = {
  all: ["dicts"] as const,
  types: (params: DictTypeListParams) => ["dicts", "types", params] as const,
  items: (typeId: string) => ["dicts", "items", typeId] as const,
  byCode: (code: string) => ["dicts", "code", code] as const,
}

export function useDictTypes(params: DictTypeListParams = {}) {
  return useQuery({
    queryKey: dictKeys.types(params),
    queryFn: async () => {
      const sp = new URLSearchParams()
      if (params.search) sp.set("search", params.search)
      if (params.status) sp.set("status", params.status)
      if (params.page) sp.set("page", String(params.page))
      if (params.per_page) sp.set("per_page", String(params.per_page))
      const res = await api.get<PaginatedResponse<DictType>>(`/dict-types?${sp.toString()}`)
      return res.data
    },
  })
}

export function useCreateDictType() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (data: CreateDictTypeRequest) => {
      const res = await api.post("/dict-types", data)
      return res.data as DictType
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: dictKeys.all })
    },
  })
}

export function useUpdateDictType() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ id, data }: { id: string; data: UpdateDictTypeRequest }) => {
      const res = await api.put(`/dict-types/${id}`, data)
      return res.data as DictType
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: dictKeys.all })
    },
  })
}

export function useDeleteDictType() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/dict-types/${id}`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: dictKeys.all })
    },
  })
}

export function useDictItems(typeId: string | null) {
  return useQuery({
    queryKey: dictKeys.items(typeId || ""),
    queryFn: async () => {
      const res = await api.get<DictItem[]>(`/dict-types/${typeId}/items`)
      const payload = res.data
      return Array.isArray(payload) ? payload : (payload as any)?.data ?? []
    },
    enabled: !!typeId,
  })
}

export function useCreateDictItem() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ typeId, data }: { typeId: string; data: CreateDictItemRequest }) => {
      const res = await api.post(`/dict-types/${typeId}/items`, data)
      return res.data as DictItem
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: dictKeys.all })
    },
  })
}

export function useUpdateDictItem() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ id, data }: { id: string; data: UpdateDictItemRequest }) => {
      const res = await api.put(`/dict-items/${id}`, data)
      return res.data as DictItem
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: dictKeys.all })
    },
  })
}

export function useDeleteDictItem() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/dict-items/${id}`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: dictKeys.all })
    },
  })
}

export function useDictByCode(code: string) {
  return useQuery({
    queryKey: dictKeys.byCode(code),
    queryFn: async () => {
      const res = await api.get(`/dicts/${code}`)
      const payload = res.data
      return (Array.isArray(payload) ? payload : (payload as any)?.data ?? []) as DictPublicItem[]
    },
    enabled: !!code,
  })
}
