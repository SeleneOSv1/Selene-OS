#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

OUTPUT_CSV="${1:-.dev/stage2_canary_metrics_snapshot.csv}"

./scripts/export_builder_stage2_canary_metrics.sh "${OUTPUT_CSV}"
./scripts/check_builder_stage2_promotion_gate.sh "${OUTPUT_CSV}"

echo "CHECK_OK builder_stage3_release_gate=pass"
