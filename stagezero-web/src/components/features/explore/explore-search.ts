import { z } from 'zod'

export const exploreSearchSchema = z.object({
  batch: z.string().optional(),
  min: z.coerce.number().int().min(0).optional(),
  max: z.coerce.number().int().min(0).optional(),
  industry: z.string().optional(),
  q: z.string().optional(),
  hide: z.boolean().optional(),
  sort: z.enum(['name', 'team_size']).optional(),
  order: z.enum(['asc', 'desc']).optional(),
  page: z.coerce.number().int().min(1).optional(),
  per_page: z.coerce.number().int().min(1).max(100).optional(),
})

export type TExploreSearch = z.infer<typeof exploreSearchSchema>
