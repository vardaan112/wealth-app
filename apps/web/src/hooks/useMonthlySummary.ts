import { useQuery } from 'urql'
import { MONTHLY_SUMMARY_QUERY } from '../graphql/queries'
import type { MonthlySummaryQuery } from '../graphql/types'
import { currentMonth } from '../lib/format'

export function useMonthlySummary(month: string = currentMonth()) {
  return useQuery<MonthlySummaryQuery>({
    query: MONTHLY_SUMMARY_QUERY,
    variables: { month },
  })
}
