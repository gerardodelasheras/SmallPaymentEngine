# ADR-0003: Single-Pass Streaming with Snapshot-Then-Shutdown

## Status

Accepted

## Date

2026-10-22

## Context

The input may be far larger than memory, so the processor must not buffer the whole file. At the same time, output requires the final state of every initialized client, and row failures must not halt the stream or corrupt output. Infrastructure failures must still end the run.

## Decision

Run CSV ingestion in a blocking task so `blocking_send` is legal, stream rows through bounded per-worker queues, and isolate row failures by logging and skipping them. When ingestion completes, send a snapshot command carrying a `oneshot` channel to each worker, await the account maps, then close the senders (or send a shutdown command) and join the worker tasks. Merge the disjoint maps and write the accounts CSV once through an `impl Write` sink. Diagnostics go to stderr; infrastructure errors propagate out of `process`.

## Consequences

- Positive: in-flight memory is bounded, there is no deadlock from a bounded return path, row failures stay isolated, and the output has a single header and a mergeable result.
- Negative: the transaction index still grows with accepted transactions because disputes reference arbitrary earlier IDs, so ledger memory is not O(1); shutdown ordering is explicit and must not regress.
- Follow-up: document the memory bound and keep tests focused on final state rather than interleaving.
