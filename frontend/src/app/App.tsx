import { useQuery } from '@tanstack/react-query'
import { useEffect } from 'react'

import { apiRequest } from '../api/client'
import type { UiSettings } from '../api/types'
import { useAuth } from '../features/auth/auth-context'
import { AppRoutes } from './router'

export function App() {
  const auth = useAuth()

  const settingsQuery = useQuery({
    queryKey: ['settings', 'theme-sync', auth.session?.user.id],
    queryFn: () => apiRequest<UiSettings>('/settings/', { token: auth.session?.token }),
    enabled: Boolean(auth.session?.token),
  })

  useEffect(() => {
    const selectedTheme = settingsQuery.data?.theme ?? 'system'
    const theme =
      selectedTheme === 'system'
        ? window.matchMedia('(prefers-color-scheme: dark)').matches
          ? 'dark'
          : 'light'
        : selectedTheme

    document.documentElement.dataset.theme = theme
  }, [settingsQuery.data?.theme])

  return <AppRoutes />
}
