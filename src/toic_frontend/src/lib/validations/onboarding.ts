import { categoryNames } from '@/types/core'
import { z } from 'zod'

const onboardingSchema = z.object({
  name: z
    .string()
    .min(3, 'Username is required')
    .regex(/^[a-zA-Z0-9]*$/, 'Username should only use alphanumeric'),
  bio: z.string().optional(),
  categories: z.array(z.enum(categoryNames)).min(3, 'Select 3 categories').max(3, 'Select only 3 categories'),
  code: z.string().optional()
})

type OnboardingValues = z.infer<typeof onboardingSchema>

export { onboardingSchema, type OnboardingValues }
