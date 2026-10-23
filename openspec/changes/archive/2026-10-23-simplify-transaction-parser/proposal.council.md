# Council Notes: proposal.md

## Author Summary

The draft proposed replacing the manual `StringRecord` parser with serde row deserialization into a raw row type, changing the `transactions` return type and header matching, and reclassifying identifier errors as CSV errors. The reviewer found the direction sound but identified correctness risks around malformed headers and several overstated behavior claims.

## Reviewer Challenges

- Malformed-header outcomes differ per missing column; a renamed `amount` column can silently accept a dispute that carries an amount.
- Only `client`/`tx` reclassification was stated; the raw field types must be pinned.
- Order-independent headers conflict with 3-column rows unless `amount` is the final column.
- `flexible(true)` and `Trim::All` are load-bearing but unstated.
- The bad-header exit-code change is process-level, not only ingestion-level.
- Retaining four unused public error variants lacks a rationale.
- Design mechanics leak into `What Changes`.
- "Same streaming and memory profile" is unverifiable, and per-row error timing changes.

## Resolutions

- Accepted: keep a name-based header gate so malformed headers stay fatal and a renamed `amount` column cannot silently accept a non-monetary row. This intentionally differs from the supplied snippet.
- Accepted: scope order-independence to full-width rows and require `amount` to be final for short rows.
- Accepted: pin the raw row fields (`type` and `amount` as strings; `client`/`tx` as integers) so only identifier errors become `Csv`.
- Accepted: preserve `flexible(true)` and `Trim::All`, with acceptance coverage for 3-column rows and padded fields.
- Accepted: no `processor-cli` delta; malformed headers remain fatal and valid-input behavior is unchanged.
- Accepted: remove `InvalidClient`, `InvalidTransactionId`, and `MissingField`; keep `InvalidHeader` and `InvalidType(String)`.
- Accepted: move implementation mechanics to `design.md` and soften the streaming/memory claim.
- Deferred: keeping a named iterator type; the opaque `Result<impl Iterator>` is the simpler target and is flagged as breaking.
- Rejected: adopting the supplied bare `impl Iterator` without a header gate (silent success and silent acceptance risk).
- Rejected: case/BOM normalization; exact-name matching keeps the gate and serde consistent and is documented as a compatibility change.

## Remaining Risks

- Case-sensitive/BOM-strict headers are a behavior change for previously tolerated input.
- The opaque iterator and removed error variants break the public library API.
- `serde` + `flexible(true)` + `#[serde(default)]` behavior for 3-column rows must be verified during apply.
