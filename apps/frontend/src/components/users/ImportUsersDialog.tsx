import { useRef, useState } from "react"
import { useImportUsers, useDownloadImportTemplate } from "@/hooks/useImport"
import { Button } from "@/components/ui/button"
import {
  Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
} from "@/components/ui/dialog"
import { Upload, Download, FileSpreadsheet, AlertCircle, CheckCircle2 } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { ImportResult } from "@/hooks/useImport"

interface ImportUsersDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function ImportUsersDialog({ open, onOpenChange }: ImportUsersDialogProps) {
  const { t } = useTranslation()
  const fileInputRef = useRef<HTMLInputElement>(null)
  const importUsers = useImportUsers()
  const downloadTemplate = useDownloadImportTemplate()
  const [result, setResult] = useState<ImportResult | null>(null)

  const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return
    try {
      const res = await importUsers.mutateAsync(file)
      setResult(res)
    } catch (err: any) {
      // Error handled by mutation
    }
    // Reset input
    if (fileInputRef.current) fileInputRef.current.value = ""
  }

  const handleDownloadTemplate = async () => {
    try {
      await downloadTemplate.mutateAsync()
    } catch (err: any) {
      // Error handled
    }
  }

  const handleClose = () => {
    setResult(null)
    onOpenChange(false)
  }

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>{t("users.importTitle")}</DialogTitle>
          <DialogDescription>{t("users.importDesc")}</DialogDescription>
        </DialogHeader>

        {result ? (
          <div className="space-y-4">
            <div className="flex items-center gap-2 text-green-600">
              <CheckCircle2 className="h-5 w-5" />
              <span className="font-medium">
                {t("users.importComplete")}: {result.success_count} {t("users.importSuccess")}
                {result.fail_count > 0 && `, ${result.fail_count} ${t("users.importFailed")}`}
              </span>
            </div>

            {result.generated_passwords.length > 0 && (
              <div className="space-y-2">
                <h4 className="text-sm font-medium">{t("users.generatedPasswords")}</h4>
                <div className="max-h-40 overflow-y-auto rounded-md border p-2 text-sm">
                  {result.generated_passwords.map((p, i) => (
                    <div key={i} className="flex justify-between py-0.5">
                      <span>{p.email}</span>
                      <code className="text-xs bg-muted px-1 rounded">{p.password}</code>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {result.errors.length > 0 && (
              <div className="space-y-2">
                <h4 className="text-sm font-medium text-destructive">{t("users.importErrors")}</h4>
                <div className="max-h-40 overflow-y-auto rounded-md border border-destructive/30 p-2 text-sm">
                  {result.errors.map((e, i) => (
                    <div key={i} className="flex items-start gap-2 py-0.5">
                      <AlertCircle className="h-3.5 w-3.5 text-destructive shrink-0 mt-0.5" />
                      <span>Row {e.row}: {e.reason}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        ) : (
          <div className="space-y-4">
            <div className="flex items-center gap-3">
              <Button variant="outline" onClick={handleDownloadTemplate} disabled={downloadTemplate.isPending}>
                <Download className="mr-2 h-4 w-4" />
                {t("users.downloadTemplate")}
              </Button>
              <span className="text-sm text-muted-foreground">{t("users.templateHint")}</span>
            </div>

            <div
              className="border-2 border-dashed rounded-lg p-8 text-center cursor-pointer hover:border-primary/50 transition-colors"
              onClick={() => fileInputRef.current?.click()}
            >
              <FileSpreadsheet className="h-8 w-8 mx-auto text-muted-foreground" />
              <p className="mt-2 text-sm text-muted-foreground">{t("users.selectFile")}</p>
              <p className="text-xs text-muted-foreground">.xlsx</p>
              <input
                ref={fileInputRef}
                type="file"
                accept=".xlsx"
                className="hidden"
                onChange={handleFileChange}
              />
            </div>

            {importUsers.isPending && (
              <p className="text-sm text-muted-foreground text-center">{t("users.importing")}</p>
            )}
          </div>
        )}

        <DialogFooter>
          <Button variant="outline" onClick={handleClose}>{t("common.close")}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
