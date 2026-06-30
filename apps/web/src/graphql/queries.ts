export const ME_QUERY = `
  query Me {
    me {
      id
      email
      displayName
    }
  }
`

export const LOGIN_MUTATION = `
  mutation Login($input: LoginInput!) {
    login(input: $input) {
      token
      user {
        id
        email
        displayName
      }
    }
  }
`

export const SIGN_UP_MUTATION = `
  mutation SignUp($input: SignUpInput!) {
    signUp(input: $input) {
      token
      user {
        id
        email
        displayName
      }
    }
  }
`

export const ACCOUNTS_QUERY = `
  query Accounts {
    accounts {
      id
      name
      accountType
      provider
      currency
      balance {
        amountCents
        currency
      }
      isActive
    }
  }
`

export const TRANSACTIONS_QUERY = `
  query Transactions($month: String) {
    transactions(month: $month) {
      id
      accountId
      merchantName
      amount {
        amountCents
        currency
      }
      categoryPrimary
      categoryDetailed
      transactionDate
      transactionType
      pending
    }
  }
`

export const CREATE_MANUAL_ACCOUNT_MUTATION = `
  mutation CreateManualAccount($input: ManualAccountInput!) {
    createManualAccount(input: $input) {
      id
      name
      accountType
      provider
      currency
      balance {
        amountCents
        currency
      }
      isActive
    }
  }
`

export const CREATE_MANUAL_TRANSACTION_MUTATION = `
  mutation CreateManualTransaction($input: ManualTransactionInput!) {
    createManualTransaction(input: $input) {
      id
      accountId
      merchantName
      amount {
        amountCents
        currency
      }
      categoryPrimary
      categoryDetailed
      transactionDate
      transactionType
      pending
    }
  }
`

export const CREATE_MANUAL_HOLDING_MUTATION = `
  mutation CreateManualHolding($input: ManualHoldingInput!) {
    createManualHolding(input: $input) {
      id
      accountId
      symbol
      assetName
      assetType
      quantity
      marketValue {
        amountCents
        currency
      }
    }
  }
`

export const IMPORT_TRANSACTIONS_CSV_MUTATION = `
  mutation ImportTransactionsCsv($input: CsvImportInput!) {
    importTransactionsCsv(input: $input) {
      importedCount
      skippedCount
      errors
    }
  }
`

export const TRIGGER_MOCK_SYNC_MUTATION = `
  mutation TriggerMockSync {
    triggerMockSync {
      accountsSynced
      transactionsSynced
      holdingsSynced
      investmentTransactionsSynced
      balanceSnapshotsSynced
      errors
    }
  }
`

export const CREATE_PLAID_LINK_TOKEN_MUTATION = `
  mutation CreatePlaidLinkToken {
    createPlaidLinkToken
  }
`

export const EXCHANGE_PLAID_PUBLIC_TOKEN_MUTATION = `
  mutation ExchangePlaidPublicToken($publicToken: String!) {
    exchangePlaidPublicToken(publicToken: $publicToken)
  }
`

export const SYNC_PLAID_TRANSACTIONS_MUTATION = `
  mutation SyncPlaidTransactions {
    syncPlaidTransactions {
      connectionsSynced
      accountsSynced
      transactionsSynced
      pendingTransactionsSynced
      rawEventsStored
      errors
    }
  }
`

export const CREATE_SNAPTRADE_CONNECTION_URL_MUTATION = `
  mutation CreateSnapTradeConnectionUrl {
    createSnapTradeConnectionUrl
  }
`

export const SYNC_SNAPTRADE_ACCOUNTS_MUTATION = `
  mutation SyncSnapTradeAccounts {
    syncSnapTradeAccounts {
      accountsSynced
      transactionsSynced
      holdingsSynced
      investmentTransactionsSynced
      balanceSnapshotsSynced
      errors
    }
  }
`

export const HOLDINGS_QUERY = `
  query Holdings {
    holdings {
      id
      accountId
      symbol
      assetName
      assetType
      quantity
      marketValue {
        amountCents
        currency
      }
    }
  }
`

export const MONTHLY_SUMMARY_QUERY = `
  query MonthlySummary($month: String!) {
    monthlySummary(month: $month) {
      month
      income {
        amountCents
        currency
      }
      expenses {
        amountCents
        currency
      }
      net {
        amountCents
        currency
      }
      savingsRate
      categorySpend {
        category
        amount {
          amountCents
          currency
        }
        percent
      }
    }
  }
`

export const NET_WORTH_TIMELINE_QUERY = `
  query NetWorthTimeline {
    netWorthTimeline {
      date
      netWorth {
        amountCents
        currency
      }
      cash {
        amountCents
        currency
      }
      investments {
        amountCents
        currency
      }
      debt {
        amountCents
        currency
      }
    }
  }
`

export const CHAT_MESSAGES_QUERY = `
  query ChatMessages {
    chatMessages {
      messages {
        id
        role
        content
        isBriefing
        createdAt
      }
      lastBriefingAt
    }
  }
`

export const SEND_CHAT_MESSAGE_MUTATION = `
  mutation SendChatMessage($input: SendChatMessageInput!) {
    sendChatMessage(input: $input) {
      userMessage {
        id
        role
        content
        isBriefing
        createdAt
      }
      assistantMessage {
        id
        role
        content
        isBriefing
        createdAt
      }
    }
  }
`
