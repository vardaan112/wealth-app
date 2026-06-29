import { PlaceholderCard } from '../components/PlaceholderCard'

export function HomePage() {
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
        />
        <PlaceholderCard
          title="Monthly surplus"
          description="Income minus expenses this month."
        />
        <PlaceholderCard
          title="Upcoming bills"
          description="Scheduled payments in the next 30 days."
        />
        <PlaceholderCard
          title="Recent activity"
          description="Latest transactions and account changes."
        />
      </div>
    </div>
  )
}
