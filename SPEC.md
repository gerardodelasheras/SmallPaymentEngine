# Transaction Processor Specification

## 1. Introduction

We deal with transactions on and off chain. Off chain transactions are generally mutable, and in a lot of ways this is a good thing. It lowers the risk of losing your credit card because you know your credit provider has your back, but for vendors integrating into payment networks or with payment partners it means we need to be vigilant and on the lookout for fraudsters. For example, a malicious actor may try to deposit fiat funds, purchase and withdraw BTC, and then reverse their fiat deposit.
We'd like you to implement a simple toy payments engine that reads a series of transactions from a CSV, updates client accounts, handles disputes and chargebacks, and then outputs the state of client accounts as a CSV.



## 1. Overview & Objectives
Implement a simple, robust, high-performance payments processor in Rust that reads a chronological sequence of transactions from a CSV file, safely tracks and mutates client accounts, handles disputes, resolutions, and account-freezing chargebacks, and outputs the final state of all mutated client accounts back as a CSV.

### Core Objectives
* **Completeness**: Fully address and implement standard transitions (Deposits, Withdrawals) alongside advanced dispute-state mutations (Disputes, Resolutions, Chargebacks). Handling a subset is acceptable only if missing edge cases are explicitly identified.
* **Correctness**: Ensure exact balance formulas and zero floating-point rounding errors via deterministic unit testing and a robust type system. Test against sample data and describe your verification approach in the repository README.
* **Safety & Robustness**: Systematically catch errors and validation failures at boundaries without causing thread panics or halting the streaming execution loop. Handle all errors deliberately.
* **Efficiency**: Ensure a flat, static memory footprint ($O(1)$ spatial complexity relative to log size) by streaming values through memory line-by-line rather than buffering datasets upfront. Solutions are evaluated against far larger data sets than the samples (`tx` is a valid `u32`), so memory must not scale with input size. 
* **Maintainability**: Deliver clean, readable, highly modularized code utilizing idiomatic Rust abstractions so humans can easily review and understand it without original author intervention. Inefficient code can be improved over time if it remains maintainable.

---

## 2. Technical Stack & Architecture System Guardrails

### 2.1 Codebase Layout & Targets
The application is configured as a single package containing **one shared library crate** (`src/lib.rs`) and **one standard binary executable** built from `src/main.rs`:
* **`transaction_processor`** (default target): Compiled from `src/main.rs`. Enforces memory-efficient, local CSV-to-stdout streaming to fulfill the standard CLI specification. The package exposes no custom `[[bin]]` targets, so the graded interface `cargo run -- transactions.csv > accounts.csv` works without a `--bin` flag.
* **Future distributed variant**: The distributed processor is not part of this main branch; it will be developed on a separate branch.
* **Shared Core**: All core transaction business logic, custom error handlers, and client account actor loops must live inside the library crate (`src/lib.rs`) to ensure total reuse by the standard binary and any future variant.

### 2.2 Architectural Pattern: Actor Pattern & Concurrency
* **Actor Design**: Every individual `u16` Client Account (or a sharded pool of them) must be isolated and managed inside an independent actor execution loop. Transactions are routed and dispatched to their respective client actors exclusively via async message-passing using **Tokio channels (`tokio::sync::mpsc`)**.
* **Isolation Invariant**: A malformed transaction row, arithmetic precision overflow, or invalid historical dispute mapping must return a distinct error response to the streaming processor pipeline without taking down the target actor state machine or crashing the active runtime thread.

### 2.3 Fixed-Point Currency Precision
To guarantee perfect correctness and rule out float-rounding vulnerabilities, raw floating-point primitives (`f32`, `f64`) are strictly prohibited within the processor ledger.
* All financial values must be parsed and stored internally as an explicit `i64` integer scaled by a factor of `10_000` (e.g., an input amount string of `1.23` translates to `12300`).
* Outputs sent to stdout must be reformatted back to represent standard four-decimal syntax.

---

## 3. Inbound & Outbound Contract Formats

### 3.1 Input Data Schema
The input file is passed as the first and only argument to the binary CLI tool. The input will be a CSV file with the columns `type`, `client`, `tx`, and `amount`. Spacing, trailing whitespaces, and numeric fraction variations must be accepted gracefully.

* `type`: Action primitive (`deposit`, `withdrawal`, `dispute`, `resolve`, `chargeback`). Mapped as a string.
* `client`: Unique identifier mapped as a valid `u16` client ID.
* `tx`: Globally unique transaction identifier mapped as a valid `u32` transaction ID.
* `amount`: Decimal value string with a precision of up to four digits past the decimal point (omitted for disputes, resolutions, and chargebacks).

#### Input CSV Example:
```csv
type,client,tx,amount
deposit,1,1,1.0
deposit,2,2,2.0
deposit,1,3,2.0
withdrawal,1,4,1.5
withdrawal,2,5,3.0
```

### 3.2 Output Data Schema
Output should be written directly to standard output (`stdout`). The output must be a list of client IDs (`client`), available amounts (`available`), held amounts (`held`), total amounts (`total`), and whether the account is locked (`locked`). Spacing, displaying decimals for round values, and row sorting order do not matter.

| Column | Description | Formula / System Constraint |
| :--- | :--- | :--- |
| `client` | Client ID | Distinct client profile identifier (`u16`). |
| `available` | Available Funds | Total funds available for trading, staking, or withdrawal. Equal to: `total - held` |
| `held` | Held Funds | Total funds frozen for dispute. Equal to: `total - available` |
| `total` | Total Funds | Total funds available or held. Equal to: `available + held` |
| `locked` | Freeze State | True if a chargeback occurs; freezes the account instantly. |

#### Output CSV Example:
```csv
client,available,held,total,locked
1,1.5,0.0,1.5,false
2,2.0,0.0,2.0,false
```

---

## 4. System & Domain Invariants
* **Single Account Layout**: Each distinct `u16` identifier maps to one single asset account. All transactions are to and from this single asset account. If an unknown client ID appears in the stream, a new ledger profile is instantly initialized with zeroed balances.
* **Chronological Ordering**: Transactions are considered to be in exact chronological order based on their location in the input CSV file (e.g., if transaction `b` appears after `a`, it occurred chronologically after `a`).
* **Global Transaction Uniqueness**: Transaction IDs (`tx`) are globally unique across all client profiles, though they are not guaranteed to be ordered numerically.
* **ATM Safety Analogy**: When a requirement choice is ambiguous, default to conservative banking rules (e.g., block all transactions on locked accounts, preserve funds during disputes).

---

## 5. Finite State Machine (FSM) & Domain Error Strategy

Transactions must alter account state maps exactly as described below. If an operational exception or input validation error occurs, the pipeline must enforce strict isolation parameters.

### 5.1 Error Handling & Stream Isolation Bounds
* **Silent Skipping**: When encountering a validation error (such as a missing transaction ID during a dispute, an invalid state transition, or a locked account invariant), the processing processor must skip that row silently.
* **Maintain the Stream**: The single row failure must be safely dropped so the parent stream pipeline can proceed smoothly without panicking, crashing threads, or halting the main execution loop.
* **Design Rationale**: This explicit error handling approach is strictly designed to avoid output file corruption during direct piping operations (`> accounts.csv`). Because standard output is reserved strictly for contract-compliant CSV records, writing raw text exceptions or log output files to stdout would pollute and break the downstream file destination.

### 5.2 Optional Error Logging
* If diagnostics, warning summaries, or skipped row trace details are required for operational visibility, they must be written exclusively to standard error (`stderr`) using standard lightweight logging interfaces (such as `log` or `tracing`). This ensures errors are visible on consoles without altering stdout content data.

### 5.3 Deposit
A credit to the client's asset account. Increases both `available` and `total` balances by the scaled `amount`.
* **State Preservation**: The transaction ID (`tx`) and its associated scaled amount must be cached inside a historical lookup index to evaluate potential future disputes.

### 5.4 Withdrawal
A debit to the client's asset account. Decreases both `available` and `total` balances by the scaled `amount`.
* **Error Boundary**: If a client does not have sufficient `available` funds, the withdrawal must fail. The row is skipped, and total amounts of funds remain unchanged.

### 5.5 Dispute
Represents a client's claim that a previous transaction was erroneous and should be reversed. 
* **Mutation**: Funds are not reversed yet, but are held. Decrease `available` by the original transaction's value; increase `held` by the same value; `total` remains identical.
* **Error Boundary**: A dispute references the transaction by ID (`tx`) and does not state an amount. Ignore the dispute and assume an error on the partner's side if:
  * The referenced `tx` ID does not exist in the historical cache.
  * The transaction is already under active dispute.

  > **Interpreted rule**: The requirements document only requires ignoring a dispute when `tx` does not exist. Rejecting disputes on already-disputed or already charge-backed transactions follows the conservative banking defaults in §4 and must be documented as such, not treated as source spec.

### 5.6 Resolve
Represents a resolution to a dispute, releasing the associated held funds. Funds that were previously disputed are no longer disputed.
* **Mutation**: Decrease `held` by the original transaction value; increase `available` by that value; `total` remains identical.
* **Error Boundary**: Refers to a transaction by ID (`tx`) and does not specify an amount. Ignore the resolution if:
  * The referenced `tx` ID does not exist in the historical cache.
  * The transaction is not currently marked as under active dispute.

### 5.7 Chargeback
The terminal state of an unresolvable dispute where a client reverses a transaction. Funds that were held have now been withdrawn.
* **Mutation**: Decrease both `held` and `total` balances by the originally disputed transaction value. Immediately flip the client's profile state to `locked = true`.
* **Error Boundary**: Refers to a transaction by ID (`tx`) and does not specify an amount. Ignore the chargeback if the referenced `tx` ID does not exist in the historical cache, or if the transaction is not currently marked as under active dispute. Once a chargeback succeeds, the account is locked and all further transactions to that account are rejected.

---

## 6. Storage & Executable Delivery Architectural Proposals

### 6.1 [Proposal A] In-Memory Streaming Processor (`src/main.rs`)
* **Objective**: Satisfy the exact standalone command-line requirement.
* **Execution Interface**: Must expose exactly the graded interface `cargo run -- transactions.csv > accounts.csv`. The package's standard binary is autodetected from `src/main.rs`, so plain `cargo run` selects the in-memory processor without `--bin` or `default-run`.
* **Architecture**: Utilizes `std::io::BufReader` and the `csv` crate to deserialize lines via `serde` on-the-fly. It hosts client profiles in a thread-safe, heap-allocated local `HashMap` or `DashMap`.
* **Consequences**: Static, non-growing execution memory bounds. State is ephemeral and is flushed to stdout once EOF is reached.

### 6.2 [Proposal B, Future Branch] Distributed Event-Driven Payment Cluster
* **Status**: Not part of this main branch. The distributed variant will be developed on a separate branch.
* **Objective**: Scale the exact same shared library rules processor to process thousands of multi-threaded concurrent data streams as an enterprise backend cluster node.
* **Execution Interface**: Defined on the future branch; this main branch exposes only the `transaction_processor` binary.
* **Architecture**: The future branch will define its own ingestion and transport; the same actor loops and error-handling rules from `src/lib.rs` run per worker, sharded by client ID for deterministic per-account state.
* **Consequences**: The distributed variant is out of scope for the main-branch CLI contract and may require external infrastructure.

---

## 7. Expectations

### 7.1 Objectives
The solution must comply with these items:

| Category | Criterion |
| :--- | :--- |
| Basics | Builds, reads/writes data per contract, correctly formatted. |
| Completeness | Covers deposits, withdrawals, disputes, resolutions, chargebacks. |
| Correctness | Correct balance math; tested against sample data and unit tests; approach documented in README. |
| Safety & Robustness | Deliberate, bounded error handling; no panics or stream halts on bad input. |
| Efficiency | Streams through memory; evaluated against far larger data sets (`tx` is a valid `u32`). |
| Maintainability | Clean, reviewable code; weighted above raw efficiency. |

### 7.2 Precedence Rule
If a requirement is inconsistent, the described **input and output files take precedence**.
If a requirement is not clear document the proper assumption and other alternatives.

### 7.3 Delivery Model (Two Repos)
Code pushed to a public repository that must not contain any information identifying the requestor of this solution.

| Repo | Location | Contents | Constraints |
| :--- | :--- | :--- | :--- |
| Publication | https://github.com/gerardodelasheras/SmallPaymentEngine | `src/` (Rust module) + `README.md` only | §7.4 submission constraints apply. |

### 7.4 Publication (Submission) Constraints
* The published repository must be a simple Rust crate (`cargo new`), buildable via `cargo build`, runnable via `cargo run`.
* The published solution (source comments and `README.md`) must **not mention** the company or person requesting this project.
* The challenge test file or any derivative of it must **not** appear in the published repository.
* The `README.md` must still describe the verification approach (sample data, unit tests) per §7.1.
