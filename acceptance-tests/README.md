# Acceptance tests

These tests execute the Gherkin scenarios in `openspec/` against the compiled
`transaction_processor` binary.

## Setup

```sh
uv venv acceptance-tests/.venv
uv pip install --python acceptance-tests/.venv/bin/python -r acceptance-tests/requirements.txt
```

## Run

```sh
acceptance-tests/.venv/bin/python acceptance-tests/run_acceptance.py
```

The suite builds the binary, extracts every `spec.md` under `openspec/` into
`acceptance-tests/.extracted/`, and runs behave. An HTML report is written to
`acceptance-tests/reports/behave-report.html`.

Other commands:

```sh
acceptance-tests/.venv/bin/python acceptance-tests/run_acceptance.py --specs
acceptance-tests/.venv/bin/python acceptance-tests/run_acceptance.py --lint
acceptance-tests/.venv/bin/python acceptance-tests/run_acceptance.py --print-locations
```

`.extracted/`, `reports/`, and `.venv/` are generated and gitignored.
