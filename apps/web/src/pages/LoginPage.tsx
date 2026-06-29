import { useState, type FormEvent } from 'react'
import {
  Field,
  FormActions,
  PrimaryButton,
  SecondaryButton,
  TextInput,
} from '../components/FormControls'
import { useAuth } from '../auth/AuthContext'
import { useLogin } from '../hooks/useLogin'
import { useSignUp } from '../hooks/useSignUp'

type AuthMode = 'signIn' | 'signUp'

export function LoginPage() {
  const { login } = useAuth()
  const [mode, setMode] = useState<AuthMode>('signIn')
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [displayName, setDisplayName] = useState('')
  const [loginResult, loginMutation] = useLogin()
  const [signUpResult, signUpMutation] = useSignUp()
  const isSignUp = mode === 'signUp'
  const result = isSignUp ? signUpResult : loginResult
  const heading = isSignUp ? 'Create your account' : 'Sign in'
  const description = isSignUp
    ? 'Start with an email and password, then connect your financial accounts from Settings.'
    : 'Use your email and password for this personal finance app.'
  let submitLabel = isSignUp ? 'Create account' : 'Sign in'
  if (result.fetching) {
    submitLabel = isSignUp ? 'Creating account...' : 'Signing in...'
  }

  async function handleSubmit(event: FormEvent) {
    event.preventDefault()

    if (isSignUp) {
      const response = await signUpMutation({
        input: {
          email,
          password,
          displayName: displayName.trim() || null,
        },
      })

      if (response.data?.signUp.token) {
        login(response.data.signUp.token)
      }

      return
    }

    const response = await loginMutation({
      input: {
        email,
        password,
      },
    })

    if (response.data?.login.token) {
      login(response.data.login.token)
    }
  }

  return (
    <main className="grid min-h-screen place-items-center px-4 text-text">
      <section className="w-full max-w-md rounded-[2rem] border border-white/[0.07] bg-surface/80 p-6 shadow-[0_30px_90px_rgba(0,0,0,0.35)] backdrop-blur">
        <p className="text-xs uppercase tracking-[0.28em] text-accent">
          Wealth
        </p>
        <h1 className="mt-4 text-3xl font-semibold tracking-[-0.04em]">
          {heading}
        </h1>
        <p className="mt-2 text-sm leading-6 text-muted">
          {description}
        </p>

        <form onSubmit={handleSubmit} className="mt-8 space-y-4">
          {isSignUp ? (
            <Field label="Name" hint="Optional. You can keep this simple.">
              <TextInput
                type="text"
                value={displayName}
                onChange={(event) => setDisplayName(event.target.value)}
                placeholder="Your name"
                autoComplete="name"
              />
            </Field>
          ) : null}
          <Field label="Email">
            <TextInput
              required
              type="email"
              value={email}
              onChange={(event) => setEmail(event.target.value)}
              placeholder="you@example.com"
              autoComplete="email"
            />
          </Field>
          <Field label="Password">
            <TextInput
              required
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              placeholder="Your password"
              autoComplete={isSignUp ? 'new-password' : 'current-password'}
            />
          </Field>
          {result.error ? (
            <p className="text-sm text-red-300">{result.error.message}</p>
          ) : null}
          <FormActions>
            <PrimaryButton
              type="submit"
              disabled={result.fetching || !email || !password}
              className="w-full"
            >
              {submitLabel}
            </PrimaryButton>
          </FormActions>
        </form>
        <div className="mt-5 border-t border-white/[0.07] pt-5">
          <p className="text-center text-sm text-muted">
            {isSignUp ? 'Already have an account?' : 'New here?'}
          </p>
          <SecondaryButton
            type="button"
            className="mt-3 w-full"
            onClick={() => setMode(isSignUp ? 'signIn' : 'signUp')}
          >
            {isSignUp ? 'Sign in instead' : 'Create an account'}
          </SecondaryButton>
        </div>
      </section>
    </main>
  )
}
