#!/usr/bin/env bash
set -euo pipefail

python3 - <<'PY'
import json
from pathlib import Path

root = Path('docs/web_search_plan')
for name in ['replay_corpus.json', 'replay_expected.json']:
    path = root / name
    with path.open('r', encoding='utf-8') as f:
        json.load(f)

fixture_dir = root / 'replay_fixtures'
fixtures = sorted(fixture_dir.glob('*.json'))
if not fixtures:
    raise SystemExit('replay_fixtures directory is empty')
for fixture in fixtures:
    with fixture.open('r', encoding='utf-8') as f:
        json.load(f)
PY

cargo test -p selene_os web_search_plan::replay::replay_tests --quiet
