Zone discipline: the specs-zone delta is authored first; keep the subsequent code-zone edits in a separate commit from `openspec/` spec edits (`tasks.md` is zone-neutral). The CLI contract is expanded, but no new acceptance step definition is needed because worker loss cannot be triggered through the public CLI.

## 1. Acceptance baseline

- [ ] 1.1 Run the acceptance suite and confirm the existing `processor-cli` scenarios still pass; verify the modified infrastructure-failure requirement introduces no undefined steps.
- [ ] 1.2 Record that worker/channel loss remains covered by worker-pool unit tests rather than a CLI-inducible acceptance step.

## 2. Public surface and command model

- [ ] 2.1 Demote `worker_pool` to `pub(crate)` in `src/lib.rs`, make `WorkerPool` crate-visible, and make `Command` and `QUEUE_CAPACITY` private.
- [ ] 2.2 Replace the sender accessors with `worker_count`, `capacities`, and `dispatch_to(worker, transaction)`; remove `senders()`, `sender()`, and `send_blocking()`.
- [ ] 2.3 Move the worker-count policy into the pool: `new()` clamps `available_parallelism()` to `1..=16`, with a crate-visible constructor for deterministic tests.
- [ ] 2.4 Replace `Snapshot` and `Shutdown` with `Process` and `TakeAccounts(oneshot::Sender<HashMap<ClientId, Account>>)`.

## 3. Terminal drain

- [ ] 3.1 Add `pub(crate) fn take_accounts(&mut self) -> HashMap<ClientId, Account>` to `TransactionProcessor`, using `std::mem::take`; keep `snapshot()` unchanged.
- [ ] 3.2 Make `TakeAccounts` terminal in the worker loop: reply with the moved accounts, then break; no command is processed afterward.
- [ ] 3.3 Implement `take_accounts(mut self)` to request every worker's accounts, retain the first error, drop all senders, and await every handle even on failure.
- [ ] 3.4 Map send, reply, and `JoinError` failures to `WorkerUnavailable`; discard collected accounts and emit no partial output when the drain fails.

## 4. Pipeline ownership and dispatch

- [ ] 4.1 Move the pool into the `spawn_blocking` producer closure and return `(WorkerPool, Result<(), ProcessingError>)` from it.
- [ ] 4.2 Recover the pool after a successful join, call `take_accounts` even when dispatch failed, and preserve dispatch-error-before-drain-error precedence.
- [ ] 4.3 Map a blocking-task join failure to `WorkerUnavailable` and skip accounts output because the pool is unavailable.
- [ ] 4.4 Route dispatch through `pool.dispatch_to(...)`; remove the cloned sender vector and per-row sender access.

## 5. Tests

- [ ] 5.1 Add a drain test: process a deposit, call `take_accounts`, assert the returned accounts and that the worker exited.
- [ ] 5.2 Add a rejected-row test: a rejected row followed by a valid row still applies the valid row.
- [ ] 5.3 Add a closed-channel test: `dispatch_to` and `take_accounts` return `WorkerUnavailable` when a receiver is gone.
- [ ] 5.4 Add a deterministic backpressure test using a start signal and a blocked `dispatch_to`, with no sleep-based race.
- [ ] 5.5 Add a worker-panic test: a deliberately panicking handle makes `take_accounts` return `WorkerUnavailable`.
- [ ] 5.6 Confirm the existing capacity, ordering, ledger, and integration tests still pass with the narrow API.

## 6. Verification

- [ ] 6.1 Run `cargo test` and confirm unit tests and the integration oracle pass.
- [ ] 6.2 Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`.
- [ ] 6.3 Run the full acceptance suite and confirm zero pending or undefined steps and an HTML report.
- [ ] 6.4 Run `tests/run_tests.sh` and confirm the committed CSV regression outputs are byte-identical.
- [ ] 6.5 Run `openspec validate harden-worker-pool-runtime --type change --strict`.
- [ ] 6.6 Run `cargo build --release` and confirm the CLI succeeds on a representative input.
- [ ] 6.7 Review README architecture text for references to removed commands or sender accessors; update only if needed.

## 7. Completion

- [ ] 7.1 Verify code-zone and specs-zone commits stayed separate, then hand off to sync/archive after implementation is merged.
