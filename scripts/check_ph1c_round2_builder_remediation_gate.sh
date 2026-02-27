#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

# Step 12 lock: recurring failure clusters must map into Builder input,
# and promotion must fail closed without permission + hard-gate evidence.
cargo test -p selene_os at_os_39_builder_remediation_maps_recurring_cluster_to_offline_input -- --nocapture >/dev/null
cargo test -p selene_os at_os_41_builder_remediation_blocks_promote_without_permission_gate_evidence -- --nocapture >/dev/null
cargo test -p selene_os at_os_42_builder_remediation_allows_non_promote_without_gate_proofs -- --nocapture >/dev/null

echo "CHECK_OK ph1c_round2_builder_remediation_gate=pass"
