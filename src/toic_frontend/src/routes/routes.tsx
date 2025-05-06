import RootLayout from '@/components/layouts/root'
import { lazy } from 'react'
import { createBrowserRouter, Navigate } from 'react-router'

const NotFoundPage = lazy(() => import('@/pages/not-found'))
const LandingPage = lazy(() => import('@/pages/landing'))
const OnboardingPage = lazy(() => import('@/pages/onboarding'))
const StoryEditorPage = lazy(() => import('@/pages/story-editor'))
const MainLayout = lazy(() => import('@/components/layouts/main'))
const MePage = lazy(() => import('@/pages/me'))
const StoryPage = lazy(() => import('@/pages/story'))

export const router = createBrowserRouter([
  {
    Component: RootLayout,
    children: [
      {
        Component: MainLayout,
        children: [
          { index: true, Component: LandingPage },
          { path: 'onboarding', Component: OnboardingPage },
          { path: 'me', Component: MePage },
          { path: 'p/:id', Component: StoryPage }
        ]
      },
      {
        Component: StoryEditorPage,
        children: [
          { path: 'new-story', element: null },
          { path: 'x/:id/edit', element: null }
        ]
      }
    ]
  },
  { path: '404', Component: NotFoundPage },
  { path: '*', element: <Navigate to='/404' replace /> }
])
