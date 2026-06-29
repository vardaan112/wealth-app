import { useMutation } from 'urql'
import { CREATE_PLAID_LINK_TOKEN_MUTATION } from '../graphql/queries'
import type { CreatePlaidLinkTokenMutation } from '../graphql/types'

export function useCreatePlaidLinkToken() {
  return useMutation<CreatePlaidLinkTokenMutation, Record<string, never>>(
    CREATE_PLAID_LINK_TOKEN_MUTATION,
  )
}
