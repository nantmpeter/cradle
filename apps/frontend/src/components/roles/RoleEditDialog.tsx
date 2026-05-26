import { useState, useEffect } from "react"
import { useUpdateRole, useUpdateRolePermissions } from "@/hooks/useRoles"
import { PermissionSelector } from "@/components/shared/PermissionSelector"
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
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { toast } from "sonner"
import { useTranslation } from "react-i18next"
import type { RoleResponse } from "@/types/api"

interface RoleEditDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  role: RoleResponse | null
}

export function RoleEditDialog({ open, onOpenChange, role }: RoleEditDialogProps) {
  const { t } = useTranslation()
  const [name, setName] = useState("")
  const [description, setDescription] = useState("")
  const [dataScope, setDataScope] = useState("all")
  const [selectedPermissions, setSelectedPermissions] = useState<string[]>([])
  const [error, setError] = useState("")

  const updateRole = useUpdateRole()
  const updateRolePermissions = useUpdateRolePermissions()

  useEffect(() => {
    if (role) {
      setName(role.name)
      setDescription(role.description || "")
      setDataScope(role.data_scope || "all")
      setSelectedPermissions([])
    }
  }, [role])

  const handleSubmit = async () => {
    if (!role) return
    setError("")
    if (!name.trim()) {
      setError(t("roles.nameRequired"))
      return
    }
    try {
      await updateRole.mutateAsync({
        id: role.id,
        data: {
          name: name.trim() !== role.name ? name.trim() : undefined,
          description: description.trim() !== (role.description || "") ? description.trim() : undefined,
          data_scope: dataScope !== (role.data_scope || "all") ? dataScope : undefined,
        },
      })
      if (selectedPermissions.length > 0) {
        await updateRolePermissions.mutateAsync({
          id: role.id,
          data: { permission_ids: selectedPermissions },
        })
      }
      toast.success(t("roles.updatedSuccess"))
      onOpenChange(false)
    } catch (err: any) {
      setError(err.response?.data?.error || t("roles.updateFailed"))
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>{t("roles.editRole")}</DialogTitle>
          <DialogDescription>
            {t("roles.editRoleDesc")}
            {role?.is_system && (
              <span className="ml-1 text-amber-600 dark:text-amber-400">
                ({t("roles.systemRoleNote")})
              </span>
            )}
          </DialogDescription>
        </DialogHeader>
        {error && (
          <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
            {error}
          </div>
        )}
        <div className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="edit-role-name">{t("roles.name")}</Label>
            <Input
              id="edit-role-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              disabled={role?.is_system}
              placeholder={t("roles.name")}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="edit-role-description">{t("roles.description")}</Label>
            <Input
              id="edit-role-description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder={t("roles.optionalDesc")}
            />
          </div>
          <div className="space-y-2">
            <Label>{t("roles.dataScope")}</Label>
            <Select value={dataScope} onValueChange={setDataScope}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">{t("roles.dataScopeAll")}</SelectItem>
                <SelectItem value="department_and_sub">{t("roles.dataScopeDeptSub")}</SelectItem>
                <SelectItem value="department">{t("roles.dataScopeDept")}</SelectItem>
                <SelectItem value="self">{t("roles.dataScopeSelf")}</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>{t("roles.permissions")}</Label>
            <div className="max-h-64 overflow-y-auto rounded-md border p-2">
              <PermissionSelector
                selectedIds={selectedPermissions}
                onChange={setSelectedPermissions}
              />
            </div>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t("common.cancel")}
          </Button>
          <Button
            onClick={handleSubmit}
            disabled={updateRole.isPending || updateRolePermissions.isPending}
          >
            {updateRole.isPending || updateRolePermissions.isPending ? t("common.saving") : t("common.save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
