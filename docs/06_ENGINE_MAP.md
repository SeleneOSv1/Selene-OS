# Engine Map (Summary + Navigation)

Purpose:
- Provide a fast orientation view of runtime flow.
- Point to canonical docs for authoritative details.

This file is non-canonical by design.

## Canonical References

- Design truth and ownership rules: `docs/00_DESIGN_TRUTH_OPTION_B.md`
- Engine registry (authoritative engine list): `docs/07_ENGINE_REGISTRY.md`
- Simulation inventory (authoritative): `docs/08_SIMULATION_CATALOG.md`
- Blueprint registry (authoritative mapping): `docs/09_BLUEPRINT_REGISTRY.md`
- DB ownership summary (authoritative): `docs/10_DB_OWNERSHIP_MATRIX.md`
- Design lock status (authoritative): `docs/11_DESIGN_LOCK_SEQUENCE.md`
- Coverage/status matrix (authoritative): `docs/COVERAGE_MATRIX.md`
- Detailed engine contracts: `docs/DB_WIRING/*.md` and `docs/ECM/*.md`
- PH1.M vNext memory architecture: `docs/12_MEMORY_ARCHITECTURE.md`

## Phase A Navigation (Design Completion)

- `PH1.F`: `docs/DB_WIRING/PH1_F.md` + `docs/ECM/PH1_F.md`
- `PH1.J`: `docs/DB_WIRING/PH1_J.md` + `docs/ECM/PH1_J.md`
- `SELENE_OS_CORE_TABLES`: `docs/DB_WIRING/SELENE_OS_CORE_TABLES.md` + `docs/ECM/SELENE_OS_CORE_TABLES.md`
- Planned-but-not-yet-finalized engines are tracked only in `docs/07_ENGINE_REGISTRY.md` and `docs/COVERAGE_MATRIX.md`.

## Phase B Navigation (Identity/Access)

- `PH1.L`: `docs/DB_WIRING/PH1_L.md` + `docs/ECM/PH1_L.md`
- `PH1.VOICE.ID`: `docs/DB_WIRING/PH1_VOICE_ID.md` + `docs/ECM/PH1_VOICE_ID.md`
- `PH1.ACCESS.001_PH2.ACCESS.002`: `docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md` + `docs/ECM/PH1_ACCESS_001_PH2_ACCESS_002.md`

## Global Policy Navigation

- `PH1.POLICY`: `docs/DB_WIRING/PH1_POLICY.md` + `docs/ECM/PH1_POLICY.md`

## Phase E Navigation (Onboarding/Delivery Controls)

- `PH1.BCAST`: `docs/DB_WIRING/PH1_BCAST.md` + `docs/ECM/PH1_BCAST.md` (active implementation ids: `PH1.BCAST.001`)
- `PH1.DELIVERY`: `docs/DB_WIRING/PH1_DELIVERY.md` + `docs/ECM/PH1_DELIVERY.md`
- `PH1.LINK`: `docs/DB_WIRING/PH1_LINK.md` + `docs/ECM/PH1_LINK.md`
- `PH1.ONB`: `docs/DB_WIRING/PH1_ONB.md` + `docs/ECM/PH1_ONB.md`
- `PH1.POSITION`: `docs/DB_WIRING/PH1_POSITION.md` + `docs/ECM/PH1_POSITION.md`
- `PH1.REM`: `docs/DB_WIRING/PH1_REM.md` + `docs/ECM/PH1_REM.md`
- `PH1.CAPREQ`: `docs/DB_WIRING/PH1_CAPREQ.md` + `docs/ECM/PH1_CAPREQ.md` (family lock active implementation ids: `PH1.CAPREQ.001`)

## Phase C Navigation (Perception + Understanding + Orchestration)

- `PH1.K`: `docs/DB_WIRING/PH1_K.md` + `docs/ECM/PH1_K.md`
- `PH1.W`: `docs/DB_WIRING/PH1_W.md` + `docs/ECM/PH1_W.md`
- `PH1.C`: `docs/DB_WIRING/PH1_C.md` + `docs/ECM/PH1_C.md`
- `PH1.SRL`: `docs/DB_WIRING/PH1_SRL.md` + `docs/ECM/PH1_SRL.md`
- `PH1.NLP`: `docs/DB_WIRING/PH1_NLP.md` + `docs/ECM/PH1_NLP.md`
- `PH1.D`: `docs/DB_WIRING/PH1_D.md` + `docs/ECM/PH1_D.md`
- `PH1.X`: `docs/DB_WIRING/PH1_X.md` + `docs/ECM/PH1_X.md`

## Phase F Navigation (Memory/Learning)

- `PH1.M (vNext++)`: `docs/DB_WIRING/PH1_M.md` + `docs/ECM/PH1_M.md` + `docs/12_MEMORY_ARCHITECTURE.md` (threads + graph + paging + tiered auto-resume + pending WorkOrder continuity + retention mode)
- `PH1.FEEDBACK`: `docs/DB_WIRING/PH1_FEEDBACK.md` + `docs/ECM/PH1_FEEDBACK.md` (runtime feedback signal capture/emission)
- `PH1.LEARN`: `docs/DB_WIRING/PH1_LEARN.md` + `docs/ECM/PH1_LEARN.md` (runtime adaptation package builder)
- `PH1.EMO.GUIDE`: `docs/DB_WIRING/PH1_EMO_GUIDE.md` + `docs/ECM/PH1_EMO_GUIDE.md` (tone-policy assist only)
- `PH1.EMO.CORE`: `docs/DB_WIRING/PH1_EMO_CORE.md` + `docs/ECM/PH1_EMO_CORE.md` (implementation-level emotional snapshot/profile/tone core)
- `PH1.PERSONA`: `docs/DB_WIRING/PH1_PERSONA.md` + `docs/ECM/PH1_PERSONA.md` (identity-verified persona/tone delivery hints)
- `PH1.KNOW`: `docs/DB_WIRING/PH1_KNOW.md` + `docs/ECM/PH1_KNOW.md` (runtime tenant dictionary/pronunciation pack composition)
- storage grouping pointer (non-runtime): `PH1.LEARN_FEEDBACK_KNOW` persists row-25 append-only semantics only in `docs/DB_WIRING/PH1_LEARN_FEEDBACK_KNOW.md` + `docs/ECM/PH1_LEARN_FEEDBACK_KNOW.md`
- Memory workflow blueprints:
  - `docs/BLUEPRINTS/MEMORY_QUERY.md`
  - `docs/BLUEPRINTS/MEMORY_FORGET_REQUEST.md`
  - `docs/BLUEPRINTS/MEMORY_REMEMBER_REQUEST.md`

## Enterprise Support Navigation

- `PH1.TENANT`: `docs/DB_WIRING/PH1_TENANT.md` + `docs/ECM/PH1_TENANT.md`
- `PH1.GOV`: `docs/DB_WIRING/PH1_GOV.md` + `docs/ECM/PH1_GOV.md`
- `PH1.QUOTA`: `docs/DB_WIRING/PH1_QUOTA.md` + `docs/ECM/PH1_QUOTA.md`
- `PH1.WORK`: `docs/DB_WIRING/PH1_WORK.md` + `docs/ECM/PH1_WORK.md`
- `PH1.LEASE`: `docs/DB_WIRING/PH1_LEASE.md` + `docs/ECM/PH1_LEASE.md`
- `PH1.OS`: `docs/DB_WIRING/PH1_OS.md` + `docs/ECM/PH1_OS.md`
- `PH1.HEALTH`: `docs/DB_WIRING/PH1_HEALTH.md` + `docs/ECM/PH1_HEALTH.md` (display-only health reporting dashboard)
- `PH1.SCHED`: `docs/DB_WIRING/PH1_SCHED.md` + `docs/ECM/PH1_SCHED.md`
- `PH1.EXPORT`: `docs/DB_WIRING/PH1_EXPORT.md` + `docs/ECM/PH1_EXPORT.md`
- `PH1.KMS`: `docs/DB_WIRING/PH1_KMS.md` + `docs/ECM/PH1_KMS.md`

## Runtime Flow (High-Level)

```text
Voice: PH1.K -> PH1.W -> PH1.VOICE.ID -> PH1.C -> PH1.SRL -> PH1.NLP -> PH1.X -> PH1.WRITE -> PH1.TTS
Text:  UI -> transcript_ok-equivalent -> PH1.NLP -> PH1.X -> PH1.WRITE -> UI
```

Execution law:
- Engines never call engines directly.
- Selene OS orchestrates all cross-engine sequencing.
- Side effects require Access + Simulation (`No Simulation -> No Execution`).

## Turn Wiring Graph (Authoritative)

Always-on turn (voice):

```text
PH1.K -> PH1.W -> PH1.VOICE.ID -> PH1.C -> PH1.SRL -> PH1.NLP -> PH1.X
```

Pre-intent multilingual normalization (voice and text paths):

```text
PH1.LANG -> PH1.SRL -> PH1.NLP
```

This pipeline normalizes broken/fragmented/code-switched utterances before intent routing.

Assists (called by Selene OS, never engine-to-engine):
- PH1.ENDPOINT assists capture boundaries (inputs from PH1.K/PH1.C; outputs one selected endpoint hint + ordered boundary hints to Selene OS, which passes bounded metadata to PH1.C/PH1.K only after validation).
- PH1.LANG assists PH1.C/PH1.SRL/PH1.NLP.
- PH1.SRL executes `SRL_FRAME_BUILD -> SRL_ARGUMENT_NORMALIZE`; only `validation_status=OK` SRL bundles may be forwarded by Selene OS to PH1.NLP.
- Non-linear/tangled utterance unraveling is handled inside PH1.NLP deterministic parsing/clarify flow.
- Clarify owner lock: only `PH1.NLP` may own clarify decisions (`clarify_owner_engine_id=PH1.NLP` when `clarify_required=true`); no assist engine may become clarify owner.
- PH1.PRON builds tenant/user pronunciation packs for PH1.TTS and robustness hints for PH1.VOICE.ID/PH1.W; user-scoped packs require explicit consent.
- Salience/focus ranking is handled inside PH1.NLP/PH1.CONTEXT deterministic assist flow.
- PH1.PRUNE assists PH1.X when multiple missing fields exist by selecting exactly one deterministic clarify target from PH1.NLP `required_fields_missing` and failing closed on order drift.
- PH1.DIAG runs before PH1.X finalizes a move; it validates intent/field/confirmation/privacy/memory consistency and may only block/clarify (never execute).
- Optional-assist policy bounds (fail-closed):
  - `PH1.PRUNE` may be requested only when `clarify_required=true`.
  - `PH1.DIAG` may be requested only when at least one of `clarify_required`, `confirm_required`, `tool_requested`, or `simulation_requested` is true.
- PH1.EXPLAIN executes `EXPLAIN_REASON_RENDER -> EXPLAIN_EVIDENCE_SELECT` only on explicit explain triggers (`why/how/what happened`) and remains advisory/non-executing.
- PH1.SEARCH/PH1.PREFETCH assist planning and evidence interpretation; PH1.E executes tools only.
- PH1.SEARCH query plans may route to PH1.E first, then PH1.SEARCH evidence interpretation/ranking before PH1.CONTEXT/PH1.X response shaping.
- PH1.COST owns unified turn-policy pacing metadata: urgency tagging (NORMAL/URGENT), delivery preference hints, and per-user/day STT/LLM/TTS/TOOL guardrails; outputs tune pacing/retries/route tier/response length only and never trigger execution or change truth.
- PH1.PREFETCH may emit read-only prefetch candidates (bounded TTL + deterministic idempotency keys) for PH1.E scheduler/cache warmer paths; PH1.PREFETCH never executes tools.
- PH1.E tool evidence may route through PH1.SEARCH evidence interpretation/ranking before PH1.CONTEXT/PH1.X; source-ranked order + URL provenance must be preserved.
- PH1.LISTEN executes `LISTEN_SIGNAL_COLLECT -> LISTEN_SIGNAL_FILTER`; only `validation_status=OK` listen bundles may be forwarded by Selene OS to PH1.ENDPOINT/PH1.C (capture + endpoint tuning) and to PH1.PAE/PH1.MULTI (advisory context only).
- PH1.EMO.GUIDE executes `EMO_GUIDE_PROFILE_BUILD -> EMO_GUIDE_PROFILE_VALIDATE`; only `validation_status=OK` tone-policy bundles may be forwarded by Selene OS to PH1.X/PH1.TTS as advisory style hints, with `tone_only=true` and `no_meaning_drift=true`.
- PH1.EMO.CORE executes `PH1EMO_CLASSIFY_PROFILE_COMMIT_ROW`, `PH1EMO_REEVALUATE_PROFILE_COMMIT_ROW`, `PH1EMO_PRIVACY_COMMAND_COMMIT_ROW`, `PH1EMO_TONE_GUIDANCE_DRAFT_ROW`, `PH1EMO_SNAPSHOT_CAPTURE_COMMIT_ROW`, `PH1EMO_AUDIT_EVENT_COMMIT_ROW`; outputs remain advisory (`tone_only=true`) and are fail-closed on simulation/capability drift.
- PH1.PERSONA executes `PERSONA_PROFILE_BUILD -> PERSONA_PROFILE_VALIDATE`; only identity-verified `validation_status=OK` persona bundles may be forwarded by Selene OS to PH1.X/PH1.TTS as advisory style/delivery hints and to PH1.CACHE as advisory `persona_profile_ref`.
- PH1.FEEDBACK captures bounded post-turn correction/confidence events and emits validated advisory signals only; Selene OS forwards these signals to PH1.LEARN and PH1.PAE without changing in-turn execution.
- PH1.LEARN executes `LEARN_SIGNAL_AGGREGATE -> LEARN_ARTIFACT_PACKAGE_BUILD`; only `validation_status=OK` governed package outputs may be forwarded as advisory artifacts to PH1.KNOW/PH1.PAE/PH1.CACHE/PH1.PRUNE/PH1.SEARCH/PH1.LISTEN.
- PH1.PAE executes `PAE_POLICY_SCORE_BUILD -> PAE_ADAPTATION_HINT_EMIT`; only `validation_status=OK` adaptation bundles may be forwarded by Selene OS to PH1.C/PH1.TTS/PH1.CACHE/PH1.MULTI as advisory route hints.
- PH1.CACHE executes `CACHE_HINT_SNAPSHOT_READ -> CACHE_HINT_SNAPSHOT_REFRESH`; only `validation_status=OK` governed-artifact snapshots may be forwarded as advisory hints to PH1.PREFETCH/PH1.CONTEXT.
- PH1.KNOW executes `KNOW_DICTIONARY_PACK_BUILD -> KNOW_HINT_BUNDLE_SELECT`; only `validation_status=OK` tenant-scoped authorized-only bundles may be forwarded as advisory hints to PH1.C/PH1.SRL/PH1.NLP and pronunciation-hint subset to PH1.TTS.
- PH1.DOC/PH1.VISION are invoked only when user documents/images are provided; PH1.VISION is opt-in only and must remain visible-content-only (no unseen inference).
- PH1.SUMMARY is invoked only when bounded evidence summaries are requested; output must remain evidence-backed and citation-valid.
- PH1.DOC outputs may route through PH1.SUMMARY before PH1.CONTEXT/PH1.NLP; PH1.VISION outputs feed PH1.MULTI + PH1.CONTEXT.
- PH1.MULTI executes `MULTI_BUNDLE_COMPOSE -> MULTI_SIGNAL_ALIGN`; only validated (`OK`) privacy-scoped multimodal bundles may be forwarded to PH1.CONTEXT.
- PH1.KG executes `KG_ENTITY_LINK -> KG_FACT_BUNDLE_SELECT`; only validated (`OK`) tenant-scoped, evidence-backed fact bundles may be forwarded to PH1.CONTEXT/PH1.NLP as advisory grounding hints.
- PH1.CONTEXT executes `CONTEXT_BUNDLE_BUILD -> CONTEXT_BUNDLE_TRIM`; only `validation_status=OK` context bundles may be forwarded to PH1.NLP/PH1.X.
- Policy-required human review routing is owned by governance/access escalation flow (`PH1.GOV` + `PH1.ACCESS` + simulation proofs); no standalone review assist runtime path.
- Before prompting: `Selene OS -> PH1.POLICY (prompt dedupe) -> PH1.X`
- Prompt dedupe is enforced via PH1.POLICY before PH1.X clarify.
- Message interruption lifecycle is PH1.BCAST (BCAST.MHP).

Broadcast/delivery side-effect wiring (Selene OS orchestrated):
- Link generation: `PH1.LINK`; link delivery: `PH1.BCAST` + `PH1.DELIVERY` (`LINK_DELIVER_INVITE`).
- Access gate returns `ALLOW | DENY | ESCALATE` before any delivery commit path.
- If delivery method is SMS and `sms_app_setup_complete=false`, Selene OS refuses the send path with setup-required reason-codes until setup is complete.
- For approved delivery paths: Selene OS runs simulation commit steps, then calls PH1.BCAST lifecycle capabilities.
- For per-recipient provider sends: Selene OS calls PH1.DELIVERY inside COMMIT simulation context and feeds resulting proof/status back into PH1.BCAST lifecycle state.
- PH1.BCAST uses BCAST.MHP phone-first lifecycle for single-recipient messages; follow-ups occur only after WAITING timeout or URGENT classification.
- AP escalation wiring:
  - Access returns `ESCALATE` (`AP_APPROVAL_REQUIRED`) ->
  - Selene OS opens PH1.BCAST approval flow ->
  - Selene OS applies override through Access simulations ->
  - Selene OS re-checks access before any execution.

Onboarding schema ownership and backfill wiring (Selene OS orchestrated):
- `PH1.POSITION` is schema owner: requirements-schema create/update/activate and rollout scope decision are position-owned lifecycle writes.
- `PH1.ONB` is schema executor: ONB runs pinned active schema only (one-question discipline), never mutates schema definitions.
- `PH1.LINK` captures selector hints in draft/token lifecycle to seed ONB schema selection deterministically; LINK does not own schema definitions.
- For rollout scope `CurrentAndNew`, ONB backfill campaign state is ONB-owned; external delivery/reminder handoff runs through `PH1.BCAST` + `PH1.REM`.
- For governed changes, Selene OS enforces access/approval gates before commit paths (`PH1.ACCESS` and CAPREQ-managed capability request lifecycle where policy requires).

Learning wiring (not in-turn execution path):
- PH1.LISTEN/PH1.FEEDBACK/PH1.PAE/PH1.CACHE/PH1.KNOW/PH1.MULTI/PH1.CONTEXT feed hints and policy snapshots only (no execution path).
- PH1.EMO.GUIDE feeds tone-policy hints only; it cannot mutate intent truth, permissions, confirmations, or execution sequencing.
- PH1.PERSONA feeds identity-verified style/delivery hints only; it cannot mutate intent truth, permissions, confirmations, or execution sequencing.
- PH1.LISTEN may affect capture/delivery-mode hints only; it must not mutate meaning.
- PH1.MULTI remains evidence-backed + privacy-scoped advisory output only.
- PH1.LEARN consumes user corrections/feedback and emits governed adaptation package outputs.
- PH1.PAE consumes governed learning signals and emits ranking hints for future PH1.C/PH1.TTS/PH1.CACHE/PH1.MULTI routing.
- PH1.PATTERN/PH1.RLL produce OFFLINE artifact proposals only.
- Offline learning chain: `PH1.PATTERN -> PH1.RLL -> governed artifact queue`; only governance-approved ACTIVE artifacts may influence PH1.PAE/PH1.PRUNE/PH1.CACHE/PH1.PREFETCH/PH1.CONTEXT.
- PH1.PATTERN executes `PATTERN_MINE_OFFLINE -> PATTERN_PROPOSAL_EMIT`; PH1.RLL executes `RLL_POLICY_RANK_OFFLINE -> RLL_ARTIFACT_RECOMMEND`; each stage is fail-closed before queue handoff.

Enterprise support wiring (OS-internal):
- PH1.TENANT executes `TENANT_POLICY_EVALUATE -> TENANT_DECISION_COMPUTE`; only deterministic tenant context outputs may be consumed by Selene OS, and unknown identity without signed-in user must return `NEEDS_CLARIFY` (no tenant guessing).
- PH1.GOV executes `GOV_POLICY_EVALUATE -> GOV_DECISION_COMPUTE`; only deterministic governance decisions (`ALLOWED | BLOCKED`) may be consumed by Selene OS, and outputs must remain non-executing (`no_execution_authority=true`).
- PH1.QUOTA executes `QUOTA_POLICY_EVALUATE -> QUOTA_DECISION_COMPUTE`; only deterministic lane decisions (`ALLOW | WAIT | REFUSE`) may be consumed by Selene OS, and outputs must preserve `no_authority_grant=true` and `no_gate_order_change=true`.
- PH1.WORK executes `WORK_POLICY_EVALUATE -> WORK_DECISION_COMPUTE`; only deterministic append/no-op decisions (`OK | REFUSED | FAIL`) may be consumed by Selene OS, and outputs must preserve append-only, idempotency, and tenant-scope invariants.
- PH1.LEASE executes `LEASE_POLICY_EVALUATE -> LEASE_DECISION_COMPUTE`; Selene OS consumes only deterministic `LeaseGranted | LeaseDenied` decisions, and renew/release paths must enforce lease-token ownership with one-active-lease invariants.
- PH1.OS executes `OS_POLICY_EVALUATE -> OS_DECISION_COMPUTE`; Selene OS consumes exactly one deterministic next move and dispatch legality while enforcing one-turn-one-move and `No Simulation -> No Execution`.
- PH1.OS top-level orchestration is canonicalized in one runtime slice: voice and text turns must pass canonical ALWAYS_ON sequence checks, TURN_OPTIONAL invocation order is computed from one control point, and unsupported sequence/order drift fails closed.
- PH1.OS enforces explicit per-turn optional budget contract fields (`optional_invocations_requested/budget/skipped` + `optional_latency_budget_ms/estimated_ms`); malformed/breached budget posture fails closed at gate U3.
- PH1.OS also runs machine-only optional utility scoring (`GATE-U4/GATE-U5`) from outcome-utilization entries, with deterministic actions (`KEEP | DEGRADE | DISABLE_CANDIDATE`) and optional-engine tier grouping (`STRICT | BALANCED | RICH`).
- PH1.OS enforces runtime-boundary guardrails: OFFLINE_ONLY engines (`PH1.PATTERN`, `PH1.RLL`) and control-plane engines (`PH1.GOV`, `PH1.EXPORT`, `PH1.KMS`) are rejected fail-closed if they appear in live turn runtime paths.
- PH1.SCHED executes `SCHED_POLICY_EVALUATE -> SCHED_DECISION_COMPUTE`; Selene OS consumes only deterministic `RETRY_AT | FAIL | WAIT` decisions, and `WAIT` must not advance attempt index.
- PH1.HEALTH executes display-only read capabilities (`HEALTH_SNAPSHOT_READ`, `HEALTH_ISSUE_TIMELINE_READ`, `HEALTH_UNRESOLVED_SUMMARY_READ`) to project issue history and unresolved/escalated visibility for app UI; no remediation execution and no authority mutation.
- PH1.KMS executes `KMS_ACCESS_EVALUATE -> KMS_MATERIAL_ISSUE`; only `validation_status=OK` material bundles may be consumed by Selene OS, and all outputs remain opaque refs (no secret values).
- PH1.EXPORT executes `EXPORT_ACCESS_EVALUATE -> EXPORT_ARTIFACT_BUILD`; only `status=OK` export bundles may be consumed by Selene OS, with deterministic redaction + tamper-evident hash output and raw-audio exclusion by default.

Wiring class declaration:
- ALWAYS_ON: `PH1.K`, `PH1.W`, `PH1.VOICE.ID`, `PH1.C`, `PH1.SRL`, `PH1.NLP`, `PH1.CONTEXT`, `PH1.POLICY`, `PH1.X`
- TURN_OPTIONAL: `PH1.ENDPOINT`, `PH1.LANG`, `PH1.PRON`, `PH1.DOC`, `PH1.SUMMARY`, `PH1.VISION`, `PH1.PRUNE`, `PH1.DIAG`, `PH1.SEARCH`, `PH1.COST`, `PH1.PREFETCH`, `PH1.EXPLAIN`, `PH1.LISTEN`, `PH1.EMO.GUIDE`, `PH1.EMO.CORE`, `PH1.PERSONA`, `PH1.FEEDBACK`, `PH1.LEARN`, `PH1.PAE`, `PH1.CACHE`, `PH1.KNOW`, `PH1.MULTI`, `PH1.KG`, `PH1.BCAST`, `PH1.DELIVERY`
- OFFLINE_ONLY: `PH1.PATTERN`, `PH1.RLL`
- ENTERPRISE_SUPPORT: `PH1.TENANT`, `PH1.GOV`, `PH1.QUOTA`, `PH1.WORK`, `PH1.LEASE`, `PH1.OS`, `PH1.HEALTH`, `PH1.SCHED`, `PH1.KMS`, `PH1.EXPORT`

## Design Hygiene

- Do not place simulation inventories in this file.
- Do not place blueprint records in this file.
- Do not place lock status tables in this file.
- Keep this document short; link out to canonical sources above.
