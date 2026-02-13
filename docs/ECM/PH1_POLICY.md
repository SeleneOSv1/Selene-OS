# PH1.POLICY ECM (Design vNext)

## Engine Header
- `engine_id`: `PH1.POLICY`
- `layer`: `Control (Global Policy)`
- `authority`: `Authoritative (policy decisions only)`
- `allowed_callers`: `SELENE_OS_ONLY`
- `placement`: `ALWAYS_ON`

## Capability List

### capability_id: POLICY_PROMPT_DEDUP_DECIDE
- `input_schema`:
  - `tenant_id`
  - `work_order_id`
  - `now`
  - `required_fields` (list)
  - `fields_json` (current known fields)
  - `asked_fields_json` (field -> attempts + last_asked_at)
  - `prompt_dedupe_keys_json` (set)
  - `authoritative_prefill_fields_json` (optional DB/draft-derived prefill)
- `output_schema`:
  - `decision`: `ASK | SKIP | ASK_DIFFERENT_FIELD | STOP`
  - `field_to_ask` (optional)
  - `prompt_dedupe_key` (optional)
  - `reason_code`: `POLICY_FIELD_ALREADY_KNOWN | POLICY_ALREADY_ASKED | POLICY_CONFLICT_REQUIRES_ONE_ASK | POLICY_NEXT_FIELD`
- `side_effects`: `NONE`
- `failure_modes`:
  - `POLICY_INPUT_SCHEMA_INVALID`
  - `POLICY_REQUIRED_FIELDS_EMPTY`
  - `POLICY_WORK_ORDER_SCOPE_INVALID`
- `reason_codes`:
  - `POLICY_FIELD_ALREADY_KNOWN`
  - `POLICY_ALREADY_ASKED`
  - `POLICY_CONFLICT_REQUIRES_ONE_ASK`
  - `POLICY_NEXT_FIELD`
  - `POLICY_INPUT_SCHEMA_INVALID`
  - `POLICY_REQUIRED_FIELDS_EMPTY`
  - `POLICY_WORK_ORDER_SCOPE_INVALID`
- `rules`:
  - if field exists in `authoritative_prefill_fields_json` or `fields_json` -> `SKIP`
  - if `asked_fields_json` indicates already asked with no state change -> `ASK_DIFFERENT_FIELD` or `STOP`
  - never return more than one `field_to_ask`
  - used only to decide whether/what single field to ask next; no interruption decisions

### capability_id: POLICY_RULESET_GET_ACTIVE
- `input_schema`:
  - `tenant_id`
  - `user_id` (optional)
  - `now`
- `output_schema`:
  - `policy_ruleset_version`
  - `ruleset_hash`
  - `enabled_rules` (bounded list of rule keys)
  - `reason_code`
- `side_effects`: `NONE`
- `failure_modes`:
  - `POLICY_INPUT_SCHEMA_INVALID`
  - `POLICY_RULESET_NOT_FOUND`
- `reason_codes`:
  - `POLICY_RULESET_OK`
  - `POLICY_RULESET_NOT_FOUND`
  - `POLICY_INPUT_SCHEMA_INVALID`

## Constraints
- engines never call engines directly; Selene OS orchestrates
- PH1.POLICY emits decisions only and has no execution authority
- policy decisions never bypass Access + Simulation gates
