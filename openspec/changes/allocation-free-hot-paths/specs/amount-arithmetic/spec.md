# amount-arithmetic

## MODIFIED Requirements

### Requirement: Amounts are stored as scaled integers

The system SHALL represent every monetary amount as an `i64` equal to the decimal value multiplied by 10,000, and MUST NOT use floating-point types in parsing, ledger arithmetic, or output.

#### Scenario: A fractional deposit is stored on the four-decimal grid

```gherkin
Given a new client deposits 1.23
When the accounts are written
Then the client's available is 1.23
And the client's total is 1.23
```

#### Scenario: A leading-dot amount is accepted

```gherkin
Given a new client deposits .5
When the accounts are written
Then the client's available is 0.5
```

#### Scenario: A leading-dot two-digit fraction is accepted

```gherkin
Given a new client deposits .25
When the accounts are written
Then the client's available is 0.25
```

#### Scenario: A leading-dot four-digit fraction is accepted

```gherkin
Given a new client deposits .0001
When the accounts are written
Then the client's available is 0.0001
```

#### Scenario: A whole amount is accepted

```gherkin
Given a new client deposits 5
When the accounts are written
Then the client's available is 5.0
```

#### Scenario: Trailing fractional zeros do not change the value

```gherkin
Given a new client deposits 1.2300
When the accounts are written
Then the client's available is 1.23
```

#### Scenario: Surrounding whitespace is ignored

```gherkin
Given a new client deposits an amount padded with spaces around 1.5
When the accounts are written
Then the client's available is 1.5
```

### Requirement: Invalid amounts are rejected

The system SHALL reject amounts that are negative, empty, non-numeric (including a bare decimal point), have more than four fractional digits, or overflow the scaled `i64`.

#### Scenario: A negative amount is rejected

```gherkin
Given a transaction row deposits the negative amount -1.0
When the processor runs
Then the row is skipped
And no account is initialized for that client
```

#### Scenario: More than four fractional digits is rejected

```gherkin
Given a transaction row deposits 1.23456
When the processor runs
Then the row is skipped
And no account is initialized for that client
```

#### Scenario: A non-numeric amount is rejected

```gherkin
Given a transaction row deposits abc
When the processor runs
Then the row is skipped
And no account is initialized for that client
```

#### Scenario: A bare decimal point is rejected

```gherkin
Given a transaction row deposits .
When the processor runs
Then the row is skipped
And no account is initialized for that client
```

#### Scenario: An amount that overflows the scaled integer is rejected

```gherkin
Given a transaction row deposits 922337203685477.5808
When the processor runs
Then the row is skipped
And no account is initialized for that client
```
