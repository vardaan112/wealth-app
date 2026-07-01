import { Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts'
import { useNetWorthTimeline } from '../hooks/useNetWorthTimeline'
import {
  centsToChartValue,
  chartLabelStyle,
  chartTooltipStyle,
  formatChartAxis,
  formatChartTooltip,
} from '../lib/chart'

export function TimelinePage() {
  const [timelineResult] = useNetWorthTimeline()
  const timeline = timelineResult.data?.netWorthTimeline ?? []
  const chartData = timeline.map((point) => ({
    date: point.date,
    netWorth: centsToChartValue(point.netWorth.amountCents),
  }))

  const events = [
    {
      title: 'Emergency fund complete',
      detail: 'Target reached',
      when: 'Mar 2026',
      done: true,
    },
    {
      title: 'Mortgage payoff',
      detail: 'Projected',
      when: 'Dec 2038',
      done: false,
    },
    {
      title: 'Retirement target',
      detail: 'Age 55, estimated',
      when: '2042',
      done: false,
    },
  ]

  return (
    <div className="animate-rise mx-auto flex max-w-5xl flex-col gap-8 pt-6">
      <header>
        <h1 className="text-3xl font-semibold tracking-[-0.04em]">Timeline</h1>
        <p className="mt-2 text-sm text-muted">
          Milestones, goals, and projected events.
        </p>
      </header>
      <section>
        <div className="flex items-baseline justify-between">
          <h2 className="text-sm font-medium tracking-tight text-text/90">
            Net worth path
          </h2>
          {timelineResult.fetching ? (
            <span className="text-xs text-muted">Loading...</span>
          ) : null}
        </div>
        <div className="mt-5 h-56 w-full sm:h-72">
          {chartData.length === 0 ? (
            <div className="grid h-full place-items-center rounded-3xl bg-white/[0.025] text-sm text-muted">
              No timeline data yet.
            </div>
          ) : (
            <ResponsiveContainer width="100%" height="100%">
              <LineChart
                data={chartData}
                margin={{ top: 8, right: 8, bottom: 0, left: 0 }}
              >
                <XAxis
                  dataKey="date"
                  stroke="#8b95a5"
                  fontSize={12}
                  tickLine={false}
                  axisLine={false}
                  minTickGap={24}
                />
                <Tooltip
                  cursor={{ stroke: 'rgba(124,156,255,0.3)', strokeWidth: 1 }}
                  contentStyle={chartTooltipStyle}
                  labelStyle={chartLabelStyle}
                  formatter={(value) => [formatChartTooltip(Number(value)), 'Net worth']}
                />
                <YAxis hide domain={['auto', 'auto']} tickFormatter={formatChartAxis} />
                <Line
                  type="monotone"
                  dataKey="netWorth"
                  stroke="#7c9cff"
                  strokeWidth={2.5}
                  dot={{ r: 2, fill: '#7c9cff', strokeWidth: 0 }}
                  activeDot={{ r: 4, fill: '#7c9cff', strokeWidth: 0 }}
                />
              </LineChart>
            </ResponsiveContainer>
          )}
        </div>
      </section>
      <ol className="relative ml-1 border-l border-white/[0.07]">
        {events.map((event) => (
          <li key={event.title} className="relative pb-8 pl-6 last:pb-0">
            <span
              className={`absolute -left-[5px] top-1.5 size-2.5 rounded-full ${
                event.done ? 'bg-accent' : 'bg-white/20'
              }`}
            />
            <p className="text-sm text-text/90">{event.title}</p>
            <p className="mt-1 text-xs text-muted">
              {event.detail} &middot; {event.when}
            </p>
          </li>
        ))}
      </ol>
    </div>
  )
}
