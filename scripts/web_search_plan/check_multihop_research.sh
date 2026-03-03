#!/usr/bin/env bash
set -euo pipefail

python3 - <<'PY'
import json
from pathlib import Path

root = Path("docs/web_search_plan/multihop_fixtures")
required = [
    "simple_chain.json",
    "cycle_case.json",
    "budget_exhaust_case.json",
    "expected_plans.json",
]

for name in required:
    path = root / name
    with path.open("r", encoding="utf-8") as f:
        json.load(f)
PY

cargo test -p selene_os web_search_plan::multihop::multihop_tests --quiet
