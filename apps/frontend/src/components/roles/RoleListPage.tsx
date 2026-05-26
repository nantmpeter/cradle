import { useState } from "react"
import { useRoles, useDeleteRole } from "@/hooks/useRoles"
import { PermissionGuard } from "@/components/shared/PermissionGuard"
import { PermissionBadge } from "@/components/shared/PermissionBadge"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Plus, MoreHorizontal, Trash2, Pencil, Shield } from "lucide-react"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { toast } from "sonner"
import { useTranslation } from "react-i18next"
import { RoleCreateDialog } from "./RoleCreateDialog"
import { RoleEditDialog } from "./RoleEditDialog"
import { RoleDeleteDialog } from "./RoleDeleteDialog"
import type { RoleResponse } from "@/types/api"

export function RoleListPage() {
  const { t } = useTranslation()
  const { data: roles = [], isLoading } = useRoles()
  const deleteRole = useDeleteRole()

  const [createOpen, setCreateOpen] = useState(false)
  const [editOpen, setEditOpen] = useState(false)
  const [deleteOpen, setDeleteOpen] = useState(false)
  const [selectedRole, setSelectedRole] = useState<RoleResponse | null>(null)

  const handleDelete = async () => {
    if (!selectedRole) return
    try {
      await deleteRole.mutateAsync(selectedRole.id)
      toast.success(t("roles.deletedSuccess"))
      setDeleteOpen(false)
      setSelectedRole(null)
    } catch (err: any) {
      toast.error(err.response?.data?.error || t("roles.deleteFailed"))
    }
  }

  const openEditDialog = (role: RoleResponse) => {
    setSelectedRole(role)
    setEditOpen(true)
  }

  const openDeleteDialog = (role: RoleResponse) => {
    setSelectedRole(role)
    setDeleteOpen(true)
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t("roles.title")}</h1>
          <p className="text-muted-foreground">{t("roles.subtitle")}</p>
        </div>
        <PermissionGuard permission="roles:create">
          <Button onClick={() => setCreateOpen(true)}>
            <Plus className="mr-2 h-4 w-4" />
            {t("roles.createRole")}
          </Button>
        </PermissionGuard>
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t("roles.name")}</TableHead>
              <TableHead>{t("roles.description")}</TableHead>
              <TableHead>{t("roles.permissions")}</TableHead>
              <TableHead>{t("roles.type")}</TableHead>
              <TableHead>{t("roles.dataScope")}</TableHead>
              <TableHead>{t("common.updatedAt")}</TableHead>
              <PermissionGuard permission="roles:update">
                <TableHead className="w-12">{t("common.actions")}</TableHead>
              </PermissionGuard>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={6} className="text-center text-muted-foreground py-8">
                  {t("common.loading")}
                </TableCell>
              </TableRow>
            ) : roles.length === 0 ? (
              <TableRow>
                <TableCell colSpan={6} className="text-center text-muted-foreground py-8">
                  {t("common.noData")}
                </TableCell>
              </TableRow>
            ) : (
              roles.map((role) => (
                <TableRow key={role.id}>
                  <TableCell className="font-medium">{role.name}</TableCell>
                  <TableCell className="text-muted-foreground">
                    {role.description || "-"}
                  </TableCell>
                  <TableCell>
                    <div className="flex flex-wrap gap-1">
                      {role.permissions.length === 0 ? (
                        <span className="text-xs text-muted-foreground">{t("roles.noPermissions")}</span>
                      ) : (
                        role.permissions.slice(0, 3).map((perm) => (
                          <PermissionBadge key={perm} permission={perm} />
                        ))
                      )}
                      {role.permissions.length > 3 && (
                        <Badge variant="outline" className="text-xs">
                          +{role.permissions.length - 3} {t("roles.more")}
                        </Badge>
                      )}
                    </div>
                  </TableCell>
                  <TableCell>
                    {role.is_system ? (
                      <Badge variant="outline" className="bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-300">
                        <Shield className="mr-1 h-3 w-3" />
                        {t("roles.system")}
                      </Badge>
                    ) : (
                      <Badge variant="outline" className="bg-secondary text-secondary-foreground">
                        {t("roles.custom")}
                      </Badge>
                    )}
                  </TableCell>
                  <TableCell>
                    <Badge variant="outline" className="text-xs">
                      {t(`roles.dataScope_${role.data_scope || 'all'}`)}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-muted-foreground text-sm">
                    {role.updated_at
                      ? new Date(role.updated_at).toLocaleDateString()
                      : "-"}
                  </TableCell>
                  <PermissionGuard permission="roles:update">
                    <TableCell>
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button variant="ghost" size="icon" className="h-8 w-8">
                            <MoreHorizontal className="h-4 w-4" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuItem onClick={() => openEditDialog(role)}>
                            <Pencil className="mr-2 h-4 w-4" />
                            {t("common.edit")}
                          </DropdownMenuItem>
                          <DropdownMenuSeparator />
                          <DropdownMenuItem
                            onClick={() => openDeleteDialog(role)}
                            className="text-destructive focus:text-destructive"
                            disabled={role.is_system}
                          >
                            <Trash2 className="mr-2 h-4 w-4" />
                            {t("common.delete")}
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </TableCell>
                  </PermissionGuard>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      <RoleCreateDialog open={createOpen} onOpenChange={setCreateOpen} />
      <RoleEditDialog
        open={editOpen}
        onOpenChange={setEditOpen}
        role={selectedRole}
      />
      <RoleDeleteDialog
        open={deleteOpen}
        onOpenChange={setDeleteOpen}
        role={selectedRole}
        onConfirm={handleDelete}
        isPending={deleteRole.isPending}
      />
    </div>
  )
}
