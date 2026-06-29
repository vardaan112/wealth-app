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
        'rounded-[1.75rem] bg-white/[0.022] p-6 ring-1 ring-white/[0.05] backdrop-blur-sm transition-colors hover:bg-white/[0.035] hover:ring-white/[0.08]',
        className,
      ].join(' ')}
    >
      <h2 className="text-sm font-medium tracking-tight text-text/90">{title}</h2>
      <p className="mt-1 text-[0.8rem] leading-6 text-muted">{description}</p>
      {children ? <div className="mt-5">{children}</div> : null}
    </section>
  )
}
