import {
  Field,
  FieldDescription,
  FieldError,
  FieldGroup,
  FieldLabel,
  FieldLegend,
  FieldSet,
} from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { toFieldErrors } from '@/libs/form'
import type { TEmailFormApi } from './use-email-form'

export function EmailFields({ form }: { form: TEmailFormApi }) {
  return (
    <FieldSet>
      <FieldLegend variant="label">Email</FieldLegend>
      <FieldGroup className="gap-4">
        <form.Field name="to">
          {(field) => (
            <Field data-invalid={field.state.meta.errors.length > 0}>
              <FieldLabel htmlFor="mail-to">To</FieldLabel>
              <Input
                id="mail-to"
                type="email"
                placeholder="founder@company.com"
                className="font-mono"
                value={field.state.value}
                onChange={(event) => field.handleChange(event.target.value)}
                aria-invalid={field.state.meta.errors.length > 0}
              />
              <FieldError errors={toFieldErrors(field.state.meta.errors)} />
            </Field>
          )}
        </form.Field>
        <form.Field name="subject">
          {(field) => (
            <Field data-invalid={field.state.meta.errors.length > 0}>
              <FieldLabel htmlFor="mail-subject">Subject</FieldLabel>
              <Input
                id="mail-subject"
                value={field.state.value}
                onChange={(event) => field.handleChange(event.target.value)}
                aria-invalid={field.state.meta.errors.length > 0}
              />
              <FieldError errors={toFieldErrors(field.state.meta.errors)} />
            </Field>
          )}
        </form.Field>
        <form.Field name="body">
          {(field) => (
            <Field data-invalid={field.state.meta.errors.length > 0}>
              <FieldLabel htmlFor="mail-body">Body</FieldLabel>
              <Textarea
                id="mail-body"
                rows={16}
                className="font-mono text-xs leading-relaxed"
                value={field.state.value}
                onChange={(event) => field.handleChange(event.target.value)}
                aria-invalid={field.state.meta.errors.length > 0}
              />
              <FieldError errors={toFieldErrors(field.state.meta.errors)} />
              <FieldDescription>
                Your resume is attached automatically. Replies go to the applicant email in your
                config.
              </FieldDescription>
            </Field>
          )}
        </form.Field>
      </FieldGroup>
    </FieldSet>
  )
}
