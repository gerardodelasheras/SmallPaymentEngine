Zone discipline: the specs-zone delta is authored before this code-zone work; keep subsequent code commits separate from `openspec/` spec edits (`tasks.md` is zone-neutral). This is a behavior-preserving refactor, so acceptance scenarios are a regression contract rather than a new red phase.

## 1. Acceptance baseline

- [x] 1.1 Run the acceptance suite and confirm the existing `transaction-processing` scenarios plus the three new atomicity scenarios pass against the current implementation; record any scenario that does not resolve to an existing step.
- [x] 1.2 If a new scenario does not resolve, add only the missing step definition first (red → green) before touching `src/`.

## 2. Entry model

- [x] 2.1 Replace the `TransactionEntry` struct and `EntryState` with `TransactionEntry::Settled { client, amount }` and `TransactionEntry::Disputed { client, amount }`; remove the unused `kind` field.
- [x] 2.2 Update `transaction_processor.rs` imports and matches so the crate compiles against the enum before changing processor logic.

## 3. Processor dispatch

- [x] 3.1 Keep zeroed-account initialization for every parsed row and the locked-account rejection as the `process` prelude; replace the type match with direct calls to `process_deposit`, `process_withdrawal`, `process_dispute`, `process_resolve`, and `process_chargeback`, and delete `apply_monetary` and its `unreachable!` arm.
- [x] 3.2 Implement `process_deposit` and `process_withdrawal` in the fixed order: require the amount, reject a duplicate transaction ID, mutate the balance, then record the entry.
- [x] 3.3 Add the private infallible `record_settled` helper and use it as the only monetary-entry insertion path.
- [x] 3.4 Implement `process_dispute`: lookup the entry and verify ownership, reject unless settled, hold the amount, then replace the entry with `Disputed`.
- [x] 3.5 Implement `process_resolve`: lookup the entry and verify ownership, reject unless disputed, release the amount, then replace the entry with `Settled`.
- [x] 3.6 Implement `process_chargeback`: lookup the entry and verify ownership, reject unless disputed, charge back the amount, then remove the entry.
- [x] 3.7 Replace remaining `expect("account initialized")` accesses with an infallible account accessor; do not add new error variants or panic paths.

## 4. Unit tests

- [x] 4.1 Keep every existing `transaction_processor` test assertion unchanged and confirm it passes after the refactor.
- [x] 4.2 Add a missing-amount test: a monetary transaction with no amount returns `MissingAmount`, leaves the client initialized at zero, and leaves the transaction ID free for a later accepted transaction.
- [x] 4.3 Add a duplicate-rejection index test: after a duplicate deposit is rejected, disputing the original ID holds the original amount, not the rejected amount.
- [x] 4.4 Add an error-precedence test: a locked account rejects a missing-amount monetary transaction with `AccountLocked` before `MissingAmount`.

## 5. Verification

- [x] 5.1 Run `cargo test` and confirm unit tests and the integration oracle pass.
- [x] 5.2 Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`.
- [x] 5.3 Run the full acceptance suite and confirm every scenario passes with zero pending or undefined steps and the HTML report is generated.
- [x] 5.4 Run `openspec validate refactor-transaction-processor --type change --strict`.
- [x] 5.5 Run `cargo build --release` and confirm the CLI behavior is unchanged.

## 6. Completion

- [x] 6.1 Verify code-zone and specs-zone commits stayed separate, then hand off to the archive workflow after implementation is merged and the delta is synced.
