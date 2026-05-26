import { useState } from "react"
import { useDepartments, useCreateDepartment, useUpdateDepartment, useDeleteDepartment } from "@/hooks/useDepartments"
import { PermissionGuard } from "@/components/shared/PermissionGuard"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
} from "@/components/ui/dialog"
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select"
import { Plus, Building2, ChevronRight, ChevronDown, Trash2, Edit } from "lucide-react"
import { toast } from "sonner"
import { useTranslation } from "react-i18next"
import type { DepartmentTreeNode, CreateDepartmentRequest, UpdateDepartmentRequest } from "@/types/department"

function DepartmentTree({
  nodes, level, onSelect, selectedId,
}: {
  nodes: DepartmentTreeNode[]
  level: number
  onSelect: (node: DepartmentTreeNode) => void
  selectedId: string | null
}) {
  const [expanded, setExpanded] = useState<Set<string>>(new Set())

  const toggle = (id: string) => {
    setExpanded((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  return (
    <>
      {nodes.map((node) => (
        <div key={node.id}>
          <div
            className={`flex items-center gap-1 py-1.5 px-2 cursor-pointer rounded-md hover:bg-muted/50 ${
              selectedId === node.id ? "bg-muted" : ""
            }`}
            style={{ paddingLeft: `${level * 20 + 8}px` }}
            onClick={() => onSelect(node)}
          >
            {node.children.length > 0 ? (
              <button
                className="h-4 w-4 shrink-0"
                onClick={(e) => { e.stopPropagation(); toggle(node.id) }}
              >
                {expanded.has(node.id) ? (
                  <ChevronDown className="h-4 w-4" />
                ) : (
                  <ChevronRight className="h-4 w-4" />
                )}
              </button>
            ) : (
              <span className="w-4 shrink-0" />
            )}
            <Building2 className="h-4 w-4 shrink-0 text-muted-foreground" />
            <span className="ml-1 text-sm truncate">{node.name}</span>
            <Badge variant="outline" className="ml-auto text-xs">
              {node.code}
            </Badge>
          </div>
          {expanded.has(node.id) && node.children.length > 0 && (
            <DepartmentTree nodes={node.children} level={level + 1} onSelect={onSelect} selectedId={selectedId} />
          )}
        </div>
      ))}
    </>
  )
}

export function DepartmentsPage() {
  const { t } = useTranslation()
  const { data, isLoading } = useDepartments()
  const createDept = useCreateDepartment()
  const updateDept = useUpdateDepartment()
  const deleteDept = useDeleteDepartment()

  const [selectedNode, setSelectedNode] = useState<DepartmentTreeNode | null>(null)
  const [createOpen, setCreateOpen] = useState(false)
  const [editOpen, setEditOpen] = useState(false)
  const [deleteOpen, setDeleteOpen] = useState(false)

  // Form state
  const [formName, setFormName] = useState("")
  const [formCode, setFormCode] = useState("")
  const [formParentId, setFormParentId] = useState<string>("")
  const [formSortOrder, setFormSortOrder] = useState(0)
  const [formStatus, setFormStatus] = useState("active")
  const [formLeader, setFormLeader] = useState("")

  const resetForm = () => {
    setFormName("")
    setFormCode("")
    setFormParentId("")
    setFormSortOrder(0)
    setFormStatus("active")
    setFormLeader("")
  }

  const handleCreate = async () => {
    try {
      const req: CreateDepartmentRequest = {
        name: formName,
        code: formCode,
        parent_id: formParentId || null,
        sort_order: formSortOrder,
        status: formStatus,
        leader: formLeader || undefined,
      }
      await createDept.mutateAsync(req)
      toast.success(t("departments.createdSuccess"))
      setCreateOpen(false)
      resetForm()
    } catch (err: any) {
      toast.error(err.response?.data?.error || "Failed to create department")
    }
  }

  const handleEdit = async () => {
    if (!selectedNode) return
    try {
      const req: UpdateDepartmentRequest = {
        name: formName,
        sort_order: formSortOrder,
        status: formStatus,
        leader: formLeader || undefined,
      }
      await updateDept.mutateAsync({ id: selectedNode.id, data: req })
      toast.success(t("departments.updatedSuccess"))
      setEditOpen(false)
    } catch (err: any) {
      toast.error(err.response?.data?.error || "Failed to update department")
    }
  }

  const handleDelete = async () => {
    if (!selectedNode) return
    try {
      await deleteDept.mutateAsync(selectedNode.id)
      toast.success(t("departments.deletedSuccess"))
      setDeleteOpen(false)
      setSelectedNode(null)
    } catch (err: any) {
      toast.error(err.response?.data?.error || "Failed to delete department")
    }
  }

  const openEditDialog = (node: DepartmentTreeNode) => {
    setSelectedNode(node)
    setFormName(node.name)
    setFormCode(node.code)
    setFormSortOrder(node.sort_order)
    setFormStatus(node.status)
    setFormLeader(node.leader || "")
    setEditOpen(true)
  }

  const tree = data?.tree || []

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t("departments.title")}</h1>
          <p className="text-muted-foreground">{t("departments.subtitle")}</p>
        </div>
        <PermissionGuard permission="departments:create">
          <Button onClick={() => { resetForm(); setCreateOpen(true) }}>
            <Plus className="mr-2 h-4 w-4" />
            {t("departments.create")}
          </Button>
        </PermissionGuard>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Tree */}
        <Card className="lg:col-span-1">
          <CardHeader>
            <CardTitle className="text-base">{t("departments.treeTitle")}</CardTitle>
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <p className="text-sm text-muted-foreground">{t("common.loading")}</p>
            ) : tree.length === 0 ? (
              <p className="text-sm text-muted-foreground">{t("common.noData")}</p>
            ) : (
              <DepartmentTree
                nodes={tree}
                level={0}
                onSelect={setSelectedNode}
                selectedId={selectedNode?.id ?? null}
              />
            )}
          </CardContent>
        </Card>

        {/* Detail */}
        <Card className="lg:col-span-2">
          <CardHeader className="flex flex-row items-center justify-between">
            <CardTitle className="text-base">
              {selectedNode ? selectedNode.name : t("departments.selectDepartment")}
            </CardTitle>
            {selectedNode && (
              <div className="flex gap-2">
                <PermissionGuard permission="departments:update">
                  <Button variant="outline" size="sm" onClick={() => openEditDialog(selectedNode)}>
                    <Edit className="mr-2 h-4 w-4" />
                    {t("common.edit")}
                  </Button>
                </PermissionGuard>
                <PermissionGuard permission="departments:delete">
                  <Button
                    variant="destructive"
                    size="sm"
                    onClick={() => setDeleteOpen(true)}
                  >
                    <Trash2 className="mr-2 h-4 w-4" />
                    {t("common.delete")}
                  </Button>
                </PermissionGuard>
              </div>
            )}
          </CardHeader>
          <CardContent>
            {selectedNode ? (
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="text-muted-foreground">{t("departments.code")}</span>
                  <p className="font-medium">{selectedNode.code}</p>
                </div>
                <div>
                  <span className="text-muted-foreground">{t("departments.sortOrder")}</span>
                  <p className="font-medium">{selectedNode.sort_order}</p>
                </div>
                <div>
                  <span className="text-muted-foreground">{t("common.status")}</span>
                  <p>
                    <Badge variant={selectedNode.status === "active" ? "default" : "secondary"}>
                      {selectedNode.status}
                    </Badge>
                  </p>
                </div>
                <div>
                  <span className="text-muted-foreground">{t("departments.leader")}</span>
                  <p className="font-medium">{selectedNode.leader || "-"}</p>
                </div>
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">{t("departments.selectHint")}</p>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Create Dialog */}
      <Dialog open={createOpen} onOpenChange={setCreateOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("departments.createTitle")}</DialogTitle>
            <DialogDescription>{t("departments.createDesc")}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label>{t("departments.name")}</Label>
              <Input value={formName} onChange={(e) => setFormName(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label>{t("departments.code")}</Label>
              <Input value={formCode} onChange={(e) => setFormCode(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label>{t("departments.parent")}</Label>
              <Select value={formParentId} onValueChange={setFormParentId}>
                <SelectTrigger><SelectValue placeholder={t("departments.noParent")} /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">{t("departments.noParent")}</SelectItem>
                  {(data?.departments || []).map((d) => (
                    <SelectItem key={d.id} value={d.id}>{d.name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>{t("departments.sortOrder")}</Label>
                <Input type="number" value={formSortOrder} onChange={(e) => setFormSortOrder(Number(e.target.value))} />
              </div>
              <div className="space-y-2">
                <Label>{t("departments.leader")}</Label>
                <Input value={formLeader} onChange={(e) => setFormLeader(e.target.value)} />
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCreateOpen(false)}>{t("common.cancel")}</Button>
            <Button onClick={handleCreate} disabled={createDept.isPending || !formName || !formCode}>
              {createDept.isPending ? t("common.creating") : t("common.create")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Edit Dialog */}
      <Dialog open={editOpen} onOpenChange={setEditOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("departments.editTitle")}</DialogTitle>
            <DialogDescription>{t("departments.editDesc")}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label>{t("departments.name")}</Label>
              <Input value={formName} onChange={(e) => setFormName(e.target.value)} />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>{t("departments.sortOrder")}</Label>
                <Input type="number" value={formSortOrder} onChange={(e) => setFormSortOrder(Number(e.target.value))} />
              </div>
              <div className="space-y-2">
                <Label>{t("departments.leader")}</Label>
                <Input value={formLeader} onChange={(e) => setFormLeader(e.target.value)} />
              </div>
            </div>
            <div className="space-y-2">
              <Label>{t("common.status")}</Label>
              <Select value={formStatus} onValueChange={setFormStatus}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="active">{t("common.enabled")}</SelectItem>
                  <SelectItem value="disabled">{t("common.disabled")}</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditOpen(false)}>{t("common.cancel")}</Button>
            <Button onClick={handleEdit} disabled={updateDept.isPending}>
              {updateDept.isPending ? t("common.saving") : t("common.save")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Dialog */}
      <Dialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("departments.deleteTitle")}</DialogTitle>
            <DialogDescription>
              {t("departments.deleteConfirm", { name: selectedNode?.name })}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteOpen(false)}>{t("common.cancel")}</Button>
            <Button variant="destructive" onClick={handleDelete} disabled={deleteDept.isPending}>
              {deleteDept.isPending ? t("common.deleting") : t("common.delete")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
