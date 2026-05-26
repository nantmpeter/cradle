import { create } from "zustand"
import type { User } from "@/types/api"
import { api } from "@/lib/api"

interface AuthState {
  user: User | null
  permissions: string[]
  isAuthenticated: boolean
  mustChangePassword: boolean
  setUser: (user: User | null) => void
  setMustChangePassword: (value: boolean) => void
  logout: () => Promise<void>
  updateUser: (updates: Partial<User>) => void
  hasPermission: (permission: string) => boolean
}

export const useAuthStore = create<AuthState>((set, get) => ({
  user: null,
  permissions: [],
  isAuthenticated: !!localStorage.getItem("access_token"),
  mustChangePassword: false,

  setUser: (user) =>
    set({
      user,
      permissions: user?.permissions ?? [],
      isAuthenticated: !!user,
      mustChangePassword: user?.must_change_password ?? false,
    }),

  setMustChangePassword: (value) => set({ mustChangePassword: value }),

  logout: async () => {
    const refreshToken = localStorage.getItem("refresh_token")
    if (refreshToken) {
      try {
        await api.post("/auth/logout", { refresh_token: refreshToken })
      } catch {
        // Ignore errors on logout (token may already be invalid)
      }
    }
    localStorage.removeItem("access_token")
    localStorage.removeItem("refresh_token")
    set({ user: null, permissions: [], isAuthenticated: false, mustChangePassword: false })
  },

  updateUser: (updates) => {
    const currentUser = get().user
    if (currentUser) {
      const updatedUser = { ...currentUser, ...updates }
      set({
        user: updatedUser,
        permissions: updatedUser.permissions ?? get().permissions,
        mustChangePassword: updatedUser.must_change_password,
      })
    }
  },

  hasPermission: (permission: string) => {
    const user = get().user
    if (!user) return false

    // superadmin always has all permissions
    if (user.role === "superadmin") return true

    const permissions = get().permissions
    return permissions.includes(permission)
  },
}))
