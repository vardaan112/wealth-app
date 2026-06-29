import type {
  ButtonHTMLAttributes,
  InputHTMLAttributes,
  ReactNode,
  SelectHTMLAttributes,
  TextareaHTMLAttributes,
} from 'react'

type FieldProps = {
  label: string
  children: ReactNode
  hint?: string
}

export function Field({ label, children, hint }: FieldProps) {
  return (
    <label className="block">
      <span className="text-xs uppercase tracking-[0.18em] text-muted">
        {label}
      </span>
      <div className="mt-2">{children}</div>
      {hint ? <p className="mt-1.5 text-xs text-muted/75">{hint}</p> : null}
    </label>
  )
}

export function TextInput(props: InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      {...props}
      className={[
        'w-full rounded-2xl border border-white/[0.07] bg-background/55 px-4 py-3 text-sm text-text outline-none',
        'placeholder:text-muted/55 focus:border-accent/45 focus:bg-background/80',
        props.className ?? '',
      ].join(' ')}
    />
  )
}

export function SelectInput(props: SelectHTMLAttributes<HTMLSelectElement>) {
  return (
    <select
      {...props}
      className={[
        'w-full rounded-2xl border border-white/[0.07] bg-background/55 px-4 py-3 text-sm text-text outline-none',
        'focus:border-accent/45 focus:bg-background/80',
        props.className ?? '',
      ].join(' ')}
    />
  )
}

export function TextAreaInput(props: TextareaHTMLAttributes<HTMLTextAreaElement>) {
  return (
    <textarea
      {...props}
      className={[
        'min-h-40 w-full rounded-2xl border border-white/[0.07] bg-background/55 px-4 py-3 text-sm text-text outline-none',
        'placeholder:text-muted/55 focus:border-accent/45 focus:bg-background/80',
        props.className ?? '',
      ].join(' ')}
    />
  )
}

export function FormActions({
  children,
}: {
  children: ReactNode
}) {
  return <div className="mt-6 flex flex-col gap-3 sm:flex-row">{children}</div>
}

export function PrimaryButton(props: ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      {...props}
      className={[
        'rounded-full bg-accent px-5 py-3 text-sm font-medium text-background hover:bg-accent/90 disabled:cursor-not-allowed disabled:opacity-50',
        props.className ?? '',
      ].join(' ')}
    />
  )
}

export function SecondaryButton(props: ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      {...props}
      className={[
        'rounded-full border border-white/[0.08] px-5 py-3 text-sm text-muted hover:bg-white/[0.04] hover:text-text',
        props.className ?? '',
      ].join(' ')}
    />
  )
}
