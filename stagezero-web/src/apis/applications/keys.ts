import type { TApplicationParams } from './types'

export const applicationKeys = {
  all: ['applications'] as const,
  lists: () => [...applicationKeys.all, 'list'] as const,
  list: (params: TApplicationParams) => [...applicationKeys.lists(), params] as const,
  summary: () => [...applicationKeys.all, 'summary'] as const,
}
