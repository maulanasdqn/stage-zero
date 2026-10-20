import { apiClient } from '@/libs/axios'
import type { TSettings } from './types'

export const settingsService = {
  get: () => apiClient.get<TSettings>('/settings'),
}
