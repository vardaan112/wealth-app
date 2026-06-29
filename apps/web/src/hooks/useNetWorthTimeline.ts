import { useQuery } from 'urql'
import { NET_WORTH_TIMELINE_QUERY } from '../graphql/queries'
import type { NetWorthTimelineQuery } from '../graphql/types'

export function useNetWorthTimeline() {
  return useQuery<NetWorthTimelineQuery>({ query: NET_WORTH_TIMELINE_QUERY })
}
