# processor-cli

## MODIFIED Requirements

### Requirement: Infrastructure failures end the run

The system SHALL exit with a non-zero status and a stderr message when the input cannot be read, the accounts CSV cannot be written, or a worker or its channel is lost; such failures SHALL NOT be silently converted into partial output.

#### Scenario: A missing input file fails the run

```gherkin
Given an input path that does not exist
When the processor runs
Then the run exits with a non-zero status
And an error is written to stderr
```
