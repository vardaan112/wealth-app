import type { ReactNode } from 'react'
import { PlaceholderCard } from '../components/PlaceholderCard'
import { useHoldings } from '../hooks/useHoldings'
import { useMonthlySummary } from '../hooks/useMonthlySummary'
import { useNetWorthTimeline } from '../hooks/useNetWorthTimeline'
import { useTransactions } from '../hooks/useTransactions'
import { formatMoney, formatPercent } from '../lib/format'
import type { Money, Transaction } from '../graphql/types'

function LoadingText() {
  return <p className="text-sm text-muted">Loading...</p>
}

function ErrorText({ message }: { message: string }) {
  return (
    <p className="rounded-2xl border border-red-400/20 bg-red-400/10 px-4 py-3 text-sm text-red-200">
      Could not load data. {message}
    </p>
  )
}

function EmptyText({ children }: { children: ReactNode }) {
  return <p className="text-sm text-muted">{children}</p>
}

function StatCard({
  label,
  value,
  detail,
  children,
}: {
  label: string
  value: ReactNode
  detail: string
  children?: ReactNode
}) {
  return (
    <div className="rounded-3xl border border-white/[0.06] bg-white/[0.035] p-4">
      <p className="text-xs uppercase tracking-[0.2em] text-muted">{label}</p>
      <p className="mt-3 text-2xl font-semibold tracking-tight text-text">
        {value}
      </p>
      <p className="mt-1 text-sm text-muted">{detail}</p>
      {children ? <div className="mt-4">{children}</div> : null}
    </div>
  )
}

function recentTransactions(transactions: Transaction[]): Transaction[] {
  return [...transactions]
    .sort((a, b) => b.transactionDate.localeCompare(a.transactionDate))
    .slice(0, 5)
}

function money(amountCents: number): Money {
  return { amountCents, currency: 'USD' }
}

function sumMoney(values: Money[]): Money {
  return money(values.reduce((total, value) => total + value.amountCents, 0))
}

export function HomePage() {
  const [timelineResult] = useNetWorthTimeline()
  const [summaryResult] = useMonthlySummary()
  const [transactionsResult] = useTransactions()
  const [holdingsResult] = useHoldings()

  const latestPoint = timelineResult.data?.netWorthTimeline.at(-1)
  const summary = summaryResult.data?.monthlySummary
  const transactions = transactionsResult.data?.transactions ?? []
  const holdings = holdingsResult.data?.holdings ?? []
  const recent = recentTransactions(transactions)
  const investmentValue = holdings.length
    ? sumMoney(holdings.map((holding) => holding.marketValue))
    : latestPoint?.investments

  const isLoading =
    timelineResult.fetching ||
    summaryResult.fetching ||
    transactionsResult.fetching ||
    holdingsResult.fetching

  const error =
    timelineResult.error ||
    summaryResult.error ||
    transactionsResult.error ||
    holdingsResult.error

  return (
    <div className="mx-auto flex max-w-6xl flex-col gap-5">
      <header className="flex flex-col justify-between gap-4 md:flex-row md:items-end">
        <div>
          <p className="text-xs font-medium uppercase tracking-[0.28em] text-accent">
            Wealth overview
          </p>
          <h1 className="mt-3 text-3xl font-semibold tracking-[-0.04em] text-text sm:text-4xl">
            Calm, clear finances.
          </h1>
          <p className="mt-3 max-w-2xl text-sm leading-6 text-muted">
            A quiet view of net worth, cash flow, investments, and recent
            activity from your GraphQL API.
          </p>
        </div>
        <div className="rounded-full border border-white/[0.08] bg-white/[0.04] px-4 py-2 text-xs text-muted">
          Mock GraphQL data
        </div>
      </header>

      {error ? <ErrorText message={error.message} /> : null}

      <section className="grid gap-4 lg:grid-cols-[1.25fr_0.75fr]">
        <div className="relative overflow-hidden rounded-[2rem] border border-accent/15 bg-[linear-gradient(135deg,rgba(124,156,255,0.16),rgba(16,19,23,0.9)_42%,rgba(255,255,255,0.045))] p-6 shadow-[0_28px_90px_rgba(0,0,0,0.38)] sm:p-8">
          <div className="pointer-events-none absolute -right-16 -top-16 size-48 rounded-full bg-accent/10 blur-3xl" />
          <div className="relative">
            <p className="text-xs uppercase tracking-[0.24em] text-muted">
              Net worth
            </p>
            {isLoading ? (
              <div className="mt-8">
                <LoadingText />
              </div>
            ) : latestPoint ? (
              <>
                <p className="mt-5 text-5xl font-semibold tracking-[-0.06em] text-text sm:text-6xl">
                  {formatMoney(latestPoint.netWorth)}
                </p>
                <p className="mt-4 max-w-lg text-sm leading-6 text-muted">
                  Assets, cash, and liabilities summarized from the latest
                  timeline point.
                </p>
              </>
            ) : (
              <div className="mt-8">
                <EmptyText>No net worth data yet.</EmptyText>
              </div>
            )}

            <div className="mt-8 grid gap-3 sm:grid-cols-3">
              <div className="rounded-2xl border border-white/[0.07] bg-background/35 p-4">
                <p className="text-xs text-muted">Cash</p>
                <p className="mt-2 text-lg font-medium text-text">
                  {latestPoint ? formatMoney(latestPoint.cash) : '-'}
                </p>
              </div>
              <div className="rounded-2xl border border-white/[0.07] bg-background/35 p-4">
                <p className="text-xs text-muted">Investments</p>
                <p className="mt-2 text-lg font-medium text-text">
                  {investmentValue ? formatMoney(investmentValue) : '-'}
                </p>
              </div>
              <div className="rounded-2xl border border-white/[0.07] bg-background/35 p-4">
                <p className="text-xs text-muted">Debt</p>
                <p className="mt-2 text-lg font-medium text-text">
                  {latestPoint ? formatMoney(latestPoint.debt) : '-'}
                </p>
              </div>
            </div>
          </div>
        </div>

        <PlaceholderCard
          title="Monthly pulse"
          description="Income, spending, and savings for the current month."
          className="flex flex-col justify-between"
        >
          {isLoading ? (
            <LoadingText />
          ) : summary ? (
            <div className="grid gap-3">
              <StatCard
                label="Income"
                value={formatMoney(summary.income)}
                detail="Total inflows"
              />
              <StatCard
                label="Spending"
                value={formatMoney(summary.expenses)}
                detail="Total outflows"
              />
              <StatCard
                label="Savings"
                value={formatPercent(summary.savingsRate)}
                detail={`${formatMoney(summary.net)} retained`}
              />
            </div>
          ) : (
            <EmptyText>No monthly summary yet.</EmptyText>
          )}
        </PlaceholderCard>
      </section>

      <section className="grid gap-4 lg:grid-cols-[0.8fr_1.2fr]">
        <PlaceholderCard
          title="Category focus"
          description="Where spending is concentrated this month."
        >
          {isLoading ? (
            <LoadingText />
          ) : summary?.categorySpend.length ? (
            <div className="space-y-4">
              {summary.categorySpend.slice(0, 4).map((item) => (
                <div key={item.category}>
                  <div className="flex items-center justify-between gap-4 text-sm">
                    <span className="text-text">{item.category}</span>
                    <span className="text-muted">{formatMoney(item.amount)}</span>
                  </div>
                  <div className="mt-2 h-1.5 overflow-hidden rounded-full bg-white/[0.06]">
                    <div
                      className="h-full rounded-full bg-accent/80"
                      style={{ width: `${Math.min(item.percent, 100)}%` }}
                    />
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <EmptyText>No category data yet.</EmptyText>
          )}
        </PlaceholderCard>

        <PlaceholderCard
          title="Recent activity"
          description="The latest transactions from GraphQL."
        >
          {isLoading ? (
            <LoadingText />
          ) : recent.length === 0 ? (
            <EmptyText>No recent transactions.</EmptyText>
          ) : (
            <ul className="divide-y divide-white/[0.06]">
              {recent.map((txn) => (
                <li
                  key={txn.id}
                  className="flex items-center justify-between gap-4 py-4 first:pt-0 last:pb-0"
                >
                  <div className="flex min-w-0 items-center gap-3">
                    <div className="grid size-10 shrink-0 place-items-center rounded-2xl bg-white/[0.05] text-xs font-medium text-accent">
                      {txn.merchantName.slice(0, 1)}
                    </div>
                    <div className="min-w-0">
                      <p className="truncate text-sm font-medium text-text">
                        {txn.merchantName}
                      </p>
                      <p className="mt-0.5 text-xs text-muted">
                        {txn.categoryPrimary} / {txn.transactionDate}
                        {txn.pending ? ' / Pending' : ''}
                      </p>
                    </div>
                  </div>
                  <p
                    className={`shrink-0 text-sm font-medium ${
                      txn.amount.amountCents >= 0
                        ? 'text-accent'
                        : 'text-text'
                    }`}
                  >
                    {formatMoney(txn.amount)}
                  </p>
                </li>
              ))}
            </ul>
          )}
        </PlaceholderCard>
      </section>
    </div>
  )
}
