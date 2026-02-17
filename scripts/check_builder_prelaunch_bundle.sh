#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

# Final pre-launch checklist in one command:
# 1) synchronized rollout-start gate (includes hard gate + replay checks)
# 2) rollback drill safety proof
# 3) explicit hard-gate recheck as final confirmation
bash scripts/check_builder_controlled_rollout_start.sh
bash scripts/check_builder_rollback_drill.sh
bash scripts/check_builder_release_hard_gate.sh

echo "CHECK_OK builder_prelaunch_bundle=pass"
