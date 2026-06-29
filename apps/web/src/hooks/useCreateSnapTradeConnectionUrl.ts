import { useMutation } from 'urql'
import { CREATE_SNAPTRADE_CONNECTION_URL_MUTATION } from '../graphql/queries'
import type { CreateSnapTradeConnectionUrlMutation } from '../graphql/types'

export function useCreateSnapTradeConnectionUrl() {
  return useMutation<
    CreateSnapTradeConnectionUrlMutation,
    Record<string, never>
  >(CREATE_SNAPTRADE_CONNECTION_URL_MUTATION)
}
