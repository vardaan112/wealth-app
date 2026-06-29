import { useMutation } from 'urql'
import { CREATE_MANUAL_HOLDING_MUTATION } from '../graphql/queries'
import type {
  CreateManualHoldingMutation,
  ManualHoldingInput,
} from '../graphql/types'

export function useCreateManualHolding() {
  return useMutation<
    CreateManualHoldingMutation,
    { input: ManualHoldingInput }
  >(CREATE_MANUAL_HOLDING_MUTATION)
}
