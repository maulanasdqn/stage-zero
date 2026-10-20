import { useQueryClient, type QueryKey } from '@tanstack/react-query'
import { useEffect } from 'react'
import type { TPaginationMeta } from '@/types/pagination'

type TPrefetchOptions = {
  pagination: TPaginationMeta | undefined
  queryKey: (page: number) => QueryKey
  queryFn: (page: number) => Promise<unknown>
  staleTime?: number
}

export function usePrefetchNextPage({
  pagination,
  queryKey,
  queryFn,
  staleTime,
}: TPrefetchOptions) {
  const queryClient = useQueryClient()
  const nextPage =
    pagination && pagination.page < pagination.total_pages ? pagination.page + 1 : null

  useEffect(() => {
    if (nextPage === null) {
      return
    }
    void queryClient.prefetchQuery({
      queryKey: queryKey(nextPage),
      queryFn: () => queryFn(nextPage),
      staleTime,
    })
  }, [nextPage, queryClient, queryKey, queryFn, staleTime])
}
