import { cacheExchange, Client, fetchExchange } from 'urql'
import { getAuthToken } from '../auth/session'

export const graphqlClient = new Client({
  url: '/graphql',
  exchanges: [cacheExchange, fetchExchange],
  fetchOptions: () => {
    const token = getAuthToken()

    return {
      method: 'POST',
      headers: token ? { Authorization: `Bearer ${token}` } : undefined,
    }
  },
})
