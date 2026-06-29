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
  categoryDetailed?: string | null
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
export type TransactionsQueryVariables = { month?: string | null }
export type HoldingsQuery = { holdings: Holding[] }
export type MonthlySummaryQuery = { monthlySummary: MonthlySummary }
export type NetWorthTimelineQuery = { netWorthTimeline: NetWorthPoint[] }

export type ManualAccountInput = {
  name: string
  accountType: string
  provider?: string | null
  currency?: string | null
}

export type ManualTransactionInput = {
  accountId: string
  amountCents: number
  currency?: string | null
  merchantName?: string | null
  rawDescription?: string | null
  categoryPrimary?: string | null
  categoryDetailed?: string | null
  transactionDate: string
  pending?: boolean | null
  transactionType?: string | null
  notes?: string | null
}

export type ManualHoldingInput = {
  accountId: string
  symbol: string
  assetName?: string | null
  assetType?: string | null
  quantity: number
  marketValueCents?: number | null
  costBasisCents?: number | null
  priceCents?: number | null
  currency?: string | null
}

export type CsvImportInput = {
  accountId: string
  source: string
  csvText: string
}

export type CsvImportResult = {
  importedCount: number
  skippedCount: number
  errors: string[]
}

export type SyncResult = {
  accountsSynced: number
  transactionsSynced: number
  holdingsSynced: number
  investmentTransactionsSynced: number
  balanceSnapshotsSynced: number
  errors: string[]
}

export type PlaidSyncResult = {
  connectionsSynced: number
  accountsSynced: number
  transactionsSynced: number
  pendingTransactionsSynced: number
  rawEventsStored: number
  errors: string[]
}

export type CreateManualAccountMutation = {
  createManualAccount: Account
}

export type CreateManualTransactionMutation = {
  createManualTransaction: Transaction
}

export type CreateManualHoldingMutation = {
  createManualHolding: Holding
}

export type ImportTransactionsCsvMutation = {
  importTransactionsCsv: CsvImportResult
}

export type TriggerMockSyncMutation = {
  triggerMockSync: SyncResult
}

export type CreatePlaidLinkTokenMutation = {
  createPlaidLinkToken: string
}

export type ExchangePlaidPublicTokenMutation = {
  exchangePlaidPublicToken: boolean
}

export type SyncPlaidTransactionsMutation = {
  syncPlaidTransactions: PlaidSyncResult
}

export type CreateSnapTradeConnectionUrlMutation = {
  createSnapTradeConnectionUrl: string
}

export type SyncSnapTradeAccountsMutation = {
  syncSnapTradeAccounts: SyncResult
}

export type LoginInput = {
  email: string
  password: string
}

export type SignUpInput = {
  email: string
  password: string
  displayName?: string | null
}

export type LoginMutation = {
  login: {
    token: string
    user: User
  }
}

export type SignUpMutation = {
  signUp: {
    token: string
    user: User
  }
}
