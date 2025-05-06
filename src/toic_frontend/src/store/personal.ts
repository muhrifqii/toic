import { create } from 'zustand'
import { toast } from 'sonner'
import { Draft, OnboardingArgs, Story, User } from '@declarations/toic_backend/toic_backend.did'
import { mapToCategory, optionOf, unwrapResult } from '@/lib/mapper'
import { beService } from './auth'
import { authService } from '@/services/auth'

type PersonalState = {
  drafts: Draft[]
  published: Story[]
  fetchingDrafts: boolean
  fetchingPublished: boolean
}

type PersonalAction = {
  getDrafts: () => void
  getPublished: () => void
  reset: () => void
}

const initialState: PersonalState = {
  drafts: [],
  published: [],
  fetchingDrafts: false,
  fetchingPublished: false
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
  getPublished: async () => {
    set({ fetchingPublished: true })
    const me = (await authService()).getPrincipal()
    const result = await beService().get_stories_by_author({
      cursor: [],
      author: [me!],
      limit: [BigInt(10000)],
      category: []
    })
    const [tuple, err] = unwrapResult(result)
    if (!!err) {
      console.error(err.message)
    }
    const [, vec] = tuple ?? [, [] as Story[]]
    set({ published: vec, fetchingPublished: false })
  },
  reset: () => {
    set({ ...initialState })
  }
}))
