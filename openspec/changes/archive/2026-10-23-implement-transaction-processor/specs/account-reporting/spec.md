# Account Reporting

After the input ends, the processor writes the final state of every initialized client to standard output as CSV. Row order is unspecified.

## ADDED Requirements

### Requirement: The output has a fixed header and columns

The system SHALL write a CSV to stdout whose header is `client,available,held,total,locked`, and each account row SHALL contain the client identifier, available funds, held funds, total funds, and lock state.

#### Scenario: The output header is written

```gherkin
Given a new client deposits 1.0
When the processor runs
Then the output header is "client,available,held,total,locked"
```

#### Scenario: An account row contains every column

```gherkin
Given a new client deposits 1.5
When the processor runs
Then the output contains the row "1,1.5,0.0,1.5,false"
```

### Requirement: Every initialized client is reported

The system SHALL emit exactly one row per initialized client, including a client whose only row was rejected after parsing.

#### Scenario: Two clients produce two rows

```gherkin
Given client 1 deposits 1.0
And client 2 deposits 2.0
When the processor runs
Then the output contains a row for client 1
And the output contains a row for client 2
And the output contains exactly 2 account rows
```

#### Scenario: A client initialized by a rejected row is reported with zeros

```gherkin
Given a new client withdraws 5.0
When the processor runs
Then the output contains the row "1,0.0,0.0,0.0,false"
```

### Requirement: Amounts use canonical decimal rendering

The system SHALL render `available`, `held`, and `total` with trailing fractional zeros removed and at least one fractional digit, including negative values with a leading minus sign.

#### Scenario: A zero balance renders as 0.0

```gherkin
Given a new client deposits 1.0
And the client withdraws 1.0
When the processor runs
Then the output contains the row "1,0.0,0.0,0.0,false"
```

#### Scenario: A negative available balance renders with a minus sign

```gherkin
Given a new client deposits 100.0
And the client withdraws 80.0
And the client disputes the deposit
When the processor runs
Then the output contains the row "1,-80.0,100.0,20.0,false"
```

### Requirement: Total always equals available plus held

The system SHALL write `total` equal to `available + held` for every account row.

#### Scenario: A dispute keeps the reported total unchanged

```gherkin
Given a new client deposits 10.0
And the client disputes the deposit
When the processor runs
Then the output contains the row "1,0.0,10.0,10.0,false"
```

### Requirement: The locked column reflects the account state

The system SHALL write `true` in the `locked` column exactly when a chargeback locked the account, and `false` otherwise.

#### Scenario: A chargeback reports a locked account

```gherkin
Given a new client deposits 10.0
And the client disputes the deposit
And the client charges back the dispute
When the processor runs
Then the output contains the row "1,0.0,0.0,0.0,true"
```

#### Scenario: An active account reports false

```gherkin
Given a new client deposits 10.0
When the processor runs
Then the output contains the row "1,10.0,0.0,10.0,false"
```

### Requirement: Empty input produces only the header

The system SHALL write the output header and no account rows when the input contains no initialized client.

#### Scenario: A header-only input produces only the header

```gherkin
Given a CSV containing only the header
When the processor runs
Then the output header is "client,available,held,total,locked"
And the output contains no account rows
```
