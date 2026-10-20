import type { TApplicationStatus } from '@/apis/startups/types'
import type { TListResponse, TPageParams } from '@/types/pagination'

export type TApplication = {
  id: number
  yc_slug: string
  company_name: string
  website: string | null
  contact_email: string | null
  status: TApplicationStatus
  applied_at: string
}

export type TApplicationSummary = {
  total: number
  applied: number
  skipped: number
  failed_no_contact: number
}

export type TApplicationSort = 'applied_at' | 'company_name' | 'status'

export type TApplicationParams = TPageParams & {
  status?: TApplicationStatus
  sort?: TApplicationSort
}

export type TApplicationsResponse = TListResponse<TApplication>

export type TResetResponse = {
  removed: number
}
