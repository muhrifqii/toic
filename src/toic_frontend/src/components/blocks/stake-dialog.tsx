import { Label } from '@radix-ui/react-label'
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from '../ui/dialog'
import { Input } from '../ui/input'
import { Button, LoadingButton } from '../ui/button'
import { useWalletStore } from '@/store/wallet'
import { useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { stakingFormSchema, StakingFormValues } from '@/lib/validations/staking'
import { zodResolver } from '@hookform/resolvers/zod'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '../ui/form'

type StakeDialogProps = {
  open: boolean
  onClose?: () => void
}

export function StakeDialog({ open, onClose }: StakeDialogProps) {
  const balance = useWalletStore(state => state.token)
  const fee = useWalletStore(state => state.fee)
  const getbalance = useWalletStore(state => state.getBalance)
  const getFee = useWalletStore(state => state.getFee)
  const stake = useWalletStore(state => state.stake)
  const isStaking = useWalletStore(state => state.stakeLoading)

  const form = useForm<StakingFormValues>({
    resolver: zodResolver(stakingFormSchema),
    defaultValues: { amount: 0, balance: 0, fee: 0 },
    mode: 'onChange'
  })

  useEffect(() => {
    getbalance()
    getFee()
  }, [])

  useEffect(() => {
    if (balance) {
      form.setValue('balance', Number(balance ?? '0'), { shouldValidate: true, shouldTouch: true })
    }
    if (fee) {
      form.setValue('fee', Number(fee ?? '0'), { shouldValidate: true, shouldTouch: true })
    }
  }, [balance, fee])

  const onSubmit = async (values: StakingFormValues) => {
    console.log('onsubmit')
    stake(values.amount!)
  }

  return (
    <Dialog open={open} onOpenChange={open => !open && onClose?.()}>
      <DialogContent className='sm:max-w-[425px]'>
        <DialogHeader>
          <DialogTitle>Stake TOIC Token</DialogTitle>
          <DialogDescription>Your TOIC Token will be locked.</DialogDescription>
        </DialogHeader>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)}>
            <div className='grid gap-4 py-4'>
              <FormField
                control={form.control}
                name='amount'
                render={({ field }) => (
                  <FormItem className='grid grid-cols-4 items-center gap-4'>
                    <FormLabel htmlFor='amount' className='text-right'>
                      Amount
                    </FormLabel>
                    <FormControl>
                      <Input type='number' placeholder='Amount' {...field} className='col-span-3' />
                    </FormControl>
                    <FormMessage className='col-span-3' />
                  </FormItem>
                )}
              />
              <FormField
                control={form.control}
                name='balance'
                render={({ field }) => <input type='number' {...field} hidden />}
              />
              <FormField
                control={form.control}
                name='fee'
                render={({ field }) => <input type='number' {...field} hidden />}
              />
              <div className='flex w-full text-muted-foreground text-xs justify-between'>
                <p className=''>
                  Transfer fee: <span>{fee}</span>
                </p>
                <p className='font-medium'>
                  Available balance: <span>{balance}</span>
                </p>
              </div>
            </div>
            <LoadingButton
              type='submit'
              disabled={!form.formState.isValid}
              isLoading={isStaking}
              className='w-full mt-8'
            >
              Submit
            </LoadingButton>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  )
}
