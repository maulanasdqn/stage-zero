import { Link, useRouterState } from '@tanstack/react-router'
import { ClipboardListIcon, RadarIcon, SearchIcon } from 'lucide-react'
import { useSettings } from '@/apis/settings'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Item, ItemContent, ItemDescription, ItemMedia, ItemTitle } from '@/components/ui/item'
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from '@/components/ui/sidebar'
import { StatusIndicators } from './status-indicators'

const LINKS = [
  { to: '/explore', label: 'Explore', icon: SearchIcon },
  { to: '/hunt', label: 'Auto hunt', icon: RadarIcon },
  { to: '/applications', label: 'Applications', icon: ClipboardListIcon },
] as const

export function AppSidebar() {
  const pathname = useRouterState({ select: (state) => state.location.pathname })
  const { data: settings } = useSettings()

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton size="lg" asChild>
              <Link to="/explore">
                <Avatar className="rounded-md">
                  <AvatarImage src="/favicon.svg" alt="StageZero" />
                  <AvatarFallback>SZ</AvatarFallback>
                </Avatar>
                <Item size="xs" className="p-0">
                  <ItemContent className="gap-0">
                    <ItemTitle>StageZero</ItemTitle>
                    <ItemDescription className="truncate text-xs">
                      {settings?.sender ?? 'Loading sender'}
                    </ItemDescription>
                  </ItemContent>
                </Item>
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Outreach</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {LINKS.map(({ to, label, icon: Icon }) => (
                <SidebarMenuItem key={to}>
                  <SidebarMenuButton asChild isActive={pathname.startsWith(to)} tooltip={label}>
                    <Link to={to}>
                      <Icon />
                      {label}
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter className="group-data-[collapsible=icon]:hidden">
        <Item variant="outline" size="sm">
          <ItemMedia variant="icon">
            <StatusIndicators />
          </ItemMedia>
        </Item>
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}
