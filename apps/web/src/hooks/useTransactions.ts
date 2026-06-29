import { useQuery } from 'urql'
import { TRANSACTIONS_QUERY } from '../graphql/queries'
import type { TransactionsQuery } from '../graphql/types'

export function useTransactions() {
  return useQuery<TransactionsQuery>({ query: TRANSACTIONS_QUERY })
}
