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
require_cmd grep

fail() {
  echo "DELIVERY_OWNERSHIP_FAIL:$1"
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

require_absent() {
  local pattern="$1"
  local file="$2"
  local msg="$3"
  if rg -n "$pattern" "$file" >/dev/null 2>&1; then
    fail "$msg ($file)"
  fi
}

SIM_CATALOG="docs/08_SIMULATION_CATALOG.md"
SIM_EXECUTOR="crates/selene_os/src/simulation_executor.rs"
PH1_LINK_DOC="docs/DB_WIRING/PH1_LINK.md"
PH1_BCAST_DOC="docs/DB_WIRING/PH1_BCAST.md"
PH1_DELIVERY_DOC="docs/DB_WIRING/PH1_DELIVERY.md"
PH1_REM_DOC="docs/DB_WIRING/PH1_REM.md"
PH1_OS_DOC="docs/DB_WIRING/PH1_OS.md"
KERNEL_LINK="crates/selene_kernel_contracts/src/ph1link.rs"
KERNEL_DELIVERY="crates/selene_kernel_contracts/src/ph1delivery.rs"
PH1_BCAST_RT="crates/selene_os/src/ph1bcast.rs"
PH1_DELIVERY_RT="crates/selene_os/src/ph1delivery.rs"
PH1_DELIVERY_ENGINE_RT="crates/selene_engines/src/ph1delivery.rs"
PH1_REM_RT="crates/selene_os/src/ph1rem.rs"

for legacy_sim in \
  LINK_INVITE_SEND_COMMIT \
  LINK_INVITE_RESEND_COMMIT \
  LINK_DELIVERY_FAILURE_HANDLING_COMMIT
do
  require_match "^\\| ${legacy_sim} \\|.*\\| LEGACY_DO_NOT_WIRE \\|" "$SIM_CATALOG" \
    "${legacy_sim} must remain LEGACY_DO_NOT_WIRE in simulation catalog"
done

require_match "LINK_DELIVER_INVITE" "$SIM_CATALOG" \
  "LINK_DELIVER_INVITE simulation must exist as the only delivery path"

require_absent "LINK_INVITE_SEND_COMMIT|LINK_INVITE_RESEND_COMMIT|LINK_DELIVERY_FAILURE_HANDLING_COMMIT" "$KERNEL_LINK" \
  "legacy LINK delivery simulation ids must not re-enter PH1.LINK kernel contract"

require_match "fn is_legacy_link_delivery_simulation_id" "$SIM_EXECUTOR" \
  "simulation executor must define explicit legacy LINK delivery guard"
require_match "is_legacy_link_delivery_simulation_id\\(&req\\.simulation_id\\)" "$SIM_EXECUTOR" \
  "execute_link must call legacy LINK delivery guard"
require_match "run_broadcast_deliver_with_delivery" "$SIM_EXECUTOR" \
  "simulation executor must expose BCAST->DELIVERY bridge wiring"
require_match "Ph1DeliveryRequest::send_commit_v1" "$SIM_EXECUTOR" \
  "BCAST->DELIVERY bridge must construct PH1.DELIVERY send requests"
require_match "LEGACY_DO_NOT_WIRE: delivery is owned by LINK_DELIVER_INVITE via PH1\\.BCAST \\+ PH1\\.DELIVERY" "$SIM_EXECUTOR" \
  "legacy LINK delivery guard must fail closed with explicit ownership reason"

require_absent "pub fn execute_bcast|pub fn execute_delivery" "$SIM_EXECUTOR" \
  "SimulationExecutor must not expose direct BCAST/DELIVERY execution entrypoints"

require_match "Legacy \\(Do Not Wire\\): .*LINK_DELIVER_INVITE.*PH1\\.BCAST.*PH1\\.DELIVERY" "$PH1_LINK_DOC" \
  "PH1.LINK DB wiring must lock legacy do-not-wire delivery ownership"
require_match "pub struct Ph1DeliveryRequest" "$KERNEL_DELIVERY" \
  "PH1.DELIVERY kernel contract module must exist"
require_match "pub struct Ph1DeliveryRuntime" "$PH1_DELIVERY_ENGINE_RT" \
  "PH1.DELIVERY engine runtime module must exist"
require_match "pub struct Ph1DeliveryWiring" "$PH1_DELIVERY_RT" \
  "PH1.DELIVERY OS wiring module must exist"
require_match "never calls PH1\\.DELIVERY directly" "$PH1_BCAST_DOC" \
  "PH1.BCAST DB wiring must enforce OS-only orchestration to PH1.DELIVERY"
require_match "Outputs_to: Selene OS .* PH1\\.BCAST recipient_state" "$PH1_DELIVERY_DOC" \
  "PH1.DELIVERY DB wiring must return proofs to Selene OS for PH1.BCAST lifecycle update"
require_match "PH1\\.REM remains timing-only" "$PH1_REM_DOC" \
  "PH1.REM DB wiring must stay timing-only"
require_match '`PH1\.BCAST\.001` owns send/resend/escalation recipient lifecycle state' "$PH1_OS_DOC" \
  "PH1.OS DB wiring must lock PH1.BCAST lifecycle ownership"
require_match '`PH1\.DELIVERY` owns provider-attempt execution/proof only' "$PH1_OS_DOC" \
  "PH1.OS DB wiring must lock PH1.DELIVERY ownership as provider-attempt proof only"
require_match "AT-OS-16: delivery ownership drift fails guardrails" "$PH1_OS_DOC" \
  "PH1.OS DB wiring must include explicit delivery ownership guardrail acceptance lock"

require_absent "Ph1Delivery|ph1delivery|Ph1Bcast|ph1bcast" "$PH1_REM_RT" \
  "PH1.REM runtime must not directly call PH1.BCAST/PH1.DELIVERY"
require_absent "Ph1Delivery|ph1delivery" "$PH1_BCAST_RT" \
  "PH1.BCAST runtime wiring must not directly call PH1.DELIVERY"

echo "CHECK_OK delivery_ownership_boundaries=pass"
