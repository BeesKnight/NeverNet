import { fireEvent, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { SettingsPage } from './SettingsPage'
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

describe('SettingsPage', () => {
  beforeEach(() => {
    mocks.apiRequest.mockReset()
    mocks.apiRequest.mockImplementation(async (path: string, options?: RequestInit) => {
      if (path === '/settings' && !options?.method) {
        return {
          user_id: 'user-1',
          theme: 'system',
          accent_color: '#b6532f',
          default_view: 'dashboard',
          created_at: '2026-03-13T10:00:00Z',
          updated_at: '2026-03-13T10:00:00Z',
        }
      }

      return {
        user_id: 'user-1',
        theme: 'dark',
        accent_color: '#112233',
        default_view: 'calendar',
        created_at: '2026-03-13T10:00:00Z',
        updated_at: '2026-03-13T10:00:00Z',
      }
    })
  })

  it('renders user settings and persists mutations', async () => {
    const user = userEvent.setup()

    renderWithProviders(<SettingsPage />)

    await waitFor(() => {
      expect(screen.getByText('Interface settings')).toBeInTheDocument()
    })

    await user.click(screen.getByRole('button', { name: 'dark' }))
    fireEvent.input(screen.getByDisplayValue('#b6532f'), {
      target: { value: '#112233' },
    })
    await user.selectOptions(screen.getByLabelText('Default start page'), 'calendar')

    await waitFor(() => {
      expect(mocks.apiRequest).toHaveBeenCalledWith(
        '/settings',
        expect.objectContaining({
          method: 'PATCH',
          body: JSON.stringify({ theme: 'dark' }),
        }),
      )
    })

    expect(mocks.apiRequest).toHaveBeenCalledWith(
      '/settings',
      expect.objectContaining({
        method: 'PATCH',
        body: JSON.stringify({ accent_color: '#112233' }),
      }),
    )
    expect(mocks.apiRequest).toHaveBeenCalledWith(
      '/settings',
      expect.objectContaining({
        method: 'PATCH',
        body: JSON.stringify({ default_view: 'calendar' }),
      }),
    )
    expect(screen.getByText('Demo User')).toBeInTheDocument()
    expect(screen.getByText('demo@eventdesign.local')).toBeInTheDocument()
  })
})
