import { useMutation } from 'urql'
import { CREATE_MANUAL_ACCOUNT_MUTATION } from '../graphql/queries'
import type {
  CreateManualAccountMutation,
  ManualAccountInput,
} from '../graphql/types'

export function useCreateManualAccount() {
  return useMutation<
    CreateManualAccountMutation,
    { input: ManualAccountInput }
  >(CREATE_MANUAL_ACCOUNT_MUTATION)
}
