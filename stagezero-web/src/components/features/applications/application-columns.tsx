import type { TApplication } from '@/apis/applications'
import { DataTableColumnHeader } from '@/components/features/shared/data-table/data-table-column-header'
import type { TColumnDef } from '@/components/features/shared/data-table/features'
import { StatusBadge } from '@/components/features/shared/status-badge'
import { Badge } from '@/components/ui/badge'
import { ItemDescription, ItemTitle } from '@/components/ui/item'
import { displayUrl, safeUrl } from '@/libs/format'
import { ApplicationActions } from './application-actions'

export function applicationColumns(
  onRemove: (application: TApplication) => void,
): TColumnDef<TApplication>[] {
  return [
    {
      id: 'applied_at',
      accessorKey: 'applied_at',
      header: ({ column }) => <DataTableColumnHeader column={column} title="When (UTC)" />,
      cell: ({ row }) => (
        <ItemDescription className="font-mono text-xs tabular-nums">
          {row.original.applied_at}
        </ItemDescription>
      ),
    },
    {
      id: 'status',
      accessorKey: 'status',
      header: ({ column }) => <DataTableColumnHeader column={column} title="Status" />,
      cell: ({ row }) => <StatusBadge status={row.original.status} />,
    },
    {
      id: 'company_name',
      accessorKey: 'company_name',
      header: ({ column }) => <DataTableColumnHeader column={column} title="Company" />,
      cell: ({ row }) => <ItemTitle>{row.original.company_name}</ItemTitle>,
    },
    {
      id: 'website',
      enableSorting: false,
      header: 'Website',
      cell: ({ row }) => (
        <ItemDescription className="max-w-48 truncate">
          {displayUrl(safeUrl(row.original.website)) || '-'}
        </ItemDescription>
      ),
    },
    {
      id: 'contact_email',
      enableSorting: false,
      header: 'Contact',
      cell: ({ row }) =>
        row.original.contact_email ? (
          <Badge variant="outline" className="font-mono">
            {row.original.contact_email}
          </Badge>
        ) : (
          '-'
        ),
    },
    {
      id: 'actions',
      enableSorting: false,
      header: () => <ItemDescription className="sr-only">Actions</ItemDescription>,
      cell: ({ row }) => <ApplicationActions application={row.original} onRemove={onRemove} />,
    },
  ]
}
