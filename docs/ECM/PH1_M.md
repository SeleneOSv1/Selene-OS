# PH1.M ECM Spec (vNext)

Contract scope note:
- This file is capability contract only.
- Behavioral memory narrative is canonical in `docs/12_MEMORY_ARCHITECTURE.md`.

## Engine Header
- `engine_id`: `PH1.M`
- `purpose`: Deterministic memory retrieval/composition and memory-control persistence for continuity only.
- `data_owned`: memory atoms, suppression rules, emotional threads, memory metrics
- `version`: `vNext`
- `status`: `DONE (design-level)`

## Capability List

### `MEM_HINT_BUNDLE_BUILD`
- `name`: Build bounded memory hint bundle for PH1.C
- `input_schema`: `(tenant_id, user_id, locale, policy_context_ref, now)`
- `output_schema`: `HintBundle`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `MEM_CONTEXT_BUNDLE_BUILD`
- `name`: Build bounded context bundle for PH1.D/PH1.X
- `input_schema`: `(tenant_id, user_id, intent_context_ref, policy_context_ref, now, resume_mode=OFF|AUTO, allow_page_in=bool, topic_hint?)`
- `output_schema`: `ContextBundle`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`
- `policy_note`: If `MEM_SUPPRESSION_SET` has active `DO_NOT_MENTION` for selected `thread_id` or `work_order_id`, suppress surfacing in bundle output.

### `MEM_ATOM_UPSERT`
- `name`: Store or update one memory atom (reason-coded)
- `input_schema`: `(tenant_id, user_id, atom_key, atom_payload, provenance, reason_code, idempotency_key)`
- `output_schema`: `(atom_event_id, atom_key, atom_state)`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `INTERNAL_DB_WRITE`

### `MEM_FORGET`
- `name`: Apply forget request (bounded scope)
- `input_schema`: `(tenant_id, user_id, forget_scope, reason_code, idempotency_key)`
- `output_schema`: `ForgetResult`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `INTERNAL_DB_WRITE`

### `MEM_SUPPRESSION_SET`
- `name`: Apply suppression control (`DO_NOT_MENTION | DO_NOT_REPEAT | DO_NOT_STORE`)
- `input_schema`: `(tenant_id, user_id, target_type=THREAD_ID|WORK_ORDER_ID|TOPIC_KEY, target_id, rule_kind, scope, reason_code, idempotency_key)`
- `output_schema`: `SuppressionRuleState`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `INTERNAL_DB_WRITE`

### `MEM_QUERY_SAFE_SUMMARY`
- `name`: Return bounded safe summary for “what do you remember about me?”
- `input_schema`: `(tenant_id, user_id, exposure_policy, now)`
- `output_schema`: `SafeRecallSummary`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `MEM_EMO_THREAD_UPDATE`
- `name`: Update emotional continuity thread (tone-only)
- `input_schema`: `(tenant_id, user_id, thread_key, thread_delta, reason_code, idempotency_key)`
- `output_schema`: `EmotionalThreadState`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `INTERNAL_DB_WRITE`

### `MEM_METRICS_EMIT`
- `name`: Emit bounded memory quality measurement event
- `input_schema`: `(tenant_id, user_id, metric_payload, reason_code, idempotency_key)`
- `output_schema`: `MetricEventRef`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `INTERNAL_DB_WRITE`

### `MEM_THREAD_DIGEST_UPSERT`
- `name`: Persist deterministic thread digest and bounded pointers on close/update
- `input_schema`: `(tenant_id, user_id, session_id, thread_candidates, evidence_pointers, reason_code, idempotency_key)`
- `output_schema`: `(thread_id, thread_digest_state)`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `INTERNAL_DB_WRITE`

### `MEM_THREAD_RESUME_SELECT`
- `name`: Select best resume thread and produce short resume summary
- `input_schema`: `(tenant_id, user_id, now, identity_ok, resume_mode, allow_auto_resume=bool, allow_suggest=bool, topic_hint?)`
- `output_schema`: `(selected_thread_id?, resume_action=AUTO_LOAD|SUGGEST|NONE, resume_summary_bullets?, reason_code)`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `MEM_GRAPH_UPDATE`
- `name`: Upsert bounded graph nodes/edges from atoms + thread digests
- `input_schema`: `(tenant_id, user_id, source_refs, reason_code, idempotency_key)`
- `output_schema`: `(graph_update_count, graph_update_state)`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `INTERNAL_DB_WRITE`

### `MEM_RETENTION_MODE_SET`
- `name`: Set memory retention preference (`DEFAULT | REMEMBER_EVERYTHING`)
- `input_schema`: `(tenant_id, user_id, memory_retention_mode, reason_code, idempotency_key)`
- `output_schema`: `(memory_retention_mode, effective_at)`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `INTERNAL_DB_WRITE`

## Failure Modes + Reason Codes

- `MEM_POLICY_BLOCKED`
- `MEM_SCOPE_VIOLATION`
- `MEM_IDEMPOTENCY_REPLAY`
- `MEM_APPEND_ONLY_VIOLATION`
- `MEM_BUNDLE_BUDGET_EXCEEDED`
- `MEM_CONFLICT_REQUIRES_CLARIFY`
- `MEM_EXPOSURE_BLOCKED`
- `MEM_INTERNAL_ONLY_LEAK_BLOCKED`
- `MEM_INVALID_RULE_REQUEST`
- `MEM_CLARIFY_REQUIRED`
- `MEM_RESUME_NOT_ALLOWED`
- `MEM_RETENTION_MODE_UPDATED`

## Hard Boundary (Non-Negotiable)

PH1.M capabilities must never:
- grant authority
- execute simulations directly
- execute external side effects
- override authoritative current-state truth
- bypass Access/simulation gates

## Audit Emission Requirements Per Capability

Write capabilities emit PH1.J reason-coded events with bounded payloads:
- `MEM_ATOM_UPSERT` -> `MEM_ATOM_STORED` or `MEM_ATOM_UPDATED`
- `MEM_FORGET` -> `MEM_ATOM_FORGOTTEN`
- `MEM_SUPPRESSION_SET` -> `MEM_SUPPRESSION_RULE_SET`
- `MEM_EMO_THREAD_UPDATE` -> `MEM_EMO_THREAD_UPDATED`
- `MEM_METRICS_EMIT` -> `MEM_METRICS_EMITTED`
- `MEM_THREAD_DIGEST_UPSERT` -> `MEM_THREAD_DIGEST_UPSERTED`
- `MEM_GRAPH_UPDATE` -> `MEM_GRAPH_UPDATED`
- `MEM_RETENTION_MODE_SET` -> `MEM_RETENTION_MODE_SET`

Read capabilities emit audit only when explicit trace mode is enabled:
- `MEM_HINT_BUNDLE_BUILD`
- `MEM_CONTEXT_BUNDLE_BUILD`
- `MEM_QUERY_SAFE_SUMMARY`
- `MEM_THREAD_RESUME_SELECT`

## Acceptance Mapping (Normalized IDs)

- `AT-MEM-01` (Q12-MEM): deterministic hint/context retrieval caps
- `AT-MEM-02` (Q13-MEM): reason-coded atom upsert with no silent overwrite
- `AT-MEM-03` (Q14-MEM): do-not-repeat suppression behavior
- `AT-MEM-04` (Q15-MEM): idempotent multi-device memory sync handoff constraints
- `AT-MEM-05` (Q16-MEM): privacy/exposure controls enforced
- `AT-MEM-06` (Q17-MEM): stable non-authoritative metrics emission
- `AT-MEM-07` (Q18-MEM): emotional continuity remains tone-only

## Blocker

- `none`
