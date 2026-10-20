import { useStartupInsight, type TStartup } from '@/apis/startups'
import { ExternalLink } from '@/components/features/shared/external-link'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { ButtonGroup } from '@/components/ui/button-group'
import { FieldGroup } from '@/components/ui/field'
import { Item, ItemContent, ItemMedia, ItemTitle } from '@/components/ui/item'
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from '@/components/ui/sheet'
import { Skeleton } from '@/components/ui/skeleton'
import { Spinner } from '@/components/ui/spinner'
import { errorMessage } from '@/libs/axios'
import { displayUrl, safeUrl, ycCompanyUrl } from '@/libs/format'
import { ReviewDetails } from './review-details'
import { ReviewEmailForm } from './review-email-form'

type TReviewSheetProps = {
  startup: TStartup | null
  onClose: () => void
}

function LoadingState() {
  return (
    <FieldGroup className="gap-4 p-4">
      <Item variant="muted" size="sm">
        <ItemMedia>
          <Spinner />
        </ItemMedia>
        <ItemContent>
          <ItemTitle>Finding founders and contact emails…</ItemTitle>
        </ItemContent>
      </Item>
      <Skeleton className="h-16 w-full" />
      <Skeleton className="h-24 w-full" />
      <Skeleton className="h-64 w-full" />
    </FieldGroup>
  )
}

export function ReviewSheet({ startup, onClose }: TReviewSheetProps) {
  const insight = useStartupInsight(startup)
  const website = safeUrl(startup?.website)

  return (
    <Sheet open={startup !== null} onOpenChange={(open) => !open && onClose()}>
      <SheetContent className="w-full gap-0 p-0 data-[side=right]:w-full data-[side=right]:sm:max-w-2xl">
        <SheetHeader className="border-b">
          <SheetTitle>{startup?.name}</SheetTitle>
          <SheetDescription>{startup?.one_liner}</SheetDescription>
          <ButtonGroup className="flex-wrap items-center gap-3">
            {startup && <Badge variant="secondary">{startup.batch}</Badge>}
            {startup?.team_size && <Badge variant="outline">{startup.team_size} people</Badge>}
            {website && <ExternalLink href={website}>{displayUrl(website)}</ExternalLink>}
            {startup && <ExternalLink href={ycCompanyUrl(startup.slug)}>YC page</ExternalLink>}
          </ButtonGroup>
        </SheetHeader>
        {insight.isPending && <LoadingState />}
        {insight.isError && (
          <FieldGroup className="p-4">
            <Alert variant="destructive">
              <AlertTitle>Could not inspect this startup</AlertTitle>
              <AlertDescription>{errorMessage(insight.error)}</AlertDescription>
            </Alert>
          </FieldGroup>
        )}
        {insight.data && (
          <ReviewEmailForm key={insight.data.startup.slug} insight={insight.data} onDone={onClose}>
            <ReviewDetails insight={insight.data} />
          </ReviewEmailForm>
        )}
      </SheetContent>
    </Sheet>
  )
}
