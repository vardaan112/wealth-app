import { useState } from 'react'
import { AccountForm } from '../components/forms/AccountForm'
import { CsvImportForm } from '../components/forms/CsvImportForm'
import { Modal } from '../components/Modal'
import { PlaceholderCard } from '../components/PlaceholderCard'
import { useAuth } from '../auth/AuthContext'
import { useAccounts } from '../hooks/useAccounts'
import { useMonthlySummary } from '../hooks/useMonthlySummary'
import { useTransactions } from '../hooks/useTransactions'

export function SettingsPage() {
  const { logout } = useAuth()
  const [isAccountModalOpen, setIsAccountModalOpen] = useState(false)
  const [accountsResult, refreshAccounts] = useAccounts()
  const [, refreshTransactions] = useTransactions()
  const [, refreshMonthlySummary] = useMonthlySummary()
  const accounts = accountsResult.data?.accounts ?? []

  return (
    <div className="animate-rise mx-auto flex max-w-5xl flex-col gap-8 pt-6">
      <header className="flex flex-col justify-between gap-4 sm:flex-row sm:items-end">
        <div>
          <h1 className="text-3xl font-semibold tracking-[-0.04em]">
            Settings
          </h1>
          <p className="mt-2 text-sm text-muted">
            Preferences and account configuration.
          </p>
        </div>
        <button
          type="button"
          onClick={() => setIsAccountModalOpen(true)}
          className="rounded-full bg-accent px-5 py-3 text-sm font-medium text-background hover:bg-accent/90"
        >
          Add account
        </button>
      </header>
      <div className="grid gap-3 sm:grid-cols-2">
        <PlaceholderCard
          title="Profile"
          description="Name, email, and display preferences."
        />
        <PlaceholderCard
          title="Connected accounts"
          description="Manual accounts you can use for entries."
        >
          {accountsResult.fetching ? (
            <p className="text-sm text-muted">Loading accounts...</p>
          ) : accounts.length === 0 ? (
            <p className="text-sm text-muted">No accounts yet.</p>
          ) : (
            <ul className="divide-y divide-white/[0.05]">
              {accounts.map((account) => (
                <li
                  key={account.id}
                  className="flex items-center justify-between gap-4 py-3 first:pt-0 last:pb-0"
                >
                  <div className="min-w-0">
                    <p className="truncate text-sm text-text/90">{account.name}</p>
                    <p className="mt-0.5 text-xs text-muted">
                      {account.accountType} / {account.provider}
                    </p>
                  </div>
                  <span className="text-xs text-muted">{account.currency}</span>
                </li>
              ))}
            </ul>
          )}
        </PlaceholderCard>
        <PlaceholderCard
          title="Currency & locale"
          description="USD · United States."
        />
        <PlaceholderCard
          title="Notifications"
          description="Alerts for bills and large transactions."
        />
        <PlaceholderCard title="Session" description="Sign out of this browser.">
          <button
            type="button"
            onClick={logout}
            className="rounded-full border border-white/[0.08] px-4 py-2 text-sm text-muted hover:border-white/[0.16] hover:text-text"
          >
            Log out
          </button>
        </PlaceholderCard>
      </div>
      <section>
        <PlaceholderCard
          title="Import Data"
          description="Paste a simple CSV export to create transactions. No file upload yet."
          className="max-w-3xl"
        >
          <CsvImportForm
            accounts={accounts}
            onImported={() => {
              refreshTransactions({ requestPolicy: 'network-only' })
              refreshMonthlySummary({ requestPolicy: 'network-only' })
            }}
          />
        </PlaceholderCard>
      </section>
      <Modal
        title="Add account"
        description="Create a manual account for transactions and holdings."
        isOpen={isAccountModalOpen}
        onClose={() => setIsAccountModalOpen(false)}
      >
        <AccountForm
          onCancel={() => setIsAccountModalOpen(false)}
          onCreated={() => {
            setIsAccountModalOpen(false)
            refreshAccounts({ requestPolicy: 'network-only' })
          }}
        />
      </Modal>
    </div>
  )
}
