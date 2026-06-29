import { cacheExchange, Client, fetchExchange } from 'urql'

export const graphqlClient = new Client({
  url: '/graphql',
  exchanges: [cacheExchange, fetchExchange],
  fetchOptions: {
    method: 'POST',
  },
})
