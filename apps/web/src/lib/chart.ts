import type { Money } from '../graphql/types'
import { formatMoney } from './format'

export const chartTooltipStyle = {
  background: 'rgba(16,19,23,0.92)',
  border: '1px solid rgba(255,255,255,0.08)',
  borderRadius: 14,
  fontSize: 12,
  color: '#f4f6f8',
  backdropFilter: 'blur(8px)',
}

export const chartLabelStyle = { color: '#8b95a5' }

export function centsToMoney(amountCents: number): Money {
  return { amountCents, currency: 'USD' }
}

export function formatCents(amountCents: number): string {
  return formatMoney(centsToMoney(amountCents))
}
