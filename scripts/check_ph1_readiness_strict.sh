#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

if [ -n "$(git status --porcelain)" ]; then
  echo "CHECK_FAIL ph1_readiness_strict_tree_dirty=fail"
  echo "Run from a clean git tree so readiness evidence is commit-pinned."
  exit 1
fi

echo "CHECK_OK ph1_readiness_strict_tree_clean=pass"

bash scripts/check_ph1_tool_parity.sh
bash scripts/check_agent_sim_finder_core_acceptance.sh
bash scripts/check_agent_execution_core.sh
bash scripts/check_bcast_mhp_acceptance.sh
AUDIT_REQUIRE_CLEAN_TREE=1 bash scripts/selene_design_readiness_audit.sh

echo "CHECK_OK ph1_readiness_strict=pass"
