import { createFileRoute } from '@tanstack/react-router'
import { useCallback } from 'react'
import { ExplorePage } from '@/components/features/explore/explore-page'
import {
  exploreSearchSchema,
  type TExploreSearch,
} from '@/components/features/explore/explore-search'

export const Route = createFileRoute('/explore')({
  validateSearch: exploreSearchSchema,
  component: ExploreRoute,
})

function ExploreRoute() {
  const search = Route.useSearch()
  const navigate = Route.useNavigate()
  const onSearchChange = useCallback(
    (patch: Partial<TExploreSearch>) =>
      navigate({ search: (previous) => ({ ...previous, ...patch }), replace: true }),
    [navigate],
  )
  return <ExplorePage search={search} onSearchChange={onSearchChange} />
}
