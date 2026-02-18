#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

cargo test -p selene_os at_builder_os_05_release_controller_blocks_production_without_resolved_approval -- --nocapture >/dev/null
cargo test -p selene_os at_builder_os_06_release_controller_promotes_after_required_approval -- --nocapture >/dev/null
cargo test -p selene_os at_builder_os_09_release_controller_promotes_staging_to_canary_when_approved -- --nocapture >/dev/null
cargo test -p selene_os at_builder_os_10_release_controller_promotes_canary_to_ramp_stages_when_approved -- --nocapture >/dev/null
cargo test -p selene_os at_builder_os_11_class_c_requires_dual_approval_before_production_promotion -- --nocapture >/dev/null

echo "CHECK_OK builder_stage2_canary_replay=pass"
