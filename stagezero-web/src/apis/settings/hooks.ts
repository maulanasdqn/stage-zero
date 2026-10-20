import { useQuery } from '@tanstack/react-query'
import { settingsService } from './service'

export const settingsKeys = {
  all: ['settings'] as const,
}

export function useSettings() {
  return useQuery({
    queryKey: settingsKeys.all,
    queryFn: () => settingsService.get().then((res) => res.data),
    staleTime: 30_000,
  })
}
