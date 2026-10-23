# Council Notes: proposal.md

## Author Summary

Proposes removing per-transaction capacity snapshots for pinned clients, allocation-free fractional amount parsing, and allocation-free canonical rendering. Adds executable-spec coverage for leading-dot and bare-dot forms while preserving all values, errors, output bytes, and public APIs.

## Reviewer Challenges

- The dispatcher fast path cannot save an allocation if the pipeline still calls `pool.capacities()` before lookup.
- A computed display width alone is wrong for interior-zero fractions; the fraction must be divided down before zero-padding.
- "Public API unchanged" conflicted with an unspecified dispatcher split.
- The MODIFIED spec delta replaces base rules in the effective suite and must copy every scenario with exact formatting.
- The template says Modified Capabilities means behavior change, while the draft called the delta coverage only.
- The allocation claim had no verification seam.
- Parse edge cases (`1.`, `00.5`, `0.0000`, untrimmed error payloads, error precedence) were under-specified.
- "Low risk" understated the parse/render rewrite risk.

## Resolutions

- Accepted: branch on the existing `ClientDispatcher::worker_of` and call capacities only on a miss, routed through a private lazy-capacity helper that can be unit-tested.
- Accepted: preserve the dispatcher public surface; no new public methods.
- Accepted: specify the correct display algorithm using `unsigned_abs`, removed trailing zeros, then zero-padding; add boundary vectors including `i64::MIN` and `i64::MAX`.
- Accepted: specify exact MODIFIED rule blocks with every existing scenario and column-0 fenced Gherkin; record the baseline run as evidence.
- Accepted: frame the capability delta explicitly as executable-spec coverage that already passes, not as a value change.
- Accepted: add a lazy-capacity unit test; explicitly state that parse/render allocation-free is covered by implementation review and unit behavior tests, not an allocation assertion.
- Accepted: enumerate preserved forms and error precedence, including `1.`, `00.5`, `0.0000`, and original-input error payloads.
- Accepted: rate risk moderate and add boundary/differential-style vectors.
- Accepted: note whole-number/error-path allocation details and that rendering happens after the stream.
- Accepted: add sequencing, verification commands, rollback of both code and delta, and explicit non-goals.
- Rejected: adding a benchmark harness or global allocation counter to this change.

## Remaining Risks

- The parse rewrite can subtly change validation order or error payloads; boundary tests and preserved precedence are the mitigation.
- The effective-spec runner will exclude the two base rules while running their delta copies; dropping a scenario from a MODIFIED block would silently shrink coverage.
- Allocation-free rendering is not automatically measured; correctness is proven by output vectors, not by an allocator test.
- The pipeline fast path is verified through a helper seam, not through end-to-end allocation measurement.
