import type { ReactNode } from 'react'

type PlaceholderCardProps = {
  title: string
  description: string
  children?: ReactNode
}

export function PlaceholderCard({
  title,
  description,
  children,
}: PlaceholderCardProps) {
  return (
    <section className="rounded-xl border border-border bg-surface p-5">
      <h2 className="text-lg font-medium text-text">{title}</h2>
      <p className="mt-1 text-sm text-muted">{description}</p>
      {children ? <div className="mt-4">{children}</div> : null}
    </section>
  )
}
