import {
  createContext,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from 'react'
import { clearAuthToken, getAuthToken, setAuthToken } from './session'

type AuthContextValue = {
  token: string | null
  login: (token: string) => void
  logout: () => void
}

const AuthContext = createContext<AuthContextValue | null>(null)

export function AuthProvider({ children }: { children: ReactNode }) {
  const [token, setToken] = useState(() => getAuthToken())

  const value = useMemo<AuthContextValue>(
    () => ({
      token,
      login(nextToken) {
        setAuthToken(nextToken)
        setToken(nextToken)
      },
      logout() {
        clearAuthToken()
        setToken(null)
      },
    }),
    [token],
  )

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}

export function useAuth() {
  const context = useContext(AuthContext)
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider')
  }
  return context
}
