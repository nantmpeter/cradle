import { useMemo } from "react"
import { usePermissions } from "@/hooks/usePermissions"
import { Checkbox } from "@/components/ui/checkbox"
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion"
import type { Permission } from "@/types/api"

interface PermissionSelectorProps {
  /** Currently selected permission IDs */
  selectedIds: string[]
  /** Callback when selection changes */
  onChange: (ids: string[]) => void
  /** Whether the selector is disabled */
  disabled?: boolean
}

/** Group permissions by their module */
function groupByModule(permissions: Permission[]): Record<string, Permission[]> {
  const groups: Record<string, Permission[]> = {}
  for (const perm of permissions) {
    const mod = perm.module || "other"
    if (!groups[mod]) {
      groups[mod] = []
    }
    groups[mod].push(perm)
  }
  return groups
}

export function PermissionSelector({
  selectedIds,
  onChange,
  disabled = false,
}: PermissionSelectorProps) {
  const { data: permissions = [], isLoading } = usePermissions()

  const grouped = useMemo(() => groupByModule(permissions), [permissions])

  const handleToggle = (permId: string, checked: boolean) => {
    if (checked) {
      onChange([...selectedIds, permId])
    } else {
      onChange(selectedIds.filter((id) => id !== permId))
    }
  }

  const handleToggleAll = (modulePerms: Permission[], checked: boolean) => {
    const moduleIds = modulePerms.map((p) => p.id)
    if (checked) {
      // Add all module IDs that aren't already selected
      const newIds = [...new Set([...selectedIds, ...moduleIds])]
      onChange(newIds)
    } else {
      // Remove all module IDs
      onChange(selectedIds.filter((id) => !moduleIds.includes(id)))
    }
  }

  if (isLoading) {
    return (
      <div className="py-4 text-center text-sm text-muted-foreground">
        Loading permissions...
      </div>
    )
  }

  return (
    <Accordion type="multiple" defaultValue={Object.keys(grouped)} className="w-full">
      {Object.entries(grouped).map(([module, perms]) => {
        const allSelected = perms.every((p) => selectedIds.includes(p.id))
        const someSelected = perms.some((p) => selectedIds.includes(p.id))

        return (
          <AccordionItem key={module} value={module}>
            <AccordionTrigger className="hover:no-underline">
              <div className="flex items-center gap-2">
                <Checkbox
                  checked={allSelected}
                  // Indeterminate state visual feedback
                  ref={(el) => {
                    if (el) {
                      // Set indeterminate state manually
                      const input = el.querySelector("input[type='checkbox']") as HTMLInputElement | null
                      if (input) {
                        input.indeterminate = someSelected && !allSelected
                      }
                    }
                  }}
                  onCheckedChange={(checked) => handleToggleAll(perms, !!checked)}
                  onClick={(e) => e.stopPropagation()}
                  disabled={disabled}
                />
                <span className="text-sm font-medium capitalize">{module}</span>
                <span className="text-xs text-muted-foreground">
                  ({perms.filter((p) => selectedIds.includes(p.id)).length}/{perms.length})
                </span>
              </div>
            </AccordionTrigger>
            <AccordionContent>
              <div className="space-y-2 pl-6">
                {perms.map((perm) => (
                  <label
                    key={perm.id}
                    className="flex items-center gap-2 text-sm cursor-pointer"
                  >
                    <Checkbox
                      checked={selectedIds.includes(perm.id)}
                      onCheckedChange={(checked) => handleToggle(perm.id, !!checked)}
                      disabled={disabled}
                    />
                    <span>{perm.name}</span>
                    {perm.description && (
                      <span className="text-xs text-muted-foreground">
                        — {perm.description}
                      </span>
                    )}
                  </label>
                ))}
              </div>
            </AccordionContent>
          </AccordionItem>
        )
      })}
    </Accordion>
  )
}
