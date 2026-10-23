# ADR Review Manifest

- Status: completed
- Review date: 2026-10-23

## Review Summary

ADR review completed for this change. The change removes hot-path allocations in pipeline routing and fixed-point amount parsing/rendering while preserving values, errors, output bytes, and public APIs. It does not change the worker pool, streaming lifecycle, money representation, or any cross-cutting contract. No repository-level ADR was created.

## In-Force ADRs Reviewed

- `adr/0001-fixed-worker-pool-sharding.md` — fixed worker pool with client affinity and worker-local state. Unaffected: the dispatcher still pins clients to a worker and the pool is untouched.
- `adr/0002-fixed-point-i64-money.md` — fixed-point `i64` money without floating point. Followed: parsing and rendering remain integer-only; the change removes allocations without changing the representation or checked arithmetic.
- `adr/0003-single-pass-streaming-lifecycle.md` — single-pass streaming with bounded queues and producer-before-collection. Unaffected: pipeline ownership, queues, snapshot, and shutdown are unchanged.

## New Durable ADRs Created

- None — the allocation strategy is a local implementation optimization, not a new architectural commitment. The existing fixed-point and streaming decisions already constrain it.
