# Selene OS Architecture Alignment and Obsolete Surface Retirement Plan

## 0. Authority and Scope

AGENTS.md controls execution.

This is docs-only planning. No runtime code was changed, no obsolete code was deleted, and no cleanup or refactor is authorized by this document.

Future removal requires explicit build instruction, approved file scope, canonical replacement proof, tests, backend evidence, JD live proof where visible, and active-caller check.

The governing lane for this planning document is:

- current project phase: PROBABILISTIC_FOUNDATION_BUILD
- selected lane: PROBABILISTIC_PUBLIC_ANSWER
- simulation required: no for this docs task
- authority required: no for this docs task
- state mutation allowed: no runtime state mutation
- protected execution allowed: no
- provider degradation allowed: no
- normal answer allowed: yes
- fail-closed required: no, because this is docs-only planning

## 1. New Architecture Target for Selene OS

Selene OS must become the runtime orchestration layer for:

- Provider Governance
- SemanticInterpreterProvider / GPT-5.5 proposal flow
- PH1.X deterministic validation
- PH1.WRITE final presentation
- Wake / Session / Voice ID / Access posture
- Selene Emotional Presentation + Quick Assist
- PH1.M scoped memory
- PH1.E evidence/tools/search/files
- Authority + Simulation protected execution
- audit and backend evidence
- Desktop/iPhone render-only
- Adapter transport-only

The new target is not a from-scratch rewrite. Current OS surfaces must be mapped, retained, migrated, or retired only after canonical replacement proof.

## 2. Current Selene OS Surface Inventory

| OS Surface / Module | File Path | Current Role | Current Status | New Architecture Role | Risk |
|---|---|---|---|---|---|
| OS crate exports | `crates/selene_os/src/lib.rs` | Exposes OS-level PH1 modules and compatibility surfaces | CURRENT_PARTIAL | Runtime orchestration export surface | Broad exports can keep old owners reachable until active-caller proof. |
| App ingress | `crates/selene_os/src/app_ingress.rs` | App/runtime ingress helper | CURRENT_PARTIAL | Admission boundary before PH1.C/PH1.X | Needs no-authority proof for client-originated fields. |
| Runtime bootstrap/foundation | `crates/selene_os/src/runtime_bootstrap.rs`; `runtime_session_foundation.rs`; `runtime_turns.rs`; `runtime_execution.rs`; `runtime_trace.rs` | Session, turn, authority posture, and runtime evidence carriers | CURRENT_PARTIAL | OS spine for session/access/authority/evidence | Exact architecture packet names differ; mapping must remain explicit. |
| PH1.X OS | `crates/selene_os/src/ph1x.rs` | Current-turn routing, active-context, deterministic follow-up helpers, Slice 3A proposal shell | CURRENT_PARTIAL | Deterministic validator of schema-bound semantic proposals | Contains deterministic time/weather/follow-up helpers that are compatibility risks. |
| PH1.X engine | `crates/selene_engines/src/ph1x.rs` | Deterministic directive engine | CURRENT_PARTIAL | PH1.X validation owner | Existing phrase/directive tests must not become new language architecture. |
| PH1.X contracts | `crates/selene_kernel_contracts/src/ph1x.rs` | `HumanConversationDirective`, `ActiveContextPacket`, `Ph1xDirective` | CURRENT_CANONICAL | Directive and current-context contract baseline | `SemanticMeaningProposalPacket` is not yet present. |
| PH1.WRITE OS | `crates/selene_os/src/ph1write.rs` | PH1.WRITE wiring and audit forwarding | CURRENT_PARTIAL | Final output boundary | Disabled passthrough is useful for compatibility but not full presentation policy. |
| PH1.WRITE engine | `crates/selene_engines/src/ph1write.rs` | Formatting, refusal preservation, one-line current equivalent | CURRENT_PARTIAL | Final display/tts validation owner | `run_one_line_current_equivalent` is compatibility-only until provider-assisted rewrite path is proven. |
| PH1.WRITE contracts | `crates/selene_kernel_contracts/src/ph1write.rs` | `Ph1WriteRequest`, `Ph1WriteOk`, `Ph1WriteResponse` | CURRENT_CANONICAL | Write output equivalent | Full `display_text`/`tts_text`/cards split remains activation work. |
| PH1.M | `crates/selene_os/src/ph1m.rs`; `crates/selene_engines/src/ph1m.rs`; `crates/selene_kernel_contracts/src/ph1m.rs` | Memory recall, fresh-memory handoff, evidence packets | CURRENT_PARTIAL | Scoped memory gateway | Must consume identity/access scope before private recall or durable preference. |
| PH1.E | `crates/selene_os/src/ph1e.rs`; `crates/selene_engines/src/ph1e.rs`; `crates/selene_kernel_contracts/src/ph1e.rs` | Tool, search, source chip, image/search evidence contracts | CURRENT_PARTIAL | Evidence/tools/search/files gateway | Must own source acceptance, prompt-injection defense, file/tool scope. |
| Web search plan | `docs/web_search_plan/*`; `crates/selene_os/src/web_search_plan/*` | URL fetch, source, vision/search planning references | CURRENT_PARTIAL | PH1.E evidence implementation substrate | Prompt-injection and accepted-source proof must be canonicalized. |
| PH1.D/provider route | `crates/selene_os/src/ph1d.rs`; `crates/selene_engines/src/ph1d.rs`; `crates/selene_kernel_contracts/src/ph1d.rs`; `crates/selene_adapter/src/lib.rs` | Provider call contracts, public answer route, model evidence tests | CURRENT_PARTIAL | Governed provider capability surface | Public-answer routes can bypass semantic spine if expanded without PH1.X. |
| Provider control | `crates/selene_engines/src/ph1providerctl.rs`; `docs/SELENE_OPENAI_MODEL_ROUTING_POLICY.md` | Provider registry, policy, fake/off counters, approved model policy | CURRENT_PARTIAL | Provider Governance baseline | Must be lifted into first foundation slice before live semantic provider use. |
| Wake | `crates/selene_os/src/ph1w.rs`; `crates/selene_engines/src/ph1w.rs`; `crates/selene_kernel_contracts/src/ph1w.rs` | Wake decision, liveness/replay/speaker gates | CURRENT_PARTIAL | Activation boundary and evidence-only wake | Wake acknowledgements must move to Quick Assist/PH1.WRITE, not hardcoded client logic. |
| Session / PH1.L | `crates/selene_os/src/ph1l.rs`; `crates/selene_engines/src/ph1l.rs`; `crates/selene_kernel_contracts/src/ph1l.rs`; `runtime_session_foundation.rs` | Session attach, access snapshots, stage packets | CURRENT_PARTIAL | Session binding and identity/access posture carrier | Exact `SessionIdentityBindingPacket` is not present; equivalents must be mapped. |
| Transcript / PH1.C | `crates/selene_os/src/ph1c.rs`; `crates/selene_engines/src/ph1c.rs`; `crates/selene_kernel_contracts/src/ph1c.rs` | Canonical ingress/admission family | CURRENT_PARTIAL | Transcript/input admission | Must remain admission, not semantic authority. |
| Voice ID | `crates/selene_os/src/ph1_voice_id.rs`; `crates/selene_engines/src/ph1_voice_id.rs`; `crates/selene_kernel_contracts/src/ph1_voice_id.rs` | Voice ID and enrollment evidence | CURRENT_PARTIAL | Speaker evidence only | Must never imply access or authority. |
| PH1.TTS | `crates/selene_os/src/ph1tts.rs`; `crates/selene_engines/src/ph1tts.rs`; `crates/selene_kernel_contracts/src/ph1tts.rs` | Approved TTS text, clean TTS checks, `VoiceRenderPlan`, `StyleProfileRef` | CURRENT_PARTIAL | Approved speech output | Persona style must pass PH1.WRITE/PH1.TTS approval. |
| PH1.PERSONA | `crates/selene_os/src/ph1persona.rs`; `crates/selene_engines/src/ph1persona.rs`; `crates/selene_kernel_contracts/src/ph1persona.rs` | Persona profile and tone-only guard surfaces | CURRENT_PARTIAL | Selene emotional presentation assist surface | Requires Selene activation before runtime personality implementation. |
| PH1.EMO.CORE | `crates/selene_os/src/ph1emocore.rs`; `crates/selene_engines/src/ph1emocore.rs`; `crates/selene_kernel_contracts/src/ph1emocore.rs` | Emotion signal bundle and tone guidance | CURRENT_PARTIAL | Emotional-state assist surface | Current signals are partial/advisory and not final persona policy. |
| PH1.EMO.GUIDE | `crates/selene_os/src/ph1emoguide.rs`; `crates/selene_engines/src/ph1emoguide.rs`; `crates/selene_kernel_contracts/src/ph1emoguide.rs` | Interaction guide with dominant/gentle/cooperative/assertive signals | CURRENT_PARTIAL | Tone guidance assist surface | Must not gain meaning, access, or execution authority. |
| PH1.FEEDBACK / PH1.LEARN | `crates/selene_os/src/ph1feedback.rs`; `crates/selene_os/src/ph1learn.rs`; matching contracts/engines | Feedback and learning surfaces | CURRENT_PARTIAL | Adaptation inputs under memory/privacy law | Durable learning needs PH1.M/access scope. |
| Enterprise ops | `crates/selene_os/src/ph1bcast.rs`; `ph1delivery.rs`; `ph1rem.rs`; `ph1onb.rs`; `ph1link.rs`; `ph1access.rs`; `ph1policy.rs`; `ph1gov.rs`; `ph1tenant.rs`; `ph1quota.rs`; `ph1work.rs`; `ph1lease.rs`; `ph1sched.rs`; `ph1health.rs`; `ph1kms.rs`; `ph1export.rs` | Broadcast, delivery, reminder, onboarding, access, tenant, quota, work, lease, health, KMS, export families | CURRENT_PARTIAL | Enterprise stacks behind semantic directive and deterministic gates | Side effects and access changes require activation packs before use. |
| Visual/media | `crates/selene_os/src/ph1vision.rs`; `crates/selene_engines/src/ph1vision.rs`; `crates/selene_engines/src/ph1vision_media.rs`; `crates/selene_kernel_contracts/src/ph1vision.rs` | OCR/image/media evidence and vision contracts | CURRENT_PARTIAL | Visual recognition/media evidence stack | OCR/source text must stay evidence, not instruction. |
| Artifact/document/export | `crates/selene_os/src/ph1art.rs`; `ph1doc.rs`; `ph1export.rs`; matching contracts/engines | Artifact trust, document, export/provenance families | CURRENT_PARTIAL | Artifact trust/document/export stack | Official/export claims require provenance and authority. |
| Provider assist/cost/cache | `crates/selene_os/src/ph1cost.rs`; `ph1prefetch.rs`; `ph1cache.rs`; `ph1pae.rs`; matching contracts/engines | Cost, prefetch, cache, provider assist/evaluation | CURRENT_PARTIAL | Provider assist/cost/arbitration stack | Hidden prefetch/provider attempts are forbidden. |
| Deterministic compute | `crates/selene_os/src/ph1comp.rs`; `crates/selene_engines/src/ph1comp.rs`; `crates/selene_kernel_contracts/src/ph1comp.rs` | Computation and consensus equivalents | CURRENT_PARTIAL | Calculation authority | Provider math cannot become official compute authority. |
| SimulationExecutor | `crates/selene_os/src/simulation_executor.rs`; `crates/selene_kernel_contracts/src/runtime_execution.rs`; `runtime_law.rs` | Protected execution and simulation certification equivalents | CURRENT_PARTIAL | Protected execution owner | Must remain fail-closed unless authority + simulation proof passes. |
| Adapter lib | `crates/selene_adapter/src/lib.rs` | HTTP/runtime bridge plus large compatibility and deterministic helper surface | WRONG_OWNER_RISK | Transport-only bridge after canonical replacement | Contains PH1.X, PH1.M, PH1.D, weather/time, protected, and fallback compatibility paths. |
| Adapter binaries | `crates/selene_adapter/src/bin/http_adapter.rs`; `grpc_adapter.rs`; `desktop_voice_e2e.rs`; `desktop_wake_life.rs` | Runtime transport, proof, and voice bridge entrypoints | CURRENT_PARTIAL | Transport/provenance only | Must not become semantic/access/provider authority. |
| Desktop | `apple/mac_desktop/SeleneMacDesktop/*` | Capture, render, TTS/playback, runtime bridge, proof views | CURRENT_PARTIAL | Capture/render/playback only | Client route parsing and fallback wording need no-authority proof. |
| iPhone | `apple/iphone/SeleneIPhone/*` | Render shell, voice/session/onboarding route rendering | CURRENT_PARTIAL | Capture/render/playback only | `inviteLike`/`appOpenLike` route parsing must remain display-only. |
| Storage/migrations | `crates/selene_storage/migrations/*.sql`; `crates/selene_storage/src/*` | Persistence and audit/event substrate | CURRENT_PARTIAL | Backend evidence store | New packet/evidence names may need mapped storage, not ad hoc logs. |
| Reports/blueprints | `docs/reports/*`; `docs/BLUEPRINTS/*` | Historical proof, architecture, and build evidence | COMPATIBILITY_RETAINED | Reference evidence only | Reports are not runtime authority. |

## 3. OS Owner Alignment Matrix

| Responsibility | Correct Owner Under New Architecture | Current OS Owner / Path | Alignment Status | Required Action |
|---|---|---|---|---|
| semantic interpretation | SemanticInterpreterProvider through Provider Governance; PH1.X validates | PH1.X active-context helpers; PH1.D public answer routes; Adapter compatibility helpers | PARTIAL / WRONG_OWNER_RISK | Build governed semantic proposal path; freeze deterministic phrase paths as compatibility. |
| PH1.X validation | PH1.X | `crates/selene_os/src/ph1x.rs`; contracts/engine PH1.X | PARTIAL | Convert provider proposals into validated directives; keep deterministic gates only. |
| PH1.WRITE presentation | PH1.WRITE | `crates/selene_os/src/ph1write.rs`; `crates/selene_engines/src/ph1write.rs` | PARTIAL | Expand to display/tts/source/image/artifact/persona final output validation. |
| wake/session | PH1.W / PH1.C / PH1.L | `ph1w.rs`; `ph1c.rs`; `ph1l.rs`; `runtime_session_foundation.rs` | PARTIAL | Bind wake/session without identity/access shortcuts; connect Quick Assist for wording. |
| Voice ID evidence | Voice ID owner | `ph1_voice_id.rs` family | PARTIAL | Evidence-only contract activation; no access/authority grant. |
| access scope | Access / Policy / Governance | `ph1access.rs`; `ph1policy.rs`; `ph1gov.rs`; `runtime_session_foundation.rs` | PARTIAL | Map `AccessScopePacket` to current equivalents and fail closed for private/protected gaps. |
| memory recall | PH1.M | `ph1m.rs` family plus Adapter memory-adjacent assertions | PARTIAL / WRONG_OWNER_RISK | PH1.M must consume access scope; retire Adapter memory assertions after proof. |
| search/tools/files | PH1.E | `ph1e.rs`; `web_search_plan/*`; Adapter/PH1.D public answer routes | PARTIAL / WRONG_OWNER_RISK | Route through PH1.E with source acceptance and prompt-injection defense. |
| provider routing | Provider Governance / PH1.D provider contracts | `ph1providerctl.rs`; `ph1d.rs`; Adapter tests/routes | PARTIAL | Provider-off/fake/model/cost/egress gates before semantic/writing providers. |
| persona/emotional tone | PH1.WRITE final; PH1.EMO/PH1.PERSONA assist | `ph1persona.rs`; `ph1emocore.rs`; `ph1emoguide.rs`; `ph1write.rs`; `ph1tts.rs` | PARTIAL | Selene activation pack; persona proposal must stay advisory and PH1.WRITE-approved. |
| Quick Assist | PH1.WRITE final; PH1.X/state owners provide facts | Quick Assist architecture official; runtime support scattered across PH1.WRITE/PH1.X/wake/TTS/tool presentation | PARTIAL | Quick Assist activation pack; remove hardcoded user-help behavior only after replacement. |
| TTS/STT | PH1.TTS / governed STT provider / voice admission owners | `ph1tts.rs`; Adapter HTTP voice paths; clients | PARTIAL | PH1.TTS speaks approved text only; STT transcript not identity. |
| protected risk | PH1.X classifies; Authority + Simulation decide | PH1.X protected directives; runtime execution; Adapter protected helpers | PARTIAL / WRONG_OWNER_RISK | Protected helpers outside canonical owners remain compatibility-only. |
| authority | Access/Governance/Authority owner | `runtime_execution.rs`; `runtime_session_foundation.rs`; access/governance contracts | PARTIAL | Exact authority decision mapping and fail-closed proof. |
| simulation execution | SimulationExecutor | `crates/selene_os/src/simulation_executor.rs` | CURRENT_CANONICAL | Preserve as only protected execution owner. |
| audit | Storage/Audit plus owner packets | migrations/storage repos/audit rows | PARTIAL | Standardize backend evidence per stack. |
| Desktop rendering | Desktop clients | `apple/mac_desktop/*` | PARTIAL | Render/capture/playback only; prove no semantic/access/provider authority. |
| Adapter transport | Adapter | `crates/selene_adapter/src/lib.rs`; adapter bins | WRONG_OWNER_RISK | Retain compatibility until canonical path proof; shrink to transport-only later. |

## 4. Obsolete / Conflicting Surface Ledger

| Surface / Symbol | File | Why It May Be Obsolete or Conflicting | Current Caller Evidence | Correct Replacement Owner | Retirement Condition | Earliest Safe Phase |
|---|---|---|---|---|---|---|
| `deterministic_active_context_followup_query` | `crates/selene_adapter/src/lib.rs` | Adapter performs current-context semantic follow-up logic | Called in Adapter runtime path around active-context fallback | SemanticInterpreterProvider proposal -> PH1.X validation | Semantic proposal path passes provider-off/fake/JD live and active-caller scan shows no dependency | OS-9 |
| `deterministic_weather_context_followup_query` | `crates/selene_adapter/src/lib.rs` | Adapter holds weather follow-up language logic | Called in Adapter follow-up path | PH1.X validated directive -> PH1.E weather/time tool -> PH1.WRITE presentation | Weather/time Quick Assist path proven with backend evidence | OS-9 |
| `deterministic_public_clarification_followup_query` | `crates/selene_adapter/src/lib.rs` | Adapter interprets clarification continuations by deterministic topic strings | Direct helper near deterministic time/weather topics | PH1.X clarification state + semantic proposal | Clarification proposal/validation tests and real route proof pass | OS-9 |
| `ph1m_actor_recent_recall_assertion` | `crates/selene_adapter/src/lib.rs` | Adapter-adjacent memory assertion risks memory brain outside PH1.M | Repo search finds symbol in Adapter compatibility surface | PH1.M scoped memory gateway | PH1.M memory scope + access posture proven; no callers remain | OS-9 |
| `maybe_run_ph1d_public_answer` | `crates/selene_adapter/src/lib.rs` | Adapter can invoke PH1.D public answer without full semantic spine | Referenced in Adapter public-answer route | Provider Governance + PH1.X directive + PH1.WRITE output | Public-answer replacement proves provider-off/fake/model evidence and no direct caller remains | OS-9 |
| `run_ph1d_public_answer` | `crates/selene_adapter/src/lib.rs` | Public provider route can become a parallel conversation brain | Referenced in Adapter and provider tests | Provider Governance + canonical owner route | Same as above, plus JD visible behavior proof | OS-9 |
| `h406_public_advisory_fallback_answer` | `crates/selene_adapter/src/lib.rs` | Hardcoded fallback answer can become deterministic UX | Repo search hit in Adapter fallback surface | PH1.WRITE + Quick Assist safe degraded wording | Provider-off degraded wording path proven through PH1.WRITE | OS-9 |
| `fallback_runtime_execution_envelope_for_voice_turn_request` | `crates/selene_adapter/src/lib.rs` | Adapter creates runtime envelope fallback | Called by Adapter voice-turn compatibility path | Runtime session foundation + PH1.C/PH1.L | Canonical voice/session envelope exists, evidence proves runtime ownership, no active callers | OS-9 |
| `fallback_runtime_execution_envelope_for_voice_turn_request_with_identities` | `crates/selene_adapter/src/lib.rs` | Adapter fallback may carry identity posture | Called by Adapter identity/voice compatibility path | PH1.L/session + Voice ID + Access scope owners | Wake/session/Voice ID/access OS posture proven; active-caller scan clean | OS-9 |
| `ph1x_universal_active_context_followup_query` | `crates/selene_os/src/ph1x.rs`; imported by Adapter | Useful current PH1.X helper, but deterministic language path | PH1.X tests and Adapter calls | Provider proposal -> PH1.X deterministic validation | Retain until GHCS semantic proposal vertical passes and old behavior regression passes | OS-9 |
| `Slice3aOneLineProviderProposal` | `crates/selene_os/src/ph1x.rs` | Narrow one-line proposal shell, not full schema-bound semantic packet | PH1.X Slice 3A tests | Versioned `SemanticMeaningProposalPacket` or repo-equivalent | Semantic packet activation and malformed rejection pass | OS-8/OS-9 |
| `DETERMINISTIC_TIME_CLARIFICATION_TOPIC` | `crates/selene_os/src/ph1x.rs`; Adapter constants | Deterministic time clarification marker can freeze old UX | PH1.X/Adapter time clarification tests | PH1.X directive state + Quick Assist wording | Time clarification route proven via semantic proposal and PH1.WRITE | OS-9 |
| `DETERMINISTIC_WEATHER_CLARIFICATION_TOPIC` | `crates/selene_os/src/ph1x.rs`; Adapter constants | Deterministic weather clarification marker can freeze old UX | PH1.X/Adapter weather clarification tests | PH1.X directive state + PH1.E weather + PH1.WRITE | Weather clarification route proven via semantic proposal and PH1.WRITE | OS-9 |
| deterministic weather/time output helpers | `crates/selene_os/src/ph1x.rs` | PH1.X owns fixed weather/time presentation fragments | Tests around weather/time follow-up and clarification | PH1.E fact owner + GPT-5.5/Quick Assist wording + PH1.WRITE validation | Provider-off/fake-safe natural presentation tests pass | OS-9 |
| `run_one_line_current_equivalent` | `crates/selene_engines/src/ph1write.rs` | Deterministic current-equivalent rewrite path | PH1.WRITE one-line tests | PH1.WRITE final validation of provider-assisted rewrite | Semantic proposal + PH1.X target + PH1.WRITE provider/fake/off rewrite proof | OS-9 |
| PH1.X leadership/CEO phrase ranking | `crates/selene_os/src/ph1x.rs` | Contains/keyword search-topic weighting risks phrase-patch architecture | `contains`-style source/search heuristics in PH1.X | Semantic proposal and PH1.E source-backed answer route | Source-backed intent path proves no phrase hijack | OS-9 |
| Desktop route parsing | `apple/mac_desktop/SeleneMacDesktop/*` | Client app can parse route-like strings for invite/open states | Desktop session and proof views contain route/render logic | Cloud runtime route packet + Desktop render-only | Client route presentation activation passes no-authority proof | OS-9 |
| iPhone `inviteLike` / `appOpenLike` / `openLike` | `apple/iphone/SeleneIPhone/SessionShellView.swift` | Client route parsing can drift into route authority | iPhone session shell route parsing hits | PH1.L/PH1.ONB/PH1.LINK cloud-authored route packets | Client route presentation proof and active-caller scan | OS-9 |
| local/fallback TTS or STT success wording | Desktop/iPhone/Adapter voice surfaces | Client/local fallback wording may mask runtime failure | Desktop shells include fail-closed/no-local-success proof text | PH1.TTS approved text + runtime evidence | Voice route proof and TTS-safe output pass | OS-9 |
| provider bypass paths | Adapter/PH1.D public answer surfaces | Provider calls outside Governance or PH1.X can become authority | Provider model and live route tests exist in Adapter | Provider Governance + owner request packets | Provider baseline proves zero hidden attempts and model policy | OS-8/OS-9 |
| memory outside PH1.M | Adapter memory-adjacent assertions and session shortcuts | Memory lookup/claims outside PH1.M risk privacy drift | Adapter `ph1m_actor_recent_recall_assertion` and context shortcuts | PH1.M + access scope | Memory scope activation + no active callers | OS-9 |
| search/tool/file logic outside PH1.E | Adapter public answer and web/search route helpers | Evidence logic outside PH1.E risks unsupported claims | Adapter PH1.D public-answer route and web search plan references | PH1.E source/tool/file gateway | PH1.E source acceptance and prompt-injection proof | OS-9 |
| protected action outside Authority + SimulationExecutor | Adapter protected/payroll helpers; PH1.X protected hints | Protected classification/helper logic can imply execution | Adapter protected/payroll tests and runtime envelopes | PH1.X classification + Authority + SimulationExecutor | Protected fail-closed and simulation proof pass | OS-9 |

Deletion is not authorized for any row in this ledger.

## 5. Keep / Migrate / Remove Classification

| Surface | Classification | Reason |
|---|---|---|
| `crates/selene_kernel_contracts/src/ph1x.rs::HumanConversationDirective` | KEEP_CURRENT_CANONICAL | Current directive contract is canonical enough to retain while semantic proposal packet is added. |
| `crates/selene_kernel_contracts/src/ph1write.rs` | KEEP_CURRENT_CANONICAL | PH1.WRITE contract is current output equivalent; needs extension/mapping, not removal. |
| `crates/selene_kernel_contracts/src/ph1e.rs::SourceChipPacket` | KEEP_CURRENT_CANONICAL | Source chip contract exists and aligns with PH1.E/PH1.WRITE presentation. |
| `crates/selene_kernel_contracts/src/ph1m.rs::MemoryEvidencePacket` | KEEP_CURRENT_CANONICAL | Memory evidence packet exists and should remain PH1.M-owned. |
| `crates/selene_os/src/simulation_executor.rs` | KEEP_CURRENT_CANONICAL | Correct protected execution owner. |
| `crates/selene_os/src/runtime_session_foundation.rs` access/authority stage packets | KEEP_CURRENT_CANONICAL | Current equivalent of session/access posture; mapping needs activation. |
| `crates/selene_os/src/ph1x.rs::ph1x_universal_active_context_followup_query` | RETAIN_COMPATIBILITY_UNTIL_PROOF | Useful current PH1.X route, but deterministic language path must be replaced by provider proposal + validation. |
| `crates/selene_os/src/ph1x.rs` weather/time deterministic helpers | RETAIN_COMPATIBILITY_UNTIL_PROOF | Current behavior coverage exists; cannot delete until Quick Assist and PH1.E replacement is proven. |
| `crates/selene_engines/src/ph1write.rs::run_one_line_current_equivalent` | RETAIN_COMPATIBILITY_UNTIL_PROOF | Current one-line vertical proof exists; provider-assisted rewrite must replace after proof. |
| Adapter deterministic follow-up helpers | MIGRATE_TO_CANONICAL_OWNER | Meaning/target resolution belongs in PH1.X, not Adapter. |
| Adapter PH1.M memory assertion helper | MIGRATE_TO_CANONICAL_OWNER | Memory belongs in PH1.M with access scope. |
| Adapter PH1.D public answer wrappers | MIGRATE_TO_CANONICAL_OWNER | Provider public answers must be governed and routed through PH1.X/PH1.WRITE/PH1.E as applicable. |
| Adapter fallback runtime envelopes | RETAIN_COMPATIBILITY_UNTIL_PROOF | Needed until canonical voice/session envelope path is proven. |
| Desktop/iPhone invite/app-open parsing | RETAIN_COMPATIBILITY_UNTIL_PROOF | Client route display is useful, but authority must move to cloud-authored packets. |
| Hardcoded wake/weather/help wording | REMOVE_AFTER_PROOF | Likely obsolete as primary UX once Quick Assist/Selene/PH1.WRITE provider-safe wording exists. |
| Any provider route without governance evidence | STALE_DANGEROUS_REQUIRES_BLOCKER | Future provider builds must block if a direct provider call bypasses governance. |
| Any protected action outside Authority + SimulationExecutor | STALE_DANGEROUS_REQUIRES_BLOCKER | Protected execution must fail closed unless canonical authority/simulation proof exists. |
| Video recognition/rendering runtime owner | REPO_TRUTH_NEEDED | Architecture names the stack; exact runtime ownership remains partial/unclear. |
| Unknown report-only surfaces | UNKNOWN_DO_NOT_TOUCH | Reports are evidence, not runtime owners; do not remove without explicit audit. |

## 6. Probabilistic Human Interaction Alignment

Human communication with Selene must be probabilistic-first through GPT-5.5 / approved OpenAI provider where applicable.

Selene OS must not implement deterministic language understanding through phrase lists, keyword contains routing, hardcoded wake phrases, hardcoded weather/time templates, or stack-local conversation brains.

Correct user interaction wiring:

`User input / process state -> GPT-5.5 semantic or persona proposal where applicable -> PH1.X or correct owner validates state, scope, risk -> PH1.WRITE approves final display_text / tts_text -> PH1.TTS speaks approved text -> Desktop/iPhone render/play only`

OS surfaces that violate or risk violating this model:

- `crates/selene_adapter/src/lib.rs::deterministic_active_context_followup_query`
- `crates/selene_adapter/src/lib.rs::deterministic_weather_context_followup_query`
- `crates/selene_adapter/src/lib.rs::deterministic_public_clarification_followup_query`
- Adapter `maybe_run_ph1d_public_answer` / `run_ph1d_public_answer`
- PH1.X deterministic time/weather clarification topics and weather/time presentation helpers
- PH1.X `ph1x_universal_active_context_followup_query` until schema-bound semantic proposals are built
- PH1.WRITE `run_one_line_current_equivalent` until provider-assisted rewrite is proven
- Desktop/iPhone invite/open route parsing if treated as authority rather than render-only state
- Any future hardcoded wake greeting, deterministic clarification phrase list, deterministic user-help flow, or stack-local language parser

Allowed deterministic logic remains:

- provider-off/fake-provider gates
- schema validation
- identity/access scope
- privacy/data-egress checks
- memory permission
- source acceptance
- tool permission
- protected-risk classification
- authority
- simulation
- audit/idempotency
- state mutation

## 7. Selene Emotional Intelligence / Emotional Engine OS Alignment

Current emotional/persona files inspected:

- `crates/selene_kernel_contracts/src/ph1emocore.rs`
- `crates/selene_engines/src/ph1emocore.rs`
- `crates/selene_os/src/ph1emocore.rs`
- `crates/selene_kernel_contracts/src/ph1emoguide.rs`
- `crates/selene_engines/src/ph1emoguide.rs`
- `crates/selene_os/src/ph1emoguide.rs`
- `crates/selene_kernel_contracts/src/ph1persona.rs`
- `crates/selene_engines/src/ph1persona.rs`
- `crates/selene_os/src/ph1persona.rs`
- `crates/selene_os/src/ph1feedback.rs`
- `crates/selene_os/src/ph1learn.rs`
- `crates/selene_os/src/ph1m.rs`
- `crates/selene_os/src/ph1write.rs`
- `crates/selene_os/src/ph1tts.rs`
- provider governance / PH1.D files
- Desktop/iPhone render and TTS surfaces
- Adapter transport surfaces

What exists now:

- PH1.EMO.CORE has `EmoSignalBundle` fields including `assertive_score`, `distress_score`, `anger_score`, and `warmth_signal`.
- PH1.EMO.CORE has `EmoToneGuidance` with style profile refs, modifiers, pacing guidance, directness level, and empathy level.
- PH1.EMO.CORE engine maps snapshot refs containing `dom`, `gentle`, or `passive` into dominant/gentle-style posture and maps assertive/anger/warmth signals into brief/formal/warm modifiers.
- PH1.EMO.GUIDE has interaction signals including `assertive_events` and `cooperative_events`, and tests for dominant/assertive and gentle/cooperative behavior.
- PH1.PERSONA maps style profile tokens such as dominant/assertive/direct and gentle/calm/warm into persona style refs.
- PH1.PERSONA OS tests include unknown-speaker no-persona behavior and tone-only/no-execution authority guardrails.
- PH1.TTS has `StyleProfileRef`, `VoiceRenderPlan`, and clean approved TTS text checks.
- PH1.WRITE is the current final wording contract/wiring equivalent.

Does the current system detect user demeanor signals and adjust Selene tone accordingly?

Yes, partially. The repo already supports some demeanor signal detection and advisory tone guidance:

- assertive: PARTIAL/YES through `assertive_score`, `assertive_events`, and assertive style tokens.
- dominant: PARTIAL/YES through `StyleProfileRef::Dominant` and `dom`/dominant-style mapping.
- passive: PARTIAL through snapshot refs containing `passive` that map to gentle style; no first-class passive score was found.
- gentle: PARTIAL/YES through gentle style mapping and gentle/cooperative guide tests.
- cooperative: PARTIAL/YES through `cooperative_events`.
- frustrated: PARTIAL through anger/distress/correction/friction-like signals; no fully named frustration detector was confirmed.
- confused: PARTIAL/MISSING as a first-class signal; likely inferred only from clarification/correction state today.
- distressed: PARTIAL/YES through `distress_score`.
- warm: PARTIAL/YES through `warmth_signal` and warm modifiers.
- direct: PARTIAL/YES through direct style tokens and directness level.

Can it choose complementary Selene tone modes?

- Calmer response for dominant/assertive user: PARTIAL. Existing guidance can shift brief/formal/fast for assertive/dominant posture, but a complementary "calmer response" Selene policy is not fully wired through PH1.WRITE.
- More guiding/assertive response for passive/uncertain user: PARTIAL. Passive/gentle signals exist, but the guiding/assertive complementary policy needs Selene activation.
- Gentle response for stressed user: PARTIAL. Distress/empathy signals exist, but final PH1.WRITE/PH1.TTS Selene behavior is not fully activated.
- Serious response for protected/high-risk context: PARTIAL/MISSING. Protected context exists in PH1.X/runtime execution, but Selene serious-mode policy needs explicit PH1.WRITE integration.
- Playful response for normal safe chat: ARCHITECTURE_ONLY/PARTIAL. Persona style exists, but GPT-5.5-assisted playful wording through Provider Governance and PH1.WRITE is not implemented as final Selene runtime.

Current classification:

`PARTIAL_REQUIRES_ACTIVATION_PACK`

Advisory-only parts:

- PH1.EMO.CORE tone guidance.
- PH1.EMO.GUIDE interaction style guidance.
- PH1.PERSONA style profile hints.
- PH1.FEEDBACK / PH1.LEARN adaptation surfaces until memory/privacy law and PH1.WRITE approval are wired.

Missing for full Selene runtime:

- Selene Emotional Intelligence + Relationship Presence Activation Pack.
- Provider-governed GPT-5.5 persona/wording proposal path.
- PH1.WRITE persona policy enforcement for final `display_text` and `tts_text`.
- PH1.TTS approved personality speech path.
- Serious-mode integration for protected/access-denial/high-risk contexts.
- PH1.M durable tone preference storage only where memory law allows.
- Desktop/iPhone render-only proof for personality output.
- Adapter transport-only proof for personality output.
- JD live/eval pack for demeanor-to-tone behavior.

Surfaces that must connect to PH1.WRITE:

- PH1.EMO.CORE tone guidance.
- PH1.EMO.GUIDE interaction guidance.
- PH1.PERSONA style profile output.
- PH1.M allowed durable preferences.
- PH1.FEEDBACK/PH1.LEARN approved adaptation hints.
- Provider-governed GPT-5.5 wording proposal.
- PH1.TTS `VoiceRenderPlan` / approved text.

Surfaces that must not gain authority:

- PH1.EMO.CORE.
- PH1.EMO.GUIDE.
- PH1.PERSONA.
- PH1.FEEDBACK.
- PH1.LEARN.
- PH1.M preference hints.
- Provider persona output.
- PH1.TTS style rendering.
- Desktop/iPhone/Adapter.

Likely Selene/persona implementation slices needed after activation pack:

1. Selene emotional presentation activation and repo-symbol mapping.
2. Provider-governed fake persona/wording proposal shell.
3. PH1.EMO.CORE / PH1.EMO.GUIDE / PH1.PERSONA handoff into PH1.WRITE.
4. PH1.WRITE / PH1.TTS final personality output with serious-mode and no-authority guards.
5. PH1.M preference + PH1.FEEDBACK/PH1.LEARN boundary with JD live/eval proof.

Proof required:

- provider-off zero persona provider attempts.
- fake-provider persona proposal accepted only as wording proposal.
- PH1.WRITE rejects unsafe, authority-granting, unsupported, or protected-inappropriate persona text.
- PH1.TTS speaks only approved personality text.
- unknown speaker gets no private personalization.
- protected/action-denial contexts use serious wording.
- Desktop/iPhone render/play only.
- Adapter transport only.
- backend evidence shows emotional/persona refs and PH1.WRITE final approval.
- JD live acceptance for demeanor/tone scenarios.

## Selene Emotional Intelligence + Emotional Engine Deep Activation Review

1. Current emotional/persona capability exists as a partial advisory system. PH1.EMO.CORE tracks assertive/distress/anger/warmth signals and produces tone guidance; PH1.EMO.GUIDE tracks assertive/cooperative interaction signals and dominant/gentle guide outcomes; PH1.PERSONA maps style profile refs and enforces tone-only/no-authority guardrails; PH1.TTS has style profile rendering surfaces; PH1.WRITE owns the final wording equivalent.

2. Current demeanor signal support:

| Signal | Current support | Evidence | Status |
|---|---|---|---|
| assertive | `assertive_score`, `assertive_events`, assertive style tokens | PH1.EMO.CORE, PH1.EMO.GUIDE, PH1.PERSONA | PARTIAL/YES |
| dominant | dominant style refs and `dom` snapshot mapping | PH1.EMO.CORE, PH1.EMO.GUIDE, PH1.PERSONA | PARTIAL/YES |
| passive | snapshot refs containing `passive` map toward gentle style | PH1.EMO.CORE | PARTIAL |
| gentle | gentle style refs and guide tests | PH1.EMO.CORE, PH1.EMO.GUIDE, PH1.PERSONA | PARTIAL/YES |
| cooperative | `cooperative_events` | PH1.EMO.GUIDE | PARTIAL/YES |
| frustrated | anger/distress/correction/friction-adjacent signals | PH1.EMO.CORE / related guide surfaces | PARTIAL |
| confused | no confirmed first-class confusion detector | PH1.X clarification state may be adjacent | PARTIAL/MISSING |
| distressed | `distress_score` | PH1.EMO.CORE | PARTIAL/YES |
| warm | `warmth_signal` and warm modifiers | PH1.EMO.CORE / PH1.PERSONA | PARTIAL/YES |
| direct | direct style token and directness level | PH1.PERSONA / PH1.EMO.CORE | PARTIAL/YES |

3. Complementary Selene tone mode support:

| Desired Selene mode | Current support | Gap |
|---|---|---|
| calmer response for dominant/assertive user | PARTIAL | Needs Selene policy and PH1.WRITE enforcement. |
| more guiding/assertive response for passive/uncertain user | PARTIAL | Passive/uncertain mapping is not a full complementary-response policy. |
| gentle response for stressed user | PARTIAL | Distress exists; final Selene wording/TTS integration is missing. |
| serious response for protected/high-risk context | PARTIAL/MISSING | Must connect PH1.X protected risk and Authority/Simulation posture to PH1.WRITE persona policy. |
| playful response for normal safe chat | ARCHITECTURE_ONLY/PARTIAL | Requires governed GPT-5.5 wording and PH1.WRITE final approval. |

4. Already implemented:

- Emotional signal fields for assertive, distress, anger, warmth.
- Tone guidance contract and engine surfaces.
- Dominant/gentle/cooperative guide tests.
- Persona style profile mapping.
- Tone-only/no-execution-authority guardrails.
- Unknown speaker no-persona behavior.
- TTS style profile and clean approved TTS text checks.

5. Advisory only:

- Tone guidance from PH1.EMO.CORE.
- Interaction guide output from PH1.EMO.GUIDE.
- Persona style profile hints.
- Feedback/learning adaptation hints.
- Provider-generated personality wording until PH1.WRITE approves it.

6. Missing for full Selene runtime:

- Selene activation pack.
- Provider-governed persona proposal interface.
- PH1.WRITE final persona policy.
- PH1.TTS approved personality speech path.
- PH1.M scoped preference storage.
- Serious-mode and protected-risk tone binding.
- Multilingual persona consistency.
- JD live/eval proof.

7. GPT-5.5 should participate as a probabilistic wording, humor, emotional phrasing, guidance, clarification, and tone-continuity proposer behind Provider Governance. It must not decide identity, access, memory permission, authority, protected execution, state mutation, or final output.

8. PH1.WRITE should validate final personality output against directive, evidence, persona policy, seriousness mode, safety boundaries, supported claims, access/privacy limits, and TTS suitability.

9. PH1.TTS should speak only PH1.WRITE-approved `tts_text` / approved TTS text with the approved voice/style plan. It must not rewrite meaning or add jokes.

10. PH1.M should store tone preferences only where memory law allows, identity/access scope is sufficient, and the user has granted or policy allows durable preference retention.

11. Desktop/iPhone/Adapter must never own persona choice, emotional inference, access, authority, memory permission, provider choice, protected execution, or final wording. Desktop/iPhone render/play; Adapter transports.

12. Likely implementation count is 5 Selene/persona slices after a dedicated activation pack.

Recommended next docs task:

`CREATE_SELENE_PERSONA_AND_EMOTIONAL_ENGINE_ACTIVATION_PACK`

Proposed slices only:

1. Selene Emotional Intelligence Repo-Truth Activation and Owner Map.
2. Provider-Governed Persona Proposal Shell with Fake/Off Proof.
3. PH1.EMO.CORE / PH1.EMO.GUIDE / PH1.PERSONA to PH1.WRITE Handoff.
4. PH1.WRITE / PH1.TTS Selene Final Output and Serious-Mode Guardrails.
5. PH1.M Preference, PH1.FEEDBACK/PH1.LEARN Adaptation, Eval, Backend Evidence, and JD Live Proof.

## 8. Quick Assist OS Alignment

Quick Assist should support:

- wake acknowledgement
- "I'm here" / natural greeting
- quick clarify
- quick confirm
- quick suggest
- user-lost-in-process help
- weather/time natural presentation
- tool/result explanation
- error/failure recovery wording
- TTS-safe friendly phrasing

Current OS support:

- Wake/session state exists in PH1.W/PH1.C/PH1.L and runtime session foundation.
- PH1.X can produce clarify/respond/dispatch directives.
- PH1.E can produce tool/search/time/weather result equivalents through `ToolResult`.
- PH1.WRITE can format/preserve final text equivalents.
- PH1.TTS can validate clean approved speech text.
- Desktop/iPhone can render and play approved runtime payloads.

Missing support:

- Provider-governed Quick Assist wording proposal path.
- PH1.WRITE validation policy for natural guidance, process help, weather/time presentation, and recovery wording.
- State/fact handoff contract from deterministic owners into Quick Assist.
- Provider-off/fake-provider proof for Quick Assist.
- JD live scenarios for wake acknowledgement, user-lost help, tool explanation, failed-step recovery, and TTS-safe friendly phrasing.

Obsolete deterministic UX paths:

- hardcoded wake acknowledgement/greeting patterns if used as primary UX.
- PH1.X/Adapter deterministic time/weather clarification wording.
- PH1.WRITE/PH1.X one-line and clarification phrase helpers if expanded as language architecture.
- client/local fallback wording that appears to answer without runtime evidence.

Quick Assist does not execute actions, grant authority, mutate state, or bypass PH1.X, PH1.WRITE, PH1.E, PH1.M, Authority, SimulationExecutor, or Provider Governance.

## 9. Provider Governance OS Alignment

Current provider governance evidence:

- `docs/SELENE_OPENAI_MODEL_ROUTING_POLICY.md` defines approved model policy surfaces and forbids unapproved model substitution.
- `crates/selene_engines/src/ph1providerctl.rs` contains provider network policy, provider call counters, governance evidence envelope, registry entry, and route decision equivalents.
- `crates/selene_kernel_contracts/src/ph1d.rs` contains PH1.D provider call request/response and transport evidence equivalents.
- Adapter tests include provider-off/fake/live model evidence and `gpt-5.5` proof surfaces.

Alignment needs:

- Approved model policy use must be explicit for semantic, writing, search, STT, TTS, vision, image, video, and eval functions.
- Provider-off must prove zero attempts and zero network dispatches.
- Fake-provider paths must be non-billable and schema-bound.
- Budget/cost counters and network dispatch counters must be part of evidence.
- Model evidence must capture expected/actual/model sent where applicable.
- Fallback behavior must not silently use cheaper or unapproved models.
- Startup must not probe providers.
- Data-egress/privacy classification must happen before provider calls.

Risks:

- Adapter/PH1.D public-answer paths can become provider bypass surfaces.
- Provider fallback tests must remain proof tools, not new live default behavior.
- Quick Assist and Selene must not start live OpenAI use until Provider Governance permits it.

## 10. Wake / Session / Voice ID / Access OS Alignment

Current OS support:

- Wake: `ph1w.rs` family includes wake decisions, liveness/replay/speaker gate equivalents.
- Session: `ph1l.rs` and `runtime_session_foundation.rs` include session attach/access/authority posture equivalents such as `SessionAccessSnapshot` and `Stage6AccessContextPacket`.
- Transcript/admission: PH1.C family exists as canonical ingress/admission surface.
- Voice ID: `ph1_voice_id.rs` family exists and should remain evidence-only.
- Access: `ph1access.rs`, `ph1policy.rs`, `ph1gov.rs`, `runtime_execution.rs`, and `runtime_session_foundation.rs` provide access/governance/authority equivalents.
- Authority/protected posture: `AuthorityExecutionState`, `SimulationCertificationState`, and SimulationExecutor equivalents exist.

Missing or partial:

- Exact architecture packet names such as `SessionIdentityBindingPacket`, `AccessScopePacket`, `SpeakerIdentityEvidencePacket`, and `AuthorityDecisionPacket` are not all present by exact name.
- Wake/session/Voice ID/access early spine needs one activation pack that maps current equivalents and proves defaults.
- PH1.X must validate semantic proposals against identity/access posture.
- PH1.M and PH1.E must consume access scope before private memory, files, connectors, and tools.
- Authority + Simulation fail-closed evidence needs product-level proof.

Obsolete or risky surfaces:

- Voice ID or wake state must not be treated as access/authority.
- Desktop/iPhone wake or voice UI must not grant identity/access.
- Adapter fallback envelopes with identities must not become authority.

## 11. PH1.X / PH1.WRITE / PH1.M / PH1.E OS Alignment

### PH1.X

- Current role: deterministic directive and active-context owner.
- Correct new role: validate provider semantic proposals into lawful directives.
- Conflicts: deterministic time/weather/current-context phrase helpers and Adapter-imported PH1.X helpers.
- Missing activation: versioned semantic proposal packet, malformed rejection, candidate ledgers, owner reroute proof, protected fail-closed proof.
- Retain temporarily: `ph1x_universal_active_context_followup_query`, deterministic time/weather clarification topics, Slice 3A proposal shell.
- Retire later: phrase/contains routing and deterministic presentation helpers after semantic path proof.

### PH1.WRITE

- Current role: final formatting/writing equivalent.
- Correct new role: final human output, source/image/artifact/persona/TTS-safe presentation validation.
- Conflicts: deterministic one-line current-equivalent helper if treated as primary language architecture.
- Missing activation: provider-governed writing/persona proposal path, display/tts/source/image/video/artifact card policy, Quick Assist, Selene policy.
- Retain temporarily: one-line current-equivalent proof path.
- Retire later: deterministic language transformations that are replaced by governed provider proposals and PH1.WRITE validation.

### PH1.M

- Current role: memory/evidence/fresh-memory handoff owner.
- Correct new role: scoped memory and preference gateway with identity/access checks.
- Conflicts: Adapter memory-adjacent assertions and session shortcut risks.
- Missing activation: access scope consumption, unknown speaker denial, preference law, durable tone preference boundaries.
- Retain temporarily: current PH1.M memory contracts and fresh-memory path.
- Retire later: memory assertions outside PH1.M.

### PH1.E

- Current role: tool/search/file/source/image result owner.
- Correct new role: evidence/search/tool/file/connector gateway.
- Conflicts: public answer/search-like logic through Adapter/PH1.D can bypass PH1.E.
- Missing activation: prompt-injection defense packet/equivalent, source acceptance ledger, claim verification, file/connector/tool scope, image/video evidence and presentation bridge.
- Retain temporarily: current `ToolRequest`, `ToolResult`, `SourceChipPacket`, `SearchImagePacket`, `ClaimVerificationPacket` equivalents.
- Retire later: source/search/tool logic outside PH1.E after proof.

## 12. Desktop / iPhone / Adapter Boundary Alignment

Desktop/iPhone are render/capture/playback only.

Adapter is transport only.

Neither Desktop, iPhone, nor Adapter may own meaning, memory, provider choice, access, authority, protected execution, or persona brain.

Current risks:

- Desktop session and proof views contain rich local state/rendering and fallback/no-local-success logic. These should remain proof/render shells only.
- iPhone `SessionShellView.swift` contains route-like parsing such as `inviteLike`, `appOpenLike`, and `openLike`; this must remain client presentation and never route authority.
- Adapter `lib.rs` is a large compatibility surface with deterministic PH1.X-style helpers, memory-adjacent assertions, PH1.D public-answer routes, protected helper tests, and fallback runtime envelopes.
- Desktop/iPhone TTS surfaces must use approved runtime text, not locally generated meaning.
- Adapter identity-bearing fallback envelopes must not become authority.

Required proof before cleanup:

- Desktop/iPhone render only approved packets from runtime.
- Desktop/iPhone do not decide semantic route, access, authority, provider model, memory permission, or protected execution.
- Adapter transports canonical packets and provenance only.
- Canonical PH1.X, PH1.WRITE, PH1.M, PH1.E, Provider Governance, Access/Authority, and SimulationExecutor replacements exist.
- Backend evidence matches visible/audible behavior.
- JD live passes where visible.
- Active-caller scan proves no live caller remains on old compatibility paths.

## 13. OS Refactor Phases

| Phase | Purpose | Files to Inspect | Expected Future Edit Scope | Proof Required | Old Paths Retained | Deletion Allowed |
|---|---|---|---|---|---|---|
| OS-0: Read-only OS alignment plan | Create this plan and classify OS surfaces | Required docs, OS/adapter/client/provider/persona files | Docs only | Clean tree, docs-only diff | All old paths retained | no |
| OS-1: Provider Governance OS foundation | Normalize provider registry, model policy, fake/off, counters, evidence | `ph1providerctl.rs`, `ph1d.rs`, Adapter provider routes/tests, model policy doc | Provider governance docs/contracts/runtime only by future scope | provider-off zero attempts, fake provider, malformed rejection, model evidence | Adapter PH1.D public-answer paths | no |
| OS-2: Semantic proposal OS routing shell | Introduce governed semantic proposal shell | PH1.X, PH1.D, provider governance, Adapter follow-up callers | Semantic provider interface/skeleton and tests | schema validation, fake/off proof, no phrase patch expansion | PH1.X/Adapter deterministic follow-up helpers | no |
| OS-3: Wake/Session/Voice ID/Access OS posture | Map wake/session/voice/access defaults | PH1.W/C/L, Voice ID, access/policy/gov, runtime session foundation | Posture packets/equivalents and tests | unknown speaker public-safe, private/protected fail closed | Adapter identity fallback envelopes | no |
| OS-4: PH1.X directive OS wiring | Validate semantic proposals against session/access/risk | PH1.X, runtime execution, PH1.M/E/WRITE handoffs | PH1.X validation and evidence only by future scope | owner routing, ambiguity, protected fail-closed | deterministic context helpers | no |
| OS-5: PH1.WRITE + Quick Assist + Selene output OS wiring | Route natural guidance/persona through PH1.WRITE/PH1.TTS | PH1.WRITE, PH1.TTS, PH1.EMO, PH1.PERSONA, PH1.M, provider governance | Writing/persona/Quick Assist shells | provider-off/fake wording, PH1.WRITE validation, TTS-safe, no authority | hardcoded greetings/weather/help/one-line helpers | no |
| OS-6: PH1.M/PH1.E scope and evidence OS wiring | Scope memory/files/tools/search/evidence | PH1.M, PH1.E, access, web_search_plan, tools/files/connectors | Memory/evidence gateway integration | source acceptance, memory scope, prompt-injection, file/tool scope | Adapter memory/search shortcuts | no |
| OS-7: Desktop/iPhone/Adapter boundary proof | Prove client/adapter no-authority boundaries | Desktop/iPhone views/bridges, Adapter lib/bins | Proof/test edits by future scope | render-only, transport-only, backend evidence | route parsing/fallbacks retained | no |
| OS-8: active-caller ledger for obsolete paths | Build caller map for retirement candidates | Adapter lib, PH1.X, PH1.WRITE, clients, provider routes | Docs/tests/ledger only | active-caller scan, dependency graph, old behavior regression | all candidates retained | no |
| OS-9: proof-based obsolete surface retirement | Retire specific old paths after proof | Only files explicitly approved by future retirement instruction | Narrow deletion/migration only after proof | canonical replacement, tests, backend evidence, JD live, no active caller | only unreplaced paths retained | yes, per approved slice only |

## 14. Obsolete Removal Rules

No obsolete code may be removed merely because it looks old.

Removal is allowed only when:

- canonical replacement exists
- backend evidence proves replacement
- tests pass
- JD live passes where visible
- active-caller scan proves no live caller
- old behavior regression passes
- file scope is explicitly approved
- clean tree is preserved

If Codex finds stale or conflicting OS code but cannot safely remove it, classify it and leave it for a future retirement slice.

## 15. Recommended Next OS Build Queue

1. Build Provider Governance OS Foundation.
2. Build Semantic Proposal OS Routing Shell.
3. Build Wake / Session / Voice ID / Access OS Posture Baseline.
4. Build PH1.X Directive Validation OS Wiring.
5. Build PH1.WRITE Quick Assist / Selene Output Shell.
6. Build PH1.M Scoped Memory and Preference Boundary.
7. Build PH1.E Source / Tool / File Evidence Boundary.
8. Build Desktop / iPhone / Adapter No-Authority Boundary Proof.
9. Build Obsolete Surface Active-Caller Ledger.
10. Build First Proof-Based Obsolete Surface Retirement Slice.

Recommended next docs task for persona:

`CREATE_SELENE_PERSONA_AND_EMOTIONAL_ENGINE_ACTIVATION_PACK`

## 16. Final Status

- Required docs read: yes.
- OS surfaces inspected: yes, including `crates/selene_os/src`, `crates/selene_kernel_contracts/src`, `crates/selene_engines/src`, `crates/selene_adapter/src`, `crates/selene_tools/src`, storage migrations, Desktop, iPhone, reports, web search plan, and blueprints.
- Obsolete surfaces identified: yes.
- Selene/persona/emotional engine alignment reviewed: yes.
- Adapter/Desktop/iPhone boundary risks reviewed: yes.
- Runtime code changed: no.
- Obsolete code deleted: no.
- Deletion authorized: no.
- Master index update required: yes, this document should be listed as the next architecture/planning document.
- Final clean tree status: to be verified after docs-only commit/push flow.
