import MainLayout from '@/components/layouts/main'
import { lazy } from 'react'
import { createBrowserRouter, Navigate } from 'react-router'

const NotFoundPage = lazy(() => import('@/pages/not-found'))
const LandingPage = lazy(() => import('@/pages/landing'))
const OnboardingPage = lazy(() => import('@/pages/onboarding'))

export const router = createBrowserRouter([
  {
    Component: MainLayout,
    children: [
      { index: true, Component: LandingPage },
      { path: 'onboarding', Component: OnboardingPage }
    ]
  },
  { path: '404', Component: NotFoundPage },
  { path: '*', element: <Navigate to='/404' replace /> }
])
