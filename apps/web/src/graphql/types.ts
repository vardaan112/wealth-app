export type Money = {
  amountCents: number
  currency: string
}

export type User = {
  id: string
  email: string
  displayName: string
}

export type Account = {
  id: string
  name: string
  accountType: string
  provider: string
  currency: string
  balance: Money
  isActive: boolean
}

export type Transaction = {
  id: string
  accountId: string
  merchantName: string
  amount: Money
  categoryPrimary: string
  transactionDate: string
  transactionType: string
  pending: boolean
}

export type Holding = {
  id: string
  accountId: string
  symbol: string
  assetName: string
  assetType: string
  quantity: number
  marketValue: Money
}

export type CategorySpend = {
  category: string
  amount: Money
  percent: number
}

export type MonthlySummary = {
  month: string
  income: Money
  expenses: Money
  net: Money
  savingsRate: number
  categorySpend: CategorySpend[]
}

export type NetWorthPoint = {
  date: string
  netWorth: Money
  cash: Money
  investments: Money
  debt: Money
}

export type MeQuery = { me: User }
export type AccountsQuery = { accounts: Account[] }
export type TransactionsQuery = { transactions: Transaction[] }
export type HoldingsQuery = { holdings: Holding[] }
export type MonthlySummaryQuery = { monthlySummary: MonthlySummary }
export type NetWorthTimelineQuery = { netWorthTimeline: NetWorthPoint[] }
