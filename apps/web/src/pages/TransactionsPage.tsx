import { useMemo, useState } from 'react'
import { TransactionForm } from '../components/forms/TransactionForm'
import { SelectInput } from '../components/FormControls'
import { Modal } from '../components/Modal'
import { useAccounts } from '../hooks/useAccounts'
import { useTransactions } from '../hooks/useTransactions'
import { formatMoney } from '../lib/format'
import {
  TRANSACTION_KIND_LABELS,
  TRANSACTION_SORT_LABELS,
  distinctCategories,
  filterAndSortTransactions,
  transactionIsIncome,
  type TransactionKind,
  type TransactionSort,
} from '../lib/transactions'

const KINDS: TransactionKind[] = ['all', 'expense', 'income', 'transfer']
const SORTS: TransactionSort[] = ['date', 'amount', 'category']

function kindDescription(kind: TransactionKind): string {
  switch (kind) {
    case 'income':
      return 'Money in — deposits, paychecks, and refunds.'
    case 'expense':
      return 'Money out — spending excluding transfers.'
    case 'transfer':
      return 'Moves between accounts.'
    default:
      return 'Recent debits, credits, and transfers.'
  }
}

export function TransactionsPage() {
  const [isTransactionModalOpen, setIsTransactionModalOpen] = useState(false)
  const [kind, setKind] = useState<TransactionKind>('all')
  const [categoryFilter, setCategoryFilter] = useState<string>('')
  const [sort, setSort] = useState<TransactionSort>('date')
  const [transactionsResult, refreshTransactions] = useTransactions()
  const [accountsResult, refreshAccounts] = useAccounts()
  const transactions = transactionsResult.data?.transactions ?? []
  const accounts = accountsResult.data?.accounts ?? []

  const categories = useMemo(
    () => distinctCategories(transactions),
    [transactions],
  )

  const visibleTransactions = useMemo(
    () =>
      filterAndSortTransactions(
        transactions,
        kind,
        categoryFilter || null,
        sort,
      ),
    [transactions, kind, categoryFilter, sort],
  )

  return (
    <div className="animate-rise mx-auto flex max-w-5xl flex-col gap-8 pt-6">
      <header className="flex flex-col justify-between gap-4 sm:flex-row sm:items-end">
        <div>
          <h1 className="text-3xl font-semibold tracking-[-0.04em]">
            Transactions
          </h1>
          <p className="mt-2 text-sm text-muted">{kindDescription(kind)}</p>
        </div>
        <button
          type="button"
          onClick={() => setIsTransactionModalOpen(true)}
          className="rounded-full bg-accent px-5 py-3 text-sm font-medium text-background hover:bg-accent/90"
        >
          Add transaction
        </button>
      </header>

      <section className="flex flex-col gap-4">
        <div className="flex flex-wrap gap-2">
          {KINDS.map((option) => {
            const isActive = kind === option
            return (
              <button
                key={option}
                type="button"
                onClick={() => setKind(option)}
                className={[
                  'rounded-full px-4 py-2 text-sm transition-colors',
                  isActive
                    ? 'bg-accent/15 text-accent ring-1 ring-accent/35'
                    : 'border border-white/[0.08] text-muted hover:bg-white/[0.04] hover:text-text',
                ].join(' ')}
              >
                {TRANSACTION_KIND_LABELS[option]}
              </button>
            )
          })}
        </div>

        <div className="grid gap-3 sm:grid-cols-2">
          <label className="block">
            <span className="text-xs uppercase tracking-[0.18em] text-muted">
              Category
            </span>
            <SelectInput
              value={categoryFilter}
              onChange={(event) => setCategoryFilter(event.target.value)}
              className="mt-2"
            >
              <option value="">All categories</option>
              {categories.map((category) => (
                <option key={category} value={category}>
                  {category}
                </option>
              ))}
            </SelectInput>
          </label>

          <label className="block">
            <span className="text-xs uppercase tracking-[0.18em] text-muted">
              Sort by
            </span>
            <SelectInput
              value={sort}
              onChange={(event) => setSort(event.target.value as TransactionSort)}
              className="mt-2"
            >
              {SORTS.map((option) => (
                <option key={option} value={option}>
                  {TRANSACTION_SORT_LABELS[option]}
                </option>
              ))}
            </SelectInput>
          </label>
        </div>

        {!transactionsResult.fetching && !transactionsResult.error ? (
          <p className="text-xs text-muted">
            Showing {visibleTransactions.length} of {transactions.length}{' '}
            transactions
            {categoryFilter ? ` in ${categoryFilter}` : ''}
            {kind !== 'all' ? ` (${TRANSACTION_KIND_LABELS[kind].toLowerCase()})` : ''}
          </p>
        ) : null}
      </section>

      <ul className="divide-y divide-white/[0.05] text-sm">
        {transactionsResult.fetching ? (
          <li className="py-4 text-muted">Loading transactions...</li>
        ) : transactionsResult.error ? (
          <li className="py-4 text-red-300">{transactionsResult.error.message}</li>
        ) : visibleTransactions.length === 0 ? (
          <li className="py-4 text-muted">
            {transactions.length === 0
              ? 'No transactions yet.'
              : 'No transactions match these filters.'}
          </li>
        ) : (
          visibleTransactions.map((tx) => {
            const isIncome = transactionIsIncome(tx)
            return (
              <li
                key={tx.id}
                className="flex items-center justify-between gap-4 py-4"
              >
                <div className="min-w-0">
                  <p className="truncate text-text/90">{tx.merchantName}</p>
                  <p className="mt-0.5 text-xs text-muted">
                    {tx.categoryPrimary}
                    {tx.categoryDetailed ? ` / ${tx.categoryDetailed}` : ''} /{' '}
                    {tx.transactionDate}
                    {tx.pending ? ' · pending' : ''}
                  </p>
                </div>
                <span
                  className={[
                    'shrink-0 tabular-nums',
                    isIncome ? 'text-accent' : 'text-text/90',
                  ].join(' ')}
                >
                  {formatMoney(tx.amount)}
                </span>
              </li>
            )
          })
        )}
      </ul>
      <Modal
        title="Add transaction"
        description="Record a manual debit, credit, transfer, or adjustment."
        isOpen={isTransactionModalOpen}
        onClose={() => setIsTransactionModalOpen(false)}
      >
        <TransactionForm
          accounts={accounts}
          onCancel={() => setIsTransactionModalOpen(false)}
          onCreated={() => {
            setIsTransactionModalOpen(false)
            refreshTransactions({ requestPolicy: 'network-only' })
            refreshAccounts({ requestPolicy: 'network-only' })
          }}
        />
      </Modal>
    </div>
  )
}
