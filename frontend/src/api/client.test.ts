import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
  apiDownload,
  apiRequest,
  buildQueryString,
  clearCsrfToken,
  refreshCsrfToken,
} from './client'

describe('api client helpers', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
    clearCsrfToken()
  })

  it('caches csrf token until it is explicitly refreshed', async () => {
    const fetchMock = vi.fn()
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          data: {
            csrf_token: 'token-1',
          },
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          data: {
            csrf_token: 'token-2',
          },
        }),
      })
    vi.stubGlobal('fetch', fetchMock)

    await expect(refreshCsrfToken()).resolves.toBe('token-1')
    await expect(refreshCsrfToken()).resolves.toBe('token-1')
    await expect(refreshCsrfToken(true)).resolves.toBe('token-2')

    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(fetchMock).toHaveBeenNthCalledWith(
      1,
      '/api/auth/csrf',
      expect.objectContaining({
        credentials: 'include',
      }),
    )
  })

  it('adds csrf and content type headers for unsafe requests', async () => {
    const fetchMock = vi.fn()
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          data: {
            csrf_token: 'csrf-token',
          },
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          data: {
            id: 'event-1',
          },
        }),
      })
    vi.stubGlobal('fetch', fetchMock)

    await expect(
      apiRequest<{ id: string }>('/events', {
        method: 'POST',
        body: JSON.stringify({ title: 'Defense rehearsal' }),
      }),
    ).resolves.toEqual({ id: 'event-1' })

    const requestOptions = fetchMock.mock.calls[1]?.[1] as RequestInit
    const headers = requestOptions.headers as Headers

    expect(requestOptions.credentials).toBe('include')
    expect(headers.get('Accept')).toBe('application/json')
    expect(headers.get('Content-Type')).toBe('application/json')
    expect(headers.get('X-CSRF-Token')).toBe('csrf-token')
  })

  it('surfaces structured api errors with request id fallback', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      status: 409,
      json: async () => ({
        error: {
          code: 'conflict',
          message: 'Category already exists',
          request_id: 'req-1',
        },
      }),
      headers: new Headers(),
    })
    vi.stubGlobal('fetch', fetchMock)

    await expect(apiRequest('/categories')).rejects.toEqual(
      expect.objectContaining({
        name: 'Error',
        message: 'Category already exists',
        status: 409,
        code: 'conflict',
        requestId: 'req-1',
      }),
    )
  })

  it('downloads binary responses through a temporary anchor element', async () => {
    const click = vi.fn()
    const appendChild = vi.spyOn(document.body, 'appendChild')
    const createObjectUrl = vi.fn(() => 'blob:download')
    const revokeObjectUrl = vi.fn()

    vi.spyOn(document, 'createElement').mockImplementation((tagName: string) => {
      if (tagName === 'a') {
        const anchor = document.createElementNS('http://www.w3.org/1999/xhtml', 'a')
        anchor.click = click
        return anchor
      }

      return document.createElementNS('http://www.w3.org/1999/xhtml', tagName)
    })
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      blob: async () => new Blob(['pdf']),
    }))
    vi.stubGlobal('URL', {
      createObjectURL: createObjectUrl,
      revokeObjectURL: revokeObjectUrl,
    })

    await apiDownload('/exports/export-1/download', 'report.pdf')

    expect(createObjectUrl).toHaveBeenCalledTimes(1)
    expect(appendChild).toHaveBeenCalledTimes(1)
    expect(click).toHaveBeenCalledTimes(1)
    expect(revokeObjectUrl).toHaveBeenCalledWith('blob:download')
  })

  it('builds query strings from defined values only', () => {
    expect(
      buildQueryString({
        search: 'conference',
        status: '',
        category_id: undefined,
        sort_by: 'starts_at',
      }),
    ).toBe('?search=conference&sort_by=starts_at')
    expect(buildQueryString({})).toBe('')
  })
})
