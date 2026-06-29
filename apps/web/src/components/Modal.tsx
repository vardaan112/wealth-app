import type { ReactNode } from 'react'

type ModalProps = {
  title: string
  description: string
  isOpen: boolean
  onClose: () => void
  children: ReactNode
}

export function Modal({
  title,
  description,
  isOpen,
  onClose,
  children,
}: ModalProps) {
  if (!isOpen) return null

  return (
    <div className="fixed inset-0 z-50 flex items-end bg-background/70 px-3 py-3 backdrop-blur-sm sm:items-center sm:justify-center">
      <button
        type="button"
        aria-label="Close modal"
        className="absolute inset-0 cursor-default"
        onClick={onClose}
      />
      <section className="relative max-h-[92vh] w-full overflow-y-auto rounded-[1.75rem] border border-white/[0.07] bg-surface p-5 shadow-[0_30px_90px_rgba(0,0,0,0.45)] sm:max-w-xl sm:p-6">
        <div className="flex items-start justify-between gap-4">
          <div>
            <h2 className="text-lg font-medium tracking-tight text-text">
              {title}
            </h2>
            <p className="mt-1 text-sm leading-6 text-muted">{description}</p>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="rounded-full px-3 py-1 text-sm text-muted hover:bg-white/[0.04] hover:text-text"
          >
            Close
          </button>
        </div>
        <div className="mt-6">{children}</div>
      </section>
    </div>
  )
}
