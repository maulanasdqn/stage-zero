import { ChevronDownIcon, RadarIcon } from 'lucide-react'
import type { THuntEvent } from '@/apis/hunt'
import { EmptyState } from '@/components/features/shared/empty-state'
import { OriginBadge } from '@/components/features/shared/origin-badge'
import { OutcomeBadge } from '@/components/features/shared/outcome-badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import {
  Item,
  ItemActions,
  ItemContent,
  ItemDescription,
  ItemFooter,
  ItemGroup,
  ItemMedia,
  ItemSeparator,
  ItemTitle,
} from '@/components/ui/item'
import { Textarea } from '@/components/ui/textarea'

function emailText(event: THuntEvent): string {
  if (!event.email) {
    return ''
  }
  return `To: ${event.email.to}\nSubject: ${event.email.subject}\n\n${event.email.body}`
}

function EventItem({ event }: { event: THuntEvent }) {
  return (
    <Collapsible asChild>
      <Item size="sm" className="px-0">
        <ItemMedia>
          <OutcomeBadge kind={event.outcome.kind} />
        </ItemMedia>
        <ItemContent>
          <ItemTitle>{event.company_name}</ItemTitle>
          {event.contact && (
            <ItemDescription className="font-mono">{event.contact.email}</ItemDescription>
          )}
          {event.outcome.detail && (
            <ItemDescription className="text-destructive">{event.outcome.detail}</ItemDescription>
          )}
        </ItemContent>
        <ItemActions>
          {event.contact && <OriginBadge origin={event.contact.origin} />}
          {event.email && (
            <CollapsibleTrigger asChild>
              <Button variant="ghost" size="sm" className="group">
                Email
                <ChevronDownIcon className="transition-transform group-data-[state=open]:rotate-180" />
              </Button>
            </CollapsibleTrigger>
          )}
        </ItemActions>
        {event.email && (
          <CollapsibleContent asChild>
            <ItemFooter>
              <Textarea
                readOnly
                aria-label={`Email preview for ${event.company_name}`}
                rows={12}
                className="font-mono text-xs leading-relaxed"
                value={emailText(event)}
              />
            </ItemFooter>
          </CollapsibleContent>
        )}
      </Item>
    </Collapsible>
  )
}

export function HuntEvents({ events, className }: { events: THuntEvent[]; className?: string }) {
  const newestFirst = events.slice().reverse()
  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle>Activity</CardTitle>
        <CardDescription>Newest first. Expand a row to read the email.</CardDescription>
      </CardHeader>
      <CardContent>
        {newestFirst.length ? (
          <ItemGroup>
            {newestFirst.map((event, index) => (
              <EventEntry
                key={`${event.slug}-${index}`}
                event={event}
                last={index === newestFirst.length - 1}
              />
            ))}
          </ItemGroup>
        ) : (
          <EmptyState
            icon={RadarIcon}
            title="Waiting for the first startup"
            description="Events appear here as each startup is processed."
          />
        )}
      </CardContent>
    </Card>
  )
}

function EventEntry({ event, last }: { event: THuntEvent; last: boolean }) {
  return (
    <>
      <EventItem event={event} />
      {!last && <ItemSeparator />}
    </>
  )
}
