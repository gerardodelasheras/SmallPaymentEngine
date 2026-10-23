# ADR Review Manifest

- Status: completed
- Review date: 2026-10-22

## Review Summary

ADR review completed for this change. No repository-level ADRs existed before this change, so there was no in-force set to reconcile and no supersession. Three decisions met the durability bar and were recorded under `adr/`; the remaining design choices are tactical or already covered by the specs.

## In-Force ADRs Reviewed

- None - `<repo>/adr/` had no in-force ADRs before this change.

## New Durable ADRs Created

- `adr/0001-fixed-worker-pool-sharding.md` - fixed worker pool with client affinity, worker-local state, and bounded message passing.
- `adr/0002-fixed-point-i64-money.md` - `i64` money scaled by 10,000, with checked arithmetic and no floating point.
- `adr/0003-single-pass-streaming-lifecycle.md` - single-pass ingestion, bounded backpressure, snapshot-then-shutdown collection, and error isolation.

## Notes

- New ADRs use MADR-minimal sections (Context, Decision, Consequences) per the artifact instruction. `preferred-style` in the ADR skill preferences remains `unset`; change it if a different style is preferred.
- Dispute semantics (overdraw allowed, any accepted monetary transaction disputable, re-dispute after resolve) and the recoverable-vs-fatal error taxonomy are recorded as spec-level assumptions, not promoted to durable ADRs.
