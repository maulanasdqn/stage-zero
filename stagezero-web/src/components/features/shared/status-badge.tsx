import { Badge } from '@/components/ui/badge'
import type { TApplicationStatus } from '@/apis/startups'
import { cn } from '@/lib/utils'

const STATUS_STYLES: Record<TApplicationStatus, { label: string; className: string }> = {
  APPLIED: { label: 'Applied', className: 'border-success/30 bg-success/10 text-success' },
  SKIPPED: { label: 'Skipped', className: 'border-warning/30 bg-warning/10 text-warning' },
  FAILED_NO_CONTACT: {
    label: 'No contact',
    className: 'border-destructive/30 bg-destructive/10 text-destructive',
  },
}

type TStatusBadgeProps = {
  status: TApplicationStatus | null
  className?: string
}

export function StatusBadge({ status, className }: TStatusBadgeProps) {
  if (!status) {
    return (
      <Badge variant="outline" className={cn('text-muted-foreground', className)}>
        New
      </Badge>
    )
  }
  const style = STATUS_STYLES[status]
  return (
    <Badge variant="outline" className={cn(style.className, className)}>
      {style.label}
    </Badge>
  )
}
