#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

# Dry-run rollback safety drill:
# 1) prove post-deploy judge can force deterministic rollback on regression
# 2) prove missing gate outcomes fail-closed (no unsafe judge execution)
cargo test -p selene_os at_builder_os_07_post_deploy_judge_reverts_on_latency_threshold_breach -- --nocapture >/dev/null
cargo test -p selene_os at_builder_os_08_post_deploy_judge_refuses_missing_gate_outcomes -- --nocapture >/dev/null

echo "CHECK_OK builder_rollback_drill=pass"
