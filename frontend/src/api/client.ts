import type { ApiResponse, CsrfTokenResponse } from './types'

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? '/api'
let csrfToken: string | null = null
let pendingCsrfToken: Promise<string> | null = null

export class ApiError extends Error {
  status: number

  constructor(message: string, status: number) {
    super(message)
    this.status = status
  }
}

type RequestOptions = RequestInit

function isSafeMethod(method?: string) {
  return !method || ['GET', 'HEAD', 'OPTIONS'].includes(method.toUpperCase())
}

export async function refreshCsrfToken(force = false): Promise<string> {
  if (!force && csrfToken) {
    return csrfToken
  }

  if (!force && pendingCsrfToken) {
    return pendingCsrfToken
  }

  pendingCsrfToken = fetch(`${API_BASE_URL}/auth/csrf`, {
    credentials: 'include',
    headers: {
      Accept: 'application/json',
    },
  })
    .then(async (response) => {
      if (!response.ok) {
        throw new ApiError('Could not refresh CSRF token', response.status)
      }

      const payload = (await response.json()) as ApiResponse<CsrfTokenResponse>
      csrfToken = payload.data.csrf_token
      return csrfToken
    })
    .finally(() => {
      pendingCsrfToken = null
    })

  return pendingCsrfToken
}

export function clearCsrfToken() {
  csrfToken = null
}

export async function apiRequest<T>(
  path: string,
  options: RequestOptions = {},
): Promise<T> {
  const headers = new Headers(options.headers)
  headers.set('Accept', 'application/json')

  if (!isSafeMethod(options.method)) {
    headers.set('X-CSRF-Token', await refreshCsrfToken())
  }

  if (options.body && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json')
  }

  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...options,
    credentials: options.credentials ?? 'include',
    headers,
  })

  if (!response.ok) {
    const fallback = 'Request failed'
    const payload = await response.json().catch(() => null)
    throw new ApiError(payload?.error?.message ?? fallback, response.status)
  }

  const payload = (await response.json()) as ApiResponse<T>
  return payload.data
}

export async function apiDownload(path: string, fileName: string) {
  const response = await fetch(`${API_BASE_URL}${path}`, {
    credentials: 'include',
  })

  if (!response.ok) {
    const payload = await response.json().catch(() => null)
    throw new ApiError(payload?.error?.message ?? 'Download failed', response.status)
  }

  const blob = await response.blob()
  const url = window.URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = fileName
  document.body.appendChild(link)
  link.click()
  link.remove()
  window.URL.revokeObjectURL(url)
}

export function buildQueryString(params: Record<string, string | undefined>) {
  const searchParams = new URLSearchParams()

  Object.entries(params).forEach(([key, value]) => {
    if (value) {
      searchParams.set(key, value)
    }
  })

  const query = searchParams.toString()
  return query ? `?${query}` : ''
}
