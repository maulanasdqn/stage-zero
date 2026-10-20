import { createFileRoute } from '@tanstack/react-router'
import { useCallback } from 'react'
import { ApplicationsPage } from '@/components/features/applications/applications-page'
import {
  applicationsSearchSchema,
  type TApplicationsSearch,
} from '@/components/features/applications/applications-search'

export const Route = createFileRoute('/applications')({
  validateSearch: applicationsSearchSchema,
  component: ApplicationsRoute,
})

function ApplicationsRoute() {
  const search = Route.useSearch()
  const navigate = Route.useNavigate()
  const onSearchChange = useCallback(
    (patch: Partial<TApplicationsSearch>) =>
      navigate({ search: (previous) => ({ ...previous, ...patch }), replace: true }),
    [navigate],
  )
  return <ApplicationsPage search={search} onSearchChange={onSearchChange} />
}
