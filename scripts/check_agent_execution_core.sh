#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "AGENT_EXECUTION_CORE_FAIL:missing_tool:$1"
    exit 2
  fi
}

fail() {
  echo "AGENT_EXECUTION_CORE_FAIL:$1"
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

APP_INGRESS="crates/selene_os/src/app_ingress.rs"
SIM_EXECUTOR="crates/selene_os/src/simulation_executor.rs"
AGENT_INPUT_CONTRACT="crates/selene_kernel_contracts/src/ph1agent.rs"
SIM_FINDER_CONTRACT="crates/selene_kernel_contracts/src/ph1simfinder.rs"

[ -f "$APP_INGRESS" ] || fail "missing_file:$APP_INGRESS"
[ -f "$SIM_EXECUTOR" ] || fail "missing_file:$SIM_EXECUTOR"
[ -f "$AGENT_INPUT_CONTRACT" ] || fail "missing_file:$AGENT_INPUT_CONTRACT"
[ -f "$SIM_FINDER_CONTRACT" ] || fail "missing_file:$SIM_FINDER_CONTRACT"

# Lock Finder packet guarantees as prerequisite.
bash scripts/check_agent_sim_finder_core_acceptance.sh

# 1) Dispatch boundary: tool vs simulation paths must remain split and simulation must route via SimulationExecutor.
require_match "DispatchRequest::Tool\\(tool_request\\) =>" "$APP_INGRESS" \
  "app ingress must keep Tool dispatch branch"
require_match "DispatchRequest::SimulationCandidate\\(_\\) \\| DispatchRequest::AccessStepUp\\(_\\) =>" "$APP_INGRESS" \
  "app ingress must keep simulation/access-step-up branch"
require_match "self\\.executor\\.execute_ph1x_dispatch_simulation_candidate\\(" "$APP_INGRESS" \
  "simulation dispatch must flow through SimulationExecutor"

# 2) SimulationExecutor must fail-closed if Tool dispatch attempts to enter simulation path.
require_match "DispatchRequest::Tool\\(_\\) => Err\\(" "$SIM_EXECUTOR" \
  "SimulationExecutor must reject Tool dispatch in simulation path"
require_match "tool dispatch must be handled by PH1\\.E" "$SIM_EXECUTOR" \
  "SimulationExecutor rejection reason for Tool dispatch must be explicit"

# 3) Guard against contract bypass via ad-hoc packet shape construction outside contract module.
bypass_hits="$(
  rg -n "(SimulationMatchPacket|ClarifyPacket|RefusePacket|MissingSimulationPacket)\\s*\\{" crates \
    | rg -v "^${SIM_FINDER_CONTRACT}:" || true
)"
if [ -n "$bypass_hits" ]; then
  echo "$bypass_hits"
  fail "finder_packet_shape_bypass_detected"
fi

# 4) Agent input contract must carry policy and catalog snapshot anchors for deterministic execution envelope.
require_match "pub policy_context_ref: PolicyContextRef" "$AGENT_INPUT_CONTRACT" \
  "AgentInputPacket must include policy_context_ref"
require_match "pub sim_catalog_snapshot_hash: String" "$AGENT_INPUT_CONTRACT" \
  "AgentInputPacket must include sim_catalog_snapshot_hash"
require_match "pub sim_catalog_snapshot_version: u64" "$AGENT_INPUT_CONTRACT" \
  "AgentInputPacket must include sim_catalog_snapshot_version"

# 5) Optional forward-compat checks: if explicit AEC runtime file exists, enforce required envelope anchors.
AEC_RUNTIME_FILES=(
  "crates/selene_os/src/agent_execution_core.rs"
  "crates/selene_os/src/ph1agent_execution.rs"
)
for file in "${AEC_RUNTIME_FILES[@]}"; do
  if [ -f "$file" ]; then
    require_match "rollback_plan_ref" "$file" \
      "AEC runtime must carry rollback_plan_ref when file exists"
    require_match "policy_snapshot_ref" "$file" \
      "AEC runtime must carry policy_snapshot_ref when file exists"
    require_match "artifact_fingerprint_bundle_ref" "$file" \
      "AEC runtime must carry artifact_fingerprint_bundle_ref when file exists"
  fi
done

echo "CHECK_OK agent_execution_core_dispatch_boundary=pass"
echo "CHECK_OK agent_execution_core_contract_anchors=pass"
echo "CHECK_OK agent_execution_core=pass"
