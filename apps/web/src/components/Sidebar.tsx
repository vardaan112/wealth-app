import { NavLink } from 'react-router-dom'
import { navItems } from '../lib/nav'

export function Sidebar() {
  return (
    <aside className="hidden w-64 shrink-0 border-r border-white/[0.06] bg-surface/72 backdrop-blur-xl md:flex md:flex-col">
      <div className="px-6 py-7">
        <div className="flex items-center gap-3">
          <div className="grid size-10 place-items-center rounded-2xl border border-accent/25 bg-accent/10 text-sm font-semibold text-accent">
            W
          </div>
          <div>
            <p className="text-sm font-semibold tracking-wide text-text">
              Wealth
            </p>
            <p className="text-xs text-muted">Private dashboard</p>
          </div>
        </div>
      </div>
      <nav className="flex flex-1 flex-col gap-1 px-3">
        {navItems.map(({ path, label, icon: Icon }) => (
          <NavLink
            key={path}
            to={path}
            end={path === '/'}
            className={({ isActive }) =>
              [
                'group flex items-center gap-3 rounded-2xl px-3.5 py-3 text-sm transition-all',
                isActive
                  ? 'bg-white/[0.06] text-text shadow-[inset_0_0_0_1px_rgba(124,156,255,0.12)]'
                  : 'text-muted hover:bg-white/[0.035] hover:text-text',
              ].join(' ')
            }
          >
            <Icon
              className="size-4 shrink-0 text-accent/80 transition-colors group-hover:text-accent"
              strokeWidth={1.75}
            />
            {label}
          </NavLink>
        ))}
      </nav>
      <div className="m-4 rounded-3xl border border-border/70 bg-background/55 p-4">
        <p className="text-xs uppercase tracking-[0.24em] text-muted">Status</p>
        <p className="mt-2 text-sm text-text">Mock data connected</p>
      </div>
    </aside>
  )
}
