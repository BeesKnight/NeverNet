import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { ReportsPage } from './ReportsPage'
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
  apiDownload: vi.fn(),
}))

vi.mock('../features/auth/auth-context', () => ({
  useAuth: () => mocks.auth,
}))

vi.mock('../api/client', () => ({
  apiRequest: mocks.apiRequest,
  apiDownload: mocks.apiDownload,
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

describe('ReportsPage', () => {
  beforeEach(() => {
    mocks.apiRequest.mockReset()
    mocks.apiDownload.mockReset()
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

      if (path.startsWith('/reports/summary')) {
        return {
          filters: {},
          period_start: '2026-03-01',
          period_end: '2026-03-31',
          total_events: 1,
          total_budget: 850,
          by_category: [
            {
              category_id: 'category-1',
              category_name: 'Conference',
              category_color: '#0f766e',
              event_count: 1,
              total_budget: 850,
            },
          ],
          by_status: [
            {
              status: 'planned',
              event_count: 1,
              total_budget: 850,
            },
          ],
          events: [
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
        }
      }

      return [
        {
          id: 'export-1',
          user_id: 'user-1',
          report_type: 'summary',
          format: 'pdf',
          status: 'completed',
          filters: {},
          object_key: '/exports/user-1/export-1.pdf',
          content_type: 'application/pdf',
          error_message: null,
          created_at: '2026-03-13T10:00:00Z',
          started_at: '2026-03-13T10:01:00Z',
          updated_at: '2026-03-13T10:02:00Z',
          finished_at: '2026-03-13T10:02:00Z',
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
        {
          id: 'export-3',
          user_id: 'user-1',
          report_type: 'summary',
          format: 'pdf',
          status: 'failed',
          filters: {},
          object_key: null,
          content_type: null,
          error_message: 'Projection snapshot is not ready yet',
          created_at: '2026-03-13T10:10:00Z',
          started_at: '2026-03-13T10:11:00Z',
          updated_at: '2026-03-13T10:12:00Z',
          finished_at: '2026-03-13T10:12:00Z',
        },
      ]
    })
  })

  it('renders report aggregates, export statuses, and download action', async () => {
    const user = userEvent.setup()

    renderWithProviders(<ReportsPage />)

    await waitFor(() => {
      expect(screen.getByText('Defense rehearsal')).toBeInTheDocument()
    })

    expect(screen.getAllByText(/average budget/i).length).toBeGreaterThan(0)
    expect(screen.getByText(/Preview sorted by starts at/i)).toBeInTheDocument()
    expect(screen.getByText(/still running in the background/i)).toBeInTheDocument()
    expect(screen.getByText(/failed and remain available for review/i)).toBeInTheDocument()
    expect(screen.getByText('Projection snapshot is not ready yet')).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'Download' }))

    expect(mocks.apiDownload).toHaveBeenCalledWith(
      '/exports/export-1/download',
      'eventdesign-report-export-1.pdf',
    )
  })
})
