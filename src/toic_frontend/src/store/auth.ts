import { devtools, persist } from 'zustand/middleware'
import { create } from 'zustand'
import { authService } from '@/services/auth'
import { toast } from 'sonner'
import { User } from '@declarations/toic_backend/toic_backend.did'

type AuthStore = {
  isAuthenticated: boolean
  principal: string | null
  isHydrating: boolean
  user: Pick<User, 'name' | 'onboarded'> | null
  hydrate: () => Promise<void>
  login: () => Promise<void>
  logout: () => Promise<void>
}

export const useAuthStore = create<AuthStore>()(
  devtools(
    persist(
      set => ({
        isAuthenticated: false,
        principal: null,
        isHydrating: true,
        user: null,

        hydrate: async () => {
          const auth = await authService()
          const isAuthenticated = await auth.isAuthenticated()
          const principal = isAuthenticated ? auth.getPrincipal()?.toText() : null

          set(({ user }) => ({ isAuthenticated, principal, isHydrating: false, user: isAuthenticated ? user : null }))
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
        }
      }),
      {
        name: 'auth-store',
        partialize(state) {
          // persisted auth is already handled by AuthService+Client
          return { user: state.user }
        }
      }
    )
  )
)
