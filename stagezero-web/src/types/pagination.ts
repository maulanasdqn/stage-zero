export type TPaginationMeta = {
  page: number
  per_page: number
  total: number
  total_pages: number
}

export type TListResponse<T> = {
  data: T[]
  pagination: TPaginationMeta
}

export type TSortOrder = 'asc' | 'desc'

export type TPageParams = {
  page?: number
  per_page?: number
  q?: string
  order?: TSortOrder
}
