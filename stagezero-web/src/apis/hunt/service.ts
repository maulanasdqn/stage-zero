import { apiClient } from '@/libs/axios'
import type { THuntRun, TStartHuntPayload } from './types'

export const huntService = {
  status: () => apiClient.get<THuntRun>('/hunt'),
  start: (payload: TStartHuntPayload) => apiClient.post('/hunt', payload),
}
