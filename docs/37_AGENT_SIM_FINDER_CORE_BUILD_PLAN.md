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
  - `PH1.REVIEW` standalone runtime is merged into `PH1.GOV + PH1.ACCESS` (no separate runtime owner).
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

### 3.1 Required inputs (canonical)

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

### 3.2 Wiring path (design target)

- Voice-first path:
  - `PH1.C -> PH1.LANG -> PH1.SRL -> PH1.NLP -> Simulation Finder Core -> PH1.X -> simulation_executor`.
- OCR-assisted path (when image/doc context exists):
  - `OCR route -> OCR->Context->NLP bridge -> Simulation Finder Core`.
- LLM-assisted understanding path (assist-only):
  - `PH1.D intent/clarify refinement output -> Simulation Finder Core ranking input`.
- Governance/review boundary:
  - runtime review routes through `PH1.GOV + PH1.ACCESS` (no standalone PH1.REVIEW runtime).

## 4) Output packet specs

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

Idempotency key recipe:
- `sim_clarify:{tenant_id}:{user_id}:{correlation_id}:{turn_id}:{missing_field}:{attempt_index}`

## 4.3 RefusePacket

Required fields:
- `reason_code` (`ACCESS_DENIED|ACCESS_AP_REQUIRED|UNSAFE|AMBIGUOUS|POLICY_BLOCKED|SIM_INACTIVE`)
- `message`
- `evidence_refs[]`

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
- `existing_draft_ref` (nullable; required when `Draft` exists)
- `idempotency_key`
- `reason_code`

Allowed reason codes:
- `SIM_FINDER_MISSING_SIMULATION`
- `SIM_FINDER_MISSING_SIMULATION_DECLINED_LOW_VALUE_HIGH_RISK`
- `SIM_FINDER_MISSING_SIMULATION_RATE_LIMITED`
- `SIM_FINDER_MISSING_SIMULATION_DAILY_CAP_REACHED`

Idempotency key recipe:
- `missing_sim:{tenant_id}:{user_id}:{dedupe_fingerprint}:{correlation_id}:{turn_id}`

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
- `SIM_FINDER_CLARIFY_MISSING_FIELD`
- `SIM_FINDER_CLARIFY_AMBIGUOUS`
- `SIM_FINDER_REFUSE_ACCESS_DENIED`
- `SIM_FINDER_REFUSE_ACCESS_AP_REQUIRED`
- `SIM_FINDER_REFUSE_UNSAFE_REQUEST`
- `SIM_FINDER_REFUSE_POLICY_BLOCKED`
- `SIM_FINDER_SIMULATION_INACTIVE`
- `SIM_FINDER_REPLAY_ARTIFACT_MISSING`
- `SIM_FINDER_ABSTAIN_LOW_CALIBRATED_CONFIDENCE`
- `SIM_FINDER_MISSING_SIMULATION`
- `SIM_FINDER_MISSING_SIMULATION_DECLINED_LOW_VALUE_HIGH_RISK`

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
- `entropy_score_bp(f) = 0.5 * domain_cardinality_bp(f) + 0.3 * candidate_split_bp(f) + 0.2 * downstream_risk_bp(f)`

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
2. PH1.X confirmation path (unchanged)
3. Simulation executor enforces access + active simulation guards (unchanged)

If catalog status is not `Active`, finder does not execute; it emits `MissingSimulationPacket` or `RefusePacket` based on policy.

### 7.1 Deterministic “No Simulation Exists” Proof Order (Mandatory)

Before emitting `MissingSimulationPacket`, finder must run this exact deterministic check order:
1. Check `simulation_catalog_current` for candidate with `status=Active`.
2. If none, check same candidate family with `status=Draft`.
3. If `Draft` exists, return deterministic fail-closed message (`not active yet`) and optionally create intake only as `link-to-existing-draft` (no duplicate capability request).
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

When no match exists after clarification:
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

Rate limits (fail closed when exceeded):
- Per user: max `3` new Dev Intake tickets per rolling `1 hour`.
- Per tenant: max `20` new Dev Intake tickets per rolling `1 hour`.
- Daily hard cap: max `100` new Dev Intake tickets per tenant per `24 hours`.

Dedupe rules:
- Primary dedupe key: `(tenant_id, user_id, dedupe_fingerprint)` over `24 hours`.
- Secondary dedupe key: `(tenant_id, dedupe_fingerprint)` over `24 hours` to suppress cross-user duplicates for same capability request.
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
- Missing-sim intake: `(tenant_id, dedupe_fingerprint, idempotency_key)`
- Gold mapping update: `(tenant_id, utterance_fingerprint, simulation_id, idempotency_key)`

Replay rule:
- `*_current` must rebuild from `*_ledger` only.
- No silent transitions; append-only enforced.

## 10) Simulation worthiness scoring (before forward)

Input components (all basis points `0..10000`):
- `frequency_bp`: normalized ask frequency for equivalent dedupe fingerprint window.
- `value_bp`: estimated business impact/time saved class.
- `feasibility_bp`: integration/runtime feasibility estimate.
- `scope_bp`: tenant-only vs multi-tenant/generalizability score.
- `risk_bp`: money/permission/external-send/safety risk estimate.

Deterministic formula (fixed):
- `worthiness_raw_bp = (30*frequency_bp + 30*value_bp + 20*feasibility_bp + 20*scope_bp) / 100`
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
