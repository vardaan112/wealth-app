import { useQuery } from 'urql'
import { TRANSACTIONS_QUERY } from '../graphql/queries'
import type {
  TransactionsQuery,
  TransactionsQueryVariables,
} from '../graphql/types'

export function useTransactions(month?: string | null) {
  return useQuery<TransactionsQuery, TransactionsQueryVariables>({
    query: TRANSACTIONS_QUERY,
    variables: month ? { month } : {},
  })
}
