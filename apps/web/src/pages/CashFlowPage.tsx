import {
  Area,
  AreaChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { PlaceholderCard } from '../components/PlaceholderCard'

const chartData = [
  { month: 'Jan', income: 8200, expenses: 5400 },
  { month: 'Feb', income: 8200, expenses: 5100 },
  { month: 'Mar', income: 8500, expenses: 5800 },
  { month: 'Apr', income: 8200, expenses: 4900 },
  { month: 'May', income: 8400, expenses: 5200 },
  { month: 'Jun', income: 8200, expenses: 5600 },
]

export function CashFlowPage() {
  return (
    <div className="mx-auto flex max-w-4xl flex-col gap-6">
      <header>
        <h1 className="text-2xl font-semibold tracking-tight">Cash Flow</h1>
        <p className="mt-1 text-sm text-muted">
          Income, expenses, and surplus over time.
        </p>
      </header>
      <PlaceholderCard
        title="Monthly trend"
        description="Placeholder chart — connect to GraphQL later."
      >
        <div className="h-56 w-full">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={chartData}>
              <CartesianGrid stroke="#252B33" strokeDasharray="3 3" />
              <XAxis dataKey="month" stroke="#8B95A5" fontSize={12} />
              <YAxis stroke="#8B95A5" fontSize={12} tickFormatter={(v) => `$${v}`} />
              <Tooltip
                contentStyle={{
                  backgroundColor: '#101317',
                  border: '1px solid #252B33',
                  borderRadius: '8px',
                  color: '#F4F6F8',
                }}
              />
              <Area
                type="monotone"
                dataKey="income"
                stroke="#7C9CFF"
                fill="#7C9CFF"
                fillOpacity={0.15}
                strokeWidth={2}
              />
              <Area
                type="monotone"
                dataKey="expenses"
                stroke="#8B95A5"
                fill="#8B95A5"
                fillOpacity={0.1}
                strokeWidth={2}
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </PlaceholderCard>
      <div className="grid gap-4 sm:grid-cols-3">
        <PlaceholderCard title="Income" description="$8,400 avg / month" />
        <PlaceholderCard title="Expenses" description="$5,333 avg / month" />
        <PlaceholderCard title="Surplus" description="$3,067 avg / month" />
      </div>
    </div>
  )
}
