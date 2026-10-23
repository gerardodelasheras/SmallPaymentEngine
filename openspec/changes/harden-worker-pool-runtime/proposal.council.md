# Council Notes: proposal.md

## Author Summary

Proposes making the worker pool crate-private and non-escaping, replacing the snapshot/shutdown pair with a terminal take-accounts drain, moving the pool through the blocking producer task, and preserving row-skip versus infrastructure-failure semantics. Adds one processor-CLI contract expansion for worker/channel loss.

## Reviewer Challenges

- The `spawn_blocking` ownership path was unspecified: the pool cannot be borrowed into a `'static` closure.
- The capability section contradicted the template by calling a normative `SHALL` expansion a wording-only clarification.
- The draft incorrectly said the current pool loses `Clone`; the current pool is already not `Clone`.
- `mem::take` requires a new mutable drain method on `TransactionProcessor`, so the file impact was incomplete.
- Making only `WorkerPool` crate-private would leave the module, command enum, and capacity constant public.
- ADR-0003 describes the snapshot/shutdown mechanism, so its impact needed assessment.
- The terminal drain could return early on error without dropping senders or joining handles.
- The worker loop could process commands after accounts were taken unless it exits.
- Deterministic backpressure and closed-channel tests need explicit seams because `blocking_send` cannot run on an async test thread and public API cannot induce closure.
- Nonexistent files (`engine.rs`, `stream_cli`) appeared in Impact.
- The risk rating was too low given ownership, termination, and test-seam changes.
- Worker-count ownership, blocking dispatch semantics, the reply channel shape, README drift, and overclaimed "type system" guarantees needed clarification.

## Resolutions

- Accepted: specify that the pool moves into `spawn_blocking` and is returned with the dispatch result; a join failure loses the pool and returns `WorkerUnavailable`.
- Accepted: frame the `processor-cli` change as a genuine contract expansion, name the delta file, and state that worker loss is not CLI-inducible.
- Accepted: remove the false `Clone` claim; the breaking surface is module/type visibility, command variants, and sender accessors.
- Accepted: add `TransactionProcessor::take_accounts` to Impact and keep `snapshot()` to avoid an extra break.
- Accepted: demote the module, `Command`, and `QUEUE_CAPACITY`, not just `WorkerPool`.
- Accepted: record the ADR-0003 assessment as a design/adr-step obligation.
- Accepted: define drain as collect-first, retain the first error, drop senders, and always await handles; map `JoinError` to `WorkerUnavailable`.
- Accepted: make `TakeAccounts` terminal in the worker loop.
- Accepted: state that tests use module-private seams and a synchronization signal rather than sleep for backpressure.
- Accepted: remove nonexistent file references and raise the risk to medium with concrete risks.
- Accepted: state that the pool owns the clamped worker count, document blocking dispatch and index expectations, and name the oneshot reply.
- Accepted: add a README review bullet and a verification command list.
- Rejected: dropping the spec delta entirely. The capability contract is being expanded, so the schema-required artifact should capture it.
- Rejected: claiming the runtime invariants are fully enforced by the type system; they are enforced by ownership plus tests.

## Remaining Risks

- The blocking-task return-and-drain pattern changes control flow in `pipeline`; a mistake could drop the pool on an error path and skip cleanup.
- Worker panic handling becomes stricter than today and could turn a previously ignored join failure into a failed run.
- The channel-loss contract expansion has unit coverage but no CLI-inducible acceptance scenario.
- ADR-0003 may need a superseding record if maintainers consider the terminal-mechanism change part of its accepted decision rather than an implementation refinement.
