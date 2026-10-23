# ADR-0001: Fixed Worker-Pool Sharding for Client State

## Status

Accepted

## Date

2026-10-22

## Context

The processor must apply per-client transaction streams in order, keep client state isolated, and avoid locks while handling up to 65,536 possible client IDs. The project also plans future distributed variants that must reuse the same core rules. A per-client actor would create up to 65,536 tasks; a shared map would require locking.

## Decision

Partition clients across a fixed pool of Tokio worker tasks. Each worker owns its clients' `HashMap<ClientId, Account>` and `HashMap<TransactionId, TransactionEntry>`. A single producer reads the CSV, assigns each client to a worker on first sight, and pins it there. Transactions cross the boundary through `tokio::sync::mpsc` with a bounded queue (`QUEUE_CAPACITY = 256`) and `blocking_send` backpressure. No account is shared between workers, so account state needs no locks.

## Consequences

- Positive: per-client ordering is preserved, account state needs no locks, the task count stays fixed, and the worker core is reusable by future variants.
- Negative: a hot client can throttle its shard, the dispatcher keeps a client-to-worker map, and cross-shard operations are impossible by construction.
- Follow-up: future distributed variants should shard by client ID with the same ownership rule so account semantics stay identical.
