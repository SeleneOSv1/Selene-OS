#!/usr/bin/env bash
set -euo pipefail

REQUIRED_CMDS=(git rg awk sort uniq comm wc sed cat grep head tail)
MISSING_CMDS=()
for cmd in "${REQUIRED_CMDS[@]}"; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    MISSING_CMDS+=("$cmd")
  fi
done

if [ "${#MISSING_CMDS[@]}" -ne 0 ]; then
  echo "MISSING_TOOLS:${MISSING_CMDS[*]}"
  exit 2
fi

cd "$(git rev-parse --show-toplevel)"

AUDIT_TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/selene_design_readiness_audit.XXXXXXXXXXXX")"
cleanup_audit_tmp_dir() {
  rm -rf "${AUDIT_TMP_DIR}"
}
trap cleanup_audit_tmp_dir EXIT

ACTIVE_BLUEPRINTS_TXT="${AUDIT_TMP_DIR}/active_blueprints.txt"
ACTIVE_CAPS_TXT="${AUDIT_TMP_DIR}/active_caps.txt"
ACTIVE_CAPS_UNIQUE_TXT="${AUDIT_TMP_DIR}/active_caps_unique.txt"
ECM_CAPS_EXACT_TXT="${AUDIT_TMP_DIR}/ecm_caps_exact.txt"
SIM_IDS_TXT="${AUDIT_TMP_DIR}/sim_ids.txt"
ACTIVE_SIMREQ_IDS_TXT="${AUDIT_TMP_DIR}/active_simreq_ids.txt"
ACTIVE_SIMREQ_IDS_UNIQUE_TXT="${AUDIT_TMP_DIR}/active_simreq_ids_unique.txt"

echo "=================================================="
echo "0) REPO STATE (MUST BE REPORTED EXACTLY)"
echo "=================================================="
echo "BRANCH:"; git branch --show-current
echo "PINNED COMMIT HASH:"; git rev-parse HEAD
echo
echo "GIT STATUS (SHORT):"; git status --short
echo
echo "GIT DIFF (NAME ONLY):"; git diff --name-only || true
echo
echo "LAST COMMIT:"; git log -1 --oneline
echo
if [ -n "$(git status --porcelain)" ]; then
  echo "AUDIT_TREE_STATE: DIRTY"
  echo "AUDIT_VALIDITY_NOTE: closure decisions require pinned commit hash plus this dirty-file listing."
else
  echo "AUDIT_TREE_STATE: CLEAN"
  echo "AUDIT_VALIDITY_NOTE: closure decisions may use this run directly."
fi

echo
echo "=================================================="
echo "1) CANONICAL DOC EXISTENCE + BASIC HEALTH"
echo "=================================================="
REQ_DOCS=(
  "docs/04_KERNEL_CONTRACTS.md"
  "docs/05_OS_CONSTITUTION.md"
  "docs/06_ENGINE_MAP.md"
  "docs/07_ENGINE_REGISTRY.md"
  "docs/08_SIMULATION_CATALOG.md"
  "docs/09_BLUEPRINT_REGISTRY.md"
  "docs/10_DB_OWNERSHIP_MATRIX.md"
  "docs/11_DESIGN_LOCK_SEQUENCE.md"
  "docs/COVERAGE_MATRIX.md"
  "docs/13_PROBLEMS_TO_FIX.md"
)
for f in "${REQ_DOCS[@]}"; do
  if [ -f "$f" ]; then echo "OK: $f"; else echo "MISSING: $f"; fi
done

echo
echo "=================================================="
echo "1B) ENGINE TRACKER DUPLICATION GUARDRAIL"
echo "=================================================="
./scripts/check_engine_tracker_duplicates.sh

echo
echo "=================================================="
echo "1B2) RETIRED ENGINE MERGE RESIDUE GUARDRAIL"
echo "=================================================="
./scripts/check_retired_engine_merge_residue.sh

echo
echo "=================================================="
echo "1C) OPTIONAL ENGINE UTILITY GATES (U4/U5)"
echo "=================================================="
./scripts/check_optional_engine_utility_gates.sh docs/fixtures/optional_engine_utility_snapshot.csv --fail-on-u4

echo
echo "=================================================="
echo "1C2) PH1.X INTERRUPT CONTINUITY RELEASE GATE"
echo "=================================================="
./scripts/check_ph1x_release_gate.sh docs/fixtures/ph1x_interrupt_continuity_snapshot.csv

echo
echo "=================================================="
echo "1C3) PH1.K 5E READINESS GATE"
echo "=================================================="
./scripts/check_ph1k_5e_readiness_gate.sh

echo
echo "=================================================="
echo "1C4) PH1.K ROUND-2 BASELINE SNAPSHOT FREEZE"
echo "=================================================="
./scripts/check_ph1k_round2_baseline_snapshot.sh docs/fixtures/ph1k_round2_baseline_snapshot.csv

echo
echo "=================================================="
echo "1C5) PH1.K ROUND-2 BENCHMARK/EVAL SNAPSHOT HARNESS"
echo "=================================================="
./scripts/check_ph1k_round2_eval_snapshot.sh docs/fixtures/ph1k_round2_eval_snapshot.csv

echo
echo "=================================================="
echo "1C6) PH1.K GLOBAL-STANDARD RELEASE GATE"
echo "=================================================="
./scripts/check_ph1k_release_gate.sh docs/fixtures/ph1k_round2_eval_snapshot.csv

echo
echo "=================================================="
echo "1D) RUNTIME BOUNDARY GUARDRAIL (OFFLINE/CONTROL-PLANE)"
echo "=================================================="
./scripts/check_runtime_boundary_guards.sh

echo
echo "=================================================="
echo "1E) DELIVERY OWNERSHIP BOUNDARY GUARDRAIL"
echo "=================================================="
./scripts/check_delivery_ownership_boundaries.sh

echo
echo "=================================================="
echo "1F) UNDERSTANDING + CLARIFY PRECEDENCE GUARDRAIL"
echo "=================================================="
./scripts/check_understanding_clarify_precedence.sh

echo
echo "=================================================="
echo "1G) LEARNING OWNERSHIP BOUNDARY GUARDRAIL"
echo "=================================================="
./scripts/check_learning_ownership_boundaries.sh

echo
echo "=================================================="
echo "1G2) PH1 READ-ONLY TOOL PARITY GUARDRAIL"
echo "=================================================="
./scripts/check_ph1_tool_parity.sh

echo
echo "=================================================="
echo "1H) BUILDER PIPELINE PHASE13-A GUARDRAIL"
echo "=================================================="
./scripts/check_builder_pipeline_phase13a.sh

echo
echo "=================================================="
echo "1I) BUILDER PIPELINE PHASE13-B GUARDRAIL"
echo "=================================================="
./scripts/check_builder_pipeline_phase13b.sh

echo
echo "=================================================="
echo "1J) BUILDER PIPELINE PHASE13-C GUARDRAIL"
echo "=================================================="
./scripts/check_builder_pipeline_phase13c.sh

echo
echo "=================================================="
echo "1K) BUILDER PIPELINE PHASE13-D GUARDRAIL"
echo "=================================================="
./scripts/check_builder_pipeline_phase13d.sh

echo
echo "=================================================="
echo "1L) BUILDER STAGE-2 CANARY REPLAY GUARDRAIL"
echo "=================================================="
./scripts/check_builder_stage2_canary_replay.sh

echo
echo "=================================================="
echo "1M) BUILDER STAGE-2 PROMOTION GATE CHECK"
echo "=================================================="
./scripts/check_builder_stage2_promotion_gate.sh docs/fixtures/stage2_canary_metrics_snapshot.csv

echo
echo "=================================================="
echo "1N) BUILDER STAGE-3 RELEASE GATE (OPTIONAL ENFORCED)"
echo "=================================================="
if [[ "${ENFORCE_STAGE3_RELEASE_GATE:-0}" == "1" ]]; then
  ./scripts/check_builder_stage3_release_gate.sh .dev/stage2_canary_metrics_snapshot.csv
else
  echo "SKIP: set ENFORCE_STAGE3_RELEASE_GATE=1 to require real telemetry export + promotion gate before Stage-3 ramp."
fi

echo
echo "=================================================="
echo "1O) BUILDER HUMAN PERMISSION LOOP (OPTIONAL ENFORCED)"
echo "=================================================="
if [[ "${ENFORCE_BUILDER_HUMAN_PERMISSION:-0}" == "1" ]]; then
  ./scripts/check_builder_human_permission_gate.sh code
  ./scripts/check_builder_human_permission_gate.sh launch
else
  echo "SKIP: set ENFORCE_BUILDER_HUMAN_PERMISSION=1 to require BCAST/REM-backed human code+launch approvals, daily-review freshness, and plain-language issue/fix permission prompts."
fi

echo
echo "=================================================="
echo "1P) BUILDER LEARNING->PATCH BRIDGE GATE (OPTIONAL ENFORCED)"
echo "=================================================="
if [[ "${ENFORCE_BUILDER_LEARNING_BRIDGE:-0}" == "1" ]]; then
  ./scripts/check_builder_learning_bridge_gate.sh
else
  echo "SKIP: set ENFORCE_BUILDER_LEARNING_BRIDGE=1 to require evidence-backed learning reports before learning-triggered builder patching."
fi

echo
echo "=================================================="
echo "1Q) BUILDER PIPELINE PHASE13-E LEARNING BRIDGE CHECK"
echo "=================================================="
./scripts/check_builder_pipeline_phase13e.sh

echo
echo "=================================================="
echo "1R) BUILDER E2E GATE FLOW (OPTIONAL ENFORCED)"
echo "=================================================="
if [[ "${ENFORCE_BUILDER_E2E_GATE_FLOW:-0}" == "1" ]]; then
  ./scripts/check_builder_e2e_gate_flow.sh
else
  echo "SKIP: set ENFORCE_BUILDER_E2E_GATE_FLOW=1 to require one-command learning->approval->stage gate chain."
fi

echo
echo "=================================================="
echo "1S) BUILDER PIPELINE PHASE13-F HUMAN BRIEF AUTOGEN CHECK"
echo "=================================================="
./scripts/check_builder_pipeline_phase13f.sh

echo
echo "=================================================="
echo "1T) BUILDER PIPELINE PHASE13-G PERMISSION PACKET CHECK"
echo "=================================================="
./scripts/check_builder_pipeline_phase13g.sh

echo
echo "=================================================="
echo "1U) BUILDER PIPELINE PHASE13-H DECISION INGEST CHECK"
echo "=================================================="
./scripts/check_builder_pipeline_phase13h.sh

echo
echo "=================================================="
echo "1V) BUILDER PIPELINE PHASE13-I DECISION-FILE INGEST CHECK"
echo "=================================================="
./scripts/check_builder_pipeline_phase13i.sh

echo
echo "=================================================="
echo "1W) BUILDER PIPELINE PHASE13-J DECISION-SEED EXPORT CHECK"
echo "=================================================="
./scripts/check_builder_pipeline_phase13j.sh

echo
echo "=================================================="
echo "1X) BUILDER PIPELINE PHASE13-K DECISION-FILE AUTO-SYNC CHECK"
echo "=================================================="
./scripts/check_builder_pipeline_phase13k.sh

echo
echo "=================================================="
echo "1Y) BUILDER RELEASE HARD GATE (OPTIONAL ENFORCED)"
echo "=================================================="
if [[ "${ENFORCE_BUILDER_RELEASE_HARD_GATE:-0}" == "1" ]]; then
  ./scripts/check_builder_release_hard_gate.sh
else
  echo "SKIP: set ENFORCE_BUILDER_RELEASE_HARD_GATE=1 to require one strict release entrypoint (auto-sync decision files + live telemetry stage gate)."
fi

echo
echo "=================================================="
echo "1Z) BUILDER PIPELINE PHASE13-L HARD-GATE GUARDRAIL CHECK"
echo "=================================================="
./scripts/check_builder_pipeline_phase13l.sh

echo
echo "=================================================="
echo "1AA) BUILDER PIPELINE PHASE13-M CONTROLLED-ROLLOUT START GUARDRAIL CHECK"
echo "=================================================="
./scripts/check_builder_pipeline_phase13m.sh

echo
echo "=================================================="
echo "1AB) BUILDER CONTROLLED ROLLOUT START GATE (OPTIONAL ENFORCED)"
echo "=================================================="
if [[ "${ENFORCE_BUILDER_CONTROLLED_ROLLOUT_START:-0}" == "1" ]]; then
  ./scripts/check_builder_controlled_rollout_start.sh
else
  echo "SKIP: set ENFORCE_BUILDER_CONTROLLED_ROLLOUT_START=1 to require synchronized remote head + freeze-tag parity + replay + strict hard-gate before rollout kickoff."
fi

echo
echo "=================================================="
echo "1AC) BUILDER PIPELINE PHASE13-N ROLLBACK-DRILL GUARDRAIL CHECK"
echo "=================================================="
./scripts/check_builder_pipeline_phase13n.sh

echo
echo "=================================================="
echo "1AD) BUILDER ROLLBACK DRILL (OPTIONAL ENFORCED)"
echo "=================================================="
if [[ "${ENFORCE_BUILDER_ROLLBACK_DRILL:-0}" == "1" ]]; then
  ./scripts/check_builder_rollback_drill.sh
else
  echo "SKIP: set ENFORCE_BUILDER_ROLLBACK_DRILL=1 to require dry-run revert safety proof before rollout progression."
fi

echo
echo "=================================================="
echo "1AE) BUILDER PIPELINE PHASE13-O PRE-LAUNCH BUNDLE GUARDRAIL CHECK"
echo "=================================================="
./scripts/check_builder_pipeline_phase13o.sh

echo
echo "=================================================="
echo "1AF) BUILDER PRE-LAUNCH BUNDLE (OPTIONAL ENFORCED)"
echo "=================================================="
if [[ "${ENFORCE_BUILDER_PRELAUNCH_BUNDLE:-0}" == "1" ]]; then
  ./scripts/check_builder_prelaunch_bundle.sh
else
  echo "SKIP: set ENFORCE_BUILDER_PRELAUNCH_BUNDLE=1 to require rollout-start + rollback-drill + hard-gate final checklist before launch progression."
fi

echo
echo "=================================================="
echo "1AG) BUILDER PIPELINE PHASE13-P CONTROLLED-LAUNCH EXECUTOR GUARDRAIL CHECK"
echo "=================================================="
./scripts/check_builder_pipeline_phase13p.sh

echo
echo "=================================================="
echo "1AH) BUILDER CONTROLLED LAUNCH EXECUTOR (OPTIONAL ENFORCED, PREVIEW-ONLY)"
echo "=================================================="
if [[ "${ENFORCE_BUILDER_CONTROLLED_LAUNCH_EXECUTE:-0}" == "1" ]]; then
  EXECUTE=0 ./scripts/check_builder_controlled_launch_execute.sh
else
  echo "SKIP: set ENFORCE_BUILDER_CONTROLLED_LAUNCH_EXECUTE=1 to require controlled launch-executor preview checks in readiness audit."
fi

echo
echo "=================================================="
echo "1AI) BUILDER PIPELINE PHASE13-Q STAGE-BOUND JUDGE GUARDRAIL CHECK"
echo "=================================================="
./scripts/check_builder_pipeline_phase13q.sh

echo
echo "=================================================="
echo "1AJ) BUILDER STAGE-BOUND JUDGE BINDING (OPTIONAL ENFORCED, PREVIEW-ONLY)"
echo "=================================================="
if [[ "${ENFORCE_BUILDER_STAGE_JUDGE_BINDING:-0}" == "1" ]]; then
  EXECUTE=0 REQUIRE_STAGE_JUDGE=1 ./scripts/check_builder_controlled_launch_execute.sh
else
  echo "SKIP: set ENFORCE_BUILDER_STAGE_JUDGE_BINDING=1 to require stage-bound judge telemetry checks per current release_state in readiness audit."
fi

echo
echo "=================================================="
echo "1AK) BUILDER PIPELINE PHASE13-R PRODUCTION-SOAK WATCHDOG GUARDRAIL CHECK"
echo "=================================================="
./scripts/check_builder_pipeline_phase13r.sh

echo
echo "=================================================="
echo "1AL) BUILDER PRODUCTION SOAK WATCHDOG (OPTIONAL ENFORCED)"
echo "=================================================="
if [[ "${ENFORCE_BUILDER_PRODUCTION_SOAK:-0}" == "1" ]]; then
  ./scripts/check_builder_production_soak_watchdog.sh
else
  echo "SKIP: set ENFORCE_BUILDER_PRODUCTION_SOAK=1 to require fresh production-stage judge telemetry and fail-closed production soak checks."
fi

echo
echo "=================================================="
echo "1AM) BUILDER PIPELINE PHASE13-S PRODUCTION-SOAK RUNNER GUARDRAIL CHECK"
echo "=================================================="
./scripts/check_builder_pipeline_phase13s.sh

echo
echo "=================================================="
echo "1AN) BUILDER PRODUCTION SOAK RUNNER (OPTIONAL ENFORCED, ONCE MODE)"
echo "=================================================="
if [[ "${ENFORCE_BUILDER_PRODUCTION_SOAK_RUNNER:-0}" == "1" ]]; then
  RUN_MODE=once ./scripts/check_builder_production_soak_runner.sh
else
  echo "SKIP: set ENFORCE_BUILDER_PRODUCTION_SOAK_RUNNER=1 to require fail-closed production-soak runner checks (once mode) with BCAST failure-alert dispatch in readiness audit."
fi

echo
echo "=================================================="
echo "1AO) BUILDER PIPELINE PHASE13-T PRODUCTION-SOAK AUTOMATION GUARDRAIL CHECK"
echo "=================================================="
./scripts/check_builder_pipeline_phase13t.sh

echo
echo "=================================================="
echo "1AP) BUILDER PRODUCTION SOAK AUTOMATION STATUS (OPTIONAL ENFORCED)"
echo "=================================================="
if [[ "${ENFORCE_BUILDER_PRODUCTION_SOAK_AUTOMATION:-0}" == "1" ]]; then
  REQUIRE_LOADED=1 ./scripts/status_builder_production_soak_launchd.sh
else
  echo "SKIP: set ENFORCE_BUILDER_PRODUCTION_SOAK_AUTOMATION=1 to require launchd automation loaded-status checks in readiness audit."
fi

echo
echo "=================================================="
echo "2) COVERAGE MATRIX — MUST IDENTIFY TODO/BLOCKER/WIP"
echo "=================================================="
rg -n "TODO|BLOCKER|WIP" docs/COVERAGE_MATRIX.md || true

echo
echo "=================================================="
echo "3) BLUEPRINT REGISTRY DISCIPLINE"
echo "   - Exactly one ACTIVE per intent_type"
echo "   - ACTIVE blueprint must be code-ready: capability_ids resolve + side-effects have sim requirements"
echo "=================================================="

echo "--- 3A) Registry uniqueness: duplicate ACTIVE intents (if any) ---"
dup_active_intents="$(
  awk -F'|' '
    /^\|/ && $0 !~ /^\|---/ {
      for(i=1;i<=NF;i++){gsub(/^ +| +$/, "", $i)}
      intent=$2; status=$5;
      if(intent!="intent_type" && status=="ACTIVE"){print intent}
    }
  ' docs/09_BLUEPRINT_REGISTRY.md | sort | uniq -c | awk '$1>1{print "DUP_ACTIVE_INTENT:",$0}'
)"
if [ -n "${dup_active_intents}" ]; then
  printf "%s\n" "${dup_active_intents}"
  dup_active_intent_count="$(printf "%s\n" "${dup_active_intents}" | wc -l | tr -d ' ')"
  echo "CHECK_FAIL blueprint_active_intent_uniqueness=fail count=${dup_active_intent_count}"
  exit 1
fi
echo "CHECK_OK blueprint_active_intent_uniqueness=pass"

echo
echo "--- 3B) List ACTIVE blueprint files (for later checks) ---"
awk -F'|' '
  /^\|/ && $0 !~ /^\|---/ {
    for(i=1;i<=NF;i++){gsub(/^ +| +$/, "", $i)}
    status=$5; path=$6;
    if(status=="ACTIVE" && path ~ /docs\/BLUEPRINTS/){
      gsub(/^`|`$/, "", path);
      print path
    }
  }
' docs/09_BLUEPRINT_REGISTRY.md > "${ACTIVE_BLUEPRINTS_TXT}"
cat "${ACTIVE_BLUEPRINTS_TXT}"

active_blueprint_count="$(wc -l < "${ACTIVE_BLUEPRINTS_TXT}" | tr -d ' ')"
if [ "${active_blueprint_count}" -eq 0 ]; then
  echo "CHECK_FAIL blueprint_active_paths=fail count=0"
  exit 1
fi

missing_active_blueprint_count=0
while read -r f; do
  [ -n "${f}" ] || continue
  if [ ! -f "${f}" ]; then
    echo "MISSING_ACTIVE_BLUEPRINT_PATH: ${f}"
    missing_active_blueprint_count=$((missing_active_blueprint_count + 1))
  fi
done < "${ACTIVE_BLUEPRINTS_TXT}"

if [ "${missing_active_blueprint_count}" -gt 0 ]; then
  echo "CHECK_FAIL blueprint_active_paths=fail missing=${missing_active_blueprint_count}"
  exit 1
fi
echo "CHECK_OK blueprint_active_paths=pass count=${active_blueprint_count}"

echo
echo "--- 3C) Extract capability_ids used by ACTIVE blueprints ---"
: > "${ACTIVE_CAPS_TXT}"
while read -r f; do
  awk -F'|' '
    /^\| / && $0 !~ /^\|---/ {
      cap=$4; gsub(/^ +| +$/, "", cap);
      if(cap!="capability_id" && cap!=""){print cap}
    }
  ' "$f" >> "${ACTIVE_CAPS_TXT}"
done < "${ACTIVE_BLUEPRINTS_TXT}"
sort -u "${ACTIVE_CAPS_TXT}" > "${ACTIVE_CAPS_UNIQUE_TXT}"
echo "ACTIVE CAPABILITIES (unique):"; wc -l "${ACTIVE_CAPS_UNIQUE_TXT}"
active_cap_count="$(wc -l < "${ACTIVE_CAPS_UNIQUE_TXT}" | tr -d ' ')"
if [ "${active_cap_count}" -eq 0 ]; then
  echo "CHECK_FAIL blueprint_active_capability_extract=fail count=0"
  exit 1
fi
invalid_active_caps="$(grep -Ev '^[A-Z0-9_]+$' "${ACTIVE_CAPS_UNIQUE_TXT}" || true)"
if [ -n "${invalid_active_caps}" ]; then
  printf "%s\n" "${invalid_active_caps}" | sed 's/^/INVALID_ACTIVE_CAPABILITY_ID: /'
  invalid_active_cap_count="$(printf "%s\n" "${invalid_active_caps}" | wc -l | tr -d ' ')"
  echo "CHECK_FAIL blueprint_active_capability_extract=fail invalid=${invalid_active_cap_count}"
  exit 1
fi
echo "CHECK_OK blueprint_active_capability_extract=pass count=${active_cap_count}"

echo
echo "--- 3D) Build ECM capability set (exact tokens) ---"
{
  rg -n '^### capability_id: ' docs/ECM/*.md -S | awk -F'capability_id: ' '{print $2}' | awk '{print $1}';
  rg -n '^#+ `[^`]+`$' docs/ECM/*.md -S | awk -F'`' '{if(NF>=3) print $2}';
} | sort -u > "${ECM_CAPS_EXACT_TXT}"
echo "ECM CAPABILITIES (unique):"; wc -l "${ECM_CAPS_EXACT_TXT}"
ecm_cap_count="$(wc -l < "${ECM_CAPS_EXACT_TXT}" | tr -d ' ')"
if [ "${ecm_cap_count}" -eq 0 ]; then
  echo "CHECK_FAIL ecm_capability_extract=fail count=0"
  exit 1
fi
invalid_ecm_caps="$(grep -Ev '^[A-Z0-9_]+$' "${ECM_CAPS_EXACT_TXT}" || true)"
if [ -n "${invalid_ecm_caps}" ]; then
  printf "%s\n" "${invalid_ecm_caps}" | sed 's/^/INVALID_ECM_CAPABILITY_ID: /'
  invalid_ecm_cap_count="$(printf "%s\n" "${invalid_ecm_caps}" | wc -l | tr -d ' ')"
  echo "CHECK_FAIL ecm_capability_extract=fail invalid=${invalid_ecm_cap_count}"
  exit 1
fi
echo "CHECK_OK ecm_capability_extract=pass count=${ecm_cap_count}"

echo
echo "--- 3E) Report missing capability_ids (ACTIVE blueprints vs ECM) ---"
missing_caps="$(comm -23 "${ACTIVE_CAPS_UNIQUE_TXT}" "${ECM_CAPS_EXACT_TXT}" || true)"
if [ -n "${missing_caps}" ]; then
  printf "%s\n" "${missing_caps}" | sed 's/^/MISSING_CAPABILITY_ID: /'
  missing_cap_count="$(printf "%s\n" "${missing_caps}" | wc -l | tr -d ' ')"
  echo "CHECK_FAIL blueprint_capability_parity=fail count=${missing_cap_count}"
  exit 1
fi
echo "CHECK_OK blueprint_capability_parity=pass"

echo
echo "--- 3F) ACTIVE blueprints: side_effects!=NONE must not have Simulation Requirements: none ---"
bad=0
while read -r f; do
  has_side=$(awk -F'|' '
    BEGIN{x=0}
    /^\| / && $0 !~ /^\|---/ {
      se=$7;
      gsub(/^ +| +$/, "", se);
      if(se!="side_effects" && se!="NONE" && se!="READ_ONLY"){x=1}
    }
    END{print x}
  ' "$f")
  has_none=$(awk '
    BEGIN{s=0;in_sec=0}
    /^## [0-9]+\) Simulation Requirements/{in_sec=1; next}
    /^## [0-9]+\)/{in_sec=0}
    {
      if(in_sec){
        line=tolower($0);
        gsub(/^[[:space:]]*-[[:space:]]+/, "", line);
        sub(/[[:space:]]*\(.*/, "", line);
        gsub(/[[:space:]]+$/, "", line);
        if(line=="none"){s=1}
      }
    }
    END{print s}
  ' "$f" || echo 0)
  if [ "$has_side" = "1" ] && [ "$has_none" = "1" ]; then
    echo "BAD_ACTIVE_SIMREQ_NONE: $f"
    bad=$((bad + 1))
  fi
done < "${ACTIVE_BLUEPRINTS_TXT}"
echo "BAD_ACTIVE_SIMREQ_NONE_FOUND:$bad"
if [ "$bad" -gt 0 ]; then
  echo "CHECK_FAIL blueprint_side_effect_simreq_none=fail count=${bad}"
  exit 1
fi
echo "CHECK_OK blueprint_side_effect_simreq_none=pass"

echo
echo "--- 3G) Simulation IDs listed by ACTIVE blueprints must exist in sim catalog ---"
rg -n "^### [A-Z0-9_]+ \(" docs/08_SIMULATION_CATALOG.md | sed 's/^.*### //;s/ (.*$//' | sort -u > "${SIM_IDS_TXT}"

: > "${ACTIVE_SIMREQ_IDS_TXT}"
while read -r f; do
  awk '
    BEGIN{in_sec=0}
    /^## [0-9]+\) Simulation Requirements/{in_sec=1; next}
    /^## [0-9]+\)/{in_sec=0}
    { if(in_sec && $0 ~ /^- /){
        gsub(/^[[:space:]]*-[[:space:]]+/,"",$0); gsub(/`/,"",$0);
        sim_id=$0;
        sub(/[[:space:]]*\(.*/, "", sim_id);
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", sim_id);
        if(tolower(sim_id)!="none"){
          if(sim_id ~ /^[A-Z0-9_]+$/) print sim_id;
          else print "NON_SIM_TEXT:"FILENAME":"sim_id;
        }
      }
    }
  ' "$f" >> "${ACTIVE_SIMREQ_IDS_TXT}"
done < "${ACTIVE_BLUEPRINTS_TXT}"

echo "NON_SIM_TEXT_LINES (if any):"
non_sim_lines="$(rg -n "^NON_SIM_TEXT:" "${ACTIVE_SIMREQ_IDS_TXT}" || true)"
if [ -n "${non_sim_lines}" ]; then
  printf "%s\n" "${non_sim_lines}"
  non_sim_count="$(printf "%s\n" "${non_sim_lines}" | wc -l | tr -d ' ')"
  echo "CHECK_FAIL blueprint_sim_requirements_non_sim_text=fail count=${non_sim_count}"
  exit 1
fi
echo "CHECK_OK blueprint_sim_requirements_non_sim_text=pass"

grep -v '^NON_SIM_TEXT:' "${ACTIVE_SIMREQ_IDS_TXT}" | sort -u > "${ACTIVE_SIMREQ_IDS_UNIQUE_TXT}"
missing_sim_ids="$(comm -23 "${ACTIVE_SIMREQ_IDS_UNIQUE_TXT}" "${SIM_IDS_TXT}" || true)"
if [ -n "${missing_sim_ids}" ]; then
  printf "%s\n" "${missing_sim_ids}" | sed 's/^/MISSING_SIM_ID: /'
  missing_sim_count="$(printf "%s\n" "${missing_sim_ids}" | wc -l | tr -d ' ')"
  echo "CHECK_FAIL blueprint_sim_catalog_parity=fail count=${missing_sim_count}"
  exit 1
fi
echo "CHECK_OK blueprint_sim_catalog_parity=pass"

echo
echo "=================================================="
echo "4) KERNEL ↔ DB_WIRING ↔ SQL ↔ SIM CATALOG PARITY (DRIFT HOTSPOTS)"
echo "   (Report mismatches; do not fix)"
echo "=================================================="

echo "--- 4A) PH1.LINK key enums (InviteeType + LinkStatus) ---"
echo "KERNEL InviteeType:"; rg -n "pub enum InviteeType" -n crates/selene_kernel_contracts/src/ph1link.rs || true
echo "KERNEL LinkStatus:"; rg -n "pub enum LinkStatus" -n crates/selene_kernel_contracts/src/ph1link.rs || true
echo "SQL onboarding_link_tokens status constraint:"; awk '
  /CREATE TABLE IF NOT EXISTS onboarding_link_tokens / {in_tbl=1}
  in_tbl && ($0 ~ /CREATE TABLE IF NOT EXISTS onboarding_link_tokens/ || $0 ~ /status IN/ || $0 ~ /DRAFT_CREATED|SENT|OPENED|ACTIVATED|CONSUMED|REVOKED|EXPIRED|BLOCKED/) {
    printf "%d:%s\n", NR, $0
  }
  in_tbl && /^\\);/ {exit}
' crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql || true
echo "DB_WIRING PH1_LINK lifecycle line:"; rg -n "lifecycle state is bounded" docs/DB_WIRING/PH1_LINK.md -n || true

echo
echo "--- 4B) Onboarding draft status keywords + constraints ---"
rg -n "onboarding_drafts|DRAFT_CREATED|DRAFT_READY|COMMITTED|REVOKED|EXPIRED" docs/04_KERNEL_CONTRACTS.md docs/DB_WIRING/PH1_LINK.md crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql -n || true

echo
echo "=================================================="
echo "5) DRIFT/BANNED LEGACY TOKENS SWEEP (CASE-INSENSITIVE)"
echo "   (Report any hit with file:line)"
echo "=================================================="
legacy_token_hits="$(rg -ni "ready_to_send|household|contractor|referral|\blink_id\b|\blinkid\b" docs crates -S --glob '!docs/archive/**' || true)"
if [ -n "${legacy_token_hits}" ]; then
  printf "%s\n" "${legacy_token_hits}"
  legacy_token_hit_count="$(printf "%s\n" "${legacy_token_hits}" | wc -l | tr -d ' ')"
  echo "CHECK_FAIL banned_legacy_token_sweep=fail count=${legacy_token_hit_count}"
  exit 1
fi
echo "CHECK_OK banned_legacy_token_sweep=pass"

echo
echo "=================================================="
echo "6) SIM CATALOG LEGACY COMPLIANCE EVIDENCE (LINK DOMAIN)"
echo "   - The lines below are expected evidence when legacy LINK delivery sims remain marked LEGACY_DO_NOT_WIRE."
echo "   - Treat as a finding only if status is not LEGACY_DO_NOT_WIRE or wording contradicts LINK_DELIVER_INVITE ownership."
echo "=================================================="
rg -n "LINK_INVITE_SEND_COMMIT|LINK_INVITE_RESEND_COMMIT|LINK_DELIVERY_FAILURE_HANDLING_COMMIT|LEGACY_DO_NOT_WIRE" docs/08_SIMULATION_CATALOG.md -n || true
rg -n "PH1\.LINK" docs/08_SIMULATION_CATALOG.md -n || true

legacy_ids=(
  "LINK_INVITE_SEND_COMMIT"
  "LINK_INVITE_RESEND_COMMIT"
  "LINK_DELIVERY_FAILURE_HANDLING_COMMIT"
)

legacy_link_fail_count=0
for legacy_id in "${legacy_ids[@]}"; do
  row_line="$(rg -n "^\| ${legacy_id} \|" docs/08_SIMULATION_CATALOG.md || true)"
  if [ -z "${row_line}" ]; then
    echo "LEGACY_LINK_MISSING_ROW:${legacy_id}"
    legacy_link_fail_count=$((legacy_link_fail_count + 1))
    continue
  fi
  if ! printf "%s\n" "${row_line}" | rg -q "LEGACY_DO_NOT_WIRE"; then
    echo "LEGACY_LINK_BAD_STATUS:${legacy_id}"
    legacy_link_fail_count=$((legacy_link_fail_count + 1))
  fi
  if ! printf "%s\n" "${row_line}" | rg -qi "delivery .*LINK_DELIVER_INVITE"; then
    echo "LEGACY_LINK_BAD_OWNERSHIP_TEXT:${legacy_id}"
    legacy_link_fail_count=$((legacy_link_fail_count + 1))
  fi

  header_line_raw="$(rg -n "^### ${legacy_id} \(COMMIT\)" docs/08_SIMULATION_CATALOG.md || true)"
  if [ -z "${header_line_raw}" ]; then
    echo "LEGACY_LINK_MISSING_SECTION:${legacy_id}"
    legacy_link_fail_count=$((legacy_link_fail_count + 1))
    continue
  fi
  header_line="$(printf "%s\n" "${header_line_raw}" | head -n 1 | cut -d: -f1)"
  section_window="$(sed -n "${header_line},$((header_line+8))p" docs/08_SIMULATION_CATALOG.md)"
  if ! printf "%s\n" "${section_window}" | rg -q "LEGACY_DO_NOT_WIRE"; then
    echo "LEGACY_LINK_SECTION_BAD_STATUS:${legacy_id}"
    legacy_link_fail_count=$((legacy_link_fail_count + 1))
  fi
  if ! printf "%s\n" "${section_window}" | rg -q "LINK_DELIVER_INVITE"; then
    echo "LEGACY_LINK_SECTION_BAD_OWNERSHIP:${legacy_id}"
    legacy_link_fail_count=$((legacy_link_fail_count + 1))
  fi
done

if [ "${legacy_link_fail_count}" -gt 0 ]; then
  echo "CHECK_FAIL legacy_link_sim_catalog_compliance=fail count=${legacy_link_fail_count}"
  exit 1
fi
echo "CHECK_OK legacy_link_sim_catalog_compliance=pass"

echo
echo "=================================================="
echo "7) OUTPUT FORMAT REQUIRED (IN YOUR CHAT RESPONSE)"
echo "=================================================="
echo "In your reply, paste:"
echo "A) This full terminal output (do not summarize it away)."
echo "B) Then a structured issue list with:"
echo "   - Severity (BLOCKER/RISK/CLEANUP)"
echo "   - File:line references"
echo "   - Why it matters (drift risk / broken contract / bad relationship)"
echo "   - The smallest safe fix (DESIGN/DOC ONLY; no code changes) and the proof command to confirm."
echo
echo "END OF AUDIT"
