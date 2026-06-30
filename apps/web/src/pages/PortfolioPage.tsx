import { useState } from 'react'
import { Cell, Pie, PieChart, ResponsiveContainer, Tooltip } from 'recharts'
import { HoldingForm } from '../components/forms/HoldingForm'
import { Modal } from '../components/Modal'
import { PlaceholderCard } from '../components/PlaceholderCard'
import { useAccounts } from '../hooks/useAccounts'
import { useHoldings } from '../hooks/useHoldings'
import {
  allocationColors,
  buildHoldingsAllocation,
} from '../lib/allocation'
import { chartLabelStyle, chartTooltipStyle, formatCents } from '../lib/chart'
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
  const allocationData = buildHoldingsAllocation(holdings)

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
        <PlaceholderCard title="Allocation" description="By holding symbol.">
          {allocationData.length === 0 ? (
            <p className="text-sm text-muted">No allocation yet.</p>
          ) : (
            <div className="grid gap-5 sm:grid-cols-[0.9fr_1.1fr] sm:items-center">
              <div className="h-44 w-full">
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Tooltip
                      contentStyle={chartTooltipStyle}
                      labelStyle={chartLabelStyle}
                      formatter={(value, _name, item) => {
                        const payload = item.payload as {
                          label: string
                          assetType: string
                        }
                        return [
                          formatCents(Number(value)),
                          `${payload.label} (${payload.assetType})`,
                        ]
                      }}
                    />
                    <Pie
                      data={allocationData}
                      dataKey="value"
                      nameKey="label"
                      innerRadius="62%"
                      outerRadius="86%"
                      paddingAngle={2}
                      stroke="rgba(8,10,13,0.7)"
                      strokeWidth={2}
                    >
                      {allocationData.map((entry, index) => (
                        <Cell
                          key={entry.label}
                          fill={
                            allocationColors[index % allocationColors.length]
                          }
                        />
                      ))}
                    </Pie>
                  </PieChart>
                </ResponsiveContainer>
              </div>
              <div className="max-h-44 space-y-3 overflow-y-auto pr-1">
                {allocationData.map((entry, index) => (
                  <div
                    key={entry.label}
                    className="flex items-center justify-between gap-4 text-sm"
                  >
                    <span className="flex min-w-0 items-center gap-2 text-text/90">
                      <span
                        className="size-2 shrink-0 rounded-full"
                        style={{
                          backgroundColor:
                            allocationColors[index % allocationColors.length],
                        }}
                      />
                      <span className="truncate">
                        {entry.label}
                        {entry.label !== 'Other' ? (
                          <span className="text-muted"> · {entry.assetType}</span>
                        ) : null}
                      </span>
                    </span>
                    <span className="shrink-0 text-xs text-muted">
                      {entry.percent.toFixed(1)}%
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </PlaceholderCard>
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
        <PlaceholderCard title="YTD return" description="Return chart coming later." />
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
