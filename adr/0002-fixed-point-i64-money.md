# ADR-0002: Fixed-Point i64 Money

## Status

Accepted

## Date

2026-10-22

## Context

The ledger must be exact and the specification forbids floating-point primitives. Input amounts carry up to four fractional digits, and balances, holds, and totals must satisfy `total = available + held` without rounding drift.

## Decision

Represent every monetary value as a newtype over `i64` scaled by `10_000` (for example, `1.23` is `12300`). Parse decimal text into that representation, render it canonically with trailing zeros stripped and at least one fractional digit, and perform balance mutations with checked integer arithmetic. Floating-point types and decimal crates are excluded from the parse, ledger, and output paths.

## Consequences

- Positive: money is exact, arithmetic is fast and dependency-free, and the representation prevents mixing amounts with identifiers.
- Negative: representable magnitude is limited to `i64::MAX / 10_000`, every mutation must use checked operations, and rendering must handle negative balances.
- Follow-up: any future ingestion source must parse into this representation rather than passing raw decimals or floats.
