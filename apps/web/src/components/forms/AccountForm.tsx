import { useState, type FormEvent } from 'react'
import {
  Field,
  FormActions,
  PrimaryButton,
  SecondaryButton,
  SelectInput,
  TextInput,
} from '../FormControls'
import { useCreateManualAccount } from '../../hooks/useCreateManualAccount'
import type { Account } from '../../graphql/types'

type AccountFormProps = {
  onCancel: () => void
  onCreated: (account: Account) => void
}

function optional(value: string): string | null {
  return value.trim() === '' ? null : value.trim()
}

export function AccountForm({ onCancel, onCreated }: AccountFormProps) {
  const [name, setName] = useState('')
  const [accountType, setAccountType] = useState('checking')
  const [provider, setProvider] = useState('manual')
  const [currency, setCurrency] = useState('USD')
  const [result, createAccount] = useCreateManualAccount()

  async function handleSubmit(event: FormEvent) {
    event.preventDefault()

    const response = await createAccount({
      input: {
        name: name.trim(),
        accountType,
        provider: optional(provider),
        currency: optional(currency),
      },
    })

    if (response.data?.createManualAccount) {
      onCreated(response.data.createManualAccount)
    }
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <Field label="Name">
        <TextInput
          required
          value={name}
          onChange={(event) => setName(event.target.value)}
          placeholder="Primary checking"
        />
      </Field>
      <Field label="Type">
        <SelectInput
          value={accountType}
          onChange={(event) => setAccountType(event.target.value)}
        >
          <option value="checking">Checking</option>
          <option value="savings">Savings</option>
          <option value="credit_card">Credit card</option>
          <option value="brokerage">Brokerage</option>
          <option value="crypto">Crypto</option>
          <option value="cash">Cash</option>
          <option value="loan">Loan</option>
          <option value="manual">Manual</option>
          <option value="other">Other</option>
        </SelectInput>
      </Field>
      <div className="grid gap-4 sm:grid-cols-2">
        <Field label="Provider">
          <TextInput
            value={provider}
            onChange={(event) => setProvider(event.target.value)}
            placeholder="manual"
          />
        </Field>
        <Field label="Currency">
          <TextInput
            value={currency}
            onChange={(event) => setCurrency(event.target.value.toUpperCase())}
            placeholder="USD"
          />
        </Field>
      </div>
      {result.error ? (
        <p className="text-sm text-red-300">{result.error.message}</p>
      ) : null}
      <FormActions>
        <PrimaryButton type="submit" disabled={result.fetching || !name.trim()}>
          {result.fetching ? 'Creating...' : 'Create account'}
        </PrimaryButton>
        <SecondaryButton type="button" onClick={onCancel}>
          Cancel
        </SecondaryButton>
      </FormActions>
    </form>
  )
}
