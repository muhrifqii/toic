import { devtools, persist } from 'zustand/middleware'
import { create } from 'zustand'
import { authService } from '@/services/auth'
import { toast } from 'sonner'
import { User } from '@declarations/toic_backend/toic_backend.did'
import { OnboardingArgsBuilder } from '@/types/core'
import { mapToCategory, optionOf } from '@/lib/mapper'
import { usePersonalStore } from './personal'
import { useWalletStore } from './wallet'
import { toic_backend } from '@declarations/toic_backend'

type AuthState = {
  isAuthenticated: boolean
  principal: string | null
  isHydrating: boolean
  isHydrated: boolean
  user: Pick<User, 'name' | 'onboarded'> | null
  actor: typeof toic_backend
}

type AuthAction = {
  hydrate: () => Promise<void>
  login: () => Promise<void>
  logout: () => Promise<void>
  onboard: (args: OnboardingArgsBuilder) => Promise<boolean>
}

const initialState: AuthState = {
  isAuthenticated: false,
  principal: null,
  isHydrating: false,
  isHydrated: false,
  user: null,
  actor: toic_backend
}

export const useAuthStore = create<AuthState & AuthAction>()((set, get) => ({
  ...initialState,

  hydrate: async () => {
    if (get().isHydrated) {
      return
    }

    const auth = await authService()
    const actor = auth.getActor()
    const isAuthenticated = await auth.isAuthenticated()
    const principal = isAuthenticated ? auth.getPrincipal()?.toText() : null
    let user: Pick<User, 'name' | 'onboarded'> | null = auth.getUser()
    if (isAuthenticated) {
      await auth.backendLogin()
      user = auth.getUser()
    }

    set(state => {
      return { isAuthenticated, principal, isHydrating: false, isHydrated: true, user, actor }
    })
  },
  login: async () => {
    const auth = await authService()
    try {
      await auth.login()
      const actor = auth.getActor()
      const principal = auth.getPrincipal()?.toText()
      const user = auth.getUser()

      set({ isAuthenticated: true, principal, user, actor })
    } catch (reason: any) {
      set({ isAuthenticated: false, principal: null, user: null })
      toast.error(reason)
    }
  },
  logout: async () => {
    const auth = await authService()
    await auth.logout()
    usePersonalStore.getState().reset()
    useWalletStore.getState().reset()
    set({ isAuthenticated: false, principal: null, user: null, actor: auth.getActor() })
  },
  onboard: async ({ name, bio, categories, code }) => {
    const auth = await authService()
    const nameOpt = optionOf(name)
    const withReferral = await auth.onboard({
      name: nameOpt,
      bio: optionOf(bio),
      categories: categories.map(mapToCategory),
      referral_code: optionOf(code)
    })
    set({ user: { name: nameOpt, onboarded: true } })
    return withReferral
  }
}))

// shortcut for not having to use hooks
export const beService = () => useAuthStore.getState().actor
