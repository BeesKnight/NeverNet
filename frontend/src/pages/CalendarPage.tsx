import { useQuery } from '@tanstack/react-query'
import {
  addDays,
  addMonths,
  endOfMonth,
  endOfWeek,
  format,
  isSameDay,
  isSameMonth,
  startOfMonth,
  startOfWeek,
  subMonths,
} from 'date-fns'
import { useMemo, useState } from 'react'

import { apiRequest, buildQueryString } from '../api/client'
import type { Event } from '../api/types'
import { useAuth } from '../features/auth/auth-context'

export function CalendarPage() {
  const { session } = useAuth()
  const token = session?.token ?? ''
  const [currentMonth, setCurrentMonth] = useState(() => new Date())

  const monthStart = startOfMonth(currentMonth)
  const monthEnd = endOfMonth(currentMonth)

  const eventsQuery = useQuery({
    queryKey: ['calendar-events', format(monthStart, 'yyyy-MM')],
    queryFn: () =>
      apiRequest<Event[]>(
        `/events/${buildQueryString({
          start_date: format(monthStart, 'yyyy-MM-dd'),
          end_date: format(monthEnd, 'yyyy-MM-dd'),
        })}`,
        { token },
      ),
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

        <div className="calendar-grid">
          {['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'].map((day) => (
            <div className="calendar-head" key={day}>
              {day}
            </div>
          ))}

          {calendarDays.map((day) => {
            const dayEvents = events.filter((event) => isSameDay(new Date(event.starts_at), day))

            return (
              <article className={`calendar-cell${isSameMonth(day, currentMonth) ? '' : ' muted-cell'}`} key={day.toISOString()}>
                <header>
                  <strong>{format(day, 'd')}</strong>
                </header>
                <div className="calendar-events">
                  {dayEvents.slice(0, 3).map((event) => (
                    <div
                      className="calendar-badge"
                      key={event.id}
                      style={{ borderLeftColor: event.category_color }}
                    >
                      <span>{format(new Date(event.starts_at), 'HH:mm')}</span>
                      <strong>{event.title}</strong>
                    </div>
                  ))}
                </div>
              </article>
            )
          })}
        </div>
      </section>
    </div>
  )
}
