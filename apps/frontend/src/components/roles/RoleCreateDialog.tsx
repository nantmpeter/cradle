import { useState } from "react"
import { useCreateRole } from "@/hooks/useRoles"
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

interface RoleCreateDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function RoleCreateDialog({ open, onOpenChange }: RoleCreateDialogProps) {
  const { t } = useTranslation()
  const [name, setName] = useState("")
  const [description, setDescription] = useState("")
  const [dataScope, setDataScope] = useState("all")
  const [selectedPermissions, setSelectedPermissions] = useState<string[]>([])
  const [error, setError] = useState("")

  const createRole = useCreateRole()

  const resetForm = () => {
    setName("")
    setDescription("")
    setDataScope("all")
    setSelectedPermissions([])
    setError("")
  }

  const handleSubmit = async () => {
    setError("")
    if (!name.trim()) {
      setError(t("roles.nameRequired"))
      return
    }
    try {
      await createRole.mutateAsync({
        name: name.trim(),
        description: description.trim() || undefined,
        permission_ids: selectedPermissions,
        data_scope: dataScope,
      })
      toast.success(t("roles.createdSuccess"))
      onOpenChange(false)
      resetForm()
    } catch (err: any) {
      setError(err.response?.data?.error || t("roles.createFailed"))
    }
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(v) => {
        if (!v) resetForm()
        onOpenChange(v)
      }}
    >
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>{t("roles.createRole")}</DialogTitle>
          <DialogDescription>{t("roles.createRoleDesc")}</DialogDescription>
        </DialogHeader>
        {error && (
          <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
            {error}
          </div>
        )}
        <div className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="role-name">{t("roles.name")}</Label>
            <Input
              id="role-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. Editor"
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="role-description">{t("roles.description")}</Label>
            <Input
              id="role-description"
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
          <Button onClick={handleSubmit} disabled={createRole.isPending || !name.trim()}>
            {createRole.isPending ? t("common.creating") : t("roles.createRole")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
