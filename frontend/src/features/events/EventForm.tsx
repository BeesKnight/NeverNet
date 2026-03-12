import { useState } from 'react'

import type { Category, Event, EventRecord, EventStatus } from '../../api/types'

type EventFormValues = {
  category_id: string
  title: string
  description: string
  location: string
  starts_at: string
  ends_at: string
  budget: string
  status: EventStatus
}

type EventFormProps = {
  categories: Category[]
  event?: Event | EventRecord | null
  isSubmitting: boolean
  onCancel: () => void
  onSubmit: (values: {
    category_id: string
    title: string
    description: string
    location: string
    starts_at: string
    ends_at: string
    budget: number
    status: EventStatus
  }) => Promise<void>
}

const STATUSES: EventStatus[] = ['planned', 'in_progress', 'completed', 'cancelled']

function emptyValues(categories: Category[]): EventFormValues {
  const now = new Date()
  const later = new Date(now.getTime() + 60 * 60 * 1000)

  return {
    category_id: categories[0]?.id ?? '',
    title: '',
    description: '',
    location: '',
    starts_at: toDatetimeInputValue(now.toISOString()),
    ends_at: toDatetimeInputValue(later.toISOString()),
    budget: '0',
    status: 'planned',
  }
}

function toDatetimeInputValue(value: string) {
  return value.slice(0, 16)
}

function initialValues(
  categories: Category[],
  event?: Event | EventRecord | null,
): EventFormValues {
  if (!event) {
    return emptyValues(categories)
  }

  return {
    category_id: event.category_id,
    title: event.title,
    description: event.description,
    location: event.location,
    starts_at: toDatetimeInputValue(event.starts_at),
    ends_at: toDatetimeInputValue(event.ends_at),
    budget: String(event.budget),
    status: event.status,
  }
}

export function EventForm({
  categories,
  event,
  isSubmitting,
  onCancel,
  onSubmit,
}: EventFormProps) {
  const [values, setValues] = useState<EventFormValues>(() => initialValues(categories, event))

  return (
    <form
      className="form-grid"
      onSubmit={async (submitEvent) => {
        submitEvent.preventDefault()
        await onSubmit({
          ...values,
          budget: Number(values.budget),
          starts_at: new Date(values.starts_at).toISOString(),
          ends_at: new Date(values.ends_at).toISOString(),
        })
        if (!event) {
          setValues(emptyValues(categories))
        }
      }}
    >
      <label>
        <span>Title</span>
        <input
          value={values.title}
          onChange={(inputEvent) =>
            setValues((current) => ({ ...current, title: inputEvent.target.value }))
          }
          placeholder="Launch event"
          required
        />
      </label>

      <label>
        <span>Category</span>
        <select
          value={values.category_id}
          onChange={(inputEvent) =>
            setValues((current) => ({ ...current, category_id: inputEvent.target.value }))
          }
          required
        >
          {categories.map((category) => (
            <option key={category.id} value={category.id}>
              {category.name}
            </option>
          ))}
        </select>
      </label>

      <label>
        <span>Location</span>
        <input
          value={values.location}
          onChange={(inputEvent) =>
            setValues((current) => ({ ...current, location: inputEvent.target.value }))
          }
          placeholder="Main hall"
          required
        />
      </label>

      <label>
        <span>Status</span>
        <select
          value={values.status}
          onChange={(inputEvent) =>
            setValues((current) => ({
              ...current,
              status: inputEvent.target.value as EventStatus,
            }))
          }
          required
        >
          {STATUSES.map((status) => (
            <option key={status} value={status}>
              {status.replace('_', ' ')}
            </option>
          ))}
        </select>
      </label>

      <label>
        <span>Starts at</span>
        <input
          type="datetime-local"
          value={values.starts_at}
          onChange={(inputEvent) =>
            setValues((current) => ({ ...current, starts_at: inputEvent.target.value }))
          }
          required
        />
      </label>

      <label>
        <span>Ends at</span>
        <input
          type="datetime-local"
          value={values.ends_at}
          onChange={(inputEvent) =>
            setValues((current) => ({ ...current, ends_at: inputEvent.target.value }))
          }
          required
        />
      </label>

      <label className="full-width">
        <span>Description</span>
        <textarea
          rows={4}
          value={values.description}
          onChange={(inputEvent) =>
            setValues((current) => ({ ...current, description: inputEvent.target.value }))
          }
          placeholder="Agenda, vendors, notes, and attendee details."
        />
      </label>

      <label>
        <span>Budget</span>
        <input
          type="number"
          min="0"
          step="0.01"
          value={values.budget}
          onChange={(inputEvent) =>
            setValues((current) => ({ ...current, budget: inputEvent.target.value }))
          }
          required
        />
      </label>

      <div className="form-actions">
        <button className="primary-button" disabled={isSubmitting || !categories.length} type="submit">
          {event ? 'Save event' : 'Create event'}
        </button>
        {event ? (
          <button className="ghost-button" type="button" onClick={onCancel}>
            Cancel
          </button>
        ) : null}
      </div>
    </form>
  )
}
