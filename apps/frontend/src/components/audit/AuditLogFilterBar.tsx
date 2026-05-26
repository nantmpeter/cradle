import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Search, X } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AuditLogListParams } from "@/types/api"

interface AuditLogFilterBarProps {
  onFilterChange: (filters: Omit<AuditLogListParams, "page" | "per_page">) => void
}

export function AuditLogFilterBar({ onFilterChange }: AuditLogFilterBarProps) {
  const { t } = useTranslation()
  const [action, setAction] = useState<string>("")
  const [from, setFrom] = useState<string>("")
  const [to, setTo] = useState<string>("")

  const applyFilters = () => {
    const filters: Omit<AuditLogListParams, "page" | "per_page"> = {}
    if (action && action !== "all") filters.action = action
    if (from) filters.from = from
    if (to) filters.to = to
    onFilterChange(filters)
  }

  const clearFilters = () => {
    setAction("")
    setFrom("")
    setTo("")
    onFilterChange({})
  }

  const hasFilters = action || from || to

  const actionOptions = [
    { value: "auth.login", label: t("auditLogs.actions.login") },
    { value: "auth.login_failed", label: t("auditLogs.actions.loginFailed") },
    { value: "auth.logout", label: t("auditLogs.actions.logout") },
    { value: "auth.account_locked", label: t("auditLogs.actions.accountLocked") },
    { value: "user.create", label: t("auditLogs.actions.userCreated") },
    { value: "user.update", label: t("auditLogs.actions.userUpdated") },
    { value: "user.delete", label: t("auditLogs.actions.userDeleted") },
    { value: "user.status_change", label: t("auditLogs.actions.userStatusChanged") },
    { value: "user.password_reset", label: t("auditLogs.actions.passwordReset") },
    { value: "user.password_change", label: t("auditLogs.actions.passwordChanged") },
    { value: "role.create", label: t("auditLogs.actions.roleCreated") },
    { value: "role.update", label: t("auditLogs.actions.roleUpdated") },
    { value: "role.delete", label: t("auditLogs.actions.roleDeleted") },
    { value: "role.permissions_change", label: t("auditLogs.actions.permissionsChanged") },
  ]

  return (
    <div className="flex flex-wrap items-end gap-4">
      <div className="space-y-2">
        <Label className="text-xs">{t("auditLogs.action")}</Label>
        <Select value={action} onValueChange={setAction}>
          <SelectTrigger className="w-48">
            <SelectValue placeholder={t("auditLogs.allActions")} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t("auditLogs.allActions")}</SelectItem>
            {actionOptions.map((opt) => (
              <SelectItem key={opt.value} value={opt.value}>
                {opt.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <div className="space-y-2">
        <Label className="text-xs">{t("auditLogs.from")}</Label>
        <Input
          type="date"
          value={from}
          onChange={(e) => setFrom(e.target.value)}
          className="w-40"
        />
      </div>

      <div className="space-y-2">
        <Label className="text-xs">{t("auditLogs.to")}</Label>
        <Input
          type="date"
          value={to}
          onChange={(e) => setTo(e.target.value)}
          className="w-40"
        />
      </div>

      <Button size="sm" onClick={applyFilters}>
        <Search className="mr-2 h-4 w-4" />
        {t("auditLogs.filter")}
      </Button>

      {hasFilters && (
        <Button size="sm" variant="ghost" onClick={clearFilters}>
          <X className="mr-2 h-4 w-4" />
          {t("common.clear")}
        </Button>
      )}
    </div>
  )
}
