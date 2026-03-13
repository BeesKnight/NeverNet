import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { apiRequest } from '../api/client'
import type { UiSettings } from '../api/types'
import { useAuth } from '../features/auth/auth-context'

export function SettingsPage() {
  const { session } = useAuth()
  const queryClient = useQueryClient()

  const settingsQuery = useQuery({
    queryKey: ['settings', session?.user.id],
    queryFn: () => apiRequest<UiSettings>('/settings'),
    enabled: Boolean(session?.user.id),
  })

  const updateSettings = useMutation({
    mutationFn: (payload: Partial<Pick<UiSettings, 'theme' | 'accent_color' | 'default_view'>>) =>
      apiRequest<UiSettings>('/settings', {
        method: 'PATCH',
        body: JSON.stringify(payload),
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
            <h2>Interface settings</h2>
          </div>
        </div>

        <div className="form-grid">
          <label>
            <span>Theme</span>
          </label>

          <div className="button-row">
            {(['system', 'light', 'dark'] as const).map((theme) => (
              <button
                key={theme}
                className={settings?.theme === theme ? 'primary-button' : 'ghost-button'}
                type="button"
                onClick={() => updateSettings.mutate({ theme })}
              >
                {theme}
              </button>
            ))}
          </div>

          <label>
            <span>Accent color</span>
            <div className="form-actions">
              <input
                type="color"
                value={settings?.accent_color ?? '#b6532f'}
                onChange={(event) =>
                  updateSettings.mutate({ accent_color: event.target.value })
                }
              />
              <span className="muted">{settings?.accent_color ?? '#b6532f'}</span>
            </div>
          </label>

          <label>
            <span>Default start page</span>
            <select
              value={settings?.default_view ?? 'dashboard'}
              onChange={(event) =>
                updateSettings.mutate({
                  default_view: event.target.value as UiSettings['default_view'],
                })
              }
            >
              <option value="dashboard">Dashboard</option>
              <option value="events">Events</option>
              <option value="calendar">Calendar</option>
              <option value="reports">Reports</option>
            </select>
          </label>
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
          <div>
            <span className="muted">Accent</span>
            <strong>{settings?.accent_color ?? '#b6532f'}</strong>
          </div>
          <div>
            <span className="muted">Default view</span>
            <strong>{settings?.default_view ?? 'dashboard'}</strong>
          </div>
        </div>
      </section>
    </div>
  )
}
