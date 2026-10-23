Zone discipline: add and commit the specs-zone delta first, verify the new scenarios against the baseline, then change only the code zone. `tasks.md` is zone-neutral. No new step definitions are expected: the existing `{amount:Amount}` type accepts `.25`, `.0001`, and `.`.

## 1. Acceptance coverage first

- [x] 1.1 Run the effective acceptance suite with the new `amount-arithmetic` delta and confirm `.25`, `.0001`, and bare `.` scenarios pass against the current implementation with zero undefined steps.
- [x] 1.2 Confirm the effective suite still includes every existing `amount-arithmetic` scenario from the two MODIFIED rule blocks; record the scenario count.
- [x] 1.3 Commit the specs-zone delta separately before touching `src/`.

## 2. Pipeline lazy capacities

- [x] 2.1 Add a private `routed_worker(dispatcher, client, capacity_supplier)` helper that uses `ClientDispatcher::worker_of` first and calls the supplier only when the client is unassigned.
- [x] 2.2 Use `routed_worker(..., || pool.capacities())` in `read_and_dispatch`; keep `ClientDispatcher::worker_for` and the assignment policy unchanged.
- [x] 2.3 Add a unit test proving the capacity supplier is not called for a pinned client and is called exactly once for a new client.
- [x] 2.4 Keep the existing dispatcher pinning and assignment-policy tests green.

## 3. Allocation-free amount parsing

- [x] 3.1 Replace the padded fraction `String` in `Amount::parse` with integer scaling by `10_i64.pow(4 - fraction.len())` using checked multiplication and addition.
- [x] 3.2 Preserve validation order, accepted/rejected forms, error variants, original-input payloads, and overflow behavior.
- [x] 3.3 Add parse tests for `.25`, `.0001`, `1.`, `00.5`, `0.0000`, bare `.`, `+1`, multiple dots, more than four decimals, negatives, and the `i64::MAX` boundary.
- [x] 3.4 Add error-precedence tests for negative vs malformed, too-many-decimals vs overflow, and overflow vs invalid amount.

## 4. Allocation-free amount rendering

- [x] 4.1 Replace `format!("{fraction:04}")` in `Display` with direct `write!` output using `unsigned_abs`, prior remainder scaling, and a computed zero-padded width.
- [x] 4.2 Preserve canonical output for whole, fractional, negative, interior-zero, and boundary values.
- [x] 4.3 Add render tests for `0`, `±1`, `±10`, `±100`, `±1000`, `±10000`, `±1203`, `i64::MIN`, and `i64::MAX`.

## 5. Verification

- [x] 5.1 Run `cargo test` and confirm all unit and integration tests pass.
- [x] 5.2 Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`.
- [x] 5.3 Run the full acceptance suite and confirm zero pending or undefined steps and an HTML report.
- [x] 5.4 Run `tests/run_tests.sh` and confirm the committed CSV regression outputs are byte-identical.
- [x] 5.5 Run `openspec validate allocation-free-hot-paths --type change --strict`.
- [x] 5.6 Run `cargo build --release` and confirm the CLI succeeds on a representative input.
- [x] 5.7 Confirm no new dependency, benchmark harness, or CI allocation gate was added.

## 6. Completion

- [x] 6.1 Verify code-zone and specs-zone commits stayed separate, then hand off to sync/archive after implementation is merged.
