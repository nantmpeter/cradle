import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { api } from "@/lib/api"
import type { SessionListResponse } from "@/types/session"

// ─── Query Keys ──────────────────────────────────────────────────────────────

const sessionKeys = {
  all: ["sessions"] as const,
  me: ["sessions", "me"] as const,
}

// ─── List All Sessions (admin) ──────────────────────────────────────────────

export function useSessions() {
  return useQuery({
    queryKey: sessionKeys.all,
    queryFn: async () => {
      const res = await api.get<SessionListResponse>("/sessions")
      return res.data
    },
  })
}

// ─── List My Sessions ───────────────────────────────────────────────────────

export function useMySessions() {
  return useQuery({
    queryKey: sessionKeys.me,
    queryFn: async () => {
      const res = await api.get<SessionListResponse>("/sessions/me")
      return res.data
    },
  })
}

// ─── Terminate Session ──────────────────────────────────────────────────────

export function useTerminateSession() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/sessions/${id}`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: sessionKeys.all })
      queryClient.invalidateQueries({ queryKey: sessionKeys.me })
    },
  })
}

// ─── Force Logout User (terminate all sessions for a user) ────────────────────

export function useForceLogoutUser() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (userId: string) => {
      const res = await api.delete(`/sessions/user/${userId}`)
      return res.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: sessionKeys.all })
      queryClient.invalidateQueries({ queryKey: sessionKeys.me })
    },
  })
}
