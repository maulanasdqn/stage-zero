import { Badge } from '@/components/ui/badge'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import type { TContactOrigin } from '@/apis/startups'

const ORIGIN_COPY: Record<TContactOrigin, { label: string; hint: string; className: string }> = {
  scraped: {
    label: 'Found',
    hint: 'Found on the startup website',
    className: 'border-success/30 bg-success/10 text-success',
  },
  guessed: {
    label: 'Guessed',
    hint: 'Guessed from the founder name. It may bounce.',
    className: 'border-warning/30 bg-warning/10 text-warning',
  },
}

export function OriginBadge({ origin }: { origin: TContactOrigin }) {
  const copy = ORIGIN_COPY[origin]
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Badge variant="outline" className={copy.className}>
          {copy.label}
        </Badge>
      </TooltipTrigger>
      <TooltipContent>{copy.hint}</TooltipContent>
    </Tooltip>
  )
}
