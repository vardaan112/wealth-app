import { useMutation } from 'urql'
import { IMPORT_TRANSACTIONS_CSV_MUTATION } from '../graphql/queries'
import type {
  CsvImportInput,
  ImportTransactionsCsvMutation,
} from '../graphql/types'

export function useImportTransactionsCsv() {
  return useMutation<
    ImportTransactionsCsvMutation,
    { input: CsvImportInput }
  >(IMPORT_TRANSACTIONS_CSV_MUTATION)
}
