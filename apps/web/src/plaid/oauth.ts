const LINK_TOKEN_STORAGE_KEY = 'wealth-app-plaid-link-token'

// Plaid appends this query parameter to the redirect URI when an OAuth bank
// (e.g. Chase) sends the user back to our app to resume the Link flow.
const OAUTH_STATE_PARAM = 'oauth_state_id'

export function savePlaidLinkToken(token: string) {
  window.localStorage.setItem(LINK_TOKEN_STORAGE_KEY, token)
}

export function getSavedPlaidLinkToken(): string | null {
  return window.localStorage.getItem(LINK_TOKEN_STORAGE_KEY)
}

export function clearSavedPlaidLinkToken() {
  window.localStorage.removeItem(LINK_TOKEN_STORAGE_KEY)
}

export function isPlaidOAuthRedirect(): boolean {
  return new URLSearchParams(window.location.search).has(OAUTH_STATE_PARAM)
}
