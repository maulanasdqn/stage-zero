import { SearchIcon } from 'lucide-react'
import { useEffect, useState } from 'react'
import { ButtonGroup, ButtonGroupText } from '@/components/ui/button-group'
import { Checkbox } from '@/components/ui/checkbox'
import { Field, FieldGroup, FieldLabel } from '@/components/ui/field'
import { InputGroup, InputGroupAddon, InputGroupInput } from '@/components/ui/input-group'
import { useDebouncedValue } from '@/hooks/use-debounced-value'
import { cn } from '@/lib/utils'

type TStartupToolbarProps = {
  search: string
  onSearchChange: (value: string) => void
  hideContacted: boolean
  onHideContactedChange: (value: boolean) => void
  matched?: number
  batchTotal?: number
  tracked?: number
  className?: string
}

export function StartupToolbar({
  search,
  onSearchChange,
  hideContacted,
  onHideContactedChange,
  matched,
  batchTotal,
  tracked,
  className,
}: TStartupToolbarProps) {
  const [draft, setDraft] = useState(search)
  const debounced = useDebouncedValue(draft)

  useEffect(() => {
    if (debounced.trim() !== search.trim()) {
      onSearchChange(debounced)
    }
  }, [debounced, search, onSearchChange])

  return (
    <FieldGroup
      className={cn('flex-col gap-3 sm:flex-row sm:items-center sm:justify-between', className)}
    >
      <Field orientation="horizontal" className="w-auto flex-1 flex-wrap gap-4">
        <InputGroup className="sm:max-w-xs">
          <InputGroupInput
            aria-label="Search startups"
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
            placeholder="Search name, tag or one-liner"
          />
          <InputGroupAddon>
            <SearchIcon />
          </InputGroupAddon>
        </InputGroup>
        <Field orientation="horizontal" className="w-auto">
          <Checkbox
            id="hide-contacted"
            checked={hideContacted}
            onCheckedChange={(checked) => onHideContactedChange(checked === true)}
          />
          <FieldLabel htmlFor="hide-contacted" className="font-normal">
            Hide contacted
          </FieldLabel>
        </Field>
      </Field>
      <ButtonGroup aria-label="Batch counts" className="shrink-0">
        <ButtonGroupText className="whitespace-nowrap tabular-nums">
          {matched ?? '–'} matching
        </ButtonGroupText>
        <ButtonGroupText className="whitespace-nowrap tabular-nums">
          {tracked ?? '–'} tracked
        </ButtonGroupText>
        <ButtonGroupText className="whitespace-nowrap tabular-nums">
          {batchTotal ?? '–'} in batch
        </ButtonGroupText>
      </ButtonGroup>
    </FieldGroup>
  )
}
