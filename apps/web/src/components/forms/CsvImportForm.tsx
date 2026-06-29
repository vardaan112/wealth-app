import { useState, type FormEvent } from 'react'
import {
  Field,
  FormActions,
  PrimaryButton,
  SelectInput,
  TextAreaInput,
} from '../FormControls'
import type { Account, CsvImportResult } from '../../graphql/types'
import { useImportTransactionsCsv } from '../../hooks/useImportTransactionsCsv'

type CsvImportFormProps = {
  accounts: Account[]
  onImported: () => void
}

const uuidPattern =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i

const sampleCsv = `date,description,amount,category
2026-06-01,Coffee Shop,-4.25,Dining
2026-06-02,Payroll,1250.00,Income`

export function CsvImportForm({ accounts, onImported }: CsvImportFormProps) {
  const importableAccounts = accounts.filter((account) =>
    uuidPattern.test(account.id),
  )
  const [accountId, setAccountId] = useState(importableAccounts[0]?.id ?? '')
  const [source, setSource] = useState('Generic')
  const [csvText, setCsvText] = useState('')
  const [importResult, setImportResult] = useState<CsvImportResult | null>(null)
  const [result, importCsv] = useImportTransactionsCsv()

  async function handleSubmit(event: FormEvent) {
    event.preventDefault()
    setImportResult(null)

    const response = await importCsv({
      input: {
        accountId,
        source,
        csvText,
      },
    })

    if (response.data?.importTransactionsCsv) {
      setImportResult(response.data.importTransactionsCsv)
      onImported()
    }
  }

  if (importableAccounts.length === 0) {
    return (
      <p className="text-sm leading-6 text-muted">
        Create a manual account first. CSV imports need a real account ID, and
        the starter mock accounts are display-only.
      </p>
    )
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="grid gap-4 sm:grid-cols-2">
        <Field label="Account">
          <SelectInput
            value={accountId}
            onChange={(event) => setAccountId(event.target.value)}
          >
            {importableAccounts.map((account) => (
              <option key={account.id} value={account.id}>
                {account.name}
              </option>
            ))}
          </SelectInput>
        </Field>
        <Field label="Source">
          <SelectInput
            value={source}
            onChange={(event) => setSource(event.target.value)}
          >
            <option value="Chase">Chase</option>
            <option value="Discover">Discover</option>
            <option value="Generic">Generic</option>
          </SelectInput>
        </Field>
      </div>
      <Field
        label="CSV text"
        hint="Expected columns: date, description, amount, category"
      >
        <TextAreaInput
          required
          value={csvText}
          onChange={(event) => setCsvText(event.target.value)}
          placeholder={sampleCsv}
          spellCheck={false}
        />
      </Field>
      {result.error ? (
        <p className="text-sm text-red-300">{result.error.message}</p>
      ) : null}
      {importResult ? (
        <div className="rounded-2xl border border-white/[0.07] bg-background/45 p-4 text-sm">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-xs uppercase tracking-[0.16em] text-muted">
                Imported
              </p>
              <p className="mt-1 text-lg text-text">{importResult.importedCount}</p>
            </div>
            <div>
              <p className="text-xs uppercase tracking-[0.16em] text-muted">
                Skipped
              </p>
              <p className="mt-1 text-lg text-text">{importResult.skippedCount}</p>
            </div>
          </div>
          {importResult.errors.length > 0 ? (
            <ul className="mt-4 list-disc space-y-1 pl-5 text-red-300">
              {importResult.errors.map((error) => (
                <li key={error}>{error}</li>
              ))}
            </ul>
          ) : (
            <p className="mt-4 text-xs text-muted">No row errors.</p>
          )}
        </div>
      ) : null}
      <FormActions>
        <PrimaryButton
          type="submit"
          disabled={result.fetching || !accountId || csvText.trim() === ''}
        >
          {result.fetching ? 'Importing...' : 'Import CSV'}
        </PrimaryButton>
      </FormActions>
    </form>
  )
}
