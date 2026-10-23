# Council Notes: proposal.md

## Author Summary

The draft proposed a greenfield Rust package implementing the streaming payments ledger described by SPEC.md and README.md: fixed-point money, CSV ingestion, the transaction state machine, account reporting, and the CLI/streaming contract, split across five behaviour capabilities. The reviewer found the direction sound but flagged overclaiming, design leakage, ambiguous capability boundaries, and unresolved SPEC/README/config conflicts.

## Reviewer Challenges

- Withdrawal insufficient-funds rejection was not stated as a requirement.
- FSM mutation effects (held/total/index transitions, chargeback locking) were only implicit.
- The memory claim conflicted with the transaction history that disputes require.
- `What Changes` leaked design mechanics (queue constant, `blocking_send`, worker-local state, module layout).
- Capability boundaries were ambiguous, and config domains `client`/`test-harness` were unaddressed.
- Cross-client rejection cannot distinguish `NotOwner` from `UnknownTx` with worker-local indexes.
- Fixture and publication policy conflicted across README, config, and SPEC §7.3.
- "Skip silently" (SPEC §5.1) conflicted with "skip with diagnostics" (README).
- Trace gating lacked the required release-build cutoff.
- CSV failure taxonomy mixed skippable rows with fatal reader errors.
- Client initialization was too broadly stated.
- Duplicate `tx` handling, numeric syntax, and output/CLI edge cases were undefined.
- No-panic/stream-continuation was not testable.
- Assumption provenance (README-derived vs SPEC §5.5) was overconfident.
- Publication sanitization omitted source comments and challenge-derived files.
- Regression coverage was promised while marked optional.
- The README verification deliverable was unstated.
- Chronology and single-account invariants were missing.

## Resolutions

- Accepted: Stated withdrawal rejection and the `available`/`held`/`locked`/index effects at requirement level.
- Accepted: Replaced the memory overclaim with the realistic bound and recorded SPEC §1's O(1) claim as a deviation.
- Accepted: Moved queue constants, `blocking_send`, fixed worker counts, and module mechanics out of the proposal and into design/tasks.
- Accepted: Clarified capability ownership; noted client routing and the test harness as design/verification concerns, not behaviour capabilities.
- Accepted: Labeled README-derived rules as documented assumptions with SPEC §7.2 provenance.
- Accepted: Split skippable row errors from fatal reader/I/O errors.
- Accepted: Narrowed client initialization to successfully parsed rows.
- Accepted: Made unit tests and the independent oracle required; fixtures and runner optional.
- Accepted: Added the README verification deliverable and expanded delivery sanitization.
- Accepted: Made silent-by-default skipping with opt-in trace diagnostics explicit.
- Deferred: Detailed FSM mechanics, numeric accept/reject tables, duplicate-`tx` policy, and output/CLI edge cases belong in the delta specs.
- Deferred: The `NotOwner` vs `UnknownTx` diagnostic label is a design/spec decision; the proposal fixes the observable outcome (reject without mutation).
- Rejected: Adding a `client-routing` capability; routing is not externally observable behaviour.
- Rejected: Adding a `test-harness` capability; tests verify behaviour rather than define it.

## Remaining Risks

- SPEC §1's O(1) memory objective cannot coexist with dispute history; the deviation must stay visible through specs, design, and README.
- SPEC §7.3 ("`src/` + README only") conflicts with the project delivery list; the config's explicit list is followed and the discrepancy recorded.
- Worker-local indexes may report cross-client references as unknown transactions; only a global ownership map would change the diagnostic, at the cost of sharding.
- A fixed worker pool can make timing-sensitive integration tests flaky; the oracle should compare final state, not interleaving.
