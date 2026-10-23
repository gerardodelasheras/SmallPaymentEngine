## Why

The worker pool exposes its channel topology: callers can clone senders, dispatch after a snapshot, and depend on a public command enum. Account collection is also split across separate snapshot and shutdown steps and clones the account map. Consolidating the pool into a crate-private component with one consuming, terminal drain makes bounded ownership, one-shot collection, and no-post-collection-work enforceable by construction instead of convention.

## What Changes

- Demote `worker_pool` to `pub(crate)`, make `WorkerPool` crate-visible, and keep its senders and handles private. Make `Command` and `QUEUE_CAPACITY` implementation details.
- Narrow the pool surface to `worker_count`, `capacities`, `dispatch_to(worker, transaction)`, and `take_accounts`. Remove `senders()`, `sender()`, and `send_blocking()`.
- Let the pool own the worker-count policy: `WorkerPool::new()` uses `available_parallelism().clamp(1, 16)`. Keep a crate-visible constructor for deterministic tests.
- Replace `Snapshot` and `Shutdown` with `Process` and `TakeAccounts(oneshot::Sender<HashMap<ClientId, Account>>)`. A worker replies with its accounts, moved out with `std::mem::take`, then exits; no command is processed afterward.
- Add a crate-visible `TransactionProcessor::take_accounts` that moves the account map out. Keep the existing `snapshot()` method unchanged to avoid an additional public break.
- Make `take_accounts(mut self)` terminal on every path: request every worker's accounts, retain the first error, drop all senders, and await every worker handle even when a send or reply fails. A worker `JoinError` surfaces as `WorkerUnavailable` so a panic is not silently ignored.
- Move the pool into the `spawn_blocking` producer task and return it alongside the dispatch result, preserving the rule that the producer finishes before accounts are collected. On a blocking-task join failure the pool is lost with the task; return `WorkerUnavailable` and do not emit partial output.
- Dispatch through `pool.dispatch_to(...)` instead of a cloned sender vector, removing per-row sender access from `pipeline`.
- Keep the existing failure distinction: row-level CSV and validation failures stay `trace!` and are skipped; loss of a worker or channel stays an `error!`/non-recoverable `WorkerUnavailable`.
- Add tests for processed accounts, rejected rows not stopping a worker, closed-channel rejection, deterministic backpressure without sleep-based timing, and terminal drain behavior.
- **BREAKING (library API):** `worker_pool` is no longer a public module, `WorkerPool` is no longer public, and `Command`, `QUEUE_CAPACITY`, `senders()`, `sender()`, and `send_blocking()` are no longer exposed. There are no in-repository consumers outside `pipeline` and module tests.
- Preserve user-facing behavior: CSV ingestion, accounts output, ledger rules, CLI arguments, exit status, and row-skip semantics are unchanged.

## Capabilities

### New Capabilities

None. This is internal runtime hardening and introduces no new user-facing capability.

### Modified Capabilities

- `processor-cli`: expand the existing **Infrastructure failures end the run** requirement to state that loss of a worker or its channel is also an infrastructure failure. This is a contract expansion, not a valid-input behavior change. The delta lives at `specs/processor-cli/spec.md`; the existing missing-input scenario remains, and no new CLI-inducible scenario is added because worker loss cannot be triggered through the public CLI. Unit tests cover closed-channel behavior.

## Impact

- **Code:** `src/worker_pool.rs` (visibility, command set, dispatch surface, terminal drain, tests), `src/pipeline.rs` (pool ownership and dispatch), `src/transaction/transaction_processor.rs` (`take_accounts`), `src/lib.rs` (module visibility), and worker-pool tests.
- **Public API:** breaking for consumers of `worker_pool`; the crate is unpublished and the repository has no external consumer. `TransactionProcessor::snapshot` remains available.
- **Behavior:** no change for valid input or existing rejection paths. Runtime guarantees become stricter: the pool cannot be cloned, senders cannot escape, collection is terminal, and worker panics are reported instead of ignored.
- **Tests:** worker-pool unit tests are rewritten for the narrow API and add deterministic backpressure and terminal-drain coverage; pipeline and integration tests continue to exercise existing behavior. Acceptance scenarios remain behaviorally unchanged except for the expanded infrastructure-failure wording.
- **Verification:** `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, the acceptance suite, `tests/run_tests.sh`, and `openspec validate harden-worker-pool-runtime --type change --strict`.
- **ADR-0003:** the single-pass streaming, bounded-queue, and producer-before-collection decisions remain in force. Only the terminal mechanism changes from snapshot-then-shutdown to take-then-drop-senders-and-join; the adr step must record whether that refinement needs a superseding ADR or a manifest rationale.
- **Documentation:** review README architecture text for references to removed command or sender APIs; update only if it names them.
- **Dependencies and operations:** none added. No persistence, migration, or output-contract change; rollback is a source revert.
- **Out of scope:** account arithmetic/overflow changes, sharding policy, transaction-index growth, global transaction-ID uniqueness, queue capacity, and output ordering.
- **Risk:** medium. The main risks are the `'static` ownership change required by `spawn_blocking`, the new processor drain method, worker termination ordering, and keeping deterministic tests without a sleep-based race. Each is covered by a design decision and targeted test.
