# Selene OS Core Tables DB Wiring Spec

## 1) Engine Header

- `engine_id`: `SELENE_OS_CORE_TABLES`
- `purpose`: Define DB wiring for `work_orders` + `work_order_ledger` and finalize foundational OS-core tables.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

Target tables in this slice:
- `os_core.work_orders_current` (`CURRENT`)
- `os_core.work_order_ledger` (`LEDGER`)
- `os_core.work_order_step_attempts` (`LEDGER-LIKE ATTEMPT HISTORY`)
- `os_core.work_order_leases` (`CURRENT/LEASE STATE`)
- `os_core.user_channel_setup_ledger` (`LEDGER`)
- `os_core.user_channel_setup_current` (`CURRENT`)
- `os_core.identities` (`CURRENT`, already locked under PH1.F)
- `os_core.devices` (`CURRENT`, already locked under PH1.F)
- `os_core.sessions` (`CURRENT`, already locked under PH1.F)
- `conversation.conversation_ledger` (`LEDGER`, already locked under PH1.F)

`work_orders_current` continuity/anti-repeat fields (design-level lock):
- `asked_fields_json` (bounded map: `field_key -> {asked_at, attempts}`)
- `resolved_fields_json` (bounded map of resolved clarifications/conflicts)
- `prompt_dedupe_keys_json` (bounded set of used prompt dedupe keys)
- `external_approval_pending` (bool)
- `external_approval_request_id` (nullable string)
- `external_approval_target_user_id` (nullable string)
- `external_approval_expires_at` (nullable timestamp)

`user_channel_setup_current` global delivery setup fields (design-level lock):
- `tenant_id`
- `user_id`
- `sms_app_setup_complete` (bool; one-time global setup marker)
- `sms_read_permission_ok` (bool)
- `sms_send_permission_ok` (bool)
- `sms_last_validated_at` (timestamp)
- `setup_source` (enum: ONBOARDING | FIRST_SEND_REQUEST | SETTINGS)

## 3) Reads (dependencies)

- `work_order_ledger` replay by `work_order_event_id` for deterministic rebuild.
- `work_orders_current` lookup by `(tenant_id, work_order_id)` for fast status reads.
- `work_orders_current` lookup by `(tenant_id, correlation_id)` for thread binding.
- `user_channel_setup_current` lookup by `(tenant_id, user_id)` before any SMS delivery commit path.

Required indices:
- `ux_work_order_ledger_tenant_work_order_event`
- `ux_work_orders_current_tenant_work_order`
- `ux_work_orders_current_tenant_correlation`
- `ux_user_channel_setup_current_tenant_user`

Scope rules:
- all reads tenant-scoped
- no cross-tenant read path

## 4) Writes (outputs)

- append to `work_order_ledger` via typed input contract
- deterministic update of `work_orders_current` projection on each ledger append
- append to `user_channel_setup_ledger` and project to `user_channel_setup_current` when SMS setup state changes
- idempotency dedupe on `(tenant_id, work_order_id, idempotency_key)`
- WorkOrder status includes `CANCELED`; cancel transitions must append a new ledger event (no in-place mutation).
- cancel path event types: `WORK_ORDER_CANCELED` and/or `STATUS_CHANGED` with required `reason_code`.
- anti-repeat rules:
  - if value already exists in `fields_json` or prefilled context, Selene OS must not ask again
  - prompt key formula: `prompt_dedupe_key = hash(turn_id + field_id + conversation_id)`
  - if `prompt_dedupe_key` already exists and no state change occurred, Selene OS suppresses repeat
  - if conflict appears later, Selene OS asks once, writes resolution to `resolved_fields_json`, and suppresses further repeats
  - explicit correction (`change it`/user-provided update) invalidates prior dedupe key and allows one fresh prompt
- external approval wait rules:
  - when `external_approval_pending=true`, Selene OS emits one wait message and then remains in wait posture
  - no repeated approval nags until approval state changes or timeout expires
- SMS onboarding gate rule:
  - before any SMS delivery request is permitted, Selene OS must require `sms_app_setup_complete=true`
  - if incomplete, Selene OS must fail closed on send path until setup completes
- multilingual pre-parse rule:
  - all voice/text interaction paths apply pre-intent normalization pipeline `PH1.LANG -> PH1.SRL -> PH1.NLP`
  - broken/fragmented/code-switched utterances must be segmented and normalized before intent dispatch

## 4A) Global Policy Gate (PH1.POLICY) — System-Wide

- Before PH1.X emits any clarify/confirm prompt, Selene OS must call `POLICY_PROMPT_DEDUP_DECIDE` and enforce the returned decision.
- PH1.POLICY returns decisions only; Selene OS remains the orchestrator and sole enforcer.
- PH1.POLICY has no execution authority and cannot mutate WorkOrders or runtime state.
- Interruption lifecycle for messages is canonical in PH1.BCAST Section BCAST.MHP; reminders use PH1.REM timing.

## 5) Relations & Keys

Key constraints implemented:
- `work_order_ledger` primary key: `work_order_event_id`
- `work_order_ledger` unique idempotency: `(tenant_id, work_order_id, idempotency_key)` (nullable key)
- `work_orders_current` primary key: `work_order_id`
- `work_orders_current` unique tenant scope: `(tenant_id, work_order_id)`
- `work_orders_current` unique correlation scope: `(tenant_id, correlation_id)`
- `user_channel_setup_ledger` unique idempotency key: `(tenant_id, user_id, idempotency_key)`
- `user_channel_setup_current` unique scope: `(tenant_id, user_id)`
- `work_orders_current.last_event_id` FK -> `work_order_ledger.work_order_event_id`
- `work_order_step_attempts.work_order_id` FK -> `work_orders_current.work_order_id`
- `work_order_leases.work_order_id` FK -> `work_orders_current.work_order_id`
- `user_channel_setup_current.last_event_id` FK -> `user_channel_setup_ledger.channel_setup_event_id`

State constraints:
- `work_order_ledger` is append-only
- `work_orders_current` is rebuildable from `work_order_ledger`
- `user_channel_setup_ledger` is append-only
- `user_channel_setup_current` is rebuildable from `user_channel_setup_ledger`
- pending continuity statuses include `{DRAFT, CLARIFY, CONFIRM}` for bounded resume selectors.
- pending continuity selectors are filtered by recency window and deterministic ordering (most recent first unless policy overrides).

## 6) Audit Emissions (PH1.J)

- WorkOrder table slice relies on PH1.J for audit envelope; this row locks storage wiring.
- Required correlation keys remain: `tenant_id`, `work_order_id`, `correlation_id`, `turn_id`.

### related engine boundary: `PH1.SCHED`
- Scheduler outputs (`RETRY_AT | FAIL | WAIT`) must be consumed via Selene OS orchestration and reflected through append-only WorkOrder ledger transitions.
- `WAIT` decisions must not advance attempt index in `work_orders_current` projection.
- `SCHED_NEXT_ACTION_DRAFT` simulation remains deterministic and contract-scoped; no hidden retries are allowed.

### related engine boundary: `PH1.LEASE`
- WorkOrder step execution paths must require an active lease before advancing lease-gated step events.
- At most one active lease may exist per `(tenant_id, work_order_id)` in `work_order_leases`.
- Renew/release paths require lease-token ownership checks; expired takeover must resume from persisted ledger state only.

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-OS-CORE-DB-01` tenant isolation enforced
  - `at_os_core_db_01_tenant_isolation_enforced`
- `AT-OS-CORE-DB-02` append-only enforcement
  - `at_os_core_db_02_append_only_enforced`
- `AT-OS-CORE-DB-03` idempotency dedupe works
  - `at_os_core_db_03_idempotency_dedupe_works`
- `AT-OS-CORE-DB-04` rebuild current from ledger
  - `at_os_core_db_04_rebuild_current_from_ledger`
- `AT-OS-20` never-ask-twice enforced through `asked_fields_json` + `prompt_dedupe_keys_json`
  - `at_os_20_never_ask_twice_enforced`
- `AT-OS-21` external approval wait state does not spam user and resumes only after approval response
  - `at_os_21_external_approval_wait_no_spam`
- `AT-OS-22` SMS setup gate blocks SMS send until `sms_app_setup_complete=true`
  - `at_os_22_sms_setup_required_before_sms_send`
- `AT-OS-23` global prompt dedupe prevents repeated asks after answer persisted
  - `at_os_23_prompt_dedupe_global`
- `AT-OS-24` mixed-language turn is segmented and normalized before NLP intent parse
  - `at_os_24_segment_then_parse`

Implementation references:
- kernel contracts: `crates/selene_kernel_contracts/src/ph1work.rs`
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- migration: `crates/selene_storage/migrations/0002_work_orders_core.sql`
- typed repo: `crates/selene_storage/src/repo.rs`
- tests: `crates/selene_storage/tests/os_core/db_wiring.rs`
