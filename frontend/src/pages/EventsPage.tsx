import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { format } from 'date-fns'
import { useState } from 'react'

import { apiRequest, buildQueryString } from '../api/client'
import { ErrorState, InlineNotice, LoadingState } from '../components/QueryState'
import type { Category, Event, EventFilters, EventRecord } from '../api/types'
import { useAuth } from '../features/auth/auth-context'
import { EventForm } from '../features/events/EventForm'

type EventFormPayload = {
  category_id: string
  title: string
  description: string
  location: string
  starts_at: string
  ends_at: string
  budget: number
  status: Event['status']
}

const DEFAULT_FILTERS: EventFilters = {
  search: '',
  status: '',
  category_id: '',
  start_date: '',
  end_date: '',
  sort_by: 'starts_at',
  sort_dir: 'asc',
}

export function EventsPage() {
  const { session } = useAuth()
  const [filters, setFilters] = useState<EventFilters>(DEFAULT_FILTERS)
  const [editingEvent, setEditingEvent] = useState<Event | null>(null)
  const queryClient = useQueryClient()

  const categoriesQuery = useQuery({
    queryKey: ['categories', session?.user.id, 'events'],
    queryFn: () => apiRequest<Category[]>('/categories'),
    enabled: Boolean(session?.user.id),
  })

  const eventsQuery = useQuery({
    queryKey: ['events', session?.user.id, filters],
    queryFn: () =>
      apiRequest<Event[]>(
        `/events${buildQueryString({
          search: filters.search || undefined,
          status: filters.status || undefined,
          category_id: filters.category_id || undefined,
          start_date: filters.start_date || undefined,
          end_date: filters.end_date || undefined,
          sort_by: filters.sort_by || undefined,
          sort_dir: filters.sort_dir || undefined,
        })}`,
      ),
    enabled: Boolean(session?.user.id),
  })

  const saveEvent = useMutation({
    mutationFn: async (values: EventFormPayload) => {
      if (editingEvent) {
        return apiRequest<EventRecord>(`/events/${editingEvent.id}`, {
          method: 'PATCH',
          body: JSON.stringify(values),
        })
      }

      return apiRequest<EventRecord>('/events', {
        method: 'POST',
        body: JSON.stringify(values),
      })
    },
    onSuccess: async () => {
      setEditingEvent(null)
      await queryClient.invalidateQueries({ queryKey: ['events'] })
    },
  })

  const deleteEvent = useMutation({
    mutationFn: (eventId: string) =>
      apiRequest(`/events/${eventId}`, {
        method: 'DELETE',
      }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['events'] })
    },
  })

  const categories = categoriesQuery.data ?? []
  const events = eventsQuery.data ?? []
  const hasLoadingState = (categoriesQuery.isPending || eventsQuery.isPending) && !categories.length && !events.length
  const hasErrorState = categoriesQuery.isError || eventsQuery.isError

  if (hasLoadingState) {
    return (
      <LoadingState
        title="Loading events"
        detail="Syncing categories, filters, and projection-backed event rows."
      />
    )
  }

  if (hasErrorState) {
    return (
      <ErrorState
        title="Events unavailable"
        detail="The event list could not be loaded from the query side."
        action={
          <button className="ghost-button" type="button" onClick={() => {
            void categoriesQuery.refetch()
            void eventsQuery.refetch()
          }}>
            Retry
          </button>
        }
      />
    )
  }

  return (
    <div className="page-shell">
      <section className="section-card">
        <div className="section-header">
          <div>
            <p className="eyebrow">Filters</p>
            <h2>Find events fast</h2>
          </div>
        </div>

        <div className="filter-grid">
          <label>
            <span>Search</span>
            <input
              value={filters.search ?? ''}
              onChange={(event) => setFilters((current) => ({ ...current, search: event.target.value }))}
              placeholder="Title, description, location"
            />
          </label>
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
          <label>
            <span>Sort by</span>
            <select
              value={filters.sort_by ?? 'starts_at'}
              onChange={(event) =>
                setFilters((current) => ({
                  ...current,
                  sort_by: event.target.value as NonNullable<EventFilters['sort_by']>,
                }))
              }
            >
              <option value="starts_at">Start time</option>
              <option value="ends_at">End time</option>
              <option value="title">Title</option>
              <option value="category_name">Category</option>
              <option value="budget">Budget</option>
              <option value="status">Status</option>
              <option value="updated_at">Last updated</option>
            </select>
          </label>
          <label>
            <span>Direction</span>
            <select
              value={filters.sort_dir ?? 'asc'}
              onChange={(event) =>
                setFilters((current) => ({
                  ...current,
                  sort_dir: event.target.value as NonNullable<EventFilters['sort_dir']>,
                }))
              }
            >
              <option value="asc">Ascending</option>
              <option value="desc">Descending</option>
            </select>
          </label>
        </div>
      </section>

      <div className="two-column-page">
        <section className="section-card">
          <div className="section-header">
            <div>
              <p className="eyebrow">Events</p>
              <h2>{editingEvent ? 'Edit event' : 'Create event'}</h2>
            </div>
          </div>

          <EventForm
            key={editingEvent?.id ?? `new-event-${categories[0]?.id ?? 'none'}`}
            categories={categories}
            event={editingEvent}
            isSubmitting={saveEvent.isPending}
            onCancel={() => setEditingEvent(null)}
            onSubmit={async (values) => {
              await saveEvent.mutateAsync(values)
            }}
          />
        </section>

        <section className="section-card">
          <div className="section-header">
            <div>
              <p className="eyebrow">List</p>
              <h2>Tracked events</h2>
            </div>
          </div>

          <InlineNotice>
            {events.length} result{events.length === 1 ? '' : 's'} sorted by{' '}
            {(filters.sort_by ?? 'starts_at').replace('_', ' ')} in {filters.sort_dir ?? 'asc'} order.
          </InlineNotice>

          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Event</th>
                  <th>Status</th>
                  <th>Schedule</th>
                  <th>Budget</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                {events.map((event) => (
                  <tr key={event.id}>
                    <td>
                      <strong>{event.title}</strong>
                      <p className="muted">{event.category_name}</p>
                    </td>
                    <td>
                      <span className={`status-pill ${event.status}`}>{event.status.replace('_', ' ')}</span>
                    </td>
                    <td>{format(new Date(event.starts_at), 'MMM d, HH:mm')}</td>
                    <td>${event.budget.toFixed(2)}</td>
                    <td className="actions-cell">
                      <button className="ghost-button" onClick={() => setEditingEvent(event)} type="button">
                        Edit
                      </button>
                      <button
                        className="ghost-button danger"
                        onClick={() => deleteEvent.mutate(event.id)}
                        type="button"
                      >
                        Delete
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>

            {!events.length ? <div className="empty-state">No matching events found.</div> : null}
          </div>
        </section>
      </div>
    </div>
  )
}
