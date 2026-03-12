import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { format } from 'date-fns'
import { useState } from 'react'

import { apiRequest } from '../api/client'
import type { Category } from '../api/types'
import { CategoryForm } from '../features/categories/CategoryForm'
import { useAuth } from '../features/auth/auth-context'

export function CategoriesPage() {
  const { session } = useAuth()
  const token = session?.token ?? ''
  const [editingCategory, setEditingCategory] = useState<Category | null>(null)
  const queryClient = useQueryClient()

  const categoriesQuery = useQuery({
    queryKey: ['categories'],
    queryFn: () => apiRequest<Category[]>('/categories/', { token }),
  })

  const saveCategory = useMutation({
    mutationFn: async (values: { name: string; color: string }) => {
      if (editingCategory) {
        return apiRequest<Category>(`/categories/${editingCategory.id}`, {
          method: 'PUT',
          token,
          body: JSON.stringify(values),
        })
      }

      return apiRequest<Category>('/categories/', {
        method: 'POST',
        token,
        body: JSON.stringify(values),
      })
    },
    onSuccess: async () => {
      setEditingCategory(null)
      await queryClient.invalidateQueries({ queryKey: ['categories'] })
    },
  })

  const deleteCategory = useMutation({
    mutationFn: (categoryId: string) =>
      apiRequest('/categories/' + categoryId, {
        method: 'DELETE',
        token,
      }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['categories'] })
    },
  })

  const categories = categoriesQuery.data ?? []

  return (
    <div className="page-shell two-column-page">
      <section className="section-card">
        <div className="section-header">
          <div>
            <p className="eyebrow">Categories</p>
            <h2>{editingCategory ? 'Edit category' : 'Add category'}</h2>
          </div>
        </div>

        <CategoryForm
          key={editingCategory?.id ?? 'new-category'}
          category={editingCategory}
          isSubmitting={saveCategory.isPending}
          onCancel={() => setEditingCategory(null)}
          onSubmit={async (values) => {
            await saveCategory.mutateAsync(values)
          }}
        />
      </section>

      <section className="section-card">
        <div className="section-header">
          <div>
            <p className="eyebrow">Library</p>
            <h2>Saved categories</h2>
          </div>
        </div>

        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>Color</th>
                <th>Created</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {categories.map((category) => (
                <tr key={category.id}>
                  <td>{category.name}</td>
                  <td>
                    <span className="color-chip" style={{ backgroundColor: category.color }} />
                    {category.color}
                  </td>
                  <td>{format(new Date(category.created_at), 'MMM d, yyyy')}</td>
                  <td className="actions-cell">
                    <button className="ghost-button" onClick={() => setEditingCategory(category)} type="button">
                      Edit
                    </button>
                    <button
                      className="ghost-button danger"
                      onClick={() => deleteCategory.mutate(category.id)}
                      type="button"
                    >
                      Delete
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>

          {!categories.length ? <div className="empty-state">No categories yet.</div> : null}
        </div>
      </section>
    </div>
  )
}
