import type { TListResponse, TPageParams } from '@/types/pagination'

export type TApplicationStatus = 'APPLIED' | 'SKIPPED' | 'FAILED_NO_CONTACT'

export type TStartup = {
  slug: string
  name: string
  website: string | null
  batch: string
  team_size: number | null
  industries: string[]
  one_liner: string
  tags: string[]
  description: string
}

export type TBatch = {
  slug: string
  name: string
  count: number
}

export type TDirectoryEntry = {
  startup: TStartup
  status: TApplicationStatus | null
}

export type TStartupSort = 'name' | 'team_size'

export type TExploreParams = TPageParams & {
  batch: string
  size: string
  industry?: string
  hide_contacted?: boolean
  sort?: TStartupSort
}

export type TExploreResponse = TListResponse<TDirectoryEntry> & {
  batch_total: number
  tracked: number
}

export type TFounder = {
  full_name: string
  title: string | null
}

export type TContactOrigin = 'scraped' | 'guessed'

export type TContact = {
  email: string
  founder: TFounder | null
  origin: TContactOrigin
}

export type TEmailPreview = {
  to: string
  subject: string
  body: string
}

export type THighlightMatch = {
  text: string
  score: number
  matched: string[]
  pinned: boolean
}

export type TStartupInsight = {
  startup: TStartup
  highlights: THighlightMatch[]
  founders: TFounder[]
  contact: TContact | null
  email: TEmailPreview
  status: TApplicationStatus | null
}

export type THuntOutcomeKind =
  | 'already_processed'
  | 'skipped_no_website'
  | 'no_contact'
  | 'previewed'
  | 'applied'
  | 'send_failed'

export type THuntOutcome = {
  kind: THuntOutcomeKind
  detail?: string
}

export type TSendPayload = {
  startup: TStartup
  email: TEmailPreview
}

export type TSendResponse = {
  outcome: THuntOutcome
}
