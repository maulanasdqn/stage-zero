import { ClipboardListIcon, InboxIcon, SendIcon, SkipForwardIcon, UserXIcon } from 'lucide-react'
import { useCallback, useMemo, useState } from 'react'
import { toast } from 'sonner'
import {
  useApplicationSummary,
  useApplications,
  useRemoveApplications,
  type TApplication,
  type TApplicationSort,
} from '@/apis/applications'
import { ConfirmDialog } from '@/components/features/shared/confirm-dialog'
import { DataTable } from '@/components/features/shared/data-table/data-table'
import { fromSortingState, toSortingState } from '@/components/features/shared/data-table/sorting'
import { EmptyState } from '@/components/features/shared/empty-state'
import { StatGrid } from '@/components/features/shared/stat-grid'
import { PAGE_GUTTER } from '@/components/layout/page-gutter'
import { PageHeader } from '@/components/layout/page-header'
import { Card } from '@/components/ui/card'
import { errorMessage } from '@/libs/axios'
import { cn } from '@/lib/utils'
import { applicationColumns } from './application-columns'
import type { TApplicationsSearch } from './applications-search'
import { ApplicationsToolbar } from './applications-toolbar'

type TApplicationsPageProps = {
  search: TApplicationsSearch
  onSearchChange: (patch: Partial<TApplicationsSearch>) => void
}

const NO_APPLICATIONS: TApplication[] = []

type TPendingRemoval = { kind: 'one'; application: TApplication } | { kind: 'all' }

export function ApplicationsPage({ search, onSearchChange }: TApplicationsPageProps) {
  const summary = useApplicationSummary()
  const params = useMemo(
    () => ({
      status: search.status,
      q: search.q || undefined,
      sort: search.sort,
      order: search.order,
      page: search.page ?? 1,
      per_page: search.per_page ?? 20,
    }),
    [search.status, search.q, search.sort, search.order, search.page, search.per_page],
  )
  const applications = useApplications(params)
  const remove = useRemoveApplications()
  const [pending, setPending] = useState<TPendingRemoval | null>(null)
  const columns = useMemo(
    () => applicationColumns((application) => setPending({ kind: 'one', application })),
    [],
  )
  const sorting = useMemo(
    () => toSortingState(search.sort, search.order),
    [search.sort, search.order],
  )
  const setQuery = useCallback(
    (q: string) => onSearchChange({ q: q || undefined, page: undefined }),
    [onSearchChange],
  )

  const confirmRemoval = () => {
    if (!pending) {
      return
    }
    const slug = pending.kind === 'one' ? pending.application.yc_slug : undefined
    remove.mutate(slug, {
      onSuccess: (result) =>
        toast(`Removed ${result.removed} record${result.removed === 1 ? '' : 's'}`),
      onError: (error) => toast.error('Could not remove', { description: errorMessage(error) }),
    })
    setPending(null)
  }

  const filtered = Boolean(search.status || search.q)

  return (
    <>
      <PageHeader
        className={cn(PAGE_GUTTER, 'mt-6')}
        title="Applications"
        description="Every startup you applied to, skipped or could not reach. Tracked startups are never emailed twice."
      />
      <StatGrid
        className={PAGE_GUTTER}
        stats={[
          { label: 'Tracked', value: summary.data?.total, icon: ClipboardListIcon },
          { label: 'Applied', value: summary.data?.applied, icon: SendIcon },
          { label: 'Skipped', value: summary.data?.skipped, icon: SkipForwardIcon },
          { label: 'No contact', value: summary.data?.failed_no_contact, icon: UserXIcon },
        ]}
      />
      <Card className={cn(PAGE_GUTTER, 'mb-6 gap-0 py-0')}>
        <ApplicationsToolbar
          search={search.q ?? ''}
          onSearchChange={setQuery}
          status={search.status}
          onStatusChange={(status) => onSearchChange({ status, page: undefined })}
          canReset={Boolean(summary.data?.total)}
          onReset={() => setPending({ kind: 'all' })}
        />
        <DataTable
          columns={columns}
          data={applications.data?.data ?? NO_APPLICATIONS}
          pagination={applications.data?.pagination}
          sorting={sorting}
          onSortingChange={(next) =>
            onSearchChange({ ...fromSortingState<TApplicationSort>(next), page: undefined })
          }
          onPageChange={(page) => onSearchChange({ page })}
          onPageSizeChange={(perPage) => onSearchChange({ per_page: perPage, page: undefined })}
          getRowId={(application) => application.yc_slug}
          loading={applications.isPending}
          fetching={applications.isPlaceholderData}
          empty={
            <EmptyState
              icon={InboxIcon}
              title={filtered ? 'Nothing matches these filters' : 'No applications yet'}
              description={
                filtered
                  ? 'Clear the search or pick another status.'
                  : 'Review a startup in Explore or run a hunt to start tracking outreach.'
              }
            />
          }
        />
      </Card>
      <ConfirmDialog
        open={pending !== null}
        onOpenChange={(open) => !open && setPending(null)}
        title={pending?.kind === 'all' ? 'Reset the whole tracker?' : 'Remove this record?'}
        description={
          pending?.kind === 'one'
            ? `${pending.application.company_name} becomes available again and could be emailed in a future hunt.`
            : 'Every tracked startup becomes available again. Startups you already applied to could be emailed twice.'
        }
        confirmLabel={pending?.kind === 'all' ? 'Reset everything' : 'Remove'}
        destructive
        onConfirm={confirmRemoval}
      />
    </>
  )
}
