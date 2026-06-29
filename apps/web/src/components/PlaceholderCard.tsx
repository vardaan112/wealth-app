import type { ReactNode } from 'react'

type PlaceholderCardProps = {
  title: string
  description: string
  children?: ReactNode
  className?: string
}

export function PlaceholderCard({
  title,
  description,
  children,
  className = '',
}: PlaceholderCardProps) {
  return (
    <section
      className={[
        'rounded-3xl border border-border/80 bg-surface/82 p-5 shadow-[0_18px_60px_rgba(0,0,0,0.28)] backdrop-blur',
        className,
      ].join(' ')}
    >
      <h2 className="text-base font-medium tracking-tight text-text">{title}</h2>
      <p className="mt-1 text-sm leading-6 text-muted">{description}</p>
      {children ? <div className="mt-4">{children}</div> : null}
    </section>
  )
}
