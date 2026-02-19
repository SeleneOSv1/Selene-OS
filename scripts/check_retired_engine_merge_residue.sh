#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "MISSING_TOOL:$1"
    exit 2
  fi
}

require_cmd rg

RETIRED_REGEX='PH1\.(WEBINT|PRIORITY|ATTN|PUZZLE|REVIEW|LEARNING_ADAPTIVE|NLP\.001)'

ALLOWED_HISTORICAL_FILES=(
  "docs/03_BUILD_LEDGER.md"
  "docs/33_ENGINE_REVIEW_TRACKER.md"
  "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md"
)

rg_args=(rg -n -S "$RETIRED_REGEX")
for path in "${ALLOWED_HISTORICAL_FILES[@]}"; do
  rg_args+=(--glob "!${path}")
done
rg_args+=(--glob "!docs/archive/**")
rg_args+=(docs crates scripts README.md)

matches="$("${rg_args[@]}" || true)"

if [ -n "$matches" ]; then
  echo "MERGE_RESIDUE_FAIL:retired standalone engine ids found outside historical merge logs"
  printf '%s\n' "$matches"
  exit 1
fi

echo "CHECK_OK merge_residue_retired_engines=pass"
