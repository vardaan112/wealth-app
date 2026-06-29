import { NavLink } from 'react-router-dom'
import { navItems } from '../lib/nav'

export function Sidebar() {
  return (
    <aside className="hidden w-56 shrink-0 border-r border-border bg-surface md:flex md:flex-col">
      <div className="border-b border-border px-5 py-6">
        <p className="text-xs font-medium uppercase tracking-widest text-accent">
          Wealth
        </p>
        <p className="mt-1 text-sm text-muted">Personal finance</p>
      </div>
      <nav className="flex flex-1 flex-col gap-1 p-3">
        {navItems.map(({ path, label, icon: Icon }) => (
          <NavLink
            key={path}
            to={path}
            end={path === '/'}
            className={({ isActive }) =>
              [
                'flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm transition-colors',
                isActive
                  ? 'bg-background text-text'
                  : 'text-muted hover:bg-background hover:text-text',
              ].join(' ')
            }
          >
            <Icon className="size-4 shrink-0" strokeWidth={1.75} />
            {label}
          </NavLink>
        ))}
      </nav>
    </aside>
  )
}
