## Context

`transaction::parser` currently owns a `Transactions<R>` iterator, reads `StringRecord` fields positionally, and validates the header against `["type", "client", "tx", "amount"]` before iteration. `pipeline::read_and_dispatch` calls `transactions(reader)?`, so a bad header is fatal. `serde` and `csv` are already dependencies.

In-force ADRs 0001 (fixed worker pool), 0002 (fixed-point `i64` money), and 0003 (single-pass streaming lifecycle) are unaffected; this change stays inside the ingestion component and does not supersede any of them.

Diagram conventions: plain Mermaid, lightweight C4-inspired. One component/data-flow view is enough for an internal parser refactor; a system-context or container view would not answer a new question.

### Component / data-flow view

```mermaid
flowchart LR
  Pipeline["pipeline::read_and_dispatch"] --> Parser["transaction::parser::transactions"]
  Parser --> Gate["header gate<br/>exact names, any order, extras allowed"]
  Gate -->|"missing/renamed/uppercase header"| Fatal["ProcessingError::InvalidHeader (fatal)"]
  Gate -->|"valid header"|   Csv["csv::Reader + serde record deserialize<br/>short rows padded"]
  Csv -->|"per-row csv::Error"| Skip["ProcessingError::Csv (skippable)"]
  Csv -->|"RawTransactionRow"| Validate["into_transaction<br/>type + amount rules"]
  Validate -->|"ok"| Tx["Transaction"]
  Validate -->|"domain error"| Skip
  Tx --> Pipeline
```

## Goals / Non-Goals

**Goals:**

- Replace hand-indexed record parsing with serde row deserialization.
- Keep valid-row behavior identical: trimming, five type names, amount presence rules, 3-column non-monetary rows.
- Keep malformed headers fatal instead of letting rows silently disappear.
- Continue streaming one row at a time.
- Remove parser-only error variants that are no longer constructed.

**Non-Goals:**

- Any change to ledger, account, amount, routing, or output behavior for valid input.
- Preserving the old case-insensitive and BOM-stripping header tolerance.
- New dependencies, output-schema changes, or a stored configuration format.

## Decisions

### D1. Deserialize through a raw row type

Use `RawTransactionRow { transaction_type: String, client: ClientId, transaction_id: TransactionId, amount: String }` with serde renames for `type`/`tx` and a default for `amount`. Deserialize each record with `StringRecord::deserialize` after padding short records to the header width, then validate into `Transaction` in `into_transaction`.

- **Rationale:** String fields keep the existing domain errors (`InvalidType`, `InvalidAmount`, amount presence); integer fields make identifier failures deserialization errors.
- **Alternatives:** Deserialize `type`/`amount` into domain types (collapses their specific errors into `Csv`); keep manual parsing (status quo).

### D2. Keep a name-based header gate

Before deserialization, require the four exact names `type`, `client`, `tx`, and `amount` in any order; allow extra columns; return `InvalidHeader` for a missing header, missing required column, or differently-cased name.

- **Rationale:** Serde alone would turn a malformed header into per-row skips (successful run, empty report), and a renamed `amount` column can otherwise silently accept an amount-bearing dispute. The gate keeps fail-fast behavior and must use the same exact matching as serde so the two cannot disagree.
- **Alternatives:** Rely on serde only (rejected: silent success/acceptance); normalize case and BOM in the gate (rejected: creates a gate/serde mismatch and extra code).

### D3. Return a fallible opaque iterator

`transactions` returns `Result<impl Iterator<Item = Result<Transaction, ProcessingError>>, ProcessingError>`; `Transactions<R>` is removed. `pipeline::read_and_dispatch` keeps its `?`.

- **Rationale:** Header errors stay fatal while row errors stay per-row; the concrete iterator type is an implementation detail.
- **Alternatives:** Bare `impl Iterator` without a `Result` (rejected: loses the fatal header path); a named wrapper (more code for no caller benefit).

### D4. Preserve CSV reader settings

Keep `has_headers(true)`, `flexible(true)`, and `Trim::All`, and keep `#[serde(default)]` on `amount`. Because `csv::into_deserialize` errors on a short record before serde defaults run, pad each short record with empty fields to the header width and deserialize it with `StringRecord::deserialize`.

- **Rationale:** These settings carry the padded-field and 3-column-row behavior. With the standard header, a short row is valid only when `amount` is the final column.
- **Alternatives:** Require equal-length records (rejected: breaks non-monetary rows); rely on `csv::into_deserialize` (rejected: it rejects short rows before defaults); trim manually in `into_transaction` (duplicate work).

### D5. Trim the parser-specific error surface

Keep `InvalidType(String)` and `InvalidHeader`; remove `InvalidClient`, `InvalidTransactionId`, and `MissingField`, which the serde path no longer constructs. Identifier failures surface as `ProcessingError::Csv`.

- **Rationale:** Leaving non-constructible public variants is misleading, and the change is already breaking at the library boundary.
- **Alternatives:** Retain them for compatibility (no in-repo consumer and no stated external contract); deserialize identifiers as strings to keep `InvalidClient` (defeats the simplification).

### D6. Update both test layers

Update the parser unit tests for the new signature, header gate, and `Csv` classification, and add the delta scenarios for reordered columns, extra columns, missing columns, and uppercase headers. The independent integration oracle is unchanged.

- **Rationale:** The acceptance scenarios are the executable contract; the oracle covers ledger behavior, not parsing.

## Risks / Trade-offs

- **Case/BOM strictness breaks previously tolerated input** → documented as breaking; malformed headers produce a clear `InvalidHeader` error.
- **Public library API break** (`Transactions<R>` and three variants removed) → only the in-repo pipeline uses `transactions`; the change is planned as breaking.
- **Identifier diagnostics become `Csv`** → rows are still skipped; only the diagnostic label changes.
- **`csv::into_deserialize` rejects short rows before serde defaults run** → short records are padded to the header width and deserialized with `StringRecord::deserialize`; the 3-column non-monetary scenario verifies it.
- **Unknown columns** → serde ignores unknown fields by default; the extra-column scenario verifies it.
- **Per-row error timing moves into iteration** → the pipeline already consumes `Result` per row.
- **A renamed `amount` column could silently change semantics** → the header gate rejects it up front.

## Migration Plan

No data or schema migration. The parser and pipeline are updated together; rollback is a source revert. The acceptance suite gains the new header scenarios, and the existing ledger tests stay green.

## Open Questions

- Confirm the case/BOM compatibility drop is acceptable (recommended: yes, for a simpler exact-match gate; normalizing would add code and a gate/serde mismatch risk).
- Confirm removing `InvalidClient`, `InvalidTransactionId`, and `MissingField` now rather than deprecating them (recommended: remove; no in-repo consumer).
- Confirm the opaque iterator return type is acceptable (recommended: yes; the concrete `Transactions<R>` type is not a useful contract).
- Resolved during apply: short records are padded to the header width and deserialized with `StringRecord::deserialize`, because `csv::into_deserialize` errors before serde defaults; the 3-column scenario passes.
