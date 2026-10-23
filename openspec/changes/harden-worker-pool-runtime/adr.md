# ADR Review Manifest

- Status: completed
- Review date: 2026-10-23

## Review Summary

ADR review completed for this change. The refactor narrows the worker-pool surface, replaces the snapshot/shutdown pair with a terminal take-accounts drain, and moves the account maps out instead of cloning them. It does not change the sharding policy, the bounded queue decision, the streaming lifecycle, money representation, or any cross-cutting contract. No repository-level ADR was created.

## In-Force ADRs Reviewed

- `adr/0001-fixed-worker-pool-sharding.md` — fixed worker pool with client affinity and worker-local state. Unaffected: the pool still owns one processor per worker and clients stay pinned to a worker.
- `adr/0002-fixed-point-i64-money.md` — fixed-point `i64` money. Unaffected: balances, amounts, and checked arithmetic are untouched.
- `adr/0003-single-pass-streaming-lifecycle.md` — single-pass streaming with bounded queues, producer-before-collection, then close senders and join. Followed: the producer still completes before collection, the bounded queues remain, and the terminal sequence still closes senders and joins workers. The only refinement is that the collection command moves the account maps out with `mem::take` rather than cloning a snapshot before shutdown.

## New Durable ADRs Created

- None — the terminal-mechanism refinement fits within ADR-0003's accepted decision ("send a snapshot ... then close the senders ... and join"), so no supersession is warranted. Removing the public module and command surface is an internal encapsulation decision, not a durable architecture commitment.
