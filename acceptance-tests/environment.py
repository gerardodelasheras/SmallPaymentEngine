"""behave hooks for the acceptance suite."""

import os
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from pages.cli import CliHarness  # noqa: E402


def before_all(context):
    if not os.environ.get("OPENSPEC_ACCEPTANCE"):
        raise RuntimeError(
            "Run the suite via `python run_acceptance.py` (not plain `behave`).\n"
            "The wrapper extracts Gherkin from openspec/**/spec.md and composes the effective spec."
        )

    root = HERE.parent
    subprocess.run(["cargo", "build", "--quiet"], cwd=root, check=True)
    (HERE / "reports").mkdir(exist_ok=True)
    context.repo_root = root
    context.binary = root / "target" / "debug" / "transaction_processor"


def after_all(context):
    pass


def before_scenario(context, scenario):
    context.cli = CliHarness(context.binary)
