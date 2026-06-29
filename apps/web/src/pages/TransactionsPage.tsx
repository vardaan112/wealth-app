import { useState } from 'react'
import { TransactionForm } from '../components/forms/TransactionForm'
import { Modal } from '../components/Modal'
import { useAccounts } from '../hooks/useAccounts'
import { useTransactions } from '../hooks/useTransactions'
import { formatMoney } from '../lib/format'

export function TransactionsPage() {
  const [isTransactionModalOpen, setIsTransactionModalOpen] = useState(false)
  const [transactionsResult, refreshTransactions] = useTransactions()
  const [accountsResult, refreshAccounts] = useAccounts()
  const transactions = transactionsResult.data?.transactions ?? []
  const accounts = accountsResult.data?.accounts ?? []

  return (
    <div className="animate-rise mx-auto flex max-w-5xl flex-col gap-8 pt-6">
      <header className="flex flex-col justify-between gap-4 sm:flex-row sm:items-end">
        <div>
          <h1 className="text-3xl font-semibold tracking-[-0.04em]">
            Transactions
          </h1>
          <p className="mt-2 text-sm text-muted">
            Recent debits, credits, and transfers.
          </p>
        </div>
        <button
          type="button"
          onClick={() => setIsTransactionModalOpen(true)}
          className="rounded-full bg-accent px-5 py-3 text-sm font-medium text-background hover:bg-accent/90"
        >
          Add transaction
        </button>
      </header>
      <ul className="divide-y divide-white/[0.05] text-sm">
        {transactionsResult.fetching ? (
          <li className="py-4 text-muted">Loading transactions...</li>
        ) : transactionsResult.error ? (
          <li className="py-4 text-red-300">{transactionsResult.error.message}</li>
        ) : transactions.length === 0 ? (
          <li className="py-4 text-muted">No transactions yet.</li>
        ) : (
          transactions.map((tx) => (
          <li
            key={tx.id}
            className="flex items-center justify-between py-4"
          >
            <div>
              <p className="text-text/90">{tx.merchantName}</p>
              <p className="mt-0.5 text-xs text-muted">
                {tx.categoryPrimary}
                {tx.categoryDetailed ? ` / ${tx.categoryDetailed}` : ''} /{' '}
                {tx.transactionDate}
              </p>
            </div>
            <span className="tabular-nums text-text/90">
              {formatMoney(tx.amount)}
            </span>
          </li>
          ))
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
