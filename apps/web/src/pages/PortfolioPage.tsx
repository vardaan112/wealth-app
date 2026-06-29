import { useState } from 'react'
import { HoldingForm } from '../components/forms/HoldingForm'
import { Modal } from '../components/Modal'
import { PlaceholderCard } from '../components/PlaceholderCard'
import { useAccounts } from '../hooks/useAccounts'
import { useHoldings } from '../hooks/useHoldings'
import { formatMoney } from '../lib/format'

export function PortfolioPage() {
  const [isHoldingModalOpen, setIsHoldingModalOpen] = useState(false)
  const [holdingsResult, refreshHoldings] = useHoldings()
  const [accountsResult, refreshAccounts] = useAccounts()
  const holdings = holdingsResult.data?.holdings ?? []
  const accounts = accountsResult.data?.accounts ?? []
  const totalValue = holdings.reduce(
    (total, holding) => total + holding.marketValue.amountCents,
    0,
  )

  return (
    <div className="animate-rise mx-auto flex max-w-5xl flex-col gap-8 pt-6">
      <header className="flex flex-col justify-between gap-4 sm:flex-row sm:items-end">
        <div>
          <h1 className="text-3xl font-semibold tracking-[-0.04em]">
            Portfolio
          </h1>
          <p className="mt-2 text-sm text-muted">
            Holdings, allocation, and performance.
          </p>
        </div>
        <button
          type="button"
          onClick={() => setIsHoldingModalOpen(true)}
          className="rounded-full bg-accent px-5 py-3 text-sm font-medium text-background hover:bg-accent/90"
        >
          Add holding
        </button>
      </header>
      <div className="grid gap-3 sm:grid-cols-2">
        <PlaceholderCard
          title="Total value"
          description={
            holdings.length
              ? `${formatMoney({ amountCents: totalValue, currency: 'USD' })} across holdings.`
              : 'No manual holdings yet.'
          }
        />
        <PlaceholderCard
          title="Manual holdings"
          description="Current entries from GraphQL."
        >
          {holdingsResult.fetching ? (
            <p className="text-sm text-muted">Loading holdings...</p>
          ) : holdingsResult.error ? (
            <p className="text-sm text-red-300">{holdingsResult.error.message}</p>
          ) : holdings.length === 0 ? (
            <p className="text-sm text-muted">No holdings yet.</p>
          ) : (
            <ul className="divide-y divide-white/[0.05]">
              {holdings.map((holding) => (
                <li
                  key={holding.id}
                  className="flex items-center justify-between gap-4 py-3 first:pt-0 last:pb-0"
                >
                  <div className="min-w-0">
                    <p className="truncate text-sm text-text/90">
                      {holding.symbol}
                    </p>
                    <p className="mt-0.5 text-xs text-muted">
                      {holding.assetName} / {holding.quantity}
                    </p>
                  </div>
                  <span className="text-sm tabular-nums text-text/90">
                    {formatMoney(holding.marketValue)}
                  </span>
                </li>
              ))}
            </ul>
          )}
        </PlaceholderCard>
        <PlaceholderCard
          title="YTD return"
          description="+8.2% vs benchmark +6.1%."
        />
        <PlaceholderCard
          title="Top holdings"
          description={
            holdings.length
              ? holdings
                  .slice(0, 3)
                  .map((holding) => holding.symbol)
                  .join(', ')
              : 'Create a holding to populate this.'
          }
        />
      </div>
      <Modal
        title="Add holding"
        description="Track a manually entered stock, ETF, crypto, bond, or cash position."
        isOpen={isHoldingModalOpen}
        onClose={() => setIsHoldingModalOpen(false)}
      >
        <HoldingForm
          accounts={accounts}
          onCancel={() => setIsHoldingModalOpen(false)}
          onCreated={() => {
            setIsHoldingModalOpen(false)
            refreshHoldings({ requestPolicy: 'network-only' })
            refreshAccounts({ requestPolicy: 'network-only' })
          }}
        />
      </Modal>
    </div>
  )
}
