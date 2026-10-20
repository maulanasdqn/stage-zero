import { ChevronRightIcon } from 'lucide-react'
import type { TDirectoryEntry } from '@/apis/startups'
import { DataTableColumnHeader } from '@/components/features/shared/data-table/data-table-column-header'
import type { TColumnDef } from '@/components/features/shared/data-table/features'
import { ExternalLink } from '@/components/features/shared/external-link'
import { StatusBadge } from '@/components/features/shared/status-badge'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { ButtonGroup } from '@/components/ui/button-group'
import { Item, ItemContent, ItemDescription, ItemTitle } from '@/components/ui/item'
import { displayUrl, safeUrl } from '@/libs/format'

export const startupColumns: TColumnDef<TDirectoryEntry>[] = [
  {
    id: 'name',
    accessorFn: (entry) => entry.startup.name,
    header: ({ column }) => <DataTableColumnHeader column={column} title="Company" />,
    cell: ({ row }) => (
      <Item size="xs" className="min-w-64 p-0">
        <ItemContent>
          <ItemTitle>{row.original.startup.name}</ItemTitle>
          <ItemDescription className="line-clamp-1 max-w-sm">
            {row.original.startup.one_liner}
          </ItemDescription>
        </ItemContent>
      </Item>
    ),
  },
  {
    id: 'team_size',
    accessorFn: (entry) => entry.startup.team_size,
    header: ({ column }) => <DataTableColumnHeader column={column} title="Team" />,
    cell: ({ row }) => (
      <Badge variant="outline" className="font-mono tabular-nums">
        {row.original.startup.team_size ?? '-'}
      </Badge>
    ),
  },
  {
    id: 'industries',
    enableSorting: false,
    header: 'Industries',
    cell: ({ row }) => (
      <ButtonGroup className="max-w-56 flex-wrap gap-1">
        {row.original.startup.industries.slice(0, 2).map((industry) => (
          <Badge key={industry} variant="secondary">
            {industry}
          </Badge>
        ))}
      </ButtonGroup>
    ),
  },
  {
    id: 'website',
    enableSorting: false,
    header: 'Website',
    cell: ({ row }) => {
      const website = safeUrl(row.original.startup.website)
      if (!website) {
        return '-'
      }
      return (
        <ExternalLink href={website} stopPropagation className="max-w-52">
          <ItemDescription className="truncate">{displayUrl(website)}</ItemDescription>
        </ExternalLink>
      )
    },
  },
  {
    id: 'status',
    enableSorting: false,
    header: 'Status',
    cell: ({ row }) => <StatusBadge status={row.original.status} />,
  },
  {
    id: 'actions',
    enableSorting: false,
    header: () => <ItemDescription className="sr-only">Review</ItemDescription>,
    cell: ({ row }) => (
      <Button variant="ghost" size="icon-sm" aria-label={`Review ${row.original.startup.name}`}>
        <ChevronRightIcon />
      </Button>
    ),
  },
]
