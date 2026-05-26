import { useNavigate } from "react-router-dom"
import { Button } from "@/components/ui/button"
import { ShieldX } from "lucide-react"
import { useTranslation } from "react-i18next"

export function ForbiddenPage() {
  const navigate = useNavigate()
  const { t } = useTranslation()

  return (
    <div className="flex min-h-[60vh] flex-col items-center justify-center gap-4">
      <ShieldX className="h-16 w-16 text-muted-foreground" />
      <h1 className="text-2xl font-bold">{t("auth.forbidden")}</h1>
      <p className="text-muted-foreground">
        {t("auth.forbiddenDesc")}
      </p>
      <Button variant="outline" onClick={() => navigate("/dashboard")}>
        {t("auth.backToDashboard")}
      </Button>
    </div>
  )
}
