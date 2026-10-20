import { useRouterState } from '@tanstack/react-router'
import { useHuntRun } from '@/apis/hunt'
import { Badge } from '@/components/ui/badge'
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@/components/ui/breadcrumb'
import { Item, ItemActions, ItemContent, ItemMedia } from '@/components/ui/item'
import { Separator } from '@/components/ui/separator'
import { SidebarTrigger } from '@/components/ui/sidebar'
import { Spinner } from '@/components/ui/spinner'
import { ThemeToggle } from './theme-toggle'

const TITLES: Record<string, string> = {
  '/explore': 'Explore',
  '/hunt': 'Auto hunt',
  '/applications': 'Applications',
}

export function AppTopbar() {
  const pathname = useRouterState({ select: (state) => state.location.pathname })
  const { data: run } = useHuntRun()

  return (
    <Item className="bg-background/80 sticky top-0 z-30 rounded-none border-b px-4 backdrop-blur">
      <ItemMedia>
        <SidebarTrigger />
        <Separator orientation="vertical" className="mx-1 data-[orientation=vertical]:h-4" />
      </ItemMedia>
      <ItemContent>
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem>StageZero</BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
              <BreadcrumbPage>{TITLES[pathname] ?? 'StageZero'}</BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </ItemContent>
      <ItemActions>
        {run?.running && (
          <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary">
            <Spinner />
            Hunting
          </Badge>
        )}
        <ThemeToggle />
      </ItemActions>
    </Item>
  )
}
