import { fireEvent, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'

import { CategoryForm } from './CategoryForm'
import { renderWithProviders } from '../../test/test-utils'

describe('CategoryForm', () => {
  it('submits new categories and resets to defaults', async () => {
    const user = userEvent.setup()
    const onSubmit = vi.fn().mockResolvedValue(undefined)

    renderWithProviders(
      <CategoryForm
        isSubmitting={false}
        onCancel={vi.fn()}
        onSubmit={onSubmit}
      />,
    )

    await user.type(screen.getByLabelText('Category name'), 'Conference')
    fireEvent.input(screen.getByDisplayValue('#0f766e'), {
      target: { value: '#123456' },
    })
    await user.click(screen.getByRole('button', { name: 'Create category' }))

    expect(onSubmit).toHaveBeenCalledWith({
      name: 'Conference',
      color: '#123456',
    })
    expect(screen.getByLabelText('Category name')).toHaveValue('')
    expect(screen.getByLabelText('Color')).toHaveValue('#0f766e')
  })

  it('keeps edit values and exposes cancel action', async () => {
    const user = userEvent.setup()
    const onCancel = vi.fn()
    const onSubmit = vi.fn().mockResolvedValue(undefined)

    renderWithProviders(
      <CategoryForm
        category={{
          id: 'category-1',
          user_id: 'user-1',
          name: 'Planning',
          color: '#ff6600',
          created_at: '2026-03-13T10:00:00Z',
          updated_at: '2026-03-13T10:00:00Z',
        }}
        isSubmitting={false}
        onCancel={onCancel}
        onSubmit={onSubmit}
      />,
    )

    await user.clear(screen.getByLabelText('Category name'))
    await user.type(screen.getByLabelText('Category name'), 'Delivery')
    await user.click(screen.getByRole('button', { name: 'Save category' }))
    await user.click(screen.getByRole('button', { name: 'Cancel' }))

    expect(onSubmit).toHaveBeenCalledWith({
      name: 'Delivery',
      color: '#ff6600',
    })
    expect(onCancel).toHaveBeenCalledTimes(1)
  })
})
