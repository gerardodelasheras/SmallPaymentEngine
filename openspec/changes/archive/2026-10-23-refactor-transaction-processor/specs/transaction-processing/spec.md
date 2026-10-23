# transaction-processing

## ADDED Requirements

### Requirement: Rejected operations do not mutate ledger state

When the system rejects a parsed transaction, it SHALL leave every account's `available`, `held`, and `locked` state unchanged and SHALL leave the accepted-transaction index unchanged. The only permitted state effect of a rejected parsed row SHALL be the already-specified zeroed-account initialization for that client.

#### Scenario: A rejected duplicate deposit leaves balances unchanged

```gherkin
Given a new client deposits 10.0 with transaction 1
When the client deposits 5.0 with transaction 1
Then the client's available is 10.0
And the client's held is 0.0
And the client's total is 10.0
```

#### Scenario: A rejected duplicate deposit preserves the original entry

```gherkin
Given a new client deposits 10.0 with transaction 1
When the client deposits 5.0 with transaction 1
And the client disputes transaction 1
Then the client's available is 0.0
And the client's held is 10.0
And the client's total is 10.0
```

#### Scenario: A rejected dispute leaves other transactions intact

```gherkin
Given a new client deposits 250.0 with transaction 11
And the client withdraws 100.0
When the client disputes transaction 99
Then the client's available is 150.0
And the client's held is 0.0
And the client's total is 150.0
```

#### Scenario: A rejected operation on a locked account changes nothing

```gherkin
Given a new client deposits 40.0 with transaction 21
When the client disputes transaction 21
And the client charges back the dispute
And the client deposits 15.0 with transaction 22
Then the client's available is 0.0
And the client's held is 0.0
And the client's total is 0.0
And the client is locked
```
