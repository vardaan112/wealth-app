import type { Transaction } from '../graphql/types'

export type TransactionKind = 'all' | 'expense' | 'income' | 'transfer'

export type TransactionSort = 'date' | 'amount' | 'category'

export function transactionCategory(transaction: Transaction): string {
  return transaction.categoryPrimary?.trim() || 'Uncategorized'
}

export function transactionIsTransfer(transaction: Transaction): boolean {
  return transaction.transactionType.toLowerCase() === 'transfer'
}

export function transactionIsIncome(transaction: Transaction): boolean {
  return (
    transaction.amount.amountCents > 0 ||
    transaction.transactionType.toLowerCase() === 'income'
  )
}

export function transactionIsSpending(transaction: Transaction): boolean {
  return (
    transaction.amount.amountCents < 0 ||
    transaction.transactionType.toLowerCase() === 'expense'
  )
}

export function transactionMatchesKind(
  transaction: Transaction,
  kind: TransactionKind,
): boolean {
  if (kind === 'all') {
    return true
  }

  if (kind === 'transfer') {
    return transactionIsTransfer(transaction)
  }

  if (kind === 'income') {
    return transactionIsIncome(transaction) && !transactionIsTransfer(transaction)
  }

  return (
    transactionIsSpending(transaction) &&
    !transactionIsIncome(transaction) &&
    !transactionIsTransfer(transaction)
  )
}

export function distinctCategories(transactions: Transaction[]): string[] {
  const categories = new Set(transactions.map(transactionCategory))
  return [...categories].sort((left, right) => left.localeCompare(right))
}

function compareTransactions(
  left: Transaction,
  right: Transaction,
  sort: TransactionSort,
): number {
  if (sort === 'date') {
    return (
      right.transactionDate.localeCompare(left.transactionDate) ||
      right.id.localeCompare(left.id)
    )
  }

  if (sort === 'amount') {
    return (
      Math.abs(right.amount.amountCents) - Math.abs(left.amount.amountCents) ||
      right.transactionDate.localeCompare(left.transactionDate)
    )
  }

  return (
    transactionCategory(left).localeCompare(transactionCategory(right)) ||
    right.transactionDate.localeCompare(left.transactionDate)
  )
}

export function filterAndSortTransactions(
  transactions: Transaction[],
  kind: TransactionKind,
  category: string | null,
  sort: TransactionSort,
): Transaction[] {
  let result = transactions.filter((transaction) =>
    transactionMatchesKind(transaction, kind),
  )

  if (category) {
    result = result.filter(
      (transaction) => transactionCategory(transaction) === category,
    )
  }

  return [...result].sort((left, right) =>
    compareTransactions(left, right, sort),
  )
}

export const TRANSACTION_KIND_LABELS: Record<TransactionKind, string> = {
  all: 'All',
  expense: 'Expenses',
  income: 'Income',
  transfer: 'Transfers',
}

export const TRANSACTION_SORT_LABELS: Record<TransactionSort, string> = {
  date: 'Date',
  amount: 'Amount',
  category: 'Category',
}
