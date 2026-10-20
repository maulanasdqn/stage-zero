import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback } from 'react'
import { usePrefetchNextPage } from '@/hooks/use-prefetch-next-page'
import { applicationKeys } from '@/apis/applications/keys'
import { startupsService } from './service'
import type { TExploreParams, TStartup } from './types'

export const startupKeys = {
  all: ['startups'] as const,
  batches: () => [...startupKeys.all, 'batches'] as const,
  lists: () => [...startupKeys.all, 'list'] as const,
  list: (params: TExploreParams) => [...startupKeys.lists(), params] as const,
  insights: () => [...startupKeys.all, 'insight'] as const,
  insight: (slug: string) => [...startupKeys.insights(), slug] as const,
}

export function useBatches() {
  return useQuery({
    queryKey: startupKeys.batches(),
    queryFn: () => startupsService.batches().then((res) => res.data),
    staleTime: 60 * 60_000,
  })
}

const EXPLORE_STALE_TIME = 5 * 60_000

function fetchStartups(params: TExploreParams) {
  return startupsService.explore(params).then((res) => res.data)
}

export function useStartups(params: TExploreParams | null) {
  const query = useQuery({
    queryKey: params ? startupKeys.list(params) : startupKeys.lists(),
    queryFn: () => fetchStartups(params as TExploreParams),
    enabled: params !== null,
    placeholderData: keepPreviousData,
    staleTime: EXPLORE_STALE_TIME,
  })
  const pageKey = useCallback(
    (page: number) => startupKeys.list({ ...(params as TExploreParams), page }),
    [params],
  )
  const pageFn = useCallback(
    (page: number) => fetchStartups({ ...(params as TExploreParams), page }),
    [params],
  )
  usePrefetchNextPage({
    pagination: params ? query.data?.pagination : undefined,
    queryKey: pageKey,
    queryFn: pageFn,
    staleTime: EXPLORE_STALE_TIME,
  })
  return query
}

export function useStartupInsight(startup: TStartup | null) {
  return useQuery({
    queryKey: startupKeys.insight(startup?.slug ?? ''),
    queryFn: () => startupsService.inspect(startup as TStartup).then((res) => res.data),
    enabled: startup !== null,
    staleTime: 10 * 60_000,
  })
}

function useInvalidateOutreach() {
  const queryClient = useQueryClient()
  return () =>
    Promise.all([
      queryClient.invalidateQueries({ queryKey: startupKeys.lists() }),
      queryClient.invalidateQueries({ queryKey: startupKeys.insights() }),
      queryClient.invalidateQueries({ queryKey: applicationKeys.all }),
    ])
}

export function useSendApplication() {
  const invalidate = useInvalidateOutreach()
  return useMutation({
    mutationFn: startupsService.send,
    onSuccess: invalidate,
  })
}

export function useSkipStartup() {
  const invalidate = useInvalidateOutreach()
  return useMutation({
    mutationFn: startupsService.skip,
    onSuccess: invalidate,
  })
}
