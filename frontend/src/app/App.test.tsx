import { render, screen } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { App } from './App'

const mocks = vi.hoisted(() => ({
  auth: {
    session: null as
      | null
      | {
          user: {
            id: string
          }
        },
  },
  useQuery: vi.fn(),
}))

vi.mock('@tanstack/react-query', async () => {
  const actual = await vi.importActual<typeof import('@tanstack/react-query')>(
    '@tanstack/react-query',
  )

  return {
    ...actual,
    useQuery: mocks.useQuery,
  }
})

vi.mock('../features/auth/auth-context', () => ({
  useAuth: () => mocks.auth,
}))

vi.mock('./router', () => ({
  AppRoutes: ({ defaultAuthenticatedPath }: { defaultAuthenticatedPath: string }) => (
    <div data-testid="routes">{defaultAuthenticatedPath}</div>
  ),
}))

describe('App', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mocks.auth.session = null
    window.matchMedia = vi.fn().mockImplementation((query: string) => ({
      matches: query.includes('dark'),
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }))
    document.documentElement.dataset.theme = ''
    document.documentElement.style.removeProperty('--accent')
    document.documentElement.style.removeProperty('--accent-strong')
  })

  it('syncs theme settings and passes the configured default route', () => {
    mocks.auth.session = { user: { id: 'user-1' } }
    mocks.useQuery.mockReturnValue({
      data: {
        theme: 'system',
        accent_color: '#abcdef',
        default_view: 'reports',
      },
    })

    render(<App />)

    expect(screen.getByTestId('routes')).toHaveTextContent('/reports')
    expect(document.documentElement.dataset.theme).toBe('dark')
    expect(document.documentElement.style.getPropertyValue('--accent')).toBe('#abcdef')
    expect(document.documentElement.style.getPropertyValue('--accent-strong')).toBe('#8fb1d3')
  })

  it('falls back to dashboard and default accent when settings are absent', () => {
    mocks.useQuery.mockReturnValue({ data: undefined })

    render(<App />)

    expect(screen.getByTestId('routes')).toHaveTextContent('/dashboard')
    expect(document.documentElement.dataset.theme).toBe('dark')
    expect(document.documentElement.style.getPropertyValue('--accent')).toBe('#b6532f')
    expect(document.documentElement.style.getPropertyValue('--accent-strong')).toBe('#9a3713')
  })
})
