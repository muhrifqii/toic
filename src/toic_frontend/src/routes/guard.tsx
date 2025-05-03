import { useAuthStore } from '@/store/auth'
import { PropWithChild } from '@/types/ui'
import { Navigate, useLocation } from 'react-router'

function hasPermission(path: string, authed: boolean, onboarded: boolean) {
  if (path === '/new-story' || path.startsWith('/me/')) {
    return authed
  }
  if (authed && !onboarded && path !== '/onboarding') {
    return false
  }
  return true
}

export default function RouteGuard({ children }: PropWithChild) {
  const { pathname } = useLocation()
  const authed = useAuthStore(state => state.isAuthenticated)
  const user = useAuthStore(state => state.user)
  const onboarded = user?.onboarded ?? false
  const redirect = authed ? '/onboarding' : '/'

  if (hasPermission(pathname, authed, onboarded)) {
    return children
  }

  return <Navigate to={redirect} />
}
