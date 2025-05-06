import { categoryNames } from '@/types/core'
import { z } from 'zod'

export const publishSchema = z
  .object({
    description: z.string().min(5, 'Description is required'),
    category: z.enum(categoryNames).optional()
  })
  .refine(val => {
    return !!val.category
  })

export type PublishSingValues = z.infer<typeof publishSchema>
