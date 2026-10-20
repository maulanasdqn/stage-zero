import { createFileRoute } from '@tanstack/react-router'
import { HuntPage } from '@/components/features/hunt/hunt-page'

export const Route = createFileRoute('/hunt')({
  component: HuntPage,
})
