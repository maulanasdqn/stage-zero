import { z } from 'zod'

export const applicationsSearchSchema = z.object({
  status: z.enum(['APPLIED', 'SKIPPED', 'FAILED_NO_CONTACT']).optional(),
  q: z.string().optional(),
  sort: z.enum(['applied_at', 'company_name', 'status']).optional(),
  order: z.enum(['asc', 'desc']).optional(),
  page: z.coerce.number().int().min(1).optional(),
  per_page: z.coerce.number().int().min(1).max(100).optional(),
})

export type TApplicationsSearch = z.infer<typeof applicationsSearchSchema>
