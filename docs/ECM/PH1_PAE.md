# PH1_PAE ECM (Design vNext)

## Engine Header
- engine_id: PH1.PAE
- role: Deterministic provider-arbitration score build and adaptation hint emission
- placement: TURN_OPTIONAL

## Capability List

### capability_id: PAE_POLICY_SCORE_BUILD
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_signals`, `max_candidates`, `max_scores`, `max_hints`, `max_diagnostics`)
  - bounded adaptation signal vectors (`source`, `route_domain`, `signal_key`, `signal_value_bp`, `confidence_bp`, `governed_artifact_active`, `evidence_ref`)
  - bounded candidate plans (`route_domain`, `provider_slot`, `proposed_mode`, quality/latency/cost/regression metrics, sample size, governed artifact refs, rollback pointer)
  - bounded policy controls (`minimum_sample_size`, `promotion_threshold_bp`, `demotion_failure_threshold`, `consecutive_threshold_failures`)
- output_schema:
  - selected candidate id
  - ordered score entries (deterministic ordering)
  - selected mode (`SHADOW|ASSIST|LEAD`)
  - promotion eligibility + rollback readiness flags
  - boundary flags (`advisory_only=true`, `no_execution_authority=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, GOVERNED_ARTIFACT_REQUIRED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_PAE_OK_POLICY_SCORE_BUILD
  - PH1_PAE_INPUT_SCHEMA_INVALID
  - PH1_PAE_UPSTREAM_INPUT_MISSING
  - PH1_PAE_BUDGET_EXCEEDED
  - PH1_PAE_GOVERNED_ARTIFACT_REQUIRED
  - PH1_PAE_INTERNAL_PIPELINE_ERROR

### capability_id: PAE_ADAPTATION_HINT_EMIT
- input_schema:
  - selected candidate id + selected mode from `PAE_POLICY_SCORE_BUILD`
  - deterministic ordered score entries
  - bounded target-engine list (`PH1.C|PH1.TTS|PH1.CACHE|PH1.MULTI`)
  - `require_no_runtime_authority_drift` flag
- output_schema:
  - validation result (`OK|FAIL`) + bounded diagnostics
  - bounded adaptation hints (`hint_id`, `target_engine`, `route_domain`, `hint_key`, `hint_value`, `priority_bp`, `provenance_ref`)
  - boundary flags (`no_runtime_authority_drift`, `advisory_only=true`, `no_execution_authority=true`)
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_PAE_OK_ADAPTATION_HINT_EMIT
  - PH1_PAE_VALIDATION_FAILED
  - PH1_PAE_INPUT_SCHEMA_INVALID
  - PH1_PAE_UPSTREAM_INPUT_MISSING
  - PH1_PAE_BUDGET_EXCEEDED
  - PH1_PAE_INTERNAL_PIPELINE_ERROR

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Promotion discipline is deterministic: no direct `SHADOW -> LEAD` jump in one step.
- `LEAD` mode requires rollback readiness; missing rollback must fail closed or demote.
- PAE outputs must not include executable directives; they are bounded hints only.

## Related Engine Boundaries
- PH1.FEEDBACK signals must be validated (`FEEDBACK_SIGNAL_EMIT=OK`) before PAE consumption.
- PH1.LISTEN signals must be validated (`LISTEN_SIGNAL_FILTER=OK`) before PAE consumption.
- PH1.LEARN package outputs must be validated (`LEARN_ARTIFACT_PACKAGE_BUILD=OK`) before PAE consumption.
- PH1.RLL artifacts are OFFLINE_ONLY; PAE may consume them only after governance activation.
- PH1.CACHE/PH1.MULTI/PH1.C/PH1.TTS consume PAE output as bounded advisory hints only.
