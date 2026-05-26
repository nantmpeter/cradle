import { useState, useCallback } from "react"
import { useTranslation } from "react-i18next"
import { useMenus, useMenuTree, useCreateMenu, useUpdateMenu, useDeleteMenu } from "@/hooks/useMenus"
import { PermissionGuard } from "@/components/shared/PermissionGuard"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
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
import { MoreHorizontal, Plus, Trash2, Pencil, ChevronRight, ChevronDown } from "lucide-react"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { toast } from "sonner"
import type { MenuItem, MenuTreeNode, CreateMenuRequest, UpdateMenuRequest } from "@/types/menu"

// ─── Tree View Component ─────────────────────────────────────────────────────

function MenuTreeView({ nodes, depth = 0 }: { nodes: MenuTreeNode[]; depth?: number }) {
  const [expanded, setExpanded] = useState<Set<string>>(new Set())

  const toggleExpand = (id: string) => {
    setExpanded((prev) => {
      const next = new Set(prev)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }

  if (nodes.length === 0) return null

  return (
    <div>
      {nodes.map((node) => {
        const hasChildren = node.children && node.children.length > 0
        const isExpanded = expanded.has(node.id)
        return (
          <div key={node.id}>
            <div
              className="flex items-center gap-2 py-1.5 hover:bg-muted/50 rounded px-2 cursor-pointer"
              style={{ paddingLeft: `${depth * 24 + 8}px` }}
              onClick={() => hasChildren && toggleExpand(node.id)}
            >
              {hasChildren ? (
                isExpanded ? (
                  <ChevronDown className="h-4 w-4 text-muted-foreground shrink-0" />
                ) : (
                  <ChevronRight className="h-4 w-4 text-muted-foreground shrink-0" />
                )
              ) : (
                <span className="w-4 shrink-0" />
              )}
              <span className="font-medium text-sm">{node.title_label}</span>
              <span className="text-xs text-muted-foreground ml-2">({node.path})</span>
            </div>
            {hasChildren && isExpanded && (
              <MenuTreeView nodes={node.children} depth={depth + 1} />
            )}
          </div>
        )
      })}
    </div>
  )
}

// ─── Main Page ───────────────────────────────────────────────────────────────

export function MenusPage() {
  const { t } = useTranslation()

  const { data: menusData, isLoading } = useMenus()
  const { data: treeData } = useMenuTree()
  const createMenu = useCreateMenu()
  const updateMenu = useUpdateMenu()
  const deleteMenu = useDeleteMenu()

  const menus = menusData?.data || []
  const tree = treeData?.data || []

  // Dialog states
  const [createOpen, setCreateOpen] = useState(false)
  const [editOpen, setEditOpen] = useState(false)
  const [deleteOpen, setDeleteOpen] = useState(false)
  const [selectedMenu, setSelectedMenu] = useState<MenuItem | null>(null)
  const [formError, setFormError] = useState("")

  // Create form
  const [newTitleKey, setNewTitleKey] = useState("")
  const [newTitleLabel, setNewTitleLabel] = useState("")
  const [newPath, setNewPath] = useState("")
  const [newIcon, setNewIcon] = useState("")
  const [newSortOrder, setNewSortOrder] = useState(0)
  const [newParentId, setNewParentId] = useState<string>("")
  const [newPermissionId, setNewPermissionId] = useState("")

  // Edit form
  const [editTitleKey, setEditTitleKey] = useState("")
  const [editTitleLabel, setEditTitleLabel] = useState("")
  const [editPath, setEditPath] = useState("")
  const [editIcon, setEditIcon] = useState("")
  const [editSortOrder, setEditSortOrder] = useState(0)
  const [editParentId, setEditParentId] = useState<string>("")
  const [editPermissionId, setEditPermissionId] = useState("")
  const [editStatus, setEditStatus] = useState("active")

  const resetCreateForm = useCallback(() => {
    setNewTitleKey("")
    setNewTitleLabel("")
    setNewPath("")
    setNewIcon("")
    setNewSortOrder(0)
    setNewParentId("")
    setNewPermissionId("")
    setFormError("")
  }, [])

  const handleCreate = async () => {
    setFormError("")
    if (!newTitleKey || !newTitleLabel || !newPath) {
      setFormError("Title key, label, and path are required")
      return
    }
    try {
      const req: CreateMenuRequest = {
        title_key: newTitleKey,
        title_label: newTitleLabel,
        path: newPath,
        icon: newIcon || null,
        sort_order: newSortOrder,
        parent_id: newParentId || null,
        permission_id: newPermissionId || null,
      }
      await createMenu.mutateAsync(req)
      toast.success(t("common.success"))
      setCreateOpen(false)
      resetCreateForm()
    } catch (err: any) {
      setFormError(err.response?.data?.error || t("common.error"))
    }
  }

  const handleEdit = async () => {
    if (!selectedMenu) return
    setFormError("")
    try {
      const req: UpdateMenuRequest = {
        title_key: editTitleKey,
        title_label: editTitleLabel,
        path: editPath,
        icon: editIcon || null,
        sort_order: editSortOrder,
        parent_id: editParentId || null,
        permission_id: editPermissionId || null,
        status: editStatus,
      }
      await updateMenu.mutateAsync({ id: selectedMenu.id, data: req })
      toast.success(t("common.success"))
      setEditOpen(false)
      setSelectedMenu(null)
    } catch (err: any) {
      setFormError(err.response?.data?.error || t("common.error"))
    }
  }

  const handleDelete = async () => {
    if (!selectedMenu) return
    try {
      await deleteMenu.mutateAsync(selectedMenu.id)
      toast.success(t("common.success"))
      setDeleteOpen(false)
      setSelectedMenu(null)
    } catch (err: any) {
      toast.error(err.response?.data?.error || t("common.error"))
    }
  }

  const openEditDialog = (menu: MenuItem) => {
    setSelectedMenu(menu)
    setEditTitleKey(menu.title_key)
    setEditTitleLabel(menu.title_label)
    setEditPath(menu.path)
    setEditIcon(menu.icon || "")
    setEditSortOrder(menu.sort_order)
    setEditParentId(menu.parent_id || "")
    setEditPermissionId(menu.permission_id || "")
    setEditStatus(menu.status)
    setFormError("")
    setEditOpen(true)
  }

  const openDeleteDialog = (menu: MenuItem) => {
    setSelectedMenu(menu)
    setDeleteOpen(true)
  }

  // Build a map of menu IDs to titles for showing parent names
  const menuById = new Map(menus.map((m) => [m.id, m]))

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t("menus.title")}</h1>
          <p className="text-muted-foreground">{t("menus.title")}</p>
        </div>
        <PermissionGuard permission="menus:create">
          <Button onClick={() => { resetCreateForm(); setCreateOpen(true) }}>
            <Plus className="mr-2 h-4 w-4" />
            {t("menus.createMenu")}
          </Button>
        </PermissionGuard>
      </div>

      {/* Menu Table */}
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t("menus.titleLabel")}</TableHead>
              <TableHead>{t("menus.titleKey")}</TableHead>
              <TableHead>{t("menus.path")}</TableHead>
              <TableHead>{t("menus.icon")}</TableHead>
              <TableHead>{t("menus.sortOrder")}</TableHead>
              <TableHead>{t("menus.parent")}</TableHead>
              <TableHead>{t("common.status")}</TableHead>
              <TableHead className="w-12">{t("common.actions")}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={8} className="text-center text-muted-foreground py-8">
                  {t("common.loading")}
                </TableCell>
              </TableRow>
            ) : menus.length === 0 ? (
              <TableRow>
                <TableCell colSpan={8} className="text-center text-muted-foreground py-8">
                  {t("common.noData")}
                </TableCell>
              </TableRow>
            ) : (
              menus.map((menu) => (
                <TableRow key={menu.id}>
                  <TableCell className="font-medium">{menu.title_label}</TableCell>
                  <TableCell className="text-muted-foreground font-mono text-sm">
                    {menu.title_key}
                  </TableCell>
                  <TableCell className="text-muted-foreground text-sm">{menu.path}</TableCell>
                  <TableCell className="text-muted-foreground text-sm">{menu.icon || "-"}</TableCell>
                  <TableCell className="text-muted-foreground text-sm">{menu.sort_order}</TableCell>
                  <TableCell className="text-muted-foreground text-sm">
                    {menu.parent_id ? (menuById.get(menu.parent_id)?.title_label || menu.parent_id) : "-"}
                  </TableCell>
                  <TableCell>
                    <span
                      className={
                        menu.status === "active"
                          ? "text-green-600 dark:text-green-400"
                          : "text-red-600 dark:text-red-400"
                      }
                    >
                      {menu.status}
                    </span>
                  </TableCell>
                  <TableCell>
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="icon" className="h-8 w-8">
                          <MoreHorizontal className="h-4 w-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <PermissionGuard permission="menus:update">
                          <DropdownMenuItem onClick={() => openEditDialog(menu)}>
                            <Pencil className="mr-2 h-4 w-4" />
                            {t("common.edit")}
                          </DropdownMenuItem>
                        </PermissionGuard>
                        <PermissionGuard permission="menus:delete">
                          <>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem
                              onClick={() => openDeleteDialog(menu)}
                              className="text-destructive focus:text-destructive"
                            >
                              <Trash2 className="mr-2 h-4 w-4" />
                              {t("common.delete")}
                            </DropdownMenuItem>
                          </>
                        </PermissionGuard>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      {/* Menu Tree View */}
      {tree.length > 0 && (
        <div className="rounded-md border p-4">
          <h3 className="text-sm font-semibold mb-3">Menu Tree</h3>
          <MenuTreeView nodes={tree} />
        </div>
      )}

      {/* Create Menu Dialog */}
      <Dialog open={createOpen} onOpenChange={setCreateOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("menus.createMenu")}</DialogTitle>
            <DialogDescription>Create a new menu item</DialogDescription>
          </DialogHeader>
          {formError && (
            <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
              {formError}
            </div>
          )}
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="create-title-key">{t("menus.titleKey")}</Label>
              <Input
                id="create-title-key"
                value={newTitleKey}
                onChange={(e) => setNewTitleKey(e.target.value)}
                placeholder="nav.users"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="create-title-label">{t("menus.titleLabel")}</Label>
              <Input
                id="create-title-label"
                value={newTitleLabel}
                onChange={(e) => setNewTitleLabel(e.target.value)}
                placeholder="Users"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="create-path">{t("menus.path")}</Label>
              <Input
                id="create-path"
                value={newPath}
                onChange={(e) => setNewPath(e.target.value)}
                placeholder="/dashboard/users"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="create-icon">{t("menus.icon")}</Label>
              <Input
                id="create-icon"
                value={newIcon}
                onChange={(e) => setNewIcon(e.target.value)}
                placeholder="Users"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="create-sort">{t("menus.sortOrder")}</Label>
              <Input
                id="create-sort"
                type="number"
                value={newSortOrder}
                onChange={(e) => setNewSortOrder(Number(e.target.value))}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="create-parent">{t("menus.parent")}</Label>
              <Select value={newParentId} onValueChange={setNewParentId}>
                <SelectTrigger>
                  <SelectValue placeholder="None (top-level)" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">None (top-level)</SelectItem>
                  {menus.map((m) => (
                    <SelectItem key={m.id} value={m.id}>
                      {m.title_label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="create-permission">{t("menus.permission")}</Label>
              <Input
                id="create-permission"
                value={newPermissionId}
                onChange={(e) => setNewPermissionId(e.target.value)}
                placeholder="users:read"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCreateOpen(false)}>
              {t("common.cancel")}
            </Button>
            <Button
              onClick={handleCreate}
              disabled={createMenu.isPending || !newTitleKey || !newTitleLabel || !newPath}
            >
              {createMenu.isPending ? t("common.loading") : t("common.create")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Edit Menu Dialog */}
      <Dialog open={editOpen} onOpenChange={setEditOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("menus.editMenu")}</DialogTitle>
            <DialogDescription>Update menu item details</DialogDescription>
          </DialogHeader>
          {formError && (
            <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
              {formError}
            </div>
          )}
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="edit-title-key">{t("menus.titleKey")}</Label>
              <Input
                id="edit-title-key"
                value={editTitleKey}
                onChange={(e) => setEditTitleKey(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="edit-title-label">{t("menus.titleLabel")}</Label>
              <Input
                id="edit-title-label"
                value={editTitleLabel}
                onChange={(e) => setEditTitleLabel(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="edit-path">{t("menus.path")}</Label>
              <Input
                id="edit-path"
                value={editPath}
                onChange={(e) => setEditPath(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="edit-icon">{t("menus.icon")}</Label>
              <Input
                id="edit-icon"
                value={editIcon}
                onChange={(e) => setEditIcon(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="edit-sort">{t("menus.sortOrder")}</Label>
              <Input
                id="edit-sort"
                type="number"
                value={editSortOrder}
                onChange={(e) => setEditSortOrder(Number(e.target.value))}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="edit-parent">{t("menus.parent")}</Label>
              <Select value={editParentId} onValueChange={setEditParentId}>
                <SelectTrigger>
                  <SelectValue placeholder="None (top-level)" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">None (top-level)</SelectItem>
                  {menus
                    .filter((m) => m.id !== selectedMenu?.id)
                    .map((m) => (
                      <SelectItem key={m.id} value={m.id}>
                        {m.title_label}
                      </SelectItem>
                    ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="edit-permission">{t("menus.permission")}</Label>
              <Input
                id="edit-permission"
                value={editPermissionId}
                onChange={(e) => setEditPermissionId(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="edit-status">{t("common.status")}</Label>
              <Select value={editStatus} onValueChange={setEditStatus}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="active">Active</SelectItem>
                  <SelectItem value="inactive">Inactive</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditOpen(false)}>
              {t("common.cancel")}
            </Button>
            <Button onClick={handleEdit} disabled={updateMenu.isPending}>
              {updateMenu.isPending ? t("common.loading") : t("common.save")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Menu Dialog */}
      <Dialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("menus.deleteMenu")}</DialogTitle>
            <DialogDescription>{t("common.confirmDelete")}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteOpen(false)}>
              {t("common.cancel")}
            </Button>
            <Button variant="destructive" onClick={handleDelete} disabled={deleteMenu.isPending}>
              {deleteMenu.isPending ? t("common.loading") : t("common.delete")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
