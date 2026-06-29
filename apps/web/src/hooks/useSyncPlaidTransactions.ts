import { useMutation } from 'urql'
import { SYNC_PLAID_TRANSACTIONS_MUTATION } from '../graphql/queries'
import type { SyncPlaidTransactionsMutation } from '../graphql/types'

export function useSyncPlaidTransactions() {
  return useMutation<SyncPlaidTransactionsMutation, Record<string, never>>(
    SYNC_PLAID_TRANSACTIONS_MUTATION,
  )
}
