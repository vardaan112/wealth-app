import {
  Bar,
  BarChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { useQuery } from 'urql'
import type { CategorySpend, Money } from '../graphql/types'
import { chartLabelStyle, chartTooltipStyle, formatCents } from '../lib/chart'
import { formatMoney } from '../lib/format'

const CASH_FLOW_SUMMARIES_QUERY = `
  query CashFlowSummaries(
    $m0: String!
    $m1: String!
    $m2: String!
    $m3: String!
    $m4: String!
    $m5: String!
  ) {
    m0: monthlySummary(month: $m0) {
      ...MonthlyCashFlow
    }
    m1: monthlySummary(month: $m1) {
      ...MonthlyCashFlow
    }
    m2: monthlySummary(month: $m2) {
      ...MonthlyCashFlow
    }
    m3: monthlySummary(month: $m3) {
      ...MonthlyCashFlow
    }
    m4: monthlySummary(month: $m4) {
      ...MonthlyCashFlow
    }
    m5: monthlySummary(month: $m5) {
      ...MonthlyCashFlow
    }
  }

  fragment MonthlyCashFlow on MonthlySummary {
    month
    income {
      amountCents
      currency
    }
    expenses {
      amountCents
      currency
    }
    net {
      amountCents
      currency
    }
    categorySpend {
      category
      amount {
        amountCents
        currency
      }
      percent
    }
  }
`

type MonthlyCashFlow = {
  month: string
  income: Money
  expenses: Money
  net: Money
  categorySpend: CategorySpend[]
}

type CashFlowSummariesQuery = {
  m0: MonthlyCashFlow
  m1: MonthlyCashFlow
  m2: MonthlyCashFlow
  m3: MonthlyCashFlow
  m4: MonthlyCashFlow
  m5: MonthlyCashFlow
}

function lastSixMonths(): string[] {
  const months: string[] = []
  const cursor = new Date()

  for (let index = 5; index >= 0; index -= 1) {
    const date = new Date(cursor.getFullYear(), cursor.getMonth() - index, 1)
    const month = String(date.getMonth() + 1).padStart(2, '0')
    months.push(`${date.getFullYear()}-${month}`)
  }

  return months
}

export function CashFlowPage() {
  const months = lastSixMonths()
  const [result] = useQuery<CashFlowSummariesQuery>({
    query: CASH_FLOW_SUMMARIES_QUERY,
    variables: {
      m0: months[0],
      m1: months[1],
      m2: months[2],
      m3: months[3],
      m4: months[4],
      m5: months[5],
    },
  })

  const summaries = result.data
    ? [result.data.m0, result.data.m1, result.data.m2, result.data.m3, result.data.m4, result.data.m5]
    : []

  const chartData = summaries.map((summary) => ({
    month: summary.month.slice(5),
    income: summary.income.amountCents,
    spending: summary.expenses.amountCents,
  }))

  const latestSummary = summaries.at(-1)
  const categoryData =
    latestSummary?.categorySpend.slice(0, 6).map((category) => ({
      category: category.category,
      amount: category.amount.amountCents,
    })) ?? []

  return (
    <div className="animate-rise mx-auto flex max-w-5xl flex-col gap-8 pt-6">
      <header>
        <h1 className="text-3xl font-semibold tracking-[-0.04em]">Cash Flow</h1>
        <p className="mt-2 text-sm text-muted">
          Income, expenses, and surplus over time.
        </p>
      </header>
      {result.error ? (
        <p className="text-sm text-red-300">{result.error.message}</p>
      ) : null}
      <section>
        <div className="flex items-baseline justify-between">
          <h2 className="text-sm font-medium tracking-tight text-text/90">
            Income vs spending
          </h2>
          {result.fetching ? (
            <span className="text-xs text-muted">Loading...</span>
          ) : null}
        </div>
        <div className="mt-5 h-64 w-full sm:h-80">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart
              data={chartData}
              margin={{ top: 8, right: 0, bottom: 0, left: 0 }}
              barGap={6}
            >
              <XAxis
                dataKey="month"
                stroke="#8b95a5"
                fontSize={12}
                tickLine={false}
                axisLine={false}
              />
              <YAxis hide />
              <Tooltip
                cursor={{ fill: 'rgba(255,255,255,0.025)' }}
                contentStyle={chartTooltipStyle}
                labelStyle={chartLabelStyle}
                formatter={(value) => formatCents(Number(value))}
              />
              <Bar
                dataKey="income"
                name="Income"
                fill="#7c9cff"
                radius={[8, 8, 0, 0]}
              />
              <Bar
                dataKey="spending"
                name="Spending"
                fill="#8b95a5"
                radius={[8, 8, 0, 0]}
              />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </section>
      <section className="grid gap-10 lg:grid-cols-[0.9fr_1.1fr]">
        <div>
          <h2 className="text-sm font-medium tracking-tight text-text/90">
            Category spending
          </h2>
          <div className="mt-5 h-64 w-full">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart
                data={categoryData}
                layout="vertical"
                margin={{ top: 0, right: 8, bottom: 0, left: 0 }}
              >
                <XAxis type="number" hide />
                <YAxis
                  type="category"
                  dataKey="category"
                  width={92}
                  tickLine={false}
                  axisLine={false}
                  stroke="#8b95a5"
                  fontSize={12}
                />
                <Tooltip
                  cursor={{ fill: 'rgba(255,255,255,0.025)' }}
                  contentStyle={chartTooltipStyle}
                  labelStyle={chartLabelStyle}
                  formatter={(value) => formatCents(Number(value))}
                />
                <Bar
                  dataKey="amount"
                  name="Spending"
                  fill="#7c9cff"
                  radius={[0, 8, 8, 0]}
                  barSize={12}
                />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </div>
        <div className="grid grid-cols-3 gap-x-6 border-t border-white/[0.06] pt-6 lg:self-end">
        <div>
          <p className="text-[0.7rem] uppercase tracking-[0.16em] text-muted">Income</p>
          <p className="mt-2 text-lg font-medium tracking-tight">
            {latestSummary ? formatMoney(latestSummary.income) : '\u2014'}
          </p>
          <p className="text-xs text-muted">latest month</p>
        </div>
        <div>
          <p className="text-[0.7rem] uppercase tracking-[0.16em] text-muted">Expenses</p>
          <p className="mt-2 text-lg font-medium tracking-tight">
            {latestSummary ? formatMoney(latestSummary.expenses) : '\u2014'}
          </p>
          <p className="text-xs text-muted">latest month</p>
        </div>
        <div>
          <p className="text-[0.7rem] uppercase tracking-[0.16em] text-muted">Surplus</p>
          <p className="mt-2 text-lg font-medium tracking-tight">
            {latestSummary ? formatMoney(latestSummary.net) : '\u2014'}
          </p>
          <p className="text-xs text-muted">latest month</p>
        </div>
      </div>
      </section>
    </div>
  )
}
