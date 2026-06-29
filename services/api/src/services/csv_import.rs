use chrono::NaiveDate;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::repositories::import_batches;
use crate::repositories::transactions;

#[derive(Debug, Clone, Serialize)]
pub struct CsvImportResult {
    pub imported_count: i32,
    pub skipped_count: i32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
struct CsvTransactionRow {
    transaction_date: NaiveDate,
    raw_description: String,
    amount_cents: i64,
    category_primary: Option<String>,
    transaction_type: String,
}

pub async fn import_transactions_csv(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    source: String,
    csv_text: String,
) -> Result<CsvImportResult, sqlx::Error> {
    let batch =
        import_batches::create_import_batch(pool, user_id, account_id, source.clone()).await?;

    let mut imported_count = 0;
    let mut skipped_count = 0;
    let mut errors = Vec::new();

    let rows = parse_csv_rows(&csv_text);

    for row_result in rows {
        let row = match row_result {
            Ok(row) => row,
            Err(error) => {
                skipped_count += 1;
                errors.push(error);
                continue;
            }
        };

        let is_duplicate = transactions::transaction_duplicate_exists(
            pool,
            account_id,
            row.transaction_date,
            row.amount_cents,
            &row.raw_description,
        )
        .await?;

        if is_duplicate {
            skipped_count += 1;
            continue;
        }

        let merchant_name = row.raw_description.clone();
        transactions::create_transaction(
            pool,
            user_id,
            transactions::CreateTransactionInput {
                account_id,
                provider: Some(source.clone()),
                provider_transaction_id: None,
                amount_cents: row.amount_cents,
                currency: Some("USD".to_string()),
                merchant_name: Some(merchant_name),
                raw_description: Some(row.raw_description),
                category_primary: row.category_primary,
                category_detailed: None,
                transaction_date: row.transaction_date,
                authorized_date: None,
                pending: Some(false),
                transaction_type: Some(row.transaction_type),
                notes: None,
            },
        )
        .await?;
        imported_count += 1;
    }

    let status = if errors.is_empty() {
        "completed"
    } else if imported_count > 0 {
        "partial"
    } else {
        "failed"
    };

    import_batches::finalize_import_batch(
        pool,
        batch.id,
        imported_count,
        skipped_count,
        errors.len() as i32,
        status.to_string(),
    )
    .await?;

    Ok(CsvImportResult {
        imported_count,
        skipped_count,
        errors,
    })
}

fn parse_csv_rows(csv_text: &str) -> Vec<Result<CsvTransactionRow, String>> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(csv_text.as_bytes());

    let headers = match reader.headers() {
        Ok(headers) => headers.clone(),
        Err(error) => {
            return vec![Err(format!("CSV header error: {error}"))];
        }
    };

    let date_index = match header_index(&headers, "date") {
        Some(index) => index,
        None => return vec![Err("CSV is missing required date column".to_string())],
    };
    let description_index = match header_index(&headers, "description") {
        Some(index) => index,
        None => return vec![Err("CSV is missing required description column".to_string())],
    };
    let amount_index = match header_index(&headers, "amount") {
        Some(index) => index,
        None => return vec![Err("CSV is missing required amount column".to_string())],
    };
    let category_index = match header_index(&headers, "category") {
        Some(index) => index,
        None => return vec![Err("CSV is missing required category column".to_string())],
    };

    reader
        .records()
        .enumerate()
        .map(|(index, record_result)| {
            let row_number = index + 2;
            let record = record_result
                .map_err(|error| format!("Row {row_number}: invalid CSV row: {error}"))?;

            let date = record.get(date_index).unwrap_or_default();
            let description = record.get(description_index).unwrap_or_default().trim();
            let amount = record.get(amount_index).unwrap_or_default();
            let category = record.get(category_index).unwrap_or_default().trim();

            if description.is_empty() {
                return Err(format!("Row {row_number}: description is required"));
            }

            let transaction_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .map_err(|error| format!("Row {row_number}: invalid date: {error}"))?;
            let amount_cents =
                parse_amount_cents(amount).map_err(|error| format!("Row {row_number}: {error}"))?;

            Ok(CsvTransactionRow {
                transaction_date,
                raw_description: description.to_string(),
                amount_cents,
                category_primary: if category.is_empty() {
                    None
                } else {
                    Some(category.to_string())
                },
                transaction_type: if amount_cents >= 0 {
                    "income".to_string()
                } else {
                    "expense".to_string()
                },
            })
        })
        .collect()
}

fn header_index(headers: &csv::StringRecord, name: &str) -> Option<usize> {
    headers
        .iter()
        .position(|header| header.trim().eq_ignore_ascii_case(name))
}

fn parse_amount_cents(raw: &str) -> Result<i64, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("amount is required".to_string());
    }

    let is_parenthesized = trimmed.starts_with('(') && trimmed.ends_with(')');
    let cleaned = trimmed
        .trim_matches(|char| char == '(' || char == ')')
        .replace(['$', ','], "");
    let value = cleaned
        .parse::<f64>()
        .map_err(|error| format!("invalid amount: {error}"))?;
    let signed_value = if is_parenthesized {
        -value.abs()
    } else {
        value
    };

    Ok((signed_value * 100.0).round() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_amount_dollars_to_cents() {
        assert_eq!(parse_amount_cents("12.34").unwrap(), 1234);
        assert_eq!(parse_amount_cents("-12.34").unwrap(), -1234);
        assert_eq!(parse_amount_cents("($1,234.56)").unwrap(), -123456);
    }

    #[test]
    fn parses_csv_rows_with_expected_columns() {
        let rows = parse_csv_rows(
            "date,description,amount,category\n2026-06-01,Coffee,-4.25,Dining\n2026-06-02,Payroll,1000.00,Income\n",
        );

        assert_eq!(rows.len(), 2);
        let first = rows[0].as_ref().unwrap();
        assert_eq!(first.raw_description, "Coffee");
        assert_eq!(first.amount_cents, -425);
        assert_eq!(first.transaction_type, "expense");

        let second = rows[1].as_ref().unwrap();
        assert_eq!(second.amount_cents, 100000);
        assert_eq!(second.transaction_type, "income");
    }
}
