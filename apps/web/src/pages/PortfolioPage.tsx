import { PlaceholderCard } from '../components/PlaceholderCard'

export function PortfolioPage() {
  return (
    <div className="animate-rise mx-auto flex max-w-5xl flex-col gap-8 pt-6">
      <header>
        <h1 className="text-3xl font-semibold tracking-[-0.04em]">Portfolio</h1>
        <p className="mt-2 text-sm text-muted">
          Holdings, allocation, and performance.
        </p>
      </header>
      <div className="grid gap-3 sm:grid-cols-2">
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
