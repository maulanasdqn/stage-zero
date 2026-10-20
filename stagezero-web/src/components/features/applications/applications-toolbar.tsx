import { SearchIcon, Trash2Icon } from 'lucide-react'
import { useEffect, useState } from 'react'
import type { TApplicationStatus } from '@/apis/startups'
import { Button } from '@/components/ui/button'
import { CardHeader } from '@/components/ui/card'
import { Field } from '@/components/ui/field'
import { InputGroup, InputGroupAddon, InputGroupInput } from '@/components/ui/input-group'
import { useDebouncedValue } from '@/hooks/use-debounced-value'
import { StatusFilter } from './status-filter'

type TApplicationsToolbarProps = {
  search: string
  onSearchChange: (value: string) => void
  status?: TApplicationStatus
  onStatusChange: (status?: TApplicationStatus) => void
  canReset: boolean
  onReset: () => void
}

export function ApplicationsToolbar({
  search,
  onSearchChange,
  status,
  onStatusChange,
  canReset,
  onReset,
}: TApplicationsToolbarProps) {
  const [draft, setDraft] = useState(search)
  const debounced = useDebouncedValue(draft)

  useEffect(() => {
    if (debounced.trim() !== search.trim()) {
      onSearchChange(debounced)
    }
  }, [debounced, search, onSearchChange])

  return (
    <CardHeader className="border-b py-4">
      <Field orientation="horizontal" className="flex-wrap gap-3">
        <InputGroup className="w-full sm:w-72">
          <InputGroupInput
            aria-label="Search applications"
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
            placeholder="Search company, slug or email"
          />
          <InputGroupAddon>
            <SearchIcon />
          </InputGroupAddon>
        </InputGroup>
        <StatusFilter value={status} onChange={onStatusChange} />
        <Button variant="destructive" className="sm:ml-auto" disabled={!canReset} onClick={onReset}>
          <Trash2Icon />
          Reset all
        </Button>
      </Field>
    </CardHeader>
  )
}
