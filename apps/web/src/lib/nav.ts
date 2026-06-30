import {
  ArrowLeftRight,
  Calendar,
  Home,
  MessageSquare,
  PieChart,
  Receipt,
  Settings,
  type LucideIcon,
} from 'lucide-react'

export type NavItem = {
  path: string
  label: string
  icon: LucideIcon
}

export const navItems: NavItem[] = [
  { path: '/', label: 'Home', icon: Home },
  { path: '/cash-flow', label: 'Cash Flow', icon: ArrowLeftRight },
  { path: '/portfolio', label: 'Portfolio', icon: PieChart },
  { path: '/advisor', label: 'Advisor', icon: MessageSquare },
  { path: '/timeline', label: 'Timeline', icon: Calendar },
  { path: '/transactions', label: 'Transactions', icon: Receipt },
  { path: '/settings', label: 'Settings', icon: Settings },
]
