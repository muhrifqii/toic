import { z } from 'zod'

export const stakingFormSchema = z
  .object({
    amount: z.coerce.number().min(0.00000001, 'Invalid amount').optional(),
    balance: z.number(),
    fee: z.number()
  })
  .refine(
    val => {
      const tot = (val.amount ?? 0) + val.fee
      return val.balance >= tot
    },
    { message: 'Insufficient balance' }
  )

export type StakingFormValues = z.infer<typeof stakingFormSchema>
