#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
TRACKER="${ROOT}/docs/33_ENGINE_REVIEW_TRACKER.md"

if [ ! -f "$TRACKER" ]; then
  echo "ERROR: tracker file not found: $TRACKER"
  exit 2
fi

awk '
function trim(s) { gsub(/^[[:space:]]+|[[:space:]]+$/, "", s); return s }

BEGIN {
  errors = 0
  rows = 0
}

{
  full_text = full_text $0 "\n"
}

/^\| [0-9][0-9] / {
  split($0, cols, "|")
  order = trim(cols[2])
  engine = trim(cols[3])
  status = trim(cols[5])
  rows++

  if (engine in seen_engine) {
    printf("ERROR:DUPLICATE_ENGINE_ID engine_id=%s rows=%s,%s\n", engine, seen_engine[engine], order)
    errors++
  } else {
    seen_engine[engine] = order
  }

  order_engine[order] = engine
  order_status[order] = status
  engine_status[engine] = status

  if (status ~ /^MERGED_INTO_[0-9]+$/) {
    target = status
    sub(/^MERGED_INTO_/, "", target)
    merge_target[order] = target
    merge_engine[order] = engine
  }

  if (engine ~ /^PH[0-9]+\.[A-Z0-9_]+\.[0-9][0-9][0-9]$/) {
    base = engine
    sub(/\.[0-9][0-9][0-9]$/, "", base)
    family_has_impl[base] = 1
    impl_count[base]++
    if (status !~ /^MERGED_INTO_/) {
      impl_non_merged_count[base]++
    }
  } else if (engine ~ /^PH[0-9]+\.[A-Z0-9_]+$/) {
    family_status[engine] = status
    family_order[engine] = order
  }
}

END {
  if (rows == 0) {
    print "ERROR: no engine rows found in tracker table"
    exit 2
  }

  for (order in merge_target) {
    target = merge_target[order]
    engine = merge_engine[order]
    if (!(target in order_engine)) {
      printf("ERROR:MERGE_TARGET_MISSING row=%s engine_id=%s target_row=%s\n", order, engine, target)
      errors++
    } else if (order_status[target] ~ /^MERGED_INTO_/) {
      printf("ERROR:MERGE_TARGET_NON_CANONICAL row=%s engine_id=%s target_row=%s target_status=%s\n", order, engine, target, order_status[target])
      errors++
    }

    expected = sprintf("Row %s (`%s`) is merged into row %s", order, engine, target)
    if (index(full_text, expected) == 0) {
      printf("ERROR:MERGE_NOTE_MISSING row=%s engine_id=%s expected_note=\"%s\"\n", order, engine, expected)
      errors++
    }
  }

  for (base in family_has_impl) {
    if (!(base in family_status)) {
      continue
    }
    status = family_status[base]
    order = family_order[base]
    non_merged_impl = impl_non_merged_count[base] + 0
    canonical_family_done = 0
    if (status == "DONE") {
      target_status = "MERGED_INTO_" order
      for (engine in engine_status) {
        if (engine ~ ("^" base "\\.[0-9][0-9][0-9]$") && engine_status[engine] == target_status) {
          canonical_family_done = 1
          break
        }
      }
    }

    if (status !~ /^MERGED_INTO_/ && status != "EXEMPT" && status != "REMOVED" && !canonical_family_done) {
      printf("ERROR:NAMESPACE_NOT_COLLAPSED row=%s engine_id=%s status=%s expected=MERGED_INTO_*|EXEMPT|REMOVED\n", order, base, status)
      errors++
    }

    if (status ~ /^MERGED_INTO_/ && non_merged_impl == 0) {
      printf("ERROR:NO_CANONICAL_IMPLEMENTATION base=%s family_status=%s\n", base, status)
      errors++
    }
  }

  if (errors > 0) {
    printf("CHECK_FAILED errors=%d\n", errors)
    exit 1
  }

  printf("CHECK_OK rows=%d duplicate_namespace_guardrail=pass\n", rows)
}
' "$TRACKER"
