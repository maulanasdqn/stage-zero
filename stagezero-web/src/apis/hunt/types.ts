import type { TContact, TEmailPreview, THuntOutcome } from '@/apis/startups/types'

export type TTeamSizeRange = {
  min: number
  max: number
}

export type THuntCriteria = {
  batch: string
  team_size: TTeamSizeRange
  industry: string | null
  limit: number | null
  dry_run: boolean
}

export type THuntEvent = {
  slug: string
  company_name: string
  outcome: THuntOutcome
  contact: TContact | null
  email: TEmailPreview | null
}

export type THuntReport = {
  discovered: number
  already_processed: number
  skipped_no_website: number
  no_contact: number
  previewed: number
  applied: number
  send_failed: number
  daily_limit_reached: boolean
}

export type THuntRun = {
  running: boolean
  criteria: THuntCriteria | null
  discovered: number | null
  events: THuntEvent[]
  report: THuntReport | null
  error: string | null
}

export type TStartHuntPayload = {
  batch: string
  size: string
  industry?: string
  limit?: number
  dry_run: boolean
}
