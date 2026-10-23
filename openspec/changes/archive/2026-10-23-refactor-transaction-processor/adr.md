# ADR Review Manifest

- Status: completed
- Review date: 2026-10-23

## Review Summary

ADR review completed for this change. The refactor reshapes one worker-local state machine: per-type handlers, a private settled-entry path, and `TransactionEntry` as a `Settled`/`Disputed` enum. It introduces no new pattern, technology, boundary, or cross-cutting contract, and it does not revisit any in-force ADR. No repository-level ADR was created.

## In-Force ADRs Reviewed

- `adr/0001-fixed-worker-pool-sharding.md` — fixed worker pool with client affinity and worker-local state. Unaffected: the processor stays worker-local, keeps its client and transaction maps, and does not change sharding or dispatch.
- `adr/0002-fixed-point-i64-money.md` — fixed-point `i64` money. Unaffected: amounts, parsing, rendering, and checked arithmetic are untouched; the refactor only moves where `Account` methods are called.
- `adr/0003-single-pass-streaming-lifecycle.md` — single-pass streaming with bounded backpressure and snapshot/shutdown. Unaffected: pipeline, queues, worker loop, and snapshot/shutdown behavior are out of scope.

## New Durable ADRs Created

- None — no major durable architectural decisions were introduced. The entry enum is a local data-model refactor inside the existing worker component, not a long-term architectural commitment that future changes must inherit.
