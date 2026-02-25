#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

INPUT_CSV="${1:-docs/fixtures/ph1x_interrupt_continuity_snapshot.csv}"

./scripts/check_ph1x_interrupt_continuity_benchmarks.sh "${INPUT_CSV}"

echo "CHECK_OK ph1x_release_gate=pass input=${INPUT_CSV}"
