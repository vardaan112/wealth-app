import type { Money } from '../graphql/types'

export function formatMoney(money: Money): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: money.currency,
  }).format(money.amountCents / 100)
}

export function formatPercent(value: number): string {
  return `${value.toFixed(1)}%`
}

export function currentMonth(): string {
  const now = new Date()
  const month = String(now.getMonth() + 1).padStart(2, '0')
  return `${now.getFullYear()}-${month}`
}

export function formatMonthLabel(month: string): string {
  const [year, monthNumber] = month.split('-')
  const date = new Date(Number(year), Number(monthNumber) - 1, 1)
  return date.toLocaleDateString('en-US', { month: 'long', year: 'numeric' })
}

export function recentMonths(count: number): string[] {
  const months: string[] = []
  const cursor = new Date()

  for (let index = count - 1; index >= 0; index -= 1) {
    const date = new Date(cursor.getFullYear(), cursor.getMonth() - index, 1)
    const month = String(date.getMonth() + 1).padStart(2, '0')
    months.push(`${date.getFullYear()}-${month}`)
  }

  return months
}
