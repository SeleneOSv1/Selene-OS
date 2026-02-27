#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

# PH1.C round-2 Step 13 acceptance lock.
# 1) OpenAI primary success
cargo test -p selene_engines at_c_step13_openai_primary_success_prefers_primary_slot -- --nocapture >/dev/null
# 2) Google fallback when OpenAI(primary) fails
cargo test -p selene_engines at_c_step13_google_fallback_on_openai_fail_uses_secondary_slot -- --nocapture >/dev/null
# 3) Terminal fail-closed fallback when both providers fail
cargo test -p selene_engines at_c_step13_terminal_fail_closed_when_openai_and_google_fail -- --nocapture >/dev/null
# 4) Partial transcript revision correctness
cargo test -p selene_engines partials_are_ordered_and_deduped_deterministically -- --nocapture >/dev/null
# 5) Provider schema-drift fail-closed behavior (PH1.D typed boundary)
cargo test -p selene_engines at_d_provider_boundary_05_schema_drift_fails_closed -- --nocapture >/dev/null
# 6) Gold-case creation on correction + escalation
cargo test -p selene_os at_feedback_12_gold_case_creation_on_correction_and_escalation -- --nocapture >/dev/null

echo "CHECK_OK ph1c_round2_acceptance_tests=pass"
