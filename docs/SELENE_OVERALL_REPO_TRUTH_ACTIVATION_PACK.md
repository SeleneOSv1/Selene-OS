# Selene Overall Repo-Truth Activation Pack

## 0. Authority and Scope

AGENTS.md controls execution.

This is docs-only activation planning. It does not authorize runtime implementation, provider implementation, packet/schema implementation, Desktop edits, Adapter edits, PH1 runtime edits, protected execution changes, old-path deletion, or cleanup.

Future implementation requires explicit build instruction, approved file scope, clean-tree proof, existing-owner discovery, tests, backend evidence, provider-off proof, fake-provider proof, malformed-provider proof where relevant, and JD live proof where visible.

The governing lane for this activation pack is:

- current project phase: PROBABILISTIC_FOUNDATION_BUILD
- selected lane: PROBABILISTIC_PUBLIC_ANSWER
- simulation required: no for this docs task
- authority required: no for this docs task
- state mutation allowed: no runtime state mutation
- protected execution allowed: no
- provider degradation allowed: no
- normal answer allowed: yes
- fail-closed required: no, because this is read-only/docs-only activation planning

## 1. Architecture Set Verification

| # | Architecture document | Actual path | Found | Purpose | Read in this run |
|---|---|---|---|---|---|
| 1 | Selene Provider-First OpenAI Assisted Pivot Master Build Plan | `docs/SELENE_PROVIDER_FIRST_OPENAI_ASSISTED_PIVOT_MASTER_BUILD_PLAN.md` | yes | Strategic provider-first pivot plan | yes |
| 2 | Selene Provider-First Function Architecture Cards | `docs/SELENE_PROVIDER_FIRST_FUNCTION_ARCHITECTURE_CARDS.md` | yes | Canonical function and owner architecture cards | yes |
| 3 | Selene Provider-First Vertical Slice Build Pack | `docs/SELENE_PROVIDER_FIRST_VERTICAL_SLICE_BUILD_PACK.md` | yes | Executable vertical slice sequencing | yes |
| 4 | Selene Global Human Conversation Spine Master Architecture | `docs/SELENE_GLOBAL_HUMAN_CONVERSATION_SPINE_MASTER_ARCHITECTURE.md` | yes | Probabilistic semantic proposal plus deterministic Selene validation spine | yes |
| 5 | Selene Identity + Access + Authority Spine Master Architecture | `docs/SELENE_IDENTITY_ACCESS_AUTHORITY_SPINE_MASTER_ARCHITECTURE.md` | yes | Wake/session/identity/access/authority/simulation permission spine | yes |
| 6 | Selene Function Stack Architecture - Intent and Enterprise Stack Map | `docs/SELENE_FUNCTION_STACK_ARCHITECTURE_INTENT_AND_STACK_MAP.md` | yes | Enterprise function-stack map connecting OpenAI capabilities to Selene-owned stacks | yes |
| 7 | Selene Master Architecture Expansion Register | `docs/SELENE_MASTER_ARCHITECTURE_EXPANSION_REGISTER.md` | yes | Repo-truth gap register for missing or underdefined stacks | yes |
| 8 | Selene Final Overall Architecture Build Plan | `docs/SELENE_FINAL_OVERALL_ARCHITECTURE_BUILD_PLAN.md` | yes | Ordered implementation roadmap from the architecture set and repo-truth audit | yes |
| 9 | Selene PH1.M Human Memory Core Master Design | `docs/SELENE_PH1M_HUMAN_MEMORY_CORE_MASTER_DESIGN.md` | yes | Dedicated PH1.M governed human-like memory lifecycle master design | yes |

## 2. Repo-Truth Inspection Method

Inspection was shell-only. No Python was used.

Primary commands and search categories used:

- `cat AGENTS.md`
- `pwd`
- `git status --short`
- `git fetch origin`
- `git rev-parse HEAD`
- `git rev-parse origin/main`
- `wc -l` for required architecture documents
- `cat` for required architecture documents
- `find ... | rg` over `crates/selene_engines/src`, `crates/selene_os/src`, `crates/selene_kernel_contracts/src`, `crates/selene_adapter/src`, `crates/selene_storage/migrations`, `crates/selene_tools/src`, `apple/mac_desktop`, `apple/iphone`, `docs/reports`, `docs/web_search_plan`, and `docs/BLUEPRINTS`
- `rg` for owner families, packet names, provider surfaces, model policy, PH1.X directives, PH1.WRITE output, PH1.E search/source/image packets, PH1.M memory packets, visual/media contracts, access/governance/authority symbols, enterprise ops engines, Desktop/iPhone route parsing, Adapter compatibility paths, and old phrase/contains/fallback paths

Repo areas inspected:

- PH1.X, PH1.WRITE, PH1.M, PH1.E, PH1.D, PH1.W, PH1.C, PH1.L, PH1.TTS, STT, Voice ID
- Access, policy, governance, authority, runtime law, SimulationExecutor
- PH1.BCAST, PH1.DELIVERY, PH1.REM
- PH1.ONB, PH1.LINK, invite/enrollment surfaces
- PH1.TENANT, PH1.GOV, PH1.QUOTA, PH1.WORK, PH1.LEASE, PH1.SCHED, PH1.HEALTH, PH1.KMS, PH1.EXPORT
- PH1.VISION, OCR, media, visual rendering, PH1.ART, PH1.DOC, artifact/export/provenance
- PH1.PERSONA, PH1.EMO, PH1.FEEDBACK, PH1.LEARN
- PH1.COST, PH1.PREFETCH, PH1.CACHE, PH1.PAE, PH1.COMP
- Desktop, iPhone, Adapter, migrations, docs/reports, tests/checks/evals, old compatibility paths, phrase-patch paths, provider bypass paths, Desktop/Adapter authority risks

## 3. Master Owner Map

| Owner / Engine Family | Current Files / Paths | Current Symbols | Status | Architecture Doc Coverage | Notes |
|---|---|---|---|---|---|
| PH1.X | `crates/selene_kernel_contracts/src/ph1x.rs`; `crates/selene_engines/src/ph1x.rs`; `crates/selene_os/src/ph1x.rs` | `HumanConversationDirective`, `ActiveContextPacket`, `Ph1xDirective`, `Ph1xResponse`, `Slice3aOneLineProviderProposal`, `ph1x_universal_active_context_followup_query` | ACTIVE/PARTIAL | GHCS, Function Stack Map, Final Plan | Current deterministic active-context paths exist; semantic provider proposal layer is not yet canonical. |
| PH1.WRITE | `crates/selene_kernel_contracts/src/ph1write.rs`; `crates/selene_engines/src/ph1write.rs`; `crates/selene_os/src/ph1write.rs` | `Ph1WriteRequest`, `Ph1WriteOk`, `Ph1WriteResponse`, `formatted_text`, `run_one_line_current_equivalent` | ACTIVE/PARTIAL | GHCS, Function Stack Map, Final Plan | Owns output equivalents, but full display/tts/source/image/artifact presentation activation remains needed. |
| PH1.M | `crates/selene_kernel_contracts/src/ph1m.rs`; `crates/selene_engines/src/ph1m.rs`; `crates/selene_os/src/ph1m.rs`; migrations `0021_ph1m_vnext_memory_tables.sql` | `MemoryEvidencePacket`, `MemoryRecallRequest`, `FreshMemoryHandoff`, `MemoryContinuationDecision`, `MemoryTurnInput`, `MemoryTurnOutput` | ACTIVE/PARTIAL | GHCS, IAA, Function Stack Map | Exact access-scoped memory gateway needs activation against identity/access posture. |
| PH1.E / search / tools / files | `crates/selene_kernel_contracts/src/ph1e.rs`; `crates/selene_engines/src/ph1e.rs`; `crates/selene_os/src/ph1e.rs`; `crates/selene_tools/src/ph1e.rs`; `docs/web_search_plan/*` | `ToolRequest`, `ToolResult`, `SourceChipPacket`, `PresentationPacket`, `SearchImagePacket`, `ClaimVerificationPacket`, `WebAnswerVerificationPacket` | ACTIVE/PARTIAL | Function Stack Map, Final Plan | Source chips and image metadata exist; search/file/deep research activation still needs source acceptance and prompt-injection proof. |
| PH1.D / provider route | `crates/selene_kernel_contracts/src/ph1d.rs`; `crates/selene_engines/src/ph1d.rs`; `crates/selene_adapter/src/lib.rs` | `Ph1dProviderCallRequest`, `Ph1dProviderCallResponse`, `Ph1dProviderTransportEvidence`, `Ph1dChat`, `Ph1dFailureKind`, `run_ph1d_public_answer` | ACTIVE/PARTIAL | Provider-first docs, GHCS, Final Plan | Live public-answer route exists; follow-up routing must not bypass PH1.X. |
| Provider Governance / provider control | `crates/selene_engines/src/ph1providerctl.rs`; `docs/SELENE_OPENAI_MODEL_ROUTING_POLICY.md` | `ProviderNetworkPolicy`, `ProviderCallCounter`, `ProviderGovernanceEvidenceEnvelope`, `ProviderRegistryEntry`, `ProviderRouteDecision` | ACTIVE/PARTIAL | Provider-first docs, Function Stack Map, Final Plan | Provider-off/fake-provider coverage exists; no kernel contract file named `ph1providerctl.rs` was found. |
| PH1.W / wake | `crates/selene_kernel_contracts/src/ph1w.rs`; `crates/selene_engines/src/ph1w.rs`; `crates/selene_os/src/ph1w.rs`; migration `0011_ph1w_wake_tables.sql` | `WakeDecision`, `WakePolicyContext`, `WakeGateResults`, `WakeEnrollmentSessionId` | ACTIVE/PARTIAL | IAA, Expansion Register, Final Plan | Wake is evidence only; conversational acknowledgements need Quick Assist activation. |
| PH1.C / canonical ingress | `crates/selene_kernel_contracts/src/ph1c.rs`; `crates/selene_engines/src/ph1c.rs`; `crates/selene_os/src/ph1c.rs` | PH1.C contracts/runtime modules | ACTIVE/PARTIAL | CORE, GHCS, IAA | Transcript/admission mapping to semantic proposal needs activation. |
| PH1.L / session | `crates/selene_kernel_contracts/src/ph1l.rs`; `crates/selene_engines/src/ph1l.rs`; `crates/selene_os/src/ph1l.rs`; `crates/selene_os/src/runtime_session_foundation.rs` | `SessionAttachOutcome`, `SessionAccessSnapshot`, `Stage5TurnAuthorityPacket`, `Stage6AccessContextPacket` | ACTIVE/PARTIAL | IAA, Final Plan | Session has authority posture carriers; exact `SessionIdentityBindingPacket` name not found. |
| PH1.TTS / STT / realtime | `crates/selene_kernel_contracts/src/ph1tts.rs`; `crates/selene_engines/src/ph1tts.rs`; `crates/selene_os/src/ph1tts.rs`; `crates/selene_adapter/src/bin/http_adapter.rs`; Desktop/iPhone voice surfaces | `gpt-4o-transcribe`, `gpt-4o-mini-tts`, voice adapter payloads | ACTIVE/PARTIAL | Function Stack Map, IAA, Final Plan | OpenAI TTS/STT model references exist; realtime activation remains partial/unclear. |
| Voice ID | `crates/selene_kernel_contracts/src/ph1_voice_id.rs`; `crates/selene_engines/src/ph1_voice_id.rs`; `crates/selene_os/src/ph1_voice_id.rs`; migration `0008_ph1vid_voice_enrollment_tables.sql` | `Ph1VoiceIdRequest`, `Ph1VoiceIdResponse`, voice embedding/liveness equivalents | ACTIVE/PARTIAL | IAA, Final Plan | Evidence-only contract needs early activation. |
| Access / policy / governance / authority | `crates/selene_kernel_contracts/src/ph1access.rs`; `ph1policy.rs`; `ph1gov.rs`; `runtime_execution.rs`; `runtime_law.rs` | `AccessProfileSchemaRecord`, `AccessOverlayOpSpec`, `PolicyPromptDecision`, `GovDecisionStatus`, `AuthorityExecutionState`, `RuntimeProtectedActionClass` | ACTIVE/PARTIAL | IAA, Expansion Register, Final Plan | Exact `AccessScopePacket`/`AuthorityDecisionPacket` names not found; equivalents exist. |
| SimulationExecutor | `crates/selene_os/src/simulation_executor.rs`; `crates/selene_kernel_contracts/src/runtime_law.rs`; `runtime_execution.rs` | Simulation dispatch/execution and runtime law state equivalents | ACTIVE/PARTIAL | IAA, Function Stack Map, Final Plan | Must remain protected execution owner. |
| Broadcast / delivery / reminders | `crates/selene_kernel_contracts/src/ph1bcast.rs`; `ph1delivery.rs`; `ph1rem.rs`; matching engines/OS; `docs/DB_WIRING/PH1_REM.md` | `BcastDraftCreateRequest`, `DeliverySendRequest`, `ReminderScheduleCommitRequest` | ACTIVE/PARTIAL | Expansion Register, Final Plan | Dedicated stack needed before side-effecting messaging builds. |
| Onboarding / invite / link / enrollment | `crates/selene_os/src/ph1onb.rs`; `crates/selene_os/src/ph1link.rs`; `crates/selene_kernel_contracts/src/ph1link.rs`; migrations `0012`, `0025`; Desktop/iPhone invite surfaces | `OnboardingSessionId`, `LinkRecord`, `LinkActivationResult`, onboarding invite route surfaces | ACTIVE/PARTIAL | Expansion Register, Final Plan | Client route parsing exists and must remain render-only. |
| Tenant / governance / quota | `crates/selene_kernel_contracts/src/ph1tenant.rs`; `ph1gov.rs`; `ph1quota.rs`; matching engines/OS | `TenantBinding`, `GovPolicyEvaluateRequest`, `QuotaPolicyEvaluateRequest` | ACTIVE/PARTIAL | Expansion Register, Final Plan | Needed as early access/provider scope dependency. |
| Work / lease / scheduling / health / KMS / export | `ph1work.rs`; `ph1lease.rs`; `ph1sched.rs`; `ph1health.rs`; `ph1kms.rs`; `ph1export.rs` across contracts/engines/OS | Work, lease, schedule, health, KMS, export request/response families | ACTIVE/PARTIAL | Expansion Register, Final Plan | Platform ops stack requires own activation before mutation or export builds. |
| Visual / OCR / media | `crates/selene_kernel_contracts/src/ph1vision.rs`; `crates/selene_engines/src/ph1vision.rs`; `crates/selene_engines/src/ph1vision_media.rs`; `crates/selene_os/src/ph1os.rs` | `VisionAnalyzeMediaRequest`, `VisionOcrResult`, `VisionDetectedObject`, `VisionMediaOutput`, `PH1D_OPENAI_MODEL` OCR config | ACTIVE/PARTIAL | Function Stack Map, Expansion Register, Final Plan | Exact multimodal evidence packets need mapping. |
| Artifact / document / export / provenance | `crates/selene_kernel_contracts/src/ph1art.rs`; `ph1doc.rs`; `ph1export.rs`; matching engines/OS | `ArtifactTrustDecisionProvenance`, `DocumentSourceRef`, `ExportRequestEnvelope` | ACTIVE/PARTIAL | Function Stack Map, Expansion Register, Final Plan | Trust-root and provenance stack needs activation. |
| Persona / emotion / feedback / learning | `ph1persona.rs`; `ph1emocore.rs`; `ph1emoguide.rs`; `ph1feedback.rs`; `ph1learn.rs`; `ph1know.rs`; `ph1kg.rs`; `ph1pattern.rs` | Persona, emotion, feedback, learning, knowledge graph/pattern engine families | ACTIVE/PARTIAL | Expansion Register, Final Plan | Needs memory/access-bounded personalization activation. |
| Cost / prefetch / cache / PAE | `ph1cost.rs`; `ph1prefetch.rs`; `ph1cache.rs`; `ph1pae.rs`; `ph1providerctl.rs` | `CostRouteBudget`, `Prefetch`, `Cache`, `PaePolicyScoreBuildRequest`, provider routing equivalents | ACTIVE/PARTIAL | Function Stack Map, Expansion Register, Final Plan | Must be tied to provider governance and privacy gates. |
| Deterministic compute | `crates/selene_kernel_contracts/src/ph1comp.rs`; `crates/selene_engines/src/ph1comp.rs`; `crates/selene_os/src/ph1comp.rs` | `ComputationPacket`, `ComputationConsensusResult`, `ComputationExecutionState` | ACTIVE/PARTIAL | Expansion Register, Final Plan | Official calculation authority must be separated from provider advice. |
| Desktop | `apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`; `SeleneMacDesktopRuntimeBridge.swift`; Desktop proof views | `VoiceTurnAdapterResponsePayload`, `responseText`, `desktopPersistAuthoritativeReplyIfNeeded`, invite/open route parsing | ACTIVE/PARTIAL | IAA, Expansion Register, Final Plan | Render-only boundary needs proof; route parsing is old-risk ledger item. |
| iPhone | `apple/iphone/SeleneIPhone/SessionShellView.swift` | `inviteLike`, `appOpenLike`, `openLike`, explicit voice/session/onboarding surfaces | ACTIVE/PARTIAL | IAA, Expansion Register, Final Plan | Render-only boundary needs proof; route parsing is old-risk ledger item. |
| Adapter | `crates/selene_adapter/src/lib.rs`; `crates/selene_adapter/src/bin/http_adapter.rs`; `grpc_adapter.rs`; `desktop_voice_e2e.rs` | `run_ph1d_public_answer`, `maybe_run_ph1d_public_answer`, `fallback_runtime_execution_envelope_for_voice_turn_request`, `deterministic_active_context_followup_query`, `deterministic_weather_context_followup_query` | ACTIVE/PARTIAL/RISK | Provider-first docs, Final Plan | Large compatibility surface exists; transport-only target requires staged retirement ledger. |
| Migrations / storage / audit | `crates/selene_storage/migrations/*.sql`; `crates/selene_storage/src/*`; docs storage reports | stage migrations through `0026_stage7_internal_history_evidence.sql` | ACTIVE/PARTIAL | CORE, Function Stack Map, Final Plan | Audit packet names vary by owner and need stack-specific activation. |

## 4. Architecture-to-Repo Symbol Mapping

| Architecture Concept | Exact Repo Symbol If Found | File Path | Status | Notes |
|---|---|---|---|---|
| SemanticMeaningProposalPacket | none found | n/a | NOT_FOUND | Future schema-bound semantic proposal packet is required. Current closest surfaces are PH1.D provider call/result packets and `Slice3aOneLineProviderProposal`. |
| HumanConversationDirective | `HumanConversationDirective` | `crates/selene_kernel_contracts/src/ph1x.rs` | FOUND | PH1.X owns current directive equivalent. |
| ProviderPreflightPacket | `ProviderGateDecision`, `ProviderNetworkPolicy`, `Ph1dProviderTransportEvidence` | `crates/selene_engines/src/ph1providerctl.rs`; `crates/selene_kernel_contracts/src/ph1d.rs` | EQUIVALENT_FOUND | Exact name absent; provider gate/evidence equivalents exist. |
| ProviderDecisionTracePacket | `ProviderGovernanceEvidenceEnvelope`, `ProviderRouteDecision` | `crates/selene_engines/src/ph1providerctl.rs` | EQUIVALENT_FOUND | Needs canonical packet naming alignment. |
| ProviderCallRequestPacket | `Ph1dProviderCallRequest` | `crates/selene_kernel_contracts/src/ph1d.rs` | EQUIVALENT_FOUND | PH1.D request equivalent. |
| ProviderCallResultPacket | `Ph1dProviderCallResponse` | `crates/selene_kernel_contracts/src/ph1d.rs` | EQUIVALENT_FOUND | PH1.D response equivalent. |
| AccessScopePacket | `SessionAccessSnapshot`, `Stage6AccessContextPacket`, access profile contracts | `crates/selene_os/src/runtime_session_foundation.rs`; `crates/selene_kernel_contracts/src/ph1access.rs` | PARTIAL | Exact name not found; access posture equivalents exist. |
| WakeDecisionPacket | `WakeDecision` | `crates/selene_kernel_contracts/src/ph1w.rs` | EQUIVALENT_FOUND | Packet suffix differs. |
| SpeakerIdentityEvidencePacket | `Ph1VoiceIdResponse`, `VoiceEmbeddingCaptureRef` equivalents | `crates/selene_kernel_contracts/src/ph1_voice_id.rs` | EQUIVALENT_FOUND | Exact name not found; evidence-only posture needs activation. |
| AuthorityDecisionPacket | `AuthorityExecutionState`, `AuthorityPolicyDecision` | `crates/selene_kernel_contracts/src/runtime_execution.rs` | EQUIVALENT_FOUND | Exact name not found. |
| SimulationGateDecisionPacket | simulation dispatch/outcome equivalents | `crates/selene_os/src/simulation_executor.rs`; `crates/selene_kernel_contracts/src/runtime_law.rs` | PARTIAL | Exact name not found in scan; SimulationExecutor owner exists. |
| WriteOutputPacket | `Ph1WriteOk`, `PresentationPacket` | `crates/selene_kernel_contracts/src/ph1write.rs`; `crates/selene_kernel_contracts/src/ph1e.rs` | EQUIVALENT_FOUND | `formatted_text`, `response_text`, and `tts_text` equivalents exist. |
| SourceChipPacket | `SourceChipPacket` | `crates/selene_kernel_contracts/src/ph1e.rs` | FOUND | PH1.E source presentation packet exists. |
| MemoryEvidencePacket | `MemoryEvidencePacket` | `crates/selene_kernel_contracts/src/ph1m.rs` | FOUND | Current repo has exact packet. |
| ToolProposalPacket | none found | n/a | NOT_FOUND | Closest are `ToolRequest`, `ToolResponse`, and provider proposal concepts. |
| ToolExecutionDecisionPacket | no exact symbol found | n/a | PARTIAL | Tool request/result equivalents exist in PH1.E/tool router. |
| VisualEvidencePacket | `VisionEvidenceItem`, `VisionEvidenceExtractOk`, `VisionAnalyzeMediaOk` | `crates/selene_kernel_contracts/src/ph1vision.rs` | EQUIVALENT_FOUND | Exact name absent; vision evidence equivalents exist. |
| ArtifactProvenancePacket | `ArtifactTrustDecisionProvenance`, `ArtifactTrustExecutionState` | `crates/selene_kernel_contracts/src/ph1art.rs` | EQUIVALENT_FOUND | Exact name absent; artifact trust equivalents exist. |
| SearchEvidencePacket | `SourceEvaluationPacket`, `WebAnswerVerificationPacket` | `crates/selene_kernel_contracts/src/ph1e.rs` | EQUIVALENT_FOUND | Search/source evidence split needs activation. |
| PromptInjectionDefensePacket | no exact symbol found | n/a | NOT_FOUND | Required future PH1.E/file/source defense packet or equivalent. |
| VoiceOutputPacket | TTS and Desktop voice response equivalents | `crates/selene_kernel_contracts/src/ph1tts.rs`; Desktop bridge files | PARTIAL | Exact architecture packet name not found. |
| ProviderModelVersionPacket | model evidence fields | `crates/selene_kernel_contracts/src/ph1d.rs`; `docs/SELENE_OPENAI_MODEL_ROUTING_POLICY.md` | PARTIAL | Model governance doc exists; packet naming needs activation. |

## 5. Complete Stack Activation Matrix

| Stack | Phase | Current Owner Files | Current Readiness | Activation Pack Needed | Estimated Slice Count | First Slice | Major Risks |
|---|---:|---|---|---|---:|---|---|
| A. Global Human Interface / Semantic Intent Stack | 3-4 | `ph1x.rs`, `ph1d.rs`, `ph1providerctl.rs`, Adapter | PARTIAL | GHCS activation | 4 | SemanticInterpreterProvider fake proposal path | Phrase-patch drift; missing `SemanticMeaningProposalPacket`; PH1.D public-answer bypass. |
| B. Web Search + Source Evidence Stack | 6 | `ph1e.rs`, `ph1search.rs`, web_search_plan | PARTIAL | PH1.E search activation | 5 | Source acceptance baseline | Search/provider bypass; source dump; missing prompt-injection packet. |
| C. Image-Backed Search + Visual Presentation Stack | 12 | PH1.E, PH1.VISION, Desktop/iPhone | PARTIAL | Visual/media activation | 3 | Approved image-card packet proof | Raw image URL display; fabricated image risk. |
| D. Deep Research Stack | 6/23 | PH1.E, providerctl, model policy | ARCHITECTURE_ONLY/PARTIAL | Deep research activation | 3 | Deep research disabled-by-default proof | Cost/budget/live provider drift. |
| E. Writing + Transformation Stack | 5 | PH1.WRITE, PH1.X | PARTIAL | PH1.WRITE activation | 4 | One-line target handoff canonical path | Deterministic wording patches; unsupported claim insertion. |
| F. Presentation + TTS-Safe Output Stack | 5/10 | PH1.WRITE, PH1.TTS, PH1.E, clients | PARTIAL | Presentation activation | 5 | Display/tts/source split baseline | Client wording/fallback leakage; TTS metadata. |
| G. Memory + Recall + Preference Stack | 8 | PH1.M, PH1.X, PH1.L, PH1.E, PH1.WRITE, access/session, Adapter/Desktop compatibility surfaces | PARTIAL / MASTER_DESIGN_ADDED | PH1.M Human Memory Core activation | 11 | Build 0 - PH1.M Repo Truth And Gap Audit | Session-search drift; duplicate memory owner; Adapter/Desktop/PH1.X/PH1.E/PH1.WRITE memory shortcuts; current-vs-recent confusion. |
| H. File QA + Document Stack | 9/17 | PH1.E, PH1.DOC, PH1.EXPORT | PARTIAL | File/document activation | 4 | File evidence admission and injection defense | File text as instruction; private file scope gaps. |
| I. Tool / Connector / MCP Stack | 9 | PH1.E, `crates/selene_tools`, providerctl | PARTIAL | Tool/connector activation | 5 | Read-only tool proposal validation | Provider tool execution authority drift. |
| J. Protected Action + Simulation Stack | 7/17/20 | PH1.X, runtime_law, SimulationExecutor, access/gov | PARTIAL | Protected execution activation | 5 | Protected risk fail-closed evidence | Provider/voice/client authority drift. |
| K. Identity + Access + Authority Stack | 7/8 | PH1.W, PH1.L, Voice ID, access/gov/runtime_execution | PARTIAL | VIA activation | 7 | Session identity/access posture baseline | Voice ID/access equivalence confusion. |
| L. Voice / Wake / Session / Realtime Stack | 10 | PH1.W, PH1.C, PH1.L, PH1.TTS, Voice ID, Adapter, clients | PARTIAL | Voice/realtime activation | 6 | Wake/session/voice evidence-only route | Local STT/TTS fallback claims; session boundary ambiguity. |
| M. Translation + Language Adaptation Stack | 5/10 | PH1.WRITE, PH1.LANG, model policy | PARTIAL | Language activation | 2 | Translation request/output validation | Language drift and unsupported claims. |
| N. Summarization + Compression Stack | 5 | PH1.WRITE, PH1.SUMMARY, PH1.X | PARTIAL | Summary activation | 2 | Target-scoped summary baseline | Phrase-triggered target hijack. |
| O. Artifact / Document / Slide / Spreadsheet Stack | 17 | PH1.ART, PH1.DOC, PH1.EXPORT, PH1.WRITE | PARTIAL | Artifact trust activation | 5 | Artifact provenance baseline | Official/draft confusion; provenance gaps. |
| P. Image Generation / Editing Stack | 15/21 | Model policy, PH1.VISION/media/artifact owners | ARCHITECTURE_ONLY/PARTIAL | Image generation activation | 3 | Provider governance and provenance shell | Model approval unclear; generated-vs-real risk. |
| Q. Video Generation Stack | 21 | PH1.VISION media, model policy | PARTIAL/REPO_TRUTH_NEEDED | Video activation | 4 | Video provider-off/fake-provider baseline | Cost/safety/provenance/model approval. |
| R. Data Analysis + Report Drafting Stack | 20 | PH1.E, PH1.COMP, PH1.WRITE, SimulationExecutor | PARTIAL | Data/compute activation | 4 | Advisory vs official report boundary | Provider numeric authority drift. |
| S. Code / Developer Assistance Stack | 23 | PH1.WRITE, Codex/AGENTS docs, model policy | PARTIAL | Code-assist activation | 2 | Codex instruction packet/eval baseline | AGENTS bypass; unapproved repo edits. |
| T. Cost / Provider Governance / Observability Stack | 2/19 | `ph1providerctl.rs`, PH1.COST, PH1.PREFETCH, PH1.PAE, PH1.CACHE | PARTIAL | Provider governance activation | 5 | Provider-off/fake-provider counters baseline | Fallback/cost optimization drift. |
| U. Evaluation / Regression / JD Live Acceptance Stack | 23 | tests, reports, runtime evidence paths | PARTIAL | Eval/JD activation | 4 | Backend evidence matrix baseline | Passing tests without real-route proof. |
| Broadcast / Delivery / Reminder / Messaging Stack | 13 | PH1.BCAST, PH1.DELIVERY, PH1.REM | PARTIAL | Broadcast activation | 4 | Draft-vs-send boundary | Side effects without authority/simulation. |
| Onboarding / Invite / Link / Enrollment Stack | 14 | PH1.ONB, PH1.LINK, PH1.W, Voice ID, clients | PARTIAL | Onboarding activation | 5 | Link/onboarding state proof | Client route authority drift. |
| Master Access Template / Role / Permission / Admin Controls Stack | 15 | PH1.ACCESS, PH1.POLICY, PH1.GOV | PARTIAL | Access template activation | 5 | Access schema/overlay read-only proof | Admin authority shortcuts. |
| Tenant / Workspace / Governance / Quota Stack | 15 | PH1.TENANT, PH1.GOV, PH1.QUOTA | PARTIAL | Tenant/governance activation | 4 | Tenant scope baseline | Wrong-tenant data egress. |
| Work / Lease / Scheduling / Health / KMS / Export Platform Ops Stack | 16 | PH1.WORK, PH1.LEASE, PH1.SCHED, PH1.HEALTH, PH1.KMS, PH1.EXPORT | PARTIAL | Platform ops activation | 6 | Health/KMS/export no-secret evidence | Secret leakage; protected export. |
| Visual Recognition / OCR / Media Ingestion / Multimodal Evidence Stack | 11 | PH1.VISION, PH1.DOC, PH1.MULTI, PH1.OS OCR | PARTIAL | Visual/media activation | 5 | Media admission and OCR evidence | OCR/source text as instruction. |
| Visual Rendering / Image Cards / Media Presentation Stack | 12 | PH1.E, PH1.WRITE, clients | PARTIAL | Media presentation activation | 4 | Image-card display eligibility | Client chooses visuals. |
| Video Recognition / Video Rendering / Video Generation Stack | 21 | PH1.VISION media, model policy, clients | PARTIAL/REPO_TRUTH_NEEDED | Video activation | 5 | Video evidence/provenance map | Model/safety/cost gaps. |
| Artifact Trust / Document / Export / Provenance Stack | 17 | PH1.ART, PH1.DOC, PH1.EXPORT | PARTIAL | Artifact trust activation | 5 | Trust/provenance evidence packet map | Official record overclaim. |
| Persona / Preference / Emotion / Feedback / Learning Stack | 18 | PH1.PERSONA, PH1.EMO, PH1.FEEDBACK, PH1.LEARN, PH1.M | PARTIAL | Persona/learning activation | 5 | Preference scope baseline | Private personalization without identity. |
| Provider Assist / Cost / Prefetch / Arbitration Stack | 19 | PH1.COST, PH1.PREFETCH, PH1.CACHE, PH1.PAE, providerctl | PARTIAL | Provider assist activation | 5 | Provider arbitration evidence | Hidden provider attempts; fallback drift. |
| Deterministic Compute / Consensus / Calculation Authority Stack | 20 | PH1.COMP, PH1.E, SimulationExecutor | PARTIAL | Compute activation | 4 | Computation packet authority boundary | Provider numeric authority drift. |
| Client Route Presentation / App Open / Invite Rendering Stack | 22 | Desktop/iPhone session shells and runtime bridge | PARTIAL | Client route activation | 4 | Client render-only route proof | Client route parsing authority drift. |
| Old Compatibility Path Retirement Register | 24 | Adapter, PH1.X, clients, PH1.D public answer route | PARTIAL | Old path retirement activation | 6 | Active-caller ledger | Destructive cleanup before proof. |
| Conversational Experience + Quick Assist Stack | 3/5/10 | PH1.WRITE, PH1.X, PH1.W/C/L/TTS, PH1.E, PH1.M, clients, Adapter | OFFICIAL_ARCHITECTURE / ACTIVATION_PACK_REQUIRED | Quick Assist activation | 5 | Provider-assisted wording under PH1.WRITE approval | Official Expansion Register stack #15; Quick Assist Activation Pack required; deterministic user-help wording drift. |
| Selene Emotional Intelligence + Relationship Presence Stack | 3/5/10/18 | PH1.WRITE, PH1.PERSONA, PH1.EMO, PH1.EMO.CORE, PH1.EMO.GUIDE, PH1.FEEDBACK, PH1.LEARN, PH1.M, PH1.TTS, Provider Governance, clients, Adapter | OFFICIAL_ARCHITECTURE_STACK / ACTIVATION_PACK_REQUIRED / NO_RUNTIME_IMPLEMENTATION_YET | Selene Emotional Intelligence + Relationship Presence activation | 5 | Persona wording under PH1.WRITE and PH1.TTS approval | Persona safety bypass; private personalization without memory/access scope; Desktop/Adapter persona brain. |

### Global Semantic Proposal Wiring Gate

Default wiring for user-facing stacks:

`User input -> SemanticInterpreterProvider / GPT-5.5 proposal where applicable -> PH1.X deterministic validation -> HumanConversationDirective -> canonical stack owner -> PH1.WRITE final output -> Adapter transport -> Desktop/iPhone render/speak only`

| Stack | Semantic proposal required | Intent categories / route hints | PH1.X directive fields required | Canonical owner receiving directive | Old deterministic paths blocked or compatibility-only |
|---|---|---|---|---|---|
| Global Human Interface / Semantic Intent | yes | intent classification, target resolution, risk flags | validated intent/operation/target/owner/risk | PH1.X then selected owner | `ph1x_universal_active_context_followup_query` retained until semantic proposal replacement. |
| Web Search + Source Evidence | yes for user language | public_search_required, source_backed_answer_required, freshness_required | search gateway, evidence requirements, public/private/protected split | PH1.E | Search phrase shortcuts must not expand. |
| Image-Backed Search + Visual Presentation | yes | image_needed, visual_source_result | PH1.E/PH1.WRITE visual presentation policy | PH1.E, PH1.WRITE | Client image choice forbidden. |
| Deep Research | yes | deep_research_request | explicit research eligibility, budget, evidence requirements | PH1.E | No implicit live deep research. |
| Writing + Transformation | yes | rewrite_previous_answer, change_tone, change_format | target refs, write policy, output mode | PH1.WRITE | One-line phrase helpers compatibility-only. |
| Presentation + TTS-Safe Output | yes for user-requested style; no for owner state packets | format_for_display, format_for_tts, source presentation | write policy, tts policy, presentation directive | PH1.WRITE/PH1.TTS | Hardcoded presentation wording requires approval. |
| Memory + Recall + Preference | yes | memory_recall, preference_recall, update/forget | memory gateway, access scope, target refs | PH1.M | Adapter memory assertions compatibility-only. |
| File QA + Document | yes | file_question_answering, file_transformation | file gateway, evidence requirements | PH1.E/PH1.DOC/PH1.WRITE | File text must not route itself. |
| Tool / Connector / MCP | yes | tool_read_only_lookup, tool_write_request | tool gateway, protected gate, owner | PH1.E/Authority/SimulationExecutor | Provider tool proposal cannot execute. |
| Protected Action + Simulation | yes for language; deterministic for execution | protected_action_requested, approval_requested | protected gate, authority gate, simulation gate | Authority/SimulationExecutor | Payroll/protected helpers compatibility-only and fail-closed. |
| Identity + Access + Authority | yes only for meaning/risk; deterministic for permission | identity_uncertain, authority_uncertain | identity/access refs, protected posture | Access/Governance/Authority | Voice ID/access cannot be inferred from wording. |
| Voice / Wake / Session / Realtime | mixed | committed_voice_turn, wake/session state help | session refs, transcript refs, tts policy | PH1.C/L/W/TTS/PH1.X | Wake greetings must move to Quick Assist under validation. |
| Translation + Language | yes | translate_content, answer_in_language | target, language policy | PH1.WRITE | Language detection cannot be client-owned. |
| Summarization + Compression | yes | summarize_previous_answer, compress | target refs, output mode | PH1.WRITE/PH1.SUMMARY | Summary phrase patches compatibility-only. |
| Artifact / Document / Slides / Sheets | yes | artifact_generation, artifact_edit | artifact gateway, evidence refs | Artifact owner/PH1.WRITE | Provider draft not official artifact state. |
| Image Generation / Editing | yes | image_generation_request | media safety, provider policy, provenance | Media/artifact owner | No model call without governance. |
| Video Generation | yes | video_generation_request | media safety, cost cap, provenance | Media/video owner | MODEL_POLICY_MISSING if no approved exact model. |
| Data Analysis + Report Drafting | yes | data_analysis, report_draft | evidence refs, official/advisory status | PH1.E/PH1.COMP/PH1.WRITE | Provider math not deterministic authority. |
| Code / Developer Assistance | yes | code_help, codex_instruction | AGENTS law, scope, test plan | PH1.WRITE/Codex workflow | No repo edit from advice alone. |
| Cost / Provider Governance | no for gates; yes for user explanations | provider_status_question | provider evidence refs | Provider Governance/PH1.WRITE | Provider status wording must not expose secrets. |
| Eval / JD Live Acceptance | mixed | eval_requested, acceptance scenario | evidence requirements | Eval/audit owners | Unit-test-only acceptance forbidden. |
| Broadcast / Delivery / Reminder | yes | message_draft, send_request, reminder_request | side-effect/protected gates | PH1.BCAST/DELIVERY/REM | No message sending from provider text. |
| Onboarding / Invite / Link | yes | onboarding_help, invite_open, enrollment | session/access/enrollment refs | PH1.ONB/PH1.LINK/W/Voice ID | Client `inviteLike`/`openLike` is route rendering only. |
| Access Template / Admin | yes for admin wording; deterministic for permissions | access_admin_request | authority gate, access schema refs | Access/Gov/Policy | GPT cannot grant roles. |
| Tenant / Workspace / Quota | yes for explanation; deterministic for scope | tenant_context_question | tenant/workspace/quota refs | PH1.TENANT/GOV/QUOTA | No inferred tenant from phrase similarity. |
| Platform Ops | yes for explanation; deterministic for operations | health/status/export/work request | platform op refs, protected gate | PH1.WORK/LEASE/SCHED/HEALTH/KMS/EXPORT | No KMS/export from provider. |
| Visual Recognition / OCR | yes where media asks meaning | image_question, ocr_request | media/file/evidence refs | PH1.VISION/PH1.E | OCR text is evidence, not instruction. |
| Visual Rendering | no for packet rendering; yes for caption wording | image_card_presentation | presentation policy | PH1.WRITE/clients render | Clients cannot select visuals. |
| Video Recognition/Rendering/Generation | yes | video_understanding, video_generation | media refs, provenance, provider caps | PH1.VISION/media owner | Video safety/model gaps. |
| Artifact Trust / Export | yes for request meaning; deterministic for trust/export | artifact_export, provenance_check | artifact/export refs | PH1.ART/DOC/EXPORT | No official/export claim without proof. |
| Persona / Preference / Learning | yes | preference_recall/update, feedback | memory/access refs | PH1.M/PERSONA/LEARN | Private personalization without identity forbidden. |
| Provider Assist / Cost / Arbitration | no for gates; yes for status wording | provider_status/capability question | provider evidence refs | Provider Governance/PH1.WRITE | Hidden prefetch/provider attempts forbidden. |
| Deterministic Compute | yes for task meaning; deterministic for calculation | calculate, compare, consensus | compute/evidence refs | PH1.COMP | Provider calculation cannot be official. |
| Client Route Presentation | no for rendering; yes for help wording | app_open_help, invite_help | client provenance, route refs | Client render-only + PH1.WRITE | Client route parsing cannot grant authority. |
| Conversational Experience + Quick Assist | yes | quick_clarify, reassure, next_step_help, failed_step_recovery | state refs, owner facts, write/tts policy | PH1.WRITE after relevant owner facts | Hardcoded wake/help/weather wording is drift risk. |
| Selene Emotional Intelligence + Relationship Presence | yes | persona_tone, emotional_presentation, wake_greeting_personality, serious_mode | state/risk refs, persona policy, memory/access refs where personalized, write/tts policy | PH1.WRITE with PH1.EMO/PH1.PERSONA/PH1.M assist surfaces | Persona wording cannot grant authority, approve access, mutate state, or bypass PH1.WRITE/PH1.X/memory law. |
| Old Compatibility Retirement | no | cleanup planning only | active caller/proof refs | Correct canonical owners | Deletion only after proof. |

Any stack receiving raw user language directly without PH1.X validation must be marked `GLOBAL_SEMANTIC_SPINE_BYPASS_RISK`.

### Probabilistic Human Interaction Doctrine Matrix

| Stack | User Interaction Mode | GPT-5.5 Role | Deterministic Gate | Forbidden Deterministic Rubbish | Notes |
|---|---|---|---|---|---|
| Global Human Interface / Semantic Intent | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Meaning proposals | PH1.X schema/owner/target/risk validation | Keyword routing as architecture | Current repo has deterministic compatibility helpers. |
| Web Search + Source Evidence | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Query/summary/draft assistance | PH1.E source acceptance, budget, prompt-injection defense | Search phrase patches | Provider-off proof required. |
| Image-Backed Search + Visual Presentation | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Relevance/caption assistance | Image safety/source page/display eligibility | Client-picked images | PH1.E/PH1.WRITE own. |
| Deep Research | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Research planning/synthesis | Explicit request, budget, source ledger | Implicit deep research trigger | Live opt-in only. |
| Writing + Transformation | PROBABILISTIC_FIRST | Draft/rewrite/format | PH1.X target and PH1.WRITE validation | Phrase-only one-line routing | Current one-line path is compatibility. |
| Presentation + TTS-Safe Output | PROBABILISTIC_FIRST | Natural wording and style | PH1.WRITE output validation, TTS policy | Hardcoded presentation templates as primary UX | Quick Assist should cover normal guidance. |
| Memory + Recall + Preference | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Salience/summary proposal | PH1.M memory permission/access scope | Adapter memory shortcuts | Private memory fails closed. |
| File QA + Document | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | File understanding/draft | File scope, evidence, injection defense | File text instructing runtime | PH1.E owns file evidence. |
| Tool / Connector / MCP | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Tool proposal only | Tool scope, parameter validation, authority | Provider-executed tool action | Writes route to authority/simulation. |
| Protected Action + Simulation | DETERMINISTIC_PROTECTED_EXECUTION | Explain/extract proposed action | Authority and SimulationExecutor | Provider/voice/client approval | Fail closed. |
| Identity + Access + Authority | DETERMINISTIC_PROTECTED_EXECUTION | None for permission | Wake/session/Voice ID/access/authority | Voice ID equals authority | GPT may only see sanitized posture. |
| Voice / Wake / Session / Realtime | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Semantic/writing after admission | Wake/session/identity gates | Hardcoded wake greetings as main UX | Quick Assist activation needed. |
| Translation + Language | PROBABILISTIC_FIRST | Translation/adaptation | PH1.WRITE factual/evidence validation | Language keyword routing | Preserve evidence. |
| Summarization + Compression | PROBABILISTIC_FIRST | Summarize/compress | Target/evidence validation | Phrase-target hijack | PH1.X target owner. |
| Artifact / Document / Slide / Spreadsheet | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Draft/transform content | Artifact provenance/export/access | Provider official document claim | Draft vs official required. |
| Image Generation / Editing | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Generate/edit | Safety/provenance/model governance | Ungoverned media call | Model details need activation. |
| Video Generation | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Generate/understand | Safety/provenance/cost/model governance | Ungoverned video call | REPO_TRUTH_NEEDED. |
| Data Analysis + Report Drafting | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Explain/draft | PH1.COMP deterministic calculation/evidence | Provider numeric authority | Advisory/official split. |
| Code / Developer Assistance | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Code reasoning/instruction draft | AGENTS and scope/test gates | Auto-edit from chat advice | Codex execution remains separate. |
| Cost / Provider Governance | REQUIRED_DETERMINISTIC_GATE | None for routing; optional explanation | Provider-off/fake/budget/model policy | Cost-driven silent fallback | Deterministic gate only. |
| Eval / JD Live Acceptance | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Eval generation/help | Evidence/JD acceptance rules | Test-only product pass | Backend evidence required. |
| Broadcast / Delivery / Reminder | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Draft/clarify message | Recipient/channel/side-effect gates | Phrase sends message | Protected side effects fail closed. |
| Onboarding / Invite / Link | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Explain step/help | Link/session/access/enrollment state | Client completes onboarding | Client render-only. |
| Access Template / Admin | DETERMINISTIC_PROTECTED_EXECUTION | Explain/admin draft | Role/template authority/simulation | GPT grants role | Protected. |
| Tenant / Workspace / Quota | DETERMINISTIC_PROTECTED_EXECUTION | Explain status | Tenant/quota/gov policy | Tenant inferred by language | Scope-sensitive. |
| Platform Ops | DETERMINISTIC_PROTECTED_EXECUTION | Explain status | Work/lease/KMS/export authority | Hardcoded ops actions | Secrets never exposed. |
| Visual Recognition / OCR | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Extract/describe | Media scope/evidence/injection defense | OCR instruction execution | Visual evidence only. |
| Visual Rendering | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Caption wording | Display eligibility | Client choosing evidence | Render-only clients. |
| Video Recognition/Rendering/Generation | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Understand/generate | Safety/model/provenance | Video call by phrase | REPO_TRUTH_NEEDED. |
| Artifact Trust / Export | DETERMINISTIC_PROTECTED_EXECUTION | Draft/explain | Provenance/export/access | Provider/export official claim | Audit-heavy. |
| Persona / Preference / Learning | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Tone/preference proposal | PH1.M identity/access/memory law | Persistent preference by vibe | Private scope. |
| Provider Assist / Cost / Arbitration | REQUIRED_DETERMINISTIC_GATE | None for policy | Budget/circuit/model gates | Hidden prefetch or fallback | Evidence required. |
| Deterministic Compute | DETERMINISTIC_PROTECTED_EXECUTION | Explain/propose formula | PH1.COMP replayable compute | Provider final math | Calculation authority. |
| Client Route Presentation | MIXED_PROBABILISTIC_WITH_DETERMINISTIC_GATES | Upstream wording only | Cloud-authored route state | Client semantic routing | Client is render-only. |
| Conversational Experience + Quick Assist | PROBABILISTIC_FIRST | Comfort, guide, clarify, explain, format | State/risk/scope validation + PH1.WRITE approval | Hardcoded wake/weather/help wording | Official Expansion Register stack #15; Quick Assist Activation Pack required before runtime implementation. |
| Selene Emotional Intelligence + Relationship Presence | PROBABILISTIC_FIRST | Persona wording, humor, warmth, emotional phrasing, tone continuity | Persona policy, PH1.WRITE/PH1.TTS approval, PH1.M memory law, access/protected-risk gates | Hardcoded personality phrase lists or Desktop/Adapter persona brain | Official Expansion Register stack #16; Selene Emotional Intelligence + Relationship Presence Activation Pack required before runtime implementation. |
| Old Compatibility Retirement | REQUIRED_DETERMINISTIC_GATE | None | Active-caller/proof ledger | Delete because it looks old | No deletion now. |

Detected drift risks from repo truth:

- `crates/selene_adapter/src/lib.rs` contains `deterministic_active_context_followup_query`, `deterministic_weather_context_followup_query`, `ph1m_actor_recent_recall_assertion`, `maybe_run_ph1d_public_answer`, and `run_ph1d_public_answer`. These are retained compatibility/risk surfaces, not expansion targets.
- `crates/selene_os/src/ph1x.rs` contains `ph1x_universal_active_context_followup_query` and `Slice3aOneLineProviderProposal`. These are PH1.X-owned current/recent compatibility surfaces awaiting semantic proposal activation.
- `crates/selene_engines/src/ph1write.rs` contains deterministic `run_one_line_current_equivalent` and fallback output guards. Safety validation is allowed; deterministic language behavior must not become primary UX architecture without JD approval.
- Desktop/iPhone route parsing contains `inviteLike`, `appOpenLike`, and `openLike`. These are client presentation risks and must remain render-only.

These surfaces remain `PROBABILISTIC_INTERACTION_DRIFT_RISK` until canonical probabilistic-first proposal/writing paths and deterministic Selene validation gates replace or contain them with backend evidence.

### Deterministic Implementation Approval Gate

This activation pack plans no runtime deterministic logic. Future deterministic behavior must be classified before editing:

| Deterministic item | Classification | Approval rule |
|---|---|---|
| Access checks, authority gates, simulation gates, provider-off gates, schema validation, source acceptance, memory permission, audit/idempotency | REQUIRED_DETERMINISTIC_GATE | Allowed only in canonical owner with tests and evidence. |
| Phrase matching, keyword contains routing, hardcoded wake replies, hardcoded clarification wording, hardcoded weather/time presentation, stack-local language understanding outside SemanticInterpreterProvider/PH1.X, Desktop/Adapter semantic shortcuts | FORBIDDEN_DETERMINISTIC_LANGUAGE_PATCH | Do not implement; report as old compatibility or wrong-owner risk. |
| Any deterministic behavior that affects user interaction, routing, presentation, conversation flow, clarification, suggestions, or normal assistant behavior | JD_APPROVAL_REQUIRED_DETERMINISTIC_BEHAVIOR | Stop before editing with `DETERMINISTIC_IMPLEMENTATION_APPROVAL_REQUIRED`. |

## 6. Phase-by-Phase Slice Breakdown

| Phase | Title | Stacks included | Repo owner files to inspect before implementation | Activation packs required | Estimated slices | Slice names | Proof required | Provider-off/fake-provider | JD live | Old paths retained / blockers |
|---:|---|---|---|---|---:|---|---|---|---|---|
| 0 | Architecture Docs and Index Complete | Docs/index only | `docs/*ARCHITECTURE*`, index docs | none | 1 | Docs publication proof | clean tree, docs-only diff | no | no | no old paths deleted |
| 1 | Repo-Truth Activation Pack | All stacks | all owner files in this pack | this pack | 1 | Overall repo-truth activation | all stacks mapped, no-skip proof | no | no | unknowns marked |
| 2 | Provider Governance Baseline | T, Provider Assist | `ph1providerctl.rs`, `ph1d.rs`, model policy, Adapter provider route | Provider governance activation | 5 | provider-off counters; fake provider; model policy evidence; malformed rejection; data-egress shell | zero attempts/dispatches when off, model evidence, fake provider non-billable | yes/yes | no | fallback and live provider routes retained |
| 3 | Semantic Meaning Proposal Baseline | A, Quick Assist, Selene Emotional Intelligence | PH1.X, PH1.D, providerctl, Adapter, PH1.WRITE/PERSONA/EMO references | GHCS activation + Selene Emotional Intelligence + Relationship Presence activation | 4 | semantic packet skeleton; fake semantic provider; malformed proposal rejection; Quick Assist and Selene wording proposal shells | schema rejection, owner candidate not authority, persona proposal cannot grant access/authority | yes/yes | no | PH1.X/Adapter deterministic helpers retained |
| 4 | PH1.X Deterministic Validation Baseline | A, J, K | PH1.X contracts/engines/OS, runtime_session_foundation | GHCS + VIA activation | 5 | proposal-to-directive; target ledger; protected risk; wrong-owner reject; active/recent boundary | HumanConversationDirective evidence | yes/yes | targeted when visible | phrase patches retained but blocked from expansion |
| 5 | PH1.WRITE Presentation Baseline | E, F, M, N, Quick Assist, Selene Emotional Intelligence | PH1.WRITE, PH1.TTS, PH1.E presentation packet, PH1.PERSONA, PH1.EMO, PH1.M, clients | PH1.WRITE activation + Selene Emotional Intelligence + Relationship Presence activation | 6 | display/tts split; one-line rewrite; style/format; Quick Assist final wording; Selene emotional presentation final wording; raw provider output guard | formatted/display/tts/persona evidence | yes/yes if provider wording | yes for visible text/TTS | deterministic formatting/persona helpers retained |
| 6 | Web Search + Source Evidence + Source Chips | B, D | PH1.E, PH1.SEARCH, web_search_plan, providerctl | PH1.E search activation | 6 | search need directive; provider-off; fake search; source acceptance; claim verification; source chips | accepted-source and claim ledger | yes/yes | yes for user-visible search | no live search by default |
| 7 | Identity + Access + Authority Baseline | K, J | PH1.W/L/C, Voice ID, access/policy/gov, runtime_execution | VIA activation | 7 | wake boundary; session binding; Voice ID evidence-only; access scope; authority packet; PH1.X access validation; audit posture | unknown/confirmed/protected matrices | no unless wording | yes for identity flows | Voice ID cannot grant authority |
| 8 | PH1.M Human Memory Core Lifecycle + Preference Boundary | G, Persona/Preference, Selene Emotional Intelligence | PH1.M, PH1.X, PH1.L, PH1.E, PH1.WRITE, access/session, persona/learn/emo/write, Desktop/Adapter compatibility surfaces | PH1.M Human Memory Core activation + Selene Emotional Intelligence + Relationship Presence activation | 11 | repo-truth gap audit; memory core contracts; recall orchestrator; encoding/salience/consolidation; fresh memory continuation; topic memory/graph; natural memory language via PH1.WRITE; deep recall; trust/privacy/conflict/staleness; natural memory UI; human memory eval matrix | PH1.M evidence, recall style, age labels, trust/privacy/conflict/staleness refs, provenance/audit refs, persona preference refs | yes/yes if provider salience/consolidation/persona wording is used | yes for memory/persona UX | Adapter/Desktop/PH1.X/PH1.E/PH1.WRITE memory shortcuts retained until proof |
| 9 | Tool/File/Connector Scope | H, I | PH1.E, selene_tools, PH1.DOC, providerctl | Tool/file activation | 6 | file admission; injection defense; tool proposal; read-only execution; connector scope; write fail-closed | tool/file/source evidence | yes/yes | visible where tool/file used | provider tool execution forbidden |
| 10 | Voice/Wake/Session/Realtime Route Proof | L, Quick Assist, Selene Emotional Intelligence | PH1.W/C/L/TTS, Voice ID, PH1.WRITE, PH1.EMO/PERSONA, Adapter http/bin, Desktop/iPhone | Voice route activation + Quick Assist activation + Selene Emotional Intelligence + Relationship Presence activation | 7 | transcript admission; realtime/off proof; TTS approved text; wake acknowledgement via Quick Assist; Selene wake/emotional presentation wording under PH1.WRITE/TTS; barge-in; real app route proof | transcript/session/tts/write/persona/Desktop provenance | yes/yes | yes | local STT/TTS fallback claims retained fail-closed |
| 11 | Visual Recognition + OCR + Media Evidence | Visual Recognition, H | PH1.VISION, PH1.DOC, PH1.OS OCR | Visual/media activation | 5 | media admission; OCR evidence; object/evidence extraction; injection defense; privacy/data-egress | visual evidence refs | yes/yes | yes where visible | OCR text not instruction |
| 12 | Visual Rendering + Image Cards + Media Presentation | C, Visual Rendering | PH1.E, PH1.WRITE, Desktop/iPhone | Media presentation activation | 4 | image source-page proof; display eligibility; client render-only; TTS visual summary | image-card evidence | yes/yes | yes | clients cannot choose images |
| 13 | Broadcast/Delivery/Reminder/Messaging | Broadcast/Delivery | PH1.BCAST, DELIVERY, REM, Access, Simulation | Broadcast activation | 4 | draft; schedule; send side-effect gate; delivery audit | delivery/reminder evidence | maybe/yes for wording | yes | no side effects without authority |
| 14 | Onboarding/Invite/Link/Enrollment | Onboarding/Invite | PH1.ONB, LINK, W, Voice ID, clients | Onboarding activation | 5 | link state; onboarding requirements; enrollment handoff; client route render; recovery | onboarding/link evidence | yes for help wording | yes | client route parsing retained |
| 15 | Master Access Template/Admin/Tenant/Governance | Access Template, Tenant/Gov/Quota | PH1.ACCESS, POLICY, GOV, TENANT, QUOTA | Access/admin activation | 6 | role/template schema; overlay; tenant binding; quota; admin presentation; authority fail-closed | access lineage/evidence | yes for explanation | yes | no admin shortcuts |
| 16 | Platform Ops | Work/Lease/Sched/Health/KMS/Export | PH1.WORK, LEASE, SCHED, HEALTH, KMS, EXPORT | Platform ops activation | 6 | work ledger; lease; scheduler; health; KMS handle; export proof | ops audit refs | maybe/yes for explanation | visible where app uses | no secrets in output |
| 17 | Artifact Trust/Document/Export/Provenance | O, Artifact Trust | PH1.ART, DOC, EXPORT, WRITE | Artifact activation | 5 | provenance; draft/official split; export scope; artifact cards; trust-root gap | artifact provenance refs | yes/yes | yes | no official claim without proof |
| 18 | Persona/Preference/Emotion/Feedback/Learning | Persona stack, G, Selene Emotional Intelligence | PH1.PERSONA, EMO, FEEDBACK, LEARN, M, WRITE, TTS | Persona activation + Selene Emotional Intelligence + Relationship Presence activation | 6 | persona hints; emotion guide; feedback capture; learning signal; preference permission; Selene emotional presentation policy and serious-mode proof | identity-scoped evidence plus persona/write/tts refs | yes/yes | yes | no private personalization for unknown speaker; no persona safety bypass |
| 19 | Provider Assist/Cost/Prefetch/Arbitration | Provider Assist, T | PH1.COST, PREFETCH, CACHE, PAE, providerctl | Provider assist activation | 5 | cost policy; prefetch no-network proof; cache evidence; arbitration; circuit breaker | provider counters/budget refs | yes/yes | no | hidden prefetch forbidden |
| 20 | Deterministic Compute/Consensus/Calculation Authority | R, Deterministic Compute, J | PH1.COMP, PH1.E, SimulationExecutor | Compute activation | 4 | computation packet; consensus; advisory/official split; protected calculation fail-closed | replayable compute evidence | no unless explanation | maybe | provider math not official |
| 21 | Video Recognition/Rendering/Generation | Q, video expansion | PH1.VISION media, model policy, clients | Video activation | 5 | video owner map; provider-off/fake; safety; provenance; render card | media/provenance/cost refs | yes/yes | yes | model approval unclear |
| 22 | Client Route Presentation/App Open/Invite Rendering | Client Route | Desktop/iPhone, Adapter | Client route activation | 4 | app-open render-only; invite render-only; route provenance; no-authority tests | client provenance | no | yes | client contains route parsing retained |
| 23 | Evaluation/JD Live Acceptance System | U | tests, reports, storage/audit, clients | Eval activation | 4 | eval case packet; backend evidence verifier; JD live pack; smoke matrix | backend evidence agreement | as stack requires | yes | no stale path wins |
| 24 | Old Compatibility Path Retirement | Old register | Adapter, PH1.X, PH1.WRITE, PH1.D, clients | Old path retirement activation | 6 | ledger; active-caller check; replacement proof; regression; JD live; deletion slice | no active caller, clean proof | where relevant | yes where visible | deletion forbidden until all gates pass |

## 7. First Safe Implementation Recommendation

Recommended first real implementation build:

`Build Provider Governance Baseline + SemanticInterpreterProvider Fake Proposal Path`

Repo truth supports this as the safest first build because:

- Provider governance already has strong partial surfaces in `crates/selene_engines/src/ph1providerctl.rs`, including provider-off/fake-provider/budget/fallback/counter concepts.
- PH1.D provider transport evidence already records model, dispatch count, fallback/cheap/unapproved flags, and raw-output exposure status in `crates/selene_kernel_contracts/src/ph1d.rs`.
- The model policy exists in `docs/SELENE_OPENAI_MODEL_ROUTING_POLICY.md`.
- Semantic proposal packet names are not yet exact repo truth, so a minimal fake-provider proposal skeleton must be built before live semantic routing.
- Wake/Voice ID/Access is product-critical and should run alongside the semantic foundation soon, but semantic provider governance must exist before GPT-5.5-assisted interaction is allowed.

The first build should not call live OpenAI. It should prove provider-off zero attempts, fake provider non-billable behavior, malformed semantic proposal rejection, no fallback/cheaper model, and backend evidence envelope correctness.

## 8. Wake / Session / Voice ID / Access Early Spine Activation

Current files:

- Wake: `crates/selene_kernel_contracts/src/ph1w.rs`, `crates/selene_engines/src/ph1w.rs`, `crates/selene_os/src/ph1w.rs`, `crates/selene_storage/migrations/0011_ph1w_wake_tables.sql`
- Session: `crates/selene_kernel_contracts/src/ph1l.rs`, `crates/selene_os/src/runtime_session_foundation.rs`, `crates/selene_os/src/runtime_ingress_turn_foundation.rs`
- Voice ID: `crates/selene_kernel_contracts/src/ph1_voice_id.rs`, `crates/selene_engines/src/ph1_voice_id.rs`, `crates/selene_os/src/ph1_voice_id.rs`, migration `0008_ph1vid_voice_enrollment_tables.sql`
- Access/authority posture: `crates/selene_kernel_contracts/src/ph1access.rs`, `ph1policy.rs`, `ph1gov.rs`, `runtime_execution.rs`, `runtime_law.rs`
- Clients: `apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`, `SeleneMacDesktopRuntimeBridge.swift`, `apple/iphone/SeleneIPhone/SessionShellView.swift`

Current identity/access packet names or equivalents:

- `WakeDecision` is the current wake packet equivalent.
- `SessionAttachOutcome`, `SessionAccessSnapshot`, `Stage5TurnAuthorityPacket`, and `Stage6AccessContextPacket` are current session/access posture equivalents.
- `Ph1VoiceIdRequest` and `Ph1VoiceIdResponse` are current Voice ID evidence equivalents.
- `AuthorityExecutionState` and `AuthorityPolicyDecision` are current authority posture equivalents.

What exists now:

- Wake, session, Voice ID, access/policy/gov, and runtime law files exist.
- Desktop/iPhone expose bounded voice/session/onboarding/render surfaces and repeatedly state no local authority.
- Runtime session foundation carries access/authority posture concepts.

What is missing:

- Exact `SessionIdentityBindingPacket`, `AccessScopePacket`, `SpeakerIdentityEvidencePacket`, `AuthorityDecisionPacket`, and `SimulationGateDecisionPacket` architecture names are not fully present as exact symbols.
- Voice ID evidence-only behavior needs canonical proof against access/authority grants.
- Wake/session acknowledgement wording is not yet probabilistic-first Quick Assist under PH1.WRITE validation.
- Selene wake/emotional presentation wording is not yet official runtime behavior and must be routed through PH1.WRITE/PH1.TTS after wake/session state validation.

Earliest safe implementation slices:

1. `Build Session Identity/Access Posture Baseline`
2. `Build Voice ID Evidence-Only Contract Proof`
3. `Build Access Scope Baseline For Public/Private/Protected`
4. `Build PH1.X Identity/Access Validation Integration`
5. `Build Authority + Simulation Fail-Closed Proof`
6. `Build Quick Assist Wake/Session Reassurance Wording Through PH1.WRITE`
7. `Build Selene Wake Greeting And Serious-Mode Persona Through PH1.WRITE`

Connections:

- PH1.X consumes identity/access posture when validating semantic proposals.
- PH1.M consumes access scope before private or durable memory recall.
- PH1.E consumes access/tool/file/search scope before private files/connectors/tools.
- Authority and Simulation consume PH1.X protected risk plus deterministic access/authority state.

JD live tests needed:

- Unknown speaker public question allowed where policy permits.
- Unknown speaker private memory denied.
- Unknown speaker protected action fails closed.
- Confirmed JD private memory route only with access proof.
- Wake accepted acknowledgement is natural, varied, and PH1.WRITE-approved.
- Client displays only runtime-approved state.

## 9. OpenAI Model Governance Activation

Model policy evidence is in `docs/SELENE_OPENAI_MODEL_ROUTING_POLICY.md`. Repo route evidence appears in `crates/selene_adapter/src/lib.rs`, `crates/selene_adapter/src/bin/http_adapter.rs`, `crates/selene_os/src/ph1os.rs`, and PH1.D provider evidence contracts.

| Function | Approved Repo Model / Provider | Current Evidence Path | Status | MODEL_POLICY_MISSING | Activation Needed |
|---|---|---|---|---|---|
| semantic interpretation | `gpt-5.5` | `docs/SELENE_OPENAI_MODEL_ROUTING_POLICY.md`; PH1.D live envs `SELENE_PH1D_LIVE_MODEL_ID`/`OPENAI_MODEL` | PARTIAL | no | SemanticInterpreterProvider interface and fake provider. |
| writing | `gpt-5.5` | model policy; PH1.WRITE current deterministic output | PARTIAL | no | Writing provider governance and PH1.WRITE final validation. |
| web search | `gpt-5.5 with web search tool where permitted` | model policy; PH1.E/providerctl | PARTIAL | no | Search provider capability and source acceptance activation. |
| file search | policy-covered as file/search capability; exact model unclear | model policy and PH1.E/PH1.DOC surfaces | UNCLEAR | yes for exact model | File-search model/capability activation. |
| deep research | `o3-deep-research` if available and approved; fallback only with JD approval | model policy | PARTIAL | no | Deep research opt-in/provider governance activation. |
| embeddings | `text-embedding-3-large` | model policy | PARTIAL | no | Embedding governance and PH1.M retrieval scope activation. |
| STT | `gpt-4o-transcribe`; diarization `gpt-4o-transcribe-diarize` when needed | model policy; `http_adapter.rs` hardcoded/default references | PARTIAL | no | STT provider governance and no local fallback proof. |
| TTS | `gpt-4o-mini-tts` | model policy; `http_adapter.rs`; Desktop TTS surfaces | PARTIAL | no | TTS approval path and PH1.WRITE/PH1.TTS split. |
| realtime voice | `gpt-realtime-2`, `gpt-realtime-whisper`, `gpt-realtime-translate` | model policy | PARTIAL | no | Realtime activation and provider-off proof. |
| image understanding | `gpt-5.5` / approved vision route per policy | model policy; PH1.VISION | PARTIAL | no | Vision provider governance and media privacy gates. |
| image generation | `ImageGenerationProvider` | model policy | UNCLEAR | yes for exact model | ImageGenerationProvider activation and JD approval. |
| video generation | `VideoGenerationProvider`; Sora/current official model only after JD approval | model policy | UNCLEAR | yes until approved for slice | Video generation activation and model approval proof. |
| translation | `gpt-5.5` / realtime translate where realtime | model policy | PARTIAL | no | Language/translation activation. |
| code assistance | `gpt-5.5 with code interpreter tool where explicitly approved` | model policy | PARTIAL | no | Code-assist policy and AGENTS execution boundary. |
| evals/graders | model policy mentions eval/grader use as governed capability | model policy | UNCLEAR | yes for exact stack model | Eval/grader activation pack. |
| moderation/safety | `omni-moderation-latest` | model policy | PARTIAL | no | Safety/moderation capability activation. |

Codex must not choose models from memory or preference. Any missing exact model remains `MODEL_POLICY_MISSING` until repo policy is updated by JD-approved instruction.

## 10. Provider Governance Activation

Current mapped pieces:

- Provider registry/arbitration: `crates/selene_engines/src/ph1providerctl.rs` with `ProviderRegistryEntry`, `ProviderCapability`, `ProviderRouteDecision`, `ProviderControlProvider`.
- Model policy: `docs/SELENE_OPENAI_MODEL_ROUTING_POLICY.md`.
- Kill switch/provider-off: `ProviderNetworkPolicy`, `ProviderControlMode`, provider disabled tests in `ph1providerctl.rs`.
- Provider-off behavior: counters and gate decisions exist; activation must prove zero attempts and zero network dispatches by stack.
- Fake provider: `test_fake_provider` and fake provider decisions exist in providerctl.
- Budget counters/network counters: `ProviderCallCounter`, `provider_network_dispatch_count`, `provider_budget_denied_count`.
- PH1.D evidence: `Ph1dProviderTransportEvidence`, `Ph1dProviderErrorEvidence`.
- Model evidence: expected/actual model fields in PH1.D evidence and Adapter log lines.
- Provider failure behavior: PH1.D failure evidence and providerctl fixture failures exist.
- Data-egress/privacy: architecture requires packets; exact packet names not fully mapped.
- Live provider opt-in tests: Slice 3C/3G/3H/3I evidence surfaces in Adapter tests; live env vars are explicit.
- No startup provider probes: provider-off tests should be expanded to every new stack.

Missing pieces:

- Canonical `SemanticInterpreterProvider` interface.
- Exact `ProviderPreflightPacket`/`ProviderDecisionTracePacket` architecture names.
- Stack-level provider-off/fake-provider proof matrix.
- Data-egress/privacy/redaction packet equivalents for semantic provider calls.

## 11. PH1.X / Semantic Meaning Activation

Current PH1.X files:

- `crates/selene_kernel_contracts/src/ph1x.rs`
- `crates/selene_engines/src/ph1x.rs`
- `crates/selene_os/src/ph1x.rs`

Current repo truth:

- `HumanConversationDirective` exists in contracts.
- `ActiveContextPacket`, `Ph1xDirective`, and rejection/candidate ledger structures exist.
- `Slice3aOneLineProviderProposal` exists in OS PH1.X.
- `ph1x_universal_active_context_followup_query` exists and is currently a deterministic active-context compatibility path.

Activation gaps:

- No exact `SemanticMeaningProposalPacket` found.
- Provider proposal schema, multiple candidates, confidence/risk fields, owner candidate validation, and malformed output rejection need implementation.
- PH1.X must validate provider proposals into `HumanConversationDirective` without letting provider output become authority.
- PH1.X must reject or reroute wrong-owner proposals.
- PH1.X must preserve current-vs-recent boundary and not become durable memory.

Phrase-patch risks:

- Existing deterministic follow-up and target helpers must be retained only as compatibility until provider semantic proposal path proves replacement.
- Future builds must mark any raw-language stack routing outside PH1.X as `GLOBAL_SEMANTIC_SPINE_BYPASS_RISK`.

First PH1.X activation slices:

1. Semantic proposal contract mapping.
2. Fake semantic proposal validation.
3. Previous-answer target directive proof.
4. Protected-risk proposal fail-closed proof.
5. Candidate/rejection ledger evidence.

## 12. PH1.WRITE / Presentation Activation

Current files:

- `crates/selene_kernel_contracts/src/ph1write.rs`
- `crates/selene_engines/src/ph1write.rs`
- `crates/selene_os/src/ph1write.rs`
- `crates/selene_kernel_contracts/src/ph1e.rs` for `PresentationPacket`
- Desktop/iPhone render paths

Current output equivalents:

- `Ph1WriteRequest`
- `Ph1WriteOk`
- `Ph1WriteResponse`
- `formatted_text`
- PH1.E `PresentationPacket` with `response_text`, `tts_text`, `source_chips`, and `image_cards`

Activation needs:

- Canonical display/tts/source/image/video/artifact output contract map.
- PH1.WRITE validation for provider-written text.
- Source chips and image cards accepted only from PH1.E.
- Artifact cards accepted only from artifact/export/provenance owners.
- Language/tone/format routing from PH1.X directive.
- Quick Assist stack for comfort, guidance, wake acknowledgements, failed-step recovery, and process explanations.
- Selene emotional presentation and emotional presentation through PH1.WRITE, PH1.EMO, PH1.PERSONA, PH1.M where memory law allows, PH1.TTS, and Provider Governance.

Missing presentation packet contracts:

- Exact `WriteOutputPacket` not found; repo equivalents exist.
- Video card packet not mapped.
- Artifact card packet needs artifact/export activation.
- Conversational Experience + Quick Assist is now official in the Expansion Register as stack #15, but still requires a Quick Assist Activation Pack before runtime implementation.
- Selene Emotional Intelligence + Relationship Presence is now official in the Expansion Register as stack #16, but still requires a Selene Emotional Intelligence + Relationship Presence Activation Pack before runtime implementation.

First presentation slices:

1. PH1.WRITE display/tts split baseline.
2. Provider-assisted wording with validation.
3. One-line rewrite via validated target.
4. Source/image/artifact presentation packet acceptance.
5. Quick Assist natural wording under PH1.WRITE.
6. Selene emotional presentation/emotional wording under PH1.WRITE and PH1.TTS.

## 13. PH1.E / Search / Tool / File / Evidence Activation

Mapped files:

- `crates/selene_kernel_contracts/src/ph1e.rs`
- `crates/selene_engines/src/ph1e.rs`
- `crates/selene_os/src/ph1e.rs`
- `crates/selene_tools/src/ph1e.rs`
- `docs/web_search_plan/*`
- `crates/selene_kernel_contracts/src/ph1doc.rs`

Mapped capabilities:

- Search/source: `SourceEvaluationPacket`, `SourceChipPacket`, `SourceCardPacket`, `ClaimVerificationPacket`, `WebAnswerVerificationPacket`
- Tool: `ToolRequest`, `ToolResult`, `ToolResponse`, tool router
- Image search/presentation: `SearchImagePacket`
- File/document: PH1.DOC surfaces and file/document source refs

Activation gaps:

- Exact `ToolProposalPacket` and `PromptInjectionDefensePacket` names not found.
- Deep research stack needs explicit opt-in and budget.
- File search model/capability exact policy is unclear.
- URL/page fetch policy must be mapped before live fetch implementation.
- Prompt-injection defense must treat source/file/tool text as facts, not instructions.

First PH1.E slices:

1. Source acceptance and source-chip proof.
2. Provider-off/fake search proof.
3. Claim verification and unsupported claim removal.
4. File admission and prompt-injection defense.
5. Tool proposal vs execution decision split.
6. Image-backed search display eligibility.

## 14. PH1.M / Memory / Preference / Learning Activation

Dedicated master design:

- `docs/SELENE_PH1M_HUMAN_MEMORY_CORE_MASTER_DESIGN.md`

Status:

- MASTER_DESIGN_ADDED
- PARTIAL current repo support
- ACTIVATION_PACK_REQUIRED
- NO_RUNTIME_IMPLEMENTATION_YET

Rule:

PH1.M must not be implemented as session search. PH1.M must be Selene's governed human memory brain.

PH1.M is the single memory authority. Other engines may request or consume memory evidence, but must not own memory:

- PH1.X owns live context.
- PH1.L owns sleep/wake boundaries.
- PH1.E owns tools/search/files.
- PH1.WRITE owns memory wording.
- Desktop renders only.
- Adapter transports only.
- Storage files/audits.

Mapped files:

- `crates/selene_kernel_contracts/src/ph1m.rs`
- `crates/selene_engines/src/ph1m.rs`
- `crates/selene_os/src/ph1m.rs`
- `crates/selene_storage/migrations/0021_ph1m_vnext_memory_tables.sql`
- Persona/learning/emotion files: PH1.PERSONA, PH1.EMO, PH1.FEEDBACK, PH1.LEARN, PH1.KNOW, PH1.KG, PH1.PATTERN

Mapped symbols:

- `MemoryEvidencePacket`
- `MemoryRecallRequest`
- `FreshMemoryHandoff`
- `MemoryContinuationDecision`
- `MemoryTurnInput`
- `MemoryTurnOutput`

Required PH1.M lifecycle modules from the master design:

- Recall Orchestrator
- Encoding Engine
- Salience Engine
- Consolidation Engine
- Fresh Memory
- Day Memory
- Topic Memory
- Topic Graph
- Deep Recall
- Permanent Governed Memory
- Continuation Gate
- Memory Posture Engine
- Freshness Gradient
- Conflict + Staleness Checker
- Memory Trust Engine
- Memory Privacy Gate
- No-Record Handler
- Memory Evidence Packet
- Memory Use Policy
- Human Memory Eval Matrix

Activation gaps:

- Access-scoped memory recall must consume identity/access posture.
- Current vs recent vs durable memory boundary must be enforced by PH1.X/PH1.M.
- Preference/persona/emotion/learning must not create private personalization without identity/access scope.
- Selene emotional presentation must use PH1.M only for lawful durable tone preferences and must not infer private emotional state as memory without permission.
- Memory write/update/forget must obey memory law and audit.
- Current memory support is PARTIAL and must be expanded into the full governed human memory lifecycle: notice, encode, consolidate, connect, recall, continue, update, forget/decay.
- Fresh memory, day memory, topic memory, topic graph, deep recall, permanent governed memory, conflict/staleness, trust, privacy, no-record honesty, natural memory UI, and human memory evals require activation.
- Adapter `ph1m_actor_recent_recall_assertion` is a compatibility risk.
- Any memory logic in Desktop, Adapter, PH1.X, PH1.E, or PH1.WRITE must be classified as wrong-owner or compatibility until proven otherwise.

Future activation pack:

PH1.M Human Memory Core Activation Pack

Specific repo-truth questions for that activation pack:

- what current PH1.M contracts exist?
- what current recent recall exists?
- what memory evidence packets exist?
- what storage digest rows exist?
- what adapter recall routes exist?
- what duplicate recall paths exist?
- what stale memory paths exist?
- what current tests exist?
- what current provenance/audit records exist?

PH1.M Build 0-10 sequence:

1. Build 0 - PH1.M Repo Truth And Gap Audit
2. Build 1 - PH1.M Memory Core Contracts
3. Build 2 - PH1.M Recall Orchestrator
4. Build 3 - Encoding + Salience + Consolidation
5. Build 4 - Fresh Memory Continuation
6. Build 5 - Topic Memory + Topic Graph
7. Build 6 - Natural Memory Language via PH1.WRITE
8. Build 7 - Deep Recall
9. Build 8 - Trust, Privacy, Conflict, And Staleness
10. Build 9 - Natural Memory UI
11. Build 10 - Human Memory Eval Matrix

First PH1.M activation slices must start with repo-truth inventory and central contracts, not a new disconnected fresh-memory subsystem.

## 15. Visual / Media / Artifact Activation

Mapped files:

- Visual/media: `crates/selene_kernel_contracts/src/ph1vision.rs`, `crates/selene_engines/src/ph1vision.rs`, `crates/selene_engines/src/ph1vision_media.rs`, `crates/selene_os/src/ph1os.rs`
- Document/artifact/export: `crates/selene_kernel_contracts/src/ph1doc.rs`, `ph1art.rs`, `ph1export.rs`, matching engine/OS files

Mapped symbols:

- `VisionAnalyzeMediaRequest`
- `VisionOcrResult`
- `VisionDetectedObject`
- `VisionVideoTranscriptResult`
- `VisionKeyframeIndexResult`
- `VisionMediaOutput`
- `VisionEvidenceItem`
- `DocumentSourceRef`
- `ArtifactTrustDecisionProvenance`
- `ExportRequestEnvelope`

Activation gaps:

- Exact `VisualEvidencePacket` exists only as equivalents.
- OCR prompt-injection defense must be activated.
- Image rendering and media presentation must remain PH1.WRITE/PH1.E approved.
- Video recognition/rendering/generation need dedicated activation and model approval.
- Artifact trust/provenance/export must separate draft/generated/exported/official states.

First visual/media/artifact slices:

1. Media admission and visual evidence packet map.
2. OCR extraction with prompt-injection defense.
3. Image card display eligibility.
4. Artifact provenance baseline.
5. Export access/redaction proof.

## 16. Enterprise Operations Activation

Mapped files:

- Broadcast/delivery/reminder: `ph1bcast.rs`, `ph1delivery.rs`, `ph1rem.rs`
- Onboarding/invite/link: `ph1onb.rs`, `ph1link.rs`, Desktop/iPhone route surfaces
- Access templates/admin: `ph1access.rs`, `ph1policy.rs`, `ph1gov.rs`
- Tenant/quota/governance: `ph1tenant.rs`, `ph1quota.rs`, `ph1gov.rs`
- Platform ops: `ph1work.rs`, `ph1lease.rs`, `ph1sched.rs`, `ph1health.rs`, `ph1kms.rs`, `ph1export.rs`

Activation gaps:

- External messaging side effects must be separated from drafts.
- Onboarding/invite link state must be cloud-authoritative.
- Role/template/admin controls require deterministic authority and simulation where mutating.
- Tenant/workspace/quota must be available before private/provider stack work.
- KMS/export must never expose secrets; export needs access/redaction/audit.

First enterprise ops slices:

1. Broadcast/delivery draft-vs-send boundary.
2. Reminder schedule authority boundary.
3. Onboarding/link state activation.
4. Access template/admin read-only proof.
5. Tenant/quota/governance scope baseline.
6. Health/KMS/export no-secret proof.

## 17. Deterministic Compute / Protected Execution Activation

Mapped files:

- `crates/selene_kernel_contracts/src/ph1comp.rs`
- `crates/selene_engines/src/ph1comp.rs`
- `crates/selene_os/src/ph1comp.rs`
- `crates/selene_os/src/simulation_executor.rs`
- `crates/selene_kernel_contracts/src/runtime_law.rs`
- `crates/selene_kernel_contracts/src/runtime_execution.rs`

Mapped symbols:

- `ComputationPacket`
- `ComputationConsensusResult`
- `ComputationExecutionState`
- `RuntimeProtectedActionClass`
- `RuntimeLawExecutionState`
- `AuthorityExecutionState`

Activation gaps:

- Official vs advisory calculation/reporting boundary.
- Protected business execution must route through authority + SimulationExecutor.
- Provider can explain/propose formulas but cannot own deterministic compute authority.
- Consensus/calc evidence must be replayable and auditable.

First compute/protected slices:

1. Advisory vs official compute classification.
2. PH1.COMP computation packet evidence.
3. Protected calculation authority fail-closed.
4. Simulation gate integration with compute/report outputs.

## 18. Desktop / iPhone / Adapter Boundary Activation

Desktop current responsibilities:

- Captures user actions/audio, renders session/wake/onboarding/provenance/output state, plays approved TTS.
- Files: `apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`, `SeleneMacDesktopRuntimeBridge.swift`.

iPhone current responsibilities:

- Renders session/onboarding/app-open/explicit voice state and produces bounded explicit turn requests.
- File: `apple/iphone/SeleneIPhone/SessionShellView.swift`.

Adapter current responsibilities:

- Transports runtime requests/responses, provider evidence, Desktop voice/runtime payloads, and compatibility paths.
- Files: `crates/selene_adapter/src/lib.rs`, `crates/selene_adapter/src/bin/http_adapter.rs`, `grpc_adapter.rs`.

Current app-open/invite route parsing:

- iPhone has repeated `inviteLike`, `appOpenLike`, and `openLike` route classifiers.
- Desktop has invite/open route parsing and route-kind assignment in `DesktopSessionShellView.swift`.

Current render-only proof gaps:

- Clients display authoritative response text and route states, but route parsing must be proven not to grant authority.
- Desktop TTS surfaces must prove approved `tts_text`/authoritative response only.
- iPhone and Desktop local STT fallback is intentionally disabled in visible wording; keep fail-closed.

Current Adapter `lib.rs` compatibility surfaces requiring later retirement:

- `maybe_run_ph1d_public_answer`
- `run_ph1d_public_answer`
- `h406_public_advisory_fallback_answer`
- `fallback_runtime_execution_envelope_for_voice_turn_request`
- `fallback_runtime_execution_envelope_for_voice_turn_request_with_identities`
- `deterministic_active_context_followup_query`
- `deterministic_weather_context_followup_query`
- `ph1m_actor_recent_recall_assertion`

Proof required before retirement:

- Canonical replacement owner built.
- Provider-off and fake-provider proof where relevant.
- Backend evidence proves correct owner.
- JD live acceptance passes where visible.
- Old behavior regression passes.
- Active-caller check proves no remaining live caller.

First client/adapter boundary slices:

1. Adapter transport-only evidence map.
2. Desktop render-only route proof.
3. iPhone render-only route proof.
4. TTS approved-output proof.
5. Active-caller ledger for compatibility paths.

## 19. Old Compatibility Path Retirement Ledger

| Old Path / Symbol | File | Current Status | Correct Owner Replacement | Active Caller Check Needed | Proof Before Retirement | Earliest Retirement Phase |
|---|---|---|---|---|---|---|
| `deterministic_active_context_followup_query` | `crates/selene_adapter/src/lib.rs` | ACTIVE compatibility | PH1.X semantic proposal + target validation | yes | one-line/rewrite/slot follow-up backend evidence + JD live | Phase 24 |
| `deterministic_weather_context_followup_query` | `crates/selene_adapter/src/lib.rs` | ACTIVE compatibility | SemanticInterpreterProvider -> PH1.X -> PH1.E time/weather/tool owner -> PH1.WRITE | yes | weather/time follow-up tests, provider-off proof, JD live | Phase 24 |
| `ph1x_universal_active_context_followup_query` | `crates/selene_os/src/ph1x.rs` | ACTIVE PH1.X compatibility | Canonical semantic proposal validation in PH1.X | yes | target ledger and stale rejection evidence | Phase 24 |
| `run_ph1d_public_answer` / `maybe_run_ph1d_public_answer` | `crates/selene_adapter/src/lib.rs` | ACTIVE route | PH1.X first; PH1.D public answer only when no directive claims turn | yes | live follow-up route proof and PH1.D public-answer fallback tests | Phase 24 |
| `h406_public_advisory_fallback_answer` | `crates/selene_adapter/src/lib.rs` | ACTIVE fallback wording | PH1.WRITE + Quick Assist provider-assisted wording | yes | presentation and provider-off degraded wording proof | Phase 24 |
| `fallback_runtime_execution_envelope_for_voice_turn_request*` | `crates/selene_adapter/src/lib.rs`; `crates/selene_os/src/ph1os.rs` | ACTIVE fallback envelope | Runtime session/voice canonical envelope owner | yes | voice/session route proof, no authority widening | Phase 24 |
| `ph1m_actor_recent_recall_assertion` | `crates/selene_adapter/src/lib.rs` | ACTIVE memory-adjacent compatibility | PH1.M access-scoped memory evidence | yes | memory scope/access/JD live proof | Phase 24 |
| PH1.WRITE deterministic `run_one_line_current_equivalent` | `crates/selene_engines/src/ph1write.rs` | ACTIVE output helper | PH1.WRITE final output behind PH1.X directive and provider writing proposal where allowed | yes | provider-assisted one-line validated output proof | Phase 24 |
| iPhone `inviteLike`/`appOpenLike`/`openLike` | `apple/iphone/SeleneIPhone/SessionShellView.swift` | ACTIVE client route parsing | Cloud-authored route state + render-only client | yes | client no-authority and route provenance proof | Phase 24 |
| Desktop invite/open route parsing | `apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift` | ACTIVE client route parsing | Cloud-authored route state + render-only client | yes | client no-authority and route provenance proof | Phase 24 |
| Adapter payroll/protected helper shortcuts | `crates/selene_adapter/src/lib.rs` | RISK/compatibility | PH1.X protected risk + Authority + SimulationExecutor | yes | protected fail-closed and backend evidence | Phase 24 |
| Provider fallback routes | `crates/selene_engines/src/ph1providerctl.rs` | ACTIVE governed fallback surface | Provider Governance policy with explicit JD approval | yes | fallback disabled by default, cap proof, model policy proof | Phase 24 or never if retained as policy path |
| Desktop/iPhone local STT/TTS fallback messages | Desktop/iPhone files | ACTIVE fail-closed UX | PH1.TTS/STT governed provider path | yes | no local fallback success, approved TTS proof | Phase 24 |

Deletion is not authorized by this ledger.

## 20. Test / Proof / JD Live Matrix

| Stack / Phase | Unit Tests | Integration Tests | Provider-Off Proof | Fake-Provider Proof | Backend Evidence | JD Live Proof | Smoke Path |
|---|---|---|---|---|---|---|---|
| Phase 0 Docs | docs/index diff | n/a | n/a | n/a | git clean/diff proof | no | docs-only |
| Phase 1 Activation | no runtime tests | repo-truth scans | n/a | n/a | activation matrix | no | docs-only |
| Phase 2 Provider Governance | providerctl, PH1.D tests | Adapter/provider route | yes | yes | provider counters/model evidence | no | provider-off/fake |
| Phase 3 Semantic Proposal | semantic schema tests | PH1.X fake proposal route | yes | yes | proposal + rejection ledger | no | fake provider |
| Phase 4 PH1.X Validation | PH1.X directive tests | runtime turn fixture | yes if provider involved | yes | HumanConversationDirective refs | yes for visible follow-up | two-turn fixture |
| Phase 5 PH1.WRITE/Quick Assist/Selene | PH1.WRITE/persona output tests | PH1.X -> PH1.WRITE -> PH1.TTS with PH1.EMO/PERSONA refs where used | yes if provider wording | yes | WriteOutput/formatted_text/persona/TTS refs | yes | visible text/TTS |
| Phase 6 Search | PH1.E/source tests | provider-off/fake search | yes | yes | SourceChip/ClaimVerification refs | yes | source-backed answer |
| Phase 7 Identity/Access | wake/session/Voice ID/access tests | runtime protected/public matrix | no | no | identity/access/authority refs | yes | public/private/protected voice/text |
| Phase 8 Memory | PH1.M tests | PH1.X -> PH1.M with access | yes if salience provider | yes | MemoryEvidence refs | yes | recall/deny |
| Phase 9 Tools/Files | PH1.E/tool/file tests | file/tool route | yes | yes | Tool/File evidence refs | yes where visible | read-only and fail-closed |
| Phase 10 Voice/Realtime | PH1.W/L/TTS/adapter/persona tests | Desktop/iPhone voice route | yes | yes | transcript/session/tts/persona/client refs | yes | real app voice |
| Phase 11 Visual Recognition | PH1.VISION tests | media admission route | yes | yes | Vision evidence refs | yes | image/OCR fixture |
| Phase 12 Visual Rendering | PH1.E/WRITE presentation tests | client render route | yes where provider | yes | image card/source refs | yes | visual card |
| Phase 13 Broadcast/Delivery | BCAST/DELIVERY/REM tests | side-effect gate route | maybe | yes for wording | delivery/reminder audit refs | yes | draft/send denial |
| Phase 14 Onboarding/Invite | ONB/LINK tests | client route render | maybe | yes for help wording | link/onboarding refs | yes | invite/open flow |
| Phase 15 Access/Admin/Tenant | access/policy/gov/tenant/quota tests | admin/tenant route | no | no | access lineage/quota refs | yes | admin read-only/fail-closed |
| Phase 16 Platform Ops | work/lease/sched/health/kms/export tests | ops route | no except explanations | maybe | ops/KMS/export refs | yes where visible | health/export denial |
| Phase 17 Artifact Trust | art/doc/export tests | artifact route | yes if drafting | yes | provenance/export refs | yes | artifact card/export denial |
| Phase 18 Persona/Learning/Selene | persona/emo/feedback/learn/memory/write/tts tests | preference and persona route | yes | yes | memory/preference/persona/write/tts refs | yes | personalization allow/deny and serious-mode proof |
| Phase 19 Provider Assist | cost/prefetch/cache/PAE/provider tests | arbitration route | yes | yes | budget/cost/cache refs | no unless visible | provider-off/preload no-network |
| Phase 20 Compute | PH1.COMP tests | compute/report route | no | no | computation/consensus refs | maybe | advisory vs official |
| Phase 21 Video | media/video tests | video route | yes | yes | media/provenance/cost refs | yes | fake video card |
| Phase 22 Client Route | client tests where available | app-open/invite render | n/a | n/a | client provenance refs | yes | Desktop/iPhone route |
| Phase 23 Eval/JD | eval packet tests | real-path smoke | per stack | per stack | evidence verifier refs | yes | acceptance pack |
| Phase 24 Old Retirement | active-caller tests | replacement route regression | per path | per path | old/new route evidence | yes where visible | no active caller |

## 21. Activation Blockers and Unknowns

- `SemanticMeaningProposalPacket` exact repo symbol not found.
- Exact `ProviderPreflightPacket`, `ProviderDecisionTracePacket`, `AccessScopePacket`, `SpeakerIdentityEvidencePacket`, `AuthorityDecisionPacket`, `SimulationGateDecisionPacket`, `WriteOutputPacket`, `ToolProposalPacket`, `PromptInjectionDefensePacket`, and `ArtifactProvenancePacket` names are missing or only equivalent-found.
- Exact approved model for image generation is not named beyond `ImageGenerationProvider`.
- Exact approved model for video generation requires JD approval and current official provider/model confirmation.
- File search and eval/grader exact model/function mapping is unclear: mark `MODEL_POLICY_MISSING` for exact activation until policy is explicit.
- Quick Assist / Conversational Experience is now official in the Expansion Register as stack #15; runtime implementation remains blocked until a Quick Assist Activation Pack defines exact owners, tests, provider-off/fake-provider proof, backend evidence, and JD live scenarios.
- Selene Emotional Intelligence + Relationship Presence is now official in the Expansion Register as stack #16; runtime implementation remains blocked until a Selene Emotional Intelligence + Relationship Presence Activation Pack defines PH1.WRITE, PH1.PERSONA, PH1.EMO/CORE/GUIDE, PH1.FEEDBACK, PH1.LEARN, PH1.M, PH1.TTS, Provider Governance, Desktop/iPhone render-only proof, Adapter transport-only proof, provider-off/fake-provider proof, backend evidence, and JD live scenarios.
- Existing deterministic phrase/follow-up/weather/time paths are active compatibility surfaces; do not extend.
- Any memory logic in Desktop, Adapter, PH1.X, PH1.E, or PH1.WRITE must be classified as wrong-owner or compatibility until PH1.M proves the governed memory packet path.
- Desktop/iPhone route parsing must be proven render-only before client route expansion.
- Adapter `lib.rs` is a large compatibility surface; no cleanup is lawful until active-caller and replacement proof.
- Provider-off and fake-provider proof must be stack-level, not only global.
- JD live path for some future stacks is unclear until client/runtime smoke routes are selected.

Future implementation must stop with `DETERMINISTIC_IMPLEMENTATION_APPROVAL_REQUIRED` if it proposes deterministic user-language behavior outside required gates.

## 22. Recommended Build Instruction Queue

1. Build Provider Governance Baseline With Fake Semantic Provider Shell
2. Build SemanticInterpreterProvider Fake Proposal Path And Malformed Rejection
3. Build Wake / Session / Voice ID / Access Posture Baseline
4. Build PH1.X Semantic Proposal Validation Into HumanConversationDirective
5. Build Conversational Experience + Quick Assist Wording Through PH1.WRITE
6. Build Selene Emotional Intelligence + Relationship Presence Through PH1.WRITE
7. Build PH1.WRITE Display / TTS / Source Presentation Baseline
8. Build Live One-Line Follow-Up Routing To PH1.X / PH1.WRITE
9. Build PH1.M Human Memory Core Activation Pack
10. Build PH1.M Access-Scoped Fresh Memory Continuation Boundary

## 23. Final Activation Pack Status

- This is an activation document: yes
- Runtime code changed: no
- All required architecture docs linked/read, including PH1.M Human Memory Core Master Design: yes
- Every Function Stack Map stack mapped: yes
- Every Expansion Register stack mapped: yes
- Conversational Experience + Quick Assist Stack official in Expansion Register and included: yes
- Selene Emotional Intelligence + Relationship Presence Stack official in Expansion Register and included: yes
- PH1.M Human Memory Core Master Design added and activation-pack-gated: yes
- Global semantic proposal wiring gate included: yes
- Probabilistic human interaction doctrine matrix included: yes
- Deterministic implementation approval gate included: yes
- Every stack mapped: yes
- Phase slice counts estimated: yes
- First implementation build recommended: yes
- Final repo cleanliness: must be proven after commit/push
