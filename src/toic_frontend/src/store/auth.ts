import { devtools, persist } from 'zustand/middleware'
import { create } from 'zustand'
import { authService } from '@/services/auth'
import { toast } from 'sonner'
import { OnboardingArgs, User } from '@declarations/toic_backend/toic_backend.did'
import { OnboardingArgsBuilder } from '@/types/core'
import { mapToCategory, optionOf } from '@/lib/mapper'

type AuthState = {
  isAuthenticated: boolean
  principal: string | null
  isHydrating: boolean
  isHydrated: boolean
  user: Pick<User, 'name' | 'onboarded'> | null
}

type AuthAction = {
  hydrate: () => Promise<void>
  login: () => Promise<void>
  logout: () => Promise<void>
  onboard: (args: OnboardingArgsBuilder) => Promise<void>
}

const initialState: AuthState = {
  isAuthenticated: false,
  principal: null,
  isHydrating: false,
  isHydrated: false,
  user: null
}

export const useAuthStore = create<AuthState & AuthAction>()((set, get) => ({
  ...initialState,

  hydrate: async () => {
    if (get().isHydrated) {
      return
    }

    const auth = await authService()
    const isAuthenticated = await auth.isAuthenticated()
    const principal = isAuthenticated ? auth.getPrincipal()?.toText() : null
    let user: Pick<User, 'name' | 'onboarded'> | null = auth.getUser()
    if (isAuthenticated) {
      await auth.backendLogin()
      user = auth.getUser()
    }

    set(state => {
      // console.log('setting user', user)
      return { isAuthenticated, principal, isHydrating: false, isHydrated: true, user }
    })
  },
  login: async () => {
    const auth = await authService()
    try {
      await auth.login()
      const principal = auth.getPrincipal()?.toText()
      const user = auth.getUser()

      set({ isAuthenticated: true, principal, user })
    } catch (reason: any) {
      set({ isAuthenticated: false, principal: null, user: null })
      toast.error(reason)
    }
  },
  logout: async () => {
    const auth = await authService()
    await auth.logout()
    set({ isAuthenticated: false, principal: null, user: null })
  },
  onboard: async ({ name, bio, categories }) => {
    const auth = await authService()
    const nameOpt = optionOf(name)
    await auth.onboard({
      name: nameOpt,
      bio: optionOf(bio),
      categories: categories.map(mapToCategory)
    })
    set({ user: { name: nameOpt, onboarded: true } })
  }
}))
