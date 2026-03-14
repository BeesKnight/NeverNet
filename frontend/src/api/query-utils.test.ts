import { describe, expect, it, vi } from 'vitest'

import { invalidateReadSideQueries } from './query-utils'

describe('invalidateReadSideQueries', () => {
  it('invalidates every read-side query prefix', async () => {
    const queryClient = {
      invalidateQueries: vi.fn().mockResolvedValue(undefined),
    }

    await invalidateReadSideQueries(queryClient as never)

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(5)
    expect(queryClient.invalidateQueries).toHaveBeenNthCalledWith(1, {
      queryKey: ['categories'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenNthCalledWith(2, {
      queryKey: ['events'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenNthCalledWith(3, {
      queryKey: ['dashboard'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenNthCalledWith(4, {
      queryKey: ['calendar-events'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenNthCalledWith(5, {
      queryKey: ['reports'],
    })
  })
})
