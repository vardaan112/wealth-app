import { PlaceholderCard } from '../components/PlaceholderCard'

export function TimelinePage() {
  return (
    <div className="mx-auto flex max-w-4xl flex-col gap-6">
      <header>
        <h1 className="text-2xl font-semibold tracking-tight">Timeline</h1>
        <p className="mt-1 text-sm text-muted">
          Milestones, goals, and projected events.
        </p>
      </header>
      <div className="flex flex-col gap-4">
        <PlaceholderCard
          title="Emergency fund complete"
          description="Target reached — Mar 2026."
        />
        <PlaceholderCard
          title="Mortgage payoff"
          description="Projected — Dec 2038."
        />
        <PlaceholderCard
          title="Retirement target"
          description="Age 55 — estimated 2042."
        />
      </div>
    </div>
  )
}
