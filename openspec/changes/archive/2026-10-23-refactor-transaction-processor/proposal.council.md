# Council Notes: proposal.md

## Author Summary

Proposes a behavior-preserving refactor of `TransactionProcessor`: per-type handlers, a private `record_settled` path, and `TransactionEntry` as a `Settled`/`Disputed` enum. Preserves lock, duplicate, ownership, dispute, and initialization semantics; flags the public entry/`EntryState` representation as breaking; adds one explicit atomic-rejection requirement to the existing `transaction-processing` capability.

## Reviewer Challenges

- Capabilities section contradicted the template by claiming "no behavior changes" while adding a clarification requirement.
- Duplicate-ID check and mutation ordering were under-specified, risking a mutated balance before a duplicate error.
- Error precedence (`AccountLocked`, `MissingAmount`, `UnknownTransaction`/`NotOwner` before state errors) was implicit.
- Pre-dispatch account initialization conflicted with "get/create the account" inside handlers.
- `record_settled` visibility and the true public-API surface were unclear.
- The atomicity addition needed to be a real added requirement with a scenario, not a loose clarification.
- Test claims overstated coverage and wrongly implied processor tests referenced the entry types.
- Proposal mixed design detail into the "why" and lacked a verification note.
- Worker-local scope of duplicate detection and the unpublised-crate migration stance needed stating.

## Resolutions

- Accepted: list `transaction-processing` as a modified capability and make the atomicity guarantee an added spec requirement with scenarios.
- Accepted: document the required order as amount presence, then duplicate ID, then mutation, then index insertion; `record_settled` performs no fallible validation.
- Accepted: state the full error-precedence chain explicitly.
- Accepted: keep pre-dispatch initialization as the sole creation point; handlers only operate on the already-initialized account.
- Accepted: state that `record_settled` is private and narrow the breaking surface to `TransactionEntry`, `EntryState`, and entry fields.
- Accepted: replace the "migrate tests to the enum" claim with targeted tests for missing amount, precedence, and index non-mutation.
- Accepted: add verification commands and the worker-local duplicate-enforcement scope.
- Rejected: dropping the spec delta entirely. The intent-driven schema requires a specs artifact, and an explicit atomic-rejection requirement is externally observable and testable.
- Rejected: treating the public entry change as a runtime behavior change. It is a library-contract break only.

## Remaining Risks

- Duplicate transaction IDs are still enforced per processor/worker, not globally; the proposal preserves that existing gap rather than fixing it.
- The public `TransactionEntry` break has no migration path because the crate is unpublished; publishing later would require a semver decision.
- Atomicity across every rejection path is asserted, but new tests can only sample the paths being reshaped.
