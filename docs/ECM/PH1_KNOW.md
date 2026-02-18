# PH1_KNOW ECM (Design vNext)

## Engine Header
- engine_id: PH1.KNOW
- role: Tenant dictionary and pronunciation-hint pack composition
- placement: TURN_OPTIONAL

## Capability List

### capability_id: KNOW_DICTIONARY_PACK_BUILD
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_entries`, `max_diagnostics`)
  - required `tenant_id`
  - bounded term entries (`entry_id`, `tenant_id`, `entry_kind`, `source_kind`, `canonical_term`, `normalized_term`, `locale_tag`, optional `pronunciation_hint`, `evidence_ref`)
  - consent/authorization controls (`user_terms_present`, `user_consent_asserted`, `hr_org_authorized`)
  - optional `learn_artifact_ref`
  - hard boundary flags (`tenant_scope_required=true`, `authorized_only_required=true`, `no_cross_tenant_required=true`)
- output_schema:
  - deterministic `pack_id`
  - deterministic ordered entries
  - selected target engines (`PH1.C`, `PH1.SRL`, `PH1.NLP`, optional `PH1.TTS`)
  - boundary flags: `tenant_scoped=true`, `authorized_only=true`, `no_cross_tenant=true`, `advisory_only=true`, `no_execution_authority=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, CONSENT_REQUIRED, UNAUTHORIZED_SOURCE, VALIDATION_FAILED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_KNOW_OK_DICTIONARY_PACK_BUILD
  - PH1_KNOW_INPUT_SCHEMA_INVALID
  - PH1_KNOW_UPSTREAM_INPUT_MISSING
  - PH1_KNOW_BUDGET_EXCEEDED
  - PH1_KNOW_CONSENT_REQUIRED
  - PH1_KNOW_UNAUTHORIZED_SOURCE
  - PH1_KNOW_VALIDATION_FAILED
  - PH1_KNOW_INTERNAL_PIPELINE_ERROR

### capability_id: KNOW_HINT_BUNDLE_SELECT
- input_schema:
  - validated pack inputs (`tenant_id`, `pack_id`, deterministic ordered entries)
  - requested `target_engines`
  - hard boundary flags (`tenant_scope_required=true`, `authorized_only_required=true`, `no_cross_tenant_required=true`)
- output_schema:
  - `validation_status (OK|FAIL)`
  - bounded diagnostics
  - selected target engines
  - preservation flags (`preserved_tenant_scope`, `preserved_authorized_only`, `preserved_no_cross_tenant`)
  - boundary flags: `advisory_only=true`, `no_execution_authority=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, VALIDATION_FAILED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_KNOW_OK_HINT_BUNDLE_SELECT
  - PH1_KNOW_INPUT_SCHEMA_INVALID
  - PH1_KNOW_UPSTREAM_INPUT_MISSING
  - PH1_KNOW_BUDGET_EXCEEDED
  - PH1_KNOW_VALIDATION_FAILED
  - PH1_KNOW_INTERNAL_PIPELINE_ERROR

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Tenant scope is mandatory; cross-tenant entries are forbidden.
- Authorized-only source policy is mandatory; unverified terms fail closed.
- User-provided terms require explicit consent assertion.
- PH1.KNOW output may affect vocabulary/pronunciation rendering only; semantic meaning must remain unchanged.

## Related Engine Boundaries
- PH1.C / PH1.SRL / PH1.NLP consume PH1.KNOW hints only from Selene OS and only after `KNOW_HINT_BUNDLE_SELECT=OK`.
- PH1.TTS consumes only pronunciation-hint subset from PH1.KNOW validated output.
- PH1.PRON remains separate enrollment/lexicon owner; PH1.KNOW does not replace PH1.PRON capability scope.
- PH1.KG may consume PH1.KNOW artifacts only through tenant-scoped Selene OS routing as seed metadata.
