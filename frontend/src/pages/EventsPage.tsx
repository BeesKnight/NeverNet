import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { format } from 'date-fns'
import { useState } from 'react'

import { apiRequest, buildQueryString } from '../api/client'
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
}

export function EventsPage() {
  const { session } = useAuth()
  const token = session?.token ?? ''
  const [filters, setFilters] = useState<EventFilters>(DEFAULT_FILTERS)
  const [editingEvent, setEditingEvent] = useState<Event | null>(null)
  const queryClient = useQueryClient()

  const categoriesQuery = useQuery({
    queryKey: ['categories', 'events'],
    queryFn: () => apiRequest<Category[]>('/categories/', { token }),
  })

  const eventsQuery = useQuery({
    queryKey: ['events', filters],
    queryFn: () =>
      apiRequest<Event[]>(
        `/events/${buildQueryString({
          search: filters.search || undefined,
          status: filters.status || undefined,
          category_id: filters.category_id || undefined,
          start_date: filters.start_date || undefined,
          end_date: filters.end_date || undefined,
        })}`,
        { token },
      ),
  })

  const saveEvent = useMutation({
    mutationFn: async (values: EventFormPayload) => {
      if (editingEvent) {
        return apiRequest<EventRecord>(`/events/${editingEvent.id}`, {
          method: 'PUT',
          token,
          body: JSON.stringify(values),
        })
      }

      return apiRequest<EventRecord>('/events/', {
        method: 'POST',
        token,
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
        token,
      }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['events'] })
    },
  })

  const categories = categoriesQuery.data ?? []
  const events = eventsQuery.data ?? []

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
