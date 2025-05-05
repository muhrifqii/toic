import { Label } from '@radix-ui/react-label'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '../ui/dialog'
import { Input } from '../ui/input'
import { Button } from '../ui/button'
import { useWalletStore } from '@/store/wallet'
import { useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'

type StakeDialogProps = {
  open: boolean
  onClose?: () => void
}

export function StakeDialog({ open, onClose }: StakeDialogProps) {
  // wip use zod+form
  const [amount, setAmount] = useState('')
  const [error, setError] = useState('')
  const balance = useWalletStore(state => state.token)
  const getbalance = useWalletStore(state => state.getBalance)

  const form = useForm()

  useEffect(() => {
    if (Number(amount) > Number(balance ?? '0')) {
      setError('Insufficient amount')
    } else {
      setError('')
    }
  }, [amount])

  useEffect(() => {
    getbalance()
  }, [])

  return (
    <Dialog open={open} onOpenChange={open => !open && onClose?.()}>
      <form>
        <DialogContent className='sm:max-w-[425px]'>
          <DialogHeader>
            <DialogTitle>Stake TOIC Token</DialogTitle>
            <DialogDescription>Your TOIC Token will be locked.</DialogDescription>
          </DialogHeader>
          <div className='grid gap-4 py-4'>
            <div className='grid grid-cols-4 items-center gap-4'>
              <Label htmlFor='amount' className='text-right'>
                Amount
              </Label>
              <Input
                id='amount'
                placeholder='Amount'
                className='col-span-3'
                type='number'
                value={amount}
                onChange={e => setAmount(e.target.value)}
              />
            </div>
            <p className='text-muted-foreground text-xs ml-auto'>Available balance: {balance}</p>
          </div>
          <DialogFooter>
            <Button type='submit' disabled={!!error || !amount || amount === '0'}>
              Submit
            </Button>
          </DialogFooter>
        </DialogContent>
      </form>
    </Dialog>
  )
}
