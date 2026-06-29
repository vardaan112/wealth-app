import type { ReactNode } from 'react'
import { PlaceholderCard } from '../components/PlaceholderCard'
import { useMonthlySummary } from '../hooks/useMonthlySummary'
import { useNetWorthTimeline } from '../hooks/useNetWorthTimeline'
import { useTransactions } from '../hooks/useTransactions'
import { formatMoney, formatPercent } from '../lib/format'
import type { Transaction } from '../graphql/types'

function LoadingText() {
  return <p className="mt-4 text-sm text-muted">Loading…</p>
}

function ErrorText({ message }: { message: string }) {
  return (
    <p className="mt-4 text-sm text-red-400">
      Could not load data. {message}
    </p>
  )
}

function StatValue({ children }: { children: ReactNode }) {
  return <p className="mt-4 text-2xl font-semibold text-text">{children}</p>
}

function recentTransactions(transactions: Transaction[]): Transaction[] {
  return [...transactions]
    .sort((a, b) => b.transactionDate.localeCompare(a.transactionDate))
    .slice(0, 5)
}

export function HomePage() {
  const [timelineResult] = useNetWorthTimeline()
  const [summaryResult] = useMonthlySummary()
  const [transactionsResult] = useTransactions()

  const latestPoint = timelineResult.data?.netWorthTimeline.at(-1)
  const summary = summaryResult.data?.monthlySummary
  const transactions = transactionsResult.data?.transactions ?? []
  const recent = recentTransactions(transactions)

  return (
    <div className="mx-auto flex max-w-4xl flex-col gap-6">
      <header>
        <h1 className="text-2xl font-semibold tracking-tight">Home</h1>
        <p className="mt-1 text-sm text-muted">
          Overview of your financial picture.
        </p>
      </header>

      <div className="grid gap-4 sm:grid-cols-2">
        <PlaceholderCard
          title="Net worth"
          description="Total assets minus liabilities."
        >
          {timelineResult.fetching ? (
            <LoadingText />
          ) : timelineResult.error ? (
            <ErrorText message={timelineResult.error.message} />
          ) : latestPoint ? (
            <StatValue>{formatMoney(latestPoint.netWorth)}</StatValue>
          ) : (
            <p className="mt-4 text-sm text-muted">No net worth data yet.</p>
          )}
        </PlaceholderCard>

        <PlaceholderCard
          title="Monthly income"
          description="Total income this month."
        >
          {summaryResult.fetching ? (
            <LoadingText />
          ) : summaryResult.error ? (
            <ErrorText message={summaryResult.error.message} />
          ) : summary ? (
            <StatValue>{formatMoney(summary.income)}</StatValue>
          ) : (
            <p className="mt-4 text-sm text-muted">No income data yet.</p>
          )}
        </PlaceholderCard>

        <PlaceholderCard
          title="Monthly spending"
          description="Total expenses this month."
        >
          {summaryResult.fetching ? (
            <LoadingText />
          ) : summaryResult.error ? (
            <ErrorText message={summaryResult.error.message} />
          ) : summary ? (
            <StatValue>{formatMoney(summary.expenses)}</StatValue>
          ) : (
            <p className="mt-4 text-sm text-muted">No spending data yet.</p>
          )}
        </PlaceholderCard>

        <PlaceholderCard
          title="Savings rate"
          description="Share of income saved this month."
        >
          {summaryResult.fetching ? (
            <LoadingText />
          ) : summaryResult.error ? (
            <ErrorText message={summaryResult.error.message} />
          ) : summary ? (
            <StatValue>{formatPercent(summary.savingsRate)}</StatValue>
          ) : (
            <p className="mt-4 text-sm text-muted">No savings data yet.</p>
          )}
        </PlaceholderCard>

        <PlaceholderCard
          title="Investment value"
          description="Current investment portfolio value."
        >
          {timelineResult.fetching ? (
            <LoadingText />
          ) : timelineResult.error ? (
            <ErrorText message={timelineResult.error.message} />
          ) : latestPoint ? (
            <StatValue>{formatMoney(latestPoint.investments)}</StatValue>
          ) : (
            <p className="mt-4 text-sm text-muted">No investment data yet.</p>
          )}
        </PlaceholderCard>

        <PlaceholderCard
          title="Recent activity"
          description="Latest transactions."
        >
          {transactionsResult.fetching ? (
            <LoadingText />
          ) : transactionsResult.error ? (
            <ErrorText message={transactionsResult.error.message} />
          ) : recent.length === 0 ? (
            <p className="mt-4 text-sm text-muted">No recent transactions.</p>
          ) : (
            <ul className="mt-4 divide-y divide-border">
              {recent.map((txn) => (
                <li
                  key={txn.id}
                  className="flex items-center justify-between py-3 first:pt-0 last:pb-0"
                >
                  <div className="min-w-0">
                    <p className="truncate text-sm font-medium text-text">
                      {txn.merchantName}
                    </p>
                    <p className="text-xs text-muted">
                      {txn.transactionDate}
                      {txn.pending ? ' · Pending' : ''}
                    </p>
                  </div>
                  <p
                    className={`ml-3 shrink-0 text-sm font-medium ${
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
      </div>
    </div>
  )
}
