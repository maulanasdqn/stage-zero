import type { RowData } from '@tanstack/react-table'
import { ArrowDownIcon, ArrowUpDownIcon, ArrowUpIcon } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'
import type { TColumn } from './features'

type TDataTableColumnHeaderProps<TData extends RowData, TValue> = {
  column: TColumn<TData, TValue>
  title: string
  className?: string
}

export function DataTableColumnHeader<TData extends RowData, TValue>({
  column,
  title,
  className,
}: TDataTableColumnHeaderProps<TData, TValue>) {
  if (!column.getCanSort()) {
    return title
  }
  const sorted = column.getIsSorted()
  const Icon = sorted === 'asc' ? ArrowUpIcon : sorted === 'desc' ? ArrowDownIcon : ArrowUpDownIcon
  return (
    <Button
      variant="ghost"
      size="sm"
      className={cn('data-[sorted=true]:text-foreground -ml-2 h-7', className)}
      data-sorted={Boolean(sorted)}
      onClick={column.getToggleSortingHandler()}
    >
      {title}
      <Icon className={cn(!sorted && 'text-muted-foreground/60')} />
    </Button>
  )
}
