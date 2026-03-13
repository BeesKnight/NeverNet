import { NavLink } from 'react-router-dom'
import type { PropsWithChildren } from 'react'

import { useAuth } from '../features/auth/auth-context'

const navigation = [
  { to: '/dashboard', label: 'Dashboard' },
  { to: '/events', label: 'Events' },
  { to: '/categories', label: 'Categories' },
  { to: '/calendar', label: 'Calendar' },
  { to: '/reports', label: 'Reports' },
  { to: '/settings', label: 'Settings' },
]

export function AppLayout({ children }: PropsWithChildren) {
  const { session, logout } = useAuth()

  return (
    <div className="shell">
      <aside className="sidebar">
        <div className="brand-block">
          <p className="eyebrow">EventDesign</p>
          <h1>Plan events with enough structure to stay defendable.</h1>
          <p className="sidebar-copy">
            Modular event planning with reports, exports, persistent settings, and a live calendar view.
          </p>
        </div>

        <nav className="nav-list">
          {navigation.map((item) => (
            <NavLink
              key={item.to}
              className={({ isActive }) => `nav-item${isActive ? ' active' : ''}`}
              to={item.to}
            >
              {item.label}
            </NavLink>
          ))}
        </nav>

        <div className="sidebar-footer">
          <div>
            <p className="eyebrow">Signed in as</p>
            <strong>{session?.user.full_name}</strong>
            <p className="muted">{session?.user.email}</p>
          </div>
          <button className="ghost-button" type="button" onClick={logout}>
            Log out
          </button>
        </div>
      </aside>

      <main className="content">{children}</main>
    </div>
  )
}
