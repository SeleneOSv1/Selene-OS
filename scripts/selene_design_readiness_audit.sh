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
awk -F'|' '
  /^\|/ && $0 !~ /^\|---/ {
    for(i=1;i<=NF;i++){gsub(/^ +| +$/, "", $i)}
    intent=$2; status=$5;
    if(intent!="intent_type" && status=="ACTIVE"){print intent}
  }
' docs/09_BLUEPRINT_REGISTRY.md | sort | uniq -c | awk '$1>1{print "DUP_ACTIVE_INTENT:",$0}' || true

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
' docs/09_BLUEPRINT_REGISTRY.md > /tmp/active_blueprints.txt
cat /tmp/active_blueprints.txt

echo
echo "--- 3C) Extract capability_ids used by ACTIVE blueprints ---"
: > /tmp/active_caps.txt
while read -r f; do
  awk -F'|' '
    /^\| / && $0 !~ /^\|---/ {
      cap=$4; gsub(/^ +| +$/, "", cap);
      if(cap!="capability_id" && cap!=""){print cap}
    }
  ' "$f" >> /tmp/active_caps.txt
done < /tmp/active_blueprints.txt
sort -u /tmp/active_caps.txt > /tmp/active_caps_unique.txt
echo "ACTIVE CAPABILITIES (unique):"; wc -l /tmp/active_caps_unique.txt

echo
echo "--- 3D) Build ECM capability set (exact tokens) ---"
{
  rg -n '^### capability_id: ' docs/ECM/*.md -S | awk -F'capability_id: ' '{print $2}' | awk '{print $1}';
  rg -n '^#+ `[^`]+`$' docs/ECM/*.md -S | awk -F'`' '{if(NF>=3) print $2}';
} | sort -u > /tmp/ecm_caps_exact.txt
echo "ECM CAPABILITIES (unique):"; wc -l /tmp/ecm_caps_exact.txt

echo
echo "--- 3E) Report missing capability_ids (ACTIVE blueprints vs ECM) ---"
comm -23 /tmp/active_caps_unique.txt /tmp/ecm_caps_exact.txt | sed 's/^/MISSING_CAPABILITY_ID: /' || true

echo
echo "--- 3F) ACTIVE blueprints: side_effects!=NONE must not have Simulation Requirements: none ---"
bad=0
while read -r f; do
  has_side=$(awk -F'|' 'BEGIN{x=0} /^\| / && $0 !~ /^\|---/ {se=$7; gsub(/^ +| +$/, "", se); if(se!="side_effects" && se!="NONE"){x=1}} END{print x}' "$f")
  has_none=$(awk 'BEGIN{s=0;in_sec=0} /^## [0-9]+\) Simulation Requirements/{in_sec=1; next} /^## [0-9]+\)/{in_sec=0} {if(in_sec){line=tolower($0); if(line ~ /^- *none *$/){s=1}}} END{print s}' "$f" || echo 0)
  if [ "$has_side" = "1" ] && [ "$has_none" = "1" ]; then
    echo "BAD_ACTIVE_SIMREQ_NONE: $f"
    bad=1
  fi
done < /tmp/active_blueprints.txt
echo "BAD_ACTIVE_SIMREQ_NONE_FOUND:$bad"

echo
echo "--- 3G) Simulation IDs listed by ACTIVE blueprints must exist in sim catalog ---"
rg -n "^### [A-Z0-9_]+ \(" docs/08_SIMULATION_CATALOG.md | sed 's/^.*### //;s/ (.*$//' | sort -u > /tmp/sim_ids.txt

: > /tmp/active_simreq_ids.txt
while read -r f; do
  awk '
    BEGIN{in_sec=0}
    /^## [0-9]+\) Simulation Requirements/{in_sec=1; next}
    /^## [0-9]+\)/{in_sec=0}
    { if(in_sec && $0 ~ /^- /){
        gsub(/^[[:space:]]*-[[:space:]]+/,"",$0); gsub(/`/,"",$0);
        if(tolower($0)!="none"){
          sub(/[[:space:]]*\(.*/,"",$0);
          if($0 ~ /^[A-Z0-9_]+$/) print $0;
          else print "NON_SIM_TEXT:"FILENAME":"$0;
        }
      }
    }
  ' "$f" >> /tmp/active_simreq_ids.txt
done < /tmp/active_blueprints.txt

echo "NON_SIM_TEXT_LINES (if any):"
rg -n "^NON_SIM_TEXT:" /tmp/active_simreq_ids.txt || true

grep -v '^NON_SIM_TEXT:' /tmp/active_simreq_ids.txt | sort -u > /tmp/active_simreq_ids_unique.txt
comm -23 /tmp/active_simreq_ids_unique.txt /tmp/sim_ids.txt | sed 's/^/MISSING_SIM_ID: /' || true

echo
echo "=================================================="
echo "4) KERNEL ↔ DB_WIRING ↔ SQL ↔ SIM CATALOG PARITY (DRIFT HOTSPOTS)"
echo "   (Report mismatches; do not fix)"
echo "=================================================="

echo "--- 4A) PH1.LINK key enums (InviteeType + LinkStatus) ---"
echo "KERNEL InviteeType:"; rg -n "pub enum InviteeType" -n crates/selene_kernel_contracts/src/ph1link.rs || true
echo "KERNEL LinkStatus:"; rg -n "pub enum LinkStatus" -n crates/selene_kernel_contracts/src/ph1link.rs || true
echo "SQL onboarding_link_tokens status constraint:"; rg -n "CREATE TABLE IF NOT EXISTS onboarding_link_tokens|status IN" crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql -n || true
echo "DB_WIRING PH1_LINK lifecycle line:"; rg -n "lifecycle state is bounded" docs/DB_WIRING/PH1_LINK.md -n || true

echo
echo "--- 4B) Onboarding draft status keywords + constraints ---"
rg -n "onboarding_drafts|DRAFT_CREATED|DRAFT_READY|COMMITTED|REVOKED|EXPIRED" docs/04_KERNEL_CONTRACTS.md docs/DB_WIRING/PH1_LINK.md crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql -n || true

echo
echo "=================================================="
echo "5) DRIFT/BANNED LEGACY TOKENS SWEEP (CASE-INSENSITIVE)"
echo "   (Report any hit with file:line)"
echo "=================================================="
rg -ni "ready_to_send|household|contractor|referral|\blink_id\b|\blinkid\b" docs crates -S || true

echo
echo "=================================================="
echo "6) SIM CATALOG DISCIPLINE"
echo "   - Any sim referenced by blueprints must exist and be ACTIVE (or explicitly LEGACY_DO_NOT_WIRE when appropriate)"
echo "   - Look for stale statuses / delivery-semantics in Link domain"
echo "=================================================="
rg -n "LINK_INVITE_SEND_COMMIT|LINK_INVITE_RESEND_COMMIT|LINK_DELIVERY_FAILURE_HANDLING_COMMIT|LEGACY_DO_NOT_WIRE" docs/08_SIMULATION_CATALOG.md -n || true
rg -n "PH1\.LINK" docs/08_SIMULATION_CATALOG.md -n || true

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
