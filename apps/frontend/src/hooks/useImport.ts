import { useMutation, useQueryClient } from "@tanstack/react-query"
import { api } from "@/lib/api"

export interface ImportResult {
  success_count: number
  fail_count: number
  errors: { row: number; email: string; reason: string }[]
  generated_passwords: { email: string; password: string }[]
}

export function useImportUsers() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (file: File) => {
      const formData = new FormData()
      formData.append("file", file)
      const res = await api.post("/users/import", formData, {
        headers: { "Content-Type": "multipart/form-data" },
      })
      return res.data as ImportResult
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["users"] })
    },
  })
}

export function useDownloadImportTemplate() {
  return useMutation({
    mutationFn: async () => {
      const res = await api.get("/users/import/template", { responseType: "blob" })
      const url = window.URL.createObjectURL(new Blob([res.data]))
      const link = document.createElement("a")
      link.href = url
      link.setAttribute("download", "import_template.xlsx")
      document.body.appendChild(link)
      link.click()
      link.remove()
      window.URL.revokeObjectURL(url)
    },
  })
}
