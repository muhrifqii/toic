import EditorLayout from '@/components/layouts/editor'
import MainLayout from '@/components/layouts/main'
import RootLayout from '@/components/layouts/root'
import { lazy } from 'react'
import { createBrowserRouter, Navigate } from 'react-router'

const NotFoundPage = lazy(() => import('@/pages/not-found'))
const LandingPage = lazy(() => import('@/pages/landing'))
const OnboardingPage = lazy(() => import('@/pages/onboarding'))
const NewStoryPage = lazy(() => import('@/pages/new-story'))

export const router = createBrowserRouter([
  {
    Component: RootLayout,
    children: [
      {
        Component: MainLayout,
        children: [
          { index: true, Component: LandingPage },
          { path: 'onboarding', Component: OnboardingPage }
        ]
      },
      {
        Component: EditorLayout,
        children: [{ path: 'new-story', Component: NewStoryPage }]
      }
    ]
  },
  { path: '404', Component: NotFoundPage },
  { path: '*', element: <Navigate to='/404' replace /> }
])
