# CSV Ingestion

## MODIFIED Requirements

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
