import { useState, useEffect } from "react"
import { useSetup2Fa, useEnable2Fa, useDisable2Fa } from "@/hooks/useTwoFactor"
import { useAuthStore } from "@/stores/authStore"
import { useMe } from "@/hooks/useUsers"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { Shield, ShieldCheck, ShieldOff, Copy, Check } from "lucide-react"
import { toast } from "sonner"
import { useTranslation } from "react-i18next"

type Step = "idle" | "setup" | "verify" | "enabled"

export function TwoFactorSetup() {
  const { t } = useTranslation()
  const authUser = useAuthStore((s) => s.user)
  const { data: profileUser } = useMe()
  const user = profileUser || authUser
  const [step, setStep] = useState<Step>("idle")
  const [setupData, setSetupData] = useState<{ qr_code_base64: string; secret: string; recovery_codes: string[] } | null>(null)
  const [enableCode, setEnableCode] = useState("")
  const [disablePassword, setDisablePassword] = useState("")
  const [disableCode, setDisableCode] = useState("")
  const [codesCopied, setCodesCopied] = useState(false)
  const [disableDialogOpen, setDisableDialogOpen] = useState(false)

  // Sync step when user data loads (e.g. after page refresh)
  useEffect(() => {
    if (user?.two_factor_enabled && step === "idle") {
      setStep("enabled")
    }
  }, [user?.two_factor_enabled])

  const setupMutation = useSetup2Fa()
  const enableMutation = useEnable2Fa()
  const disableMutation = useDisable2Fa()

  const handleSetup = async () => {
    try {
      const data = await setupMutation.mutateAsync()
      setSetupData(data)
      setStep("setup")
    } catch (err: any) {
      toast.error(err.response?.data?.error || t("profile.setupFailed"))
    }
  }

  const handleEnable = async () => {
    try {
      await enableMutation.mutateAsync({ code: enableCode })
      toast.success(t("profile.enabledSuccess"))
      setStep("enabled")
      setEnableCode("")
    } catch (err: any) {
      toast.error(err.response?.data?.error || t("profile.invalidCode"))
    }
  }

  const handleDisable = async () => {
    try {
      await disableMutation.mutateAsync({
        password: disablePassword || undefined,
        code: disableCode || undefined,
      })
      toast.success(t("profile.disabledSuccess"))
      setStep("idle")
      setDisablePassword("")
      setDisableCode("")
      setDisableDialogOpen(false)
    } catch (err: any) {
      toast.error(err.response?.data?.error || t("profile.disableFailed"))
    }
  }

  const copyCodes = () => {
    if (setupData) {
      navigator.clipboard.writeText(setupData.recovery_codes.join("\n"))
      setCodesCopied(true)
      toast.success(t("profile.codesCopied"))
      setTimeout(() => setCodesCopied(false), 2000)
    }
  }

  if (step === "enabled") {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <ShieldCheck className="h-5 w-5 text-green-500" />
            {t("profile.twoFactor")}
          </CardTitle>
          <CardDescription>{t("profile.twoFactorEnabledDesc")}</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2 mb-4">
            <Badge variant="default" className="bg-green-500">{t("profile.enable2fa")}</Badge>
          </div>
          <Dialog open={disableDialogOpen} onOpenChange={setDisableDialogOpen}>
            <DialogTrigger asChild>
              <Button variant="destructive" size="sm">
                <ShieldOff className="mr-2 h-4 w-4" />
                {t("profile.disable2fa")}
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>{t("profile.disable2fa")}</DialogTitle>
                <DialogDescription>
                  {t("profile.disable2faDesc")}
                </DialogDescription>
              </DialogHeader>
              <div className="space-y-4">
                <div className="space-y-2">
                  <Label>{t("auth.password")}</Label>
                  <Input
                    type="password"
                    placeholder={t("auth.password")}
                    value={disablePassword}
                    onChange={(e) => setDisablePassword(e.target.value)}
                  />
                </div>
                <div className="text-center text-sm text-muted-foreground">{t("common.or")}</div>
                <div className="space-y-2">
                  <Label>{t("profile.totpCode")}</Label>
                  <Input
                    type="text"
                    placeholder="000000"
                    maxLength={6}
                    value={disableCode}
                    onChange={(e) => setDisableCode(e.target.value.replace(/\D/g, ""))}
                  />
                </div>
              </div>
              <DialogFooter>
                <Button variant="outline" onClick={() => setDisableDialogOpen(false)}>{t("common.cancel")}</Button>
                <Button variant="destructive" onClick={handleDisable} disabled={disableMutation.isPending}>
                  {disableMutation.isPending ? t("profile.disabling") : t("profile.disable2fa")}
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </CardContent>
      </Card>
    )
  }

  if (step === "setup" && setupData) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            {t("profile.setup2fa")}
          </CardTitle>
          <CardDescription>{t("profile.scanQrCode")}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="flex justify-center">
            <img
              src={setupData.qr_code_base64}
              alt="2FA QR Code"
              className="rounded-lg border bg-white p-2"
              width={200}
              height={200}
            />
          </div>

          <div className="space-y-2">
            <Label className="text-xs text-muted-foreground">{t("profile.manualEntryKey")}</Label>
            <code className="block rounded bg-muted p-2 text-xs break-all">{setupData.secret}</code>
          </div>

          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label className="text-sm font-medium">{t("profile.recoveryCodes")}</Label>
              <Button variant="ghost" size="sm" onClick={copyCodes}>
                {codesCopied ? <Check className="mr-1 h-3 w-3" /> : <Copy className="mr-1 h-3 w-3" />}
                {codesCopied ? t("profile.copied") : t("profile.copy")}
              </Button>
            </div>
            <div className="grid grid-cols-2 gap-2 rounded-lg border bg-muted/50 p-3">
              {setupData.recovery_codes.map((code, i) => (
                <code key={i} className="text-center text-sm font-mono">{code}</code>
              ))}
            </div>
            <p className="text-xs text-destructive">
              {t("profile.recoveryCodesWarning")}
            </p>
          </div>

          {step === "setup" && (
            <div className="space-y-4 border-t pt-4">
              <Label>{t("profile.enterCodeToVerify")}</Label>
              <div className="flex gap-2">
                <Input
                  type="text"
                  placeholder="000000"
                  maxLength={6}
                  value={enableCode}
                  onChange={(e) => setEnableCode(e.target.value.replace(/\D/g, ""))}
                  className="text-center text-xl tracking-widest"
                />
                <Button onClick={handleEnable} disabled={enableCode.length !== 6 || enableMutation.isPending}>
                  {enableMutation.isPending ? t("profile.verifying") : t("profile.enable2fa")}
                </Button>
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    )
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Shield className="h-5 w-5" />
          {t("profile.twoFactor")}
        </CardTitle>
        <CardDescription>{t("profile.twoFactorDesc")}</CardDescription>
      </CardHeader>
      <CardContent>
        <p className="text-sm text-muted-foreground mb-4">
          {t("profile.twoFactorExplanation")}
        </p>
        <Button onClick={handleSetup} disabled={setupMutation.isPending}>
          {setupMutation.isPending ? t("profile.settingUp") : t("profile.setup2fa")}
        </Button>
      </CardContent>
    </Card>
  )
}
