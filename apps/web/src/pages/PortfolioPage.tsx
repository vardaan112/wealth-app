import { PlaceholderCard } from '../components/PlaceholderCard'

export function PortfolioPage() {
  return (
    <div className="mx-auto flex max-w-4xl flex-col gap-6">
      <header>
        <h1 className="text-2xl font-semibold tracking-tight">Portfolio</h1>
        <p className="mt-1 text-sm text-muted">
          Holdings, allocation, and performance.
        </p>
      </header>
      <div className="grid gap-4 sm:grid-cols-2">
        <PlaceholderCard
          title="Total value"
          description="$124,500 across all accounts."
        />
        <PlaceholderCard
          title="Allocation"
          description="Stocks 62% · Bonds 28% · Cash 10%."
        />
        <PlaceholderCard
          title="YTD return"
          description="+8.2% vs benchmark +6.1%."
        />
        <PlaceholderCard
          title="Top holdings"
          description="VTI, BND, VXUS — placeholder list."
        />
      </div>
    </div>
  )
}
