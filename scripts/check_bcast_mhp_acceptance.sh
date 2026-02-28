#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

if ! command -v cargo >/dev/null 2>&1; then
  echo "MISSING_TOOL:cargo"
  exit 2
fi

tests=(
  at_bcast_mhp_01_selene_app_first
  at_bcast_mhp_02_non_urgent_followup_waits_five_minutes
  at_bcast_mhp_03_urgent_followup_immediate_after_delivery
  at_bcast_mhp_04_app_reply_auto_concludes_and_forwards_to_wife
  at_bcast_mhp_05_reminder_set_and_fired_flow_via_ph1_rem
  at_bcast_mhp_06_fallback_order_only_when_app_unavailable
)

for test_name in "${tests[@]}"; do
  cargo test -p selene_os "simulation_executor::tests::${test_name}" -- --nocapture
done

echo "CHECK_OK bcast_mhp_acceptance=pass"
