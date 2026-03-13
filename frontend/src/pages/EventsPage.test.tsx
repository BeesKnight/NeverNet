import { screen, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { EventsPage } from './EventsPage'
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

describe('EventsPage', () => {
  beforeEach(() => {
    mocks.apiRequest.mockReset()
    mocks.apiRequest.mockImplementation(async (path: string) => {
      if (path === '/categories') {
        return [
          {
            id: 'category-1',
            user_id: 'user-1',
            name: 'Conference',
            color: '#0f766e',
            created_at: '2026-03-13T10:00:00Z',
            updated_at: '2026-03-13T10:00:00Z',
          },
        ]
      }

      return [
        {
          id: 'event-1',
          user_id: 'user-1',
          category_id: 'category-1',
          category_name: 'Conference',
          category_color: '#0f766e',
          title: 'Defense rehearsal',
          description: 'Dry run',
          location: 'Room 301',
          starts_at: '2026-03-15T10:00:00Z',
          ends_at: '2026-03-15T12:00:00Z',
          budget: 850,
          status: 'planned',
          created_at: '2026-03-13T10:00:00Z',
          updated_at: '2026-03-13T10:00:00Z',
        },
      ]
    })
  })

  it('renders projection-backed event rows', async () => {
    renderWithProviders(<EventsPage />)

    await waitFor(() => {
      expect(screen.getByText('Defense rehearsal')).toBeInTheDocument()
    })

    expect(screen.getByText(/sorted by starts at/i)).toBeInTheDocument()
    expect(screen.getAllByText('Conference').length).toBeGreaterThan(0)
  })
})
