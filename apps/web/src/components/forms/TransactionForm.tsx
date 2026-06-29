import { useState, type FormEvent } from 'react'
import {
  Field,
  FormActions,
  PrimaryButton,
  SecondaryButton,
  SelectInput,
  TextInput,
} from '../FormControls'
import { useCreateManualTransaction } from '../../hooks/useCreateManualTransaction'
import type { Account, Transaction } from '../../graphql/types'

type TransactionFormProps = {
  accounts: Account[]
  onCancel: () => void
  onCreated: (transaction: Transaction) => void
}

const uuidPattern =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i

function optional(value: string): string | null {
  return value.trim() === '' ? null : value.trim()
}

function today(): string {
  return new Date().toISOString().slice(0, 10)
}

export function TransactionForm({
  accounts,
  onCancel,
  onCreated,
}: TransactionFormProps) {
  const manualAccounts = accounts.filter((account) => uuidPattern.test(account.id))
  const [accountId, setAccountId] = useState(manualAccounts[0]?.id ?? '')
  const [amountCents, setAmountCents] = useState('')
  const [currency, setCurrency] = useState('USD')
  const [merchantName, setMerchantName] = useState('')
  const [rawDescription, setRawDescription] = useState('')
  const [categoryPrimary, setCategoryPrimary] = useState('')
  const [categoryDetailed, setCategoryDetailed] = useState('')
  const [transactionDate, setTransactionDate] = useState(today())
  const [pending, setPending] = useState(false)
  const [transactionType, setTransactionType] = useState('expense')
  const [notes, setNotes] = useState('')
  const [result, createTransaction] = useCreateManualTransaction()

  async function handleSubmit(event: FormEvent) {
    event.preventDefault()

    const response = await createTransaction({
      input: {
        accountId,
        amountCents: Number(amountCents),
        currency: optional(currency),
        merchantName: optional(merchantName),
        rawDescription: optional(rawDescription),
        categoryPrimary: optional(categoryPrimary),
        categoryDetailed: optional(categoryDetailed),
        transactionDate,
        pending,
        transactionType: optional(transactionType),
        notes: optional(notes),
      },
    })

    if (response.data?.createManualTransaction) {
      onCreated(response.data.createManualTransaction)
    }
  }

  if (manualAccounts.length === 0) {
    return (
      <div>
        <p className="text-sm leading-6 text-muted">
          Create a manual account first. Transactions need a real account ID,
          and the starter mock accounts are display-only.
        </p>
        <FormActions>
          <SecondaryButton type="button" onClick={onCancel}>
            Close
          </SecondaryButton>
        </FormActions>
      </div>
    )
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <Field label="Account">
        <SelectInput
          value={accountId}
          onChange={(event) => setAccountId(event.target.value)}
        >
          {manualAccounts.map((account) => (
            <option key={account.id} value={account.id}>
              {account.name}
            </option>
          ))}
        </SelectInput>
      </Field>
      <div className="grid gap-4 sm:grid-cols-2">
        <Field label="Amount cents" hint="Use negative cents for spending.">
          <TextInput
            required
            inputMode="numeric"
            value={amountCents}
            onChange={(event) => setAmountCents(event.target.value)}
            placeholder="-4200"
          />
        </Field>
        <Field label="Date">
          <TextInput
            required
            type="date"
            value={transactionDate}
            onChange={(event) => setTransactionDate(event.target.value)}
          />
        </Field>
      </div>
      <Field label="Merchant">
        <TextInput
          value={merchantName}
          onChange={(event) => setMerchantName(event.target.value)}
          placeholder="Coffee Bar"
        />
      </Field>
      <Field label="Raw description">
        <TextInput
          value={rawDescription}
          onChange={(event) => setRawDescription(event.target.value)}
          placeholder="Optional original text"
        />
      </Field>
      <div className="grid gap-4 sm:grid-cols-2">
        <Field label="Category">
          <TextInput
            value={categoryPrimary}
            onChange={(event) => setCategoryPrimary(event.target.value)}
            placeholder="Dining"
          />
        </Field>
        <Field label="Detail">
          <TextInput
            value={categoryDetailed}
            onChange={(event) => setCategoryDetailed(event.target.value)}
            placeholder="Coffee"
          />
        </Field>
      </div>
      <div className="grid gap-4 sm:grid-cols-2">
        <Field label="Type">
          <SelectInput
            value={transactionType}
            onChange={(event) => setTransactionType(event.target.value)}
          >
            <option value="income">Income</option>
            <option value="expense">Expense</option>
            <option value="transfer">Transfer</option>
            <option value="payment">Payment</option>
            <option value="refund">Refund</option>
            <option value="fee">Fee</option>
            <option value="interest">Interest</option>
            <option value="adjustment">Adjustment</option>
            <option value="other">Other</option>
          </SelectInput>
        </Field>
        <Field label="Currency">
          <TextInput
            value={currency}
            onChange={(event) => setCurrency(event.target.value.toUpperCase())}
            placeholder="USD"
          />
        </Field>
      </div>
      <label className="flex items-center gap-3 text-sm text-muted">
        <input
          type="checkbox"
          checked={pending}
          onChange={(event) => setPending(event.target.checked)}
          className="size-4 rounded border-white/[0.15] bg-background"
        />
        Pending
      </label>
      <Field label="Notes">
        <TextInput
          value={notes}
          onChange={(event) => setNotes(event.target.value)}
          placeholder="Optional"
        />
      </Field>
      {result.error ? (
        <p className="text-sm text-red-300">{result.error.message}</p>
      ) : null}
      <FormActions>
        <PrimaryButton
          type="submit"
          disabled={result.fetching || !accountId || amountCents.trim() === ''}
        >
          {result.fetching ? 'Creating...' : 'Create transaction'}
        </PrimaryButton>
        <SecondaryButton type="button" onClick={onCancel}>
          Cancel
        </SecondaryButton>
      </FormActions>
    </form>
  )
}
