import { useState, type FormEvent } from 'react'
import {
  Field,
  FormActions,
  PrimaryButton,
  TextInput,
} from '../components/FormControls'
import { useAuth } from '../auth/AuthContext'
import { useLogin } from '../hooks/useLogin'

export function LoginPage() {
  const { login } = useAuth()
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [result, loginMutation] = useLogin()

  async function handleSubmit(event: FormEvent) {
    event.preventDefault()
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
          Sign in
        </h1>
        <p className="mt-2 text-sm leading-6 text-muted">
          Use the single account configured for this personal app.
        </p>

        <form onSubmit={handleSubmit} className="mt-8 space-y-4">
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
              autoComplete="current-password"
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
              {result.fetching ? 'Signing in...' : 'Sign in'}
            </PrimaryButton>
          </FormActions>
        </form>
      </section>
    </main>
  )
}
