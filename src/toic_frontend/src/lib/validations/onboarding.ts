import { categoryNames } from '@/types/core'
import { z } from 'zod'

const onboardingSchema = z.object({
  name: z.string().min(3, 'Name is required'),
  bio: z.string().optional(),
  categories: z.array(z.enum(categoryNames)).min(3, 'Select at least 3 categories').max(3, 'Select only 3 categories')
})

type OnboardingValues = z.infer<typeof onboardingSchema>

export { onboardingSchema, type OnboardingValues }
