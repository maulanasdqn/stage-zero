import { useForm } from '@tanstack/react-form'
import { PlayIcon } from 'lucide-react'
import { useState } from 'react'
import { toast } from 'sonner'
import { useStartHunt } from '@/apis/hunt'
import type { TBatch } from '@/apis/startups'
import { ConfirmDialog } from '@/components/features/shared/confirm-dialog'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { Field, FieldError, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Spinner } from '@/components/ui/spinner'
import { toFieldErrors } from '@/libs/form'
import { errorMessage } from '@/libs/axios'
import { BatchSelect } from '@/components/features/shared/batch-select'
import { huntSchema, type THuntForm } from './hunt-schema'

type THuntFormProps = {
  batches: TBatch[]
  defaultBatch: string
  running: boolean
  canSend: boolean
  className?: string
}

export function HuntForm({ batches, defaultBatch, running, canSend, className }: THuntFormProps) {
  const start = useStartHunt()
  const [pending, setPending] = useState<THuntForm | null>(null)

  const launch = (value: THuntForm) =>
    start.mutate(
      {
        batch: value.batch,
        size: `${value.min}-${value.max}`,
        industry: value.industry || undefined,
        limit: value.limit,
        dry_run: value.dryRun,
      },
      {
        onSuccess: () => toast.success(value.dryRun ? 'Dry-run hunt started' : 'Live hunt started'),
        onError: (error) =>
          toast.error('Could not start hunt', { description: errorMessage(error) }),
      },
    )

  const form = useForm({
    defaultValues: { batch: defaultBatch, min: 1, max: 10, industry: '', limit: 10, dryRun: true },
    validators: { onChange: huntSchema },
    onSubmit: ({ value }) => (value.dryRun ? launch(value) : setPending(value)),
  })

  const numberField = (name: 'min' | 'max' | 'limit', label: string) => (
    <form.Field name={name}>
      {(field) => (
        <Field data-invalid={field.state.meta.errors.length > 0}>
          <FieldLabel htmlFor={`hunt-${name}`}>{label}</FieldLabel>
          <Input
            id={`hunt-${name}`}
            type="number"
            min={name === 'limit' ? 1 : 0}
            value={field.state.value}
            onChange={(event) => field.handleChange(Number(event.target.value))}
            aria-invalid={field.state.meta.errors.length > 0}
          />
          <FieldError errors={toFieldErrors(field.state.meta.errors)} />
        </Field>
      )}
    </form.Field>
  )

  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle>New hunt</CardTitle>
        <CardDescription>
          Processes new startups in a batch one by one, with a short random delay between each.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form
          onSubmit={(event) => {
            event.preventDefault()
            void form.handleSubmit()
          }}
        >
          <FieldGroup className="grid gap-4 sm:grid-cols-2 lg:grid-cols-[1.4fr_0.6fr_0.6fr_1.2fr_0.6fr]">
            <form.Field name="batch">
              {(field) => (
                <BatchSelect
                  id="hunt-batch"
                  batches={batches}
                  value={field.state.value}
                  onChange={field.handleChange}
                />
              )}
            </form.Field>
            {numberField('min', 'Team min')}
            {numberField('max', 'Team max')}
            <form.Field name="industry">
              {(field) => (
                <Field>
                  <FieldLabel htmlFor="hunt-industry">Industry keywords</FieldLabel>
                  <Input
                    id="hunt-industry"
                    placeholder="e.g. Fintech"
                    value={field.state.value}
                    onChange={(event) => field.handleChange(event.target.value)}
                  />
                </Field>
              )}
            </form.Field>
            {numberField('limit', 'Limit')}
            <Field orientation="responsive" className="justify-between gap-3 sm:col-span-full">
              <form.Field name="dryRun">
                {(field) => (
                  <Field orientation="horizontal" className="w-auto">
                    <Checkbox
                      id="hunt-dry-run"
                      checked={field.state.value}
                      onCheckedChange={(checked) => field.handleChange(checked === true)}
                    />
                    <FieldLabel htmlFor="hunt-dry-run" className="font-normal">
                      Dry run, preview emails without sending or recording
                    </FieldLabel>
                  </Field>
                )}
              </form.Field>
              <form.Subscribe selector={(state) => [state.canSubmit, state.values.dryRun] as const}>
                {([canSubmit, dryRun]) => (
                  <Button
                    type="submit"
                    variant={dryRun ? 'outline' : 'default'}
                    disabled={!canSubmit || running || start.isPending || (!dryRun && !canSend)}
                  >
                    {start.isPending ? <Spinner /> : <PlayIcon />}
                    {dryRun ? 'Start dry run' : 'Start live hunt'}
                  </Button>
                )}
              </form.Subscribe>
            </Field>
          </FieldGroup>
        </form>
      </CardContent>
      <ConfirmDialog
        open={pending !== null}
        onOpenChange={(open) => !open && setPending(null)}
        title="Send real emails?"
        description={`StageZero will email founders of up to ${pending?.limit ?? 0} new startups in this batch, with your resume attached.`}
        confirmLabel="Start live hunt"
        onConfirm={() => {
          if (pending) {
            launch(pending)
          }
          setPending(null)
        }}
      />
    </Card>
  )
}
