#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "SIM_FINDER_ACCEPTANCE_FAIL:missing_tool:$1"
    exit 2
  fi
}

fail() {
  echo "SIM_FINDER_ACCEPTANCE_FAIL:$1"
  exit 1
}

require_match() {
  local pattern="$1"
  local file="$2"
  local msg="$3"
  if ! rg -n "$pattern" "$file" >/dev/null 2>&1; then
    fail "$msg ($file)"
  fi
}

require_cmd rg
require_cmd awk

CONTRACT_FILE="crates/selene_kernel_contracts/src/ph1simfinder.rs"
KERNEL_DIR="crates/selene_kernel_contracts/src"
RUNTIME_DIRS=(
  "crates/selene_os"
  "crates/selene_engines"
  "crates/selene_adapter"
  "crates/selene_storage"
)

[ -f "$CONTRACT_FILE" ] || fail "missing_contract_file:$CONTRACT_FILE"

# 1) Canonical terminal packet type lock.
require_match "^pub enum FinderTerminalPacket" "$CONTRACT_FILE" \
  "FinderTerminalPacket enum must exist"
require_match "SimulationMatch\\(SimulationMatchPacket\\)" "$CONTRACT_FILE" \
  "FinderTerminalPacket must include SimulationMatch"
require_match "Clarify\\(ClarifyPacket\\)" "$CONTRACT_FILE" \
  "FinderTerminalPacket must include Clarify"
require_match "Refuse\\(RefusePacket\\)" "$CONTRACT_FILE" \
  "FinderTerminalPacket must include Refuse"
require_match "MissingSimulation\\(MissingSimulationPacket\\)" "$CONTRACT_FILE" \
  "FinderTerminalPacket must include MissingSimulation"

variant_count="$(
  awk '
    /pub enum FinderTerminalPacket/ { in_enum=1; next }
    in_enum && /^\}/ { print count+0; exit }
    in_enum && $0 ~ /^[[:space:]]*[A-Za-z_]+\(.*\),?[[:space:]]*$/ { count++ }
  ' "$CONTRACT_FILE"
)"
if [ "${variant_count:-0}" -ne 4 ]; then
  fail "FinderTerminalPacket must contain exactly 4 variants (found=${variant_count:-0})"
fi

duplicate_terminal_enums="$(
  rg -n "pub enum .*TerminalPacket" "$KERNEL_DIR" \
    | rg -v "ph1simfinder.rs:.*pub enum FinderTerminalPacket" || true
)"
if [ -n "$duplicate_terminal_enums" ]; then
  echo "$duplicate_terminal_enums"
  fail "duplicate_terminal_packet_union_detected"
fi
echo "CHECK_OK sim_finder_terminal_packet_types=pass"

# 2) Runtime reason codes must be declared in canonical contract registry.
declared_codes="$(
  rg -o "SIM_FINDER_[A-Z0-9_]+" "$CONTRACT_FILE" | sort -u
)"
[ -n "$declared_codes" ] || fail "no_declared_sim_finder_reason_codes_in_contract"

runtime_codes="$(
  rg --no-filename -o "SIM_FINDER_[A-Z0-9_]+" "${RUNTIME_DIRS[@]}" 2>/dev/null | sort -u || true
)"

if [ -n "$runtime_codes" ]; then
  while IFS= read -r code; do
    [ -z "$code" ] && continue
    if ! rg -n "^${code}$" <(printf "%s\n" "$declared_codes") >/dev/null 2>&1; then
      fail "runtime_reason_code_not_declared_in_contract:${code}"
    fi
  done <<<"$runtime_codes"
fi
echo "CHECK_OK sim_finder_reason_codes_declared=pass"

# 3) Fail if packet construction bypasses contract validation.
bypass_hits="$(
  rg -n "(SimulationMatchPacket|ClarifyPacket|RefusePacket|MissingSimulationPacket)\\s*\\{" crates \
    | rg -v "^${CONTRACT_FILE}:" || true
)"
if [ -n "$bypass_hits" ]; then
  echo "$bypass_hits"
  fail "packet_struct_literal_bypass_detected"
fi

validate_ctor_count="$(rg -n "packet\\.validate\\(\\)\\?;" "$CONTRACT_FILE" | wc -l | tr -d ' ')"
if [ "${validate_ctor_count:-0}" -lt 4 ]; then
  fail "expected_validate_calls_in_v1_constructors_missing(count=${validate_ctor_count:-0})"
fi

require_match "FinderTerminalPacket::SimulationMatch\\(packet\\) => packet\\.validate\\(\\)," "$CONTRACT_FILE" \
  "terminal union validate must delegate SimulationMatch"
require_match "FinderTerminalPacket::Clarify\\(packet\\) => packet\\.validate\\(\\)," "$CONTRACT_FILE" \
  "terminal union validate must delegate Clarify"
require_match "FinderTerminalPacket::Refuse\\(packet\\) => packet\\.validate\\(\\)," "$CONTRACT_FILE" \
  "terminal union validate must delegate Refuse"
require_match "FinderTerminalPacket::MissingSimulation\\(packet\\) => packet\\.validate\\(\\)," "$CONTRACT_FILE" \
  "terminal union validate must delegate MissingSimulation"

echo "CHECK_OK sim_finder_contract_validation_bypass_guard=pass"
echo "CHECK_OK agent_sim_finder_core_acceptance=pass"
