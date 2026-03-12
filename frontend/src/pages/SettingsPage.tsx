import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { apiRequest } from '../api/client'
import type { UiSettings } from '../api/types'
import { useAuth } from '../features/auth/auth-context'

export function SettingsPage() {
  const { session } = useAuth()
  const token = session?.token ?? ''
  const queryClient = useQueryClient()

  const settingsQuery = useQuery({
    queryKey: ['settings'],
    queryFn: () => apiRequest<UiSettings>('/settings/', { token }),
  })

  const updateSettings = useMutation({
    mutationFn: (theme: UiSettings['theme']) =>
      apiRequest<UiSettings>('/settings/', {
        method: 'PUT',
        token,
        body: JSON.stringify({ theme }),
      }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['settings'] })
    },
  })

  const settings = settingsQuery.data

  return (
    <div className="page-shell two-column-page">
      <section className="section-card">
        <div className="section-header">
          <div>
            <p className="eyebrow">Preferences</p>
            <h2>Theme settings</h2>
          </div>
        </div>

        <div className="button-row">
          {(['system', 'light', 'dark'] as const).map((theme) => (
            <button
              key={theme}
              className={settings?.theme === theme ? 'primary-button' : 'ghost-button'}
              type="button"
              onClick={() => updateSettings.mutate(theme)}
            >
              {theme}
            </button>
          ))}
        </div>
      </section>

      <section className="section-card">
        <div className="section-header">
          <div>
            <p className="eyebrow">Account</p>
            <h2>User profile</h2>
          </div>
        </div>

        <div className="profile-grid">
          <div>
            <span className="muted">Full name</span>
            <strong>{session?.user.full_name}</strong>
          </div>
          <div>
            <span className="muted">Email</span>
            <strong>{session?.user.email}</strong>
          </div>
          <div>
            <span className="muted">Theme</span>
            <strong>{settings?.theme ?? 'system'}</strong>
          </div>
        </div>
      </section>
    </div>
  )
}
