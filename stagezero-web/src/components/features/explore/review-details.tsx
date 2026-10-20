import { CheckCircle2Icon, MailIcon, SparklesIcon, TriangleAlertIcon } from 'lucide-react'
import type { TFounder, TStartupInsight } from '@/apis/startups'
import { OriginBadge } from '@/components/features/shared/origin-badge'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Badge } from '@/components/ui/badge'
import { ButtonGroup } from '@/components/ui/button-group'
import { FieldLegend, FieldSet } from '@/components/ui/field'
import {
  Item,
  ItemActions,
  ItemContent,
  ItemDescription,
  ItemGroup,
  ItemMedia,
  ItemTitle,
} from '@/components/ui/item'
import { humanize } from '@/libs/format'

function initials(founder: TFounder): string {
  return founder.full_name
    .split(/\s+/)
    .slice(0, 2)
    .map((part) => part.charAt(0).toUpperCase())
    .join('')
}

function Notices({ insight }: { insight: TStartupInsight }) {
  return (
    <>
      {insight.status === 'APPLIED' && (
        <Alert>
          <CheckCircle2Icon />
          <AlertTitle>Already applied</AlertTitle>
          <AlertDescription>You have already sent an application to this startup.</AlertDescription>
        </Alert>
      )}
      {insight.status && insight.status !== 'APPLIED' && (
        <Alert>
          <TriangleAlertIcon />
          <AlertTitle>Tracked as {humanize(insight.status)}</AlertTitle>
          <AlertDescription>Sending will mark it as applied.</AlertDescription>
        </Alert>
      )}
      {insight.contact?.origin === 'guessed' && (
        <Alert>
          <TriangleAlertIcon />
          <AlertTitle>Guessed address</AlertTitle>
          <AlertDescription>
            Based on the founder name. Guesses can bounce and hurt your sender reputation.
          </AlertDescription>
        </Alert>
      )}
    </>
  )
}

export function ReviewDetails({ insight }: { insight: TStartupInsight }) {
  const { startup } = insight
  return (
    <>
      <Notices insight={insight} />
      <Item variant="muted">
        <ItemContent>
          <ItemDescription className="line-clamp-none">
            {startup.description || startup.one_liner || 'No description provided.'}
          </ItemDescription>
        </ItemContent>
      </Item>
      <FieldSet>
        <FieldLegend variant="label">Founders</FieldLegend>
        {insight.founders.length ? (
          <ItemGroup className="grid gap-2 sm:grid-cols-2">
            {insight.founders.map((founder) => (
              <Item key={founder.full_name} variant="outline" size="sm">
                <ItemMedia>
                  <Avatar>
                    <AvatarFallback>{initials(founder)}</AvatarFallback>
                  </Avatar>
                </ItemMedia>
                <ItemContent>
                  <ItemTitle>{founder.full_name}</ItemTitle>
                  {founder.title && <ItemDescription>{founder.title}</ItemDescription>}
                </ItemContent>
              </Item>
            ))}
          </ItemGroup>
        ) : (
          <ItemDescription>No founders listed on YC.</ItemDescription>
        )}
      </FieldSet>
      <FieldSet>
        <FieldLegend variant="label">Contact</FieldLegend>
        {insight.contact ? (
          <Item variant="outline" size="sm">
            <ItemMedia variant="icon">
              <MailIcon />
            </ItemMedia>
            <ItemContent>
              <ItemTitle className="font-mono">{insight.contact.email}</ItemTitle>
              {insight.contact.founder && (
                <ItemDescription>matches {insight.contact.founder.full_name}</ItemDescription>
              )}
            </ItemContent>
            <ItemActions>
              <OriginBadge origin={insight.contact.origin} />
            </ItemActions>
          </Item>
        ) : (
          <ItemDescription>
            No email found on the website. Type one below or skip this startup.
          </ItemDescription>
        )}
      </FieldSet>
      <FieldSet>
        <FieldLegend variant="label">Highlights picked for this startup</FieldLegend>
        <ItemGroup className="gap-2">
          {insight.highlights.map((highlight) => (
            <Item key={highlight.text} variant="outline" size="sm" className="items-start">
              <ItemMedia variant="icon">
                <SparklesIcon className="text-primary" />
              </ItemMedia>
              <ItemContent>
                <ItemDescription className="text-foreground line-clamp-none">
                  {highlight.text}
                </ItemDescription>
                <ButtonGroup className="mt-1 flex-wrap gap-1">
                  {highlight.pinned && <Badge variant="secondary">Pinned</Badge>}
                  {highlight.matched.map((keyword) => (
                    <Badge key={keyword} variant="outline">
                      {keyword}
                    </Badge>
                  ))}
                  {!highlight.pinned && !highlight.matched.length && (
                    <Badge variant="ghost">Fallback, no keyword match</Badge>
                  )}
                </ButtonGroup>
              </ItemContent>
            </Item>
          ))}
        </ItemGroup>
      </FieldSet>
    </>
  )
}
