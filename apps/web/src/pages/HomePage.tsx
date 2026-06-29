import type { ReactNode } from 'react'
import { Area, AreaChart, ResponsiveContainer, Tooltip } from 'recharts'
import { useHoldings } from '../hooks/useHoldings'
import { useMonthlySummary } from '../hooks/useMonthlySummary'
import { useNetWorthTimeline } from '../hooks/useNetWorthTimeline'
import { useTransactions } from '../hooks/useTransactions'
import { formatMoney, formatPercent } from '../lib/format'
import type { Money, NetWorthPoint, Transaction } from '../graphql/types'

function money(amountCents: number): Money {
  return { amountCents, currency: 'USD' }
}

function sumMoney(values: Money[]): Money {
  return money(values.reduce((total, value) => total + value.amountCents, 0))
}

function recentTransactions(transactions: Transaction[]): Transaction[] {
  return [...transactions]
    .sort((a, b) => b.transactionDate.localeCompare(a.transactionDate))
    .slice(0, 5)
}

function trendDelta(points: NetWorthPoint[]): number | null {
  if (points.length < 2) return null
  const first = points[0].netWorth.amountCents
  const last = points[points.length - 1].netWorth.amountCents
  if (first === 0) return null
  return ((last - first) / Math.abs(first)) * 100
}

function Stat({
  label,
  value,
  hint,
}: {
  label: string
  value: ReactNode
  hint?: string
}) {
  return (
    <div>
      <p className="text-[0.7rem] uppercase tracking-[0.18em] text-muted">
        {label}
      </p>
      <p className="mt-2 text-xl font-medium tracking-tight text-text">
        {value}
      </p>
      {hint ? <p className="mt-1 text-xs text-muted">{hint}</p> : null}
    </div>
  )
}

export function HomePage() {
  const [timelineResult] = useNetWorthTimeline()
  const [summaryResult] = useMonthlySummary()
  const [transactionsResult] = useTransactions()
  const [holdingsResult] = useHoldings()

  const timeline = timelineResult.data?.netWorthTimeline ?? []
  const latestPoint = timeline.at(-1)
  const summary = summaryResult.data?.monthlySummary
  const transactions = transactionsResult.data?.transactions ?? []
  const holdings = holdingsResult.data?.holdings ?? []
  const recent = recentTransactions(transactions)
  const investmentValue = holdings.length
    ? sumMoney(holdings.map((holding) => holding.marketValue))
    : latestPoint?.investments
  const delta = trendDelta(timeline)
  const chartData = timeline.map((point) => ({
    date: point.date,
    value: point.netWorth.amountCents / 100,
  }))

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

  if (error) {
    return (
      <div className="mx-auto max-w-5xl pt-10">
        <p className="text-sm text-muted">
          We couldn&rsquo;t reach your data right now.
        </p>
        <p className="mt-2 text-xs text-muted/70">{error.message}</p>
      </div>
    )
  }

  return (
    <div className="mx-auto flex max-w-5xl flex-col gap-14">
      <section className="animate-rise pt-6">
        <p className="text-[0.7rem] uppercase tracking-[0.28em] text-muted">
          Total net worth
        </p>

        <div className="mt-3 flex flex-wrap items-end gap-x-5 gap-y-2">
          <h1 className="text-5xl font-semibold tracking-[-0.045em] text-text sm:text-7xl">
            {isLoading
              ? '\u2014'
              : latestPoint
                ? formatMoney(latestPoint.netWorth)
                : '\u2014'}
          </h1>
          {delta !== null ? (
            <span
              className={`mb-2 text-sm font-medium ${
                delta >= 0 ? 'text-accent' : 'text-muted'
              }`}
            >
              {delta >= 0 ? '\u2191' : '\u2193'} {Math.abs(delta).toFixed(1)}%
              <span className="ml-1 text-muted">over period</span>
            </span>
          ) : null}
        </div>

        <div className="mt-6 h-40 w-full sm:h-52">
          {chartData.length > 1 ? (
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart
                data={chartData}
                margin={{ top: 8, right: 0, bottom: 0, left: 0 }}
              >
                <defs>
                  <linearGradient id="netWorthFill" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="#7c9cff" stopOpacity={0.32} />
                    <stop offset="100%" stopColor="#7c9cff" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <Tooltip
                  cursor={{ stroke: 'rgba(124,156,255,0.3)', strokeWidth: 1 }}
                  contentStyle={{
                    background: 'rgba(16,19,23,0.92)',
                    border: '1px solid rgba(255,255,255,0.08)',
                    borderRadius: 14,
                    fontSize: 12,
                    color: '#f4f6f8',
                    backdropFilter: 'blur(8px)',
                  }}
                  labelStyle={{ color: '#8b95a5' }}
                  formatter={(value) => [
                    formatMoney(money(Math.round(Number(value) * 100))),
                    'Net worth',
                  ]}
                />
                <Area
                  type="monotone"
                  dataKey="value"
                  stroke="#7c9cff"
                  strokeWidth={2}
                  fill="url(#netWorthFill)"
                  animationDuration={900}
                  animationEasing="ease-out"
                />
              </AreaChart>
            </ResponsiveContainer>
          ) : (
            <div className="h-full w-full rounded-2xl bg-white/[0.02]" />
          )}
        </div>

        <div className="mt-2 grid grid-cols-3 gap-px overflow-hidden rounded-2xl bg-white/[0.05]">
          {[
            { label: 'Cash', value: latestPoint?.cash },
            { label: 'Investments', value: investmentValue },
            { label: 'Debt', value: latestPoint?.debt },
          ].map((item) => (
            <div key={item.label} className="bg-background/40 px-4 py-4">
              <p className="text-[0.7rem] uppercase tracking-[0.16em] text-muted">
                {item.label}
              </p>
              <p className="mt-2 text-base font-medium tracking-tight text-text">
                {item.value ? formatMoney(item.value) : '\u2014'}
              </p>
            </div>
          ))}
        </div>
      </section>

      <section className="animate-rise [animation-delay:90ms]">
        <div className="flex items-baseline justify-between">
          <h2 className="text-sm font-medium tracking-tight text-text/90">
            This month
          </h2>
          {summary ? (
            <span className="text-xs text-muted">{summary.month}</span>
          ) : null}
        </div>
        <div className="mt-5 grid grid-cols-2 gap-x-6 gap-y-8 sm:grid-cols-4">
          <Stat
            label="Income"
            value={summary ? formatMoney(summary.income) : '\u2014'}
            hint="Inflows"
          />
          <Stat
            label="Spending"
            value={summary ? formatMoney(summary.expenses) : '\u2014'}
            hint="Outflows"
          />
          <Stat
            label="Saved"
            value={summary ? formatMoney(summary.net) : '\u2014'}
            hint="Net retained"
          />
          <Stat
            label="Savings rate"
            value={summary ? formatPercent(summary.savingsRate) : '\u2014'}
            hint="Of income"
          />
        </div>
      </section>

      <section className="animate-rise grid gap-12 [animation-delay:160ms] lg:grid-cols-[0.85fr_1.15fr]">
        <div>
          <h2 className="text-sm font-medium tracking-tight text-text/90">
            Where it goes
          </h2>
          <div className="mt-6 space-y-5">
            {summary?.categorySpend.length ? (
              summary.categorySpend.slice(0, 5).map((item) => (
                <div key={item.category}>
                  <div className="flex items-baseline justify-between text-sm">
                    <span className="text-text/90">{item.category}</span>
                    <span className="text-xs text-muted">
                      {formatMoney(item.amount)}
                    </span>
                  </div>
                  <div className="mt-2 h-px w-full bg-white/[0.06]">
                    <div
                      className="h-px bg-accent/70"
                      style={{ width: `${Math.min(item.percent, 100)}%` }}
                    />
                  </div>
                </div>
              ))
            ) : (
              <p className="text-sm text-muted">Nothing to show yet.</p>
            )}
          </div>
        </div>

        <div>
          <h2 className="text-sm font-medium tracking-tight text-text/90">
            Recent activity
          </h2>
          <ul className="mt-4 divide-y divide-white/[0.05]">
            {recent.length === 0 ? (
              <li className="py-4 text-sm text-muted">No recent transactions.</li>
            ) : (
              recent.map((txn) => (
                <li
                  key={txn.id}
                  className="flex items-center justify-between gap-4 py-4"
                >
                  <div className="min-w-0">
                    <p className="truncate text-sm text-text/90">
                      {txn.merchantName}
                    </p>
                    <p className="mt-0.5 text-xs text-muted">
                      {txn.categoryPrimary} &middot; {txn.transactionDate}
                      {txn.pending ? ' \u00b7 pending' : ''}
                    </p>
                  </div>
                  <p
                    className={`shrink-0 text-sm tabular-nums ${
                      txn.amount.amountCents >= 0 ? 'text-accent' : 'text-text/90'
                    }`}
                  >
                    {formatMoney(txn.amount)}
                  </p>
                </li>
              ))
            )}
          </ul>
        </div>
      </section>
    </div>
  )
}
