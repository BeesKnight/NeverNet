import { useQuery } from '@tanstack/react-query'
import { format } from 'date-fns'

import { apiRequest } from '../api/client'
import type { Category, Event, ExportJob, ReportSummary } from '../api/types'
import { useAuth } from '../features/auth/auth-context'

export function DashboardPage() {
  const { session } = useAuth()
  const token = session?.token ?? ''

  const categoriesQuery = useQuery({
    queryKey: ['categories', 'dashboard'],
    queryFn: () => apiRequest<Category[]>('/categories/', { token }),
  })

  const eventsQuery = useQuery({
    queryKey: ['events', 'dashboard'],
    queryFn: () => apiRequest<Event[]>('/events/', { token }),
  })

  const reportsQuery = useQuery({
    queryKey: ['reports', 'dashboard'],
    queryFn: () => apiRequest<ReportSummary>('/reports/summary', { token }),
  })

  const exportsQuery = useQuery({
    queryKey: ['exports', 'dashboard'],
    queryFn: () => apiRequest<ExportJob[]>('/exports/', { token }),
  })

  const events = eventsQuery.data ?? []
  const exports = exportsQuery.data ?? []
  const categories = categoriesQuery.data ?? []
  const report = reportsQuery.data

  return (
    <div className="page-shell">
      <section className="hero-card">
        <div>
          <p className="eyebrow">Overview</p>
          <h2>Keep the event pipeline visible from planning to delivery.</h2>
        </div>
        <p className="hero-copy">
          Use categories for structure, filter events by status and date, and export the same
          report view as PDF or XLSX.
        </p>
      </section>

      <section className="stats-grid">
        <article className="stat-card">
          <p className="eyebrow">Events</p>
          <strong>{report?.total_events ?? 0}</strong>
          <span>Tracked across all statuses</span>
        </article>
        <article className="stat-card">
          <p className="eyebrow">Budget</p>
          <strong>${(report?.total_budget ?? 0).toFixed(2)}</strong>
          <span>Total budget in current scope</span>
        </article>
        <article className="stat-card">
          <p className="eyebrow">Categories</p>
          <strong>{categories.length}</strong>
          <span>Reusable event groups</span>
        </article>
        <article className="stat-card">
          <p className="eyebrow">Exports</p>
          <strong>{exports.length}</strong>
          <span>Generated and queued files</span>
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
            {events.slice(0, 5).map((event) => (
              <div className="list-row" key={event.id}>
                <div>
                  <strong>{event.title}</strong>
                  <p className="muted">
                    {event.category_name} · {format(new Date(event.starts_at), 'MMM d, yyyy HH:mm')}
                  </p>
                </div>
                <span className={`status-pill ${event.status}`}>{event.status.replace('_', ' ')}</span>
              </div>
            ))}
            {!events.length ? <div className="empty-state">No events created yet.</div> : null}
          </div>
        </article>

        <article className="section-card">
          <div className="section-header">
            <div>
              <p className="eyebrow">Background jobs</p>
              <h2>Recent exports</h2>
            </div>
          </div>

          <div className="list-stack">
            {exports.slice(0, 5).map((job) => (
              <div className="list-row" key={job.id}>
                <div>
                  <strong>{job.format.toUpperCase()} export</strong>
                  <p className="muted">{format(new Date(job.created_at), 'MMM d, yyyy HH:mm')}</p>
                </div>
                <span className={`status-pill ${job.status}`}>{job.status}</span>
              </div>
            ))}
            {!exports.length ? <div className="empty-state">No export jobs yet.</div> : null}
          </div>
        </article>
      </section>
    </div>
  )
}
