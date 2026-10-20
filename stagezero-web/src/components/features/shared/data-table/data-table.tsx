import { useTable, type OnChangeFn, type RowData, type SortingState } from '@tanstack/react-table'
import type { ReactNode } from 'react'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { cn } from '@/lib/utils'
import type { TPaginationMeta } from '@/types/pagination'
import { DataTablePagination } from './data-table-pagination'
import { dataTableFeatures, type TColumnDef } from './features'

type TDataTableProps<TData extends RowData> = {
  columns: TColumnDef<TData>[]
  data: TData[]
  pagination: TPaginationMeta | undefined
  sorting: SortingState
  onSortingChange: (sorting: SortingState) => void
  onPageChange: (page: number) => void
  onPageSizeChange: (perPage: number) => void
  getRowId: (row: TData) => string
  loading: boolean
  fetching?: boolean
  empty: ReactNode
  onRowClick?: (row: TData) => void
}

export function DataTable<TData extends RowData>({
  columns,
  data,
  pagination,
  sorting,
  onSortingChange,
  onPageChange,
  onPageSizeChange,
  getRowId,
  loading,
  fetching,
  empty,
  onRowClick,
}: TDataTableProps<TData>) {
  const handleSortingChange: OnChangeFn<SortingState> = (updater) =>
    onSortingChange(typeof updater === 'function' ? updater(sorting) : updater)

  const table = useTable({
    features: dataTableFeatures,
    data,
    columns,
    getRowId,
    manualPagination: true,
    manualSorting: true,
    enableMultiSort: false,
    rowCount: pagination?.total,
    state: {
      sorting,
      pagination: {
        pageIndex: (pagination?.page ?? 1) - 1,
        pageSize: pagination?.per_page ?? 20,
      },
    },
    onSortingChange: handleSortingChange,
  })

  const rows = table.getRowModel().rows
  const columnCount = table.getAllLeafColumns().length

  return (
    <>
      <Table>
        <TableHeader>
          {table.getHeaderGroups().map((group) => (
            <TableRow key={group.id} className="hover:bg-transparent">
              {group.headers.map((header) => (
                <TableHead key={header.id}>
                  {header.isPlaceholder ? null : <table.FlexRender header={header} />}
                </TableHead>
              ))}
            </TableRow>
          ))}
        </TableHeader>
        <TableBody className={cn('transition-opacity', fetching && !loading && 'opacity-60')}>
          {loading ? (
            Array.from({ length: pagination?.per_page ?? 8 }, (_, index) => (
              <TableRow key={index} className="hover:bg-transparent">
                <TableCell colSpan={columnCount}>
                  <Skeleton className="h-8 w-full" />
                </TableCell>
              </TableRow>
            ))
          ) : rows.length ? (
            rows.map((row) => (
              <TableRow
                key={row.id}
                className={cn(onRowClick && 'cursor-pointer')}
                onClick={onRowClick ? () => onRowClick(row.original) : undefined}
              >
                {row.getAllCells().map((cell) => (
                  <TableCell key={cell.id}>
                    <table.FlexRender cell={cell} />
                  </TableCell>
                ))}
              </TableRow>
            ))
          ) : (
            <TableRow className="hover:bg-transparent">
              <TableCell colSpan={columnCount} className="p-0">
                {empty}
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
      {pagination && pagination.total > 0 && (
        <DataTablePagination
          pagination={pagination}
          onPageChange={onPageChange}
          onPageSizeChange={onPageSizeChange}
          disabled={fetching}
        />
      )}
    </>
  )
}
