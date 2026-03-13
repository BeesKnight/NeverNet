import { screen, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { CalendarPage } from './CalendarPage'
import { renderWithProviders } from '../test/test-utils'

const mocks = vi.hoisted(() => ({
  auth: {
    session: {
      user: {
        id: 'user-1',
        email: 'demo@eventdesign.local',
        full_name: 'Demo User',
        created_at: '2026-03-13T10:00:00Z',
      },
    },
    isInitializing: false,
    login: vi.fn(),
    register: vi.fn(),
    logout: vi.fn(),
    updateUser: vi.fn(),
  },
  apiRequest: vi.fn(),
}))

vi.mock('../features/auth/auth-context', () => ({
  useAuth: () => mocks.auth,
}))

vi.mock('../api/client', () => ({
  apiRequest: mocks.apiRequest,
  buildQueryString: (params: Record<string, string | undefined>) => {
    const searchParams = new URLSearchParams()
    Object.entries(params).forEach(([key, value]) => {
      if (value) {
        searchParams.set(key, value)
      }
    })
    const query = searchParams.toString()
    return query ? `?${query}` : ''
  },
}))

describe('CalendarPage', () => {
  beforeEach(() => {
    mocks.apiRequest.mockReset()
    mocks.apiRequest.mockResolvedValue([
      {
        event_id: 'event-1',
        title: 'Defense rehearsal',
        date: '2026-03-15',
        starts_at: '2026-03-15T10:00:00Z',
        ends_at: '2026-03-15T12:00:00Z',
        status: 'planned',
        category_color: '#0f766e',
      },
    ])
  })

  it('renders the month grid and projected events', async () => {
    renderWithProviders(<CalendarPage />)

    await waitFor(() => {
      expect(screen.getByText('Defense rehearsal')).toBeInTheDocument()
    })

    expect(screen.getByText(/scheduled between/i)).toBeInTheDocument()
  })
})
