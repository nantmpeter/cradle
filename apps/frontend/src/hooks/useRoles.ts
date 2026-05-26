import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { api } from "@/lib/api"
import type {
  RoleResponse,
  CreateRoleRequest,
  UpdateRoleRequest,
  UpdateRolePermissionsRequest,
} from "@/types/api"

// ─── Query Keys ──────────────────────────────────────────────────────────────

const roleKeys = {
  all: ["roles"] as const,
  list: ["roles", "list"] as const,
  detail: (id: string) => ["roles", "detail", id] as const,
}

// ─── List Roles ──────────────────────────────────────────────────────────────

export function useRoles() {
  return useQuery({
    queryKey: roleKeys.list,
    queryFn: async () => {
      const res = await api.get<RoleResponse[]>("/roles")
      return res.data
    },
  })
}

// ─── Get Single Role ─────────────────────────────────────────────────────────

export function useRole(id: string) {
  return useQuery({
    queryKey: roleKeys.detail(id),
    queryFn: async () => {
      const res = await api.get<RoleResponse>(`/roles/${id}`)
      return res.data
    },
    enabled: !!id,
  })
}

// ─── Create Role ─────────────────────────────────────────────────────────────

export function useCreateRole() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (data: CreateRoleRequest) => {
      const res = await api.post<RoleResponse>("/roles", data)
      return res.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: roleKeys.all })
    },
  })
}

// ─── Update Role ─────────────────────────────────────────────────────────────

export function useUpdateRole() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ id, data }: { id: string; data: UpdateRoleRequest }) => {
      const res = await api.put<RoleResponse>(`/roles/${id}`, data)
      return res.data
    },
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: roleKeys.all })
      queryClient.invalidateQueries({ queryKey: roleKeys.detail(variables.id) })
    },
  })
}

// ─── Delete Role ─────────────────────────────────────────────────────────────

export function useDeleteRole() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/roles/${id}`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: roleKeys.all })
    },
  })
}

// ─── Update Role Permissions ─────────────────────────────────────────────────

export function useUpdateRolePermissions() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ id, data }: { id: string; data: UpdateRolePermissionsRequest }) => {
      const res = await api.put<RoleResponse>(`/roles/${id}/permissions`, data)
      return res.data
    },
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: roleKeys.all })
      queryClient.invalidateQueries({ queryKey: roleKeys.detail(variables.id) })
    },
  })
}
