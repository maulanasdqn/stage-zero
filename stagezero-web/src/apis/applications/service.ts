import { apiClient } from '@/libs/axios'
import type {
  TApplicationParams,
  TApplicationsResponse,
  TApplicationSummary,
  TResetResponse,
} from './types'

export const applicationsService = {
  list: (params: TApplicationParams) =>
    apiClient.get<TApplicationsResponse>('/applications', { params }),
  summary: () => apiClient.get<TApplicationSummary>('/applications/summary'),
  remove: (slug?: string) =>
    apiClient.delete<TResetResponse>('/applications', { params: slug ? { slug } : {} }),
}
