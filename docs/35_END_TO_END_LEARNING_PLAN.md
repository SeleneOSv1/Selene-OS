# End-to-End Learning Plan (Gold Output -> Builder -> Runtime Consumption)

## 0) Purpose
- Define one complete, execution-grade learning system for Selene: what to learn, what to capture, how "gold output" is created, where it is stored, how it is approved, and how it is consumed safely by runtime engines.
- Ensure learning outputs do not stop at reporting; they must become governed, measurable improvements in live behavior.

## 1) Scope
- In scope:
  - Speech input quality (wake, endpointing, STT, pronunciation, vocabulary)
  - Speech output quality (TTS routing, pronunciation, interruption handling)
  - Understanding quality (SRL, NLP parse/field extraction, clarify quality)
  - LLM boundary quality (PH1.D schema correctness, response reliability)
  - Context/memory retrieval quality (PH1.CONTEXT/PH1.CACHE/PH1.M)
  - Routing quality (PH1.PAE policy/ranking)
  - Tone/style delivery quality (PH1.PERSONA, PH1.EMO.*)
- Out of scope:
  - Any authority or permission mutation
  - Any bypass of Access + Simulation order
  - In-turn runtime self-modification

## 2) Hard Rules (Non-Negotiable)
- Engines never call engines directly; Selene OS orchestrates.
- Learning outputs are advisory only until governance + Builder + release gates pass.
- No learning artifact can grant authority, change gate order, or execute side effects.
- Tenant/user isolation and consent boundaries are mandatory.
- Offline-only engines (`PH1.PATTERN`, `PH1.RLL`) never run in-turn.
- "No evidence -> no gold -> no activation."

## 3) What Selene Can Learn (Full Inventory)

| Learning Domain | Producer Engines | Gold Unit | Consumer Engines | Primary KPI |
|---|---|---|---|---|
| Wake/endpoint boundaries | `PH1.K`, `PH1.W`, `PH1.ENDPOINT`, `PH1.LISTEN` | Labeled boundary segments and trigger outcomes | `PH1.W`, `PH1.C` | False wake/clip rate |
| STT lexical accuracy | `PH1.C`, `PH1.FEEDBACK` | Corrected transcript with span-level edits | `PH1.C`, `PH1.SRL`, `PH1.NLP` | WER/CER, reject rate |
| Vocabulary and names | `PH1.KNOW`, `PH1.FEEDBACK` | Approved tenant/user term entries | `PH1.C`, `PH1.SRL`, `PH1.NLP` | Proper-noun accuracy |
| Pronunciation output/input robustness | `PH1.PRON`, `PH1.KNOW`, `PH1.FEEDBACK` | Pronunciation lexicon entries | `PH1.TTS`, `PH1.W`, `PH1.VOICE.ID` | Mispronunciation rate |
| TTS route quality and interruption behavior | `PH1.TTS`, `PH1.K`, `PH1.X`, `PH1.FEEDBACK` | Route/voice policy outcomes with interrupt labels | `PH1.TTS`, `PH1.PAE` | Barge-in success, playback defects |
| Semantic repair and parsing | `PH1.SRL`, `PH1.NLP`, `PH1.FEEDBACK` | Correct intent + fields + clarify target | `PH1.NLP`, `PH1.PRUNE`, `PH1.X` | Intent/slot F1, clarify loops |
| LLM boundary quality | `PH1.D`, `PH1.FEEDBACK`, `PH1.DIAG` | Schema-valid, policy-safe response decision packet | `PH1.D`, `PH1.X`, `PH1.WRITE` | Schema pass rate, policy violation rate |
| Context retrieval quality | `PH1.M`, `PH1.CONTEXT`, `PH1.CACHE`, `PH1.FEEDBACK` | Ranked context bundle judgment set | `PH1.CONTEXT`, `PH1.CACHE`, `PH1.NLP`, `PH1.X` | Context precision/recall |
| Provider/routing arbitration | `PH1.PAE`, `PH1.LEARN`, `PH1.FEEDBACK`, `PH1.LISTEN` | Ranked route policy outcome set | `PH1.PAE`, `PH1.C`, `PH1.TTS` | Latency-quality-cost score |
| Persona/tone delivery | `PH1.PERSONA`, `PH1.EMO.GUIDE`, `PH1.EMO.CORE`, `PH1.FEEDBACK` | Style/tone preference adjudications | `PH1.PERSONA`, `PH1.X`, `PH1.TTS` | Tone satisfaction and correction rate |
| Tool/search evidence quality | `PH1.E`, `PH1.SEARCH`, `PH1.FEEDBACK` | Evidence ranking correctness set | `PH1.SEARCH`, `PH1.CONTEXT`, `PH1.X` | Evidence conflict/error rate |
| Multimodal fusion quality | `PH1.DOC`, `PH1.VISION`, `PH1.MULTI`, `PH1.FEEDBACK` | Cross-modal alignment judgments | `PH1.MULTI`, `PH1.CONTEXT`, `PH1.NLP` | Alignment error rate |

## 4) Canonical Capture Contract (Every Learning Signal)
- Required fields for all captured learning signals:
  - `signal_id`, `correlation_id`, `turn_id`, `occurred_at`
  - `tenant_id`, `user_id` (or anonymous policy-safe surrogate), `device_id`, `session_id`
  - `source_engine_id`, `source_capability_id`, `reason_code`
  - `input_ref` (audio/text/evidence refs), `output_ref`, `outcome_label`
  - `consent_asserted`, `privacy_scope`, `retention_class`
  - `severity`, `confidence`, `impact_estimate`
  - `idempotency_key`
- Capture sources:
  - Runtime outcome telemetry (rejects, retries, fallbacks, interrupts)
  - Explicit user corrections (text correction, pronunciation correction, "you heard me wrong")
  - Explicit ratings (thumbs up/down, "this was wrong/right")
  - Offline replay/evaluation outcomes
  - Deterministic truth events (simulation and audited business outcomes)

## 5) Gold Output Definition

### 5.1 Gold Confidence Tiers
- `GOLD_TIER_1_HARD_TRUTH`:
  - Deterministic external truth or verified source-of-record.
  - Example: approved canonical name from authoritative tenant source, deterministic execution truth.
- `GOLD_TIER_2_HUMAN_VERIFIED`:
  - Human correction/review with reviewer identity + audit trail.
- `GOLD_TIER_3_CONSENSUS`:
  - Independent-system agreement above strict threshold with conflict checks.
  - Allowed for lower-risk ranking/tuning only, not safety-critical truth.
- `NON_GOLD_PROXY`:
  - Behavioral proxies (latency, abandon rate, retries). Used for prioritization, not direct training targets.

### 5.2 Gold Eligibility Rules by Domain
- STT/transcript gold:
  - Requires `TIER_1` or `TIER_2` for span corrections.
  - `TIER_3` can tune routing/ranking, not canonical transcript truth.
- TTS gold:
  - Requires human-verified pronunciation/prosody outcome (`TIER_2`) or deterministic route/perf truth (`TIER_1`).
- NLP/LLM gold:
  - Requires schema-valid structured target and evidence-backed expected outcome.
  - Single-model output alone is never gold.
- Context/memory gold:
  - Requires explicit relevance judgment with provenance and privacy-safe references.

## 6) Gold Generation Pipeline (End-to-End)
1. Intake:
   - PH1.FEEDBACK ingests post-turn signals and emits validated bundles.
2. Candidate build:
   - PH1.LEARN aggregates candidate improvements and assembles candidate sets.
3. Validation:
   - Contract validation, privacy checks, consent checks, tenant-scope checks.
4. Adjudication:
   - Assign candidate to hard-truth, human-review, or consensus workflow.
5. Gold set creation:
   - Output immutable `gold_set` artifact with:
     - `gold_set_id`, `domain`, `tier`, `version`, `hash`
     - reviewer or rule provenance
     - evidence references
6. Artifact packaging:
   - Convert approved gold sets into engine-targeted learning packs.
7. Governance decision:
   - `PH1.GOV` evaluates activation/deprecation/rollback request.
8. Builder bridge:
   - Builder emits structured learning report and passes learning bridge gate.
9. Human approvals and release gates:
   - Code approval, launch approval, stage gates, hard release gate.
10. Runtime activation:
   - Active packs are loaded by target engines through Selene OS orchestration.
11. Post-deploy judge:
   - Measure KPI deltas; auto-revert when fail thresholds are breached.

## 7) Storage, Versioning, and Audit Trail
- Signal persistence:
  - Feedback and outcome events in append-only `audit_events` path.
- Artifact persistence:
  - Learning/dictionary/artifact rows in append-only `artifacts_ledger`.
- Gold artifact storage model (required extension):
  - Store each gold set as immutable payload + provenance refs.
  - Register via `artifact_id`, `artifact_type`, `artifact_version`, `package_hash`, `payload_ref`, `provenance_ref`, `created_by`.
- Versioning policy:
  - Monotonic versions; no in-place overwrite.
  - Rollback pointer required for every activation candidate.
- Audit requirements:
  - Every transition (`captured`, `validated`, `golded`, `packaged`, `approved`, `activated`, `reverted`) must emit reason-coded audit events.

## 8) Two Intake Paths (Both Mandatory)

### Path A: Broken Behavior (Defect/Fault)
- Definition:
  - Crashes, contract violations, gate-order drift, unsafe output, deterministic wrong behavior.
- SLA:
  - Immediate triage and block from promotion if safety-critical.
- Flow:
  1. Intake + severity assignment
  2. Root-cause evidence bundle
  3. Minimal deterministic fix plan
  4. Regression test addition
  5. Builder report + approvals + release gates
  6. Closure with proof links

### Path B: Improvement Feedback (Not Broken, Better Quality)
- Definition:
  - Behavior is acceptable but can improve accuracy, naturalness, latency, or consistency.
- SLA:
  - Prioritized by impact and confidence score.
- Flow:
  1. Intake improvement signal
  2. Build candidate gold set
  3. Human/consensus adjudication
  4. Package generation
  5. Controlled activation and A/B or canary measurement
  6. Promote/rollback based on judge thresholds

## 9) Prioritization and Assignment
- Priority score:
  - `priority = impact * frequency * confidence * safety_weight`.
- Assignment owners:
  - Intake owner: `PH1.FEEDBACK` + Selene OS triage layer
  - Gold adjudication owner: domain owner (`PH1.C`, `PH1.TTS`, `PH1.NLP`, `PH1.D`, etc.)
  - Packaging owner: `PH1.LEARN` / `PH1.KNOW`
  - Offline optimization owner: `PH1.PATTERN` -> `PH1.RLL`
  - Activation owner: `PH1.GOV` + Builder release controller
  - Runtime consumption owner: target consumer engine + Selene OS wiring
- Closure owner:
  - Builder judge pipeline with KPI evidence and rollback outcome.

## 10) How Gold Output Is Actually Consumed (No "Output But Nothing Happens")
- Gold output is not self-applying. It is consumed only through this strict chain:
  1. Gold set -> learning pack candidate
  2. Governance approval (`PH1.GOV`)
  3. Builder learning-bridge pass (`scripts/check_builder_learning_bridge_gate.sh`)
  4. Human permission gates (code + launch)
  5. Release hard gate and staged rollout
  6. Runtime engines load ACTIVE artifacts only
- If any gate fails, consumption stops by design (fail closed).
- This is intentional: it prevents unreviewed learning from changing production behavior.

## 11) Runtime Consumption Matrix

| Artifact/Payload | Current Consumer Path | Status |
|---|---|---|
| `STT_ADAPTATION_PROFILE` | Selene OS -> `PH1.C` | Active design path |
| `STT_ROUTING_POLICY_PACK` | Selene OS -> `PH1.C` / `PH1.PAE` hints | Active design path |
| `STT_VOCAB_PACK` | `PH1.KNOW` -> Selene OS -> `PH1.C`/`PH1.SRL`/`PH1.NLP` | Active design path |
| `TTS_PRONUNCIATION_PACK` | `PH1.KNOW`/`PH1.PRON` -> Selene OS -> `PH1.TTS` | Active design path |
| `TTS_ROUTING_POLICY_PACK` | Selene OS -> `PH1.TTS` / `PH1.PAE` hints | Active design path |
| Persona/tone profile deltas | `PH1.PERSONA`/`PH1.EMO.*` -> Selene OS -> `PH1.X`/`PH1.TTS` | Active design path |
| Context/cache ranking packs | Offline queue -> governance -> Selene OS -> `PH1.CACHE`/`PH1.CONTEXT` | Governed/offline path |
| NLP/LLM gold label packs | Planned extension for `PH1.NLP`/`PH1.D` | Add artifact type + consumer lock |

## 12) Builder and Gate Integration (Mandatory)
- Learning-triggered cycles must satisfy:
  - Structured report with evidence refs
  - Validated learning context fields (`report_id`, `source_engines`, `signal_count`, `evidence_refs`)
  - Learning bridge gate pass
  - Human approval gates pass
  - Stage/release hard gates pass
- Canonical operational checks:
  - `scripts/check_builder_learning_bridge_gate.sh`
  - `scripts/check_builder_human_permission_gate.sh code`
  - `scripts/check_builder_human_permission_gate.sh launch`
  - `scripts/check_builder_release_hard_gate.sh`

## 13) Effectiveness Measurement
- Speech input:
  - WER/CER, reject rate, first-pass transcript success, wake false-positive/false-negative.
- Speech output:
  - mispronunciation rate, interruption correctness, playback failure rate.
- Understanding:
  - intent accuracy, slot accuracy, clarify-turn count, confirm abort rate.
- LLM boundary:
  - schema validity rate, refusal correctness, groundedness error rate.
- Context/memory:
  - relevant-context hit rate, stale-context incidence, correction recurrence.
- Release safety:
  - gate-order violations, duplicate side effects, rollback frequency.
- Hard metric rule:
  - no promotion on safety regression even if quality improves.

## 14) Implementation Plan (Execution Order)
1. Phase L0: Contract lock
   - Freeze canonical learning signal schema and gold tier definitions.
   - Acceptance: all learning producers emit schema-valid signals.
2. Phase L1: Gold pipeline lock
   - Implement adjudication workflows and immutable `gold_set` registration.
   - Acceptance: every gold item has provenance and hash.
3. Phase L2: Artifact expansion
   - Extend artifact-type catalog for remaining domains (NLP/LLM/context as needed).
   - Acceptance: single-writer ownership and consumer mapping are explicit.
4. Phase L3: Consumption wiring completeness
   - Ensure each ACTIVE artifact has a real runtime consumer path and fail-closed behavior.
   - Acceptance: no "orphan artifact" with zero runtime consumption.
5. Phase L4: Builder automation hardening
   - Enforce bridge + permission + release gates in one standard flow.
   - Acceptance: learning-triggered runs fail closed when any evidence/gate is missing.
6. Phase L5: Continuous judge and rollback
   - Run post-deploy judge and automatic rollback thresholds per domain.
   - Acceptance: bad promotions are reverted deterministically with audit proof.

## 15) Definition of Done (This Plan)
- Every learnable domain has:
  - defined gold unit
  - defined capture fields
  - defined storage and versioning path
  - defined packaging owner
  - defined governance/approval path
  - defined runtime consumer
  - defined KPI and rollback rule
- The deterministic Gold Policy Matrix is locked and versioned.
- The Gold State Machine is enforced with reason-coded transitions only.
- Consumption Proof Gate evidence is required before any `ACTIVE` status.
- Judge threshold table is wired to promotion/rollback automation.
- Consent revocation pipeline is implemented and replay-tested.
- No learning output can exist without either:
  - becoming an approved and consumed artifact, or
  - being explicitly closed as rejected/insufficient with reason code.

## 16) Gold Policy Matrix (Deterministic)

| Domain | Gold Tier Allowed | Minimum Sample Window | Gold Entry Criteria | Conflict Rule | Output Artifact Type |
|---|---|---|---|---|---|
| Wake/Endpoint | `TIER_1`, `TIER_2` | >= 300 labeled boundary events | boundary overlap >= 0.92 IoU and trigger label agreement >= 0.98 | any disagreement on start/end > 120 ms => `GOLD_DEFERRED` | `GOLD_SET_WAKE_ENDPOINT_V1` |
| STT Transcript | `TIER_1`, `TIER_2` (`TIER_3` tuning only) | >= 500 corrected utterances | span edit agreement >= 0.97 and unresolved token rate <= 0.01 | disagreement on entity token => escalate to human review | `GOLD_SET_STT_TRANSCRIPT_V1` |
| TTS Pronunciation | `TIER_2` plus `TIER_1` route truth | >= 250 pronunciation judgments | accepted pronunciation rate >= 0.95 and reviewer confidence >= 0.90 | if reviewer disagreement >= 0.10, block golding | `GOLD_SET_TTS_PRONUNCIATION_V1` |
| NLP Intent/Slots | `TIER_1`, `TIER_2` | >= 400 labeled turns | intent agreement >= 0.98 and required-slot agreement >= 0.97 | any required-slot conflict => `GOLD_REJECTED` | `GOLD_SET_NLP_INTENT_SLOT_V1` |
| LLM Boundary | `TIER_1`, `TIER_2` | >= 400 validated outputs | schema-valid rate = 1.0 for candidate set and policy conflict rate = 0 | any schema violation => reject candidate set | `GOLD_SET_D_BOUNDARY_V1` |
| Context Ranking | `TIER_2`, `TIER_3` | >= 300 ranked judgments | top-3 relevance precision >= 0.90 and leakage rate = 0 | any privacy leakage => immediate reject | `GOLD_SET_CONTEXT_RANK_V1` |
| PAE Routing | `TIER_1`, `TIER_3` | >= 800 routed turns | weighted score lift >= 0.05 vs baseline and rollback pointer present | missing rollback pointer => `GOLD_REJECTED` | `GOLD_SET_PAE_ROUTE_V1` |

Hard rules:
- `TIER_3` consensus requires at least 3 independent evaluators with agreement >= 0.90.
- Safety-critical domains (`STT`, `NLP`, `LLM boundary`) cannot promote from `TIER_3` alone.
- Missing minimum sample window forces `GOLD_DEFERRED`.

## 17) Gold State Machine (Fail-Closed)

States:
- `CANDIDATE`
- `VALIDATING`
- `AWAITING_REVIEW`
- `GOLD_APPROVED`
- `GOLD_DEFERRED`
- `GOLD_REJECTED`
- `PACKAGED`
- `ACTIVATED`
- `REVERTED`

Transitions:
1. `CANDIDATE -> VALIDATING`
   - guard: schema-complete input bundle present
   - fail reason: `GOLD_INPUT_SCHEMA_INVALID`
2. `VALIDATING -> AWAITING_REVIEW`
   - guard: policy/consent/privacy checks pass
   - fail reason: `GOLD_POLICY_SCOPE_VIOLATION`
3. `AWAITING_REVIEW -> GOLD_APPROVED`
   - guard: domain matrix thresholds met
   - fail reason: `GOLD_THRESHOLD_NOT_MET`
4. `AWAITING_REVIEW -> GOLD_DEFERRED`
   - guard: insufficient sample or unresolved conflicts
   - fail reason: `GOLD_INSUFFICIENT_EVIDENCE`
5. `AWAITING_REVIEW -> GOLD_REJECTED`
   - guard: hard boundary violation detected
   - fail reason: `GOLD_HARD_RULE_VIOLATION`
6. `GOLD_APPROVED -> PACKAGED`
   - guard: packaging validation `OK` with rollback pointer
   - fail reason: `GOLD_PACKAGE_BUILD_FAILED`
7. `PACKAGED -> ACTIVATED`
   - guard: governance + Builder + approval + release gates pass
   - fail reason: `GOLD_ACTIVATION_GATE_BLOCKED`
8. `ACTIVATED -> REVERTED`
   - guard: judge rollback trigger fired
   - reason: `GOLD_POST_DEPLOY_REGRESSION`

Hard rule:
- Any unknown transition is rejected with `GOLD_STATE_TRANSITION_INVALID`.

## 18) Storage Contract Addendum (Gold Sets)

Ownership:
- `PH1.F` owns schema, migrations, indices, and storage invariants.
- `PH1.LEARN` is single writer for `GOLD_SET_*` artifacts.
- `PH1.KNOW` remains single writer for `STT_VOCAB_PACK` and `TTS_PRONUNCIATION_PACK`.

Persistence model:
- Gold sets are persisted in append-only `artifacts_ledger`.
- Required fields:
  - `artifact_type` (`GOLD_SET_*`)
  - `artifact_version`
  - `scope_type`, `scope_id`
  - `package_hash`
  - `payload_ref`
  - `provenance_ref`
  - `created_by=PH1.LEARN`
  - `idempotency_key`

Idempotency and uniqueness:
- idempotency key formula:
  - `hash(scope_type + scope_id + domain + policy_version + sorted_signal_ids + artifact_version)`
- unique index:
  - `(scope_type, scope_id, artifact_type, artifact_version)`
- idempotency dedupe index:
  - `(scope_type, scope_id, artifact_type, artifact_version, idempotency_key)`

Migration sequence:
1. `MIG-LRN-01`: register `GOLD_SET_*` artifact types and validator allow-list.
2. `MIG-LRN-02`: add/verify unique + idempotency indexes for gold artifact paths.
3. `MIG-LRN-03`: add replay-safe read API for gold artifact retrieval by consumer engines.

Hard rule:
- Gold artifacts are append-only; mutation or delete is forbidden.

## 19) Consumption Proof Gate (No Orphan Artifact)

Required proof packet before `ACTIVE`:
- `artifact_id`, `artifact_type`, `artifact_version`
- `consumer_engine_id`, `consumer_capability_id`
- `apply_plan_hash`
- `replay_validation_hash`
- `shadow_eval_result_ref`
- `rollback_pointer_ref`
- `gate_bundle_ref` (governance + Builder + human approvals + release gate)

Gate pass criteria:
1. Consumer engine parser accepts artifact with `validation_status=OK`.
2. Deterministic replay hash matches expected baseline class.
3. Shadow evaluation has no safety regression and no privacy leakage.
4. Rollback pointer is present and resolvable.
5. Required gate bundle is complete and signed.

Block reasons:
- `CONSUMPTION_PROOF_MISSING`
- `CONSUMPTION_REPLAY_MISMATCH`
- `CONSUMPTION_SHADOW_REGRESSION`
- `CONSUMPTION_ROLLBACK_POINTER_MISSING`
- `CONSUMPTION_GATE_BUNDLE_INCOMPLETE`

## 20) Judge Threshold Table (Promote/Hold/Rollback)

| Domain | Minimum Evaluation Window | Promote | Hold | Rollback |
|---|---|---|---|---|
| STT | >= 1000 utterances and >= 24h | WER relative improvement >= 8%, reject-rate delta <= +0.3 pp | WER improvement 2%-8% and no safety regression | WER regression >= 3% or reject-rate delta >= +1.0 pp |
| TTS | >= 500 playback turns and >= 24h | mispronunciation complaints down >= 20%, interruption success >= 5 pp | complaints down < 20% with neutral safety | complaints up >= 10% or interruption false-positive >= 2 pp |
| NLP | >= 800 turns and >= 24h | intent F1 >= 4 pp, slot F1 >= 3 pp | F1 gains below promote threshold, neutral safety | intent or slot F1 down >= 2 pp |
| LLM Boundary | >= 800 turns and >= 24h | schema-valid rate >= 99.9%, policy conflicts = 0 | schema-valid 99.5%-99.9% with zero safety events | schema-valid < 99.5% or any policy conflict |
| Context | >= 600 turns and >= 24h | context correction recurrence down >= 15% | 5%-15% improvement with no leakage | any leakage or recurrence up >= 5% |
| PAE Routing | >= 1200 turns and >= 24h | weighted quality-cost score >= 5% and p95 latency delta <= +5% | score gain < 5%, neutral safety | score down >= 3% or p95 latency delta >= +15% |

Global immediate rollback triggers:
- any authority/gate-order violation
- any duplicate side-effect incident
- any privacy leak
- audit completeness below 100% for governed activation trail

## 21) Consent Revocation Pipeline (Deterministic Purge/Rebuild)

Trigger sources:
- explicit user revoke/forget command
- tenant policy revocation
- consent TTL expiry

Revocation sequence:
1. `REVOKE_CAPTURE`
   - record revoke event with scope and timestamp.
2. `REVOKE_FREEZE`
   - freeze affected `CANDIDATE`, `GOLD_APPROVED`, and `PACKAGED` artifacts from new activation.
3. `REVOKE_IMPACT_GRAPH`
   - resolve all dependent gold sets and downstream learning packs by provenance refs.
4. `REVOKE_INVALIDATE`
   - mark affected artifacts `REVOKED_PENDING_REBUILD` (append-only status event).
5. `REVOKE_REBUILD`
   - rebuild gold sets and packages excluding revoked signals.
6. `REVOKE_GOV_ROLLBACK`
   - rollback active artifacts that depend on revoked consent.
7. `REVOKE_REACTIVATE`
   - activate rebuilt artifacts only after normal governance + Builder + approval gates.

Revocation SLA:
- freeze within 15 minutes of revoke capture.
- rollback decision emitted within 60 minutes.

Revocation reason codes:
- `CONSENT_REVOKED_USER_REQUEST`
- `CONSENT_REVOKED_TENANT_POLICY`
- `CONSENT_EXPIRED_TTL`
- `CONSENT_REVOKE_REBUILD_REQUIRED`
- `CONSENT_REVOKE_ROLLBACK_APPLIED`

## 22) Multi-Provider Runtime Architecture (Execution Extension)

Goal:
- Run one governed routing layer in Selene OS while keeping provider integrations isolated, replaceable, and fail-closed.

Hard architecture:
- Selene OS owns policy/budget/quota gates, routing-profile resolution, health telemetry, failover policy, and audit.
- Provider adapters own vendor API integration details and normalization into Selene contracts.
- PH1.KMS owns secret lifecycle and returns opaque handles/ephemeral refs only.
- Engines must never embed raw API keys or call env secrets directly.
- Final provider selection must execute inside domain routing-owner runtimes (while preserving existing engine authority classes):
  - STT routing-owner: `PH1.C` (authoritative transcript gate)
  - TTS routing-owner: `PH1.TTS` (authoritative playback)
  - NLP/LLM routing-owner: `PH1.D` (non-authoritative boundary owner)
  - Delivery routing-owner: `PH1.DELIVERY` (authoritative delivery attempt truth)

Deterministic invocation chain (must be preserved):
1. `PH1.POLICY` gate
2. `PH1.COST` gate (when enabled)
3. `PH1.QUOTA` gate (when enabled)
4. Routing profile resolution in Selene OS
5. Final provider selection in routing-owner engine runtime
6. Provider adapter call
7. Provider-neutral normalization
8. Engine contract validation
9. PH1.J audit emission

Router layers (split for isolation):
- `SttRouter` in `PH1.C`: STT providers only.
- `TtsRouter` in `PH1.TTS`: TTS providers only.
- `LlmRouter` in `PH1.D`: NLP/LLM providers only.
- `DeliveryRouter` in `PH1.DELIVERY`: SMS/Email/WhatsApp/WeChat/App push providers.

Engine relationship wiring:
- Voice path authority:
  - all voice-turn branches must follow the single authoritative branch table in `Section 35.1` (this section, `Section 35`, and `Section 41` must remain identical in behavior).
- Voice branch summary:
  - `BR-01` explicit-trigger path (iOS default; also available on Android/desktop manual trigger).
  - `BR-02` wake-trigger path (Android/desktop default; iOS feature-flag override only).
  - `BR-03` endpoint assist overlay (`TURN_OPTIONAL`) that augments `BR-01` or `BR-02` under deterministic policy/noise triggers.
- Delivery side-effect path:
  - `PH1.X (simulation commit) -> PH1.ACCESS -> PH1.BCAST -> PH1.DELIVERY (DeliveryRouter) -> provider adapter -> status/proof -> PH1.BCAST state update`
- Learning optimization path (async only):
  - `PH1.KNOW -> PH1.PAE -> governed routing artifacts`
- Hard guard:
  - Performance/parallelization optimizations must not bypass or reorder ALWAYS_ON gates (`PH1.CONTEXT`, `PH1.POLICY`) in the canonical turn sequence.

Provider adapter ownership:
- OpenAI adapters: STT, TTS, LLM/NLP.
- Google adapters: STT, TTS.
- LLaMA adapter: LLM/NLP text boundary only (not direct STT/TTS replacement unless local speech models are installed).
- Delivery adapters: SMS, Email, WhatsApp, WeChat.
- Selene Local adapters (future): STT, TTS, LLM/NLP.

## 23) Provider-Neutral Adapter Contract

All adapters must return a provider-neutral result envelope:
- `ProviderResultV1` required fields:
  - `provider_ref`, `capability` (`STT|TTS|LLM|DELIVERY_SEND|DELIVERY_STATUS|DELIVERY_CANCEL`)
  - `request_id`, `idempotency_key`, `simulation_context_ref`
  - `status` (`OK|RETRYABLE_FAIL|TERMINAL_FAIL`)
  - `reason_code`, `latency_ms`, `quality_score`, `raw_ref`
  - `normalized_payload_ref`
- Hard rules:
  - unknown/invalid adapter output => fail closed.
  - router decisions consume only `ProviderResultV1`, never vendor-specific payload shape.
  - adapter must never expose raw secret values in payloads/logs/audit.

## 24) Default Routing and Automatic Changeover Policy (Numeric)

Routing profile overlay order (base -> most specific):
1. `global_default`
2. `tenant_override`
3. `locale_override`
4. `channel_override`
5. `user_exception`

Routing profile precedence (highest wins):
1. `user_exception`
2. `channel_override`
3. `locale_override`
4. `tenant_override`
5. `global_default`

Precedence rules:
- If a field is unset at a higher precedence layer, resolve from the next lower layer.
- Circuit and failover thresholds are evaluated per active resolved profile, not globally.
- Cross-region or cross-locale fallback is forbidden unless policy explicitly allows it.

Base thresholds table (applies after precedence resolution):

| Domain | Primary | Secondary | Tertiary | Open-Circuit Rule | Min Cooldown | Half-Open Probe | Return-to-Primary Rule |
|---|---|---|---|---|---|---|---|
| STT | OpenAI STT | Google STT | Selene Local STT | any: timeout_rate_60s >= 8%; error_rate_60s >= 5%; p95_latency_5m > 1800 ms for 3 windows; transcript_accept_rate_200 < 0.82 | 120 s | 5 probe calls | 2 consecutive windows: error_rate_60s < 2%, p95_latency_5m < 1200 ms, transcript_accept_rate_200 >= 0.88 |
| TTS | OpenAI TTS | Google TTS | Selene Local TTS | any: synth_fail_rate_60s >= 5%; p95_latency_5m > 2200 ms for 3 windows; complaint_rate_500 >= 10% | 120 s | 5 probe calls | 2 consecutive windows: synth_fail_rate_60s < 2%, p95_latency_5m < 1500 ms, complaint_rate_500 < 5% |
| NLP/LLM | OpenAI | LLaMA | Selene Local LLM | any: schema_invalid_rate_200 >= 0.5%; policy_conflict_count_200 > 0; p95_latency_5m > 3000 ms for 3 windows | 180 s | 5 probe calls | 2 consecutive windows: schema_invalid_rate_200 = 0, policy_conflict_count_200 = 0, p95_latency_5m < 2000 ms |
| SMS | Provider A | Provider B | none | send_fail_rate_100 >= 5% or auth_fail event | 300 s | 3 probe sends | 2 windows: send_fail_rate_100 < 2% and no auth fail |
| Email | Provider A | Provider B | none | send_fail_rate_100 >= 5% or auth_fail event | 300 s | 3 probe sends | 2 windows: send_fail_rate_100 < 2% and no auth fail |
| WhatsApp | Provider A | Provider B | none | send_fail_rate_100 >= 5% or policy/region hard-block | 300 s | 3 probe sends | 2 windows: send_fail_rate_100 < 2% and policy/region healthy |
| WeChat | Provider A | Provider B | none | send_fail_rate_100 >= 5% or policy/region hard-block | 300 s | 3 probe sends | 2 windows: send_fail_rate_100 < 2% and policy/region healthy |

Anti-flap hysteresis:
- each circuit-open period is sticky for minimum cooldown.
- repeated trips backoff by 2x up to 900 s.
- failover and return events must emit deterministic reason-coded PH1.J audit rows.

## 25) Delivery Exactly-Once and Fallback Safety

Global idempotency key formula:
- `delivery_idem = hash(tenant_id + message_id + recipient_id + channel + payload_hash + simulation_id)`

Exactly-once guardrails:
1. Acquire per-recipient delivery lock before provider send.
2. If provider response is terminal success (`provider_ack_ref` present), mark final and block fallback send.
3. If response is retryable/unknown and no terminal ack, allow fallback provider under same `delivery_idem`.
4. Persist all attempts under same `delivery_attempt_group_id`.
5. Reconciliation poll resolves unknowns before retry budget is exhausted.

Mandatory provider-idempotency mapping:
- Every delivery adapter must map internal `delivery_idem` to provider-native idempotency token/header/field.
- Every attempt must persist:
  - `provider_idempotency_token_sent`
  - `provider_request_id`
  - `provider_message_id` (when provider returns one)
  - `provider_ack_ref`
- Reconciliation must key on `provider_request_id` and `provider_message_id` before any fallback send.

Hard rule:
- No secondary provider send is allowed after any terminal success acknowledgment for the same `delivery_idem`.

## 26) Secrets and Key Lifecycle (One-Time Plug + Ongoing Rotation)

Objective:
- Plug keys once per environment and run handle-only runtime access forever.

Initial setup per environment (`dev`, `stage`, `prod`):
1. Register provider secrets in KMS backend.
2. Issue stable `secret_ref` and runtime `kms_handle_ref` mappings.
3. Store handle refs only in routing config.
4. Validate access via `PH1.KMS` access-evaluate/material-issue checks.
5. Run redaction test proving no secret value in runtime/audit logs.

Ongoing lifecycle policy:
- rotation interval:
  - prod: every 30 days
  - non-prod: every 7 days
- emergency revoke SLA:
  - revoke initiated <= 15 minutes after incident
  - replacement handle active <= 60 minutes
- ephemeral credentials:
  - default TTL 5 minutes
  - max TTL 60 minutes
- quarterly key-rotation + revoke drill is mandatory.

Required handle classes:
- `kms://providers/openai/stt`
- `kms://providers/openai/tts`
- `kms://providers/openai/llm`
- `kms://providers/google/stt`
- `kms://providers/google/tts`
- `kms://providers/llama/endpoint`
- `kms://delivery/sms/*`
- `kms://delivery/email/*`
- `kms://delivery/whatsapp/*`
- `kms://delivery/wechat/*`

Hard rules:
- no raw key in source files.
- no raw key in committed `.env` artifacts.
- no raw key in runtime outputs, telemetry, or PH1.J audit payloads.

## 27) Central Routing Config Contract (Single Source of Truth)

Design decision:
- One central routing profile artifact owned by Selene OS.
- Provider adapters stay separated by domain/channel implementation.

`provider_routing_profile_v1` required fields:
- `schema_version`, `profile_id`, `profile_version`, `effective_from`
- `precedence_order` (must be `user_exception > channel_override > locale_override > tenant_override > global_default`)
- `selection_owner_engine`:
  - `stt=PH1.C`
  - `tts=PH1.TTS`
  - `llm=PH1.D`
  - `delivery=PH1.DELIVERY`
- domain sections:
  - `stt.providers[]`
  - `tts.providers[]`
  - `llm.providers[]`
  - `delivery.sms.providers[]`
  - `delivery.email.providers[]`
  - `delivery.whatsapp.providers[]`
  - `delivery.wechat.providers[]`
- scoped override sections:
  - `global_default`
  - `tenant_overrides[]`
  - `locale_overrides[]`
  - `channel_overrides[]`
  - `user_exceptions[]`
- per-provider fields:
  - `provider_ref`, `priority`, `enabled`, `kms_handle_ref`
  - thresholds (`timeout_ms`, `max_retries`, `error/open rules`)
  - region/tenant allow-rules
- governance fields:
  - `artifact_id`, `rollback_pointer_ref`, `judge_threshold_set_id`

Forbidden fields:
- raw API keys/tokens/passwords.
- unbounded free-form provider payload schemas.

## 28) Build Plan: Efficient Wiring and Execution Order

1. Phase R0: contract and gate-order lock
   - freeze router contracts, `ProviderResultV1`, and invocation order (`POLICY -> COST -> QUOTA -> Router`).
   - lock authority boundaries so final provider selection executes in `PH1.C`/`PH1.TTS`/`PH1.D`/`PH1.DELIVERY`.
2. Phase R1: router core + state isolation
   - implement `SttRouter`, `TtsRouter`, `LlmRouter`, `DeliveryRouter` with independent circuit state.
   - keep `PH1.ENDPOINT` assist path TURN_OPTIONAL; do not place it in mandatory fast path.
3. Phase R2: KMS integration and secret safety
   - wire handle-only access; add leak tests, rotation tests, revoke tests.
4. Phase R3: OpenAI primary path
   - implement OpenAI adapters (STT/TTS/LLM) with provider-neutral normalization.
5. Phase R4: Google speech fallback
   - implement Google STT/TTS adapters and validate automatic failover + recovery.
6. Phase R5: delivery channels
   - implement SMS/Email/WhatsApp/WeChat adapters with exactly-once guardrails.
7. Phase R6: LLaMA fallback
   - implement LLaMA LLM adapter; enforce schema/policy compatibility tests.
8. Phase R7: staged rollout automation
   - implement shadow/canary/full promotion logic with automatic rollback hooks.
9. Phase R8: Selene local bring-up
   - add Selene Local adapters and readiness gate for controlled routing share.
10. Phase R9: learning-driven optimization
   - use async governed routing artifacts from `PH1.KNOW -> PH1.PAE` (with optional `PH1.FEEDBACK/PH1.LEARN` upstream) to tune thresholds/weights.

## 29) Rollout, Gating, and Rollback Ownership

Rollout stages:

| Stage | Traffic Share | Entry Criteria | Exit Criteria |
|---|---|---|---|
| Shadow | 100% mirrored, 0% user-visible | contract tests + failover tests pass | no safety regression and replay hash match |
| Canary 1 | 5% | shadow pass >= 24h | SLOs within thresholds for >= 24h |
| Canary 2 | 25% | canary 1 pass | SLOs within thresholds for >= 24h |
| Full | 100% | canary 2 pass | continuous judge monitoring |

Control-loop cadence ownership:

| Control Loop | Owner | Cadence | Output |
|---|---|---|---|
| Circuit health evaluation | routing-owner runtime (`PH1.C`/`PH1.TTS`/`PH1.D`/`PH1.DELIVERY`) | every 60 s | `OPEN|HALF_OPEN|CLOSED` transitions with reason codes |
| Rollout promote check | Selene OS rollout controller + `PH1.GOV` | every 15 min | `PROMOTE|HOLD` decision |
| Rollback check | Selene OS incident watcher + `PH1.GOV` | continuous event-driven + 60 s heartbeat | `ROLLBACK|NO_ACTION` decision |
| Delivery reconciliation poll | `PH1.DELIVERY` | every 60 s | attempt resolution status updates |

Mandatory gates before activation:
- contract tests pass.
- failover simulations pass (primary down, secondary down, recovery).
- no-secret-leak tests pass.
- Builder learning bridge + release gates pass for learning-triggered updates.
- rollback pointer and replay proof are present.

Rollback ownership:
- `PH1.GOV`: approve/force rollback decision.
- Selene OS orchestration: execute rollback to previous routing profile.
- PH1.J: emit complete rollback audit chain.

Immediate rollback triggers:
- any policy/safety conflict.
- schema-valid rate below domain threshold.
- duplicate side-effect incident.
- audit completeness below 100%.

## 30) Performance Design Rules (Fast + Accurate + Smooth)

End-to-end turn SLO budgets (p95):

| Stage | Budget (ms) | Breach Action (Deterministic) |
|---|---|---|
| `PH1.K` capture/frame handoff | 20 | drop optional non-critical capture enrichments |
| `BR-01` explicit-trigger identity gate (`PH1.VOICE.ID`) | 90 | switch to stricter identity policy and continue in generic mode |
| `BR-02` wake-trigger identity gate (`PH1.W -> PH1.VOICE.ID`) | 120 | switch to stricter wake+identity policy and continue in generic mode |
| `BR-03` endpoint-assist overlay (`PH1.ENDPOINT`, when triggered) | +80 overlay | disable assist overlay and continue on `BR-01/BR-02` core path |
| `PH1.C` STT route + transcript gate | 900 | invoke provider fallback or fail closed with retry advice |
| `PH1.SRL` repair | 60 | bypass SRL enrichments and keep transcript authority unchanged |
| `PH1.NLP` parse | 90 | return bounded clarify path instead of deep parse |
| `PH1.D` boundary/validation | 500 | enforce concise boundary mode or fail closed on schema risk |
| `PH1.X` move selection | 40 | force deterministic safe waiting/clarify action |
| `PH1.WRITE` render shaping | 25 | degrade to minimal formatting mode |
| `PH1.TTS` route to first audio byte | 700 | fallback TTS provider, else text-first response |
| Total voice turn `BR-01` (`K` -> first audio byte) | 2425 | apply progressive degrade policy and emit SLO breach audit |
| Total voice turn `BR-02` (`K` -> first audio byte) | 2455 | apply progressive degrade policy and emit SLO breach audit |
| Total voice turn `BR-03` (assist-triggered) | `BR-01/BR-02 total + up to 80` | disable assist overlay first; preserve core branch order |

Latency rules:
- route-selection budget <= 20 ms.
- adapter normalization budget <= 10 ms.
- routing must degrade to deterministic fallback without blocking turn progression.

Throughput rules:
- maintain persistent connection pools per provider adapter.
- pre-warm provider sessions where supported.
- avoid cold-start key fetch on critical path by using short-lived cached handles with TTL bounds.

Quality rules:
- route decisions must combine availability, latency, and quality floors.
- high-confidence STT rejection patterns must bias faster fallback on next attempt.
- TTS interruption correctness must remain above judge floor before route promotion.

Hard rule:
- Performance optimizations must not change authority order, policy gates, or simulation requirements.

## 31) Long-Term Target State (Selene-First, Provider-Backed)

Target progression:
1. provider-backed reliability first (OpenAI/Google + delivery channels).
2. hybrid routing with Selene Local in bounded lanes.
3. Selene-first routing only where judge thresholds prove sustained superiority.
4. external providers remain governed fallback for resilience and outage tolerance.

Hard rule:
- Selene-first is enabled only by measured readiness + rollback safety, never by assumption.

## 32) World-Class Voice Recognition Build Plan (Near-Zero Miss)

Purpose:
- Define the execution-ready path to raise `PH1.VOICE.ID` toward near-zero miss behavior in real environments while preserving fail-closed safety and engine authority boundaries.

Hard rules (mandatory):
- Every onboarded person must complete voice enrollment, regardless of role or relationship type.
- Voice identity is required for voice personalization and memory writes.
- No direct engine-to-engine calls; Selene OS orchestrates all turn flow and side effects.
- Learning is asynchronous and governed; no in-turn runtime self-modification.

Target metrics (realistic production target):
- True-accept rate: `>= 99.0%`.
- False-reject rate: `<= 1.0%`.
- False-accept rate: `<= 0.1%`.
- Voice-ID decision latency: `p95 <= 60 ms`, `p99 <= 120 ms`.

KPI and policy lock scope:
- Voice KPI thresholds and fail-closed policy must be configured with profile precedence:
  - `global -> tenant -> locale -> channel -> user_exception`.
- Runtime must evaluate using the highest-precedence matching profile, then fail closed on ambiguity.

Execution phases:
1. Contract and schema expansion for scored biometric decisions and quality telemetry, including per-profile threshold policy.
2. Mandatory onboarding enrollment gate for all invitee classes.
3. Runtime matcher upgrade from placeholder fingerprinting to embedding + diarization + anti-spoof + temporal smoothing + confusion separation.
4. Voice-ID-specific FEEDBACK/LEARN/BUILDER signal path and governed artifact activation.
5. Canary rollout + continuous calibration + automatic rollback on regression.
6. Weekly calibration and confusion-pair hardening loop until per-user stability remains `>= 99%`.

## 33) Onboarding Voice Enrollment Flow (Exact Capture Contract)

### 33.1 Required enrollment session setup
- Required fields:
  - `tenant_id`, `user_id`, `speaker_id`, `onboarding_session_id`, `session_id`, `device_id`, `locale_set`, `consent_asserted`.
- Enrollment session must fail closed if consent is absent.

### 33.2 Required capture set per person
- Quiet environment: `6` accepted clips.
- Normal room noise: `6` accepted clips.
- Noisy environment: `6` accepted clips.
- Prompted phrases: `4` clips (challenge phrases).
- Free speech: `4` clips (natural speaking style).
- Name/number phrases: `2` clips (high-confusion content).
- Multilingual users: reduced but valid set per active language/accent profile.
- Liveness challenge: at least `1` dynamic prompt-response clip.

### 33.3 Per-sample quality gates (must pass before acceptance)
- Speech presence and duration floor: voiced speech `>= 2.0 s` per clip.
- Clipping/level constraints: clipped frame ratio `<= 1.0%`; peak level must stay below hard clipping.
- SNR/noise-floor bounds (default, overridable by precedence profile):
  - quiet/normal captures: `SNR >= 20 dB`, noise floor `<= -45 dBFS`.
  - noisy captures: `SNR >= 10 dB`, noise floor `<= -35 dBFS`.
- Overlap/multi-speaker rejection: overlap ratio `<= 2.0%` of voiced frames.
- Channel/device quality constraints: packet-loss-equivalent `<= 1.0%` and unstable-device reason-codes rejected.
- Spoof/replay risk check must return `LOW`; `MEDIUM/HIGH` is immediate reject.
- Low-quality sample handling: reject immediately and require recapture in the same enrollment session.

### 33.4 Enrollment completion lock criteria
- Minimum accepted clip count reached across required environments:
  - quiet `6` + normal `6` + noisy `6` + prompted `4` + free speech `4` + name/number `2` + liveness `1`.
- Minimum accepted speech duration reached: `>= 90 s` total accepted voiced audio.
- Minimum environment coverage reached (quiet/normal/noisy).
- Holdout verification threshold (default, overridable by precedence profile):
  - holdout TAR `>= 99.0%`,
  - holdout FAR `<= 0.10%`,
  - evaluated with nearest-confusion-pair negatives.
- Liveness checks pass.
- Confusion-pair margin checks pass for nearest competing voices:
  - top-1 minus top-2 decision margin `>= 0.15`.

### 33.5 Onboarding completion dependency
- `PH1.ONB` completion must require locked voice enrollment for all onboarded users.
- If not locked, status remains pending with reminder/retry workflow and a limited-mode runtime policy.

### 33.6 Capture rationale (why each capture set exists)
- Quiet samples: build clean baseline identity anchor.
- Normal-room samples: improve day-to-day reliability.
- Noisy samples: prevent collapse in real-world environments.
- Prompted phrases: strengthen anti-replay and phrase-conditioned discrimination.
- Free speech: capture natural pacing and prosody variation.
- Name/number phrases: harden high-confusion content.
- Multilingual samples: preserve recognition quality across language/accent shifts.
- Liveness challenge: reduce spoof/replay acceptance risk.

## 34) Continuous Accuracy Upgrade Loop (Capture -> Gold -> Upgrade)

### 34.1 Runtime signal capture
- Capture and reason-code:
  - false reject,
  - false accept,
  - speaker switch confusion,
  - replay/spoof risk,
  - low-quality capture,
  - re-auth/device-claim friction.

### 34.1.1 Intake path classification (mandatory)
- `Path A (Defect)`: broken or unsafe behavior requiring fix-first handling.
- `Path B (Improvement)`: non-broken quality optimization for accuracy/latency/robustness gains.

### 34.2 Gold output generation
- Gold tiers:
  - `Tier 1`: trusted deterministic truth.
  - `Tier 2`: verified human correction.
  - `Tier 3`: high-confidence consensus (non-safety-critical tuning only).
- Gold records must be versioned, immutable, hashed, and auditable.

### 34.3 Learning package generation
- `PH1.FEEDBACK` converts events into validated signal candidates.
- `PH1.LEARN` builds governed Voice-ID artifacts:
  - threshold packs,
  - confusion-pair packs,
  - spoof-policy packs,
  - profile adaptation packs.
- `PH1.GOV` decides activate/hold/rollback.

### 34.4 Builder integration
- `PH1.BUILDER` consumes governed learning context, runs gates, stages rollout, and auto-rolls back on KPI/safety breach.
- No gold artifact may bypass Builder + governance + release gates.
- Activation chain is strict: `PH1.LEARN -> PH1.GOV -> PH1.BUILDER -> staged rollout -> judge -> promote/rollback`.
- Loop remains async only; no in-turn runtime self-modification is permitted.

## 35) Wiring Plan (Current Engine Chain + Voice-ID Consumption)

### 35.1 Canonical voice branch table (single source of truth)

This table is authoritative for `Section 22`, `Section 35`, and `Section 41`.

| Branch ID | Trigger policy | Platform policy | Deterministic chain |
|---|---|---|---|
| `BR-01` explicit-trigger primary path | explicit app/side-button/manual trigger | iOS default; Android/desktop optional | `PH1.K -> PH1.VOICE.ID -> PH1.C -> PH1.SRL -> PH1.NLP -> PH1.CONTEXT -> PH1.POLICY -> PH1.X -> PH1.WRITE -> PH1.TTS` |
| `BR-02` wake-trigger path | accepted wake event required | Android/desktop default; iOS feature-flag override only | `PH1.K -> PH1.W -> PH1.VOICE.ID -> PH1.C -> PH1.SRL -> PH1.NLP -> PH1.CONTEXT -> PH1.POLICY -> PH1.X -> PH1.WRITE -> PH1.TTS` |
| `BR-03` endpoint-assist overlay (`TURN_OPTIONAL`) | policy/noise/context trigger | all platforms where assist is enabled | `BR-01` or `BR-02` plus `PH1.ENDPOINT` assist before `PH1.C`; assist cannot bypass `PH1.VOICE.ID` |

Branch hard rules:
- `PH1.VOICE.ID` is mandatory for any voice turn that can personalize or write memory.
- `PH1.W` is required only for wake-trigger branch `BR-02`.
- iOS defaults to `BR-01`; wake on iOS is explicit feature-flag override to `BR-02`.
- Unknown or ambiguous trigger state fails to `BR-01` generic-safe behavior.
- Enforcement anchor: `crates/selene_os/src/ph1os.rs:1004`.

Voice-ID output contract requirements:
- Contract `V1` (current compatibility):
  - `decision_v1` (`OK | UNKNOWN`), reason code, confidence score, margin-to-next-speaker, spoof/liveness result, candidate user id (optional).
- Contract `V2` (ladder-native target):
  - `identity_tier_v2` (`CONFIRMED | PROBABLE | UNKNOWN`) + same numeric telemetry + `may_prompt_identity`.
- During migration, `V1` and `V2` are dual-emitted (see `VID.CONF.8`).

Downstream consumption rules:
- `PH1.X`: personalization enabled only on:
  - `M0/M1`: `decision_v1=OK` plus provisional tier `CONFIRMED` derived from `CONF_HIGH_BP/CONF_MID_BP`.
  - `M2+`: `identity_tier_v2=CONFIRMED`.
  - `PROBABLE` always uses generic + ask-once path; `UNKNOWN` stays safe/limited mode (`crates/selene_engines/src/ph1x.rs:911`).
- `PH1.M`: memory writes/recall must reject unknown identity (`crates/selene_engines/src/ph1m.rs:168`).
- `PH1.C`: may consume advisory routing hints; transcript authority remains with `PH1.C`.
- `PH1.PAE`: may consume Voice-ID quality/risk telemetry for routing adaptation hints only.
- Identity authority remains in `PH1.VOICE.ID`.

Side-effect and safety rules:
- Voice-ID never grants authority and never performs side effects.
- Unknown identity must fail closed for protected operations.
- Any identity ambiguity routes to clarify/reauth flow, never silent best-guess execution.
- High-risk intents require deterministic step-up chain: `PH1.X -> PH1.ACCESS/CAPREQ -> STEP_UP_CHALLENGE -> CONTINUE|REFUSE`.

## 35A) PH1.VOICE.ID - Identity Confidence Ladder + Continuous Improvement (Insert)

### VID.CONF.0 Purpose
- Define how Selene handles speaker recognition confidence in production so early usage stays forgiving, identity remains fail-closed when uncertain, and per-user accuracy improves toward stable `~99%+`.
- Apply this policy at onboarding and runtime: personalization rules, generic fallback behavior, and quick identity reconfirm flow.

### VID.CONF.1 Hard rules
- Early sessions must be forgiving and low-friction.
- Confidence-tier decisions are deterministic and threshold-based (`CONF_HIGH`, `CONF_MID`), not heuristic.
- `LLM/NLP` may generate phrasing only; gate decisions remain deterministic.
- Fail-closed identity: below minimum threshold, Selene must not claim speaker identity.
- Ask-once rule: only one identity confirmation prompt per session (or cooldown window).
- Voice-only is never sufficient for high-stakes actions; step-up authentication is mandatory.

### VID.CONF.2 Confidence ladder (deterministic)

Threshold mapping:
- Tier A (`HIGH`): `score >= CONF_HIGH`.
- Tier B (`MID`): `CONF_MID <= score < CONF_HIGH`.
- Tier C (`LOW`): `score < CONF_MID`.

Tier A behavior (`HIGH`):
- Selene may personalize and use speaker name.
- Selene proceeds normally with no confirmation interrupt.
- Runtime may still capture quality evidence (consent/policy bounded) for continuous improvement.

Tier B behavior (`MID`):
- Selene must avoid confident identity claim by default.
- Selene continues in generic mode.
- Selene may ask one quick identity confirmation if ask-once/cooldown allows.
- Example intent: `is this JD, i didnt catch your voice accurately`.
- Exact wording is generated by `NLP/LLM`; the decision to ask is deterministic.

Tier C behavior (`LOW`):
- Selene must emit `UNKNOWN` identity and fail closed for identity-sensitive personalization.
- Selene must not use personal name greeting.
- Selene may continue with safe generic help.
- Selene may ask one quick confirmation if policy/cooldown allows.
- Selene records improvement evidence (consent/policy bounded) for later learning.

### VID.CONF.3 Early usage (forgiving) + gradual improvement
- Immediately post-onboarding, threshold policy should minimize user friction for everyday conversation while protecting private/high-risk paths.
- Use Tier B ask-once flow to gather quick identity confirmation without repeated interruption.
- As verified evidence grows, confidence stability increases and personalization naturally becomes more accurate.

### VID.CONF.4 Continuous improvement from normal speaking
- Selene improves from normal conversation, not repeated dedicated training sessions.
- Runtime may capture voice evidence each turn (consent/policy bounded), then grade by deterministic quality metrics.
- Only high-quality verified evidence updates active voice profile artifacts.
- Once per-user stability reaches `~99%+`, prompt frequency should reduce while low-friction refinement remains enabled.

### VID.CONF.5 Identity strictness by risk

Low-risk casual conversation:
- Identity can be relaxed.
- If uncertain, Selene stays generic.

Personalization and private data:
- To use personal naming or private-data context, Selene requires Tier A (`HIGH`) or explicit confirmation from Tier B/C flow.

Business-critical or high-risk actions:
- Voice recognition can assist but cannot authorize.
- Mandatory step-up authentication is required.

### VID.CONF.6 Step-up authentication for high-stakes actions (always required)
- Critical actions include payouts, transfers, payroll approval, sensitive access approval, and governance-setting changes.
- Selene must require device biometrics (`face/fingerprint`) when available.
- If biometrics are unavailable, Selene must require passcode.
- Voice confidence alone (even at `99%+`) never authorizes high-stakes actions.

### VID.CONF.7 Wiring expectations (engine boundaries)

`PH1.VOICE.ID` runtime output requirements:
- Deterministic identity tier: `CONFIRMED` (Tier A) / `PROBABLE` (Tier B) / `UNKNOWN` (Tier C).
- Deterministic telemetry: score, reason code, margin-to-next, and `may_prompt_identity` boolean (ask-once/cooldown enforced).
- Contract alignment note: existing `OK|UNKNOWN` decision field remains authoritative until tier field is added; tier mapping must still be deterministic from score thresholds.

`PH1.X` runtime behavior:
- Use identity tier + `may_prompt_identity` to choose `personalize` vs `generic` vs `ask-once`.
- Trigger step-up authentication for high-stakes intents regardless of voice tier.

Hard boundary reminder:
- `LLM/NLP` controls wording only.
- Identity thresholds, ask-once/cooldown, and step-up-auth gates are deterministic and fail closed.

### VID.CONF.8 Identity contract versioning + deterministic migration

Contract versions:
- `VOICE_ID_DECISION_V1`:
  - fields: `decision_v1(OK|UNKNOWN)`, `score_bp`, `reason_code`, `margin_to_next_bp`, `spoof_liveness_status`, `candidate_set`.
- `VOICE_ID_DECISION_V2`:
  - fields: `identity_tier_v2(CONFIRMED|PROBABLE|UNKNOWN)`, `score_bp`, `reason_code`, `margin_to_next_bp`, `spoof_liveness_status`, `candidate_set`, `may_prompt_identity`.

Deterministic V1->V2 mapping (runtime-owned):
- if `decision_v1=UNKNOWN` then `identity_tier_v2=UNKNOWN`.
- if `decision_v1=OK` and `score_bp >= CONF_HIGH_BP` then `identity_tier_v2=CONFIRMED`.
- if `decision_v1=OK` and `CONF_MID_BP <= score_bp < CONF_HIGH_BP` then `identity_tier_v2=PROBABLE`.
- if `decision_v1=OK` and `score_bp < CONF_MID_BP` then `identity_tier_v2=UNKNOWN` (fail-closed guard).

Migration stages:
1. `M0`:
   - emit/read `V1` only.
2. `M1`:
   - emit `V1 + V2`; downstream reads `V1`; `V2` audited in shadow mode.
3. `M2`:
   - emit `V1 + V2`; downstream reads `V2`; `V1` retained for compatibility.
4. `M3`:
   - retire `V1` reads after rollout-stability window and compatibility window pass.

Compatibility guard for `M0/M1`:
- Even when downstream reads `V1`, it must compute provisional tier from `score_bp` using `CONF_HIGH_BP/CONF_MID_BP`.
- During `M0/M1`, provisional `PROBABLE` must follow generic + ask-once behavior (no personalization).
- This prevents Tier-B drift before full `V2` read cutover.

### VID.CONF.9 Ask-once/cooldown state contract (deterministic)

State keys:
- Session prompt-budget key:
  - `identity_prompt_budget_key = (tenant_id, user_scope_id, device_id, session_id, voice_branch_id)`.
- Cross-session cooldown key:
  - `identity_prompt_cooldown_key = (tenant_id, user_scope_id, device_id, voice_branch_id)`.
- `user_scope_id` resolution order: `candidate_user_id` -> `device_owner_user_id` -> `anon_device_scope`.

Deterministic policy defaults:
- `max_identity_prompts_per_session = 1`.
- `identity_prompt_cooldown_ns = 600_000_000_000` (`10` minutes).
- `max_identity_retry_after_prompt = 1` immediate retry turn.

Reset rules:
- Session budget resets only on new `session_id`.
- Cooldown resets only on successful explicit identity confirmation or cooldown expiry.
- No repeated prompt allowed while cooldown is active, even across session restart on the same device scope.

`may_prompt_identity` computation:
- true only when tier is `PROBABLE|UNKNOWN`, session prompt budget remains, cooldown is expired for cooldown key, and intent is not high-stakes step-up pending.
- else false.

### VID.CONF.10 Step-up authentication handoff (mandatory chain)

Deterministic high-stakes handoff:
1. `PH1.X` classifies intent as high-stakes.
2. `PH1.ACCESS`/`PH1.CAPREQ` enforces step-up requirement.
3. `STEP_UP_CHALLENGE` executes:
   - device biometric when supported; else passcode.
4. Outcome:
   - `STEP_UP_PASS -> continue guarded action`.
   - `STEP_UP_FAIL|TIMEOUT|UNAVAILABLE -> refuse/defer` with reason code.

Hard rules:
- Voice tier cannot bypass step-up for high-stakes actions.
- Step-up decisions are deterministic and reason-coded in `PH1.J`.

## 36) Concrete Upgrade List (Current Implementation Delta)

Required changes before claiming world-class readiness:

1. Replace placeholder fingerprint matching with production biometric scoring (`crates/selene_engines/src/ph1_voice_id.rs:31`, `crates/selene_engines/src/ph1_voice_id.rs:39`).
2. Expand `PH1.VOICE.ID` contracts for numeric confidence, margin, and richer decision telemetry (`crates/selene_kernel_contracts/src/ph1_voice_id.rs:90`).
3. Expand enrollment contracts to include quality metadata beyond pass/fail (`crates/selene_kernel_contracts/src/ph1_voice_id.rs:447`).
4. Remove caller-provided sample grading as source-of-truth; runtime must compute grade deterministically (`crates/selene_os/src/ph1onb.rs:646`).
5. Enforce voice enrollment completion before onboarding complete (`crates/selene_storage/src/ph1f.rs:6217`).
6. Remove synthetic speaker bypass in simulation memory paths (`crates/selene_os/src/simulation_executor.rs:2018`).
7. Extend storage schema for quality/locale/environment/profile versioning/re-enrollment lineage (`crates/selene_storage/migrations/0008_ph1vid_voice_enrollment_tables.sql:34`, `crates/selene_storage/migrations/0008_ph1vid_voice_enrollment_tables.sql:65`).
8. Add Voice-ID-specific feedback event taxonomy (`crates/selene_kernel_contracts/src/ph1feedback.rs:26`).
9. Add Voice-ID-specific learn signal and artifact targets (`crates/selene_kernel_contracts/src/ph1learn.rs:26`).
10. Include `PH1.VOICE.ID` in Builder learning-source bridge (`crates/selene_os/src/ph1builder.rs:67`).

## 37) Requested Completeness Verification (Checklist Lock)

This section confirms the requested scope is fully represented in this plan.

Checklist outcomes:
1. Realism statement and hard metrics (`99/1/0.1`, `p95/p99`) included.
2. Build execution order included with tenant/locale/channel fail-closed KPI lock.
3. Onboarding capture contract includes consent, capture set, quality gates, recapture, lock criteria, and pending/limited mode.
4. Continuous loop includes runtime events, Path A/Path B split, gold tiers, LEARN artifacts, GOV gating, BUILDER rollout, and async-only rule.
5. Wiring includes canonical engine chain, enforcement anchor, output contract, and downstream consumers.
6. Concrete codebase upgrades include all requested file anchors.
7. Weekly calibration and confusion-pair hardening loop is explicitly included.

Acceptance criteria for this section:
- No open bypass path for identity-sensitive operations.
- Mandatory enrollment is enforced at onboarding completion boundary.
- Voice-ID learning artifacts are governed and consumed by runtime through approved activation only.
- KPI dashboard includes TAR/FRR/FAR + latency SLO + spoof block rate with canary/full-stage gates.

## 38) Advanced Voice-ID Reliability and Performance Upgrades

### 38.1 Strict calibration protocol (DET/ROC-based)
- Threshold setting must be based on DET/ROC calibration runs, not static hand-tuned values.
- Calibrations must produce versioned threshold artifacts with cohort breakdown and signed audit refs.
- Calibration artifact minimums per cohort:
  - genuine samples `>= 1500`,
  - impostor comparisons `>= 15000`,
  - frozen holdout rows excluded from training/adaptation paths.
- Promotion math must include `95%` confidence intervals:
  - TAR lower bound `>= 99.0%`,
  - FAR upper bound `<= 0.10%`,
  - FRR upper bound `<= 1.0%`.

### 38.2 Gray-zone abstain policy
- Define a score-margin gray zone where identity decisions must abstain.
- In gray-zone outcomes, runtime must return `UNKNOWN` + deterministic reauth flow; never best-guess identity.

### 38.3 Per-cohort gating
- Promotion gates must pass per cohort, not only global averages.
- Required cohorts: language, accent band, device class, and noise class.

### 38.4 Two-stage latency pipeline
- Stage 1: fast candidate pruning with strict latency budget.
- Stage 2: high-precision verification only for retained candidates.
- Budget rule: Stage 2 is skipped when Stage 1 confidence is decisively high or decisively unknown.

### 38.5 Identity conflict state machine
- Add explicit conflict states for:
  - spoof risk,
  - multi-speaker overlap,
  - device-owner mismatch,
  - profile mismatch/conflict.
- Each conflict state must map to one deterministic action (`UNKNOWN`, reauth, claim flow, or retry).

Conflict transition lock (deterministic):

| State | Entry Condition | Max Automatic Retries | Deterministic Action | Terminal Escalation |
|---|---|---|---|---|
| `SPOOF_RISK` | liveness/spoof score is `MEDIUM/HIGH` | `0` | `UNKNOWN` + step-up reauth challenge | lock protected actions + human-review queue |
| `MULTI_SPEAKER_OVERLAP` | overlap ratio above gate | `1` | request single-speaker retry; else `UNKNOWN` | fail-closed to non-personalized mode |
| `DEVICE_OWNER_MISMATCH` | signed-in owner and voice identity diverge | `1` | `UNKNOWN` + claim/ownership confirmation flow | ownership dispute queue + no memory write |
| `PROFILE_MISMATCH` | speaker score below threshold with close competitor | `2` | challenge phrase retry; else `UNKNOWN` | re-enrollment-required state |

State-machine hard rule:
- Any unresolved conflict after max retries must terminate in `UNKNOWN` (never best-guess).

### 38.6 Drift triggers with auto-recovery
- Define drift detectors for:
  - quality decay,
  - FAR/FRR drift,
  - margin compression,
  - cohort-specific regression.
- Drift breach must trigger automatic re-enrollment or adaptation workflow with reason-coded audit trail.
- Drift safety rate limits (anti-thrash):
  - adaptation artifact auto-apply: max `1` per user per `24 h`.
  - re-enrollment prompt: max `1` per user per `72 h`.
  - if `>= 3` drift breaches in `14` days for same user/cohort: freeze auto-apply and require review.
  - every freeze/unfreeze transition must emit reason-coded audit rows.

### 38.7 Confusion-pair hardening program
- Continuously track highest-frequency speaker collision pairs.
- Generate hard-negative training/adaptation packs targeted to those confusion pairs.
- Promotion requires measured reduction in confusion-pair collision rate.

### 38.8 Frozen benchmark sets by cohort
- Maintain immutable evaluation sets per cohort.
- Hard rule: no training/adaptation artifact may consume frozen benchmark rows.
- Any evaluation leakage invalidates promotion and forces rollback to previous artifact set.

### 38.9 Stability-window rollout gates
- Promotion beyond canary requires a stability window pass of `7-14` days.
- Stability window must pass both global and cohort gates before promote.

### 38.10 Runtime circuit-breakers
- On uncertainty spikes or backend instability, circuit-breaker mode must force `UNKNOWN`.
- Circuit-breaker mode must never degrade into wrong identity assignment.

### 38.11 Fast unknown-identity recovery UX
- Unknown identity path must provide short challenge + immediate retry without long conversational detours.
- Recovery flow must preserve latency and reduce repeated user friction.

### 38.12 Voice embedding privacy and consent lifecycle
- Add explicit lifecycle controls for voice embeddings:
  - rotation,
  - revocation,
  - retention expiry,
  - consent withdrawal handling.
- Lifecycle events must be audited and enforceable per tenant/user policy scope.

## 39) Advanced Upgrade Acceptance Gates

All upgrades in Section 38 are required to be considered complete only when:
1. Calibration artifacts are DET/ROC-derived, versioned, and active through governance, with cohort sample minimums and CI bounds satisfied.
2. Gray-zone abstain behavior is verified in runtime tests and canary telemetry.
3. Cohort gates pass without hidden regressions behind global averages.
4. Two-stage pipeline meets latency SLO without identity-quality regression.
5. Conflict state machine actions are deterministic, transition-tested, and fully reason-coded.
6. Drift triggers automatically open re-enrollment/adaptation workflows with anti-thrash rate limits enforced.
7. Confusion-pair collision rates trend downward across successive releases.
8. Frozen benchmark integrity checks show zero train/eval leakage.
9. Rollout promotions include a successful `7-14` day stability window.
10. Circuit-breakers prove fail-to-unknown behavior under instability tests.
11. Unknown-identity recovery UX meets bounded retry latency targets.
12. Privacy/consent lifecycle operations are implemented, auditable, and enforceable.
13. `V1->V2` migration invariants pass: dual-emit correctness, provisional-tier behavior in `M0/M1`, and cutover audits show zero contract drift.
14. Step-up gate chain correctness passes: `PH1.X -> PH1.ACCESS/CAPREQ -> STEP_UP_CHALLENGE -> CONTINUE|REFUSE|DEFER` is replay-tested with deterministic reason codes.

## 40) Concrete Implementation Checklist (Owner-Mapped)

Scope conversion note:
- This section converts Sections 32-39 into an execution checklist with explicit owner accountability.
- Runtime ownership is split and fixed across: `PH1.VOICE.ID`, `PH1.ONB`, `PH1.FEEDBACK`, `PH1.LEARN`, `PH1.GOV`, `PH1.BUILDER`, `PH1.ACCESS/CAPREQ`.

Completion rule (all checklist items):
- A task is complete only when code/tests are merged, rollout state is recorded, and a reason-coded `PH1.J` audit trail exists.

### 40.1 `PH1.VOICE.ID` checklist (identity decision owner)

- [x] `VID-01` Contract expansion: emit `decision`, `score`, `margin_to_next`, `reason_code`, `spoof_liveness_status`, optional candidate set. (implemented in `crates/selene_kernel_contracts/src/ph1_voice_id.rs` + `crates/selene_engines/src/ph1_voice_id.rs`)
- [x] `VID-02` Runtime matcher upgrade: implement two-stage path (fast prune -> high-precision verify) with bounded latency. (implemented in `crates/selene_engines/src/ph1_voice_id.rs`)
- [x] `VID-03` Gray-zone abstain: enforce `UNKNOWN` for low-margin outcomes; never best-guess. (implemented in `crates/selene_engines/src/ph1_voice_id.rs` with `stage2_min_margin_bp` + `VID_FAIL_GRAY_ZONE_MARGIN` fail-closed path)
- [x] `VID-04` Conflict state machine: enforce deterministic transitions for spoof/overlap/device mismatch/profile mismatch. (implemented in `crates/selene_engines/src/ph1_voice_id.rs` with deterministic fail-closed branches and reason codes)
- [x] `VID-05` Circuit-breaker behavior: uncertainty or backend instability must fail to `UNKNOWN`. (implemented as deterministic fail-closed unknown paths under uncertainty/risk signals in `crates/selene_engines/src/ph1_voice_id.rs`)
- [x] `VID-06` Telemetry emission: publish reason-coded events for FRR/FAR/confusion/drift signals to `PH1.FEEDBACK`. (implemented in `crates/selene_os/src/ph1_voice_id.rs` `run_identity_assertion_with_signal_emission`)
- [x] `VID-07` Cohort KPI export: publish TAR/FRR/FAR/latency by language/accent/device/noise cohort. (implemented in `crates/selene_os/src/ph1_voice_id.rs` via per-turn `PH1.VOICE.ID` KPI audit emission with cohort labels and TAR/FRR/FAR/latency metrics)
- [x] `VID-08` Phone-local artifact custody: maintain active Voice-ID artifacts on-device (`active + N-1 rollback`) with deterministic pointer selection. (implemented in `crates/selene_os/src/ph1_voice_id.rs` `tenant_artifact_pointers` + deterministic `select_artifact_pointer_set` with test coverage)
- [x] `VID-09` Continuous sync contract: emit artifact-manifest deltas to Engine B outbox on every profile/threshold/confusion/spoof package change. (implemented in `crates/selene_storage/src/ph1f.rs` + `crates/selene_os/src/device_artifact_sync.rs`, covered by `AT-VID-DB-10/11`)
- [x] `VID-10` Confidence-ladder mapping: implement deterministic `CONF_HIGH_BP`/`CONF_MID_BP` mapping to `CONFIRMED|PROBABLE|UNKNOWN`. (implemented in `crates/selene_kernel_contracts/src/ph1_voice_id.rs` + consumed in `crates/selene_os/src/ph1x.rs`)
- [x] `VID-11` Ask-once state machine: implement `may_prompt_identity` using deterministic state key, cooldown, and retry budget. (implemented in `crates/selene_kernel_contracts/src/ph1x.rs`, `crates/selene_os/src/ph1x.rs`, `crates/selene_os/src/ph1os.rs`, `crates/selene_os/src/app_ingress.rs`)
- [x] `VID-12` Contract migration: implement `V1 + V2` dual-emit, shadow audit, and deterministic cutover stages. (implemented in `crates/selene_os/src/ph1_voice_id.rs` with `M0/M1/M2/M3` stage config, provisional V1-based enforcement for `M0/M1`, and migration shadow audit emission)

### 40.2 `PH1.ONB` checklist (enrollment gate owner)

- [x] `ONB-01` Mandatory voice enrollment gate: onboarding cannot complete until voice profile is `LOCKED`. (enforced in `crates/selene_storage/src/ph1f.rs` `ph1onb_complete_commit`; covered by `crates/selene_storage/tests/ph1_onb/db_wiring.rs` mandatory-complete gate tests)
- [x] `ONB-02` Capture contract enforcement: enforce required clip set by environment/type/liveness. (enforced via deterministic capture-profile lock checks in `crates/selene_storage/src/ph1f.rs` `evaluate_voice_enrollment_lock_criteria`; tested in `crates/selene_storage/tests/ph1_voice_id/db_wiring.rs`)
- [x] `ONB-03` Numeric quality gates: enforce SNR/noise-floor/duration/clipping/overlap/spoof thresholds with recapture on failure. (runtime scoring in `crates/selene_storage/src/ph1f.rs` `grade_voice_enrollment_sample`; validated by `at_vid_db_06*` tests)
- [x] `ONB-04` Lock criteria enforcement: enforce minimum accepted duration, holdout TAR/FAR, and confusion-margin threshold. (implemented with deterministic lock metrics + thresholds in `crates/selene_storage/src/ph1f.rs`; validated by `at_vid_db_06c`)
- [x] `ONB-05` Pending-mode behavior: unresolved enrollment remains `PENDING` with limited-mode runtime policy. (implemented by `VoiceEnrollStatus::Pending` transitions + `ph1onb_voice_runtime_mode` in `crates/selene_storage/src/ph1f.rs`; validated in `at_vid_db_06c`)
- [x] `ONB-06` Consent binding: enrollment session requires consent and stores consent scope for lifecycle controls. (consent required at start and persisted as `consent_scope_ref` in `VoiceEnrollmentSessionRecord`; enforced in onboarding complete path in `crates/selene_storage/src/ph1f.rs`; tested in `at_vid_db_06b`)
- [x] `ONB-07` Phone-first session start: onboarding must begin from app-open context (`token_id + app_platform + app_instance + device_fingerprint`), not web fallback. (enforced in `crates/selene_storage/src/ph1f.rs` `ph1onb_session_start_draft`; tested in `crates/selene_storage/tests/ph1_onb/db_wiring.rs`)
- [x] `ONB-08` Link->app handoff lock: after `LINK_OPEN_ACTIVATE`, onboarding enters app capture flow immediately and preserves replay-safe handoff refs. (strict match on `app_platform + app_instance_id + deep_link_nonce + link_opened_at` in `crates/selene_storage/src/ph1f.rs`; replay-safe test `at_onb_db_01b`)

### 40.3 `PH1.FEEDBACK` checklist (signal intake owner)

- [x] `FDBK-01` Voice-ID taxonomy: add explicit event classes (`FALSE_REJECT`, `FALSE_ACCEPT`, `SPOOF_RISK`, `MULTI_SPEAKER`, `DRIFT_ALERT`, `REAUTH_FRICTION`). (implemented in `crates/selene_kernel_contracts/src/ph1feedback.rs`, `crates/selene_kernel_contracts/src/ph1learn.rs`, and consumed in `crates/selene_os/src/ph1_voice_id.rs`)
- [x] `FDBK-02` Path classifier: deterministically route each event to `Path A (Defect)` or `Path B (Improvement)`. (implemented via `classify_feedback_path` in `crates/selene_kernel_contracts/src/ph1feedback.rs` and enforced in `crates/selene_storage/src/ph1f.rs`)
- [x] `FDBK-03` Idempotent ingestion: enforce dedupe by deterministic key and reject malformed events fail-closed. (implemented in `crates/selene_storage/src/ph1f.rs` `ph1feedback_event_commit` + `ph1feedback_learn_signal_bundle_commit`)
- [x] `FDBK-04` Gold-candidate packaging: output validated signal bundles with provenance refs for `PH1.LEARN`. (implemented in `crates/selene_storage/src/ph1f.rs` `FeedbackLearnSignalBundleRecord` + `ph1feedback_learn_signal_bundle_commit`; wired from `crates/selene_os/src/ph1_voice_id.rs`)
- [x] `FDBK-05` Intake SLO: maintain bounded processing latency so learning ingestion does not backlog. (implemented with `PH1_FEEDBACK_SIGNAL_BUNDLE_INGEST_SLO_MS` fail-closed gate in `crates/selene_storage/src/ph1f.rs`, covered by `at_fdbk_04_signal_bundle_enforces_ingest_slo`)

### 40.4 `PH1.LEARN` checklist (artifact builder owner)

- [x] `LRN-01` Artifact families: build versioned Voice-ID artifacts (`threshold_pack`, `confusion_pair_pack`, `spoof_policy_pack`, `profile_adaptation_pack`). (implemented in `crates/selene_os/src/ph1learn.rs` via `required_voice_artifact_families` + `voice_artifact_family_for_target` with fail-closed family validation)
- [x] `LRN-02` Frozen-eval isolation: enforce no train/eval leakage from frozen benchmark rows. (implemented in `crates/selene_os/src/ph1learn.rs` via `has_frozen_eval_evidence_leakage` pre-submit refusal path)
- [x] `LRN-03` Calibration math: compute DET/ROC outputs with cohort sample minimums and confidence intervals. (implemented in `crates/selene_os/src/ph1learn.rs` via `voice_calibration_snapshot_from_signals` + `LEARN_VOICE_CALIBRATION_MIN_SAMPLES` gate)
- [x] `LRN-04` Drift handling: trigger adaptation/re-enrollment packages with anti-thrash rate limits. (implemented in `crates/selene_os/src/ph1learn.rs` via drift-family detection and deterministic 24h/72h/14d anti-thrash refusal thresholds)
- [x] `LRN-05` Rollback readiness: every proposed artifact must include rollback pointer and prior-version compatibility metadata. (implemented in `crates/selene_os/src/ph1learn.rs` by mandatory rollback pointer checks and `LearnGovArtifactProposal` compatibility metadata validation)
- [x] `LRN-06` Governance handoff: submit only fully validated packages to `PH1.GOV` (invalid packages blocked pre-submit). (implemented in `crates/selene_os/src/ph1learn.rs` by fail-closed package validation and explicit `gov_artifact_proposals` emission only after all learn gates pass)

### 40.5 `PH1.GOV` checklist (activation authority owner)

- [x] `GOV-01` Gate enforcement: evaluate policy/privacy/consent/tenant-scope constraints for each proposed artifact. (implemented in `crates/selene_os/src/ph1gov.rs` preflight gates: `privacy_policy_passed`, `consent_scope_active`, `tenant_scope_verified`)
- [x] `GOV-02` Decision contract: emit deterministic `ALLOW|HOLD|BLOCK|ROLLBACK` outcomes with reason codes. (implemented in `crates/selene_os/src/ph1gov.rs` via `GovDecisionClass`, `GovResolvedDecision`, and `GovWiringOutcome::resolved_decision`)
- [x] `GOV-03` Cohort safety gates: require per-cohort pass, not global-average pass only. (implemented in `crates/selene_os/src/ph1gov.rs` with `required_cohort_keys` subset checks against `passing_cohort_keys`)
- [x] `GOV-04` Stability-window gate: require `7-14` day pass window before full promotion. (implemented in `crates/selene_os/src/ph1gov.rs` activation preflight using `stability_window_days_passed` bounded to `7..=14`)
- [x] `GOV-05` Revocation handling: consent revoke must freeze/rollback active voice artifacts within defined SLA. (implemented in `crates/selene_os/src/ph1gov.rs` via `consent_revoked` + `revocation_sla_met` preflight enforcing rollback-only path)
- [x] `GOV-06` Builder dispatch: only `ALLOW` artifacts may flow to `PH1.BUILDER` rollout. (implemented in `crates/selene_os/src/ph1gov.rs` `GovForwardBundle::to_builder_dispatch_ticket` allowing dispatch only when decision class is `ALLOW`)

### 40.6 `PH1.BUILDER` checklist (rollout execution owner)

- [x] `BLD-01` Build bridge ingest: consume governed artifacts and bind to release candidate metadata. (implemented in `crates/selene_os/src/ph1builder.rs` via `BuilderGovernedIngestInput`, `BuilderGovernedReleaseBinding`, and `bind_governed_artifact_to_release_candidate`)
- [x] `BLD-02` Verification suite: run contract tests, replay tests, calibration checks, and conflict-state tests. (implemented in `crates/selene_os/src/ph1builder.rs` `BuilderVerificationSuite` + fail-closed ingest gating when any suite component fails)
- [x] `BLD-03` Staged rollout: enforce shadow -> canary1 -> canary2 -> full progression with judge gates. (implemented in `crates/selene_os/src/ph1builder.rs` via `BuilderRolloutJudgeGates` + `promote_with_judge_gates`; covered by `at_builder_os_18`)
- [x] `BLD-04` Auto-rollback: trigger rollback immediately on safety/KPI gate breach. (implemented in `crates/selene_os/src/ph1builder.rs` via `auto_rollback_on_safety_or_kpi_breach`; covered by `at_builder_os_20`)
- [x] `BLD-05` Promotion report: emit deterministic rollout report with cohort deltas and decision evidence. (implemented in `crates/selene_os/src/ph1builder.rs` via `BuilderPromotionReport` + `build_promotion_report`; covered by `at_builder_os_21`)
- [x] `BLD-06` Runtime activation handoff: publish active artifact pointers for runtime consumption (no in-turn self-modification). (implemented in `crates/selene_os/src/ph1builder.rs` via `BuilderRuntimeActivationHandoff` + `publish_runtime_activation_handoff`; covered by `at_builder_os_22/23`)
- [x] `BLD-07` Prompt-rate gating: enforce ask-once KPI gates (`prompts/session`, `repeat-prompt violations`, `prompt->confirm success`) before promotion. (implemented in `crates/selene_os/src/ph1builder.rs` via `BuilderPromptRateKpis` gate enforcement inside `promote_with_judge_gates`; covered by `at_builder_os_19`)

### 40.7 Engine-to-engine handoff checklist

- [x] `HND-01` `PH1.VOICE.ID -> PH1.FEEDBACK`: reason-coded runtime event envelope is emitted on every relevant identity decision. (wired in `crates/selene_os/src/ph1_voice_id.rs` `map_voice_response_to_feedback_learn_signal` + `emit_voice_id_feedback_and_learn_signal`)
- [x] `HND-02` `PH1.ONB -> PH1.VOICE.ID`: only locked enrollment profiles are activated for identity assertions. (live ingress now resolves enrolled speakers from persisted `ph1vid` profiles in `crates/selene_os/src/app_ingress.rs` `locked_enrolled_speakers_from_store`)
- [x] `HND-03` `PH1.FEEDBACK -> PH1.LEARN`: validated and deduped signal bundles are delivered with provenance. (implemented via `ph1feedback_learn_signal_bundle_commit` in `crates/selene_storage/src/ph1f.rs` and runtime emission in `crates/selene_os/src/ph1_voice_id.rs`)
- [x] `HND-04` `PH1.LEARN -> PH1.GOV`: governed artifact proposals include metrics, CI, and rollback metadata. (implemented in `crates/selene_os/src/ph1learn.rs` `LearnGovArtifactProposal` + `VoiceCalibrationSnapshot` payload carried in `LearnForwardBundle.gov_artifact_proposals`)
- [x] `HND-05` `PH1.GOV -> PH1.BUILDER`: only explicit `ALLOW` decisions trigger staged rollout. (implemented in `crates/selene_os/src/ph1gov.rs` `GovForwardBundle::to_builder_dispatch_ticket`, which dispatches only when `decision=ALLOWED` and otherwise returns `NotDispatchedDecisionNotAllowed`)
- [x] `HND-06` `PH1.BUILDER -> runtime`: activation pointer is published only after gate pass; rollback pointer remains live. (implemented in `crates/selene_os/src/ph1builder.rs` `publish_runtime_activation_handoff`; verified by `at_builder_os_22/23`)
- [x] `HND-07` `PH1.LINK -> PH1.ONB`: activated link must carry app-open context (`deep_link_nonce`, `app_platform`, `app_instance_id`, `link_opened_at`) into onboarding start. (implemented in `crates/selene_os/src/ph1onb.rs` `start_session_from_link_activation` + storage guardrails in `crates/selene_storage/src/ph1f.rs`; covered by `onb_link_activation_handoff_*` and `at_onb_db_05`)
- [x] `HND-08` `PH1.W/PH1.VOICE.ID/PH1.EMO.* -> Engine B`: local artifact-manifest changes enqueue continuous sync deltas with ack-gated replay. (implemented in `crates/selene_storage/src/ph1f.rs` artifact-manifest sync routing + `crates/selene_os/src/device_artifact_sync.rs` ack/replay worker; covered by `at_vid_db_10/12/13`)
- [x] `HND-09` `PH1.X -> PH1.ACCESS/CAPREQ`: high-stakes intent class must trigger deterministic step-up challenge path before side effects. (implemented by deterministic high-stakes binding + `DispatchRequest::AccessStepUp` handoff in `crates/selene_os/src/ph1x.rs` and consumed in `crates/selene_os/src/simulation_executor.rs`)
- [x] `HND-10` `PH1.ACCESS/CAPREQ -> PH1.X`: step-up outcome must deterministically return `CONTINUE|REFUSE|DEFER` with reason code. (implemented via `StepUpOutcome` + `StepUpResult` contract in `crates/selene_kernel_contracts/src/ph1x.rs`, `SimulationExecutor::step_up_result_from_dispatch_outcome` in `crates/selene_os/src/simulation_executor.rs`, and step-up result handling in `crates/selene_os/src/ph1x.rs`)

### 40.8 `PH1.ACCESS/CAPREQ` checklist (step-up authority owner)

- [x] `ACC-01` High-stakes policy binding: maintain deterministic action-class map that marks step-up-required intents. (implemented by `high_stakes_policy_binding` in `crates/selene_os/src/ph1x.rs`)
- [x] `ACC-02` Challenge selection: enforce biometric-first, passcode-fallback selection with capability checks. (implemented by `StepUpCapabilities` + `select_step_up_challenge` in `crates/selene_kernel_contracts/src/ph1x.rs` and `crates/selene_os/src/ph1x.rs`; app ingress now sets deterministic platform defaults in `crates/selene_os/src/app_ingress.rs`)
- [x] `ACC-03` Outcome contract: emit only `CONTINUE|REFUSE|DEFER` with deterministic reason codes. (implemented by `StepUpOutcome`/`StepUpResult` and `SimulationDispatchOutcome::AccessStepUp` in `crates/selene_kernel_contracts/src/ph1x.rs` + `crates/selene_os/src/simulation_executor.rs`)
- [x] `ACC-04` Replay determinism: step-up decision path must be simulation/replay stable with no hidden state bypass. (covered by `at_sim_exec_16_access_step_up_biometric_requirement_defers_and_is_replay_stable` in `crates/selene_os/src/simulation_executor.rs`)
- [x] `ACC-05` Audit completeness: every step-up start/finish/fail path emits required `PH1.J` audit events. (implemented by `ph1access_capreq_step_up_audit_commit` in `crates/selene_storage/src/ph1f.rs`; covered by `at_sim_exec_14_access_step_up_returns_continue_and_emits_start_finish_audit`)

### 40.9 Exit criteria for Section 40

- [x] `EXIT-01` All checklist IDs above are complete with linked evidence. (Section `40.1` through `40.8` now marked complete with implementation/test references in this document)
- [x] `EXIT-02` Voice-ID production targets are met with cohort-safe pass criteria. (deterministic target gate + per-cohort failure detection implemented in `crates/selene_os/src/section40_exit.rs` and covered by `at_section40_exit_02_voice_targets_require_per_cohort_pass`)
- [x] `EXIT-03` No unresolved owner ambiguity across declared runtime owners (`PH1.VOICE.ID`, `PH1.ONB`, `PH1.FEEDBACK`, `PH1.LEARN`, `PH1.GOV`, `PH1.BUILDER`, `PH1.ACCESS/CAPREQ`). (authoritative owner matrix + ambiguity detector implemented in `crates/selene_os/src/section40_exit.rs` and covered by `at_section40_exit_03_owner_matrix_is_single_authoritative`)
- [x] `EXIT-04` Full audit chain proves deterministic behavior from intake through rollout/rollback. (cross-owner chain gate with idempotency/ordering checks implemented in `crates/selene_os/src/section40_exit.rs` and covered by `at_section40_exit_04_audit_chain_intake_to_rollout_is_deterministic`)
- [x] `EXIT-05` Ladder migration and ask-once rollout pass: `V1->V2` cutover complete with no contract drift and prompt-rate KPI gate pass. (cutover + prompt-rate gate evaluator implemented in `crates/selene_os/src/section40_exit.rs` and covered by `at_section40_exit_05_ladder_cutover_and_prompt_gate_pass`)

## 41) Phone-First Runtime + Artifact Custody Contract (Non-Negotiable)

Operating model lock:
- Primary runtime is phone-first for `PH1.W`, `PH1.VOICE.ID`, and `PH1.EMO.*`.
- Onboarding must start from phone app open (`onboarding link -> app open -> immediate onboarding flow`).
- Cloud remains continuity + governance + recovery authority; phone remains low-latency inference authority.

Hard rules:
- Runtime identity/wake/emotional decisions must remain available on phone even during temporary network loss.
- Local artifacts must always include `ACTIVE` and `N-1 rollback` versions.
- Raw audio is excluded from sync by default; only bounded references/features/manifest metadata may sync unless explicit policy+consent allows otherwise.
- Loss/theft continuity requires continuous sync; "sync later" batch-only mode is not acceptable for production posture.
- Platform trigger policy is explicit: `IOS` defaults to app/side-button trigger (`explicit_trigger_only=true`, no always-on wake), while `ANDROID` and desktop may run always-on wake when policy allows.

### 41.1 Local artifact set (phone, per engine)

`PH1.W` local artifacts:
- wake phrase set/version.
- wake threshold/cooldown package.
- per-device wake calibration profile.
- active wake binding pointer + rollback pointer.

`PH1.VOICE.ID` local artifacts:
- voice profile embedding package/version.
- threshold package.
- confusion-pair package.
- spoof/liveness policy package.
- active voice profile pointer + rollback pointer.

`PH1.EMO.CORE` and `PH1.EMO.GUIDE` local artifacts:
- emotional profile snapshot package/version.
- tone guidance policy package.
- privacy/consent-scoped tone continuity settings.
- active emotional policy pointer + rollback pointer.

### 41.2 Continuous sync contract (phone -> Selene)

Sync method lock:
- Engine B outbox is mandatory transport for artifact-manifest deltas.
- Every local artifact change emits an outbox delta envelope with hash/provenance refs.
- Server ack is required before outbox deletion (ack-gated durability).

Cadence lock:
- event-driven enqueue on every local artifact change.
- heartbeat reconcile at `<= 30s`.
- full startup reconcile on app launch.
- offline replay until acked (idempotent dedupe required).

### 41.3 Synced manifest minimum (versioned + auditable)

Required synced fields:
- `engine_id`, `artifact_type`, `artifact_version`, `artifact_status`.
- `scope_type`, `scope_id`, `tenant_id`, `user_id`, `device_id`.
- `package_hash`, `payload_ref`, `provenance_ref`.
- `local_created_at`, `synced_at`, `idempotency_key`.
- `rollback_pointer_ref`, `consent_scope_ref`.

Hard rule:
- Active profile pointer on phone and server-side latest accepted pointer must reconcile deterministically; conflict resolves fail-closed to previous stable pointer until governance decision is applied.

### 41.4 Platform trigger execution policy

- Must remain consistent with branch table in `Section 35.1`.
- `IOS`:
  - default runtime trigger is side-button/app-open/session-resume; always-on wake is disabled by default.
  - voice-ID still runs after trigger for identity gating.
  - optional wake enablement is feature-flagged and opt-in only.
- `ANDROID` and desktop:
  - wake runtime can be always-on (policy + battery profile bounded).
  - voice-ID runs post-wake or post-manual trigger, same contracts as iOS.
- Fail-safe:
  - uncertain trigger state or degraded wake backend must fail to explicit trigger path, never forced wake accept.

### 41.5 App-side execution profile (what runs on phone)

- `IOS` app path (default):
  - user triggers Selene via side-button/app-open/push-tap.
  - app executes local `PH1.VOICE.ID` immediately after trigger; no always-on wake loop by default.
  - if identity is uncertain, fail to `UNKNOWN` and run short challenge flow (no identity guess).
- `ANDROID` app path (default):
  - always-on wake is allowed when device/battery policy permits.
  - wake accept triggers local `PH1.VOICE.ID` before downstream turn execution.
  - if wake backend degrades, auto-fallback to explicit trigger mode.
- desktop path:
  - always-on wake is policy-controlled and device-capability bounded.
  - same identity guardrails as mobile (`UNKNOWN` on uncertainty).
- common app invariants:
  - artifacts are local-first (`ACTIVE` + `N-1 rollback`) for wake/voice/emotional engines.
  - every artifact change enqueues sync delta immediately; heartbeat reconcile remains `<= 30s`.
  - startup performs full pointer reconcile so phone-loss/device-replacement continuity is deterministic.

## 42) Phone Onboarding Handoff Contract (Link -> App -> ONB)

Required handoff fields from `PH1.LINK` to `PH1.ONB`:
- `token_id`, `draft_id`, `device_fingerprint_hash`.
- `app_platform` (`IOS|ANDROID`), `app_instance_id`.
- `deep_link_nonce`, `link_opened_at`, `activation_status`.

Deterministic flow:
1. Link open/activate in app context.
2. Device binding + nonce verification.
3. Immediate ONB session start in app.
4. Voice multi-sample enrollment in app; wake enrollment runs only on wake-required platform policy paths (`ANDROID` default, `IOS` optional/feature-flagged).
5. Artifact-lock + sync enqueue before onboarding completion.

Fail-closed rules:
- Missing/invalid app-open context blocks onboarding start.
- Device fingerprint mismatch blocks resume and requires deterministic re-open flow.
- Enrollment lock without sync enqueue proof blocks completion.

## 43) Required Contract Deltas (Docs/Contracts Scope)

`PH1.LINK`:
- add app-open context contract to `LINK_INVITE_OPEN_ACTIVATE_COMMIT`.

`PH1.ONB`:
- require app-open fields on `ONB_SESSION_START_DRAFT`.
- require phone-capture enrollment progression references for completion gate.

`PH1.W`:
- add wake artifact-manifest sync enqueue contract (Engine B outbox handoff).

`PH1.VOICE.ID`:
- add voice artifact-manifest sync enqueue contract (Engine B outbox handoff).

`PH1.EMO.CORE` / `PH1.EMO.GUIDE`:
- lock phone-local artifact custody + sync-delta emission contract (future-built but pre-locked).

`ARTIFACTS_LEDGER_TABLES` alignment:
- all synced manifest rows map to immutable artifact ledger references (`artifact_type`, `artifact_version`, `package_hash`, `payload_ref`).
