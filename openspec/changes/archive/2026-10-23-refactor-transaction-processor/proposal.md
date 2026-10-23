## Why

`process` routes deposits and withdrawals through one catch-all helper that combines amount checks, duplicate checks, balance mutation, and index insertion, while dispute state lives in a mutable field alongside the entry. That makes the per-type state machine harder to review and extend than necessary. Splitting the dispatch into explicit handlers and representing settled versus disputed entries directly removes that accidental complexity without changing ledger or CLI behavior.

## What Changes

- Split `TransactionProcessor::process` into dedicated per-type handlers: `process_deposit`, `process_withdrawal`, `process_dispute`, `process_resolve`, and `process_chargeback`.
- Add a private `record_settled` helper as the single path that inserts an accepted monetary transaction into the transaction index.
- **BREAKING**: change the public `TransactionEntry` from a struct (`client`, `amount`, `kind`, `state`) into an enum with `Settled { client, amount }` and `Disputed { client, amount }`, and remove the public `EntryState`. The unused `kind: TransactionType` field is dropped because it is written but never read. `transaction_entry` stays public, so downstream construction, field access, and `EntryState` matching will no longer compile; no compatibility shim is planned.
- Preserve the current dispatch order and error precedence exactly:
  - initialize a zeroed account for every successfully parsed row before domain validation;
  - reject locked accounts before dispatching any operation;
  - check amount presence before duplicate transaction IDs in the monetary path;
  - resolve `UnknownTransaction` / `NotOwner` before state checks such as `AlreadyDisputed` / `NotDisputed`.
- Preserve atomic rejection: a rejected operation leaves balances and the transaction index unchanged. The only permitted state effect of a rejected row remains the pre-existing zeroed-account initialization.
- Preserve every current ledger rule: deposit and withdrawal arithmetic, insufficient-funds rejection, dispute hold including overdraw, duplicate-dispute rejection, resolve and re-dispute, chargeback locking, and duplicate-ID and foreign-owner rejection.
- Keep the enforcement scope of transaction-ID uniqueness unchanged at one processor/worker index, as today.
- Add focused processor unit tests for the paths being reshaped, especially missing-amount handling, rejection error precedence, and proof that a rejected duplicate does not overwrite the original index entry. Existing behavioral tests remain unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `transaction-processing` — adds one explicit spec-level requirement that rejected operations are atomic: balances and the accepted-transaction index are unchanged, apart from the already-specified zeroed-account initialization for a parsed row. No existing requirement behavior changes; the deposit, withdrawal, dispute, resolve, chargeback, indexing/ownership, ordering, and initialization requirements keep their current outcomes.

## Impact

- **Code:** `src/transaction/transaction_processor.rs` for dispatch, handlers, and unit tests; `src/transaction/transaction_entry.rs` for the enum and removal of `EntryState`. No changes to `worker_pool`, dispatcher, pipeline, parser, account, or amount code.
- **Public API:** breaking only for consumers of `TransactionEntry`, `EntryState`, and entry fields. The processor surface (`process`, `accounts`, `snapshot`) is unchanged, and `record_settled` is private.
- **Behavior:** no ledger, error, or CLI-output changes. The same accepted rows produce the same balances, lock state, index contents, and report; the same rejected rows produce the same errors with no balance or index mutation.
- **Specs:** one `transaction-processing` delta adding the atomicity requirement with scenarios; existing requirements are not modified.
- **Tests:** existing processor tests stay behaviorally identical and are expected to pass without weakening assertions. New tests cover missing amount, error precedence, and index non-mutation after a rejected duplicate.
- **Verification:** the acceptance suite is expected to pass unchanged, plus `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check`.
- **Dependencies and operations:** none added or removed. There is no on-disk state, migration, or input/output contract change; rollback is a source revert. The crate is not published, so the public-module break has no external migration path.
- **Risk:** the main risk is accidentally changing validation order or mutating before an error while reshaping handlers. Preserving the pre-dispatch initialization and lock gate, keeping the documented error precedence, and adding targeted tests guard against that. Dropping `kind` means any future need for transaction type on an entry must be reintroduced deliberately.
