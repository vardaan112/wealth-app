import { useEffect, useState } from 'react'
import { usePlaidLink } from 'react-plaid-link'
import { AccountForm } from '../components/forms/AccountForm'
import { CsvImportForm } from '../components/forms/CsvImportForm'
import { Modal } from '../components/Modal'
import { PlaceholderCard } from '../components/PlaceholderCard'
import { useAuth } from '../auth/AuthContext'
import { useAccounts } from '../hooks/useAccounts'
import { useCreatePlaidLinkToken } from '../hooks/useCreatePlaidLinkToken'
import { useExchangePlaidPublicToken } from '../hooks/useExchangePlaidPublicToken'
import { useHoldings } from '../hooks/useHoldings'
import { useMonthlySummary } from '../hooks/useMonthlySummary'
import { useNetWorthTimeline } from '../hooks/useNetWorthTimeline'
import { useTransactions } from '../hooks/useTransactions'
import { useTriggerMockSync } from '../hooks/useTriggerMockSync'

const mockProviders = [
  {
    name: 'Chase',
    description: 'Checking account and cashflow',
  },
  {
    name: 'Discover',
    description: 'Credit card spending and payments',
  },
  {
    name: 'Robinhood',
    description: 'Brokerage holdings and activity',
  },
]

export function SettingsPage() {
  const { logout } = useAuth()
  const [isAccountModalOpen, setIsAccountModalOpen] = useState(false)
  const [lastSyncAt, setLastSyncAt] = useState<string | null>(null)
  const [plaidLinkToken, setPlaidLinkToken] = useState<string | null>(null)
  const [shouldOpenPlaid, setShouldOpenPlaid] = useState(false)
  const [plaidStatus, setPlaidStatus] = useState<
    'idle' | 'creating' | 'opening' | 'exchanging' | 'connected' | 'failed'
  >('idle')
  const [plaidMessage, setPlaidMessage] = useState(
    'Connect a bank account with Plaid Link. Transactions will not sync yet.',
  )
  const [accountsResult, refreshAccounts] = useAccounts()
  const [, refreshTransactions] = useTransactions()
  const [, refreshHoldings] = useHoldings()
  const [, refreshMonthlySummary] = useMonthlySummary()
  const [, refreshNetWorthTimeline] = useNetWorthTimeline()
  const [syncResult, triggerMockSync] = useTriggerMockSync()
  const [linkTokenResult, createPlaidLinkToken] = useCreatePlaidLinkToken()
  const [exchangeResult, exchangePlaidPublicToken] =
    useExchangePlaidPublicToken()
  const accounts = accountsResult.data?.accounts ?? []
  const synced = syncResult.data?.triggerMockSync
  const plaidBusy =
    linkTokenResult.fetching ||
    exchangeResult.fetching ||
    plaidStatus === 'creating' ||
    plaidStatus === 'opening' ||
    plaidStatus === 'exchanging'

  const { open: openPlaid, ready: plaidReady } = usePlaidLink({
    token: plaidLinkToken,
    onSuccess: async (publicToken) => {
      setPlaidStatus('exchanging')
      setPlaidMessage('Finishing secure bank connection...')

      const response = await exchangePlaidPublicToken({ publicToken })
      if (response.data?.exchangePlaidPublicToken && !response.error) {
        setPlaidStatus('connected')
        setPlaidMessage('Bank connected successfully. Transaction sync is not enabled yet.')
        setPlaidLinkToken(null)
        return
      }

      setPlaidStatus('failed')
      setPlaidMessage(response.error?.message ?? 'Could not exchange Plaid token.')
    },
    onExit: (error) => {
      if (error) {
        setPlaidStatus('failed')
        setPlaidMessage(error.display_message ?? 'Plaid Link closed with an error.')
        return
      }

      if (plaidStatus === 'opening') {
        setPlaidStatus('idle')
        setPlaidMessage('Plaid Link was closed before connecting.')
      }
    },
  })

  useEffect(() => {
    if (!shouldOpenPlaid || !plaidReady) {
      return
    }

    openPlaid()
    setShouldOpenPlaid(false)
  }, [openPlaid, plaidReady, shouldOpenPlaid])

  async function handleMockSync() {
    const response = await triggerMockSync({})

    if (response.data?.triggerMockSync && !response.error) {
      setLastSyncAt(new Date().toLocaleString())
      refreshAccounts({ requestPolicy: 'network-only' })
      refreshTransactions({ requestPolicy: 'network-only' })
      refreshHoldings({ requestPolicy: 'network-only' })
      refreshMonthlySummary({ requestPolicy: 'network-only' })
      refreshNetWorthTimeline({ requestPolicy: 'network-only' })
    }
  }

  async function handleConnectBank() {
    setPlaidStatus('creating')
    setPlaidMessage('Creating a secure Plaid Link session...')

    const response = await createPlaidLinkToken({})
    if (response.data?.createPlaidLinkToken && !response.error) {
      setPlaidLinkToken(response.data.createPlaidLinkToken)
      setShouldOpenPlaid(true)
      setPlaidStatus('opening')
      setPlaidMessage('Opening Plaid Link...')
      return
    }

    setPlaidStatus('failed')
    setPlaidMessage(response.error?.message ?? 'Could not create Plaid Link token.')
  }

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
          title="Bank Connections"
          description="Connect a bank securely with Plaid. This only establishes the connection for now."
          className="max-w-3xl"
        >
          <div className="flex flex-col gap-4 rounded-2xl border border-white/[0.06] bg-background/40 p-4 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <p className="text-xs uppercase tracking-[0.18em] text-muted">
                Plaid Link
              </p>
              <p className="mt-1 text-sm text-text">
                {plaidStatus === 'connected'
                  ? 'Connected'
                  : plaidStatus === 'failed'
                    ? 'Connection failed'
                    : plaidBusy
                      ? 'Connecting...'
                      : 'Ready to connect'}
              </p>
              <p className="mt-1 max-w-xl text-xs leading-5 text-muted">
                {plaidMessage}
              </p>
            </div>
            <button
              type="button"
              onClick={handleConnectBank}
              disabled={plaidBusy}
              className="rounded-full bg-accent px-5 py-3 text-sm font-medium text-background hover:bg-accent/90 disabled:cursor-not-allowed disabled:opacity-50"
            >
              {plaidBusy ? 'Connecting...' : 'Connect Bank'}
            </button>
          </div>
        </PlaceholderCard>
      </section>
      <section>
        <PlaceholderCard
          title="Provider Sync"
          description="Mock provider sync for Chase, Discover, and Robinhood-like data."
          className="max-w-3xl"
        >
          <div className="grid gap-3 md:grid-cols-3">
            {mockProviders.map((provider) => (
              <div
                key={provider.name}
                className="rounded-2xl border border-white/[0.06] bg-white/[0.025] p-4"
              >
                <div className="flex items-center justify-between gap-3">
                  <p className="text-sm font-medium text-text">{provider.name}</p>
                  <span className="rounded-full bg-accent/10 px-2.5 py-1 text-[10px] uppercase tracking-[0.18em] text-accent">
                    Connected
                  </span>
                </div>
                <p className="mt-2 text-xs leading-5 text-muted">
                  {provider.description}
                </p>
              </div>
            ))}
          </div>

          <div className="mt-5 flex flex-col gap-4 rounded-2xl border border-white/[0.06] bg-background/40 p-4 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <p className="text-xs uppercase tracking-[0.18em] text-muted">
                Sync status
              </p>
              <p className="mt-1 text-sm text-text">
                {syncResult.fetching
                  ? 'Syncing mock provider data...'
                  : syncResult.error
                    ? 'Sync failed'
                    : synced
                      ? 'Synced successfully'
                      : 'Ready to sync'}
              </p>
              <p className="mt-1 text-xs text-muted">
                Last sync: {lastSyncAt ?? 'Not run in this browser session'}
              </p>
            </div>
            <button
              type="button"
              onClick={handleMockSync}
              disabled={syncResult.fetching}
              className="rounded-full bg-accent px-5 py-3 text-sm font-medium text-background hover:bg-accent/90 disabled:cursor-not-allowed disabled:opacity-50"
            >
              {syncResult.fetching ? 'Syncing...' : 'Run mock sync'}
            </button>
          </div>

          {synced ? (
            <div className="mt-4 grid gap-2 text-xs text-muted sm:grid-cols-5">
              <SyncMetric label="Accounts" value={synced.accountsSynced} />
              <SyncMetric label="Transactions" value={synced.transactionsSynced} />
              <SyncMetric label="Holdings" value={synced.holdingsSynced} />
              <SyncMetric
                label="Investments"
                value={synced.investmentTransactionsSynced}
              />
              <SyncMetric label="Balances" value={synced.balanceSnapshotsSynced} />
            </div>
          ) : null}

          {syncResult.error ? (
            <p className="mt-4 text-sm text-red-300">{syncResult.error.message}</p>
          ) : null}

          {synced?.errors.length ? (
            <ul className="mt-4 list-disc space-y-1 pl-4 text-sm text-red-300">
              {synced.errors.map((error) => (
                <li key={error}>{error}</li>
              ))}
            </ul>
          ) : null}
        </PlaceholderCard>
      </section>
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

function SyncMetric({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-2xl border border-white/[0.05] bg-white/[0.025] p-3">
      <p className="text-lg font-semibold text-text">{value}</p>
      <p className="mt-0.5">{label}</p>
    </div>
  )
}
