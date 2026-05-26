import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { useTranslation } from "react-i18next"
import type { RoleResponse } from "@/types/api"

interface RoleDeleteDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  role: RoleResponse | null
  onConfirm: () => void
  isPending: boolean
}

export function RoleDeleteDialog({
  open,
  onOpenChange,
  role,
  onConfirm,
  isPending,
}: RoleDeleteDialogProps) {
  const { t } = useTranslation()

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("roles.deleteRole")}</DialogTitle>
          <DialogDescription>
            {t("roles.deleteConfirm", { name: role?.name })}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t("common.cancel")}
          </Button>
          <Button variant="destructive" onClick={onConfirm} disabled={isPending}>
            {isPending ? t("common.deleting") : t("common.delete")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
