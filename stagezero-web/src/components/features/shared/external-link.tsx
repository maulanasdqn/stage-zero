import { ExternalLinkIcon } from 'lucide-react'
import type { MouseEvent, ReactNode } from 'react'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

type TExternalLinkProps = {
  href: string
  children: ReactNode
  className?: string
  stopPropagation?: boolean
}

export function ExternalLink({ href, children, className, stopPropagation }: TExternalLinkProps) {
  return (
    <Button
      variant="link"
      size="sm"
      className={cn('text-muted-foreground hover:text-foreground h-auto gap-1 px-0', className)}
      asChild
    >
      <a
        href={href}
        target="_blank"
        rel="noopener noreferrer"
        onClick={(event: MouseEvent) => stopPropagation && event.stopPropagation()}
      >
        {children}
        <ExternalLinkIcon />
      </a>
    </Button>
  )
}
