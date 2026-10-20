import { useForm } from '@tanstack/react-form'
import { emailSchema, type TEmailDraft } from './email-schema'

export function useEmailForm(defaultValues: TEmailDraft, onSubmit: (value: TEmailDraft) => void) {
  return useForm({
    defaultValues,
    validators: { onSubmit: emailSchema },
    onSubmit: ({ value }) => onSubmit(value),
  })
}

export type TEmailFormApi = ReturnType<typeof useEmailForm>
