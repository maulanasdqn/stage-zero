import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback } from 'react'
import { usePrefetchNextPage } from '@/hooks/use-prefetch-next-page'
import { startupKeys } from '@/apis/startups/hooks'
import { applicationKeys } from './keys'
import { applicationsService } from './service'
import type { TApplicationParams } from './types'

const APPLICATIONS_STALE_TIME = 30_000

function fetchApplications(params: TApplicationParams) {
  return applicationsService.list(params).then((res) => res.data)
}

export function useApplications(params: TApplicationParams) {
  const query = useQuery({
    queryKey: applicationKeys.list(params),
    queryFn: () => fetchApplications(params),
    placeholderData: keepPreviousData,
    staleTime: APPLICATIONS_STALE_TIME,
  })
  const pageKey = useCallback((page: number) => applicationKeys.list({ ...params, page }), [params])
  const pageFn = useCallback((page: number) => fetchApplications({ ...params, page }), [params])
  usePrefetchNextPage({
    pagination: query.data?.pagination,
    queryKey: pageKey,
    queryFn: pageFn,
    staleTime: APPLICATIONS_STALE_TIME,
  })
  return query
}

export function useApplicationSummary() {
  return useQuery({
    queryKey: applicationKeys.summary(),
    queryFn: () => applicationsService.summary().then((res) => res.data),
  })
}

export function useRemoveApplications() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (slug?: string) => applicationsService.remove(slug).then((res) => res.data),
    onSuccess: () =>
      Promise.all([
        queryClient.invalidateQueries({ queryKey: applicationKeys.all }),
        queryClient.invalidateQueries({ queryKey: startupKeys.lists() }),
        queryClient.invalidateQueries({ queryKey: startupKeys.insights() }),
      ]),
  })
}
