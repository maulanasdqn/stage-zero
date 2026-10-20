import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { applicationKeys } from '@/apis/applications/keys'
import { settingsKeys } from '@/apis/settings/hooks'
import { startupKeys } from '@/apis/startups/hooks'
import { huntService } from './service'

export const huntKeys = {
  all: ['hunt'] as const,
  status: () => [...huntKeys.all, 'status'] as const,
}

export function useHuntRun() {
  const queryClient = useQueryClient()
  return useQuery({
    queryKey: huntKeys.status(),
    queryFn: async () => {
      const run = await huntService.status().then((res) => res.data)
      if (!run.running && run.report) {
        await Promise.all([
          queryClient.invalidateQueries({ queryKey: applicationKeys.all }),
          queryClient.invalidateQueries({ queryKey: startupKeys.lists() }),
        ])
      }
      return run
    },
    refetchInterval: (query) => (query.state.data?.running ? 1500 : false),
  })
}

export function useStartHunt() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: huntService.start,
    onSuccess: () =>
      Promise.all([
        queryClient.invalidateQueries({ queryKey: huntKeys.all }),
        queryClient.invalidateQueries({ queryKey: settingsKeys.all }),
      ]),
  })
}
