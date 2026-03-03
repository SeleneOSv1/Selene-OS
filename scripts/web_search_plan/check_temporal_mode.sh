#!/usr/bin/env bash
set -euo pipefail

python3 - <<'PY'
import json
from pathlib import Path

root = Path("docs/web_search_plan/temporal_fixtures")
required = [
    "baseline_rows.json",
    "compare_rows.json",
    "mixed_units.json",
    "missing_timestamps.json",
    "expected_changes.json",
    "expected_timeline.json",
]
for name in required:
    with (root / name).open("r", encoding="utf-8") as f:
        json.load(f)
PY

cargo test -p selene_os web_search_plan::temporal::temporal_tests --quiet
