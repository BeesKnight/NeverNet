import { fireEvent, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { CategoriesPage } from './CategoriesPage'
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
  invalidateReadSideQueries: vi.fn().mockResolvedValue(undefined),
}))

vi.mock('../features/auth/auth-context', () => ({
  useAuth: () => mocks.auth,
}))

vi.mock('../api/client', () => ({
  apiRequest: mocks.apiRequest,
}))

vi.mock('../api/query-utils', () => ({
  invalidateReadSideQueries: mocks.invalidateReadSideQueries,
}))

describe('CategoriesPage', () => {
  beforeEach(() => {
    mocks.apiRequest.mockReset()
    mocks.invalidateReadSideQueries.mockClear()
    mocks.apiRequest.mockImplementation(async (path: string, options?: RequestInit) => {
      if (path === '/categories' && !options?.method) {
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

      return {
        id: 'category-1',
        user_id: 'user-1',
        name: 'Updated category',
        color: '#123456',
        created_at: '2026-03-13T10:00:00Z',
        updated_at: '2026-03-13T10:00:00Z',
      }
    })
  })

  it('renders categories and supports create, edit, and delete flows', async () => {
    const user = userEvent.setup()

    renderWithProviders(<CategoriesPage />)

    await waitFor(() => {
      expect(screen.getByText('Conference')).toBeInTheDocument()
    })

    await user.type(screen.getByLabelText('Category name'), 'New category')
    fireEvent.input(screen.getByDisplayValue('#0f766e'), {
      target: { value: '#123456' },
    })
    await user.click(screen.getByRole('button', { name: 'Create category' }))

    await waitFor(() => {
      expect(mocks.apiRequest).toHaveBeenCalledWith(
        '/categories',
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify({ name: 'New category', color: '#123456' }),
        }),
      )
    })

    await user.click(screen.getByRole('button', { name: 'Edit' }))
    await user.clear(screen.getByLabelText('Category name'))
    await user.type(screen.getByLabelText('Category name'), 'Updated category')
    await user.click(screen.getByRole('button', { name: 'Save category' }))
    await user.click(screen.getByRole('button', { name: 'Delete' }))

    expect(mocks.apiRequest).toHaveBeenCalledWith(
      '/categories/category-1',
      expect.objectContaining({
        method: 'PATCH',
        body: JSON.stringify({ name: 'Updated category', color: '#0f766e' }),
      }),
    )
    expect(mocks.apiRequest).toHaveBeenCalledWith(
      '/categories/category-1',
      expect.objectContaining({
        method: 'DELETE',
      }),
    )
    expect(mocks.invalidateReadSideQueries).toHaveBeenCalledTimes(3)
  })
})
