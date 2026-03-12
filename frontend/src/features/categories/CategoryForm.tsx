import { useState } from 'react'

import type { Category } from '../../api/types'

type CategoryFormValues = {
  name: string
  color: string
}

type CategoryFormProps = {
  category?: Category | null
  isSubmitting: boolean
  onCancel: () => void
  onSubmit: (values: CategoryFormValues) => Promise<void>
}

const DEFAULT_VALUES: CategoryFormValues = {
  name: '',
  color: '#0f766e',
}

function initialValues(category?: Category | null): CategoryFormValues {
  if (!category) {
    return DEFAULT_VALUES
  }

  return {
    name: category.name,
    color: category.color,
  }
}

export function CategoryForm({
  category,
  isSubmitting,
  onCancel,
  onSubmit,
}: CategoryFormProps) {
  const [values, setValues] = useState<CategoryFormValues>(() => initialValues(category))

  return (
    <form
      className="form-grid"
      onSubmit={async (event) => {
        event.preventDefault()
        await onSubmit(values)
        if (!category) {
          setValues(DEFAULT_VALUES)
        }
      }}
    >
      <label>
        <span>Category name</span>
        <input
          value={values.name}
          onChange={(event) =>
            setValues((current) => ({ ...current, name: event.target.value }))
          }
          placeholder="Conference"
          required
        />
      </label>

      <label>
        <span>Color</span>
        <input
          type="color"
          value={values.color}
          onChange={(event) =>
            setValues((current) => ({ ...current, color: event.target.value }))
          }
          required
        />
      </label>

      <div className="form-actions">
        <button className="primary-button" disabled={isSubmitting} type="submit">
          {category ? 'Save category' : 'Create category'}
        </button>
        {category ? (
          <button className="ghost-button" type="button" onClick={onCancel}>
            Cancel
          </button>
        ) : null}
      </div>
    </form>
  )
}
