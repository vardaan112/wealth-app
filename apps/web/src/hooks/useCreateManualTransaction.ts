import { useMutation } from 'urql'
import { CREATE_MANUAL_TRANSACTION_MUTATION } from '../graphql/queries'
import type {
  CreateManualTransactionMutation,
  ManualTransactionInput,
} from '../graphql/types'

export function useCreateManualTransaction() {
  return useMutation<
    CreateManualTransactionMutation,
    { input: ManualTransactionInput }
  >(CREATE_MANUAL_TRANSACTION_MUTATION)
}
