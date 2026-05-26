import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { api } from "@/lib/api"
import type {
  User,
  CreateUserRequest,
  UpdateUserRequest,
  UpdateStatusRequest,
  ChangeMyPasswordRequest,
  ResetPasswordRequest,
  PaginatedResponse,
  ListUsersParams,
} from "@/types/api"

// ─── Query Keys ──────────────────────────────────────────────────────────────

const userKeys = {
  all: ["users"] as const,
  list: (params: ListUsersParams) => ["users", "list", params] as const,
  detail: (id: string) => ["users", "detail", id] as const,
  me: ["users", "me"] as const,
}

// ─── List Users ──────────────────────────────────────────────────────────────

export function useUsers(params: ListUsersParams = {}) {
  return useQuery({
    queryKey: userKeys.list(params),
    queryFn: async () => {
      const searchParams = new URLSearchParams()
      if (params.page) searchParams.set("page", String(params.page))
      if (params.per_page) searchParams.set("per_page", String(params.per_page))
      if (params.search) searchParams.set("search", params.search)
      if (params.role) searchParams.set("role", params.role)
      if (params.role_id) searchParams.set("role_id", params.role_id)
      if (params.status) searchParams.set("status", params.status)
      if (params.sort_by) searchParams.set("sort_by", params.sort_by)
      if (params.sort_order) searchParams.set("sort_order", params.sort_order)

      const res = await api.get<PaginatedResponse<User>>(`/users?${searchParams.toString()}`)
      return res.data
    },
  })
}

// ─── Get Current User ───────────────────────────────────────────────────────

export function useMe() {
  return useQuery({
    queryKey: userKeys.me,
    queryFn: async () => {
      const res = await api.get<User>("/users/me")
      return res.data
    },
    enabled: !!localStorage.getItem("access_token"),
  })
}

// ─── Create User ─────────────────────────────────────────────────────────────

export function useCreateUser() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (data: CreateUserRequest) => {
      const res = await api.post<User>("/users", data)
      return res.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: userKeys.all })
    },
  })
}

// ─── Update User ─────────────────────────────────────────────────────────────

export function useUpdateUser() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ id, data }: { id: string; data: UpdateUserRequest }) => {
      const res = await api.put<User>(`/users/${id}`, data)
      return res.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: userKeys.all })
    },
  })
}

// ─── Delete User ─────────────────────────────────────────────────────────────

export function useDeleteUser() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/users/${id}`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: userKeys.all })
    },
  })
}

// ─── Toggle Status ───────────────────────────────────────────────────────────

export function useToggleStatus() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ id, data }: { id: string; data: UpdateStatusRequest }) => {
      const res = await api.put<User>(`/users/${id}/status`, data)
      return res.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: userKeys.all })
    },
  })
}

// ─── Reset Password ──────────────────────────────────────────────────────────

export function useResetPassword() {
  return useMutation({
    mutationFn: async ({ id, data }: { id: string; data: ResetPasswordRequest }) => {
      await api.put(`/users/${id}/password`, data)
    },
  })
}

// ─── Change My Password ──────────────────────────────────────────────────────

export function useChangeMyPassword() {
  return useMutation({
    mutationFn: async (data: ChangeMyPasswordRequest) => {
      await api.put("/users/me/password", data)
    },
  })
}
