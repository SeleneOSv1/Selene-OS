#!/usr/bin/env bash
set -euo pipefail

DOC="docs/web_search_plan/WEB_SEARCH_PLAN_CANON.md"

fail() {
  echo "FAIL: $1"
  exit 1
}

[ -f "$DOC" ] || fail "missing $DOC"

if rg -n "Run-by-Run Functionality Upgrade Addendum" "$DOC" >/dev/null; then
  fail "forbidden heading present: Run-by-Run Functionality Upgrade Addendum"
fi

if rg -n "New Additive Runs" "$DOC" >/dev/null; then
  fail "forbidden heading present: New Additive Runs"
fi

if rg -n "Run [0-9]+\.[0-9]+" "$DOC" >/dev/null; then
  fail "decimal run numbering found (example: Run 1.1)"
fi

canonical_start="$(rg -n '^## Canonical Build Runs \(Runs 1-30\)$' "$DOC" | cut -d: -f1)"
canonical_end="$(rg -n '^## Final Wiring Contract \(Section 40\)$' "$DOC" | cut -d: -f1)"

[ -n "$canonical_start" ] || fail "missing canonical runs section start"
[ -n "$canonical_end" ] || fail "missing final wiring section marker"

if [ "$canonical_start" -ge "$canonical_end" ]; then
  fail "canonical runs section ordering is invalid"
fi

for run in $(seq 1 30); do
  count="$(rg -n "^#### Run ${run} — " "$DOC" | wc -l | tr -d ' ')"
  if [ "$count" -ne 1 ]; then
    fail "run ${run} heading count is ${count}; expected exactly 1"
  fi
done

all_run_headings="$(rg -n '^#### Run [0-9]+ — ' "$DOC" | wc -l | tr -d ' ')"
if [ "$all_run_headings" -ne 30 ]; then
  fail "total canonical run headings is ${all_run_headings}; expected 30"
fi

if rg -n '^### Run 1 — ' "$DOC" >/tmp/selene_run36_run1_h3_hits.txt; then
  while IFS=':' read -r line _; do
    if [ "$line" -le "$canonical_start" ] || [ "$line" -ge "$canonical_end" ]; then
      fail "found '### Run 1 —' outside canonical run section at line ${line}"
    fi
  done < /tmp/selene_run36_run1_h3_hits.txt
fi

rm -f /tmp/selene_run36_run1_h3_hits.txt

echo "PASS: doc canon checks passed"
