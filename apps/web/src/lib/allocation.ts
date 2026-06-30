import type { Holding } from '../graphql/types'

export type AllocationSlice = {
  label: string
  symbol: string
  assetType: string
  value: number
  percent: number
}

const TOP_N = 8

export function buildHoldingsAllocation(
  holdings: Holding[],
  topN = TOP_N,
): AllocationSlice[] {
  const bySymbol = holdings.reduce<
    Record<string, { value: number; assetType: string }>
  >((groups, holding) => {
    const value = holding.marketValue.amountCents
    const existing = groups[holding.symbol]
    if (existing) {
      existing.value += value
    } else {
      groups[holding.symbol] = { value, assetType: holding.assetType }
    }
    return groups
  }, {})

  const totalValue = Object.values(bySymbol).reduce(
    (sum, entry) => sum + entry.value,
    0,
  )

  const sorted = Object.entries(bySymbol)
    .map(([symbol, { value, assetType }]) => ({
      symbol,
      assetType,
      value,
      percent: totalValue > 0 ? (value / totalValue) * 100 : 0,
      label: symbol,
    }))
    .sort((a, b) => b.value - a.value)

  if (sorted.length <= topN) {
    return sorted
  }

  const top = sorted.slice(0, topN)
  const otherValue = sorted.slice(topN).reduce((sum, entry) => sum + entry.value, 0)

  return [
    ...top,
    {
      symbol: 'Other',
      assetType: 'mixed',
      value: otherValue,
      percent: totalValue > 0 ? (otherValue / totalValue) * 100 : 0,
      label: 'Other',
    },
  ]
}

export const allocationColors = [
  '#7c9cff',
  '#6eb5ff',
  '#a78bfa',
  '#5eead4',
  '#fbbf24',
  '#f87171',
  '#34d399',
  '#8b95a5',
  '#343b46',
]
