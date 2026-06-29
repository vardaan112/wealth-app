import { useState, type FormEvent } from 'react'
import {
  Field,
  FormActions,
  PrimaryButton,
  SecondaryButton,
  SelectInput,
  TextInput,
} from '../FormControls'
import { useCreateManualHolding } from '../../hooks/useCreateManualHolding'
import type { Account, Holding } from '../../graphql/types'

type HoldingFormProps = {
  accounts: Account[]
  onCancel: () => void
  onCreated: (holding: Holding) => void
}

const uuidPattern =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i

function optional(value: string): string | null {
  return value.trim() === '' ? null : value.trim()
}

function optionalNumber(value: string): number | null {
  return value.trim() === '' ? null : Number(value)
}

export function HoldingForm({ accounts, onCancel, onCreated }: HoldingFormProps) {
  const manualAccounts = accounts.filter((account) => uuidPattern.test(account.id))
  const [accountId, setAccountId] = useState(manualAccounts[0]?.id ?? '')
  const [symbol, setSymbol] = useState('')
  const [assetName, setAssetName] = useState('')
  const [assetType, setAssetType] = useState('stock')
  const [quantity, setQuantity] = useState('')
  const [marketValueCents, setMarketValueCents] = useState('')
  const [costBasisCents, setCostBasisCents] = useState('')
  const [priceCents, setPriceCents] = useState('')
  const [currency, setCurrency] = useState('USD')
  const [result, createHolding] = useCreateManualHolding()

  async function handleSubmit(event: FormEvent) {
    event.preventDefault()

    const response = await createHolding({
      input: {
        accountId,
        symbol: symbol.trim().toUpperCase(),
        assetName: optional(assetName),
        assetType: optional(assetType),
        quantity: Number(quantity),
        marketValueCents: optionalNumber(marketValueCents),
        costBasisCents: optionalNumber(costBasisCents),
        priceCents: optionalNumber(priceCents),
        currency: optional(currency),
      },
    })

    if (response.data?.createManualHolding) {
      onCreated(response.data.createManualHolding)
    }
  }

  if (manualAccounts.length === 0) {
    return (
      <div>
        <p className="text-sm leading-6 text-muted">
          Create a manual account first. Holdings need a real account ID, and
          the starter mock accounts are display-only.
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
        <Field label="Symbol">
          <TextInput
            required
            value={symbol}
            onChange={(event) => setSymbol(event.target.value)}
            placeholder="VTI"
          />
        </Field>
        <Field label="Asset type">
          <SelectInput
            value={assetType}
            onChange={(event) => setAssetType(event.target.value)}
          >
            <option value="stock">Stock</option>
            <option value="etf">ETF</option>
            <option value="mutual_fund">Mutual fund</option>
            <option value="option">Option</option>
            <option value="crypto">Crypto</option>
            <option value="cash">Cash</option>
            <option value="bond">Bond</option>
            <option value="other">Other</option>
          </SelectInput>
        </Field>
      </div>
      <Field label="Asset name">
        <TextInput
          value={assetName}
          onChange={(event) => setAssetName(event.target.value)}
          placeholder="Vanguard Total Stock Market ETF"
        />
      </Field>
      <div className="grid gap-4 sm:grid-cols-2">
        <Field label="Quantity">
          <TextInput
            required
            inputMode="decimal"
            value={quantity}
            onChange={(event) => setQuantity(event.target.value)}
            placeholder="10.5"
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
      <div className="grid gap-4 sm:grid-cols-3">
        <Field label="Market value cents">
          <TextInput
            inputMode="numeric"
            value={marketValueCents}
            onChange={(event) => setMarketValueCents(event.target.value)}
            placeholder="100000"
          />
        </Field>
        <Field label="Cost basis cents">
          <TextInput
            inputMode="numeric"
            value={costBasisCents}
            onChange={(event) => setCostBasisCents(event.target.value)}
            placeholder="90000"
          />
        </Field>
        <Field label="Price cents">
          <TextInput
            inputMode="numeric"
            value={priceCents}
            onChange={(event) => setPriceCents(event.target.value)}
            placeholder="25000"
          />
        </Field>
      </div>
      {result.error ? (
        <p className="text-sm text-red-300">{result.error.message}</p>
      ) : null}
      <FormActions>
        <PrimaryButton
          type="submit"
          disabled={result.fetching || !accountId || !symbol.trim() || !quantity.trim()}
        >
          {result.fetching ? 'Creating...' : 'Create holding'}
        </PrimaryButton>
        <SecondaryButton type="button" onClick={onCancel}>
          Cancel
        </SecondaryButton>
      </FormActions>
    </form>
  )
}
