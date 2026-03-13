import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { format } from 'date-fns'
import { useState } from 'react'

import { apiDownload, apiRequest, buildQueryString } from '../api/client'
import type { Category, EventFilters, ExportJob, ReportSummary } from '../api/types'
import { useAuth } from '../features/auth/auth-context'

const DEFAULT_FILTERS: EventFilters = {
  status: '',
  category_id: '',
  start_date: '',
  end_date: '',
}

export function ReportsPage() {
  const { session } = useAuth()
  const [filters, setFilters] = useState<EventFilters>(DEFAULT_FILTERS)
  const queryClient = useQueryClient()

  const categoriesQuery = useQuery({
    queryKey: ['categories', session?.user.id, 'reports'],
    queryFn: () => apiRequest<Category[]>('/categories'),
    enabled: Boolean(session?.user.id),
  })

  const reportQuery = useQuery({
    queryKey: ['reports', session?.user.id, filters],
    queryFn: () =>
      apiRequest<ReportSummary>(
        `/reports/summary${buildQueryString({
          status: filters.status || undefined,
          category_id: filters.category_id || undefined,
          start_date: filters.start_date || undefined,
          end_date: filters.end_date || undefined,
        })}`,
      ),
    enabled: Boolean(session?.user.id),
  })

  const exportsQuery = useQuery({
    queryKey: ['exports', session?.user.id],
    queryFn: () => apiRequest<ExportJob[]>('/exports'),
    enabled: Boolean(session?.user.id),
    refetchInterval: (query) => {
      const jobs = query.state.data ?? []
      return jobs.some((job) => job.status === 'queued' || job.status === 'processing')
        ? 3_000
        : false
    },
  })

  const createExport = useMutation({
    mutationFn: (formatName: 'pdf' | 'xlsx') =>
      apiRequest<ExportJob>('/exports', {
        method: 'POST',
        body: JSON.stringify({
          report_type: 'summary',
          format: formatName,
          filters,
        }),
      }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['exports'] })
    },
  })

  const report = reportQuery.data
  const categories = categoriesQuery.data ?? []
  const exportJobs = exportsQuery.data ?? []

  return (
    <div className="page-shell">
      <section className="section-card">
        <div className="section-header">
          <div>
            <p className="eyebrow">Reports</p>
            <h2>Aggregate by period and category</h2>
          </div>

          <div className="form-actions">
            <button className="primary-button" type="button" onClick={() => createExport.mutate('pdf')}>
              Export PDF
            </button>
            <button className="ghost-button" type="button" onClick={() => createExport.mutate('xlsx')}>
              Export XLSX
            </button>
          </div>
        </div>

        <div className="filter-grid">
          <label>
            <span>Status</span>
            <select
              value={filters.status ?? ''}
              onChange={(event) => setFilters((current) => ({ ...current, status: event.target.value }))}
            >
              <option value="">All</option>
              <option value="planned">Planned</option>
              <option value="in_progress">In progress</option>
              <option value="completed">Completed</option>
              <option value="cancelled">Cancelled</option>
            </select>
          </label>

          <label>
            <span>Category</span>
            <select
              value={filters.category_id ?? ''}
              onChange={(event) =>
                setFilters((current) => ({ ...current, category_id: event.target.value }))
              }
            >
              <option value="">All</option>
              {categories.map((category) => (
                <option key={category.id} value={category.id}>
                  {category.name}
                </option>
              ))}
            </select>
          </label>

          <label>
            <span>Start date</span>
            <input
              type="date"
              value={filters.start_date ?? ''}
              onChange={(event) =>
                setFilters((current) => ({ ...current, start_date: event.target.value }))
              }
            />
          </label>

          <label>
            <span>End date</span>
            <input
              type="date"
              value={filters.end_date ?? ''}
              onChange={(event) =>
                setFilters((current) => ({ ...current, end_date: event.target.value }))
              }
            />
          </label>
        </div>
      </section>

      <section className="stats-grid">
        <article className="stat-card">
          <p className="eyebrow">Events</p>
          <strong>{report?.total_events ?? 0}</strong>
          <span>Included in the current report</span>
        </article>
        <article className="stat-card">
          <p className="eyebrow">Budget</p>
          <strong>${(report?.total_budget ?? 0).toFixed(2)}</strong>
          <span>Total budget across matching events</span>
        </article>
      </section>

      <div className="two-column-page">
        <section className="section-card">
          <div className="section-header">
            <div>
              <p className="eyebrow">Categories</p>
              <h2>Budget by category</h2>
            </div>
          </div>

          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Category</th>
                  <th>Events</th>
                  <th>Budget</th>
                </tr>
              </thead>
              <tbody>
                {report?.by_category.map((row) => (
                  <tr key={row.category_id}>
                    <td>
                      <span className="color-chip" style={{ backgroundColor: row.category_color }} />
                      {row.category_name}
                    </td>
                    <td>{row.event_count}</td>
                    <td>${row.total_budget.toFixed(2)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
            {!report?.by_category.length ? <div className="empty-state">No category data.</div> : null}
          </div>
        </section>

        <section className="section-card">
          <div className="section-header">
            <div>
              <p className="eyebrow">Statuses</p>
              <h2>Execution split</h2>
            </div>
          </div>

          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Status</th>
                  <th>Events</th>
                  <th>Budget</th>
                </tr>
              </thead>
              <tbody>
                {report?.by_status.map((row) => (
                  <tr key={row.status}>
                    <td>{row.status.replace('_', ' ')}</td>
                    <td>{row.event_count}</td>
                    <td>${row.total_budget.toFixed(2)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
            {!report?.by_status.length ? <div className="empty-state">No status data.</div> : null}
          </div>
        </section>
      </div>

      <section className="section-card">
        <div className="section-header">
          <div>
            <p className="eyebrow">Events</p>
            <h2>Detailed report scope</h2>
          </div>
        </div>

        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Event</th>
                <th>Category</th>
                <th>Schedule</th>
                <th>Location</th>
                <th>Budget</th>
                <th>Status</th>
              </tr>
            </thead>
            <tbody>
              {report?.events.map((event) => (
                <tr key={event.id}>
                  <td>{event.title}</td>
                  <td>{event.category_name}</td>
                  <td>
                    {format(new Date(event.starts_at), 'MMM d, yyyy HH:mm')}
                    <br />
                    <span className="muted">to {format(new Date(event.ends_at), 'MMM d, yyyy HH:mm')}</span>
                  </td>
                  <td>{event.location || 'Not specified'}</td>
                  <td>${event.budget.toFixed(2)}</td>
                  <td>
                    <span className={`status-pill ${event.status}`}>{event.status.replace('_', ' ')}</span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {!report?.events.length ? <div className="empty-state">No events in this report scope.</div> : null}
        </div>
      </section>

      <section className="section-card">
        <div className="section-header">
          <div>
            <p className="eyebrow">Exports</p>
            <h2>Background job history</h2>
          </div>
        </div>

        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Report</th>
                <th>Format</th>
                <th>Status</th>
                <th>Created</th>
                <th>Finished</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {exportJobs.map((job) => (
                <tr key={job.id}>
                  <td>{job.report_type}</td>
                  <td>{job.format.toUpperCase()}</td>
                  <td>
                    <span className={`status-pill ${job.status}`}>{job.status}</span>
                  </td>
                  <td>{format(new Date(job.created_at), 'MMM d, yyyy HH:mm')}</td>
                  <td>{job.finished_at ? format(new Date(job.finished_at), 'MMM d, yyyy HH:mm') : 'In progress'}</td>
                  <td className="actions-cell">
                    {job.status === 'completed' ? (
                      <button
                        className="ghost-button"
                        type="button"
                        onClick={() => apiDownload(`/exports/${job.id}/download`, `eventdesign-report-${job.id}.${job.format}`)}
                      >
                        Download
                      </button>
                    ) : (
                      <span className="muted">{job.error_message ?? 'Waiting for background worker'}</span>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {!exportJobs.length ? <div className="empty-state">No export jobs yet.</div> : null}
        </div>
      </section>
    </div>
  )
}
