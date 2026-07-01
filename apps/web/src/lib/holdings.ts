import type { Holding, Money } from '../graphql/types'
import { formatMoney, formatPercent } from './format'

export type HoldingGainLoss = {
  amountCents: number
  percent: number
}

export function holdingCostBasis(holding: Holding): Money | null {
  if (holding.costBasis == null) return null
  return holding.costBasis
}

export function holdingGainLoss(holding: Holding): HoldingGainLoss | null {
  const costBasis = holdingCostBasis(holding)
  if (!costBasis || costBasis.amountCents === 0) return null

  const amountCents = holding.marketValue.amountCents - costBasis.amountCents
  const percent = (amountCents / Math.abs(costBasis.amountCents)) * 100

  return { amountCents, percent }
}

export function formatGainLoss(gainLoss: HoldingGainLoss): string {
  const sign = gainLoss.amountCents >= 0 ? '+' : '-'
  const amount = formatMoney({
    amountCents: Math.abs(gainLoss.amountCents),
    currency: 'USD',
  })
  return `${sign}${amount} (${formatPercent(gainLoss.percent / 100)})`
}

export function sumCostBasis(holdings: Holding[]): number {
  return holdings.reduce(
    (total, holding) => total + (holdingCostBasis(holding)?.amountCents ?? 0),
    0,
  )
}
