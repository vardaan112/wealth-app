import {
  Area,
  AreaChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
} from 'recharts'

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
    <div className="animate-rise mx-auto flex max-w-5xl flex-col gap-8 pt-6">
      <header>
        <h1 className="text-3xl font-semibold tracking-[-0.04em]">Cash Flow</h1>
        <p className="mt-2 text-sm text-muted">
          Income, expenses, and surplus over time.
        </p>
      </header>
      <div className="h-60 w-full">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={chartData} margin={{ top: 8, right: 0, bottom: 0, left: 0 }}>
            <defs>
              <linearGradient id="incomeFill" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor="#7c9cff" stopOpacity={0.28} />
                <stop offset="100%" stopColor="#7c9cff" stopOpacity={0} />
              </linearGradient>
              <linearGradient id="expensesFill" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor="#8b95a5" stopOpacity={0.18} />
                <stop offset="100%" stopColor="#8b95a5" stopOpacity={0} />
              </linearGradient>
            </defs>
            <XAxis
              dataKey="month"
              stroke="#8B95A5"
              fontSize={12}
              tickLine={false}
              axisLine={false}
            />
            <Tooltip
              cursor={{ stroke: 'rgba(124,156,255,0.3)', strokeWidth: 1 }}
              contentStyle={{
                background: 'rgba(16,19,23,0.92)',
                border: '1px solid rgba(255,255,255,0.08)',
                borderRadius: 14,
                fontSize: 12,
                color: '#F4F6F8',
                backdropFilter: 'blur(8px)',
              }}
            />
            <Area
              type="monotone"
              dataKey="income"
              stroke="#7C9CFF"
              fill="url(#incomeFill)"
              strokeWidth={2}
            />
            <Area
              type="monotone"
              dataKey="expenses"
              stroke="#8B95A5"
              fill="url(#expensesFill)"
              strokeWidth={2}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>
      <div className="grid grid-cols-3 gap-x-6 border-t border-white/[0.06] pt-6">
        <div>
          <p className="text-[0.7rem] uppercase tracking-[0.16em] text-muted">Income</p>
          <p className="mt-2 text-lg font-medium tracking-tight">$8,400</p>
          <p className="text-xs text-muted">avg / month</p>
        </div>
        <div>
          <p className="text-[0.7rem] uppercase tracking-[0.16em] text-muted">Expenses</p>
          <p className="mt-2 text-lg font-medium tracking-tight">$5,333</p>
          <p className="text-xs text-muted">avg / month</p>
        </div>
        <div>
          <p className="text-[0.7rem] uppercase tracking-[0.16em] text-muted">Surplus</p>
          <p className="mt-2 text-lg font-medium tracking-tight">$3,067</p>
          <p className="text-xs text-muted">avg / month</p>
        </div>
      </div>
    </div>
  )
}
