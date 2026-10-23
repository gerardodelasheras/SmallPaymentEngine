Zone discipline: the edit in 1.1 is specs-zone and must be committed on its own before scaffolding; never commit specs-zone and code-zone files together (`tasks.md` is zone-neutral). Work acceptance-first: make a scenario fail for the right reason, implement the smallest change, then commit.

## 1. First-time setup (skip this section if `acceptance-tests/` already exists)

- [x] 1.1 Confirm the acceptance stack: read `stack:` from `openspec/config.yaml` (`javascript` or `python`). If absent, ask and add it - this is a specs-zone edit and must be committed on its own, before any scaffolding
- [x] 1.2 Set up the application skeleton (Cargo package, dependencies, entry point) so `cargo run -- <path>` boots and exits
- [x] 1.3 Create `acceptance-tests/` at the repo root as an independent `<stack>` project that boots the app before the suite and shuts it down after
- [x] 1.4 Copy the acceptance-test-authoring skill's `<stack>` reference files verbatim into `acceptance-tests/` - see `references/<stack>/SETUP.md` "Files to copy"; destination filenames are load-bearing - so the runner extracts the Gherkin from every `spec.md` under `openspec/` into `acceptance-tests/.extracted/` and discovers the extracted features, excluding `openspec/changes/archive/`
- [x] 1.5 Make the single test command always generate an HTML report under `acceptance-tests/reports/`
- [x] 1.6 Add the spec-lint command (extract, then `gherkin-lint .extracted`) and gitignore `acceptance-tests/.extracted/` and `acceptance-tests/reports/`
- [x] 1.7 Write `acceptance-tests/README.md` with instructions for running the suite and where the HTML report is written

## 2. Acceptance step definitions (one task per capability; red → green → commit)

- [x] 2.1 `amount-arithmetic`: implement the shared domain steps (deposit, withdraw, dispute, account assertions, row skipping) and make every scenario in `specs/amount-arithmetic/spec.md` pass
- [x] 2.2 `csv-ingestion`: implement steps for headers, padded fields, unknown types, invalid identifiers, amount presence, invalid UTF-8, blank lines, and stream continuation; make `specs/csv-ingestion/spec.md` pass
- [x] 2.3 `transaction-processing`: implement steps for deposit, withdrawal, dispute, resolve, chargeback, duplicate identifiers, cross-client references, input ordering, and client initialization; make `specs/transaction-processing/spec.md` pass
- [x] 2.4 `account-reporting`: implement output-parsing steps for the header, canonical amounts, row set and count, total invariant, and the locked column; make `specs/account-reporting/spec.md` pass
- [x] 2.5 `processor-cli`: implement invocation, stdout/stderr, exit-status, trace-gating, and infrastructure-failure steps; make `specs/processor-cli/spec.md` pass

## 3. Library unit tests (TDD, per module)

- [x] 3.1 `src/amount.rs`: parsing (whole, fractional, leading dot, whitespace, four-digit limit), rejection (negative, malformed, overflow), canonical rendering (zero, negative, one-unit fraction), and checked arithmetic
- [x] 3.2 `src/account/account.rs`: zeroed default, `total = available + held`, credit/debit/hold/release/chargeback, and overflow leaving balances unchanged
- [x] 3.3 `src/transaction/transaction_type.rs`, `transaction_id.rs`, `transaction_entry.rs`: type parsing, identifier bounds, and settled/disputed transitions
- [x] 3.4 `src/client/id.rs` and `src/client/dispatcher.rs`: client affinity, least-loaded assignment, capacity tie-breaking, round-robin tie-breaking, and stable pinning
- [x] 3.5 `src/transaction/parser.rs`: header handling, flexible column counts, trimming, amount presence rules, invalid UTF-8 rows, and per-row `Result`
- [x] 3.6 `src/transaction/transaction_processor.rs`: every FSM path including overdraw, re-dispute after resolve, unknown and duplicate `tx`, cross-client references, locked-account rejection, and zeroed initialization
- [x] 3.7 `src/worker_pool.rs`: bounded queue capacity, closed dispatch/collection channels, backpressure, snapshot ordering, and shutdown/join
- [x] 3.8 `src/pipeline.rs`, `src/account/report.rs`, `src/processing_error.rs`, `src/main.rs`: single-pass ingestion, merged output, recoverable-vs-fatal errors, stdout purity, and exit codes

## 4. Integration and regression coverage

- [x] 4.1 Write the independent `i64` reference oracle in `tests/integration.rs` with no shared implementation with the processor
- [x] 4.2 Compare the processor against the oracle for seeded random sequences and a 10,000-row generated input
- [x] 4.3 Add table-driven edge cases: insufficient funds, dispute/resolve, chargeback locking, cross-client disputes, unknown transactions, duplicate disputes, invalid resolve/chargeback, decimal precision, leading-dot amounts, header-only input, oversized identifiers and amounts, non-ASCII and invalid UTF-8, and malformed-then-valid streams
- [x] 4.4 Optionally add synthetic `tests/transactions*.csv` fixtures and `tests/run_tests.sh`; ensure fixtures are generated, differ from the SPEC examples, and are never sample or production input data
- [x] 4.5 Run `cargo test` and the optional `./tests/run_tests.sh`; both are green

## 5. Documentation, delivery, and validation

- [x] 5.1 Update `README.md` with the verification approach and reconcile documented deviations: `UnknownTx` vs `NotOwner`, disputes on withdrawals, output precision vs SPEC §2.3, and the delivery contents
- [x] 5.2 Verify `cargo build --release` compiles trace logging out while `RUST_LOG=trace` still works in dev/test
- [x] 5.3 Sanitize the deliverable: no company or challenge references in `src/`, comments, tests, or `README.md`; no sample or production input CSV committed
- [x] 5.4 Confirm the publication set matches the project delivery list (`src/`, Cargo files, `tests/`, `README.md`)
- [x] 5.5 Run `openspec validate implement-transaction-processor --type change --strict`

## 6. Completion

- [x] 6.1 Run the full acceptance suite: every scenario passes, zero pending/undefined steps, HTML report generated
- [x] 6.2 Run `cargo test` (and `./tests/run_tests.sh` if enabled); all green
- [x] 6.3 Hand off to archive per the project archive guidance (run OpenSpec Sync first when delta specs exist)
