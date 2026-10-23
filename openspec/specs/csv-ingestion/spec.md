# CSV Ingestion Specification

## Purpose

TBD - created by archiving change implement-transaction-processor. Update Purpose after archive.

## Requirements

### Requirement: The input has a header and four columns

The system SHALL read a CSV whose first row is a header naming the columns `type`, `client`, `tx`, and `amount` in any order, and SHALL accept a trailing `amount` field that is empty or absent when the transaction type does not use it. Header names SHALL be matched exactly: a missing header, a missing required column, or a differently-cased name SHALL cause the run to fail. Extra columns SHALL be ignored. A short non-monetary row that omits the trailing amount SHALL be valid only when `amount` is the final column.

#### Scenario: A header followed by a deposit is accepted

```gherkin
Given a CSV with the header "type,client,tx,amount"
And the row "deposit,1,1,2.0"
When the processor runs
Then client 1 has available 2.0
```

#### Scenario: A non-monetary row omits the amount column

```gherkin
Given a client with an accepted deposit of 10.0
And a dispute row that has only type, client, and transaction identifier columns
When the processor runs
Then the client's held is 10.0
```

#### Scenario: Fields padded with spaces are accepted

```gherkin
Given a CSV row " deposit , 1 , 2 , 3.0 "
When the processor runs
Then client 1 has available 3.0
```

#### Scenario: Columns in a different order are accepted

```gherkin
Given a CSV with the header "tx,amount,type,client"
And a CSV row "3,2.0,deposit,1"
When the processor runs
Then client 1 has available 2.0
```

#### Scenario: An extra column is ignored

```gherkin
Given a CSV with the header "type,client,tx,amount,note"
And a CSV row "deposit,1,1,2.0,ok"
When the processor runs
Then client 1 has available 2.0
```

#### Scenario: A header missing a required column fails the run

```gherkin
Given a CSV with the header "type,client,amount"
And a CSV row "deposit,1,5.0"
When the processor runs
Then the run exits with a non-zero status
And an error is written to stderr
```

#### Scenario: An uppercase header is rejected

```gherkin
Given a CSV with the header "TYPE,CLIENT,TX,AMOUNT"
And a CSV row "deposit,1,1,2.0"
When the processor runs
Then the run exits with a non-zero status
And an error is written to stderr
```

### Requirement: Unknown transaction types are skipped

The system SHALL accept only `deposit`, `withdrawal`, `dispute`, `resolve`, and `chargeback`; any other transaction type SHALL cause the row to be skipped.

#### Scenario: An unknown type is skipped and the stream continues

```gherkin
Given a CSV row "transfer,1,1,5.0"
And a following row "deposit,1,2,5.0"
When the processor runs
Then client 1 has available 5.0
```

### Requirement: Identifiers are validated

The system SHALL parse `client` as a `u16` and `tx` as a `u32`, skipping any row whose identifier is non-numeric or out of range.

#### Scenario: A client identifier above the u16 range is skipped

```gherkin
Given a CSV row "deposit,70000,1,5.0"
When the processor runs
Then no account is initialized for that client
```

#### Scenario: A transaction identifier above the u32 range is skipped

```gherkin
Given a CSV row "deposit,1,5000000000,5.0"
When the processor runs
Then no account is initialized for client 1
```

#### Scenario: A non-numeric client identifier is skipped

```gherkin
Given a CSV row "deposit,abc,1,5.0"
When the processor runs
Then no account is initialized
```

### Requirement: Amount presence follows the transaction type

The system SHALL require a non-empty, valid amount for `deposit` and `withdrawal`, and SHALL require the amount to be empty or absent for `dispute`, `resolve`, and `chargeback`.

#### Scenario: A deposit without an amount is skipped

```gherkin
Given a CSV row "deposit,1,1,"
When the processor runs
Then no account is initialized for client 1
```

#### Scenario: A withdrawal without an amount is skipped

```gherkin
Given a client with an accepted deposit of 10.0
And a withdrawal row with no amount
When the processor runs
Then the client's available is 10.0
```

#### Scenario: A dispute carrying an amount is skipped

```gherkin
Given a client with an accepted deposit of 10.0
And a dispute row that also carries an amount
When the processor runs
Then the client's held is 0.0
```

#### Scenario: A dispute with an empty amount field is applied

```gherkin
Given a client with an accepted deposit of 10.0
And a dispute row with an empty amount field
When the processor runs
Then the client's held is 10.0
```

### Requirement: Row-level failures do not abort the stream

The system SHALL skip a row that cannot be read or validated and SHALL continue processing later rows.

#### Scenario: A malformed row is skipped before a valid row

```gherkin
Given a CSV row with too few columns
And a following valid deposit row for client 1
When the processor runs
Then client 1 has available 5.0
```

#### Scenario: A row with invalid UTF-8 is skipped

```gherkin
Given a CSV row containing invalid UTF-8 bytes
And a following valid deposit row for client 1
When the processor runs
Then client 1 has available 5.0
```

#### Scenario: A blank line between valid rows is ignored

```gherkin
Given a blank line between two valid deposit rows for client 1
When the processor runs
Then client 1 has available 7.0
```
