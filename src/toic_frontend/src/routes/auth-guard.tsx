import LoadingPage from '@/pages/loading'
import { useAuthStore } from '@/store/auth'
import { PropWithChild } from '@/types/ui'
import { Navigate, useLocation } from 'react-router'

function hasPermission(path: string, authed: boolean, onboarded: boolean) {
  if (authed && !onboarded && path !== '/onboarding') {
    return false
  }
  if (path === '/new-story' || path.startsWith('/me/') || path === '/onboarding') {
    return authed
  }
  if (path === '/' || path.startsWith('/p/')) return true
  return authed
}

export default function RouteAuthGuard({ children }: PropWithChild) {
  const { pathname } = useLocation()
  const authed = useAuthStore(state => state.isAuthenticated)
  const isHydrated = useAuthStore(state => state.isHydrated)
  const user = useAuthStore(state => state.user)
  const onboarded = user?.onboarded ?? false
  const redirect = authed ? '/onboarding' : '/'

  if (!isHydrated) {
    return <LoadingPage />
  }

  if (authed && onboarded && pathname === '/onboarding') {
    return <Navigate to='/' />
  }

  if (hasPermission(pathname, authed, onboarded)) {
    return children
  }

  return <Navigate to={redirect} />
}
