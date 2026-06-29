export function TransactionsPage() {
  return (
    <div className="animate-rise mx-auto flex max-w-5xl flex-col gap-8 pt-6">
      <header>
        <h1 className="text-3xl font-semibold tracking-[-0.04em]">Transactions</h1>
        <p className="mt-2 text-sm text-muted">
          Recent debits, credits, and transfers.
        </p>
      </header>
      <ul className="divide-y divide-white/[0.05] text-sm">
        {[
          { date: 'Jun 27', payee: 'Grocery Store', amount: '-$84.20' },
          { date: 'Jun 26', payee: 'Payroll Deposit', amount: '+$4,100.00' },
          { date: 'Jun 25', payee: 'Electric Utility', amount: '-$112.45' },
          { date: 'Jun 24', payee: 'Brokerage Transfer', amount: '-$500.00' },
        ].map((tx) => (
          <li
            key={`${tx.date}-${tx.payee}`}
            className="flex items-center justify-between py-4"
          >
            <div>
              <p className="text-text/90">{tx.payee}</p>
              <p className="mt-0.5 text-xs text-muted">{tx.date}</p>
            </div>
            <span className="tabular-nums text-text/90">{tx.amount}</span>
          </li>
        ))}
      </ul>
    </div>
  )
}
