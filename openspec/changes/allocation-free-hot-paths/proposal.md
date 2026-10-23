## Why

The streaming hot path still allocates for information it already has or can compute arithmetically: every successfully parsed transaction snapshots worker capacities even when its client is already pinned, fractional amount parsing builds a padded `String`, and account rendering builds a temporary formatted `String`. Removing those allocations improves streaming throughput without changing values, validation, errors, output bytes, or CLI behavior.

## What Changes

- Add a pinned-client fast path in `pipeline::read_and_dispatch`: use the existing `ClientDispatcher::worker_of` lookup first and call `pool.capacities()` only when the client is new. Assignment policy stays identical: fewest assigned clients, then greatest remaining capacity, then the round-robin cursor.
- Route through a private lazy-capacity helper so the "capacities only on first assignment" rule is directly unit-testable; do not change `ClientDispatcher`'s public methods or add a new public dispatcher surface.
- Replace `Amount::parse`'s padded fraction `String` with integer scaling by the remaining fraction width and checked arithmetic. Preserve every accepted form, rejected form, validation order, error variant, and error payload exactly.
- Replace `Amount::Display`'s temporary formatted `String` with direct formatter output: compute the sign with `unsigned_abs`, divide out stripped trailing zeros, then zero-pad the remaining fraction width. Preserve canonical rendering byte-for-byte, including `i64::MIN` and `i64::MAX`.
- Add unit regressions for:
  - leading-dot parsing (`.25`, `.0001`) and already-supported edge forms (`1.`, `00.5`, `0.0000`);
  - bare `.`, `+1`, multiple dots, >4 fraction digits, negatives, and scaled overflow, including error precedence and original-input payloads;
  - canonical rendering at `0`, `±1`, `±10`, `±100`, `±1000`, `±10000`, `±1203`, `i64::MIN`, and `i64::MAX`;
  - the lazy-capacity fast path not requesting capacities for an already-assigned client.
- Add `amount-arithmetic` acceptance scenarios for leading-dot `.25` and `.0001` parsing, plus bare `.` rejection. These scenarios already pass on the current baseline; they are executable-spec coverage, not a value change.
- Add no benchmark harness or CI allocation gate. Allocation-free parsing/rendering is a property of the implementation and is covered by unit tests and review, not by an automated allocation assertion.
- Leave ledger rules, error text and variants, accepted input forms, output bytes, worker-pool behavior, reader API, and public API unchanged. No breaking changes.
- Defer to the next change: library re-exports, private implementation modules, broader documentation, and additional `Account` hold/release overflow tests.

## Capabilities

### New Capabilities

None. No performance or engine capability is introduced; the dispatcher allocation optimization is internal and needs no behavior specification.

### Modified Capabilities

- `amount-arithmetic` — executable-spec coverage only; no value behavior changes:
  - `specs/amount-arithmetic/spec.md`
  - **Amounts are stored as scaled integers**: MODIFIED block retaining every existing scenario verbatim (including `.5`) and adding leading-dot `.25` and `.0001` parsing scenarios.
  - **Invalid amounts are rejected**: MODIFIED block retaining every existing scenario verbatim, clarifying that non-numeric includes a bare decimal point, and adding a bare `.` rejection scenario.
  - The two base rules are replaced by their full delta copies in the effective suite, so every existing scenario is copied, not paraphrased. The new scenarios pass against the baseline before any code changes; this delta records coverage, not new behavior.

## Impact

- **Code:** `src/pipeline.rs` (lazy capacity snapshot) and `src/amount.rs` (allocation-free parse and render), plus their unit-test modules. No dispatcher API changes.
- **Public API:** unchanged. No modules, re-exports, types, or public signatures change.
- **Behavior:** identical values, error variants and text, accepted and rejected inputs, ledger outcomes, output bytes, and CLI behavior. Parse error paths still allocate the original-input payload; whole-number inputs already avoided fraction allocation. The streaming win is the dispatcher snapshot plus fractional parsing; rendering improves account-output cost after the stream.
- **Performance:** assigned clients no longer allocate a capacity vector per successfully parsed transaction. Non-empty fractional parsing no longer allocates a padded string. Rendering no longer allocates a temporary string. A first-sight client still performs one capacity snapshot.
- **Tests:** baseline is 87 cargo tests and 76 acceptance scenarios. Add unit regressions listed above and three acceptance scenarios using the existing `{amount:Amount}` and raw-row steps. `.extracted/` is regenerated and gitignored.
- **Verification:** `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, the full acceptance suite, `tests/run_tests.sh`, and `openspec validate allocation-free-hot-paths --type change --strict`.
- **Dependencies and operations:** none added. No persistence, migration, output-schema, or deployment change. Rollback reverts both the code and the spec-coverage delta.
- **Risk:** moderate. The main risk is a subtle change in parse validation order or canonical formatting. Preserve error precedence, keep full rule blocks in the delta, and use boundary vectors including `i64::MIN` and `i64::MAX`.
- **Out of scope:** ledger semantics, worker-pool redesign, reader changes, public API cleanup, benchmark harness, CI allocation gate, worker-pool sampling changes, and the deferred documentation and `Account` test work.

## Sequencing

1. Add the delta scenarios and run the effective acceptance suite against the baseline; commit the specs zone.
2. Change `Amount` and the pipeline, add unit regressions, and keep the effective suite and CSV regressions green.
3. Commit code separately from spec edits, then validate and archive.
