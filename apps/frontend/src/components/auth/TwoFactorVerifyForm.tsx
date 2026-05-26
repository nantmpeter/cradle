import { useState } from "react"
import { useNavigate } from "react-router-dom"
import { api } from "@/lib/api"
import { useAuthStore } from "@/stores/authStore"
import { useVerify2FaLogin } from "@/hooks/useTwoFactor"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { useTranslation } from "react-i18next"

export function TwoFactorVerifyForm() {
  const [code, setCode] = useState("")
  const [recoveryCode, setRecoveryCode] = useState("")
  const [useRecovery, setUseRecovery] = useState(false)
  const [error, setError] = useState("")
  const navigate = useNavigate()
  const setUser = useAuthStore((s) => s.setUser)
  const verifyMutation = useVerify2FaLogin()
  const { t } = useTranslation()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError("")

    const tempToken = localStorage.getItem("temp_token")
    if (!tempToken) {
      setError(t("auth.sessionExpired"))
      navigate("/login")
      return
    }

    const submitCode = useRecovery ? recoveryCode : code

    try {
      const result = await verifyMutation.mutateAsync({
        temp_token: tempToken,
        code: submitCode,
      })

      localStorage.removeItem("temp_token")
      localStorage.setItem("access_token", result.access_token)
      localStorage.setItem("refresh_token", result.refresh_token)

      const userRes = await api.get("/users/me")
      setUser(userRes.data)

      if (userRes.data.must_change_password) {
        navigate("/dashboard/profile")
      } else {
        navigate("/dashboard")
      }
    } catch (err: any) {
      const errorMsg = err.response?.data?.error || "Verification failed"
      setError(errorMsg)
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-4">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>{t("auth.twoFactorTitle")}</CardTitle>
          <CardDescription>
            {useRecovery ? t("auth.recoveryCode") : t("auth.twoFactorSubtitle")}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            {error && (
              <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
                {error}
              </div>
            )}

            {useRecovery ? (
              <div className="space-y-2">
                <Label htmlFor="recovery-code">{t("auth.recoveryCode")}</Label>
                <Input
                  id="recovery-code"
                  type="text"
                  placeholder={t("auth.recoveryCode")}
                  value={recoveryCode}
                  onChange={(e) => setRecoveryCode(e.target.value)}
                  required
                />
              </div>
            ) : (
              <div className="space-y-2">
                <Label htmlFor="totp-code">{t("auth.twoFactorCode")}</Label>
                <Input
                  id="totp-code"
                  type="text"
                  placeholder="000000"
                  maxLength={6}
                  value={code}
                  onChange={(e) => setCode(e.target.value.replace(/\D/g, ""))}
                  className="text-center text-2xl tracking-widest"
                  required
                />
              </div>
            )}

            <Button type="submit" className="w-full" disabled={verifyMutation.isPending}>
              {verifyMutation.isPending ? "..." : t("auth.verify2fa")}
            </Button>

            <Button
              type="button"
              variant="ghost"
              className="w-full"
              onClick={() => {
                setUseRecovery(!useRecovery)
                setError("")
              }}
            >
              {useRecovery ? t("auth.twoFactorCode") : t("auth.useRecoveryCode")}
            </Button>

            <Button
              type="button"
              variant="link"
              className="w-full"
              onClick={() => {
                localStorage.removeItem("temp_token")
                navigate("/login")
              }}
            >
              {t("auth.backToLogin")}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  )
}
