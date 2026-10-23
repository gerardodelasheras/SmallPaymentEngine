## Why

The parser hand-indexes `StringRecord` fields and duplicates header handling that the `csv`/serde stack can express declaratively. Replacing the row path with serde deserialization removes bespoke parsing while keeping validation and failure behavior explicit.

## What Changes

- Replace the hand-written row parser with serde deserialization into a raw row type, followed by domain validation into `Transaction`.
- Add a small name-based header gate: `type`, `client`, `tx`, and `amount` are matched by exact name in any order; extra columns are ignored; a missing header, missing required column, or differently-cased name fails the run.
- **BREAKING (input compatibility):** header names are now matched exactly (case-sensitive, no BOM stripping); uppercase or BOM-prefixed headers that previously worked are rejected as malformed.
- **BREAKING (library API):** `Transactions<R>` is removed and `transactions` returns `Result<impl Iterator<Item = Result<Transaction, ProcessingError>>, ProcessingError>`; the pipeline keeps its `?`.
- Invalid or out-of-range `client`/`tx` values now surface as `ProcessingError::Csv` instead of `InvalidClient`/`InvalidTransactionId`; the rows are still skipped.
- **BREAKING (error API):** remove the now-unused `InvalidClient`, `InvalidTransactionId`, and `MissingField` variants; keep `InvalidHeader` for the gate and `InvalidType` carrying the raw trimmed type text.
- Preserve valid-row behavior: field trimming, the five accepted types, unknown-type rejection, required amounts for monetary rows, empty-or-absent amounts for the rest, and 3-column non-monetary rows when `amount` is the final column.
- Update parser unit tests and add `csv-ingestion` scenarios for reordered columns, extra columns, missing columns, and uppercase headers.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `csv-ingestion` — header recognition and malformed-header contract:
  - `specs/csv-ingestion/spec.md`
  - **The input has a header and four columns** changes: columns are matched by exact name in any order; extra columns are ignored; a header missing a required column or using different casing fails the run; a short non-monetary row is valid only when `amount` is the final column.
  - New scenarios: reordered columns accepted; an extra column ignored; a missing-column header fails; an uppercase header is rejected.
  - The other `csv-ingestion` requirements are behaviorally unchanged. No `processor-cli` delta is needed: malformed headers remain fatal and valid-input output/exit behavior is unchanged.

## Impact

- Code: `src/transaction/parser.rs` (raw row, header gate, tests), `src/processing_error.rs` (remove three variants), and import/re-export adjustments in `src/transaction/mod.rs`. `src/pipeline.rs` keeps its `?`; only the concrete iterator type changes.
- Public API: `Transactions<R>` and three error variants are removed; `transactions` remains fallible but returns an opaque iterator. This is breaking for library consumers.
- Behavior: valid input is unchanged and ingestion stays single-pass and streaming. Header matching is stricter (case/BOM), identifier failures are classified as `Csv`, and malformed headers still fail fast. Per-row error timing moves to iteration, which the pipeline already handles.
- Dependencies: none; `serde` and `csv` are already dependencies.
- Operations: no migration or output-schema change; rollback is a source revert.
- Risk: stricter header matching is the main compatibility change. The header gate prevents silent success and the previous silent acceptance of amount-bearing non-monetary rows when the `amount` column is renamed.
