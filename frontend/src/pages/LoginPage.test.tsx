import { Route, Routes } from 'react-router-dom'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { LoginPage } from './LoginPage'
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

describe('LoginPage', () => {
  beforeEach(() => {
    mocks.auth.session = null
    mocks.auth.isInitializing = false
    mocks.auth.login.mockReset().mockResolvedValue(undefined)
  })

  it('submits credentials through the auth context and redirects home', async () => {
    const user = userEvent.setup()

    renderWithProviders(
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/" element={<div>Home</div>} />
      </Routes>,
      '/login',
    )

    await user.type(screen.getByLabelText('Email'), 'demo@eventdesign.local')
    await user.type(screen.getByLabelText('Password'), 'DemoPass123!')
    await user.click(screen.getByRole('button', { name: 'Sign in' }))

    expect(mocks.auth.login).toHaveBeenCalledWith({
      email: 'demo@eventdesign.local',
      password: 'DemoPass123!',
    })
    await waitFor(() => {
      expect(screen.getByText('Home')).toBeInTheDocument()
    })
  })
})
