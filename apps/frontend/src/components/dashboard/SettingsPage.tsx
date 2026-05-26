import { useDashboardSettings } from "@/hooks/useDashboard"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Separator } from "@/components/ui/separator"
import { Button } from "@/components/ui/button"
import { Server, Database, UserCircle, ShieldCheck, ShieldOff, Palette } from "lucide-react"
import { useTranslation } from "react-i18next"
import { useState, useEffect } from "react"
import { themeColors, applyTheme } from "@/lib/theme"

/** Available theme accent colors — oklch values for shadcn CSS vars */

function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const parts: string[] = []
  if (days > 0) parts.push(`${days}d`)
  if (hours > 0) parts.push(`${hours}h`)
  if (minutes > 0 || parts.length === 0) parts.push(`${minutes}m`)
  return parts.join(" ")
}

function formatDateTime(isoString: string): string {
  return new Date(isoString).toLocaleString()
}

export function SettingsPage() {
  const { t } = useTranslation()
  const { data, isLoading, error } = useDashboardSettings()

  // Restore saved theme accent on mount
  useEffect(() => {
    const saved = localStorage.getItem("theme-accent")
    if (saved) {
      const color = themeColors.find((c) => c.name === saved)
      if (color) applyTheme(color)
    }
  }, [])

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t("settings.title")}</h1>
          <p className="text-muted-foreground">{t("common.loading")}</p>
        </div>
        <div className="grid gap-4">
          {[1, 2, 3].map((i) => (
            <Card key={i}>
              <CardContent className="p-6">
                <div className="h-32 animate-pulse rounded bg-muted" />
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t("settings.title")}</h1>
          <p className="text-muted-foreground">{t("settings.subtitle")}</p>
        </div>
        <Card>
          <CardContent className="p-6">
            <p className="text-destructive">
              {t("settings.loadError")}: {error instanceof Error ? error.message : t("settings.unknownError")}
            </p>
          </CardContent>
        </Card>
      </div>
    )
  }

  if (!data) return null

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">{t("settings.title")}</h1>
        <p className="text-muted-foreground">{t("settings.subtitle")}</p>
      </div>

      <div className="grid gap-6">
        <Card>
          <CardHeader className="flex flex-row items-center gap-3">
            <Server className="h-5 w-5 text-muted-foreground" />
            <CardTitle className="text-base">{t("settings.systemInfo")}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="grid grid-cols-3 gap-4 text-sm">
              <div>
                <span className="text-muted-foreground">{t("settings.systemName")}</span>
                <p className="font-medium">{data.system.name}</p>
              </div>
              <div>
                <span className="text-muted-foreground">{t("settings.version")}</span>
                <p className="font-medium">v{data.system.version}</p>
              </div>
              <div>
                <span className="text-muted-foreground">{t("settings.uptime")}</span>
                <p className="font-medium">{formatUptime(data.system.uptime_seconds)}</p>
              </div>
            </div>
          </CardContent>
        </Card>

        {data.database !== null && (
          <Card>
            <CardHeader className="flex flex-row items-center gap-3">
              <Database className="h-5 w-5 text-muted-foreground" />
              <CardTitle className="text-base">{t("settings.databaseStatus")}</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="text-muted-foreground">{t("settings.connectionStatus")}</span>
                  <div className="font-medium">
                    <Badge variant={data.database.status === "connected" ? "default" : "destructive"}>
                      {data.database.status === "connected" ? t("settings.connected") : data.database.status}
                    </Badge>
                  </div>
                </div>
                <div>
                  <span className="text-muted-foreground">{t("settings.maxConnections")}</span>
                  <p className="font-medium">{data.database.max_connections}</p>
                </div>
              </div>
            </CardContent>
          </Card>
        )}

        <Card>
          <CardHeader className="flex flex-row items-center gap-3">
            <UserCircle className="h-5 w-5 text-muted-foreground" />
            <CardTitle className="text-base">{t("settings.currentUser")}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span className="text-muted-foreground">{t("users.email")}</span>
                <p className="font-medium">{data.current_user.email}</p>
              </div>
              <div>
                <span className="text-muted-foreground">{t("common.name")}</span>
                <p className="font-medium">{data.current_user.name || t("settings.notSet")}</p>
              </div>
              <div>
                <span className="text-muted-foreground">{t("users.role")}</span>
                <p className="font-medium">
                  <Badge variant="outline">{data.current_user.role}</Badge>
                </p>
              </div>
              <div>
                <span className="text-muted-foreground">{t("common.createdAt")}</span>
                <p className="font-medium">{formatDateTime(data.current_user.created_at)}</p>
              </div>
              <div>
                <span className="text-muted-foreground">{t("profile.twoFactor")}</span>
                <p className="font-medium flex items-center gap-1.5">
                  {data.current_user.two_factor_enabled ? (
                    <>
                      <ShieldCheck className="h-4 w-4 text-green-500" />
                      <Badge variant="default" className="bg-green-500">{t("profile.enable2fa")}</Badge>
                    </>
                  ) : (
                    <>
                      <ShieldOff className="h-4 w-4 text-muted-foreground" />
                      <Badge variant="outline">{t("settings.notSet")}</Badge>
                    </>
                  )}
                </p>
              </div>
            </div>

            <Separator />

            <div className="space-y-2">
              <span className="text-sm text-muted-foreground">{t("settings.permissions")}</span>
              <div className="flex flex-wrap gap-1.5">
                {data.current_user.permissions.length > 0 ? (
                  data.current_user.permissions.map((perm) => (
                    <Badge key={perm} variant="secondary" className="text-xs">
                      {perm}
                    </Badge>
                  ))
                ) : (
                  <span className="text-sm text-muted-foreground">{t("settings.noPermissions")}</span>
                )}
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center gap-3">
            <Palette className="h-5 w-5 text-muted-foreground" />
            <CardTitle className="text-base">{t("settings.themeCustomizer")}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <p className="text-sm text-muted-foreground">{t("settings.themeDesc")}</p>
            <div className="flex flex-wrap gap-3">
              {themeColors.map((color) => (
                <Button
                  key={color.name}
                  variant="outline"
                  className="flex items-center gap-2"
                  onClick={() => applyTheme(color)}
                >
                  <span
                    className="h-4 w-4 rounded-full border"
                    style={{ backgroundColor: color.preview }}
                  />
                  {color.name}
                </Button>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
