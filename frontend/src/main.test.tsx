import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => {
  const render = vi.fn()

  return {
    render,
    createRoot: vi.fn(() => ({ render })),
  }
})

vi.mock('react-dom/client', () => ({
  createRoot: mocks.createRoot,
}))

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual<typeof import('react-router-dom')>('react-router-dom')

  return {
    ...actual,
    BrowserRouter: ({ children }: { children: React.ReactNode }) => children,
  }
})

vi.mock('./features/auth/auth-context', () => ({
  AuthProvider: ({ children }: { children: React.ReactNode }) => children,
}))

vi.mock('./app/App', () => ({
  App: () => <div>App Shell</div>,
}))

describe('main bootstrap', () => {
  beforeEach(() => {
    vi.resetModules()
    mocks.createRoot.mockClear()
    mocks.render.mockClear()
    document.body.innerHTML = '<div id="root"></div>'
  })

  it('mounts the application into the root container', async () => {
    await import('./main')

    expect(mocks.createRoot).toHaveBeenCalledWith(document.getElementById('root'))
    expect(mocks.render).toHaveBeenCalledTimes(1)
  })
})
