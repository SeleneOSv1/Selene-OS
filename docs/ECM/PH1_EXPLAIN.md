# PH1_EXPLAIN ECM (Design vNext)

## Engine Header
- engine_id: PH1.EXPLAIN
- role: deterministic explanation packet generator (`WHY | WHY_NOT | HOW_KNOW | WHAT_NEXT | WHAT_HAPPENED`)
- placement: TURN_OPTIONAL

## Capability List

### capability_id: EXPLAIN_REASON_RENDER
- input_schema:
  - `Ph1ExplainInput` bounded envelope:
    - explain request (`request_type`, optional short utterance)
    - event context (`primary_reason_code`, bounded related reason list)
    - optional directive context (`clarify`/`confirm`)
    - optional barge-in `verbatim_trigger`
    - policy context (`privacy_mode`, `do_not_disturb`, safety tier)
- output_schema:
  - `Ph1ExplainResponse::Explanation` or `Ph1ExplainResponse::ExplanationRefuse`
  - explanation text bounded to 1–2 sentences
  - reason-safe wording with no provider/internal leakage
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, EX_FORBIDDEN_BY_PRIVACY, EX_INTERNAL
- reason_codes:
  - EX_FORBIDDEN_BY_PRIVACY
  - EX_INTERNAL

### capability_id: EXPLAIN_EVIDENCE_SELECT
- input_schema:
  - `Ph1ExplainInput` with optional `memory_candidate_ref` (`evidence_quote`, optional provenance, `is_sensitive`)
  - request_type `HOW_KNOW` path
- output_schema:
  - if allowed: evidence-backed `Explanation` with optional `evidence_quote`
  - if blocked: `ExplanationRefuse` with privacy reason code
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, EX_FORBIDDEN_BY_PRIVACY, EX_INTERNAL
- reason_codes:
  - EX_FORBIDDEN_BY_PRIVACY
  - EX_INTERNAL

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- PH1.EXPLAIN output is advisory only and never executes or grants authority.
- Explanation text must remain deterministic, bounded, and calm.
- Internal/provider/debug details must never appear in user-facing explanation text.

## Related Engine Boundaries
- `PH1.X`: explanation output is surfaced only when PH1.X chooses to do so.
- `PH1.J`: reason-code context source.
- `PH1.M`: optional evidence source for `HOW_KNOW`, privacy-gated.
- `PH1.C` / `PH1.W` / `PH1.D` / `PH1.E` / `PH1.L` / `PH1.K` / `PH1.TTS` / `PH1.VOICE.ID`: reason-code mapping domains consumed by PH1.EXPLAIN.
