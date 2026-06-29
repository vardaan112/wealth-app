import { useQuery } from 'urql'
import { ACCOUNTS_QUERY } from '../graphql/queries'
import type { AccountsQuery } from '../graphql/types'

export function useAccounts() {
  return useQuery<AccountsQuery>({ query: ACCOUNTS_QUERY })
}
