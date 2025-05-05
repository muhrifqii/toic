import LoadingPage from '@/pages/loading'
import { useAuthStore } from '@/store/auth'
import { NewStoryIdPlaceholder, useDraftingStore } from '@/store/drafting'
import { PropWithChild } from '@/types/ui'
import { useEffect, useState } from 'react'
import { Navigate, useLocation, useNavigate, useParams } from 'react-router'
import RouteAuthGuard from './auth-guard'

function isEditingPath(pathname: string): boolean {
  return pathname.startsWith('/x/') && pathname.endsWith('/edit')
}

export default function RouteEditorGuard({ children }: PropWithChild) {
  const { pathname } = useLocation()
  const params = useParams()
  const navigate = useNavigate()
  const id = params['id']

  const [validating, setValidating] = useState(true)

  const isHydrated = useAuthStore(state => state.isHydrated)

  const fetching = useDraftingStore(state => state.fetching)
  const setActiveDraft = useDraftingStore(state => state.setActiveDraft)
  const getDraft = useDraftingStore(state => state.getDraft)
  const error = useDraftingStore(state => state.error)
  const errorHandled = useDraftingStore(state => state.errorHandled)
  const prevSelected = useDraftingStore(state => state.selectedId)

  useEffect(() => {
    console.log('trigger param id changed', id)
    setValidating(true)
    // Check if the current route is for creating a new story
    if (pathname === '/new-story') {
      setActiveDraft(NewStoryIdPlaceholder)
      setValidating(false)
    } else if (id) {
      getDraft(id).finally(() => {
        setValidating(false)
      })
    } else {
      navigate('/', { replace: true })
      setValidating(false)
    }

    return () => {
      setActiveDraft(null)
      errorHandled()
    }
  }, [id])

  if (!isHydrated || (!prevSelected && (fetching || validating))) {
    return <LoadingPage />
  }

  if (error === '404') {
    return <Navigate to='/404' />
  }

  return <RouteAuthGuard>{children}</RouteAuthGuard>
}
