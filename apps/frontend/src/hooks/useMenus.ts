import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { api } from "@/lib/api"
import type {
  MenuListResponse,
  MenuTreeNode,
  CreateMenuRequest,
  UpdateMenuRequest,
  MenuItem,
} from "@/types/menu"

// ─── Query Keys ──────────────────────────────────────────────────────────────

const menuKeys = {
  all: ["menus"] as const,
  tree: ["menus", "tree"] as const,
}

// ─── List Menus ──────────────────────────────────────────────────────────────

export function useMenus() {
  return useQuery({
    queryKey: menuKeys.all,
    queryFn: async () => {
      const res = await api.get<MenuListResponse>("/menus")
      return res.data
    },
  })
}

// ─── Get Menu Tree ───────────────────────────────────────────────────────────

export function useMenuTree() {
  return useQuery({
    queryKey: menuKeys.tree,
    queryFn: async () => {
      const res = await api.get<{ data: MenuTreeNode[] }>("/menus/tree")
      return res.data
    },
  })
}

// ─── Create Menu ─────────────────────────────────────────────────────────────

export function useCreateMenu() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (data: CreateMenuRequest) => {
      const res = await api.post<MenuItem>("/menus", data)
      return res.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: menuKeys.all })
      queryClient.invalidateQueries({ queryKey: menuKeys.tree })
    },
  })
}

// ─── Update Menu ─────────────────────────────────────────────────────────────

export function useUpdateMenu() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ id, data }: { id: string; data: UpdateMenuRequest }) => {
      const res = await api.put<MenuItem>(`/menus/${id}`, data)
      return res.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: menuKeys.all })
      queryClient.invalidateQueries({ queryKey: menuKeys.tree })
    },
  })
}

// ─── Delete Menu ─────────────────────────────────────────────────────────────

export function useDeleteMenu() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/menus/${id}`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: menuKeys.all })
      queryClient.invalidateQueries({ queryKey: menuKeys.tree })
    },
  })
}
