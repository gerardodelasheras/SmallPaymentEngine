Zone discipline: this change touches the code zone (`src/`, `tests/`) after the specs-zone delta was authored; keep those commits separate (`tasks.md` is zone-neutral). Work acceptance-first: see the new scenarios fail for the right reason, then implement.

## 1. Acceptance scenarios (red → green)

- [x] 1.1 Confirm the new `csv-ingestion` scenarios with the existing step definitions: reordered and uppercase headers were red, extra-column and missing-column headers were already green; all pass after the parser change. No new step definitions were needed.

## 2. Parser simplification

- [x] 2.1 Add the exact-name header gate for `type`, `client`, `tx`, and `amount` (any order, extra columns allowed); keep missing, renamed, and differently-cased headers fatal with `InvalidHeader`.
- [x] 2.2 Replace `Transactions`/`parse_record` with `RawTransactionRow` + `StringRecord::deserialize`, padding short rows to the header width; keep `has_headers(true)`, `flexible(true)`, `Trim::All`, and `#[serde(default)]` on `amount`.
- [x] 2.3 Change `transactions` to return `Result<impl Iterator<Item = Result<Transaction, ProcessingError>>, ProcessingError>`; remove `Transactions<R>` and keep the pipeline's `?`.
- [x] 2.4 Keep `InvalidType(String)` and `InvalidHeader`; remove `InvalidClient`, `InvalidTransactionId`, and `MissingField`, and let identifier failures surface as `ProcessingError::Csv`.
- [x] 2.5 Update parser unit tests for the new signature, header gate, `Csv` identifier classification, 3-column non-monetary rows, and padded fields.

## 3. Verification

- [x] 3.1 Run the full acceptance suite: 72 scenarios pass (main specs plus the delta), zero pending or undefined steps.
- [x] 3.2 Run `cargo test` (unit tests plus the independent integration oracle).
- [x] 3.3 Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`.
- [x] 3.4 Run `cargo build --release` and confirm `RUST_LOG=trace` produces no trace output.
- [x] 3.5 Run `openspec validate simplify-transaction-parser --type change --strict`.

## 4. Completion

- [x] 4.1 Hand off to archive (run OpenSpec Sync first for the delta spec).
