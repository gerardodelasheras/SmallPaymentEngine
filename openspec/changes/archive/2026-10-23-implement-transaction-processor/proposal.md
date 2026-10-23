## Why

The repository contains a complete written contract for a streaming payments ledger (`SPEC.md` and `README.md`) but no implementation, so none of the documented behaviour can be executed or verified. This change delivers the first working processor: a buildable CLI backed by an independent correctness oracle and reproducible tests.

## What Changes

- Add the Cargo package and module layout described in the README: one shared library crate (`src/lib.rs`) holding all domain logic and a thin default binary (`src/main.rs`) exposing `cargo run -- transactions.csv > accounts.csv`.
- Add fixed-point money: `i64` scaled by `10_000`, decimal parsing with up to four fraction digits, leading-dot support, surrounding-whitespace trimming, checked arithmetic, overflow rejection, and canonical rendering. Floating point is excluded from parsing, ledger, and output.
- Add streaming CSV ingestion of `type,client,tx,amount`: validate the five operation types, `u16` client IDs, `u32` transaction IDs, and amount presence/format; trim rows; skip invalid rows without aborting. Row-level failures are skipped, while reader/I/O failures are fatal.
- Add the ledger state machine: deposit, withdrawal (reject when `available < amount`), dispute, resolve, and chargeback, with per-client `available`/`held`/`locked`, the `total = available + held` invariant, a historical transaction index, and rejection of operations on locked accounts.
- Add the documented behaviour assumptions, each recorded per SPEC §7.2 with its alternative:
  - Disputes follow the literal rule and may overdraw `available`; `total` stays `available + held`.
  - A resolved transaction may be disputed again.
  - A row that parses but fails domain validation may still initialize a zeroed client; a row rejected during parsing does not.
  - Non-monetary rows accept an empty or absent amount but reject a non-empty one.
  - A reference to a transaction not owned by the client is rejected without mutation.
  - A transaction already under active dispute is not disputed twice.
  - Locked accounts reject all later operations.
- Add streaming dispatch that preserves per-client transaction order and isolates row failures, so a malformed or rejected row never halts the run or mutates another client. Ingestion is single-pass with bounded in-flight work rather than a full-file buffer.
- Add account reporting: header `client,available,held,total,locked`, one row per initialized client, amounts with trailing zeros stripped and at least one decimal place, order unspecified.
- Add the CLI contract: exactly one positional input path, CSV-only stdout, diagnostics on stderr (silent by default, trace opt-in in dev/test and compiled out of release builds), and non-zero exit for infrastructure failures.
- Add required unit tests and an independent `i64` oracle with seeded random and edge-case coverage; optional synthetic CSV fixtures and a runner.
- Update `README.md` to describe the verification approach and delivery sanitization.
- No **BREAKING** changes: the repository has no source, package metadata, or public API yet.

## Capabilities

### New Capabilities

- `amount-arithmetic` — Fixed-point value semantics: scaled decimal parsing, canonical rendering, checked arithmetic, overflow handling, and the prohibition on floating point.
- `csv-ingestion` — Input schema and row-level validation/skipping: header and columns, trimming, type/identifier/amount rules, and the boundary between skippable row errors and fatal reader errors.
- `transaction-processing` — Deposit, withdrawal, dispute, resolve, and chargeback transitions; balances and locking; transaction history; ordering; ownership checks; error boundaries; and the documented behaviour assumptions above.
- `account-reporting` — Output contract: columns, row population for initialized clients, decimal formatting, and order-independent output.
- `processor-cli` — Process-level contract: single-argument invocation, stdout purity, stderr diagnostics and trace gating, exit codes, single-pass streaming, and row-failure isolation.

Client routing and the test harness are implementation and verification concerns handled in `design.md` and `tasks.md`; they do not define new externally observable capabilities.

### Modified Capabilities

None. `openspec/specs/` is empty and this is the first change, so no existing capability behaviour is modified.

## Impact

- **Code:** New `Cargo.toml` and `Cargo.lock`; the `src/` module tree described in the README; `tests/integration.rs`; optional `tests/run_tests.sh` and synthetic fixtures. No existing source is replaced, and `main.rs` stays thin CLI wiring over the library.
- **APIs:** Adds the process contract `cargo run -- transactions.csv > accounts.csv` and a reusable library surface centred on `pipeline::process`, account/reporting types, transaction parsing, and processing errors. No prior API exists to break.
- **Dependencies:** Tokio (runtime, channels, blocking-task orchestration), Serde and the `csv` crate for streaming input, `thiserror` for the unified error type, and a `RUST_LOG`-compatible logging facade/subscriber with a release-time trace cutoff. Test support needs a seeded RNG for reproducible oracle runs.
- **Systems:** Local filesystem input and stdout output only. No network services, database, distributed transport, or future streaming variants are introduced.
- **Memory:** Ingestion is single-pass and queue-bounded, so memory does not scale with the input buffer. Ledger state grows with initialized clients, and the transaction index grows with accepted transactions because disputes must resolve arbitrary earlier IDs. SPEC §1's O(1) claim is therefore unattainable with dispute support and is recorded as a documented deviation rather than claimed as compliance.
- **Cross-client references:** With worker-local transaction indexes, a reference to another client's transaction is rejected without mutation. The observable outcome is a skipped row; the diagnostic label may be indistinguishable from an unknown transaction unless a global ownership index is added (design decision).
- **Delivery and sanitization:** The published repository contains only sanitized `src/`, Cargo files, `tests/`, and a `README.md` that describes verification and mentions neither the requesting company nor the challenge source. Source comments are sanitized too, and no sample or production input CSV is committed. This follows the project's explicit delivery list where SPEC §7.3 says "`src/` + `README.md` only"; the discrepancy is documented.
- **Fixtures:** Any committed regression fixtures are synthetic and distinct from the SPEC examples; the runner and fixtures remain optional, while unit tests and the independent oracle are required.
- **Verification deliverable:** `README.md` is updated to describe the verification approach (sample data, unit tests, oracle) so the delivery satisfies SPEC §7.1/§7.4.
