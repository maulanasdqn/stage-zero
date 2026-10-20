import type { SortingState } from '@tanstack/react-table'
import type { TSortOrder } from '@/types/pagination'

export function toSortingState(sort?: string, order?: TSortOrder): SortingState {
  return sort ? [{ id: sort, desc: order === 'desc' }] : []
}

export function fromSortingState<TSort extends string>(
  sorting: SortingState,
): { sort?: TSort; order?: TSortOrder } {
  const [first] = sorting
  if (!first) {
    return { sort: undefined, order: undefined }
  }
  return { sort: first.id as TSort, order: first.desc ? 'desc' : 'asc' }
}
