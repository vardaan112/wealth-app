import { NavLink } from 'react-router-dom'
import { navItems } from '../lib/nav'

export function BottomNav() {
  return (
    <nav className="fixed inset-x-0 bottom-0 z-10 border-t border-white/[0.05] bg-background/70 backdrop-blur-2xl md:hidden">
      <ul className="flex items-stretch justify-around px-2 py-2">
        {navItems.map(({ path, label, icon: Icon }) => (
          <li key={path} className="flex-1">
            <NavLink
              to={path}
              end={path === '/'}
              className={({ isActive }) =>
                [
                  'flex flex-col items-center gap-1 rounded-xl px-1 py-2 text-[10px] transition-colors',
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
