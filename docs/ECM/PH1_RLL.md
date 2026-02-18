# PH1_RLL ECM (Design vNext)

## Engine Header
- engine_id: PH1.RLL
- role: Offline RL ladder ranking for governed artifact recommendations
- placement: OFFLINE_ONLY

## Capability List

### capability_id: RLL_POLICY_RANK_OFFLINE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_candidates`, `max_recommendations`, `offline_pipeline_only=true`)
  - bounded artifact candidates (`artifact_id`, optimization target, expected_effect_bp, confidence, approval_tier, evidence_ref)
  - bounded training controls (`training_window_days`, `minimum_sample_size`)
- output_schema:
  - one selected artifact id
  - deterministic ordered recommendations
  - boundary flags: `offline_only=true`, `approval_required=true`, `no_execution_authority=true`
- allowed_callers: OFFLINE_PIPELINE_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, OFFLINE_ONLY_REQUIRED, VALIDATION_FAILED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_RLL_OK_POLICY_RANK_OFFLINE
  - PH1_RLL_INPUT_SCHEMA_INVALID
  - PH1_RLL_UPSTREAM_INPUT_MISSING
  - PH1_RLL_BUDGET_EXCEEDED
  - PH1_RLL_OFFLINE_ONLY_REQUIRED
  - PH1_RLL_VALIDATION_FAILED
  - PH1_RLL_INTERNAL_PIPELINE_ERROR

### capability_id: RLL_ARTIFACT_RECOMMEND
- input_schema:
  - selected artifact id + ordered recommendations from `RLL_POLICY_RANK_OFFLINE`
  - deterministic envelope constraints (`offline_pipeline_only=true`)
- output_schema:
  - validation_result (`OK|FAIL`)
  - bounded diagnostics
  - boundary flags: `offline_only=true`, `no_execution_authority=true`
- allowed_callers: OFFLINE_PIPELINE_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, BUDGET_EXCEEDED, OFFLINE_ONLY_REQUIRED
- reason_codes:
  - PH1_RLL_OK_ARTIFACT_RECOMMEND
  - PH1_RLL_VALIDATION_FAILED
  - PH1_RLL_INPUT_SCHEMA_INVALID
  - PH1_RLL_BUDGET_EXCEEDED
  - PH1_RLL_OFFLINE_ONLY_REQUIRED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Offline-only constraint: callable only by `OFFLINE_PIPELINE_ONLY` and never in-turn.
- RL recommendations are governance-gated proposals; runtime activation is out of PH1.RLL scope.
- Tier-3 strict approval requirement applies to RL-driven recommendations.
- Upstream proposal source is PH1.PATTERN (`PATTERN_MINE_OFFLINE` + `PATTERN_PROPOSAL_EMIT`) and must remain deterministic.
- Downstream runtime consumers (including PH1.LEARN and PH1.PAE) may consume RLL-derived artifacts only after governance activation.
- PH1.PAE may consume activated RLL artifacts only in `PAE_POLICY_SCORE_BUILD`; `PAE_ADAPTATION_HINT_EMIT` cannot activate artifacts.
