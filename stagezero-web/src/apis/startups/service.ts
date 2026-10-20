import { apiClient } from '@/libs/axios'
import type {
  TBatch,
  TExploreParams,
  TExploreResponse,
  TSendPayload,
  TSendResponse,
  TStartup,
  TStartupInsight,
} from './types'

export const startupsService = {
  batches: () => apiClient.get<TBatch[]>('/batches'),
  explore: (params: TExploreParams) => apiClient.get<TExploreResponse>('/startups', { params }),
  inspect: (startup: TStartup) => apiClient.post<TStartupInsight>('/startups/inspect', { startup }),
  send: (payload: TSendPayload) => apiClient.post<TSendResponse>('/startups/send', payload),
  skip: (startup: TStartup) => apiClient.post('/startups/skip', { startup }),
}
