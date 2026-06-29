import { Outlet } from 'react-router-dom'
import { BottomNav } from './BottomNav'
import { Sidebar } from './Sidebar'

export function AppShell() {
  return (
    <div className="relative flex min-h-screen overflow-hidden bg-background text-text">
      <div className="pointer-events-none absolute inset-0 bg-[linear-gradient(180deg,rgba(255,255,255,0.035),transparent_28rem)]" />
      <Sidebar />
      <div className="relative flex min-h-screen flex-1 flex-col">
        <main className="flex-1 overflow-y-auto px-4 py-5 pb-24 sm:px-6 md:px-10 md:py-8 md:pb-8">
          <Outlet />
        </main>
        <BottomNav />
      </div>
    </div>
  )
}
