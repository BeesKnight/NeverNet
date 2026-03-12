import { Navigate, Outlet, Route, Routes } from 'react-router-dom'

import { AppLayout } from '../components/AppLayout'
import { useAuth } from '../features/auth/auth-context'
import { CalendarPage } from '../pages/CalendarPage'
import { CategoriesPage } from '../pages/CategoriesPage'
import { DashboardPage } from '../pages/DashboardPage'
import { EventsPage } from '../pages/EventsPage'
import { LoginPage } from '../pages/LoginPage'
import { RegisterPage } from '../pages/RegisterPage'
import { ReportsPage } from '../pages/ReportsPage'
import { SettingsPage } from '../pages/SettingsPage'

function ProtectedOutlet() {
  const auth = useAuth()

  if (auth.isInitializing) {
    return <div className="page-shell"><div className="empty-state">Loading session...</div></div>
  }

  if (!auth.session) {
    return <Navigate to="/login" replace />
  }

  return <AppLayout><Outlet /></AppLayout>
}

function PublicOnlyOutlet() {
  const auth = useAuth()

  if (auth.isInitializing) {
    return <div className="page-shell"><div className="empty-state">Loading session...</div></div>
  }

  if (auth.session) {
    return <Navigate to="/dashboard" replace />
  }

  return <Outlet />
}

export function AppRoutes() {
  return (
    <Routes>
      <Route element={<PublicOnlyOutlet />}>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/register" element={<RegisterPage />} />
      </Route>

      <Route element={<ProtectedOutlet />}>
        <Route path="/" element={<Navigate to="/dashboard" replace />} />
        <Route path="/dashboard" element={<DashboardPage />} />
        <Route path="/categories" element={<CategoriesPage />} />
        <Route path="/events" element={<EventsPage />} />
        <Route path="/calendar" element={<CalendarPage />} />
        <Route path="/reports" element={<ReportsPage />} />
        <Route path="/settings" element={<SettingsPage />} />
      </Route>

      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  )
}
