# Agent Orchestrator / Simulation Finder Core (Build Plan)

Status: design-only. No implementation in this packet.

## 1) Purpose and hard law

This module is Selene's deterministic simulation finder for messy user language.

Hard law:
- It never executes side effects.
- It only returns one of four outcomes:
  - `SimulationMatchPacket` (exact simulation candidate)
  - `ClarifyPacket` (exactly one missing-field question)
  - `RefusePacket` (fail closed with reason code)
  - `MissingSimulationPacket` (forward to Dev Intake)

Core guarantee:
- After clarification, the only legitimate unresolved terminal path is `NO_MATCHING_SIMULATION_EXISTS`.

### 1.1 Canonical terminal outcomes (single source of truth)

The Finder terminal packet set is exactly:
- `SimulationMatchPacket`
- `ClarifyPacket`
- `RefusePacket`
- `MissingSimulationPacket`

No additional terminal type is allowed. No section in this document may redefine this set.

## 2) Repo-grounded reuse (no duplication)

The core should be an extension of existing PH1 flow, not a new autonomous engine.

### Reused runtime contracts and wiring

- Understanding contract and evidence fields:
  - `crates/selene_kernel_contracts/src/ph1n.rs`
  - `IntentDraft` already carries `required_fields_missing`, `overall_confidence`, `evidence_spans`.
- Existing deterministic intent extraction + one-question clarify primitives:
  - `crates/selene_engines/src/ph1n.rs`
  - `detect_intents`, `normalize_*`, `clarify_for_missing`, `select_primary_missing`.
- Existing dispatch contract for simulation candidates:
  - `crates/selene_kernel_contracts/src/ph1x.rs`
  - `SimulationCandidateDispatch`, `DispatchRequest::SimulationCandidate`, `requires_clarify`.
- Existing PH1.X one-question clarify discipline:
  - `crates/selene_engines/src/ph1x.rs`
  - `clarify_for_missing`, `select_primary_missing`.
- Existing execution guards (access + simulation active + fail-closed reason codes):
  - `crates/selene_os/src/simulation_executor.rs`
  - `execute_ph1x_dispatch_simulation_candidate`, `ensure_simulation_active_for_tenant`, `enforce_access_gate`.
- Existing simulation catalog DB truth (ACTIVE/DRAFT awareness available):
  - `crates/selene_kernel_contracts/src/ph1simcat.rs` (`SimulationStatus`: `Draft|Active|Deprecated|Disabled`)
  - `crates/selene_storage/src/ph1f.rs` (`simulation_catalog_events`, `simulation_catalog_current`).
- Existing multilingual helpers (already implemented, currently separate wiring):
  - `crates/selene_os/src/ph1lang.rs`, `crates/selene_engines/src/ph1lang.rs`
  - `crates/selene_os/src/ph1srl.rs`, `crates/selene_engines/src/ph1srl.rs`.
- Existing LLM assist surface (non-authoritative model-boundary decisions):
  - `docs/ECM/PH1_D.md`, `docs/DB_WIRING/PH1_D.md`
  - `crates/selene_engines/src/ph1d.rs`, `crates/selene_os/src/ph1d.rs`.
- Existing OCR assist route + OCR->Context->NLP bridge:
  - `crates/selene_os/src/ph1os.rs` (`OsOcrRoute*`, `Ph1OsOcrContextNlpWiring::run_handoff`).
- Existing context + memory assist surfaces:
  - `docs/ECM/PH1_CONTEXT.md`, `docs/DB_WIRING/PH1_CONTEXT.md`
  - `docs/ECM/PH1_M.md`, `docs/DB_WIRING/PH1_M.md`.
- Existing voice identity assertion surface:
  - `crates/selene_os/src/ph1_voice_id.rs`
  - `docs/DB_WIRING/PH1_OS.md` top-level path includes `PH1.VOICE.ID`.
- Existing vocabulary/pronunciation hint surfaces:
  - `docs/ECM/PH1_KNOW.md`, `docs/DB_WIRING/PH1_KNOW.md`
  - `docs/ECM/PH1_PRON.md`, `docs/DB_WIRING/PH1_PRON.md`.
- Existing learning signal loop (gold capture/corrections):
  - `crates/selene_adapter/src/lib.rs`
  - `emit_ph1c_gold_capture_and_learning`, `emit_ph1d_gold_capture_and_learning`, correction capture path.
- Existing feedback/learn/promotion governance path:
  - `crates/selene_adapter/src/lib.rs`
  - `crates/selene_os/src/ph1pae.rs`
  - `crates/selene_os/src/ph1builder.rs`
  - `docs/ECM/PH1_OS.md` (`PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE -> PH1.BUILDER`).
- Existing review/governance status:
  - standalone review runtime is merged into `PH1.GOV + PH1.ACCESS` (no separate runtime owner).
  - source: `docs/33_ENGINE_REVIEW_TRACKER.md`.

### Important current gap (must be added)

Today PH1.NLP detection is mostly keyword/rule matching and static intent->simulation mapping, and there is no dedicated missing-simulation intake workflow linked to runtime matching.

Additional gaps for reliable multilingual/broken speech matching:
- No unified deterministic ranking layer that combines NLP + LLM assist + OCR assist + context/memory + gold mappings into one top-1 simulation decision.
- No canonical packet schema for simulation-missing intake tied to catalog ACTIVE/DRAFT proof order.
- No dedicated intake abuse controls/rate caps in finder path.

## 3) New module scope (minimal)

Do not build a new standalone execution engine.

Add a bounded module inside PH1.NLP/PH1.X boundary:
- `Simulation Finder Core` (deterministic ranking and packet emission).
- It consumes PH1.LANG/SRL outputs when available.
- It can consume PH1.D (LLM) and OCR bridge output as assist-only evidence, never as execution authority.
- It emits only packets, then PH1.X and simulation executor keep existing gate order.

No autonomy, no bypass of access/confirm/simulation gates.

### 3.1 Authority boundary (single source of truth)

- Finder owns: candidate generation, deterministic ranking, and terminal packet emission.
- Execution Core/PH1.X owns: clarify relay state, confirm flow, access + ACTIVE gate orchestration, and dispatch orchestration.
- SimulationExecutor owns: simulation execution and side effects.

No layer may duplicate or override another layer's authority.

### 3.2 Required inputs (canonical)

- Transcript (`PH1.C` output; noisy input tolerated).
- STT quality metadata and optional N-best/probe artifacts when available from upstream STT routing.
- Voice identity assertion and identity tier (`PH1.VOICE.ID`, when available).
- Language hints and segmentation (`PH1.LANG`, when available).
- SRL repaired frame + ambiguity flags (`PH1.SRL`, when available).
- Tenant context (tenant id, policy scope, allowed domains).
- Simulation catalog snapshot with `Active|Draft|Deprecated|Disabled`.
- ECM capability map/capability ownership references.
- Known synonyms/vocabulary/pronunciation packs (`PH1.KNOW` + `PH1.PRON` hints).
- Context/memory references (`PH1.CONTEXT` + `PH1.M`, identity-gated).
- OCR-derived text/context bundle (when OCR route is active).
- LLM assist candidates from PH1.D (assist-only, bounded).
- Gold outputs (confirmed mappings/correction pairs).

### 3.3 Wiring path (design target)

- Voice-first path:
  - `PH1.C -> PH1.LANG -> PH1.SRL -> PH1.NLP -> Simulation Finder Core -> PH1.X -> simulation_executor`.
- OCR-assisted path (when image/doc context exists):
  - `OCR route -> OCR->Context->NLP bridge -> Simulation Finder Core`.
- LLM-assisted understanding path (assist-only):
  - `PH1.D intent/clarify refinement output -> Simulation Finder Core ranking input`.
- Governance/review boundary:
  - runtime review routes through `PH1.GOV + PH1.ACCESS` (no standalone review runtime).

## 4) Output packet specs

Reason code canonical rule:
- Section `6) Thresholds and reason codes` is the canonical registry.
- Per-packet reason code lists in this section are allowed subsets only.

## 4.1 SimulationMatchPacket

Schema (`SimulationMatchPacket.v1`) required fields:
- `packet_type = "SIMULATION_MATCH"`
- `schema_version`
- `tenant_id`
- `user_id`
- `correlation_id`
- `turn_id`
- `intent_family` (canonical)
- `simulation_id` (exact candidate)
- `candidate_rank = 1` (top-1 only)
- `confidence_bp`
- `required_fields_present[]`
- `required_fields_missing[]`
- `evidence_spans[]`
- `risk_tier` (`LOW|MEDIUM|HIGH`)
- `confirm_required`
- `access_actions_required[]`
- `idempotency_key`
- `idempotency_recipe_ref`
- `fallback_if_inactive_or_missing` (`CLARIFY|MISSING_SIMULATION|REFUSE`)
- `reason_code`

Allowed reason codes:
- `SIM_FINDER_MATCH_OK`
- `SIM_FINDER_MATCH_OK_GOLD_BOOSTED`
- `SIM_FINDER_MATCH_OK_CATALOG_ACTIVE`

Idempotency key recipe:
- `sim_match:{tenant_id}:{user_id}:{correlation_id}:{turn_id}:{simulation_id}:{required_fields_fingerprint}`

## 4.2 ClarifyPacket

Schema (`ClarifyPacket.v1`) required fields:
- `packet_type = "CLARIFY"`
- `schema_version`
- `tenant_id`
- `user_id`
- `correlation_id`
- `turn_id`
- `question` (exactly one question)
- `missing_field` (single canonical field)
- `allowed_answer_formats` (2-3 examples)
- `attempt_index`
- `max_attempts`
- `on_exceed` (`MISSING_SIMULATION` or deterministic `REFUSE`)
- `candidate_context_ref` (reference to ranked candidate snapshot)
- `idempotency_key`
- `reason_code`

Allowed reason codes:
- `SIM_FINDER_CLARIFY_MISSING_FIELD`
- `SIM_FINDER_CLARIFY_AMBIGUOUS`
- `SIM_FINDER_CLARIFY_LOW_CONFIDENCE_TIE`
- `SIM_FINDER_ABSTAIN_LOW_CALIBRATED_CONFIDENCE`

Idempotency key recipe:
- `sim_clarify:{tenant_id}:{user_id}:{correlation_id}:{turn_id}:{missing_field}:{attempt_index}`

## 4.3 RefusePacket

Required fields:
- `reason_code` (`SIM_FINDER_REFUSE_ACCESS_DENIED|SIM_FINDER_REFUSE_ACCESS_AP_REQUIRED|SIM_FINDER_REFUSE_UNSAFE_REQUEST|SIM_FINDER_REFUSE_AMBIGUOUS|SIM_FINDER_REFUSE_POLICY_BLOCKED|SIM_FINDER_SIMULATION_INACTIVE|SIM_FINDER_REPLAY_ARTIFACT_MISSING`)
- `message`
- `evidence_refs[]`
- `existing_draft_ref` (nullable; required when `reason_code=SIM_FINDER_SIMULATION_INACTIVE`)

## 4.4 MissingSimulationPacket (Dev Intake)

Schema (`MissingSimulationPacket.v1`) required fields:
- `packet_type = "MISSING_SIMULATION"`
- `schema_version`
- `tenant_id`
- `user_id`
- `correlation_id`
- `turn_id`
- `requested_capability_name_normalized`
- `raw_user_utterance`
- `cleaned_paraphrase`
- `category`
- `estimated_frequency_score_bp`
- `estimated_value_score_bp`
- `estimated_roi_score_bp`
- `estimated_feasibility_score_bp`
- `estimated_risk_score_bp`
- `worthiness_score_bp`
- `scope_class` (`tenant_only|cross_tenant`)
- `required_integrations[]`
- `proposed_simulation_family`
- `required_fields_schema_json`
- `acceptance_test_suggestion[]`
- `dedupe_fingerprint`
- `catalog_check_trace[]` (`ACTIVE_CHECK`, `DRAFT_CHECK`, `NONE_FOUND` with timestamped proof refs)
- `active_check_proof_ref`
- `draft_check_proof_ref`
- `no_match_proof_ref`
- `existing_draft_ref` (nullable; must be `null` for canonical `MissingSimulationPacket` flow because `Draft` routes to `RefusePacket`)
- `idempotency_key`
- `reason_code`

Allowed reason codes:
- `SIM_FINDER_MISSING_SIMULATION`
- `SIM_FINDER_MISSING_SIMULATION_DECLINED_LOW_VALUE_HIGH_RISK`
- `SIM_FINDER_MISSING_SIMULATION_RATE_LIMITED`
- `SIM_FINDER_MISSING_SIMULATION_DAILY_CAP_REACHED`

Idempotency key recipe:
- Canonical source: see Section `16.12` (`SIL` idempotency registry, authoritative).

## 5) Deterministic matching/ranking logic

Input order (fixed):
1. Transcript (post-STT)
2. STT N-best/probe hints (if present)
3. Voice identity assertion + tenant context
4. PH1.LANG hints (if present)
5. PH1.SRL repaired transcript/ambiguity flags (if present)
6. PH1.KNOW/PH1.PRON/tenant synonym hints (if present)
7. PH1.CONTEXT/PH1.M identity-gated context refs (if present)
8. OCR bridge context (if present)
9. PH1.NLP candidate intents + extracted fields
10. PH1.D assist candidates (if present; assist-only)
11. Simulation catalog status snapshot (`simulation_catalog_current`)
12. ECM capability map references (for candidate-family validity checks)
13. Access action map (required actions per candidate)
14. Gold mapping cache (confirmed corrections)

### 5.1 “Understand anything” helper stack (deterministic output)

- STT improvements + N-best/probe hints (advisory only).
- PH1.LANG language probe/segmentation for multilingual/code-switch boundaries.
- PH1.SRL repair without meaning drift.
- PH1.NLP candidate extraction (not final authority).
- PH1.D LLM assist candidate normalization/refinement (assist-only, bounded).
- PH1.KNOW/PH1.PRON vocabulary and pronunciation hints.
- PH1.CONTEXT/PH1.M identity-gated context disambiguation.
- Gold outputs from confirmed corrections as ranking bonus only.

Hard rule:
- Helper outputs cannot directly dispatch simulations.
- Finder always emits one deterministic terminal packet (`Match`, `Clarify`, `Refuse`, or `MissingSimulation`).

### 5.2 Candidate Generation (Top-K) and Stable Ranking

Candidate generation (deterministic `top_k`):
- Source 1: simulation-family synonym map.
- Source 2: tenant vocabulary/synonym packs (`PH1.KNOW`/`PH1.PRON` derived hints).
- Source 3: gold mapping memory from confirmed corrections.
- Source 4: recent thread context and identity-gated memory refs.

Deterministic ranking order:
1. `confidence_score_bp` (descending)
2. `gold_match_bonus_bp` (descending)
3. `simulation_priority` (descending)
4. stable lexicographic tie-break by `simulation_id` (ascending)

Only one terminal emission is allowed:
- top-1 `SimulationMatchPacket`, or
- single `ClarifyPacket`, or
- `RefusePacket`, or
- `MissingSimulationPacket`.

Scoring (basis points, deterministic):
- `intent_confidence_bp`
- `required_field_coverage_bp`
- `evidence_coverage_bp`
- `catalog_status_bp` (Active > Draft > Disabled/Deprecated)
- `context_alignment_bp`
- `ocr_alignment_bp`
- `llm_assist_alignment_bp` (bounded assist bonus only)
- `gold_match_bonus_bp`
- penalties: ambiguity, contradictory fields, policy mismatch.

Canonical `confidence_score_bp` formula (integer math):
- `penalty_bp_total = ambiguity_penalty_bp + contradictory_field_penalty_bp + policy_mismatch_penalty_bp`
- `raw_score_bp = (35*intent_confidence_bp + 20*required_field_coverage_bp + 10*evidence_coverage_bp + 10*catalog_status_bp + 10*context_alignment_bp + 5*ocr_alignment_bp + 5*llm_assist_alignment_bp + 5*gold_match_bonus_bp) / 100`
- `confidence_score_bp = clamp(raw_score_bp - penalty_bp_total, 0, 10000)`
- rounding mode: floor integer division only.

Selection rule:
- Emit top-1 only.
- If top-1 below threshold, or top-1/top-2 margin below tie threshold -> `ClarifyPacket` (one question).
- If still unresolved after clarify budget and no valid catalog target -> `MissingSimulationPacket`.

### 5.3 Assist artifact pinning and replay determinism

Any assist input that influences ranking must carry immutable artifact references:
- `stt_artifact_ref` + `stt_model_id` + `stt_model_version` + `stt_decode_profile_hash`
- `lang_artifact_ref` + `lang_model_version`
- `srl_artifact_ref` + `srl_model_version`
- `llm_assist_ref` + `llm_model_id` + `llm_model_version` + `llm_prompt_template_hash`
- `ocr_artifact_ref` + `ocr_model_version`

Deterministic replay rule:
- If any referenced artifact is missing, packet replay is invalid and must fail closed with `SIM_FINDER_REPLAY_ARTIFACT_MISSING`.
- Finder scoring must be reproducible from packet refs + catalog snapshot + policy snapshot only.

## 6) Thresholds and reason codes

Initial deterministic thresholds (design target):
- `MATCH_DIRECT_MIN_BP = 9000`
- `MATCH_WITH_CLARIFY_MIN_BP = 7000`
- `TIE_MARGIN_MIN_BP = 800`
- `MAX_CLARIFY_ATTEMPTS = 2`

Reason code family (design target):
- `SIM_FINDER_MATCH_OK`
- `SIM_FINDER_MATCH_OK_GOLD_BOOSTED`
- `SIM_FINDER_MATCH_OK_CATALOG_ACTIVE`
- `SIM_FINDER_CLARIFY_MISSING_FIELD`
- `SIM_FINDER_CLARIFY_AMBIGUOUS`
- `SIM_FINDER_CLARIFY_LOW_CONFIDENCE_TIE`
- `SIM_FINDER_REFUSE_ACCESS_DENIED`
- `SIM_FINDER_REFUSE_ACCESS_AP_REQUIRED`
- `SIM_FINDER_REFUSE_UNSAFE_REQUEST`
- `SIM_FINDER_REFUSE_AMBIGUOUS`
- `SIM_FINDER_REFUSE_POLICY_BLOCKED`
- `SIM_FINDER_SIMULATION_INACTIVE`
- `SIM_FINDER_REPLAY_ARTIFACT_MISSING`
- `SIM_FINDER_ABSTAIN_LOW_CALIBRATED_CONFIDENCE`
- `SIM_FINDER_MISSING_SIMULATION`
- `SIM_FINDER_MISSING_SIMULATION_DECLINED_LOW_VALUE_HIGH_RISK`
- `SIM_FINDER_MISSING_SIMULATION_RATE_LIMITED`
- `SIM_FINDER_MISSING_SIMULATION_DAILY_CAP_REACHED`

### 6.1 Clarify Quality Discipline (Sharper One-Question Strategy)

Clarify selection must be deterministic and single-question:
- choose the highest-entropy missing field first
- never ask more than one question in the same turn
- provide exactly 2-3 accepted answer formats
- increment attempt counter deterministically
- escalate deterministically after `MAX_CLARIFY_ATTEMPTS`
- never repeat the exact same clarify question twice for the same attempt state

### 6.2 Deterministic entropy formula for clarify field selection

For each missing field `f`, compute:
- `entropy_score_bp(f) = (50*domain_cardinality_bp(f) + 30*candidate_split_bp(f) + 20*downstream_risk_bp(f)) / 100`
- rounding mode: floor integer division only.

Selection order:
1. Highest `entropy_score_bp(f)`
2. Highest `downstream_risk_bp(f)`
3. Lexicographic field name tie-break (ascending)

This is the only valid clarify-field selector. No heuristic override is allowed.

### 6.3 Confidence calibration and abstention policy

Calibration contract:
- Score deciles must be calibrated per tenant and language cohort using a frozen validation window.
- `MATCH_DIRECT_MIN_BP`, `MATCH_WITH_CLARIFY_MIN_BP`, and `TIE_MARGIN_MIN_BP` are policy values and must be versioned.

Abstention law:
- If calibrated confidence falls below threshold, Finder must abstain from match and emit `ClarifyPacket` or `MissingSimulationPacket`.
- Abstention is preferred over false-positive dispatch suggestion.

## 7) Access and simulation gating integration

Finder core is pre-dispatch only.

Gate order remains:
1. PH1.NLP/PH1.X produce packet
2. PH1.X/Execution Core confirmation path (unchanged)
3. Execution Core performs pre-dispatch access + active simulation precheck.
4. Simulation executor performs final authoritative access + active simulation hard gate.

If catalog status is not `Active`, finder does not execute; it emits `MissingSimulationPacket` or `RefusePacket` based on policy.

### 7.1 Deterministic “No Simulation Exists” Proof Order (Mandatory)

Before emitting `MissingSimulationPacket`, finder must run this exact deterministic check order:
1. Check `simulation_catalog_current` for candidate with `status=Active`.
2. If none, check same candidate family with `status=Draft`.
3. If `Draft` exists, emit `RefusePacket` with reason `SIM_FINDER_SIMULATION_INACTIVE`, include `existing_draft_ref`, and do not create a new `MissingSimulationPacket`.
4. Only when neither `Active` nor `Draft` exists may finder emit `MissingSimulationPacket` with reason `SIM_FINDER_MISSING_SIMULATION`.

This order is non-optional and must be auditable in packet evidence refs.

## 8) Learning and gold-output loop

Use existing correction capture and FEEDBACK/LEARN pipeline.

Add a small finder-specific gold mapping layer:
- Capture only confirmed tuples:
  - `(normalized_utterance_fingerprint, tenant_id, language_tag) -> (intent_family, simulation_id, required_field_pattern)`
- Consume as ranking bonus only (never authority override).

Promotion path:
- Shadow: observe and log candidate + final chosen outcome.
- Assist: apply bounded score bonus from gold map.
- Lead: not applicable for authority; finder remains advisory to PH1.X/executor gates.

## 9) Dev Intake workflow

When no matching simulation exists after clarification and deterministic `ACTIVE -> DRAFT -> NONE` proof order:
1. Emit `MissingSimulationPacket`.
2. Persist intake ledger row (append-only).
3. Update current projection row deterministically.
4. Notify user that request was submitted for review.
5. Later, when simulation becomes `Active`, send availability notification to original requester.

Dev lifecycle closure (system-wide):
6. Dev team triages intake packet (accept/reject/defer).
7. If accepted, simulation is implemented and activated in catalog.
8. Finder linkage marks intake row resolved by `simulation_id + version`.
9. Original requester is notified capability is now active.

### 9.1 Abuse/Spam Controls (Mandatory)

Canonical window model (single source of truth):
- All SIL dedupe and rate-limit evaluations use `UTC day buckets` only.
- Window key: `window_bucket_utc = yyyymmdd_utc`.

Rate limits (fail closed when exceeded):
- Per user/day: max `3` new Dev Intake tickets per `window_bucket_utc`.
- Per tenant/day: max `20` new Dev Intake tickets per `window_bucket_utc`.
- Per capability/day: max `10` new Dev Intake tickets per `(tenant_id, dedupe_fingerprint, window_bucket_utc)`.
- Daily hard cap: max `100` new Dev Intake tickets per tenant per `window_bucket_utc`.

Dedupe rules:
- Primary dedupe key: `(tenant_id, user_id, dedupe_fingerprint, window_bucket_utc)`.
- Secondary dedupe key: `(tenant_id, dedupe_fingerprint, window_bucket_utc)` to suppress cross-user duplicates for same capability request.
- If deduped, do not create new ticket; return existing intake reference.

Cap behavior:
- On per-user/per-tenant rate exceed -> reason `SIM_FINDER_MISSING_SIMULATION_RATE_LIMITED`.
- On daily hard cap exceed -> reason `SIM_FINDER_MISSING_SIMULATION_DAILY_CAP_REACHED`.

### PH1.F storage design (new tables)

- `sim_finder_missing_sim_ledger` (append-only)
- `sim_finder_missing_sim_current` (projection)
- `sim_finder_gold_mapping_ledger` (append-only confirmed mapping events)
- `sim_finder_gold_mapping_current` (projection)

Idempotency/dedupe keys:
- Missing-sim intake and SIL write-path idempotency recipes are canonicalized in Section `16.12`.
- Gold mapping update: `(tenant_id, utterance_fingerprint, simulation_id, idempotency_key)`

Replay rule:
- `*_current` must rebuild from `*_ledger` only.
- No silent transitions; append-only enforced.

## 10) Simulation worthiness scoring (before forward)

Input components (all basis points `0..10000`):
- `frequency_bp`: normalized ask frequency for equivalent dedupe fingerprint window.
- `value_bp`: estimated business impact/time saved class.
- `estimated_roi_score_bp`: deterministic ROI estimate for implementation payoff.
- `feasibility_bp`: integration/runtime feasibility estimate.
- `scope_bp`: tenant-only vs multi-tenant/generalizability score.
- `risk_bp`: money/permission/external-send/safety risk estimate.

Deterministic formula (fixed):
- `worthiness_raw_bp = (25*frequency_bp + 25*value_bp + 15*estimated_roi_score_bp + 20*feasibility_bp + 15*scope_bp) / 100`
- `risk_penalty_bp = risk_bp / 2`
- `worthiness_score_bp = max(0, worthiness_raw_bp - risk_penalty_bp)`

Forwarding thresholds (fixed):
- `FORWARD_MIN_BP = 6500`
- `HIGH_RISK_BLOCK_BP = 7000`
- `LOW_VALUE_BLOCK_BP = 3000`

Forwarding policy:
- Forward to Dev Intake only if:
  - `worthiness_score_bp >= FORWARD_MIN_BP`, and
  - `risk_bp < HIGH_RISK_BLOCK_BP`.
- Decline forwarding with `SIM_FINDER_MISSING_SIMULATION_DECLINED_LOW_VALUE_HIGH_RISK` if:
  - `risk_bp >= HIGH_RISK_BLOCK_BP`, or
  - `value_bp <= LOW_VALUE_BLOCK_BP` and `risk_bp >= 5000`.

## 11) Milestones

| Milestone | Goal | Artifacts Produced | Sims/Policies Affected | Acceptance Tests | Proof Commands |
|---|---|---|---|---|---|
| M0 | Repo baseline overlap audit | overlap matrix, gap list, no-duplication map | none | `AT-SIM-FINDER-M0-01`, `AT-SIM-FINDER-M0-02` | `rg -n "Ph1n|SimulationCandidate|simulation_catalog" crates docs` |
| M1 | Packet contract spec | packet schemas + reason code registry + threshold pack | none | `AT-SIM-FINDER-M1-01`, `AT-SIM-FINDER-M1-02` | `cargo test -p selene_kernel_contracts ph1n ph1x -- --nocapture` |
| M2 | Deterministic ranking core | score formula spec, tie-break spec, deterministic ordering spec | none | `AT-SIM-FINDER-M2-01`..`AT-SIM-FINDER-M2-08` | `cargo test -p selene_engines ph1n -- --nocapture` |
| M3 | One-question clarify + fail-closed | clarify attempt policy, escalation matrix | PH1.X clarify policy pack | `AT-SIM-FINDER-M3-01`..`AT-SIM-FINDER-M3-04` | `cargo test -p selene_os ph1x -- --nocapture` |
| M4 | Catalog/access integration | ACTIVE/DRAFT status gating spec, guard mapping table | simulation guard policy only | `AT-SIM-FINDER-M4-01`, `AT-SIM-FINDER-M4-02` | `cargo test -p selene_os simulation_executor -- --nocapture` |
| M5 | Gold-output loop | gold mapping schema + correction->gold mapping spec | LEARN/PAE consumption policy | `AT-SIM-FINDER-M5-01`, `AT-SIM-FINDER-M5-02`, `AT-SIM-FINDER-M5-03` | `cargo test -p selene_adapter at_adapter_03i_user_correction_phrase_emits_feedback_and_learn_signal_bundle -- --nocapture` |
| M6 | Dev Intake ledger + dedupe + worthiness | PH1.F table specs, dedupe rules, worthiness rubric, lifecycle statuses | `SIM_FINDER_MISSING_SIM_CREATE_COMMIT` (design target) | `AT-SIM-FINDER-M6-01`, `AT-SIM-FINDER-M6-02`, `AT-SIM-FINDER-M6-03`, `AT-SIM-FINDER-M6-04` | `cargo test -p selene_storage sim_finder_missing_sim -- --nocapture` |
| M7 | Notification closure | availability notification trigger spec on simulation activation | `SIM_FINDER_MISSING_SIM_NOTIFY_AVAILABLE_COMMIT` (design target) | `AT-SIM-FINDER-M7-01`, `AT-SIM-FINDER-M7-02` | `cargo test -p selene_os sim_finder_notify -- --nocapture` |
| M8 | CI guardrails | acceptance script + readiness gate wiring + proof checklist | CI policy only | `AT-SIM-FINDER-M8-01`..`AT-SIM-FINDER-M8-06` | `bash scripts/check_agent_sim_finder_core_acceptance.sh && bash scripts/check_ph1_readiness_strict.sh` |

## 12) Acceptance test inventory (design target)

- `AT-SIM-FINDER-M0-01-overlap-matrix-complete`
- `AT-SIM-FINDER-M0-02-no-duplicate-authority-path`
- `AT-SIM-FINDER-M1-01-packet-schemas-validate`
- `AT-SIM-FINDER-M1-02-reason-codes-are-bounded`
- `AT-SIM-FINDER-M2-01-identical-input-yields-identical-top1`
- `AT-SIM-FINDER-M2-02-tie-below-margin-forces-clarify`
- `AT-SIM-FINDER-M2-03-no-candidate-after-clarify-yields-missing-simulation`
- `AT-SIM-FINDER-M2-04-broken-english-maps-to-correct-simulation`
- `AT-SIM-FINDER-M2-05-code-switch-phrase-maps-to-correct-simulation`
- `AT-SIM-FINDER-M2-06-rambling-unfinished-sentence-triggers-single-clarify`
- `AT-SIM-FINDER-M2-07-calibrated-low-confidence-abstains-from-match`
- `AT-SIM-FINDER-M2-08-assist-artifact-fingerprint-required-for-ranking`
- `AT-SIM-FINDER-M3-01-only-one-question-per-turn`
- `AT-SIM-FINDER-M3-02-missing-field-priority-order-deterministic`
- `AT-SIM-FINDER-M3-03-clarify-attempt-budget-escalates-deterministically`
- `AT-SIM-FINDER-M3-04-entropy-formula-selects-primary-missing-field-deterministically`
- `AT-SIM-FINDER-M4-01-inactive-simulation-fails-closed`
- `AT-SIM-FINDER-M4-02-access-deny-or-escalate-blocks-dispatch`
- `AT-SIM-FINDER-M5-01-confirmed-correction-creates-single-gold-row`
- `AT-SIM-FINDER-M5-02-gold-map-bonus-never-overrides-access-or-confirm`
- `AT-SIM-FINDER-M5-03-gold-map-idempotency-dedupes-retries`
- `AT-SIM-FINDER-M6-01-missing-sim-intake-appends-ledger-and-projects-current`
- `AT-SIM-FINDER-M6-02-missing-sim-intake-dedupes-by-fingerprint`
- `AT-SIM-FINDER-M6-03-low-value-high-risk-declines-forwarding`
- `AT-SIM-FINDER-M6-04-current-rebuilds-from-ledger-exactly`
- `AT-SIM-FINDER-M6-05-dev-intake-rate-limit-enforced-per-user-and-tenant`
- `AT-SIM-FINDER-M6-06-dev-intake-daily-cap-enforced`
- `AT-SIM-FINDER-M6-07-active-then-draft-then-none-check-order-enforced`
- `AT-SIM-FINDER-M7-01-activation-event-notifies-original-requester-once`
- `AT-SIM-FINDER-M7-02-notification-is-idempotent-and-auditable`
- `AT-SIM-FINDER-M8-01-ci-fails-on-missing-proof-chain`
- `AT-SIM-FINDER-M8-02-ci-fails-on-missing-dedupe-or-replay-tests`
- `AT-SIM-FINDER-M8-03-ci-fails-on-multi-question-clarify-regression`
- `AT-SIM-FINDER-M8-04-ci-fails-on-terminal-outcome-set-drift`
- `AT-SIM-FINDER-M8-05-ci-fails-on-missing-artifact-fingerprints`
- `AT-SIM-FINDER-M8-06-ci-fails-on-active-draft-none-proof-gap`

## 12.1 World-Class Scoreboards (Required)

Operational scoreboards (tenant-scoped, deterministic windows):
- top-1 match accuracy (`top1_correct / total_dispatches`) by tenant
- clarify efficiency (`clarify_turns_to_dispatch` p50/p95)
- false positive rate (`wrong_sim_selected / total_dispatches`)
- missing-simulation detection hit rate (`true_missing_sim / total_missing_sim_flags`)

Required acceptance tests for scoreboards:
- `AT-SIM-FINDER-METRIC-01-top1-match-accuracy-by-tenant-computed-deterministically`
- `AT-SIM-FINDER-METRIC-02-clarify-efficiency-turns-to-dispatch-computed`
- `AT-SIM-FINDER-METRIC-03-false-positive-rate-computed-and-bounded`
- `AT-SIM-FINDER-METRIC-04-missing-sim-hit-rate-computed-and-auditable`
- `AT-SIM-FINDER-METRIC-05-deployment-gates-block-promotion-on-threshold-failure`

### 12.2 Deployment quality gates (builder-governed, fail closed)

Promotion gate (Shadow -> Assist) minimums:
- `top1_match_accuracy >= 0.95`
- `false_positive_rate <= 0.01`
- `missing_sim_hit_rate >= 0.98`
- `clarify_turns_to_dispatch_p95 <= 2`

Promotion gate (Assist -> Lead-equivalent advisory profile) minimums:
- `top1_match_accuracy >= 0.98`
- `false_positive_rate <= 0.005`
- `missing_sim_hit_rate >= 0.99`
- multilingual suite pass rate `>= 0.98`

If any gate fails, release is blocked and policy remains at lower mode.

### 12.3 Benchmark governance (frozen corpora and drift checks)

Required benchmark sets:
- `BENCH_SIM_FINDER_BASELINE_EN` (clean and noisy English)
- `BENCH_SIM_FINDER_MULTILINGUAL_CS` (code-switch and mixed-script)
- `BENCH_SIM_FINDER_ASR_NOISE` (accent, truncation, filler-heavy)
- `BENCH_SIM_FINDER_ADVERSARIAL` (entity collision, contradictory slots, prompt-like noise)

Governance rules:
- Benchmark sets are immutable per release candidate.
- Any score regression above tolerance requires explicit builder approval.
- Monthly drift report is mandatory and attached to promotion review.

### 12.4 Red-team suite (mandatory before promotion)

Red-team scenarios must include:
- spoofed high-confidence wrong-sim attempts
- conflicting entities in one utterance
- multilingual homophone collisions
- ambiguous confirmations disguised as casual phrases

Required red-team tests:
- `AT-SIM-FINDER-REDTEAM-01-spoofed-confidence-does-not-force-match`
- `AT-SIM-FINDER-REDTEAM-02-entity-collision-forces-clarify`
- `AT-SIM-FINDER-REDTEAM-03-homophone-collision-forces-deterministic-clarify`
- `AT-SIM-FINDER-REDTEAM-04-ambiguous-input-never-emits-match`

### 12.5 CI guardrails (mandatory)

`check_agent_sim_finder_core_acceptance.sh` must fail if:
- terminal outcome set deviates from canonical four-packet set
- missing-simulation packet omits ACTIVE->DRAFT->NONE proof refs
- assist-influenced decisions omit artifact fingerprints
- calibrated abstention thresholds are missing from policy snapshot
- replay from persisted refs does not reproduce the terminal packet

Implementation note:
- `check_agent_sim_finder_core_acceptance.sh` is a design-target script for `M8`; before `M8`, milestone proof commands remain the source of truth.

Production lock condition:
- Finder is not production-locked until `check_agent_sim_finder_core_acceptance.sh` exists and passes in CI.

## 13) Non-goals

- No autonomous execution authority.
- No bypass of PH1.X confirmation.
- No bypass of access gates.
- No bypass of simulation catalog ACTIVE checks.
- No schema migrations or runtime behavior changes in this design packet.
- No replacement of PH1.NLP/PH1.X; this is a bounded extension.

## 14) Deterministic replay contract appendix

Replay packet minimums (all terminal packets):
- `policy_snapshot_ref`
- `catalog_snapshot_ref`
- `artifact_fingerprint_bundle_ref`
- `score_breakdown_ref`
- `idempotency_key`
- `decision_reason_code`
- `decision_timestamp`

Replay invariants:
- Same inputs and same snapshot refs must reproduce same terminal packet.
- If replay cannot be reproduced, decision is invalid and must fail closed.

## 15) Naming recommendation

Preferred canonical docs name: `Simulation Finder Core`.

Reason: it is precise about function, avoids overlap with broader agent/runtime concepts, and matches the hard law that it only finds/routes but does not execute.

## 16) Controlled Self-Improvement Loop (Design -> Build)

Goal:
- Make missing-simulation handling an end-to-end governed pipeline:
  - user request -> `MissingSimulationPacket` -> dedupe + worthiness score -> builder proposal (`Draft`) -> human review -> simulation added/activated -> requester notified.

Hard laws:
- No auto-execution of new simulations.
- Builder remains the only `Active` promoter.
- Access rules apply to notification and any downstream action.
- Dedupe + rate limits are mandatory.
- Full audit chain is required.

### 16.1 Milestone M0 — End-to-end data model (DB truth)

Deliverables:
- Dev Intake tables (`PH1.F`):
  - `sim_finder_missing_sim_ledger` (append-only)
  - `sim_finder_missing_sim_current` (projection)
  - `sim_finder_missing_sim_dedupe_index`
- Builder linkage + notify tables:
  - `missing_sim_to_builder_proposal_link` (append-only)
  - `notify_requester_ledger` / `notify_requester_current` (append-only + projection)

Acceptance tests:
- `AT-SIL-M0-01`: insert missing-sim ledger row idempotently.
- `AT-SIL-M0-02`: dedupe index prevents duplicates.
- `AT-SIL-M0-03`: projection rebuild equals current.

Proof command:
- `cargo test -p selene_storage sil_m0_ -- --nocapture`

### 16.2 Milestone M1 — Wire MissingSimulation -> Dev Intake (verify current wiring)

Deliverables:
- Finder emits `MissingSimulationPacket` with proof-trace fields.
- Storage commit persists:
  - `dedupe_fingerprint`
  - `worthiness_score_bp`
  - `requester_user_id`
  - notify-stub audit row

Acceptance tests:
- `AT-SIL-M1-01`: missing-sim packet writes ledger row.
- `AT-SIL-M1-02`: duplicate request dedupes.
- `AT-SIL-M1-03`: worthiness score stored correctly.

Proof commands:
- `cargo test -p selene_os at_finder_exec_missing_sim_ -- --nocapture`
- `cargo test -p selene_storage sil_m1_ -- --nocapture`

### 16.3 Milestone M2 — Worthiness scoring + spam controls (deterministic)

Deliverables:
- Deterministic scoring function (stored + versioned):
  - `score = f(frequency, value, risk, feasibility)`
- Threshold policy:
  - `auto_propose_threshold_bp`
  - `block_threshold_bp`
  - daily cap per user/tenant
- Rate limits:
  - per user/day
  - per tenant/day
  - per capability/day
- Escalation to refuse on abuse.

Acceptance tests:
- `AT-SIL-M2-01`: same inputs produce same score.
- `AT-SIL-M2-02`: caps enforced.
- `AT-SIL-M2-03`: blocked reason stored when rejected.

Proof command:
- `cargo test -p selene_storage sil_m2_ -- --nocapture`

### 16.4 Milestone M3 — Auto-proposal generation (proposal-only; no activation)

Deliverables:
- When a missing-sim reaches threshold + dedupe count, create builder proposal row as `Draft`.

Rules:
- Proposal is `Draft` only.
- No `Active` simulation is created automatically.
- No code changes are made automatically.
- Proposal payload includes:
  - suggested simulation family
  - required fields schema
  - integration requirements
  - acceptance test skeleton
  - rollback plan requirement
  - risk class

Acceptance tests:
- `AT-SIL-M3-01`: threshold triggers proposal row.
- `AT-SIL-M3-02`: below threshold does not.
- `AT-SIL-M3-03`: proposal is draft only.
- `AT-SIL-M3-04`: audit chain links `missing_sim_id -> proposal_id`.

Proof commands:
- `cargo test -p selene_os sil_m3_ -- --nocapture`
- `cargo test -p selene_storage sil_m3_ -- --nocapture`

### 16.5 Milestone M4 — Human review + activation handshake

Deliverables:
- Builder process picks up proposal.
- Human approves.
- Simulation is implemented + registered.
- Simulation becomes `Active` only through builder promotion flow.

Acceptance tests:
- `AT-SIL-M4-01`: non-builder cannot activate.
- `AT-SIL-M4-02`: builder activation creates `Active` row.
- `AT-SIL-M4-03`: missing-sim linked proposal marked `resolved`.

Proof command:
- `cargo test -p selene_os builder_activation_ -- --nocapture`

### 16.6 Milestone M5 — Notify original requester when capability becomes Active

Deliverables:
- On activation:
  - find all requesters linked to `dedupe_fingerprint`
  - send notification (`BCAST` or app message)
  - persist notify proof and prevent duplicates

Acceptance tests:
- `AT-SIL-M5-01`: notify sends once per requester.
- `AT-SIL-M5-02`: idempotent replay does not double-notify.
- `AT-SIL-M5-03`: notification includes "now supported" + simulation name.

Proof command:
- `cargo test -p selene_os sil_notify_ -- --nocapture`

### 16.7 Milestone M6 — CI guardrails (no dead reporting)

Deliverables:
- Guard script:
  - `scripts/check_sim_finder_self_improvement_loop.sh`
- CI fail conditions:
  - missing-sim rows without legal lifecycle transition evidence from Section `16.9`
  - proposals without linked missing-sim evidence
  - notify rows missing proof refs

Acceptance tests:
- `AT-SIL-M6-01`
- `AT-SIL-M6-02`
- `AT-SIL-M6-03`

Proof command:
- `bash scripts/check_sim_finder_self_improvement_loop.sh`

### 16.8 Build order (canonical)

- `M0 -> M1 (verify) -> M2 -> M3 -> M4 -> M5 -> M6`

Execution note:
- If `M1` is already mostly wired, verify it early, but do not run `M5` notify flow before `M4` activation linkage proofs are complete.

### 16.9 Canonical missing-sim lifecycle (authoritative)

`sim_finder_missing_sim_current.status` allowed values and transitions:
- `NEW -> DEDUPED`
- `NEW -> BLOCKED`
- `NEW -> PROPOSED`
- `DEDUPED -> BLOCKED`
- `DEDUPED -> PROPOSED`
- `PROPOSED -> RESOLVED`
- `RESOLVED -> NOTIFIED`

Hard rules:
- Any transition not listed above is invalid.
- Every transition requires exactly one append-only ledger row in `sim_finder_missing_sim_ledger`.
- Projection state must be reconstructable from ledger rows only (`no silent transitions`).

Resolve semantics (canonical):
- A row may enter `RESOLVED` only when all are present:
  - `resolved_simulation_id`
  - `resolved_capability_version`
  - `resolved_tenant_scope`
  - `activated_at`
  - `resolution_proof_ref`

### 16.10 SIL windowing + counters (authoritative)

Windowing rules:
- SIL uses UTC-day buckets only (`window_bucket_utc = yyyymmdd_utc`).
- No rolling 60-minute or rolling 24-hour windows are permitted for SIL decisions.

Counter model (design target):
- `sim_finder_missing_sim_rate_limit_current` keyed by:
  - `(tenant_id, user_id, window_bucket_utc)`
  - `(tenant_id, dedupe_fingerprint, window_bucket_utc)`
  - `(tenant_id, window_bucket_utc)`

Reset semantics:
- Counters reset only by natural bucket rollover at UTC midnight.
- Rebuild from ledger must reproduce identical counter values for a given bucket.

### 16.11 Access action registry for SIL writes (authoritative)

Required access action strings:
- `SIM_FINDER_MISSING_SIM_PROPOSAL_CREATE`
- `SIM_FINDER_MISSING_SIM_NOTIFY_REQUESTER`
- `SIM_FINDER_MISSING_SIM_ROLLBACK_OR_REOPEN`

Fail-closed behavior:
- `ALLOW` -> proceed to simulation-gated SIL commit.
- `DENY` -> stop and emit `SIM_FINDER_REFUSE_ACCESS_DENIED`.
- `ESCALATE` -> stop and emit `SIM_FINDER_REFUSE_ACCESS_AP_REQUIRED`.

No SIL write path may query a master/template policy for runtime allow/deny decisions.

### 16.12 SIL idempotency registry (authoritative)

Every SIL write path must use exactly one deterministic key recipe:
- Intake create:
  - `sil_intake:{tenant_id}:{user_id}:{dedupe_fingerprint}:{window_bucket_utc}:{correlation_id}:{turn_id}`
- Dedupe index upsert:
  - `sil_dedupe:{tenant_id}:{dedupe_fingerprint}:{window_bucket_utc}`
- Proposal-link append:
  - `sil_proposal_link:{tenant_id}:{missing_sim_id}:{proposal_id}`
- Resolve append:
  - `sil_resolve:{tenant_id}:{missing_sim_id}:{resolved_simulation_id}:{resolved_capability_version}`
- Notify enqueue:
  - `sil_notify_enqueue:{tenant_id}:{missing_sim_id}:{requester_user_id}:{resolved_capability_version}`
- Notify delivery attempt:
  - `sil_notify_attempt:{tenant_id}:{notify_request_id}:{attempt_index}`
- Notify finalize:
  - `sil_notify_finalize:{tenant_id}:{notify_request_id}:{terminal_status}`

Rule:
- This section is the single source of truth for SIL idempotency recipes.

### 16.13 Dedupe fingerprint specification (authoritative)

`dedupe_fingerprint` generation inputs:
- `requested_capability_name_normalized`
- `cleaned_paraphrase`
- `required_integrations[]` (sorted lexicographically)
- `category`
- `tenant_id`

Normalization:
- Unicode NFKC normalization.
- Lowercase.
- Trim + collapse internal whitespace to single spaces.
- Remove punctuation except connector separators (`/`, `+`, `-`) inside tokens.

Hash method:
- `sha256(normalized_payload_json)` hex-encoded.
- Include `dedupe_spec_version` alongside the fingerprint.

Versioning:
- initial version: `dedupe_spec_version = 1`.
- any normalization/hash input change requires version increment.

### 16.14 Notify requester status model (authoritative)

`notify_requester_current.status` allowed values:
- `PENDING`
- `SENT`
- `RETRY_SCHEDULED`
- `FAILED_POISONED`
- `SUPPRESSED_DUPLICATE`

Allowed transitions:
- `PENDING -> SENT`
- `PENDING -> RETRY_SCHEDULED`
- `RETRY_SCHEDULED -> SENT`
- `RETRY_SCHEDULED -> RETRY_SCHEDULED` (next attempt with incremented `attempt_index`)
- `RETRY_SCHEDULED -> FAILED_POISONED`
- `PENDING -> SUPPRESSED_DUPLICATE`

Retry/poison policy:
- `max_attempts = 3`
- if delivery still fails after `max_attempts`, transition to `FAILED_POISONED` with `poison_reason_code` and `poison_proof_ref`.
- once `SENT` or `SUPPRESSED_DUPLICATE`, row is terminal and immutable.

### 16.15 Finder/SIL milestone crosswalk (authoritative rollout map)

Authoritative track for missing-simulation self-improvement loop:
- SIL milestones in Section `16` are canonical.
- Finder milestones in Section `11` remain reference-level for broader Finder scope.

Crosswalk:
- SIL `M0` + `M1` + `M2` <-> Finder `M6` (Dev Intake storage, dedupe, worthiness).
- SIL `M3` + `M4` <-> Finder `M6` extension (proposal linkage + builder activation resolution).
- SIL `M5` <-> Finder `M7` (notification closure).
- SIL `M6` <-> Finder `M8` (CI guardrails).

### 16.16 SIL reliability SLOs (global-standard gates)

Required SLOs:
- `intake_to_proposal_p95 <= 24h`
- `proposal_to_resolved_p95 <= 14d`
- `false_proposal_rate <= 2%`
- `notify_delivery_success_rate >= 99%` (rolling 30d)
- `duplicate_notification_rate = 0`

Operational gate:
- If any SIL SLO is out of bounds, promotion remains blocked until corrected or explicitly builder-approved with bounded waiver.
