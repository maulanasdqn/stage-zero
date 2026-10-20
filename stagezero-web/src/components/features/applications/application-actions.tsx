import { ExternalLinkIcon, GlobeIcon, MoreHorizontalIcon, Trash2Icon } from 'lucide-react'
import type { TApplication } from '@/apis/applications'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { safeUrl, ycCompanyUrl } from '@/libs/format'

type TApplicationActionsProps = {
  application: TApplication
  onRemove: (application: TApplication) => void
}

function openExternal(url: string) {
  window.open(url, '_blank', 'noopener,noreferrer')
}

export function ApplicationActions({ application, onRemove }: TApplicationActionsProps) {
  const website = safeUrl(application.website)
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label={`Actions for ${application.company_name}`}
        >
          <MoreHorizontalIcon />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem onSelect={() => openExternal(ycCompanyUrl(application.yc_slug))}>
          <ExternalLinkIcon />
          Open YC page
        </DropdownMenuItem>
        {website && (
          <DropdownMenuItem onSelect={() => openExternal(website)}>
            <GlobeIcon />
            Visit website
          </DropdownMenuItem>
        )}
        <DropdownMenuSeparator />
        <DropdownMenuItem variant="destructive" onSelect={() => onRemove(application)}>
          <Trash2Icon />
          Remove from tracker
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
