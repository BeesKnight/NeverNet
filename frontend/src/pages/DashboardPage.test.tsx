import { screen, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { DashboardPage } from './DashboardPage'
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
}))

describe('DashboardPage', () => {
  beforeEach(() => {
    mocks.apiRequest.mockReset()
    mocks.apiRequest.mockImplementation(async (path: string) => {
      if (path === '/dashboard') {
        return {
          cards: {
            total_events: 4,
            upcoming_events: 2,
            completed_events: 1,
            cancelled_events: 1,
            total_budget: 2750,
          },
          upcoming: [
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
          ],
          recent_activity: [
            {
              id: 'activity-1',
              entity_type: 'event',
              entity_id: 'event-1',
              action: 'created',
              title: 'Defense rehearsal',
              occurred_at: '2026-03-13T11:00:00Z',
            },
          ],
        }
      }

      return [
        {
          id: 'export-1',
          user_id: 'user-1',
          report_type: 'summary',
          format: 'pdf',
          status: 'queued',
          filters: {},
          object_key: null,
          content_type: null,
          error_message: null,
          created_at: '2026-03-13T10:00:00Z',
          started_at: null,
          updated_at: '2026-03-13T10:00:00Z',
          finished_at: null,
        },
        {
          id: 'export-2',
          user_id: 'user-1',
          report_type: 'summary',
          format: 'xlsx',
          status: 'processing',
          filters: {},
          object_key: null,
          content_type: null,
          error_message: null,
          created_at: '2026-03-13T10:05:00Z',
          started_at: '2026-03-13T10:06:00Z',
          updated_at: '2026-03-13T10:06:00Z',
          finished_at: null,
        },
      ]
    })
  })

  it('renders dashboard cards, upcoming events, and export queue state', async () => {
    renderWithProviders(<DashboardPage />)

    await waitFor(() => {
      expect(screen.getAllByText('Defense rehearsal').length).toBeGreaterThan(0)
    })

    expect(screen.getByText('4')).toBeInTheDocument()
    expect(screen.getByText(/\$2750/i)).toBeInTheDocument()
    expect(screen.getByText(/1 queued and 1 processing export jobs/i)).toBeInTheDocument()
    expect(screen.getByText('Latest event updates')).toBeInTheDocument()
  })
})
