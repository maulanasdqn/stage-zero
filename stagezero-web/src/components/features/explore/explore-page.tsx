import { SearchXIcon, TelescopeIcon } from 'lucide-react'
import { useCallback, useMemo, useState } from 'react'
import {
  useBatches,
  useStartups,
  type TDirectoryEntry,
  type TStartup,
  type TStartupSort,
} from '@/apis/startups'
import { DataTable } from '@/components/features/shared/data-table/data-table'
import { fromSortingState, toSortingState } from '@/components/features/shared/data-table/sorting'
import { EmptyState } from '@/components/features/shared/empty-state'
import { PageHeader } from '@/components/layout/page-header'
import { PAGE_GUTTER } from '@/components/layout/page-gutter'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Card } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { errorMessage } from '@/libs/axios'
import { cn } from '@/lib/utils'
import { ExploreFilters } from './explore-filters'
import type { TExploreSearch } from './explore-search'
import { ReviewSheet } from './review-sheet'
import { startupColumns } from './startup-columns'
import { StartupToolbar } from './startup-toolbar'

const NO_ENTRIES: TDirectoryEntry[] = []

type TExplorePageProps = {
  search: TExploreSearch
  onSearchChange: (patch: Partial<TExploreSearch>) => void
}

export function ExplorePage({ search, onSearchChange }: TExplorePageProps) {
  const batches = useBatches()
  const [selected, setSelected] = useState<TStartup | null>(null)

  const defaultBatch = batches.data?.find((batch) => batch.count >= 20)?.slug ?? ''
  const batch = search.batch ?? defaultBatch
  const min = search.min ?? 1
  const max = search.max ?? 10
  const industry = search.industry ?? ''

  const params = useMemo(
    () =>
      batch
        ? {
            batch,
            size: `${min}-${max}`,
            industry: industry || undefined,
            q: search.q || undefined,
            hide_contacted: search.hide || undefined,
            sort: search.sort,
            order: search.order,
            page: search.page ?? 1,
            per_page: search.per_page ?? 20,
          }
        : null,
    [
      batch,
      min,
      max,
      industry,
      search.q,
      search.hide,
      search.sort,
      search.order,
      search.page,
      search.per_page,
    ],
  )
  const startups = useStartups(params)
  const result = startups.data
  const sorting = useMemo(
    () => toSortingState(search.sort, search.order),
    [search.sort, search.order],
  )
  const setQuery = useCallback(
    (q: string) => onSearchChange({ q: q || undefined, page: undefined }),
    [onSearchChange],
  )

  return (
    <>
      <PageHeader
        className={cn(PAGE_GUTTER, 'mt-6')}
        title="Explore YC startups"
        description="Scrape a batch, then open any startup to find its founders and review your email."
      />
      {batches.data ? (
        <ExploreFilters
          key={defaultBatch}
          className={PAGE_GUTTER}
          batches={batches.data}
          initial={{ batch, min, max, industry }}
          loading={startups.isFetching}
          onSubmit={(value) =>
            onSearchChange({ ...value, industry: value.industry || undefined, page: undefined })
          }
        />
      ) : (
        <Skeleton className={cn(PAGE_GUTTER, 'h-20')} />
      )}
      {startups.isError ? (
        <Alert variant="destructive" className={cn(PAGE_GUTTER, 'w-auto')}>
          <AlertTitle>Could not load this batch</AlertTitle>
          <AlertDescription>{errorMessage(startups.error)}</AlertDescription>
        </Alert>
      ) : (
        <>
          <StartupToolbar
            className={PAGE_GUTTER}
            search={search.q ?? ''}
            onSearchChange={setQuery}
            hideContacted={Boolean(search.hide)}
            onHideContactedChange={(hide) =>
              onSearchChange({ hide: hide || undefined, page: undefined })
            }
            matched={result?.pagination.total}
            batchTotal={result?.batch_total}
            tracked={result?.tracked}
          />
          <Card className={cn(PAGE_GUTTER, 'mb-6 gap-0 py-0')}>
            <DataTable
              columns={startupColumns}
              data={result?.data ?? NO_ENTRIES}
              pagination={result?.pagination}
              sorting={sorting}
              onSortingChange={(next) =>
                onSearchChange({ ...fromSortingState<TStartupSort>(next), page: undefined })
              }
              onPageChange={(page) => onSearchChange({ page })}
              onPageSizeChange={(perPage) => onSearchChange({ per_page: perPage, page: undefined })}
              getRowId={(entry) => entry.startup.slug}
              loading={startups.isPending}
              fetching={startups.isPlaceholderData}
              onRowClick={(entry) => setSelected(entry.startup)}
              empty={
                <EmptyState
                  icon={result?.batch_total ? SearchXIcon : TelescopeIcon}
                  title={result?.batch_total ? 'No startups match' : 'No startups in this batch'}
                  description={
                    result?.batch_total
                      ? 'Try a different search or show contacted startups again.'
                      : 'Widen the team size range or clear the industry keywords.'
                  }
                />
              }
            />
          </Card>
        </>
      )}
      <ReviewSheet startup={selected} onClose={() => setSelected(null)} />
    </>
  )
}
