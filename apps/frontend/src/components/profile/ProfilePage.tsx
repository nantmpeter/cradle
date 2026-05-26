import { useState } from "react"
import { useForm } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { useAuthStore } from "@/stores/authStore"
import { useMe, useChangeMyPassword } from "@/hooks/useUsers"
import { validatePasswordStrength } from "@/types/api"
import { RoleBadge } from "@/components/shared/RoleBadge"
import { StatusBadge } from "@/components/shared/StatusBadge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Separator } from "@/components/ui/separator"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { AlertCircle, KeyRound, UserCircle, Shield, Camera } from "lucide-react"
import { toast } from "sonner"
import { api } from "@/lib/api"
import { useTranslation } from "react-i18next"
import { TwoFactorSetup } from "@/components/twoFactor/TwoFactorSetup"

const changePasswordSchema = z
  .object({
    current_password: z.string().min(1, "Current password is required"),
    new_password: z.string().min(8, "Password must be at least 8 characters"),
    confirm_password: z.string().min(1, "Please confirm your password"),
  })
  .refine((data) => data.new_password === data.confirm_password, {
    message: "Passwords do not match",
    path: ["confirm_password"],
  })

type ChangePasswordFormData = z.infer<typeof changePasswordSchema>

const updateNameSchema = z.object({
  name: z.string().min(1, "Name is required").max(100, "Name must be at most 100 characters"),
})

type UpdateNameFormData = z.infer<typeof updateNameSchema>

export function ProfilePage() {
  const { t } = useTranslation()
  const authUser = useAuthStore((s) => s.user)
  const updateUser = useAuthStore((s) => s.updateUser)
  const setMustChangePassword = useAuthStore((s) => s.setMustChangePassword)
  const mustChangePassword = useAuthStore((s) => s.mustChangePassword)
  const { data: profileUser } = useMe()

  const user = profileUser || authUser

  const [nameError, setNameError] = useState("")
  const [nameSuccess, setNameSuccess] = useState(false)

  const {
    register: registerName,
    handleSubmit: handleSubmitName,
    formState: { errors: nameErrors, isSubmitting: isNameSubmitting },
    reset: resetNameForm,
  } = useForm<UpdateNameFormData>({
    resolver: zodResolver(updateNameSchema),
    defaultValues: { name: user?.name || "" },
  })

  const onNameSubmit = async (data: UpdateNameFormData) => {
    setNameError("")
    setNameSuccess(false)
    try {
      const res = await api.put(`/users/${user?.id}`, { name: data.name })
      updateUser(res.data)
      setNameSuccess(true)
      toast.success(t("profile.nameUpdated"))
    } catch (err: any) {
      setNameError(err.response?.data?.error || t("profile.nameUpdateFailed"))
    }
  }

  const handleAvatarUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return
    if (file.size > 2 * 1024 * 1024) {
      toast.error(t("profile.avatarTooLarge"))
      return
    }
    const formData = new FormData()
    formData.append("file", file)
    try {
      const res = await api.post("/users/me/avatar", formData)
      updateUser({ avatar_url: res.data.avatar_url })
      toast.success(t("profile.avatarUpdated"))
    } catch (err: any) {
      toast.error(err.response?.data?.error || t("profile.avatarUpdateFailed"))
    }
  }

  const [pwError, setPwError] = useState("")
  const changeMyPassword = useChangeMyPassword()

  const {
    register: registerPw,
    handleSubmit: handleSubmitPw,
    formState: { errors: pwErrors, isSubmitting: isPwSubmitting },
    reset: resetPwForm,
  } = useForm<ChangePasswordFormData>({
    resolver: zodResolver(changePasswordSchema),
  })

  const onPasswordSubmit = async (data: ChangePasswordFormData) => {
    setPwError("")
    const strengthError = validatePasswordStrength(data.new_password)
    if (strengthError) {
      setPwError(strengthError)
      return
    }
    try {
      await changeMyPassword.mutateAsync({
        current_password: data.current_password,
        new_password: data.new_password,
      })
      setMustChangePassword(false)
      updateUser({ must_change_password: false })
      resetPwForm()
      toast.success(t("profile.passwordChanged"))
    } catch (err: any) {
      setPwError(err.response?.data?.error || t("profile.passwordChangeFailed"))
    }
  }

  if (!user) return null

  return (
    <div className="space-y-6 max-w-2xl">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">{t("profile.title")}</h1>
        <p className="text-muted-foreground">{t("profile.subtitle")}</p>
      </div>

      {mustChangePassword && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>
            {t("profile.mustChangePassword")}
          </AlertDescription>
        </Alert>
      )}

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <UserCircle className="h-5 w-5" />
            {t("profile.accountInfo")}
          </CardTitle>
          <CardDescription>{t("profile.accountInfoDesc")}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Avatar Upload */}
          <div className="flex items-center gap-4">
            <div className="relative group">
              {user.avatar_url ? (
                <img
                  src={user.avatar_url}
                  alt="Avatar"
                  className="h-16 w-16 rounded-full object-cover border"
                />
              ) : (
                <div className="flex h-16 w-16 items-center justify-center rounded-full bg-muted border">
                  <UserCircle className="h-10 w-10 text-muted-foreground" />
                </div>
              )}
              <label className="absolute inset-0 flex items-center justify-center rounded-full bg-black/40 opacity-0 group-hover:opacity-100 cursor-pointer transition-opacity">
                <Camera className="h-5 w-5 text-white" />
                <input
                  type="file"
                  accept="image/*"
                  className="hidden"
                  onChange={handleAvatarUpload}
                />
              </label>
            </div>
            <div className="text-sm text-muted-foreground">
              <p>{t("profile.avatarHint")}</p>
            </div>
          </div>
          <Separator />
          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-sm font-medium text-muted-foreground">{t("users.email")}</p>
              <p className="text-sm">{user.email}</p>
            </div>
            <div>
              <p className="text-sm font-medium text-muted-foreground">{t("users.role")}</p>
              <RoleBadge role={user.role} />
            </div>
            <div>
              <p className="text-sm font-medium text-muted-foreground">{t("common.status")}</p>
              <StatusBadge status={user.status} />
            </div>
            <div>
              <p className="text-sm font-medium text-muted-foreground">{t("common.createdAt")}</p>
              <p className="text-sm">
                {user.created_at ? new Date(user.created_at).toLocaleDateString() : "-"}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t("profile.displayName")}</CardTitle>
          <CardDescription>{t("profile.displayNameDesc")}</CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmitName(onNameSubmit)} className="space-y-4">
            {nameError && (
              <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
                {nameError}
              </div>
            )}
            {nameSuccess && (
              <div className="rounded-md bg-green-50 p-3 text-sm text-green-700 dark:bg-green-900/20 dark:text-green-400">
                {t("profile.nameUpdated")}
              </div>
            )}
            <div className="space-y-2">
              <Label htmlFor="name">{t("common.name")}</Label>
              <Input id="name" {...registerName("name")} />
              {nameErrors.name && (
                <p className="text-xs text-destructive">{nameErrors.name.message}</p>
              )}
            </div>
            <Button type="submit" disabled={isNameSubmitting}>
              {isNameSubmitting ? t("common.saving") : t("profile.updateName")}
            </Button>
          </form>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            {t("profile.twoFactor")}
          </CardTitle>
          <CardDescription>
            {t("profile.twoFactorDesc")}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <TwoFactorSetup />
        </CardContent>
      </Card>

      <Separator />

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <KeyRound className="h-5 w-5" />
            {t("profile.changePassword")}
          </CardTitle>
          <CardDescription>
            {t("profile.passwordHint")}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmitPw(onPasswordSubmit)} className="space-y-4">
            {pwError && (
              <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
                {pwError}
              </div>
            )}
            <div className="space-y-2">
              <Label htmlFor="current_password">{t("profile.currentPassword")}</Label>
              <Input id="current_password" type="password" {...registerPw("current_password")} />
              {pwErrors.current_password && (
                <p className="text-xs text-destructive">{pwErrors.current_password.message}</p>
              )}
            </div>
            <div className="space-y-2">
              <Label htmlFor="new_password">{t("profile.newPassword")}</Label>
              <Input id="new_password" type="password" {...registerPw("new_password")} />
              {pwErrors.new_password && (
                <p className="text-xs text-destructive">{pwErrors.new_password.message}</p>
              )}
            </div>
            <div className="space-y-2">
              <Label htmlFor="confirm_password">{t("profile.confirmPassword")}</Label>
              <Input id="confirm_password" type="password" {...registerPw("confirm_password")} />
              {pwErrors.confirm_password && (
                <p className="text-xs text-destructive">{pwErrors.confirm_password.message}</p>
              )}
            </div>
            <Button type="submit" disabled={isPwSubmitting}>
              {isPwSubmitting ? t("profile.changing") : t("profile.changePassword")}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  )
}
