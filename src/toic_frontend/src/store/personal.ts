import { create } from 'zustand'
import { toast } from 'sonner'
import { Draft, OnboardingArgs, User } from '@declarations/toic_backend/toic_backend.did'
import { mapToCategory, optionOf, unwrapResult } from '@/lib/mapper'
import { beService } from './auth'

type PersonalState = {
  drafts: Draft[]
  fetchingDrafts: boolean
}

type PersonalAction = {
  getDrafts: () => void
  reset: () => void
}

const initialState: PersonalState = {
  drafts: [],
  fetchingDrafts: false
}

export const usePersonalStore = create<PersonalState & PersonalAction>()((set, get) => ({
  ...initialState,
  getDrafts: async () => {
    set({ fetchingDrafts: true })
    let result = await beService().get_drafts()
    let [drafts, err] = unwrapResult(result)
    if (!!err) {
      console.error(err.message)
    }
    set({ drafts: drafts ?? [], fetchingDrafts: false })
  },
  reset: () => {
    set({ ...initialState })
  }
}))
