import { useQuery } from 'urql'
import { HOLDINGS_QUERY } from '../graphql/queries'
import type { HoldingsQuery } from '../graphql/types'

export function useHoldings() {
  return useQuery<HoldingsQuery>({ query: HOLDINGS_QUERY })
}
