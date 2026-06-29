const tokenKey = 'wealth-app-token'

export function getAuthToken(): string | null {
  return window.sessionStorage.getItem(tokenKey)
}

export function setAuthToken(token: string) {
  window.sessionStorage.setItem(tokenKey, token)
}

export function clearAuthToken() {
  window.sessionStorage.removeItem(tokenKey)
}
