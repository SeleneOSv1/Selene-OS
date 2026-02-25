#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

PLAN_DOC="docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md"
TRACKER_DOC="docs/33_ENGINE_REVIEW_TRACKER.md"
DB_WIRING_DOC="docs/DB_WIRING/PH1_K.md"
ECM_DOC="docs/ECM/PH1_K.md"
K_RUNTIME_SRC="crates/selene_engines/src/ph1k.rs"
X_CONTRACT_SRC="crates/selene_kernel_contracts/src/ph1x.rs"

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/ph1k_5e_gate.XXXXXXXX")"
cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

FIVE_E_SECTION="${TMP_DIR}/5e_section.txt"
awk '/^### 5E\)/{flag=1} /^### Round 3:/{flag=0} flag{print}' "${PLAN_DOC}" > "${FIVE_E_SECTION}"

fail=0

check_fixed() {
  local file="$1"
  local pattern="$2"
  local code="$3"
  if rg -q --fixed-strings "${pattern}" "${file}"; then
    echo "CHECK_OK:${code}"
  else
    echo "GATE_FAIL:${code}"
    fail=1
  fi
}

for step in 1 2 3 4 5 6 7 8 9 10; do
  check_fixed "${FIVE_E_SECTION}" "Step ${step}." "ph1k_5e_step_${step}_declared"
  check_fixed "${FIVE_E_SECTION}" "Step ${step}: COMPLETE" "ph1k_5e_step_${step}_complete"
done

check_fixed "${PLAN_DOC}" "Step 14: COMPLETE (closure/handoff completed; build ledger proof appended and 5D is now fully closed for transition to 5E Step 1)." "ph1x_5d_handoff_complete"
check_fixed "${PLAN_DOC}" "PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE -> PH1.BUILDER" "learning_sink_chain_declared"
check_fixed "${TRACKER_DOC}" '`5E` Step 1-10 complete;' "tracker_progress_locked"

check_fixed "${DB_WIRING_DOC}" "timing_markers" "db_wiring_timing_markers_documented"
check_fixed "${DB_WIRING_DOC}" "speech_window_metrics" "db_wiring_speech_window_metrics_documented"
check_fixed "${DB_WIRING_DOC}" "subject_relation_confidence_bundle" "db_wiring_subject_bundle_documented"
check_fixed "${DB_WIRING_DOC}" "K_INTERRUPT_LEXICAL_TRIGGER_REJECTED" "db_wiring_reason_code_lexical_rejected"
check_fixed "${DB_WIRING_DOC}" "K_INTERRUPT_NOISE_GATE_REJECTED" "db_wiring_reason_code_noise_rejected"
check_fixed "${DB_WIRING_DOC}" "K_INTERRUPT_CANDIDATE_EMITTED_HIGH" "db_wiring_reason_code_candidate_high"
check_fixed "${DB_WIRING_DOC}" "K_INTERRUPT_FEEDBACK_FALSE_LEXICAL_TRIGGER" "db_wiring_feedback_reason_code_false"

check_fixed "${ECM_DOC}" "timing_markers" "ecm_timing_markers_documented"
check_fixed "${ECM_DOC}" "speech_window_metrics" "ecm_speech_window_metrics_documented"
check_fixed "${ECM_DOC}" "subject_relation_confidence_bundle" "ecm_subject_bundle_documented"
check_fixed "${ECM_DOC}" "PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE -> PH1.BUILDER" "ecm_learning_sink_chain_documented"

check_fixed "${K_RUNTIME_SRC}" "build_interrupt_feedback_signal" "runtime_feedback_mapper_exists"
check_fixed "${K_RUNTIME_SRC}" "K_INTERRUPT_CANDIDATE_EMITTED_HIGH" "runtime_reason_code_high_exists"
check_fixed "${K_RUNTIME_SRC}" "K_INTERRUPT_NOISE_GATE_REJECTED" "runtime_reason_code_noise_rejected_exists"

check_fixed "${X_CONTRACT_SRC}" "PH1X_MAX_INTERRUPT_SNAPSHOT_AGE_NS" "x_contract_snapshot_age_guard_exists"
check_fixed "${X_CONTRACT_SRC}" "subject_relation_confidence_bundle" "x_contract_subject_bundle_guard_exists"
check_fixed "${X_CONTRACT_SRC}" "timing_markers.window_end" "x_contract_timing_window_guard_exists"

if [[ "${fail}" -ne 0 ]]; then
  exit 1
fi

echo "CHECK_OK ph1k_5e_readiness_gate=pass"
