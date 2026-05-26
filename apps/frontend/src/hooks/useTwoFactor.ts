import { useMutation, useQueryClient } from "@tanstack/react-query"
import { api } from "@/lib/api"
import type { Setup2FaResponse, Enable2FaRequest, Disable2FaRequest, Verify2FaRequest } from "@/types/twoFactor"
import type { TokenResponse } from "@/types/api"

export function useSetup2Fa() {
  return useMutation({
    mutationFn: async (): Promise<Setup2FaResponse> => {
      const res = await api.post("/auth/2fa/setup")
      return res.data
    },
  })
}

export function useEnable2Fa() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (data: Enable2FaRequest) => {
      const res = await api.post("/auth/2fa/enable", data)
      return res.data
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["user", "me"] })
      qc.invalidateQueries({ queryKey: ["dashboard", "settings"] })
    },
  })
}

export function useDisable2Fa() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (data: Disable2FaRequest) => {
      const res = await api.post("/auth/2fa/disable", data)
      return res.data
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["user", "me"] })
      qc.invalidateQueries({ queryKey: ["dashboard", "settings"] })
    },
  })
}

export function useVerify2FaLogin() {
  return useMutation({
    mutationFn: async (data: Verify2FaRequest): Promise<TokenResponse> => {
      const res = await api.post("/auth/2fa/verify", data)
      return res.data
    },
  })
}

export function useResetUser2Fa() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (userId: string) => {
      const res = await api.post(`/users/${userId}/2fa/reset`)
      return res.data
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["users"] })
    },
  })
}
