import { Suspense, useEffect } from 'react'
import { Outlet } from 'react-router'
import { Toaster } from '@/components/ui/sonner'
import LoadingPage from '@/pages/loading'
import { useAuthStore } from '@/store/auth'
import RouteAuthGuard from '@/routes/auth-guard'

export default function RootLayout() {
  const hydrate = useAuthStore(state => state.hydrate)

  useEffect(() => {
    hydrate()
  }, [])

  return (
    <Suspense fallback={<LoadingPage />}>
      <RouteAuthGuard>
        <Toaster duration={3500} position='top-center' richColors />
        <Outlet />
      </RouteAuthGuard>
    </Suspense>
  )
}
