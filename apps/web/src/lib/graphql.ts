import { cacheExchange, Client, fetchExchange } from 'urql'

export const graphqlClient = new Client({
  url: 'http://localhost:8000/graphql',
  exchanges: [cacheExchange, fetchExchange],
})
