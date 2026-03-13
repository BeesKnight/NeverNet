import { screen } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { AppRoutes } from './router'
import { renderWithProviders } from '../test/test-utils'

const mocks = vi.hoisted(() => ({
  auth: {
    session: null as
      | null
      | {
          user: {
            id: string
            email: string
            full_name: string
            created_at: string
          }
        },
    isInitializing: false,
    login: vi.fn(),
    register: vi.fn(),
    logout: vi.fn(),
    updateUser: vi.fn(),
  },
}))

vi.mock('../features/auth/auth-context', () => ({
  useAuth: () => mocks.auth,
}))

vi.mock('../pages/DashboardPage', () => ({
  DashboardPage: () => <div>Dashboard Page</div>,
}))

vi.mock('../pages/CategoriesPage', () => ({
  CategoriesPage: () => <div>Categories Page</div>,
}))

vi.mock('../pages/EventsPage', () => ({
  EventsPage: () => <div>Events Page</div>,
}))

vi.mock('../pages/CalendarPage', () => ({
  CalendarPage: () => <div>Calendar Page</div>,
}))

vi.mock('../pages/ReportsPage', () => ({
  ReportsPage: () => <div>Reports Page</div>,
}))

vi.mock('../pages/SettingsPage', () => ({
  SettingsPage: () => <div>Settings Page</div>,
}))

describe('AppRoutes', () => {
  beforeEach(() => {
    mocks.auth.isInitializing = false
  })

  it('redirects unauthenticated users to login', () => {
    mocks.auth.session = null

    renderWithProviders(<AppRoutes defaultAuthenticatedPath="/dashboard" />, '/events')

    expect(screen.getByRole('heading', { name: 'Sign in' })).toBeInTheDocument()
  })

  it('renders protected routes for authenticated users', () => {
    mocks.auth.session = {
      user: {
        id: 'user-1',
        email: 'demo@eventdesign.local',
        full_name: 'Demo User',
        created_at: '2026-03-13T10:00:00Z',
      },
    }

    renderWithProviders(<AppRoutes defaultAuthenticatedPath="/dashboard" />, '/events')

    expect(screen.getByText('Events Page')).toBeInTheDocument()
  })
})
