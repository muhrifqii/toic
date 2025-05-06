import { unwrapResult } from '@/lib/mapper'
import { authService } from '@/services/auth'
import { create } from 'zustand'
import { beService } from './auth'

type WalletState = {
  token: string | null
  lockedToken: string | null
  fee: string | null
  stakeLoading: boolean
}

type WalletAction = {
  getBalance: () => void
  getLockedBalance: () => void
  getFee: () => void
  stake: (amount: number) => void
  reset: () => void
}

const initialState: WalletState = {
  token: null,
  lockedToken: null,
  fee: null,
  stakeLoading: false
}

export const useWalletStore = create<WalletState & WalletAction>()((set, get) => ({
  ...initialState,
  getBalance: async () => {
    const principal = (await authService()).getPrincipal()
    if (principal == null) {
      return
    }
    const token = await beService().icrc1_balance_of({ owner: principal, subaccount: [] })
    set({ token: token.toString() })
  },
  getFee: async () => {
    const fee = await beService().icrc1_fee()
    set({ fee: fee.toString() })
  },
  getLockedBalance: async () => {
    const principal = (await authService()).getPrincipal()
    if (principal == null) {
      return
    }
    const token = await beService().staked_balance_of({ owner: principal, subaccount: [] })
    set({ lockedToken: token.toString() })
  },
  stake: async amount => {
    set({ stakeLoading: true })
    const result = await beService().stake({
      from_subaccount: [],
      amount: BigInt(amount)
    })
    const [, err] = unwrapResult(result)
    set({ stakeLoading: false })
    if (!!err) {
      console.error(err)
      // toast.error()
    }
    get().getBalance()
  },
  reset: () => set({ ...initialState })
}))
