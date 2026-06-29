import { useEffect, useRef, useState } from 'react'
import { usePlaidLink } from 'react-plaid-link'
import { useNavigate } from 'react-router-dom'
import { useExchangePlaidPublicToken } from '../hooks/useExchangePlaidPublicToken'
import {
  clearSavedPlaidLinkToken,
  getSavedPlaidLinkToken,
  isPlaidOAuthRedirect,
} from '../plaid/oauth'

// Plaid relaunches Link twice for OAuth banks (e.g. Chase): once on Settings
// before redirecting to the institution, and again here after the institution
// redirects back to our registered redirect URI. This page re-initializes Link
// with the saved link_token plus the received redirect URI so the flow resumes.
export function PlaidOAuthPage() {
  const navigate = useNavigate()
  const [, exchangePlaidPublicToken] = useExchangePlaidPublicToken()
  const [savedToken] = useState(() => getSavedPlaidLinkToken())
  const [isOAuthReturn] = useState(() => isPlaidOAuthRedirect())
  const [message, setMessage] = useState('Finishing secure bank connection...')
  const openedRef = useRef(false)

  const { open, ready } = usePlaidLink({
    token: savedToken,
    receivedRedirectUri: isOAuthReturn ? window.location.href : undefined,
    onSuccess: async (publicToken) => {
      setMessage('Saving your bank connection...')
      const response = await exchangePlaidPublicToken({ publicToken })
      clearSavedPlaidLinkToken()

      if (response.data?.exchangePlaidPublicToken && !response.error) {
        navigate('/settings?plaid=connected', { replace: true })
        return
      }

      navigate('/settings?plaid=error', { replace: true })
    },
    onExit: () => {
      clearSavedPlaidLinkToken()
      navigate('/settings', { replace: true })
    },
  })

  useEffect(() => {
    if (!isOAuthReturn || !savedToken) {
      // Not a valid OAuth return (no state param or no persisted token);
      // send the user back to Settings to start over.
      navigate('/settings', { replace: true })
      return
    }

    if (ready && !openedRef.current) {
      openedRef.current = true
      open()
    }
  }, [isOAuthReturn, savedToken, ready, open, navigate])

  return (
    <div className="flex min-h-screen items-center justify-center px-6">
      <div className="max-w-md text-center">
        <p className="text-sm uppercase tracking-[0.18em] text-muted">
          Plaid Link
        </p>
        <p className="mt-2 text-lg text-text">{message}</p>
        <p className="mt-2 text-sm text-muted">
          Please keep this tab open. You'll return to Settings automatically.
        </p>
      </div>
    </div>
  )
}
