import { useQuery } from '@tanstack/react-query'
import { useEffect } from 'react'

import { apiRequest } from '../api/client'
import type { UiSettings } from '../api/types'
import { useAuth } from '../features/auth/auth-context'
import { AppRoutes } from './router'

function resolveTheme(theme: UiSettings['theme']) {
  if (theme === 'system') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  }

  return theme
}

function darkenHexColor(value: string, amount: number) {
  const normalized = value.replace('#', '')
  const channels = [0, 2, 4].map((start) => {
    const channel = Number.parseInt(normalized.slice(start, start + 2), 16)
    const next = Math.max(0, Math.min(255, channel - amount))
    return next.toString(16).padStart(2, '0')
  })

  return `#${channels.join('')}`
}

export function App() {
  const auth = useAuth()

  const settingsQuery = useQuery({
    queryKey: ['settings', 'theme-sync', auth.session?.user.id],
    queryFn: () => apiRequest<UiSettings>('/settings', { token: auth.session?.token }),
    enabled: Boolean(auth.session?.token),
  })

  useEffect(() => {
    const theme = resolveTheme(settingsQuery.data?.theme ?? 'system')
    const accentColor = settingsQuery.data?.accent_color ?? '#b6532f'

    document.documentElement.dataset.theme = theme
    document.documentElement.style.setProperty('--accent', accentColor)
    document.documentElement.style.setProperty('--accent-strong', darkenHexColor(accentColor, 28))
  }, [settingsQuery.data?.accent_color, settingsQuery.data?.theme])

  const defaultAuthenticatedPath = settingsQuery.data?.default_view
    ? `/${settingsQuery.data.default_view}`
    : '/dashboard'

  return <AppRoutes defaultAuthenticatedPath={defaultAuthenticatedPath} />
}
