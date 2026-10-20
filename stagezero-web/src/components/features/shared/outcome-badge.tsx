import { Badge } from '@/components/ui/badge'
import type { THuntOutcomeKind } from '@/apis/startups'

const OUTCOME_STYLES: Record<THuntOutcomeKind, { label: string; className: string }> = {
  applied: { label: 'Sent', className: 'border-success/30 bg-success/10 text-success' },
  previewed: { label: 'Preview', className: 'border-primary/30 bg-primary/10 text-primary' },
  no_contact: {
    label: 'No contact',
    className: 'border-destructive/30 bg-destructive/10 text-destructive',
  },
  send_failed: {
    label: 'Failed',
    className: 'border-destructive/30 bg-destructive/10 text-destructive',
  },
  already_processed: { label: 'Already done', className: 'text-muted-foreground' },
  skipped_no_website: { label: 'No website', className: 'text-muted-foreground' },
}

export function OutcomeBadge({ kind }: { kind: THuntOutcomeKind }) {
  const style = OUTCOME_STYLES[kind]
  return (
    <Badge variant="outline" className={style.className}>
      {style.label}
    </Badge>
  )
}
