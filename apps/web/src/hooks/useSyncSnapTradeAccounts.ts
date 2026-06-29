import { useMutation } from 'urql'
import { SYNC_SNAPTRADE_ACCOUNTS_MUTATION } from '../graphql/queries'
import type { SyncSnapTradeAccountsMutation } from '../graphql/types'

export function useSyncSnapTradeAccounts() {
  return useMutation<SyncSnapTradeAccountsMutation, Record<string, never>>(
    SYNC_SNAPTRADE_ACCOUNTS_MUTATION,
  )
}
