-- One-off category backfill from stored Plaid raw events.
-- Safe to re-run: fills blank categories and reclassifies transfer/payment noise.

CREATE OR REPLACE FUNCTION format_plaid_pfc_label(raw text)
RETURNS text
LANGUAGE plpgsql
IMMUTABLE
AS $$
DECLARE
  word text;
  words text[] := string_to_array(trim(raw), '_');
  result text[] := '{}';
  lower_word text;
  formatted text;
BEGIN
  IF raw IS NULL OR trim(raw) = '' THEN
    RETURN NULL;
  END IF;

  FOREACH word IN ARRAY words LOOP
    IF word = '' THEN
      CONTINUE;
    END IF;

    lower_word := lower(word);
    IF lower_word = 'and' THEN
      formatted := lower_word;
    ELSE
      formatted := upper(left(lower_word, 1)) || substr(lower_word, 2);
    END IF;

    result := array_append(result, formatted);
  END LOOP;

  RETURN array_to_string(result, ' ');
END;
$$;

CREATE OR REPLACE FUNCTION format_plaid_pfc_detailed(raw_primary text, raw_detailed text)
RETURNS text
LANGUAGE plpgsql
IMMUTABLE
AS $$
DECLARE
  stripped text;
BEGIN
  IF raw_detailed IS NULL OR trim(raw_detailed) = '' THEN
    RETURN NULL;
  END IF;

  IF raw_primary IS NOT NULL AND raw_primary <> '' THEN
    stripped := regexp_replace(raw_detailed, '^' || raw_primary || '_', '');
    IF stripped = raw_detailed THEN
      stripped := raw_detailed;
    END IF;
  ELSE
    stripped := raw_detailed;
  END IF;

  RETURN format_plaid_pfc_label(stripped);
END;
$$;

CREATE OR REPLACE FUNCTION looks_like_poker_detailed_sql(
  raw_description text,
  merchant_name text
)
RETURNS text
LANGUAGE plpgsql
IMMUTABLE
AS $$
DECLARE
  text_value text := lower(coalesce(raw_description, '') || ' ' || coalesce(merchant_name, ''));
  collapsed text;
BEGIN
  IF text_value ~* 'pure social' THEN
    RETURN 'Pure Social';
  END IF;

  IF text_value ~* 'pure poker' THEN
    RETURN 'Pure Poker';
  END IF;

  collapsed := regexp_replace(text_value, '\s+', '', 'g');
  IF collapsed ~* 'clubwptgold' THEN
    RETURN 'ClubWPT Gold';
  END IF;

  RETURN NULL;
END;
$$;

CREATE OR REPLACE FUNCTION looks_like_transfer_payment_sql(
  raw_description text,
  merchant_name text
)
RETURNS boolean
LANGUAGE sql
IMMUTABLE
AS $$
  SELECT lower(coalesce(raw_description, '') || ' ' || coalesce(merchant_name, '')) ~*
    '(payment thank you|autopay payment|autopay|directpay|credit card payment|online payment|mobile payment|payment to .*( card |card ending)|real time transfer|zelle payment )';
$$;

CREATE OR REPLACE FUNCTION transfer_payment_detailed_sql(
  raw_description text,
  category_detailed text
)
RETURNS text
LANGUAGE plpgsql
IMMUTABLE
AS $$
DECLARE
  text_value text := lower(coalesce(raw_description, ''));
BEGIN
  IF text_value ~* '(payment thank you|credit card payment)' THEN
    RETURN 'Credit Card Payment';
  END IF;

  IF text_value ~* 'payment to .*card' THEN
    RETURN 'Credit Card Payment';
  END IF;

  IF text_value ~* 'zelle' THEN
    RETURN 'Zelle';
  END IF;

  IF text_value ~* 'real time transfer' THEN
    RETURN 'Bank Transfer';
  END IF;

  IF NULLIF(trim(category_detailed), '') IS NOT NULL THEN
    RETURN category_detailed;
  END IF;

  RETURN 'Internal Transfer';
END;
$$;

CREATE OR REPLACE FUNCTION normalize_tx_type_sql(
  amount_cents bigint,
  category_primary text
)
RETURNS text
LANGUAGE plpgsql
IMMUTABLE
AS $$
DECLARE
  category text := lower(coalesce(category_primary, ''));
BEGIN
  IF category ~* '(transfer|payment|loan disbursement|loan payment)' THEN
    RETURN 'transfer';
  ELSIF category ~* 'fee' THEN
    RETURN 'fee';
  ELSIF category ~* 'interest' THEN
    RETURN 'interest';
  ELSIF category ~* 'refund' THEN
    RETURN 'refund';
  ELSIF amount_cents > 0 THEN
    RETURN 'income';
  ELSIF amount_cents < 0 THEN
    RETURN 'expense';
  END IF;

  RETURN 'other';
END;
$$;

WITH latest_plaid AS (
  SELECT DISTINCT ON (tx->>'transaction_id')
    tx->>'transaction_id' AS provider_transaction_id,
    tx->'personal_finance_category'->>'primary' AS pfc_primary,
    tx->'personal_finance_category'->>'detailed' AS pfc_detailed,
    tx->'category'->>0 AS legacy_primary,
    tx->'category'->>1 AS legacy_detailed
  FROM raw_provider_events r,
       jsonb_array_elements(r.payload_json->'transactions') tx
  WHERE r.endpoint = 'transactions/get'
  ORDER BY tx->>'transaction_id', r.received_at DESC
),
derived AS (
  SELECT
    t.id,
    CASE
      WHEN looks_like_poker_detailed_sql(t.raw_description, t.merchant_name) IS NOT NULL
        THEN 'Poker'
      WHEN looks_like_transfer_payment_sql(t.raw_description, t.merchant_name) THEN 'Transfer'
      WHEN NULLIF(trim(format_plaid_pfc_label(p.pfc_primary)), '') IS NOT NULL
        THEN format_plaid_pfc_label(p.pfc_primary)
      WHEN NULLIF(trim(p.legacy_primary), '') IS NOT NULL THEN trim(p.legacy_primary)
      WHEN NULLIF(trim(t.category_primary), '') IS NOT NULL THEN trim(t.category_primary)
      ELSE NULL
    END AS new_category_primary,
    CASE
      WHEN looks_like_poker_detailed_sql(t.raw_description, t.merchant_name) IS NOT NULL
        THEN looks_like_poker_detailed_sql(t.raw_description, t.merchant_name)
      WHEN looks_like_transfer_payment_sql(t.raw_description, t.merchant_name)
        THEN transfer_payment_detailed_sql(t.raw_description, NULL)
      WHEN NULLIF(trim(format_plaid_pfc_detailed(p.pfc_primary, p.pfc_detailed)), '') IS NOT NULL
        THEN format_plaid_pfc_detailed(p.pfc_primary, p.pfc_detailed)
      WHEN NULLIF(trim(p.legacy_detailed), '') IS NOT NULL THEN trim(p.legacy_detailed)
      WHEN NULLIF(trim(t.category_detailed), '') IS NOT NULL THEN trim(t.category_detailed)
      ELSE NULL
    END AS new_category_detailed,
    t.amount_cents,
    t.raw_description,
    t.merchant_name
  FROM transactions t
  LEFT JOIN latest_plaid p
    ON p.provider_transaction_id = t.provider_transaction_id
)
UPDATE transactions t
SET
  category_primary = d.new_category_primary,
  category_detailed = d.new_category_detailed,
  transaction_type = CASE
    WHEN looks_like_transfer_payment_sql(d.raw_description, d.merchant_name) THEN 'transfer'
    ELSE normalize_tx_type_sql(d.amount_cents, d.new_category_primary)
  END
FROM derived d
WHERE t.id = d.id
  AND (
    NULLIF(trim(t.category_primary), '') IS NULL
    OR looks_like_poker_detailed_sql(t.raw_description, t.merchant_name) IS NOT NULL
    OR looks_like_transfer_payment_sql(t.raw_description, t.merchant_name)
    OR (
      normalize_tx_type_sql(d.amount_cents, d.new_category_primary) = 'transfer'
      AND t.transaction_type <> 'transfer'
    )
  );
