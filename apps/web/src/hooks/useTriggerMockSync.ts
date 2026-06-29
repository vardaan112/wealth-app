import { useMutation } from 'urql'
import { TRIGGER_MOCK_SYNC_MUTATION } from '../graphql/queries'
import type { TriggerMockSyncMutation } from '../graphql/types'

export function useTriggerMockSync() {
  return useMutation<TriggerMockSyncMutation, Record<string, never>>(
    TRIGGER_MOCK_SYNC_MUTATION,
  )
}
