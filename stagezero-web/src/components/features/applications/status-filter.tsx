import type { TApplicationStatus } from '@/apis/startups'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

const ALL = 'ALL'

const OPTIONS: { value: TApplicationStatus | typeof ALL; label: string }[] = [
  { value: ALL, label: 'All statuses' },
  { value: 'APPLIED', label: 'Applied' },
  { value: 'SKIPPED', label: 'Skipped' },
  { value: 'FAILED_NO_CONTACT', label: 'No contact' },
]

type TStatusFilterProps = {
  value?: TApplicationStatus
  onChange: (value?: TApplicationStatus) => void
}

export function StatusFilter({ value, onChange }: TStatusFilterProps) {
  return (
    <Select
      value={value ?? ALL}
      onValueChange={(next) => onChange(next === ALL ? undefined : (next as TApplicationStatus))}
    >
      <SelectTrigger className="w-40 shrink-0" aria-label="Filter by status">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {OPTIONS.map((option) => (
          <SelectItem key={option.value} value={option.value}>
            {option.label}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}
