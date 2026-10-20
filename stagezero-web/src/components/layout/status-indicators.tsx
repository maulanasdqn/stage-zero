import { FileTextIcon, MailIcon } from 'lucide-react'
import { useSettings } from '@/apis/settings'
import { Badge } from '@/components/ui/badge'
import { ButtonGroup } from '@/components/ui/button-group'
import { Skeleton } from '@/components/ui/skeleton'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'

const READY = 'border-success/30 bg-success/10 text-success'
const MISSING = 'border-destructive/30 bg-destructive/10 text-destructive'

export function StatusIndicators() {
  const { data: settings } = useSettings()

  if (!settings) {
    return <Skeleton className="h-5 w-40" />
  }

  return (
    <ButtonGroup className="flex-wrap gap-1.5">
      <Tooltip>
        <TooltipTrigger asChild>
          <Badge variant="outline" className={cn(settings.mailer_configured ? READY : MISSING)}>
            <MailIcon />
            {settings.mailer_provider}
          </Badge>
        </TooltipTrigger>
        <TooltipContent>
          {settings.mailer_configured
            ? `Sending as ${settings.sender}`
            : 'API token or account id missing'}
        </TooltipContent>
      </Tooltip>
      <Badge variant="outline" className={cn(settings.resume_loaded ? READY : MISSING)}>
        <FileTextIcon />
        {settings.resume_loaded ? 'Resume' : 'No resume'}
      </Badge>
    </ButtonGroup>
  )
}
