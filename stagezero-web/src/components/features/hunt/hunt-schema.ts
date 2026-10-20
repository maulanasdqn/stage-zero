import { z } from 'zod'

export const huntSchema = z
  .object({
    batch: z.string().min(1, 'Pick a batch'),
    min: z.number().int().min(0),
    max: z.number().int().min(0),
    industry: z.string(),
    limit: z.number().int().min(1, 'At least 1').max(200, 'At most 200'),
    dryRun: z.boolean(),
  })
  .refine((value) => value.min <= value.max, {
    message: 'Min must not exceed max',
    path: ['max'],
  })

export type THuntForm = z.infer<typeof huntSchema>
