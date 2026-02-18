#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "CHECK_FAIL builder_pipeline_phase13b: $1" >&2
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
check_contains "crates/selene_os/src/lib.rs" "pub mod ph1builder;" "selene_os lib must export ph1builder"
check_contains "crates/selene_os/src/ph1builder.rs" "struct Ph1BuilderOrchestrator" "missing orchestrator type"
check_contains "crates/selene_os/src/ph1builder.rs" "fn run_offline" "missing offline run entrypoint"
check_contains "crates/selene_os/src/ph1builder.rs" "trait BuilderSandboxValidator" "missing sandbox validator trait"
check_contains "crates/selene_os/src/ph1builder.rs" "collect_gate_evaluations" "missing gate collection function"
check_contains "crates/selene_os/src/ph1builder.rs" "PatternWiringOutcome::Forwarded" "missing pattern wiring integration"
check_contains "crates/selene_os/src/ph1builder.rs" "RllWiringOutcome::Forwarded" "missing rll wiring integration"
check_contains "crates/selene_os/src/ph1builder.rs" "BuilderSeleneRepo" "missing builder storage repo integration"
check_contains "crates/selene_os/src/ph1builder.rs" "at_builder_os_01_offline_run_persists_validated_proposal_run_and_gates" "missing stage13b success test"
check_contains "crates/selene_os/src/ph1builder.rs" "at_builder_os_02_fails_closed_when_gate_collection_is_incomplete" "missing stage13b fail-closed test"
check_contains "crates/selene_os/src/ph1builder.rs" "at_builder_os_03_idempotent_replay_keeps_single_rows" "missing stage13b idempotency test"

if rg -n "PH1\\.BUILDER" "crates/selene_os/src/ph1os.rs" >/dev/null; then
  fail "PH1.BUILDER must not be wired into runtime PH1.OS turn path"
fi

echo "CHECK_OK builder_pipeline_phase13b=pass"
