import { useMutation } from 'urql'
import { LOGIN_MUTATION } from '../graphql/queries'
import type { LoginInput, LoginMutation } from '../graphql/types'

export function useLogin() {
  return useMutation<LoginMutation, { input: LoginInput }>(LOGIN_MUTATION)
}
