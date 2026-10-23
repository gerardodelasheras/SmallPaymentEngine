# ADR Review Manifest

- Status: completed
- Review date: 2026-10-23

## Review Summary

ADR review completed for this change. The change replaces the ingestion parser's hand-written row handling with serde deserialization and adds an exact-name header gate. It stays inside the ingestion component, introduces no new pattern, technology, boundary, or cross-cutting contract, and does not revisit any in-force ADR. No repository-level ADR was created.

## In-Force ADRs Reviewed

- `adr/0001-fixed-worker-pool-sharding.md` - fixed worker pool with client affinity and worker-local state; unaffected because the parser still feeds the same dispatch path.
- `adr/0002-fixed-point-i64-money.md` - fixed-point `i64` money; unaffected because amount parsing and rendering are unchanged.
- `adr/0003-single-pass-streaming-lifecycle.md` - single-pass streaming with bounded backpressure and snapshot/shutdown; unaffected because the parser remains a lazy row iterator.

## New Durable ADRs Created

- None - no major durable architectural decisions were introduced.
