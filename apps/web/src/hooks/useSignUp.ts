import { useMutation } from 'urql'
import { SIGN_UP_MUTATION } from '../graphql/queries'
import type { SignUpInput, SignUpMutation } from '../graphql/types'

export function useSignUp() {
  return useMutation<SignUpMutation, { input: SignUpInput }>(SIGN_UP_MUTATION)
}
