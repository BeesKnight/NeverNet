import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { AuthProvider, useAuth } from './auth-context'

const mocks = vi.hoisted(() => ({
  ApiError: class MockApiError extends Error {
    status: number

    constructor(message: string, status: number) {
      super(message)
      this.status = status
    }
  },
  apiRequest: vi.fn(),
  refreshCsrfToken: vi.fn(),
  clearCsrfToken: vi.fn(),
}))

vi.mock('../../api/client', () => ({
  ApiError: mocks.ApiError,
  apiRequest: mocks.apiRequest,
  refreshCsrfToken: mocks.refreshCsrfToken,
  clearCsrfToken: mocks.clearCsrfToken,
}))

function AuthProbe() {
  const auth = useAuth()

  return (
    <div>
      <div data-testid="initializing">{String(auth.isInitializing)}</div>
      <div data-testid="email">{auth.session?.user.email ?? 'guest'}</div>
      <button
        type="button"
        onClick={() =>
          void auth.login({
            email: 'demo@eventdesign.local',
            password: 'DemoPass123!',
          })
        }
      >
        Login
      </button>
      <button
        type="button"
        onClick={() =>
          void auth.register({
            full_name: 'Demo User',
            email: 'demo@eventdesign.local',
            password: 'DemoPass123!',
          })
        }
      >
        Register
      </button>
      <button type="button" onClick={() => auth.logout()}>
        Logout
      </button>
      <button
        type="button"
        onClick={() =>
          auth.updateUser({
            id: 'user-1',
            email: 'updated@eventdesign.local',
            full_name: 'Updated User',
            created_at: '2026-03-13T10:00:00Z',
          })
        }
      >
        Update user
      </button>
    </div>
  )
}

function renderAuthProvider() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
      mutations: {
        retry: false,
      },
    },
  })

  return {
    queryClient,
    ...render(
      <QueryClientProvider client={queryClient}>
        <AuthProvider>
          <AuthProbe />
        </AuthProvider>
      </QueryClientProvider>,
    ),
  }
}

describe('AuthProvider', () => {
  beforeEach(() => {
    mocks.apiRequest.mockReset()
    mocks.refreshCsrfToken.mockReset().mockResolvedValue('csrf-token')
    mocks.clearCsrfToken.mockReset()
  })

  it('bootstraps the current user session on mount', async () => {
    mocks.apiRequest.mockResolvedValueOnce({
      user: {
        id: 'user-1',
        email: 'demo@eventdesign.local',
        full_name: 'Demo User',
        created_at: '2026-03-13T10:00:00Z',
      },
    })

    renderAuthProvider()

    await waitFor(() => {
      expect(screen.getByTestId('initializing')).toHaveTextContent('false')
    })

    expect(mocks.refreshCsrfToken).toHaveBeenCalledWith()
    expect(mocks.apiRequest).toHaveBeenCalledWith('/auth/me')
    expect(screen.getByTestId('email')).toHaveTextContent('demo@eventdesign.local')
  })

  it('treats unauthorized bootstrap as a logged-out state', async () => {
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => undefined)
    mocks.apiRequest.mockRejectedValueOnce(new mocks.ApiError('Unauthorized', 401))

    const { queryClient } = renderAuthProvider()
    queryClient.setQueryData(['stale'], { value: true })

    await waitFor(() => {
      expect(screen.getByTestId('initializing')).toHaveTextContent('false')
    })

    expect(screen.getByTestId('email')).toHaveTextContent('guest')
    expect(queryClient.getQueryData(['stale'])).toBeUndefined()
    expect(consoleError).not.toHaveBeenCalled()
  })

  it('handles login, register, logout, and updateUser actions', async () => {
    const user = userEvent.setup()
    mocks.apiRequest
      .mockResolvedValueOnce({
        user: {
          id: 'user-1',
          email: 'demo@eventdesign.local',
          full_name: 'Demo User',
          created_at: '2026-03-13T10:00:00Z',
        },
      })
      .mockResolvedValueOnce({
        user: {
          id: 'user-1',
          email: 'demo@eventdesign.local',
          full_name: 'Demo User',
          created_at: '2026-03-13T10:00:00Z',
        },
      })
      .mockResolvedValueOnce({
        user: {
          id: 'user-1',
          email: 'registered@eventdesign.local',
          full_name: 'Registered User',
          created_at: '2026-03-13T10:00:00Z',
        },
      })
      .mockResolvedValueOnce(undefined)

    renderAuthProvider()

    await waitFor(() => {
      expect(screen.getByTestId('email')).toHaveTextContent('demo@eventdesign.local')
    })

    await user.click(screen.getByRole('button', { name: 'Login' }))
    await waitFor(() => {
      expect(mocks.apiRequest).toHaveBeenCalledWith(
        '/auth/login',
        expect.objectContaining({
          method: 'POST',
        }),
      )
    })

    await user.click(screen.getByRole('button', { name: 'Register' }))
    await waitFor(() => {
      expect(screen.getByTestId('email')).toHaveTextContent('registered@eventdesign.local')
    })

    await user.click(screen.getByRole('button', { name: 'Update user' }))
    expect(screen.getByTestId('email')).toHaveTextContent('updated@eventdesign.local')

    await user.click(screen.getByRole('button', { name: 'Logout' }))
    await waitFor(() => {
      expect(screen.getByTestId('email')).toHaveTextContent('guest')
    })

    expect(mocks.apiRequest).toHaveBeenCalledWith(
      '/auth/register',
      expect.objectContaining({
        method: 'POST',
      }),
    )
    expect(mocks.apiRequest).toHaveBeenCalledWith(
      '/auth/logout',
      expect.objectContaining({
        method: 'POST',
      }),
    )
    expect(mocks.clearCsrfToken).toHaveBeenCalledTimes(1)
    expect(mocks.refreshCsrfToken).toHaveBeenCalledWith(true)
  })
})
