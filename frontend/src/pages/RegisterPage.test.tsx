import { Route, Routes } from 'react-router-dom'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { ApiError } from '../api/client'
import { RegisterPage } from './RegisterPage'
import { renderWithProviders } from '../test/test-utils'

const mocks = vi.hoisted(() => ({
  auth: {
    session: null,
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

describe('RegisterPage', () => {
  beforeEach(() => {
    mocks.auth.register.mockReset().mockResolvedValue(undefined)
  })

  it('submits registration data and redirects to home', async () => {
    const user = userEvent.setup()

    renderWithProviders(
      <Routes>
        <Route path="/register" element={<RegisterPage />} />
        <Route path="/" element={<div>Home</div>} />
      </Routes>,
      '/register',
    )

    await user.type(screen.getByLabelText('Full name'), 'Demo User')
    await user.type(screen.getByLabelText('Email'), 'demo@eventdesign.local')
    await user.type(screen.getByLabelText('Password'), 'DemoPass123!')
    await user.click(screen.getByRole('button', { name: 'Create account' }))

    expect(mocks.auth.register).toHaveBeenCalledWith({
      full_name: 'Demo User',
      email: 'demo@eventdesign.local',
      password: 'DemoPass123!',
    })
    await waitFor(() => {
      expect(screen.getByText('Home')).toBeInTheDocument()
    })
  })

  it('shows api errors without navigating away', async () => {
    const user = userEvent.setup()
    mocks.auth.register.mockRejectedValueOnce(new ApiError('Email already exists', 409))

    renderWithProviders(
      <Routes>
        <Route path="/register" element={<RegisterPage />} />
        <Route path="/" element={<div>Home</div>} />
      </Routes>,
      '/register',
    )

    await user.type(screen.getByLabelText('Full name'), 'Demo User')
    await user.type(screen.getByLabelText('Email'), 'demo@eventdesign.local')
    await user.type(screen.getByLabelText('Password'), 'DemoPass123!')
    await user.click(screen.getByRole('button', { name: 'Create account' }))

    await waitFor(() => {
      expect(screen.getByText('Email already exists')).toBeInTheDocument()
    })
    expect(screen.queryByText('Home')).not.toBeInTheDocument()
  })
})
