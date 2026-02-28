#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "MISSING_TOOL:$1"
    exit 2
  fi
}

require_cmd rg
require_cmd awk
require_cmd sort
require_cmd comm
require_cmd mktemp

fail() {
  echo "PH1_TOOL_PARITY_FAIL:$1"
  exit 1
}

require_match() {
  local pattern="$1"
  local file="$2"
  local msg="$3"
  if ! rg -n "$pattern" "$file" >/dev/null 2>&1; then
    fail "$msg ($file)"
  fi
}

require_file() {
  local path="$1"
  local msg="$2"
  if [ ! -f "$path" ]; then
    fail "$msg ($path)"
  fi
}

require_dispatch_mapping() {
  local file="$1"
  local intent="$2"
  local tool="$3"
  local label="$4"
  if ! awk -v intent="IntentType::${intent}" -v tool="ToolName::${tool}" '
    /Read-only tool dispatch \(PH1.E\)\./ { in_block=1 }
    in_block && /return self.out_dispatch_tool/ { exit }
    in_block {
      if ($0 ~ intent " =>") { window=4 }
      if (window > 0) {
        if ($0 ~ tool) found=1
        window--
      }
    }
    END { exit(found ? 0 : 1) }
  ' "$file"; then
    fail "${label} missing dispatch mapping ${intent} -> ${tool} (${file})"
  fi
}

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/ph1_tool_parity.XXXXXXXX")"
cleanup_tmp_dir() {
  rm -rf "$TMP_DIR"
}
trap cleanup_tmp_dir EXIT

BLUEPRINT_REGISTRY="docs/09_BLUEPRINT_REGISTRY.md"
COVERAGE_MATRIX="docs/COVERAGE_MATRIX.md"
NEW_CHAT_CONTEXT="docs/14_NEW_CHAT_SYSTEM_CONTEXT.md"
FULL_BUILD_CONTEXT="docs/15_FULL_SYSTEM_BUILD_CONTEXT.md"

KERNEL_N="crates/selene_kernel_contracts/src/ph1n.rs"
KERNEL_E="crates/selene_kernel_contracts/src/ph1e.rs"
ENGINE_N="crates/selene_engines/src/ph1n.rs"
ENGINE_E="crates/selene_engines/src/ph1e.rs"
ENGINE_X="crates/selene_engines/src/ph1x.rs"
OS_X="crates/selene_os/src/ph1x.rs"
TOOLS_E="crates/selene_tools/src/ph1e.rs"
OS_SIM_EXEC="crates/selene_os/src/simulation_executor.rs"
APP_INGRESS="crates/selene_os/src/app_ingress.rs"

BLUEPRINT_IDS=(
  "TOOL_TIME_QUERY"
  "TOOL_WEATHER_QUERY"
  "TOOL_WEB_SEARCH"
  "TOOL_NEWS"
  "TOOL_URL_FETCH_AND_CITE"
  "TOOL_DOCUMENT_UNDERSTAND"
  "TOOL_PHOTO_UNDERSTAND"
  "TOOL_DATA_ANALYSIS"
  "TOOL_DEEP_RESEARCH"
  "TOOL_RECORD_MODE"
  "TOOL_CONNECTOR_QUERY"
)
INTENT_TYPES=(
  "TimeQuery"
  "WeatherQuery"
  "WebSearchQuery"
  "NewsQuery"
  "UrlFetchAndCiteQuery"
  "DocumentUnderstandQuery"
  "PhotoUnderstandQuery"
  "DataAnalysisQuery"
  "DeepResearchQuery"
  "RecordModeQuery"
  "ConnectorQuery"
)
INTENT_TOKENS=(
  "TIME_QUERY"
  "WEATHER_QUERY"
  "WEB_SEARCH_QUERY"
  "NEWS_QUERY"
  "URL_FETCH_AND_CITE_QUERY"
  "DOCUMENT_UNDERSTAND_QUERY"
  "PHOTO_UNDERSTAND_QUERY"
  "DATA_ANALYSIS_QUERY"
  "DEEP_RESEARCH_QUERY"
  "RECORD_MODE_QUERY"
  "CONNECTOR_QUERY"
)
TOOL_VARIANTS=(
  "Time"
  "Weather"
  "WebSearch"
  "News"
  "UrlFetchAndCite"
  "DocumentUnderstand"
  "PhotoUnderstand"
  "DataAnalysis"
  "DeepResearch"
  "RecordMode"
  "ConnectorQuery"
)
TOOL_KEYS=(
  "time"
  "weather"
  "web_search"
  "news"
  "url_fetch_and_cite"
  "document_understand"
  "photo_understand"
  "data_analysis"
  "deep_research"
  "record_mode"
  "connector_query"
)

EXPECTED_BP="$TMP_DIR/expected_blueprints.txt"
ACTUAL_BP="$TMP_DIR/actual_blueprints.txt"
printf '%s\n' "${BLUEPRINT_IDS[@]}" | sort -u > "$EXPECTED_BP"
awk -F'|' '
  /^\| TOOL_/ && $0 !~ /^\|---/ {
    for(i=1;i<=NF;i++){gsub(/^ +| +$/, "", $i)}
    print $2
  }
' "$BLUEPRINT_REGISTRY" | sort -u > "$ACTUAL_BP"

missing_bp="$(comm -23 "$EXPECTED_BP" "$ACTUAL_BP" | tr '\n' ' ' | sed 's/[[:space:]]\+$//')"
unexpected_bp="$(comm -13 "$EXPECTED_BP" "$ACTUAL_BP" | tr '\n' ' ' | sed 's/[[:space:]]\+$//')"

if [ -n "$missing_bp" ]; then
  fail "missing tool blueprints in registry: $missing_bp"
fi
if [ -n "$unexpected_bp" ]; then
  fail "unexpected tool blueprints in registry: $unexpected_bp"
fi

for i in "${!BLUEPRINT_IDS[@]}"; do
  bp="${BLUEPRINT_IDS[$i]}"
  intent="${INTENT_TYPES[$i]}"
  token="${INTENT_TOKENS[$i]}"
  tool="${TOOL_VARIANTS[$i]}"
  key="${TOOL_KEYS[$i]}"

  require_match "^\\| ${bp} \\| ${bp} \\| v1 \\| ACTIVE \\| \`docs/BLUEPRINTS/${bp}\\.md\` \\|" \
    "$BLUEPRINT_REGISTRY" "blueprint row must stay ACTIVE for ${bp}"
  require_file "docs/BLUEPRINTS/${bp}.md" "blueprint file missing for ${bp}"

  require_match "${bp}" "$NEW_CHAT_CONTEXT" "new chat context missing ${bp}"
  require_match "${bp}" "$FULL_BUILD_CONTEXT" "full build context missing ${bp}"
  require_match "${bp}" "$COVERAGE_MATRIX" "coverage matrix missing ${bp}"

  require_match "${intent}," "$KERNEL_N" "kernel intent enum missing ${intent}"
  require_match "push\\(IntentType::${intent}\\)" "$ENGINE_N" "engine NLP detection missing ${intent}"
  require_match "IntentType::${intent} =>" "$ENGINE_N" "engine NLP mapping missing ${intent}"

  require_dispatch_mapping "$ENGINE_X" "$intent" "$tool" "engine PH1.X"
  require_dispatch_mapping "$OS_X" "$intent" "$tool" "os PH1.X"

  require_match "IntentType::${intent} => \"${token}\"" "$OS_SIM_EXEC" \
    "simulation executor token missing ${intent}"

  require_match "${tool}," "$KERNEL_E" "kernel ToolName missing ${tool}"
  require_match "ToolName::${tool} => \"${key}\"" "$KERNEL_E" "kernel ToolName.as_str missing ${tool}"
  require_match "ToolName::${tool} =>" "$ENGINE_E" \
    "engine PH1.E runtime missing ${tool} branch"
  require_match "ToolResult::${tool}" "$ENGINE_E" \
    "engine PH1.E runtime missing ${tool} result variant"
  require_match "\"${key}\" => reason_codes::E_OK_" "$TOOLS_E" \
    "tool router ok reason-code mapping missing ${key}"
done

require_match "^\\| PH1\\.E \\|.*\\| \\[\\] \\| \\[" "$COVERAGE_MATRIX" \
  "PH1.E coverage row must remain read-only with empty simulations list"

E2E_TESTS=(
  "run_a_desktop_voice_turn_end_to_end_dispatches_web_search_and_returns_provenance"
  "run_b_desktop_voice_turn_end_to_end_dispatches_news_and_returns_provenance"
  "run_c_desktop_voice_turn_end_to_end_dispatches_url_fetch_and_cite_and_returns_provenance"
  "run_d_desktop_voice_turn_end_to_end_dispatches_document_understand_and_returns_provenance"
  "run_e_desktop_voice_turn_end_to_end_dispatches_photo_understand_and_returns_provenance"
  "run_da_desktop_voice_turn_end_to_end_dispatches_data_analysis_and_returns_provenance"
  "run_dr_desktop_voice_turn_end_to_end_dispatches_deep_research_and_returns_provenance"
  "run_rm_desktop_voice_turn_end_to_end_dispatches_record_mode_and_returns_provenance"
  "run_cn_desktop_voice_turn_end_to_end_dispatches_connector_query_and_returns_provenance"
)
for test_name in "${E2E_TESTS[@]}"; do
  require_match "fn ${test_name}\\(" "$APP_INGRESS" "missing desktop e2e test ${test_name}"
done

echo "CHECK_OK ph1_tool_parity=pass"
