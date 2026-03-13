import { useQuery } from '@tanstack/react-query'
import {
  addDays,
  addMonths,
  endOfMonth,
  endOfWeek,
  format,
  isSameMonth,
  startOfMonth,
  startOfWeek,
  subMonths,
} from 'date-fns'
import { useMemo, useState } from 'react'

import { apiRequest, buildQueryString } from '../api/client'
import { ErrorState, InlineNotice, LoadingState } from '../components/QueryState'
import type { CalendarItem } from '../api/types'
import { useAuth } from '../features/auth/auth-context'

export function CalendarPage() {
  const { session } = useAuth()
  const [currentMonth, setCurrentMonth] = useState(() => new Date())

  const monthStart = startOfMonth(currentMonth)
  const monthEnd = endOfMonth(currentMonth)

  const eventsQuery = useQuery({
    queryKey: ['calendar-events', session?.user.id, format(monthStart, 'yyyy-MM')],
    queryFn: () =>
      apiRequest<CalendarItem[]>(
        `/calendar${buildQueryString({
          start_date: format(monthStart, 'yyyy-MM-dd'),
          end_date: format(monthEnd, 'yyyy-MM-dd'),
        })}`,
      ),
    enabled: Boolean(session?.user.id),
  })

  const calendarDays = useMemo(() => {
    const start = startOfWeek(monthStart, { weekStartsOn: 1 })
    const end = endOfWeek(monthEnd, { weekStartsOn: 1 })
    const days: Date[] = []
    let cursor = start

    while (cursor <= end) {
      days.push(cursor)
      cursor = addDays(cursor, 1)
    }

    return days
  }, [monthEnd, monthStart])

  const events = eventsQuery.data ?? []

  if (eventsQuery.isPending && !events.length) {
    return (
      <LoadingState
        title="Loading calendar"
        detail="Reading the month projection and laying events onto the current grid."
      />
    )
  }

  if (eventsQuery.isError) {
    return (
      <ErrorState
        title="Calendar unavailable"
        detail="The month view could not be loaded from the calendar projection."
        action={
          <button className="ghost-button" type="button" onClick={() => void eventsQuery.refetch()}>
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
            <p className="eyebrow">Calendar</p>
            <h2>{format(currentMonth, 'MMMM yyyy')}</h2>
          </div>

          <div className="form-actions">
            <button className="ghost-button" type="button" onClick={() => setCurrentMonth(subMonths(currentMonth, 1))}>
              Previous
            </button>
            <button className="ghost-button" type="button" onClick={() => setCurrentMonth(new Date())}>
              Today
            </button>
            <button className="ghost-button" type="button" onClick={() => setCurrentMonth(addMonths(currentMonth, 1))}>
              Next
            </button>
          </div>
        </div>

        <InlineNotice>
          {events.length} event{events.length === 1 ? '' : 's'} scheduled between{' '}
          {format(monthStart, 'MMM d')} and {format(monthEnd, 'MMM d')}.
        </InlineNotice>

        <div className="calendar-grid">
          {['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'].map((day) => (
            <div className="calendar-head" key={day}>
              {day}
            </div>
          ))}

          {calendarDays.map((day) => {
            const dayEvents = events.filter((event) => event.date === format(day, 'yyyy-MM-dd'))
            const isToday = format(day, 'yyyy-MM-dd') === format(new Date(), 'yyyy-MM-dd')

            return (
              <article
                className={`calendar-cell${isSameMonth(day, currentMonth) ? '' : ' muted-cell'}${isToday ? ' current-day' : ''}`}
                key={day.toISOString()}
              >
                <header>
                  <strong>{format(day, 'd')}</strong>
                </header>
                <div className="calendar-events">
                  {dayEvents.slice(0, 3).map((event) => (
                    <div
                      className="calendar-badge"
                      key={event.event_id}
                      style={{ borderLeftColor: event.category_color }}
                    >
                      <span>{format(new Date(event.starts_at), 'HH:mm')}</span>
                      <strong>{event.title}</strong>
                    </div>
                  ))}
                  {dayEvents.length > 3 ? (
                    <div className="calendar-overflow">+{dayEvents.length - 3} more</div>
                  ) : null}
                </div>
              </article>
            )
          })}
        </div>

        {!events.length ? <div className="empty-state">No events fall inside this month yet.</div> : null}
      </section>
    </div>
  )
}
