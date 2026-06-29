export function TimelinePage() {
  const events = [
    { title: 'Emergency fund complete', detail: 'Target reached', when: 'Mar 2026', done: true },
    { title: 'Mortgage payoff', detail: 'Projected', when: 'Dec 2038', done: false },
    { title: 'Retirement target', detail: 'Age 55, estimated', when: '2042', done: false },
  ]

  return (
    <div className="animate-rise mx-auto flex max-w-5xl flex-col gap-8 pt-6">
      <header>
        <h1 className="text-3xl font-semibold tracking-[-0.04em]">Timeline</h1>
        <p className="mt-2 text-sm text-muted">
          Milestones, goals, and projected events.
        </p>
      </header>
      <ol className="relative ml-1 border-l border-white/[0.07]">
        {events.map((event) => (
          <li key={event.title} className="relative pb-8 pl-6 last:pb-0">
            <span
              className={`absolute -left-[5px] top-1.5 size-2.5 rounded-full ${
                event.done ? 'bg-accent' : 'bg-white/20'
              }`}
            />
            <p className="text-sm text-text/90">{event.title}</p>
            <p className="mt-1 text-xs text-muted">
              {event.detail} &middot; {event.when}
            </p>
          </li>
        ))}
      </ol>
    </div>
  )
}
