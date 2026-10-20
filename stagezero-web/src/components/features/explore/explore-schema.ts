import { z } from 'zod'

export const exploreFilterSchema = z
  .object({
    batch: z.string().min(1, 'Pick a batch'),
    min: z.number().int().min(0),
    max: z.number().int().min(0),
    industry: z.string(),
  })
  .refine((value) => value.min <= value.max, {
    message: 'Min must not exceed max',
    path: ['max'],
  })

export type TExploreFilter = z.infer<typeof exploreFilterSchema>
