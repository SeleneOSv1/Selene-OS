#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

FREEZE_TAG="${FREEZE_TAG:-freeze-stage3-fresh-cycle-20260217}"
REMOTE_NAME="${REMOTE_NAME:-origin}"
REMOTE_BRANCH="${REMOTE_BRANCH:-main}"

local_head="$(git rev-parse HEAD)"
remote_head="$(git ls-remote --heads "${REMOTE_NAME}" "${REMOTE_BRANCH}" | awk '{print $1}')"
if [[ -z "${remote_head}" ]]; then
  echo "ROLLOUT_START_FAIL:missing_remote_head remote=${REMOTE_NAME} branch=${REMOTE_BRANCH}" >&2
  exit 1
fi
if [[ "${local_head}" != "${remote_head}" ]]; then
  echo "ROLLOUT_START_FAIL:local_remote_head_mismatch local=${local_head} remote=${remote_head}" >&2
  exit 1
fi

if ! git rev-parse "${FREEZE_TAG}^{}" >/dev/null 2>&1; then
  echo "ROLLOUT_START_FAIL:missing_local_freeze_tag tag=${FREEZE_TAG}" >&2
  exit 1
fi
local_tag_target="$(git rev-parse "${FREEZE_TAG}^{}")"
remote_tag_target="$(git ls-remote --tags "${REMOTE_NAME}" "${FREEZE_TAG}^{}" | awk '{print $1}')"
if [[ -z "${remote_tag_target}" ]]; then
  echo "ROLLOUT_START_FAIL:missing_remote_freeze_tag tag=${FREEZE_TAG}" >&2
  exit 1
fi
if [[ "${local_tag_target}" != "${remote_tag_target}" ]]; then
  echo "ROLLOUT_START_FAIL:freeze_tag_target_mismatch local=${local_tag_target} remote=${remote_tag_target}" >&2
  exit 1
fi

# Replay test coverage for release-controller state transitions.
bash scripts/check_builder_stage2_canary_replay.sh

# Canonical strict release readiness gate (live telemetry + approvals + learning bridge).
bash scripts/check_builder_release_hard_gate.sh

echo "CHECK_OK builder_controlled_rollout_start=pass commit=${local_head} freeze_tag=${FREEZE_TAG} freeze_target=${local_tag_target}"
