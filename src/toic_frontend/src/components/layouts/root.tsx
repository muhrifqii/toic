import { Suspense, useEffect } from 'react'
import { Outlet } from 'react-router'
import { Toaster } from '@/components/ui/sonner'
import LoadingPage from '@/pages/loading'
import { useAuthStore } from '@/store/auth'
import RouteGuard from '@/routes/guard'
import { Navbar } from '../blocks/navbar'

export default function RootLayout() {
  const hydrate = useAuthStore(state => state.hydrate)

  useEffect(() => {
    hydrate()
  }, [])

  return (
    <Suspense fallback={<LoadingPage />}>
      <RouteGuard>
        <Toaster duration={3500} position='top-center' richColors />
        <Outlet />
      </RouteGuard>
    </Suspense>
  )
}
