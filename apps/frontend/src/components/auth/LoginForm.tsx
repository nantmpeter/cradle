import { useState } from "react"
import { useNavigate } from "react-router-dom"
import { useForm } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { api } from "@/lib/api"
import { useAuthStore } from "@/stores/authStore"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { useTranslation } from "react-i18next"

const loginSchema = z.object({
  email: z.string().email(),
  password: z.string().min(6),
})

type LoginFormData = z.infer<typeof loginSchema>

export function LoginForm() {
  const [error, setError] = useState("")
  const navigate = useNavigate()
  const setUser = useAuthStore((s) => s.setUser)
  const setMustChangePassword = useAuthStore((s) => s.setMustChangePassword)
  const { t } = useTranslation()

  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<LoginFormData>({
    resolver: zodResolver(loginSchema),
  })

  const onSubmit = async (data: LoginFormData) => {
    try {
      setError("")
      const res = await api.post("/auth/login", data)
      const { access_token, refresh_token, must_change_password, requires_2fa, temp_token } = res.data

      if (requires_2fa && temp_token) {
        localStorage.setItem("temp_token", temp_token)
        navigate("/login/2fa")
        return
      }

      localStorage.setItem("access_token", access_token)
      localStorage.setItem("refresh_token", refresh_token)
      setMustChangePassword(must_change_password)

      const userRes = await api.get("/users/me")
      setUser(userRes.data)

      if (must_change_password) {
        navigate("/dashboard/profile")
      } else {
        navigate("/dashboard")
      }
    } catch (err: any) {
      const errorMsg = err.response?.data?.error || t("auth.loginFailed")
      setError(errorMsg)
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-4">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>{t("auth.signIn")}</CardTitle>
          <CardDescription>{t("auth.email")}</CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
            {error && (
              <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
                {error}
              </div>
            )}
            <div className="space-y-2">
              <Label htmlFor="email">{t("auth.email")}</Label>
              <Input
                id="email"
                type="email"
                placeholder="admin@example.com"
                {...register("email")}
              />
              {errors.email && <p className="text-xs text-destructive">{errors.email.message}</p>}
            </div>
            <div className="space-y-2">
              <Label htmlFor="password">{t("auth.password")}</Label>
              <Input
                id="password"
                type="password"
                placeholder="••••••"
                {...register("password")}
              />
              {errors.password && (
                <p className="text-xs text-destructive">{errors.password.message}</p>
              )}
            </div>
            <Button type="submit" className="w-full" disabled={isSubmitting}>
              {isSubmitting ? "..." : t("auth.signIn")}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  )
}
