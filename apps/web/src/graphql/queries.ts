export const ME_QUERY = `
  query Me {
    me {
      id
      email
      displayName
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
  query Transactions {
    transactions {
      id
      accountId
      merchantName
      amount {
        amountCents
        currency
      }
      categoryPrimary
      transactionDate
      transactionType
      pending
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
