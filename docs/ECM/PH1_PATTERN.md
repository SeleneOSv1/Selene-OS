# PH1_PATTERN ECM (Design vNext)

## Engine Header
- engine_id: PH1.PATTERN
- role: Offline pattern mining and proposal emission for governed artifact candidates
- placement: OFFLINE_ONLY

## Capability List

### capability_id: PATTERN_MINE_OFFLINE
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_signals`, `max_proposals`, `offline_pipeline_only=true`)
  - bounded historical signal list (`signal_id`, `source_engine`, `metric_key`, `metric_value_bp`, `occurrence_count`, `evidence_ref`)
  - bounded analysis window
- output_schema:
  - one selected proposal id
  - deterministic ordered proposal list (`proposal_id`, target, rank, confidence, approval_tier, evidence_ref)
  - boundary flags: `offline_only=true`, `no_execution_authority=true`
- allowed_callers: OFFLINE_PIPELINE_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, OFFLINE_ONLY_REQUIRED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_PATTERN_OK_MINE_OFFLINE
  - PH1_PATTERN_INPUT_SCHEMA_INVALID
  - PH1_PATTERN_UPSTREAM_INPUT_MISSING
  - PH1_PATTERN_BUDGET_EXCEEDED
  - PH1_PATTERN_OFFLINE_ONLY_REQUIRED
  - PH1_PATTERN_INTERNAL_PIPELINE_ERROR

### capability_id: PATTERN_PROPOSAL_EMIT
- input_schema:
  - selected proposal id + ordered proposal list from `PATTERN_MINE_OFFLINE`
  - deterministic envelope constraints (`offline_pipeline_only=true`)
- output_schema:
  - validation_result (`OK|FAIL`)
  - bounded diagnostics
  - boundary flags: `offline_only=true`, `no_execution_authority=true`
- allowed_callers: OFFLINE_PIPELINE_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, BUDGET_EXCEEDED, OFFLINE_ONLY_REQUIRED
- reason_codes:
  - PH1_PATTERN_OK_PROPOSAL_EMIT
  - PH1_PATTERN_VALIDATION_FAILED
  - PH1_PATTERN_INPUT_SCHEMA_INVALID
  - PH1_PATTERN_BUDGET_EXCEEDED
  - PH1_PATTERN_OFFLINE_ONLY_REQUIRED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Offline-only constraint: callable only by `OFFLINE_PIPELINE_ONLY` and never in-turn.
- Proposal output is non-activating; governance approval is required before any downstream runtime consumption.

## Related Engine Boundary (`PH1.RLL`)
- PH1.PATTERN output is candidate input for PH1.RLL ranking only.
- PH1.PATTERN does not perform RL ranking, recommendation finalization, or runtime activation.

## Related Engine Boundary (`PH1.LEARN`)
- PH1.LEARN may consume only governance-approved artifacts derived from PH1.PATTERN/PH1.RLL outputs through Selene OS routing.
- PH1.PATTERN never emits runtime-active artifacts directly to PH1.LEARN or any turn-time engine.
