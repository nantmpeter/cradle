import { useState, useCallback } from "react"
import { useUsers, useCreateUser, useUpdateUser, useDeleteUser, useToggleStatus, useResetPassword } from "@/hooks/useUsers"
import { useResetUser2Fa } from "@/hooks/useTwoFactor"
import { useRoles } from "@/hooks/useRoles"
import { useExportUsers } from "@/hooks/useExport"
import { useAuthStore } from "@/stores/authStore"
import { validatePasswordStrength } from "@/types/api"
import type { User, UserStatus, CreateUserRequest, UpdateUserRequest } from "@/types/api"
import { RoleBadge } from "@/components/shared/RoleBadge"
import { StatusBadge } from "@/components/shared/StatusBadge"
import { PermissionGuard } from "@/components/shared/PermissionGuard"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Badge } from "@/components/ui/badge"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Pagination } from "@/components/ui/pagination"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { MoreHorizontal, Plus, Search, KeyRound, Trash2, UserCog, Download, ShieldOff } from "lucide-react"
import { toast } from "sonner"
import { useTranslation } from "react-i18next"

export function UserListPage() {
  const { t } = useTranslation()
  const currentUser = useAuthStore((s) => s.user)

  // Filters
  const [search, setSearch] = useState("")
  const [roleFilter, setRoleFilter] = useState<string>("")
  const [statusFilter, setStatusFilter] = useState<string>("")
  const [page, setPage] = useState(1)
  const perPage = 20

  // Debounced search
  const [searchInput, setSearchInput] = useState("")
  const handleSearchChange = (value: string) => {
    setSearchInput(value)
    setPage(1)
    setSearch(value)
  }

  const { data, isLoading } = useUsers({
    page,
    per_page: perPage,
    search: search || undefined,
    role: roleFilter || undefined,
    status: statusFilter || undefined,
    sort_by: "created_at",
    sort_order: "desc",
  })

  // Roles for role selection dropdown
  const { data: roles = [] } = useRoles()

  // Mutations
  const createUser = useCreateUser()
  const updateUser = useUpdateUser()
  const deleteUser = useDeleteUser()
  const toggleStatus = useToggleStatus()
  const resetPassword = useResetPassword()
  const exportUsers = useExportUsers()
  const resetUser2Fa = useResetUser2Fa()

  // Dialog states
  const [createOpen, setCreateOpen] = useState(false)
  const [editOpen, setEditOpen] = useState(false)
  const [deleteOpen, setDeleteOpen] = useState(false)
  const [resetPwOpen, setResetPwOpen] = useState(false)
  const [reset2FaOpen, setReset2FaOpen] = useState(false)
  const [selectedUser, setSelectedUser] = useState<User | null>(null)

  // Form states
  const [formError, setFormError] = useState("")

  // Create form
  const [newEmail, setNewEmail] = useState("")
  const [newName, setNewName] = useState("")
  const [newPassword, setNewPassword] = useState("")
  const [newRoleId, setNewRoleId] = useState<string>("")

  // Edit form
  const [editName, setEditName] = useState("")
  const [editRoleId, setEditRoleId] = useState<string>("")

  // Reset password form
  const [resetPwValue, setResetPwValue] = useState("")

  const resetCreateForm = useCallback(() => {
    setNewEmail("")
    setNewName("")
    setNewPassword("")
    setNewRoleId("")
    setFormError("")
  }, [])

  const resetEditForm = useCallback(() => {
    setEditName("")
    setEditRoleId("")
    setFormError("")
  }, [])

  const handleCreateUser = async () => {
    setFormError("")
    const pwError = validatePasswordStrength(newPassword)
    if (pwError) {
      setFormError(pwError)
      return
    }
    if (!newRoleId) {
      setFormError("Please select a role")
      return
    }
    try {
      const req: CreateUserRequest = {
        email: newEmail,
        password: newPassword,
        name: newName || undefined,
        role_id: newRoleId,
      }
      await createUser.mutateAsync(req)
      toast.success(t("users.createdSuccess"))
      setCreateOpen(false)
      resetCreateForm()
    } catch (err: any) {
      setFormError(err.response?.data?.error || "Failed to create user")
    }
  }

  const handleEditUser = async () => {
    if (!selectedUser) return
    setFormError("")
    try {
      const req: UpdateUserRequest = {}
      if (editName !== (selectedUser.name || "")) req.name = editName
      if (editRoleId && editRoleId !== (selectedUser.role_id || "")) req.role_id = editRoleId
      await updateUser.mutateAsync({ id: selectedUser.id, data: req })
      toast.success(t("users.updatedSuccess"))
      setEditOpen(false)
      resetEditForm()
    } catch (err: any) {
      setFormError(err.response?.data?.error || "Failed to update user")
    }
  }

  const handleDeleteUser = async () => {
    if (!selectedUser) return
    try {
      await deleteUser.mutateAsync(selectedUser.id)
      toast.success(t("users.deletedSuccess"))
      setDeleteOpen(false)
      setSelectedUser(null)
    } catch (err: any) {
      toast.error(err.response?.data?.error || "Failed to delete user")
    }
  }

  const handleToggleStatus = async (user: User) => {
    const newStatus: UserStatus = user.status === "active" ? "disabled" : "active"
    try {
      await toggleStatus.mutateAsync({ id: user.id, data: { status: newStatus } })
      toast.success(`User ${newStatus === "active" ? t("users.enabled") : t("users.disabled")} ${t("common.success").toLowerCase()}`)
    } catch (err: any) {
      toast.error(err.response?.data?.error || "Failed to update status")
    }
  }

  const handleResetPassword = async () => {
    if (!selectedUser) return
    setFormError("")
    const pwError = validatePasswordStrength(resetPwValue)
    if (pwError) {
      setFormError(pwError)
      return
    }
    try {
      await resetPassword.mutateAsync({ id: selectedUser.id, data: { new_password: resetPwValue } })
      toast.success(t("users.passwordResetSuccess"))
      setResetPwOpen(false)
      setResetPwValue("")
      setFormError("")
    } catch (err: any) {
      setFormError(err.response?.data?.error || "Failed to reset password")
    }
  }

  const openEditDialog = (user: User) => {
    setSelectedUser(user)
    setEditName(user.name || "")
    setEditRoleId(user.role_id || "")
    setFormError("")
    setEditOpen(true)
  }

  const openDeleteDialog = (user: User) => {
    setSelectedUser(user)
    setDeleteOpen(true)
  }

  const openResetPwDialog = (user: User) => {
    setSelectedUser(user)
    setResetPwValue("")
    setFormError("")
    setResetPwOpen(true)
  }

  const users = data?.data || []
  const total = data?.pagination.total || 0

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t("users.title")}</h1>
          <p className="text-muted-foreground">{t("users.subtitle")}</p>
        </div>
        <div className="flex items-center gap-2">
          <PermissionGuard permission="export:users">
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => exportUsers.mutate({ format: "csv", filters: { search: search || undefined, role: roleFilter || undefined, status: statusFilter || undefined } })}
                disabled={exportUsers.isPending}
              >
                <Download className="mr-2 h-4 w-4" />
                {t("common.exportCsv")}
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => exportUsers.mutate({ format: "xlsx", filters: { search: search || undefined, role: roleFilter || undefined, status: statusFilter || undefined } })}
                disabled={exportUsers.isPending}
              >
                <Download className="mr-2 h-4 w-4" />
                {t("common.exportXlsx")}
              </Button>
            </div>
          </PermissionGuard>
          <PermissionGuard permission="users:create">
            <Button onClick={() => { resetCreateForm(); setCreateOpen(true) }}>
              <Plus className="mr-2 h-4 w-4" />
              {t("users.createUser")}
            </Button>
          </PermissionGuard>
        </div>
      </div>

      {/* Filters */}
      <div className="flex flex-wrap items-center gap-4">
        <div className="relative w-72">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder={t("users.searchPlaceholder")}
            value={searchInput}
            onChange={(e) => handleSearchChange(e.target.value)}
            className="pl-9"
          />
        </div>
        <Select value={roleFilter} onValueChange={(v) => { setRoleFilter(v === "all" ? "" : v); setPage(1) }}>
          <SelectTrigger className="w-36">
            <SelectValue placeholder={t("users.allRoles")} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t("users.allRoles")}</SelectItem>
            <SelectItem value="user">User</SelectItem>
            <SelectItem value="admin">Admin</SelectItem>
            <PermissionGuard permission="system:manage">
              <SelectItem value="superadmin">SuperAdmin</SelectItem>
            </PermissionGuard>
          </SelectContent>
        </Select>
        <Select value={statusFilter} onValueChange={(v) => { setStatusFilter(v === "all" ? "" : v); setPage(1) }}>
          <SelectTrigger className="w-36">
            <SelectValue placeholder={t("users.allStatus")} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t("users.allStatus")}</SelectItem>
            <SelectItem value="active">{t("users.active")}</SelectItem>
            <SelectItem value="disabled">{t("users.disabled")}</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Users Table */}
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t("users.email")}</TableHead>
              <TableHead>{t("common.name")}</TableHead>
              <TableHead>{t("users.role")}</TableHead>
              <TableHead>{t("common.status")}</TableHead>
              <TableHead>{t("common.createdAt")}</TableHead>
              <PermissionGuard permission="users:update">
                <TableHead className="w-12">Actions</TableHead>
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
            ) : users.length === 0 ? (
              <TableRow>
                <TableCell colSpan={6} className="text-center text-muted-foreground py-8">
                  {t("common.noData")}
                </TableCell>
              </TableRow>
            ) : (
              users.map((user) => (
                <TableRow key={user.id}>
                  <TableCell className="font-medium">{user.email}</TableCell>
                  <TableCell>{user.name || "-"}</TableCell>
                  <TableCell>
                    <RoleBadge role={user.role} />
                  </TableCell>
                  <TableCell>
                    <StatusBadge status={user.status} />
                  </TableCell>
                  <TableCell className="text-muted-foreground text-sm">
                    {user.created_at
                      ? new Date(user.created_at).toLocaleDateString()
                      : "-"}
                  </TableCell>
                  <PermissionGuard permission="users:update">
                    <TableCell>
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button variant="ghost" size="icon" className="h-8 w-8">
                            <MoreHorizontal className="h-4 w-4" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuItem onClick={() => openEditDialog(user)}>
                            <UserCog className="mr-2 h-4 w-4" />
                            {t("common.edit")}
                          </DropdownMenuItem>
                          <DropdownMenuItem onClick={() => handleToggleStatus(user)}>
                            {user.status === "active" ? t("users.disable") : t("users.enable")}
                          </DropdownMenuItem>
                          <DropdownMenuItem onClick={() => openResetPwDialog(user)}>
                            <KeyRound className="mr-2 h-4 w-4" />
                            {t("users.resetPassword")}
                          </DropdownMenuItem>
                          <PermissionGuard permission="2fa:manage">
                            <DropdownMenuItem
                              onClick={() => {
                                setSelectedUser(user)
                                setReset2FaOpen(true)
                              }}
                              disabled={!user.two_factor_enabled}
                            >
                            <ShieldOff className="mr-2 h-4 w-4" />
                            {t("users.reset2fa")}
                            </DropdownMenuItem>
                          </PermissionGuard>
                          <DropdownMenuSeparator />
                          <DropdownMenuItem
                            onClick={() => openDeleteDialog(user)}
                            className="text-destructive focus:text-destructive"
                            disabled={user.id === currentUser?.id}
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

      {/* Pagination */}
      <Pagination page={page} total={total} perPage={perPage} onPageChange={setPage} />

      {/* Create User Dialog */}
      <Dialog open={createOpen} onOpenChange={setCreateOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("users.createUserTitle")}</DialogTitle>
            <DialogDescription>{t("users.createUserDesc")}</DialogDescription>
          </DialogHeader>
          {formError && (
            <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
              {formError}
            </div>
          )}
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="create-email">{t("users.email")}</Label>
              <Input
                id="create-email"
                type="email"
                value={newEmail}
                onChange={(e) => setNewEmail(e.target.value)}
                placeholder="user@example.com"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="create-name">{t("common.name")}</Label>
              <Input
                id="create-name"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                placeholder="John Doe"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="create-password">{t("auth.password")}</Label>
              <Input
                id="create-password"
                type="password"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                placeholder="••••••••"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="create-role">{t("users.role")}</Label>
              <Select value={newRoleId} onValueChange={setNewRoleId}>
                <SelectTrigger>
                  <SelectValue placeholder={t("users.selectRole")} />
                </SelectTrigger>
                <SelectContent>
                  {roles.map((role) => (
                    <SelectItem key={role.id} value={role.id}>
                      {role.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCreateOpen(false)}>
              {t("common.cancel")}
            </Button>
            <Button
              onClick={handleCreateUser}
              disabled={createUser.isPending || !newEmail || !newPassword || !newRoleId}
            >
              {createUser.isPending ? t("users.creating") : t("users.create")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Edit User Dialog */}
      <Dialog open={editOpen} onOpenChange={setEditOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("users.editUser")}</DialogTitle>
            <DialogDescription>{t("users.editUserDesc")}</DialogDescription>
          </DialogHeader>
          {formError && (
            <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
              {formError}
            </div>
          )}
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="edit-name">{t("common.name")}</Label>
              <Input
                id="edit-name"
                value={editName}
                onChange={(e) => setEditName(e.target.value)}
              />
            </div>
            <PermissionGuard permission="users:update">
              <div className="space-y-2">
                <Label htmlFor="edit-role">{t("users.role")}</Label>
                <Select value={editRoleId} onValueChange={setEditRoleId}>
                  <SelectTrigger>
                    <SelectValue placeholder={t("users.selectRole")} />
                  </SelectTrigger>
                  <SelectContent>
                    {roles.map((role) => (
                      <SelectItem key={role.id} value={role.id}>
                        {role.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </PermissionGuard>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditOpen(false)}>
              {t("common.cancel")}
            </Button>
            <Button onClick={handleEditUser} disabled={updateUser.isPending}>
              {updateUser.isPending ? t("common.saving") : t("common.save")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete User Dialog */}
      <Dialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("users.deleteUser")}</DialogTitle>
            <DialogDescription>
              {t("users.deleteConfirm", { email: selectedUser?.email })}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteOpen(false)}>
              {t("common.cancel")}
            </Button>
            <Button variant="destructive" onClick={handleDeleteUser} disabled={deleteUser.isPending}>
              {deleteUser.isPending ? t("common.deleting") : t("common.delete")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Reset Password Dialog */}
      <Dialog open={resetPwOpen} onOpenChange={setResetPwOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("users.resetPassword")}</DialogTitle>
            <DialogDescription>
              {t("users.resetPasswordDesc", { email: selectedUser?.email })}
            </DialogDescription>
          </DialogHeader>
          {formError && (
            <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
              {formError}
            </div>
          )}
          <div className="space-y-2">
            <Label htmlFor="reset-pw">{t("users.newPassword")}</Label>
            <Input
              id="reset-pw"
              type="password"
              value={resetPwValue}
              onChange={(e) => setResetPwValue(e.target.value)}
              placeholder="••••••••"
            />
            <p className="text-xs text-muted-foreground">
              {t("users.passwordHint")}
            </p>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setResetPwOpen(false)}>
              {t("common.cancel")}
            </Button>
            <Button onClick={handleResetPassword} disabled={resetPassword.isPending || !resetPwValue}>
              {resetPassword.isPending ? t("users.resetting") : t("users.resetPassword")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Reset 2FA Dialog */}
      <Dialog open={reset2FaOpen} onOpenChange={setReset2FaOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("profile.reset2fa")}</DialogTitle>
            <DialogDescription>
              {t("users.reset2faDesc", { email: selectedUser?.email })}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setReset2FaOpen(false)}>
              {t("common.cancel")}
            </Button>
            <Button
              variant="destructive"
              disabled={resetUser2Fa.isPending}
              onClick={async () => {
                if (!selectedUser) return
                try {
                  await resetUser2Fa.mutateAsync(selectedUser.id)
                  toast.success(t("users.reset2faSuccess"))
                  setReset2FaOpen(false)
                } catch (err: any) {
                  toast.error(err.response?.data?.error || t("users.reset2faFailed"))
                }
              }}
            >
              {resetUser2Fa.isPending ? t("users.resetting") : t("users.reset2fa")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
