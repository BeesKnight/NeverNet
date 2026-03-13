import type { QueryClient } from '@tanstack/react-query'

const READ_SIDE_QUERY_PREFIXES = [
  ['categories'],
  ['events'],
  ['dashboard'],
  ['calendar-events'],
  ['reports'],
] as const

export async function invalidateReadSideQueries(queryClient: QueryClient) {
  await Promise.all(
    READ_SIDE_QUERY_PREFIXES.map((queryKey) =>
      queryClient.invalidateQueries({ queryKey: [...queryKey] }),
    ),
  )
}
