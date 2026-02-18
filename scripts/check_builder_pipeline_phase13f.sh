#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "CHECK_FAIL builder_pipeline_phase13f: $1" >&2
  exit 1
}

check_file() {
  local file="$1"
  [ -f "$file" ] || fail "missing file: $file"
}

check_contains() {
  local file="$1"
  local pattern="$2"
  local message="$3"
  rg -n "$pattern" "$file" >/dev/null || fail "$message ($file)"
}

check_file "crates/selene_os/src/ph1builder.rs"
check_file "scripts/check_builder_human_permission_gate.sh"
check_file "docs/fixtures/builder_change_brief_template.md"

check_contains "crates/selene_os/src/ph1builder.rs" "DEFAULT_CHANGE_BRIEF_OUTPUT_PATH" "missing default change brief output path"
check_contains "crates/selene_os/src/ph1builder.rs" "change_brief_output_path" "missing change brief input/output wiring"
check_contains "crates/selene_os/src/ph1builder.rs" "generate_change_brief" "missing auto change brief generator"
check_contains "crates/selene_os/src/ph1builder.rs" "render_change_brief_markdown" "missing change brief markdown renderer"
check_contains "crates/selene_os/src/ph1builder.rs" "write_change_brief_to_path" "missing change brief writer"
check_contains "crates/selene_os/src/ph1builder.rs" "at_builder_os_14_change_brief_auto_generated_for_permission_gate" "missing change brief auto generation test"

check_contains "scripts/check_builder_human_permission_gate.sh" "Should I proceed\\?" "missing proceed prompt gate"
check_contains "scripts/check_builder_human_permission_gate.sh" "All tests passed\\. Can I launch\\?" "missing launch prompt gate"

check_contains "docs/fixtures/builder_change_brief_template.md" "Should I proceed\\?" "brief template missing proceed question"
check_contains "docs/fixtures/builder_change_brief_template.md" "All tests passed\\. Can I launch\\?" "brief template missing launch question"

echo "CHECK_OK builder_pipeline_phase13f=pass"
