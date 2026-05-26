import { useState } from "react"
import { useSystemConfigs, useUpdateConfig } from "@/hooks/useSystemConfigs"
import type { SystemConfig } from "@/types/systemConfig"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent } from "@/components/ui/card"
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { Switch } from "@/components/ui/switch"
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion"
import { Settings, Pencil } from "lucide-react"
import { toast } from "sonner"
import { useTranslation } from "react-i18next"

export function SystemConfigPage() {
  const { t } = useTranslation()
  const { data: configs, isLoading } = useSystemConfigs()
  const updateConfig = useUpdateConfig()

  const [editConfig, setEditConfig] = useState<SystemConfig | null>(null)
  const [editValue, setEditValue] = useState("")
  const [editDialogOpen, setEditDialogOpen] = useState(false)

  const grouped = (configs ?? []).reduce<Record<string, SystemConfig[]>>((acc, c) => {
    if (!acc[c.group_key]) acc[c.group_key] = []
    acc[c.group_key].push(c)
    return acc
  }, {})

  const handleEdit = (config: SystemConfig) => {
    setEditConfig(config)
    setEditValue(config.value)
    setEditDialogOpen(true)
  }

  const handleSave = async () => {
    if (!editConfig) return
    try {
      await updateConfig.mutateAsync({
        group: editConfig.group_key,
        key: editConfig.config_key,
        value: editValue,
      })
      toast.success(t("configs.saveSuccess"))
      setEditDialogOpen(false)
      setEditConfig(null)
    } catch (err: any) {
      toast.error(err.response?.data?.error || t("configs.saveFailed"))
    }
  }

  const renderValueInput = () => {
    if (!editConfig) return null

    if (editConfig.value_type === "boolean") {
      return (
        <div className="flex items-center gap-2">
          <Switch
            checked={editValue === "true"}
            onCheckedChange={(checked) => setEditValue(checked ? "true" : "false")}
          />
          <span className="text-sm">{editValue === "true" ? t("configs.enabled") : t("configs.disabled")}</span>
        </div>
      )
    }

    if (editConfig.value_type === "json") {
      return (
        <textarea
          className="flex min-h-[120px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          value={editValue}
          onChange={(e) => setEditValue(e.target.value)}
        />
      )
    }

    return (
      <Input
        type={editConfig.value_type === "number" ? "number" : "text"}
        value={editValue}
        onChange={(e) => setEditValue(e.target.value)}
      />
    )
  }

  const valueTypeColors: Record<string, string> = {
    string: "bg-blue-100 text-blue-800",
    number: "bg-purple-100 text-purple-800",
    boolean: "bg-green-100 text-green-800",
    json: "bg-orange-100 text-orange-800",
  }

  if (isLoading) {
    return <div className="py-8 text-center text-muted-foreground">{t("common.loading")}</div>
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold flex items-center gap-2">
          <Settings className="h-6 w-6" /> {t("configs.title")}
        </h1>
        <p className="text-muted-foreground">{t("configs.subtitle")}</p>
      </div>

      <Accordion type="multiple" defaultValue={Object.keys(grouped)}>
        {Object.entries(grouped).map(([group, items]) => (
          <AccordionItem key={group} value={group}>
            <AccordionTrigger className="text-lg font-medium capitalize">
              {group.replace(/_/g, " ")} ({items.length})
            </AccordionTrigger>
            <AccordionContent>
              <div className="space-y-3 pt-2">
                {items.map((config) => (
                  <Card key={config.config_key}>
                    <CardContent className="flex items-center justify-between p-4">
                      <div className="space-y-1 flex-1">
                        <div className="flex items-center gap-2">
                          <span className="font-mono text-sm font-medium">{config.config_key}</span>
                          <Badge variant="secondary" className={valueTypeColors[config.value_type] || ""}>
                            {config.value_type}
                          </Badge>
                          {config.is_public && <Badge variant="outline">{t("configs.public")}</Badge>}
                        </div>
                        {config.description && (
                          <p className="text-xs text-muted-foreground">{config.description}</p>
                        )}
                        <p className="text-sm font-mono bg-muted rounded px-2 py-1 mt-1 break-all">
                          {config.value_type === "boolean"
                            ? (config.value === "true" ? `✓ ${t("configs.enabled")}` : `✗ ${t("configs.disabled")}`)
                            : config.value.length > 100
                              ? config.value.slice(0, 100) + "..."
                              : config.value}
                        </p>
                      </div>
                      <Button variant="ghost" size="icon" onClick={() => handleEdit(config)}>
                        <Pencil className="h-4 w-4" />
                      </Button>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </AccordionContent>
          </AccordionItem>
        ))}
      </Accordion>

      <Dialog open={editDialogOpen} onOpenChange={setEditDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("configs.editConfig")}</DialogTitle>
            <DialogDescription>
              {editConfig?.group_key} / {editConfig?.config_key}
            </DialogDescription>
          </DialogHeader>
          {editConfig && (
            <div className="space-y-4">
              {editConfig.description && (
                <p className="text-sm text-muted-foreground">{editConfig.description}</p>
              )}
              <div className="space-y-2">
                <Label>{t("configs.value")} ({editConfig.value_type})</Label>
                {renderValueInput()}
              </div>
            </div>
          )}
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditDialogOpen(false)}>{t("common.cancel")}</Button>
            <Button onClick={handleSave} disabled={updateConfig.isPending}>
              {updateConfig.isPending ? t("common.saving") : t("common.save")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
