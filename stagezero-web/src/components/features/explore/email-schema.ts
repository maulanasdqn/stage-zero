import { z } from 'zod'

export const emailSchema = z.object({
  to: z.email('Enter a valid recipient address'),
  subject: z.string().trim().min(1, 'Subject is required'),
  body: z.string().trim().min(1, 'Body is required'),
})

export type TEmailDraft = z.infer<typeof emailSchema>
