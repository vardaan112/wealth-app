import { NavLink } from 'react-router-dom'
import { navItems } from '../lib/nav'

export function BottomNav() {
  return (
    <nav className="fixed inset-x-0 bottom-0 z-10 border-t border-border bg-surface md:hidden">
      <ul className="flex items-stretch justify-around px-1 py-1">
        {navItems.map(({ path, label, icon: Icon }) => (
          <li key={path} className="flex-1">
            <NavLink
              to={path}
              end={path === '/'}
              className={({ isActive }) =>
                [
                  'flex flex-col items-center gap-0.5 rounded-lg px-1 py-2 text-[10px] transition-colors',
                  isActive ? 'text-accent' : 'text-muted',
                ].join(' ')
              }
            >
              <Icon className="size-5" strokeWidth={1.75} />
              <span className="truncate">{label.split(' ')[0]}</span>
            </NavLink>
          </li>
        ))}
      </ul>
    </nav>
  )
}
