import { useState, useRef } from "react"
import { useTranslation } from "react-i18next"
import { useFiles, useUploadFile, useDeleteFile } from "@/hooks/useFiles"
import { PermissionGuard } from "@/components/shared/PermissionGuard"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
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
import { Pagination } from "@/components/ui/pagination"
import { Upload, Trash2, File as FileIcon } from "lucide-react"
import { toast } from "sonner"
import type { FileRecord } from "@/types/file"

/** Format bytes into human-readable string */
function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

export function FilesPage() {
  const { t } = useTranslation()

  const [page, setPage] = useState(1)
  const perPage = 20

  const { data, isLoading } = useFiles({ page, per_page: perPage })
  const uploadFile = useUploadFile()
  const deleteFile = useDeleteFile()

  const fileInputRef = useRef<HTMLInputElement>(null)

  // Delete dialog state
  const [deleteOpen, setDeleteOpen] = useState(false)
  const [selectedFile, setSelectedFile] = useState<FileRecord | null>(null)

  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return
    try {
      await uploadFile.mutateAsync(file)
      toast.success(t("common.success"))
    } catch (err: any) {
      toast.error(err.response?.data?.error || t("common.error"))
    }
    // Reset the input so the same file can be re-selected
    if (fileInputRef.current) fileInputRef.current.value = ""
  }

  const handleDelete = async () => {
    if (!selectedFile) return
    try {
      await deleteFile.mutateAsync(selectedFile.id)
      toast.success(t("common.success"))
      setDeleteOpen(false)
      setSelectedFile(null)
    } catch (err: any) {
      toast.error(err.response?.data?.error || t("common.error"))
    }
  }

  const openDeleteDialog = (file: FileRecord) => {
    setSelectedFile(file)
    setDeleteOpen(true)
  }

  const files = data?.data || []
  const total = data?.pagination.total || 0

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t("files.title")}</h1>
          <p className="text-muted-foreground">{t("files.upload")}</p>
        </div>
        <PermissionGuard permission="files:upload">
          <>
            <input
              ref={fileInputRef}
              type="file"
              className="hidden"
              onChange={handleFileSelect}
            />
            <Button onClick={() => fileInputRef.current?.click()} disabled={uploadFile.isPending}>
              <Upload className="mr-2 h-4 w-4" />
              {uploadFile.isPending ? t("common.loading") : t("files.upload")}
            </Button>
          </>
        </PermissionGuard>
      </div>

      {/* Files Table */}
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t("files.fileName")}</TableHead>
              <TableHead>{t("files.fileType")}</TableHead>
              <TableHead>{t("files.fileSize")}</TableHead>
              <TableHead>{t("files.uploadTime")}</TableHead>
              <PermissionGuard permission="files:delete">
                <TableHead className="w-12">{t("common.actions")}</TableHead>
              </PermissionGuard>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={5} className="text-center text-muted-foreground py-8">
                  {t("common.loading")}
                </TableCell>
              </TableRow>
            ) : files.length === 0 ? (
              <TableRow>
                <TableCell colSpan={5} className="text-center text-muted-foreground py-8">
                  {t("common.noData")}
                </TableCell>
              </TableRow>
            ) : (
              files.map((file) => (
                <TableRow key={file.id}>
                  <TableCell className="font-medium">
                    <div className="flex items-center gap-2">
                      <FileIcon className="h-4 w-4 text-muted-foreground" />
                      {file.original_name}
                    </div>
                  </TableCell>
                  <TableCell className="text-muted-foreground">{file.mime_type}</TableCell>
                  <TableCell className="text-muted-foreground">{formatFileSize(file.size)}</TableCell>
                  <TableCell className="text-muted-foreground text-sm">
                    {file.created_at
                      ? new Date(file.created_at).toLocaleDateString()
                      : "-"}
                  </TableCell>
                  <PermissionGuard permission="files:delete">
                    <TableCell>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 text-destructive hover:text-destructive"
                        onClick={() => openDeleteDialog(file)}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
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

      {/* Delete File Dialog */}
      <Dialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("common.delete")}</DialogTitle>
            <DialogDescription>{t("files.deleteConfirm")}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteOpen(false)}>
              {t("common.cancel")}
            </Button>
            <Button variant="destructive" onClick={handleDelete} disabled={deleteFile.isPending}>
              {deleteFile.isPending ? t("common.loading") : t("common.delete")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
