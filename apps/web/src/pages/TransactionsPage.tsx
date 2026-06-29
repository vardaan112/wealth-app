import { PlaceholderCard } from '../components/PlaceholderCard'

export function TransactionsPage() {
  return (
    <div className="mx-auto flex max-w-4xl flex-col gap-6">
      <header>
        <h1 className="text-2xl font-semibold tracking-tight">Transactions</h1>
        <p className="mt-1 text-sm text-muted">
          Recent debits, credits, and transfers.
        </p>
      </header>
      <PlaceholderCard
        title="Recent transactions"
        description="Placeholder list — no data connected yet."
      >
        <ul className="divide-y divide-border text-sm">
          {[
            { date: 'Jun 27', payee: 'Grocery Store', amount: '-$84.20' },
            { date: 'Jun 26', payee: 'Payroll Deposit', amount: '+$4,100.00' },
            { date: 'Jun 25', payee: 'Electric Utility', amount: '-$112.45' },
            { date: 'Jun 24', payee: 'Brokerage Transfer', amount: '-$500.00' },
          ].map((tx) => (
            <li
              key={`${tx.date}-${tx.payee}`}
              className="flex items-center justify-between py-3 first:pt-0 last:pb-0"
            >
              <div>
                <p className="font-medium text-text">{tx.payee}</p>
                <p className="text-xs text-muted">{tx.date}</p>
              </div>
              <span className="tabular-nums text-text">{tx.amount}</span>
            </li>
          ))}
        </ul>
      </PlaceholderCard>
    </div>
  )
}
