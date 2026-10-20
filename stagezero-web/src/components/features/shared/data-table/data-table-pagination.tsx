import {
  ChevronLeftIcon,
  ChevronRightIcon,
  ChevronsLeftIcon,
  ChevronsRightIcon,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ButtonGroup, ButtonGroupText } from '@/components/ui/button-group'
import { CardFooter } from '@/components/ui/card'
import { Field, FieldDescription, FieldLabel } from '@/components/ui/field'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import type { TPaginationMeta } from '@/types/pagination'

export const PAGE_SIZES = [10, 20, 50, 100] as const

type TDataTablePaginationProps = {
  pagination: TPaginationMeta
  onPageChange: (page: number) => void
  onPageSizeChange: (perPage: number) => void
  disabled?: boolean
}

export function DataTablePagination({
  pagination,
  onPageChange,
  onPageSizeChange,
  disabled,
}: TDataTablePaginationProps) {
  const { page, per_page, total, total_pages } = pagination
  const from = total === 0 ? 0 : (page - 1) * per_page + 1
  const to = Math.min(page * per_page, total)
  const canPrevious = page > 1 && !disabled
  const canNext = page < total_pages && !disabled

  return (
    <CardFooter className="flex-wrap justify-between gap-3 border-t">
      <FieldDescription className="tabular-nums">
        Showing {from}–{to} of {total}
      </FieldDescription>
      <Field orientation="horizontal" className="w-auto gap-6">
        <Field orientation="horizontal" className="w-auto">
          <FieldLabel htmlFor="rows-per-page" className="text-muted-foreground font-normal">
            Rows per page
          </FieldLabel>
          <Select
            value={String(per_page)}
            onValueChange={(value) => onPageSizeChange(Number(value))}
          >
            <SelectTrigger id="rows-per-page" size="sm" className="w-18">
              <SelectValue />
            </SelectTrigger>
            <SelectContent align="end">
              {PAGE_SIZES.map((size) => (
                <SelectItem key={size} value={String(size)}>
                  {size}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        <ButtonGroup aria-label="Pagination">
          <Button
            variant="outline"
            size="icon-sm"
            aria-label="First page"
            disabled={!canPrevious}
            onClick={() => onPageChange(1)}
          >
            <ChevronsLeftIcon />
          </Button>
          <Button
            variant="outline"
            size="icon-sm"
            aria-label="Previous page"
            disabled={!canPrevious}
            onClick={() => onPageChange(page - 1)}
          >
            <ChevronLeftIcon />
          </Button>
          <ButtonGroupText className="h-7 tabular-nums">
            Page {page} of {total_pages}
          </ButtonGroupText>
          <Button
            variant="outline"
            size="icon-sm"
            aria-label="Next page"
            disabled={!canNext}
            onClick={() => onPageChange(page + 1)}
          >
            <ChevronRightIcon />
          </Button>
          <Button
            variant="outline"
            size="icon-sm"
            aria-label="Last page"
            disabled={!canNext}
            onClick={() => onPageChange(total_pages)}
          >
            <ChevronsRightIcon />
          </Button>
        </ButtonGroup>
      </Field>
    </CardFooter>
  )
}
