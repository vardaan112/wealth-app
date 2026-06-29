import { NavLink } from 'react-router-dom'
import { useAuth } from '../auth/AuthContext'
import { navItems } from '../lib/nav'

export function Sidebar() {
  const { logout } = useAuth()

  return (
    <aside className="hidden w-60 shrink-0 px-3 py-8 md:flex md:flex-col">
      <div className="px-3.5">
        <p className="text-sm font-semibold tracking-tight text-text">Wealth</p>
        <p className="mt-0.5 text-xs text-muted">Private dashboard</p>
      </div>
      <nav className="mt-8 flex flex-1 flex-col gap-0.5">
        {navItems.map(({ path, label, icon: Icon }) => (
          <NavLink
            key={path}
            to={path}
            end={path === '/'}
            className={({ isActive }) =>
              [
                'group flex items-center gap-3 rounded-xl px-3.5 py-2.5 text-sm transition-colors',
                isActive
                  ? 'bg-white/[0.05] text-text'
                  : 'text-muted hover:text-text',
              ].join(' ')
            }
          >
            <Icon
              className="size-4 shrink-0 opacity-70 transition-opacity group-hover:opacity-100"
              strokeWidth={1.75}
            />
            {label}
          </NavLink>
        ))}
      </nav>
      <button
        type="button"
        onClick={logout}
        className="mx-3.5 rounded-xl px-3.5 py-2.5 text-left text-sm text-muted hover:bg-white/[0.04] hover:text-text"
      >
        Log out
      </button>
    </aside>
  )
}
