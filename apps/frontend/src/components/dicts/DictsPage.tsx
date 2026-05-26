import { useState } from "react"
import {
  useDictTypes, useCreateDictType, useUpdateDictType, useDeleteDictType,
  useDictItems, useCreateDictItem, useUpdateDictItem, useDeleteDictItem,
} from "@/hooks/useDicts"
import { PermissionGuard } from "@/components/shared/PermissionGuard"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Card } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from "@/components/ui/table"
import {
  Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
} from "@/components/ui/dialog"
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select"
import { Pagination } from "@/components/ui/pagination"
import { Plus, Edit, Trash2, BookOpen } from "lucide-react"
import { toast } from "sonner"
import { useTranslation } from "react-i18next"
import type { DictType, DictItem } from "@/types/dict"

export function DictsPage() {
  const { t } = useTranslation()
  const [page, setPage] = useState(1)
  const perPage = 20
  const [selectedType, setSelectedType] = useState<DictType | null>(null)

  // Dict types
  const { data: typesData, isLoading: typesLoading } = useDictTypes({ page, per_page: perPage })
  const createType = useCreateDictType()
  const updateType = useUpdateDictType()
  const deleteType = useDeleteDictType()

  // Dict items for selected type
  const { data: items = [], isLoading: itemsLoading } = useDictItems(selectedType?.id || null)
  const createItem = useCreateDictItem()
  const updateItem = useUpdateDictItem()
  const deleteItem = useDeleteDictItem()

  // Dialog states
  const [typeFormOpen, setTypeFormOpen] = useState(false)
  const [typeDeleteOpen, setTypeDeleteOpen] = useState(false)
  const [itemFormOpen, setItemFormOpen] = useState(false)
  const [itemDeleteOpen, setItemDeleteOpen] = useState(false)
  const [editingType, setEditingType] = useState<DictType | null>(null)
  const [editingItem, setEditingItem] = useState<DictItem | null>(null)

  // Type form
  const [typeName, setTypeName] = useState("")
  const [typeCode, setTypeCode] = useState("")
  const [typeStatus, setTypeStatus] = useState("active")
  const [typeRemark, setTypeRemark] = useState("")

  // Item form
  const [itemLabel, setItemLabel] = useState("")
  const [itemValue, setItemValue] = useState("")
  const [itemSortOrder, setItemSortOrder] = useState(0)
  const [itemStatus, setItemStatus] = useState("active")
  const [itemRemark, setItemRemark] = useState("")

  const resetTypeForm = () => { setTypeName(""); setTypeCode(""); setTypeStatus("active"); setTypeRemark("") }
  const resetItemForm = () => { setItemLabel(""); setItemValue(""); setItemSortOrder(0); setItemStatus("active"); setItemRemark("") }

  // Type CRUD
  const handleSaveType = async () => {
    try {
      if (editingType) {
        await updateType.mutateAsync({ id: editingType.id, data: { name: typeName, status: typeStatus, remark: typeRemark || undefined } })
        toast.success(t("dicts.typeUpdated"))
      } else {
        await createType.mutateAsync({ name: typeName, code: typeCode, status: typeStatus, remark: typeRemark || undefined })
        toast.success(t("dicts.typeCreated"))
      }
      setTypeFormOpen(false)
      resetTypeForm()
    } catch (err: any) {
      toast.error(err.response?.data?.error || "Failed")
    }
  }

  const handleDeleteType = async () => {
    if (!editingType) return
    try {
      await deleteType.mutateAsync(editingType.id)
      toast.success(t("dicts.typeDeleted"))
      setTypeDeleteOpen(false)
      if (selectedType?.id === editingType.id) setSelectedType(null)
    } catch (err: any) {
      toast.error(err.response?.data?.error || "Failed")
    }
  }

  // Item CRUD
  const handleSaveItem = async () => {
    if (!selectedType) return
    try {
      if (editingItem) {
        await updateItem.mutateAsync({ id: editingItem.id, data: { label: itemLabel, value: itemValue, sort_order: itemSortOrder, status: itemStatus } })
        toast.success(t("dicts.itemUpdated"))
      } else {
        await createItem.mutateAsync({ typeId: selectedType.id, data: { label: itemLabel, value: itemValue, sort_order: itemSortOrder, status: itemStatus } })
        toast.success(t("dicts.itemCreated"))
      }
      setItemFormOpen(false)
      resetItemForm()
    } catch (err: any) {
      toast.error(err.response?.data?.error || "Failed")
    }
  }

  const handleDeleteItem = async () => {
    if (!editingItem) return
    try {
      await deleteItem.mutateAsync(editingItem.id)
      toast.success(t("dicts.itemDeleted"))
      setItemDeleteOpen(false)
    } catch (err: any) {
      toast.error(err.response?.data?.error || "Failed")
    }
  }

  const types = typesData?.data || []
  const total = (typesData as any)?.total || 0

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t("dicts.title")}</h1>
          <p className="text-muted-foreground">{t("dicts.subtitle")}</p>
        </div>
        <PermissionGuard permission="dicts:create">
          <Button onClick={() => { setEditingType(null); resetTypeForm(); setTypeFormOpen(true) }}>
            <Plus className="mr-2 h-4 w-4" />
            {t("dicts.createType")}
          </Button>
        </PermissionGuard>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Dict Types Table */}
        <Card className="lg:col-span-1">
          <div className="p-4">
            <h2 className="text-base font-semibold mb-3">{t("dicts.typeList")}</h2>
            {typesLoading ? (
              <p className="text-sm text-muted-foreground">{t("common.loading")}</p>
            ) : (
              <div className="space-y-1">
                {types.map((dt) => (
                  <div
                    key={dt.id}
                    className={`flex items-center justify-between py-2 px-3 rounded-md cursor-pointer hover:bg-muted/50 ${
                      selectedType?.id === dt.id ? "bg-muted" : ""
                    }`}
                    onClick={() => setSelectedType(dt)}
                  >
                    <div className="flex items-center gap-2">
                      <BookOpen className="h-4 w-4 text-muted-foreground" />
                      <span className="text-sm font-medium">{dt.name}</span>
                    </div>
                    <Badge variant="outline" className="text-xs">{dt.code}</Badge>
                  </div>
                ))}
              </div>
            )}
            <Pagination page={page} total={total} perPage={perPage} onPageChange={setPage} />
          </div>
        </Card>

        {/* Dict Items Table */}
        <div className="lg:col-span-2">
          <div className="rounded-md border">
            <div className="flex items-center justify-between p-4 border-b">
              <h2 className="text-base font-semibold">
                {selectedType ? `${selectedType.name} — ${t("dicts.items")}` : t("dicts.selectType")}
              </h2>
              {selectedType && (
                <div className="flex gap-2">
                  <PermissionGuard permission="dicts:update">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        setEditingType(selectedType)
                        setTypeName(selectedType.name)
                        setTypeCode(selectedType.code)
                        setTypeStatus(selectedType.status)
                        setTypeRemark(selectedType.remark || "")
                        setTypeFormOpen(true)
                      }}
                    >
                      <Edit className="mr-1 h-3 w-3" /> {t("common.edit")}
                    </Button>
                  </PermissionGuard>
                  <PermissionGuard permission="dicts:delete">
                    <Button variant="destructive" size="sm" onClick={() => { setEditingType(selectedType); setTypeDeleteOpen(true) }}>
                      <Trash2 className="mr-1 h-3 w-3" /> {t("common.delete")}
                    </Button>
                  </PermissionGuard>
                  <PermissionGuard permission="dicts:create">
                    <Button size="sm" onClick={() => { setEditingItem(null); resetItemForm(); setItemFormOpen(true) }}>
                      <Plus className="mr-1 h-3 w-3" /> {t("dicts.createItem")}
                    </Button>
                  </PermissionGuard>
                </div>
              )}
            </div>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{t("dicts.itemLabel")}</TableHead>
                  <TableHead>{t("dicts.itemValue")}</TableHead>
                  <TableHead>{t("departments.sortOrder")}</TableHead>
                  <TableHead>{t("common.status")}</TableHead>
                  <TableHead className="w-20">{t("common.actions")}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {itemsLoading ? (
                  <TableRow><TableCell colSpan={5} className="text-center py-8">{t("common.loading")}</TableCell></TableRow>
                ) : !selectedType ? (
                  <TableRow><TableCell colSpan={5} className="text-center py-8 text-muted-foreground">{t("dicts.selectTypeHint")}</TableCell></TableRow>
                ) : items.length === 0 ? (
                  <TableRow><TableCell colSpan={5} className="text-center py-8 text-muted-foreground">{t("common.noData")}</TableCell></TableRow>
                ) : (
                  items.map((item) => (
                    <TableRow key={item.id}>
                      <TableCell className="font-medium">{item.label}</TableCell>
                      <TableCell><code className="text-xs bg-muted px-1.5 py-0.5 rounded">{item.value}</code></TableCell>
                      <TableCell>{item.sort_order}</TableCell>
                      <TableCell><Badge variant={item.status === "active" ? "default" : "secondary"}>{item.status}</Badge></TableCell>
                      <TableCell>
                        <div className="flex gap-1">
                          <PermissionGuard permission="dicts:update">
                            <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => {
                              setEditingItem(item)
                              setItemLabel(item.label)
                              setItemValue(item.value)
                              setItemSortOrder(item.sort_order)
                              setItemStatus(item.status)
                              setItemRemark(item.remark || "")
                              setItemFormOpen(true)
                            }}>
                              <Edit className="h-3.5 w-3.5" />
                            </Button>
                          </PermissionGuard>
                          <PermissionGuard permission="dicts:delete">
                            <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => { setEditingItem(item); setItemDeleteOpen(true) }}>
                              <Trash2 className="h-3.5 w-3.5 text-destructive" />
                            </Button>
                          </PermissionGuard>
                        </div>
                      </TableCell>
                    </TableRow>
                  ))
                )}
              </TableBody>
            </Table>
          </div>
        </div>
      </div>

      {/* Type Form Dialog */}
      <Dialog open={typeFormOpen} onOpenChange={setTypeFormOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{editingType ? t("dicts.editType") : t("dicts.createType")}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label>{t("common.name")}</Label>
              <Input value={typeName} onChange={(e) => setTypeName(e.target.value)} />
            </div>
            {!editingType && (
              <div className="space-y-2">
                <Label>{t("dicts.typeCode")}</Label>
                <Input value={typeCode} onChange={(e) => setTypeCode(e.target.value)} />
              </div>
            )}
            <div className="space-y-2">
              <Label>{t("common.status")}</Label>
              <Select value={typeStatus} onValueChange={setTypeStatus}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="active">{t("common.enabled")}</SelectItem>
                  <SelectItem value="disabled">{t("common.disabled")}</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>{t("common.description")}</Label>
              <Input value={typeRemark} onChange={(e) => setTypeRemark(e.target.value)} />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setTypeFormOpen(false)}>{t("common.cancel")}</Button>
            <Button onClick={handleSaveType} disabled={createType.isPending || updateType.isPending || !typeName || (!editingType && !typeCode)}>
              {createType.isPending || updateType.isPending ? t("common.saving") : t("common.save")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Type Delete Dialog */}
      <Dialog open={typeDeleteOpen} onOpenChange={setTypeDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("dicts.deleteType")}</DialogTitle>
            <DialogDescription>{t("dicts.deleteTypeConfirm", { name: editingType?.name })}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setTypeDeleteOpen(false)}>{t("common.cancel")}</Button>
            <Button variant="destructive" onClick={handleDeleteType} disabled={deleteType.isPending}>
              {deleteType.isPending ? t("common.deleting") : t("common.delete")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Item Form Dialog */}
      <Dialog open={itemFormOpen} onOpenChange={setItemFormOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{editingItem ? t("dicts.editItem") : t("dicts.createItem")}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>{t("dicts.itemLabel")}</Label>
                <Input value={itemLabel} onChange={(e) => setItemLabel(e.target.value)} />
              </div>
              <div className="space-y-2">
                <Label>{t("dicts.itemValue")}</Label>
                <Input value={itemValue} onChange={(e) => setItemValue(e.target.value)} />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>{t("departments.sortOrder")}</Label>
                <Input type="number" value={itemSortOrder} onChange={(e) => setItemSortOrder(Number(e.target.value))} />
              </div>
              <div className="space-y-2">
                <Label>{t("common.status")}</Label>
                <Select value={itemStatus} onValueChange={setItemStatus}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="active">{t("common.enabled")}</SelectItem>
                    <SelectItem value="disabled">{t("common.disabled")}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setItemFormOpen(false)}>{t("common.cancel")}</Button>
            <Button onClick={handleSaveItem} disabled={createItem.isPending || updateItem.isPending || !itemLabel || !itemValue}>
              {createItem.isPending || updateItem.isPending ? t("common.saving") : t("common.save")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Item Delete Dialog */}
      <Dialog open={itemDeleteOpen} onOpenChange={setItemDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("dicts.deleteItem")}</DialogTitle>
            <DialogDescription>{t("dicts.deleteItemConfirm", { label: editingItem?.label })}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setItemDeleteOpen(false)}>{t("common.cancel")}</Button>
            <Button variant="destructive" onClick={handleDeleteItem} disabled={deleteItem.isPending}>
              {deleteItem.isPending ? t("common.deleting") : t("common.delete")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
