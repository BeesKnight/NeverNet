import { useQuery } from '@tanstack/react-query'
import { format } from 'date-fns'

import { apiRequest } from '../api/client'
import type { Event, ExportJob, ReportSummary } from '../api/types'
import { useAuth } from '../features/auth/auth-context'

export function DashboardPage() {
  const { session } = useAuth()
  const token = session?.token ?? ''

  const eventsQuery = useQuery({
    queryKey: ['events', session?.user.id, 'dashboard'],
    queryFn: () => apiRequest<Event[]>('/events', { token }),
  })

  const reportsQuery = useQuery({
    queryKey: ['reports', session?.user.id, 'dashboard'],
    queryFn: () => apiRequest<ReportSummary>('/reports/summary', { token }),
  })

  const exportsQuery = useQuery({
    queryKey: ['exports', session?.user.id, 'dashboard'],
    queryFn: () => apiRequest<ExportJob[]>('/exports', { token }),
  })

  const events = eventsQuery.data ?? []
  const exports = exportsQuery.data ?? []
  const report = reportsQuery.data
  const now = new Date()
  const upcomingEvents = events.filter((event) => new Date(event.starts_at) >= now && event.status !== 'cancelled')
  const completedEvents = events.filter((event) => event.status === 'completed')
  const cancelledEvents = events.filter((event) => event.status === 'cancelled')
  const recentActivity = [...events]
    .sort((left, right) => new Date(right.updated_at).getTime() - new Date(left.updated_at).getTime())
    .slice(0, 5)

  return (
    <div className="page-shell">
      <section className="hero-card">
        <div>
          <p className="eyebrow">Overview</p>
          <h2>Keep the event pipeline visible from planning to delivery.</h2>
        </div>
        <p className="hero-copy">
          Track the full schedule, keep category ownership clean, and export the same report scope as PDF or XLSX.
        </p>
      </section>

      <section className="stats-grid">
        <article className="stat-card">
          <p className="eyebrow">Total events</p>
          <strong>{report?.total_events ?? 0}</strong>
          <span>Tracked across all statuses</span>
        </article>
        <article className="stat-card">
          <p className="eyebrow">Upcoming</p>
          <strong>{upcomingEvents.length}</strong>
          <span>Future events still on the board</span>
        </article>
        <article className="stat-card">
          <p className="eyebrow">Completed</p>
          <strong>{completedEvents.length}</strong>
          <span>Events marked as delivered</span>
        </article>
        <article className="stat-card">
          <p className="eyebrow">Cancelled</p>
          <strong>{cancelledEvents.length}</strong>
          <span>Events removed from execution</span>
        </article>
      </section>

      <section className="content-grid">
        <article className="section-card">
          <div className="section-header">
            <div>
              <p className="eyebrow">Upcoming</p>
              <h2>Next events</h2>
            </div>
          </div>

          <div className="list-stack">
            {upcomingEvents.slice(0, 5).map((event) => (
              <div className="list-row" key={event.id}>
                <div>
                  <strong>{event.title}</strong>
                  <p className="muted">
                    {event.category_name} | {format(new Date(event.starts_at), 'MMM d, yyyy HH:mm')}
                  </p>
                </div>
                <span className={`status-pill ${event.status}`}>{event.status.replace('_', ' ')}</span>
              </div>
            ))}
            {!upcomingEvents.length ? <div className="empty-state">No upcoming events scheduled.</div> : null}
          </div>
        </article>

        <article className="section-card">
          <div className="section-header">
            <div>
              <p className="eyebrow">Recent activity</p>
              <h2>Latest event updates</h2>
            </div>
          </div>

          <div className="list-stack">
            {recentActivity.map((event) => (
              <div className="list-row" key={event.id}>
                <div>
                  <strong>{event.title}</strong>
                  <p className="muted">
                    Updated {format(new Date(event.updated_at), 'MMM d, yyyy HH:mm')} | {event.location}
                  </p>
                </div>
                <span className={`status-pill ${event.status}`}>{event.status.replace('_', ' ')}</span>
              </div>
            ))}
            {!recentActivity.length ? <div className="empty-state">No recent activity yet.</div> : null}
          </div>
        </article>
      </section>

      <section className="section-card">
        <div className="section-header">
          <div>
            <p className="eyebrow">Background jobs</p>
            <h2>Export queue</h2>
          </div>
        </div>

        <div className="list-stack">
          {exports.slice(0, 5).map((job) => (
            <div className="list-row" key={job.id}>
              <div>
                <strong>{job.format.toUpperCase()} summary export</strong>
                <p className="muted">{format(new Date(job.created_at), 'MMM d, yyyy HH:mm')}</p>
              </div>
              <span className={`status-pill ${job.status}`}>{job.status}</span>
            </div>
          ))}
          {!exports.length ? <div className="empty-state">No export jobs yet.</div> : null}
        </div>
      </section>
    </div>
  )
}
