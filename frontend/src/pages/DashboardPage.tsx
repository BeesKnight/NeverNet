import { useQuery } from '@tanstack/react-query'
import { format } from 'date-fns'

import { apiRequest } from '../api/client'
import { ErrorState, InlineNotice, LoadingState } from '../components/QueryState'
import type { DashboardResponse, ExportJob } from '../api/types'
import { useAuth } from '../features/auth/auth-context'

export function DashboardPage() {
  const { session } = useAuth()

  const dashboardQuery = useQuery({
    queryKey: ['dashboard', session?.user.id],
    queryFn: () => apiRequest<DashboardResponse>('/dashboard'),
    enabled: Boolean(session?.user.id),
  })

  const exportsQuery = useQuery({
    queryKey: ['exports', session?.user.id, 'dashboard'],
    queryFn: () => apiRequest<ExportJob[]>('/exports'),
    enabled: Boolean(session?.user.id),
  })

  const dashboard = dashboardQuery.data
  const exports = exportsQuery.data ?? []
  const upcomingEvents = dashboard?.upcoming ?? []
  const recentActivity = dashboard?.recent_activity ?? []
  const cards = dashboard?.cards
  const queuedExports = exports.filter((job) => job.status === 'queued').length
  const processingExports = exports.filter((job) => job.status === 'processing').length

  if ((dashboardQuery.isPending || exportsQuery.isPending) && !dashboard) {
    return (
      <LoadingState
        title="Loading dashboard"
        detail="Reading cached summary cards, upcoming events, and export job health."
      />
    )
  }

  if (dashboardQuery.isError || exportsQuery.isError) {
    return (
      <ErrorState
        title="Dashboard unavailable"
        detail="The dashboard summary could not be loaded from the query side."
        action={
          <button
            className="ghost-button"
            type="button"
            onClick={() => {
              void dashboardQuery.refetch()
              void exportsQuery.refetch()
            }}
          >
            Retry
          </button>
        }
      />
    )
  }

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
          <strong>{cards?.total_events ?? 0}</strong>
          <span>Tracked across all statuses</span>
        </article>
        <article className="stat-card">
          <p className="eyebrow">Upcoming</p>
          <strong>{cards?.upcoming_events ?? 0}</strong>
          <span>Future events still on the board</span>
        </article>
        <article className="stat-card">
          <p className="eyebrow">Completed</p>
          <strong>{cards?.completed_events ?? 0}</strong>
          <span>Events marked as delivered</span>
        </article>
        <article className="stat-card">
          <p className="eyebrow">Cancelled</p>
          <strong>{cards?.cancelled_events ?? 0}</strong>
          <span>Events removed from execution</span>
        </article>
        <article className="stat-card">
          <p className="eyebrow">Budget tracked</p>
          <strong>${(cards?.total_budget ?? 0).toFixed(0)}</strong>
          <span>Budget attached to all events in scope</span>
        </article>
      </section>

      <InlineNotice tone={queuedExports || processingExports ? 'success' : 'neutral'}>
        {queuedExports + processingExports
          ? `${queuedExports} queued and ${processingExports} processing export jobs are visible from the background pipeline.`
          : 'The export queue is currently idle.'}
      </InlineNotice>

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
                    {event.action.replace('_', ' ')} | {format(new Date(event.occurred_at), 'MMM d, yyyy HH:mm')}
                  </p>
                </div>
                <span className="status-pill planned">{event.entity_type}</span>
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
