import { useMutation } from 'urql'
import { EXCHANGE_PLAID_PUBLIC_TOKEN_MUTATION } from '../graphql/queries'
import type { ExchangePlaidPublicTokenMutation } from '../graphql/types'

export function useExchangePlaidPublicToken() {
  return useMutation<
    ExchangePlaidPublicTokenMutation,
    { publicToken: string }
  >(EXCHANGE_PLAID_PUBLIC_TOKEN_MUTATION)
}
