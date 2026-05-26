import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { api } from "@/lib/api"
import type { Department, DepartmentTreeNode, CreateDepartmentRequest, UpdateDepartmentRequest } from "@/types/department"

const deptKeys = {
  all: ["departments"] as const,
  list: ["departments", "list"] as const,
}

export function useDepartments() {
  return useQuery({
    queryKey: deptKeys.list,
    queryFn: async () => {
      const res = await api.get("/departments")
      return res.data as { departments: Department[]; tree: DepartmentTreeNode[] }
    },
  })
}

export function useCreateDepartment() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (data: CreateDepartmentRequest) => {
      const res = await api.post("/departments", data)
      return res.data as Department
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: deptKeys.all })
    },
  })
}

export function useUpdateDepartment() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ id, data }: { id: string; data: UpdateDepartmentRequest }) => {
      const res = await api.put(`/departments/${id}`, data)
      return res.data as Department
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: deptKeys.all })
    },
  })
}

export function useDeleteDepartment() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/departments/${id}`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: deptKeys.all })
    },
  })
}
