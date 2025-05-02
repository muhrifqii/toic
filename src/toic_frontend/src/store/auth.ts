import { create } from 'zustand'
import { authService } from '@/services/auth'
import { toast } from 'sonner'
import { withMiddleware } from './common'

type AuthStore = {
  isAuthenticated: boolean
  principal: string | null
  isHydrating: boolean
  hydrate: () => Promise<void>
  login: () => Promise<void>
  logout: () => Promise<void>
}

export const useAuthStore = create<AuthStore>()(
  withMiddleware((set, get) => ({
    isAuthenticated: false,
    principal: null,
    isHydrating: true,

    hydrate: async () => {
      const auth = await authService()
      const isAuthenticated = await auth.isAuthenticated()
      const principal = isAuthenticated ? auth.getPrincipal()?.toText() : null

      set({ isAuthenticated, principal, isHydrating: false })
    },
    login: async () => {
      const auth = await authService()
      try {
        await auth.login()
        const principal = auth.getPrincipal()?.toText()
        set({ isAuthenticated: true, principal })
      } catch (reason: any) {
        set({ isAuthenticated: false, principal: null })
        toast.error(reason)
      }
    },
    logout: async () => {
      const auth = await authService()
      await auth.logout()
      set({ isAuthenticated: false, principal: null })
    }
  }))
)
