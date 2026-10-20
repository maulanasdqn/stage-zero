import type { TBatch } from '@/apis/startups'
import { Badge } from '@/components/ui/badge'
import { Field, FieldLabel } from '@/components/ui/field'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

type TBatchSelectProps = {
  id: string
  batches: TBatch[]
  value: string
  onChange: (value: string) => void
}

export function BatchSelect({ id, batches, value, onChange }: TBatchSelectProps) {
  return (
    <Field>
      <FieldLabel htmlFor={id}>Batch</FieldLabel>
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger id={id} className="w-full">
          <SelectValue placeholder="Pick a batch" />
        </SelectTrigger>
        <SelectContent>
          {batches.map((batch) => (
            <SelectItem key={batch.slug} value={batch.slug}>
              {batch.name}
              <Badge variant="secondary">{batch.count}</Badge>
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </Field>
  )
}
