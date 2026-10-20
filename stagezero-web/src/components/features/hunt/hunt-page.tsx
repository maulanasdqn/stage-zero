import { RadarIcon } from 'lucide-react'
import { useHuntRun } from '@/apis/hunt'
import { useSettings } from '@/apis/settings'
import { useBatches } from '@/apis/startups'
import { EmptyState } from '@/components/features/shared/empty-state'
import { PAGE_GUTTER } from '@/components/layout/page-gutter'
import { PageHeader } from '@/components/layout/page-header'
import { Card } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { cn } from '@/lib/utils'
import { HuntEvents } from './hunt-events'
import { HuntForm } from './hunt-form'
import { HuntReport } from './hunt-report'
import { HuntStatus } from './hunt-status'

export function HuntPage() {
  const batches = useBatches()
  const settings = useSettings()
  const run = useHuntRun()
  const defaultBatch = batches.data?.find((batch) => batch.count >= 20)?.slug ?? ''
  const canSend = Boolean(settings.data?.mailer_configured && settings.data.resume_loaded)

  return (
    <>
      <PageHeader
        className={cn(PAGE_GUTTER, 'mt-6')}
        title="Auto hunt"
        description="Run a whole batch at once. Start with a dry run to check the emails before sending."
      />
      {batches.data ? (
        <HuntForm
          key={defaultBatch}
          className={PAGE_GUTTER}
          batches={batches.data}
          defaultBatch={defaultBatch}
          running={Boolean(run.data?.running)}
          canSend={canSend}
        />
      ) : (
        <Skeleton className={cn(PAGE_GUTTER, 'h-48')} />
      )}
      {run.data?.criteria ? (
        <>
          <HuntStatus run={run.data} className={PAGE_GUTTER} />
          {run.data.report && <HuntReport report={run.data.report} className={PAGE_GUTTER} />}
          <HuntEvents events={run.data.events} className={cn(PAGE_GUTTER, 'mb-6')} />
        </>
      ) : (
        <Card className={cn(PAGE_GUTTER, 'mb-6 py-0')}>
          <EmptyState
            icon={RadarIcon}
            title="No hunt yet"
            description="Hunts started since the server came up show their progress here."
          />
        </Card>
      )}
    </>
  )
}
