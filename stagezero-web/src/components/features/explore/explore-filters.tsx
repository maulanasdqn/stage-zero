import { useForm } from '@tanstack/react-form'
import { DownloadCloudIcon } from 'lucide-react'
import type { TBatch } from '@/apis/startups'
import { BatchSelect } from '@/components/features/shared/batch-select'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Field, FieldError, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Spinner } from '@/components/ui/spinner'
import { toFieldErrors } from '@/libs/form'
import { exploreFilterSchema, type TExploreFilter } from './explore-schema'

type TExploreFiltersProps = {
  batches: TBatch[]
  initial: TExploreFilter
  loading: boolean
  onSubmit: (value: TExploreFilter) => void
  className?: string
}

export function ExploreFilters({
  batches,
  initial,
  loading,
  onSubmit,
  className,
}: TExploreFiltersProps) {
  const form = useForm({
    defaultValues: initial,
    validators: { onChange: exploreFilterSchema },
    onSubmit: ({ value }) => onSubmit(value),
  })

  return (
    <Card size="sm" className={className}>
      <CardContent>
        <form
          onSubmit={(event) => {
            event.preventDefault()
            void form.handleSubmit()
          }}
        >
          <FieldGroup className="grid gap-3 sm:grid-cols-2 lg:grid-cols-[1.4fr_0.6fr_0.6fr_1.4fr_auto] lg:items-end">
            <form.Field name="batch">
              {(field) => (
                <BatchSelect
                  id="explore-batch"
                  batches={batches}
                  value={field.state.value}
                  onChange={field.handleChange}
                />
              )}
            </form.Field>
            {(['min', 'max'] as const).map((name) => (
              <form.Field key={name} name={name}>
                {(field) => (
                  <Field data-invalid={field.state.meta.errors.length > 0}>
                    <FieldLabel htmlFor={`explore-${name}`}>
                      {name === 'min' ? 'Team min' : 'Team max'}
                    </FieldLabel>
                    <Input
                      id={`explore-${name}`}
                      type="number"
                      min={0}
                      value={field.state.value}
                      onChange={(event) => field.handleChange(Number(event.target.value))}
                      aria-invalid={field.state.meta.errors.length > 0}
                    />
                    <FieldError errors={toFieldErrors(field.state.meta.errors)} />
                  </Field>
                )}
              </form.Field>
            ))}
            <form.Field name="industry">
              {(field) => (
                <Field>
                  <FieldLabel htmlFor="explore-industry">Industry keywords</FieldLabel>
                  <Input
                    id="explore-industry"
                    placeholder="e.g. B2B SaaS"
                    value={field.state.value}
                    onChange={(event) => field.handleChange(event.target.value)}
                  />
                </Field>
              )}
            </form.Field>
            <form.Subscribe selector={(state) => state.canSubmit}>
              {(canSubmit) => (
                <Button type="submit" disabled={!canSubmit || loading}>
                  {loading ? <Spinner /> : <DownloadCloudIcon />}
                  Scrape batch
                </Button>
              )}
            </form.Subscribe>
          </FieldGroup>
        </form>
      </CardContent>
    </Card>
  )
}
