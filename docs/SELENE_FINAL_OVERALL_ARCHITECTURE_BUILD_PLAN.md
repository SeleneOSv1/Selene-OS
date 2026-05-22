# Selene Final Overall Architecture Build Plan

## 0. Authority and Read Order

AGENTS.md controls execution for every build derived from this plan. If this plan, any architecture document, or any future build instruction conflicts with AGENTS.md, AGENTS.md wins unless JD explicitly overrides it in-thread.

Read order:

1. [Selene Provider-First OpenAI Assisted Pivot Master Build Plan](SELENE_PROVIDER_FIRST_OPENAI_ASSISTED_PIVOT_MASTER_BUILD_PLAN.md)
2. [Selene Provider-First Function Architecture Cards](SELENE_PROVIDER_FIRST_FUNCTION_ARCHITECTURE_CARDS.md)
3. [Selene Provider-First Vertical Slice Build Pack](SELENE_PROVIDER_FIRST_VERTICAL_SLICE_BUILD_PACK.md)
4. [Selene Global Human Conversation Spine Master Architecture](SELENE_GLOBAL_HUMAN_CONVERSATION_SPINE_MASTER_ARCHITECTURE.md)
5. [Selene Identity + Access + Authority Spine Master Architecture](SELENE_IDENTITY_ACCESS_AUTHORITY_SPINE_MASTER_ARCHITECTURE.md)
6. [Selene Function Stack Architecture — Intent and Enterprise Stack Map](SELENE_FUNCTION_STACK_ARCHITECTURE_INTENT_AND_STACK_MAP.md)
7. [Selene Master Architecture Expansion Register](SELENE_MASTER_ARCHITECTURE_EXPANSION_REGISTER.md)
8. [Selene PH1.M Human Memory Core Master Design](SELENE_PH1M_HUMAN_MEMORY_CORE_MASTER_DESIGN.md)

This plan does not authorize runtime implementation by itself. Runtime implementation still requires build-specific activation packs, approved file scope, clean-tree proof, existing-owner discovery, exact tests, backend evidence, provider-off/fake-provider proof where relevant, and JD live proof where user-visible behavior changes.

Repo-truth references inspected for this plan include [SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md](SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md), [COVERAGE_MATRIX.md](COVERAGE_MATRIX.md), [SELENE_OPENAI_MODEL_ROUTING_POLICY.md](SELENE_OPENAI_MODEL_ROUTING_POLICY.md), the `crates/selene_engines`, `crates/selene_os`, `crates/selene_kernel_contracts`, `crates/selene_adapter`, `apple/mac_desktop`, and `apple/iphone` trees, plus Stage 6-8 reports and web-search plan evidence.

## 1. Executive Build Intent

Selene is not an OpenAI wrapper. Selene is a human interface and runtime system where OpenAI supplies probabilistic intelligence behind Selene-owned enterprise function stacks.

The target system is:

- probabilistic OpenAI intelligence for semantic interpretation, language, media, and provider-assisted reasoning;
- Selene-owned enterprise function stacks for search, memory, files, tools, voice, identity, access, presentation, artifacts, operations, and protected work;
- deterministic identity/access/authority posture;
- protected simulation gates for business mutation and official execution;
- auditable presentation and evidence proof;
- JD live acceptance for visible runtime behavior.

OpenAI may propose, draft, classify, summarize, translate, generate, transcribe, synthesize, or reason. Selene validates, routes, permits, executes only when lawful, writes final user output, renders through clients, and audits.

## 2. Architecture Completeness Gate

This plan accounts for:

| Gate | Status | Evidence |
| --- | --- | --- |
| All seven original architecture docs plus PH1.M Human Memory Core Master Design | INCLUDED | Listed in Section 0 and indexed in `SELENE_MASTER_ARCHITECTURE_BUILD_SET.md`. |
| All Function Stack Map stacks A-U | INCLUDED | Section 4 and Section 13. |
| All 16 Expansion Register stacks | INCLUDED | Section 4 and Section 13. |
| All major repo engine families | INCLUDED | Section 6 and Section 13 map PH1.X, PH1.WRITE, PH1.M, PH1.E, PH1.D, PH1.W/C/L/TTS/Voice ID, access/gov/policy, ops, media, broadcast, onboarding, storage, clients, adapter, and SimulationExecutor. |
| All OpenAI capability surfaces | INCLUDED | Section 5 maps every required surface and marks missing/unclear model policy where repo truth is incomplete. |
| Presentation surfaces | INCLUDED | Section 10 covers text, bullets, tables, reports, email, source chips, image/video/artifact cards, multilingual output, and TTS-safe split. |
| Identity/access/authority surfaces | INCLUDED | Section 7 places Wake/Session/Voice ID/Access near the front, with authority/simulation fail-closed. |
| Old-path retirement areas | INCLUDED | Section 12 and Section 13 include Adapter, PH1.D, PH1.X, Desktop/iPhone, provider, phrase, and compatibility surfaces. |

No stack is marked runtime-ready only because it appears in architecture. Each uncertain area remains `REPO_TRUTH_NEEDED`, `ACTIVATION_PACK_REQUIRED`, `PARTIAL`, or `ARCHITECTURE_ONLY` until a later activation slice proves current files, tests, and evidence.

## 3. Global Runtime Model

Selene has two interlocking spines.

The Global Human Conversation Spine answers: what does the user mean?

The Identity + Access + Authority Spine answers: who may be speaking, what can they access, and what can they do now?

Every user-visible function then routes into a Selene-owned enterprise function stack. The stack owns its admission, scope checks, provider role, validation, evidence, presentation handoff, audit, provider-off behavior, fake-provider behavior, and JD live proof where visible.

Global flow:

1. Input admission and turn/session boundary.
2. Wake/session/identity/access posture where relevant.
3. Provider governance preflight for any provider-assisted step.
4. Semantic proposal where needed.
5. PH1.X deterministic validation and directive.
6. Stack owner execution, refusal, or clarification.
7. PH1.WRITE final output.
8. Adapter transport.
9. Desktop/iPhone render or playback only.
10. Backend evidence and audit.

## 4. Complete Stack Universe

| Stack | Intent | Current repo evidence | Canonical owner family | OpenAI role | Selene validation role | Identity/access/authority implications | Presentation implications | Audit/backend evidence | Provider-off/fake-provider requirement | JD live proof | Priority | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A. Global Human Interface / Semantic Intent | Understand messy user language and route lawfully. | `crates/selene_engines/src/ph1x.rs`, `crates/selene_kernel_contracts/src/ph1x.rs`, Stage 8.5 reports. | PH1.X, Provider Governance, PH1.WRITE. | Schema-bound semantic proposals. | PH1.X validates intent, target, owner, risk. | Must consider current identity/access posture. | Clarification or directive to PH1.WRITE. | Semantic proposal, candidate ledger, directive refs. | Required for semantic provider. | Required for visible conversation. | HIGH | PARTIAL |
| B. Web Search + Source Evidence | Public current/source-backed answers. | `crates/selene_engines/src/ph1e.rs`, `crates/selene_tools/src/ph1e.rs`, `docs/web_search_plan`. | PH1.E, Provider Governance, PH1.WRITE. | Query planning, search/synthesis where approved. | PH1.E accepts sources, verifies claims. | Public-safe unless private/protected mixed. | Source chips, clean answer, TTS without source noise. | Search/source/claim/source-chip packets. | Required. | Required for live search. | HIGH | PARTIAL |
| C. Image-Backed Search + Visual Presentation | Add approved image evidence/cards to search. | `crates/selene_kernel_contracts/src/ph1e.rs`, `docs/web_search_plan/IMAGE_METADATA_PROVIDER_PATH.md`, vision fixtures. | PH1.E, PH1.WRITE, clients render only. | Image metadata/relevance assistance. | Source-page, relevance, safety validation. | Public/private media scope. | Image cards only from approved metadata. | Visual evidence/image display refs. | Required. | Required where visible. | MEDIUM | PARTIAL |
| D. Deep Research | Multi-source high-effort research. | `docs/BLUEPRINTS/TOOL_DEEP_RESEARCH.md`, provider-first docs. | PH1.E, Provider Governance, PH1.WRITE. | Deep research provider only if approved. | Eligibility, budget, source ledger, claim verification. | Public/private/protected split. | Report output, source ledger summary. | Research plan, budget, source ledger. | Required. | Required for visible research. | MEDIUM | ARCHITECTURE_ONLY |
| E. Writing + Transformation | Write, rewrite, summarize, tone, format. | `crates/selene_engines/src/ph1write.rs`, `crates/selene_kernel_contracts/src/ph1write.rs`. | PH1.WRITE with PH1.X target validation. | Draft/rewrite assistance. | Obey directive, target, evidence, style. | Avoid protected claims and private leakage. | display_text/tts_text/formatted_text. | WriteOutput refs and hashes. | Required where provider-assisted. | Required for visible writing. | HIGH | PARTIAL |
| F. Presentation + TTS-Safe Output | Decide display and speech presentation. | `crates/selene_engines/src/ph1write.rs`, `crates/selene_engines/src/ph1tts.rs`, Desktop TTS reports. | PH1.WRITE, PH1.TTS, clients render/play only. | Wording or speech proposal where approved. | Final text, TTS safety, source/card separation. | Identity-safe wording and refusal wording. | Text, source chips, cards, speech split. | Presentation/write/voice output refs. | Required where provider-assisted TTS/writing. | Required where visible/audible. | HIGH | PARTIAL |
| G. Memory + Recall + Preference | Govern Selene's human-like memory lifecycle: notice, encode, consolidate, connect, recall, continue, update, forget/decay. | `docs/SELENE_PH1M_HUMAN_MEMORY_CORE_MASTER_DESIGN.md`, `crates/selene_engines/src/ph1m.rs`, `crates/selene_kernel_contracts/src/ph1m.rs`, Stage 6.5/7 reports. | PH1.M is the single memory authority; PH1.X owns live context; PH1.L owns sleep/wake boundary; PH1.E owns tools/search/files; PH1.WRITE owns memory wording; Desktop/iPhone render only; Adapter transports only. | Salience, embedding, summary, consolidation, topic, or recall proposal where governed. | PH1.M validates scope, trust, freshness, conflict, staleness, privacy, continuation, update, forget/decay, and evidence. | Access scope required for private/company memory; memory never grants authority or bypasses protected execution/simulation. | Human-facing recall language through PH1.WRITE: "I remember", "Earlier today", "Yesterday", not "session search result". | MemoryEvidencePacket/equivalent, recall style, age labels, trust/privacy/conflict refs, access denial refs. | Required where provider-assisted. | Required for memory behavior. | HIGH | MASTER_DESIGN_ADDED / ACTIVATION_PACK_REQUIRED / NO_RUNTIME_IMPLEMENTATION_YET |
| H. File QA + Document | Answer/summarize/transform files. | `crates/selene_engines/src/ph1doc.rs`, `docs/web_search_plan/document_fixtures`. | PH1.E, PH1.WRITE, artifact owner. | File understanding and extraction. | File scope, prompt-injection defense, evidence validation. | File access scope and tenant/private boundaries. | File answer, derived artifact, citations. | File evidence/artifact provenance. | Required. | Required where visible. | MEDIUM | PARTIAL |
| I. Tool / Connector / MCP | Validate and execute read/write tool paths. | `docs/BLUEPRINTS/TOOL_*`, `crates/selene_tools/src/ph1e.rs`, PH1.E contracts. | PH1.E, Access/Authority, SimulationExecutor. | Tool proposal and parameter extraction only. | Permission, parameters, read/write/protected split. | Connector/tool scope and authority. | Tool result summary, refusal. | Tool proposal/execution decision refs. | Required. | Required where visible. | HIGH | PARTIAL |
| J. Protected Action + Simulation | Lawful protected execution only. | `crates/selene_os/src/simulation_executor.rs`, `crates/selene_kernel_contracts/src/runtime_law.rs`, `ph1simfinder`. | PH1.X, Access/Authority, SimulationExecutor, Audit. | Understand/explain protected request only. | Authority, simulation, confirmation, idempotency. | Central protected fail-closed lane. | Protected refusal/success wording through PH1.WRITE. | Authority, simulation, audit, fail-closed refs. | Provider output never grants authority. | Required. | HIGH | PARTIAL |
| K. Identity + Access + Authority | Wake/session/voice/access/authority posture. | `ph1_voice_id`, `ph1access`, `ph1policy`, `ph1gov`, migrations 0008/0009/0015/0016. | Wake/PH1.L/C, Voice ID, Access/Gov/Policy, Authority. | No identity/access/authority grant. | Deterministic identity/access/authority resolution. | Core spine. | Verification/denial wording. | Identity/access/authority refs. | Provider-independent baseline required. | Required for voice/private/protected. | HIGH | PARTIAL |
| L. Voice / Wake / Session / Realtime | Natural voice with session and render/playback boundaries. | `ph1w`, `ph1c`, `ph1l`, `ph1tts`, `ph1listen`, Desktop/iPhone shells. | PH1.W/C/L/TTS/Voice ID; clients capture/play only. | STT, realtime, TTS where governed. | Session admission, transcript provenance, TTS approval. | Voice ID evidence only; access separate. | Transcript, listening state, TTS playback. | Wake/session/transcript/voice output refs. | Required for provider voice. | Required. | HIGH | PARTIAL |
| M. Translation + Language Adaptation | Translate and adapt language without claim drift. | `crates/selene_engines/src/ph1lang.rs`, `ph1write`. | PH1.X, PH1.WRITE, Provider Governance. | Translation/language detection. | Preserve evidence, language policy. | Private/source scope unchanged. | Multilingual display and TTS. | Language request/validation refs. | Required where provider-assisted. | Required where visible. | MEDIUM | PARTIAL |
| N. Summarization + Compression | Summarize prior answers, files, sources, memory. | `crates/selene_engines/src/ph1summary.rs`, PH1.WRITE. | PH1.X, PH1.E/M as needed, PH1.WRITE. | Summarize/compress. | Target/evidence/scope validation. | Memory/file/source scope. | One-line, bullets, executive summary. | Summary target/validation refs. | Required where provider-assisted. | Required where visible. | HIGH | PARTIAL |
| O. Artifact / Document / Slide / Spreadsheet | Durable artifacts and exports. | `crates/selene_kernel_contracts/src/ph1art.rs`, `crates/selene_os/src/device_artifact_sync.rs`, migrations 0006. | Artifact/doc/export owners, PH1.WRITE, PH1.E. | Draft/structure content. | Provenance, source scope, export rules. | Private/protected data scope. | Artifact cards/links. | Artifact provenance/export refs. | Required where provider-assisted. | Required where visible. | MEDIUM | PARTIAL |
| P. Image Generation / Editing | Generate/edit images safely. | Provider-first docs mention `ImageGenerationProvider`; exact runtime mostly architecture. | Provider Governance, PH1.X, media/artifact owner. | Image generation/editing after approval. | Safety, provenance, likeness policy. | Identity/likeness/private media scope. | Generated image card. | Generated media provenance. | Required. | Required. | LOW | ARCHITECTURE_ONLY |
| Q. Video Generation | Generate/transform video where approved. | Provider-first docs mention `VideoGenerationProvider`; no mature runtime found. | Provider Governance, PH1.X, media/artifact owner. | Video generation after JD approval. | Safety, cost, provenance. | Identity/likeness/private media scope. | Video card/output. | Generated video provenance. | Required. | Required. | LOW | ARCHITECTURE_ONLY |
| R. Data Analysis + Report Drafting | Analyze data and draft advisory/official reports. | `ph1comp`, web-search analytics fixtures, tool data analysis blueprint. | PH1.E, PH1.COMP, PH1.WRITE, SimulationExecutor for official. | Reason over data and draft. | Deterministic compute where required, source validation. | Official/protected split. | Tables, charts, reports. | Dataset/calculation/report refs. | Required where provider-assisted. | Required where visible. | MEDIUM | PARTIAL |
| S. Code / Developer Assistance | Repo/code help and Codex instruction drafting. | AGENTS.md, docs templates, builder docs. | PH1.WRITE for advice; Codex obeys AGENTS. | Code reasoning/drafting. | Repo truth, owner, tests, no unauthorized edits. | Protected/repo safety law. | Instructions/explanations. | Test/build proof refs. | Required where provider-assisted. | Usually not JD live. | MEDIUM | PARTIAL |
| T. Cost / Provider Governance / Observability | Provider registry, budget, counters, health, model policy. | `ph1providerctl`, `ph1cost`, `ph1quota`, PH1.D contracts, model policy. | Provider Governance, PH1.D evidence, Storage/Audit. | Use allowed capability only. | Model allowlist, provider-off, fake-provider, cost. | Data-egress/privacy gate. | Provider errors sanitized. | Provider counters/cost/failure refs. | Mandatory foundation. | Required for provider-visible behavior. | HIGH | PARTIAL |
| U. Evaluation / Regression / JD Live Acceptance | Prove behavior in tests and live routes. | `docs/web_search_plan/eval`, release evidence, Stage reports. | Eval harness, Storage/Audit, JD live acceptance. | Eval/grader assistance only when governed. | Backend evidence and real route proof. | Protected/user-visible gates. | Acceptance reports. | Eval/backend/JD refs. | Required for provider/eval provider. | Required where visible. | HIGH | PARTIAL |
| Expansion 1. Broadcast / Delivery / Reminder / Messaging | Deliver messages, reminders, announcements. | `ph1bcast`, `ph1delivery`, `ph1rem`, blueprints. | PH1.BCAST, PH1.DELIVERY, PH1.REM, PH1.WRITE. | Draft message text only. | Delivery permission, timing, recipient validation. | User/tenant/authority for delivery. | Notifications/reminders/message UI. | Delivery/audit refs. | Required where provider-assisted. | Required where visible. | MEDIUM | PARTIAL |
| Expansion 2. Onboarding / Invite / Link / Enrollment | Invite, setup, enrollment, app-open flow. | `ph1onb`, `ph1link`, iPhone invite parsing, migrations 0012. | PH1.ONB, PH1.LINK, Access, clients render only. | Explain/draft onboarding text. | Token, invite, enrollment, access validation. | Identity/access bootstrap. | Invite rendering/app-open status. | Link/onboarding/access refs. | Provider not required for baseline. | Required for visible onboarding. | HIGH | PARTIAL |
| Expansion 3. Master Access Template / Role / Permission / Admin Controls | Roles, templates, admin permissions. | `ph1access`, `ph1policy`, `ph1gov`, migrations 0015/0016, docs 29/30. | Access, Policy, Governance. | Explain/admin drafting only. | Deterministic role/template/permission checks. | Core authority/access. | Admin UI summaries. | Access policy/audit refs. | Provider not required. | Required for admin UI. | HIGH | PARTIAL |
| Expansion 4. Tenant / Workspace / Governance / Quota | Tenant/workspace governance and quotas. | `ph1tenant`, `ph1quota`, `ph1gov`, `ph1policy`. | Tenant, Governance, Quota. | Summarize/explain only. | Tenant/workspace scope and quota gates. | Tenant isolation. | Scope/quota messages. | Tenant/quota/gov refs. | Provider not required. | Required where visible. | HIGH | PARTIAL |
| Expansion 5. Work / Lease / Scheduling / Health / KMS / Export Ops | Platform operations. | `ph1work`, `ph1lease`, `ph1sched`, `ph1health`, `ph1kms`, `ph1export`. | Platform ops owners. | Draft ops summaries only. | Deterministic health, leases, scheduling, KMS/export gates. | KMS/export may be protected/private. | Ops dashboards/status. | Work/lease/sched/health/export refs. | Provider not required for baseline. | Required where visible. | MEDIUM | PARTIAL |
| Expansion 6. Visual Recognition / OCR / Media Ingestion / Multimodal Evidence | Understand images/media and OCR. | `ph1vision`, `ph1vision_media`, `ph1os` OCR, vision fixtures. | PH1.VISION, PH1.E, Provider Governance. | Vision/OCR/image understanding where approved. | Media scope, OCR evidence, safety. | Private media/access scope. | Visual evidence summary. | Vision/media evidence refs. | Required. | Required where visible. | MEDIUM | PARTIAL |
| Expansion 7. Visual Rendering / Image Cards / Media Presentation | Render approved media. | PH1.E image cards, Desktop/iPhone render shells. | PH1.WRITE, PH1.E, clients render only. | Optional visual caption help. | Approved-card-only presentation. | Media privacy scope. | Image/media cards. | Visual presentation refs. | Required if provider-assisted. | Required. | MEDIUM | PARTIAL |
| Expansion 8. Video Recognition / Rendering / Generation | Video understanding/render/generation. | Provider-first video docs; repo mature support unclear. | PH1.VISION/media, Provider Governance, artifact owner. | Video recognition/generation after approval. | Safety, provenance, cost. | Identity/private media/protected risk. | Video cards. | Video evidence/provenance refs. | Required. | Required. | LOW | REPO_TRUTH_NEEDED |
| Expansion 9. Artifact Trust / Document / Export / Provenance | Trust chain for artifacts/docs/exports. | `ph1art`, `ph1doc`, `ph1export`, storage migrations. | Artifact/doc/export, PH1.E, PH1.WRITE. | Draft/transform only. | Provenance, export permission, source trust. | Private/protected data export gate. | Artifact/export cards. | Artifact trust/export refs. | Required if provider-assisted. | Required where visible. | HIGH | PARTIAL |
| Expansion 10. Persona / Preference / Emotion / Feedback / Learning | Personalization and learning. | `ph1persona`, `ph1emocore`, `ph1emoguide`, `ph1feedback`, `ph1learn`. | PH1.PERSONA, PH1.EMO, PH1.FEEDBACK, PH1.LEARN, PH1.M. | Preference/emotion suggestion. | Memory law, privacy, feedback validation. | Speaker identity/access required for durable personalization. | Tone/presentation preferences. | Preference/feedback/learning refs. | Required where provider-assisted. | Required where visible. | MEDIUM | PARTIAL |
| Expansion 11. Provider Assist / Cost / Prefetch / Arbitration | Provider assist, cache/prefetch, arbitration. | `ph1pae`, `ph1cost`, `ph1prefetch`, `ph1cache`, `ph1providerctl`. | Provider Governance, Cost, Cache/Prefetch. | Provider capability only through governance. | Budget, no hidden provider calls, arbitration evidence. | Data-egress/privacy. | Sanitized degraded output. | Cost/cache/prefetch/provider refs. | Mandatory. | Required for visible provider behavior. | HIGH | PARTIAL |
| Expansion 12. Deterministic Compute / Consensus / Calculation Authority | Calculation and consensus authority. | `ph1comp`, computation fixtures, PH1.COST/QUOTA. | PH1.COMP, PH1.E, SimulationExecutor where official. | Explain/draft, not calculation authority unless validated. | Deterministic calculation/consensus validation. | Official/protected calculations gated. | Tables/reports. | Calculation/consensus refs. | Provider-off deterministic path required. | Required where visible. | MEDIUM | PARTIAL |
| Expansion 13. Client Route Presentation / App Open / Invite Rendering | App open, route display, invite rendering. | iPhone `SessionShellView.swift`, Desktop runtime bridge, `ph1link`. | PH1.LINK/ONB/runtime, clients render only. | None for routing authority. | Server/runtime route validation. | Invite/access bootstrap. | App-open and invite UI. | Client provenance refs. | Provider not required. | Required. | HIGH | PARTIAL |
| Expansion 14. Old Compatibility Path Retirement Register | Controlled cleanup after proof. | Adapter `lib.rs`, Stage 8.5 reports, Desktop/iPhone route parsing. | Correct canonical owner per old path. | None. | Replacement proof, active-caller checks. | Prevent wrong-owner authority. | No user-visible regression. | Retirement evidence. | Provider-off/fake as relevant. | Required where visible. | HIGH | ACTIVATION_PACK_REQUIRED |
| Expansion 15. Conversational Experience + Quick Assist | Natural user guidance, reassurance, clarification, wake acknowledgements, process help, result explanation, weather/time presentation, and friendly TTS-safe phrasing. | Expansion Register stack 15; activation pack maps PH1.WRITE, PH1.X, PH1.W/C/L, PH1.TTS, PH1.E, PH1.M, Desktop/iPhone, and Adapter surfaces. | PH1.WRITE final wording; PH1.X validated intent/state; PH1.W/C/L state; PH1.E facts/tools; PH1.M lawful preferences; clients render/play only. | GPT-5.5 may propose wording, clarification, comfort, options, next-step guidance, formatting, and TTS-safe phrasing. | Selene validates state, scope, evidence, risk, provider governance, and final output. | No access, authority, memory permission, tool permission, protected execution, or mutation may come from GPT-5.5. | Natural display_text/tts_text through PH1.WRITE, never Desktop/Adapter brain. | Provider wording proposal, deterministic state/fact owner refs, PH1.WRITE validation, client provenance. | Required for provider-assisted wording; first slice must be provider-off/fake-provider safe. | Required where visible/audible. | HIGH | ACTIVATION_PACK_REQUIRED |
| Expansion 16. Selene Emotional Intelligence + Relationship Presence | Official Selene emotional presentation layer for warm, witty, emotionally adaptive, serious-when-needed communication. | Expansion Register stack 16; activation pack must map PH1.WRITE, PH1.PERSONA, PH1.EMO/GUIDE/CORE, PH1.FEEDBACK, PH1.LEARN, PH1.M, PH1.TTS, Provider Governance, clients, and Adapter. | PH1.WRITE final persona wording; PH1.EMO/PERSONA assist surfaces; PH1.M lawful preferences; PH1.X risk/state validation; PH1.TTS approved speech; clients render/play only. | GPT-5.5 may propose persona wording, humor, emotional phrasing, comfort, wake greetings, tone adaptation, and multilingual TTS-safe phrasing. | Selene validates persona policy, seriousness level, safety boundaries, identity/access/privacy limits, memory permission, protected-risk context, final display_text, and final tts_text. | No persona output may grant access, authority, memory permission, tool permission, protected execution, or mutation. | Selene tone appears only through PH1.WRITE/PH1.TTS approval, never Desktop/Adapter brain. | Persona proposal refs where used, emotional/persona/preference refs, PH1.WRITE validation, TTS/client provenance. | Required for provider-assisted persona wording; first slice must be provider-off/fake-provider safe. | Required where visible/audible. | HIGH | ACTIVATION_PACK_REQUIRED |

### PH1.M Human Memory Core Master Design Status

The PH1.M Human Memory Core Master Design is now an official source document for this plan.

PH1.M must be Selene's single governed memory authority. Desktop, Adapter, PH1.X, PH1.E, PH1.WRITE, and clients may request or consume memory evidence, but they must not build their own memory systems.

PH1.M memory is no longer treated as a small recall scope. Future PH1.M planning must include the full lifecycle module set:

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

Required PH1.M build sequence:

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

Status:

- MASTER_DESIGN_ADDED
- ACTIVATION_PACK_REQUIRED
- NO_RUNTIME_IMPLEMENTATION_YET

PH1.M must not be implemented as session search. It must be Selene's governed human memory brain.

## 5. OpenAI Capability to Selene Stack Mapping

| OpenAI Capability | Selene Stack | OpenAI May Do | Selene Must Own | Provider-Off Behavior | Proof Required |
| --- | --- | --- | --- | --- | --- |
| Semantic interpretation | A, GHCS, PH1.X | Return schema-bound meaning proposals. | Proposal validation, owner routing, target selection, protected risk. | Zero attempts; deterministic fallback/clarification where lawful. | Provider counters, malformed rejection, PH1.X directive evidence. |
| Structured outputs | A, T | Produce proposal/result JSON matching schema. | Schema/version validation and rejection of malformed output. | Zero attempts; no hidden schema probe. | Fake valid/malformed provider tests. |
| Tool/function proposals | I, PH1.E | Propose tool name/parameters. | Permission, scope, parameter validation, execution decision. | No provider attempt; no tool execution. | Tool proposal rejected/accepted evidence. |
| Web search | B, PH1.E | Search/query/summarize where permitted. | Search need, provider route, source acceptance, claim verification. | No network/provider dispatch; safe degrade. | Source ledger, claim verification, source chips. |
| File search | H, PH1.E | Assist retrieval/QA where approved. | File scope, injection defense, evidence acceptance. | No provider attempt; no file egress. | File evidence and prompt-injection tests. |
| Deep research | D, PH1.E | High-effort research where explicitly approved. | Eligibility, budget, source ledger, contradiction handling. | No provider attempt; explain unavailable/deferred. | Budget/source/claim ledger. |
| Embeddings | G, PH1.M, T | Generate embeddings where approved. | Memory permission, retrieval scope, vector ownership. | No embedding call; deterministic/no-recall degrade. | Embedding model/cost/privacy evidence. |
| STT | L | Transcribe audio. | Transcript admission, session binding, identity separation. | No STT provider call; local/no transcript degrade. | STT provider evidence and transcript provenance. |
| TTS | F, L | Synthesize approved text. | PH1.WRITE/PH1.TTS approve spoken text first. | No TTS call; text remains displayed. | TTS request/voice/model/audio digest evidence. |
| Realtime voice | L | Realtime session/transcription where approved. | Wake/session/turn admission and output approval. | No realtime session; no hidden probes. | Realtime session counters/provenance. |
| Image understanding | C, H, Expansion 6 | Describe/OCR/analyze images. | Media scope, source/evidence validation, safety. | No image provider call; safe unsupported/degraded answer. | Vision/OCR evidence and privacy proof. |
| Image generation | P, Expansion 7 | Generate/edit images after JD approval. | Request validation, safety, provenance, presentation. | No provider call; no fabricated media. | Generated media provenance and safety tests. |
| Video generation | Q, Expansion 8 | Generate video after JD approval. | Request validation, cost, safety, provenance. | No provider call. | Video provenance/cost/safety proof. |
| Writing | E, F, PH1.WRITE | Draft/rewrite text. | Final output, evidence limits, tone/format policy. | Deterministic PH1.WRITE or clarification. | WriteOutput refs and leak checks. |
| Conversational guidance / Quick Assist | Expansion 15, E, F, L | Propose natural guidance, reassurance, clarification, next-step explanations, wake acknowledgements, and TTS-safe phrasing. | State/fact/risk validation, PH1.X where needed, PH1.WRITE final text, no authority or mutation. | No provider attempt; safe deterministic state display or clarification without hardcoded language becoming production law. | Provider-off/fake-provider wording proof and PH1.WRITE output evidence. |
| Persona and emotional presentation | Expansion 16, Expansion 10, E, F, L | Propose Selene emotional presentation wording, humor, emotional phrasing, comfort, tone adaptation, wake greeting personality, and multilingual TTS-safe phrasing. | Persona policy, emotional appropriateness, memory/privacy boundaries, PH1.WRITE final text, PH1.TTS approval, no access/authority/mutation. | No provider attempt; safe neutral PH1.WRITE wording or clarification without hardcoded personality phrase lists becoming production law. | Provider-off/fake-provider persona proof, PH1.WRITE/TTS evidence, PH1.EMO/PH1.PERSONA/PH1.M scope evidence where used. |
| Summarization | N, E | Summarize/compress. | Target validation, scope, evidence preservation. | Deterministic summary or clarification where lawful. | Summary target/evidence proof. |
| Translation | M | Translate/adapt language. | Language policy, fact preservation, TTS compatibility. | Deterministic/unsupported degrade. | Translation validation evidence. |
| Code assistance | S | Explain/draft/suggest code/instructions. | AGENTS law, repo truth, approved file scope, tests. | No provider attempt if disabled. | Build instruction/test proof. |
| Evals/graders | U | Assist graders/eval generation where approved. | Acceptance criteria, backend evidence, JD status. | No provider grader call; deterministic checks only. | Eval case and acceptance evidence. |
| Provider governance/cost | T, Expansion 11 | Nothing without governance permission. | Model policy, budget, counters, circuit breakers, privacy. | Default proof target: zero attempts and zero dispatches. | Provider preflight/cost/counter evidence. |

### Model-to-Function Map

Codex inspected repo model policy, provider routes, model evidence contracts, env/config references, and current provider surfaces before assigning any model. The source of truth is [SELENE_OPENAI_MODEL_ROUTING_POLICY.md](SELENE_OPENAI_MODEL_ROUTING_POLICY.md); code evidence includes PH1.D model evidence in `crates/selene_kernel_contracts/src/ph1d.rs`, live Responses route/env wiring in `crates/selene_adapter/src/lib.rs`, voice HTTP adapter defaults in `crates/selene_adapter/src/bin/http_adapter.rs`, Desktop STT request defaults in `apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`, OCR config in `crates/selene_os/src/ph1os.rs`, and vision model fields in `crates/selene_engines/src/ph1vision.rs` and `crates/selene_engines/src/ph1vision_media.rs`.

| Function | Repo evidence/path | Approved model/provider if found | Status | MODEL_POLICY_MISSING |
| --- | --- | --- | --- | --- |
| Semantic interpretation | `SELENE_OPENAI_MODEL_ROUTING_POLICY.md`, `crates/selene_adapter/src/lib.rs` | `gpt-5.5` | PARTIAL live PH1.D route exists | no |
| Writing | `SELENE_OPENAI_MODEL_ROUTING_POLICY.md`, PH1.WRITE contracts | `gpt-5.5` | PARTIAL | no |
| Web search synthesis | `SELENE_OPENAI_MODEL_ROUTING_POLICY.md`, `docs/web_search_plan` | `gpt-5.5 with web search tool where permitted` | PARTIAL | no |
| File search | File/document stacks and PH1.E/file docs | Exact File Search provider/model not pinned in model policy | UNCLEAR | yes for exact file-search provider/model |
| Deep research | `SELENE_OPENAI_MODEL_ROUTING_POLICY.md` | `o3-deep-research` if available and approved; fallback only with explicit JD approval | ARCHITECTURE_ONLY/PARTIAL | no for primary, approval required |
| Embeddings | `SELENE_OPENAI_MODEL_ROUTING_POLICY.md` | `text-embedding-3-large` | PARTIAL | no |
| STT | `SELENE_OPENAI_MODEL_ROUTING_POLICY.md`, `http_adapter.rs`, Desktop bridge | `gpt-4o-transcribe`; diarization `gpt-4o-transcribe-diarize` | PARTIAL/ACTIVE in adapter routes | no |
| TTS | `SELENE_OPENAI_MODEL_ROUTING_POLICY.md`, `http_adapter.rs`, Desktop TTS evidence | `gpt-4o-mini-tts` | PARTIAL/ACTIVE behind flags | no |
| Realtime voice | `SELENE_OPENAI_MODEL_ROUTING_POLICY.md` | `gpt-realtime-2`; transcription `gpt-realtime-whisper` | PARTIAL | no |
| Image understanding | `SELENE_OPENAI_MODEL_ROUTING_POLICY.md`, PH1.VISION model fields | `gpt-5.5` | PARTIAL | no |
| Image generation | Provider-first docs and model policy | Current official OpenAI image generation model only after JD approval | ARCHITECTURE_ONLY | yes for exact model ID |
| Video generation | Provider-first docs and model policy | Sora/current official OpenAI video generation model only after JD approval | ARCHITECTURE_ONLY | yes for exact model ID |
| Translation | `SELENE_OPENAI_MODEL_ROUTING_POLICY.md`, PH1.LANG | Text translation via `gpt-5.5`; live translation `gpt-realtime-translate` deferred until JD approval | PARTIAL | no for text, deferred for live |
| Code assistance | `SELENE_OPENAI_MODEL_ROUTING_POLICY.md` | `gpt-5.5 with code interpreter tool where explicitly approved` | ARCHITECTURE_ONLY for tool use | no, approval required |
| Evals/graders | Eval docs and provider-first plan | Exact eval/grader model not pinned | UNCLEAR | yes |
| Moderation/safety support | `SELENE_OPENAI_MODEL_ROUTING_POLICY.md` | `omni-moderation-latest` | ARCHITECTURE_ONLY/PARTIAL | no |

No future build may substitute, downgrade, fallback, or optimize model choice unless JD explicitly updates the model policy.

## 6. Canonical Owner Map

| Owner | May own | Must not own |
| --- | --- | --- |
| PH1.X | Current-turn semantic validation, target/reference validation, HumanConversationDirective, protected-risk classification. | Final writing, memory truth, search/source acceptance, protected execution, provider/model choice. |
| PH1.WRITE | Final display/response/TTS-safe text, source/card presentation wording, refusal/clarification wording. | Target selection, provider authority, memory permission, protected execution. |
| PH1.M | Memory recall/write/update/forget, memory evidence, preference memory under scope. | Current-turn semantic routing, access authority, Desktop memory shortcuts. |
| PH1.E | Search, tools, files, source acceptance, claim verification, source/image evidence. | Memory truth, PH1.X intent ownership, final presentation, protected execution. |
| Provider Governance / PH1.D | Provider preflight, request/response plumbing, model/cost/privacy evidence, provider-off/fake behavior, normalized provider output safety. | Meaning ownership, final answer ownership, protected authority, Desktop/Adapter behavior. |
| Voice/Wake/Session/TTS/STT | Wake activation, transcript admission, session binding, approved speech synthesis/playback posture. | Identity proof by wake alone, authority, semantic routing, final content. |
| Voice ID | Speaker evidence and liveness/anti-replay evidence. | Access grant, private memory grant, authority, protected execution. |
| Access/Governance/Authority | User/tenant/role scope, authority decision, admin/template policy. | Simulation execution, semantic meaning, final output. |
| SimulationExecutor | Approved protected simulations/process execution and fail-closed evidence. | Provider interpretation, access grant, generic public answer writing. |
| Broadcast/Delivery/Reminder | Message/reminder/announcement delivery under scope. | Semantic intent, recipient authority bypass, protected mutation. |
| Onboarding/Invite/Link | Invite/link/enrollment/app-open server truth. | Client-side authority, semantic routing beyond onboarding lane. |
| Tenant/Governance/Quota/Ops | Tenant/workspace, quota, platform operations, health, KMS, export gates. | User-facing writing without PH1.WRITE, provider bypass. |
| Visual/Media | OCR, vision evidence, media ingestion/provenance. | Final answer, access bypass, fabricated source truth. |
| Artifact/Export | Artifact provenance, document/export ownership. | Evidence acceptance outside PH1.E, protected export bypass. |
| Persona/Learning | Persona/preferences/emotion/feedback/learning under memory and scope law. | Identity proof, protected authority, hidden manipulation. |
| Provider Assist/Cost/Prefetch | Provider assist, budget, cost, cache/prefetch, arbitration evidence. | Hidden provider calls, model downgrade, semantic authority. |
| Deterministic Compute | Calculations, consensus, quantitative authority where deterministic proof is required. | Provider-only math authority for official decisions. |
| Conversational Experience + Quick Assist | Provider-assisted natural communication only after deterministic state/fact owners provide context and PH1.WRITE approves output. | Access, authority, memory permission, tool permission, protected execution, state mutation, Desktop/Adapter conversational brain. |
| Selene Emotional Intelligence + Relationship Presence | Persona and emotional presentation through PH1.WRITE with PH1.EMO/PH1.PERSONA/PH1.M scoped assist surfaces and Provider Governance. | Access, authority, state mutation, protected execution, unsupported factual claims, hidden manipulation, Desktop/Adapter persona brain. |
| Desktop/iPhone | Capture, render, playback, app-open display, approved client UI. | Meaning, memory, access, authority, provider calls, protected execution. |
| Adapter | Transport, provenance, health bridge, evidence carriage. | Semantic decisions, target selection, rewrite, access, authority, memory/tool owner behavior. |

## 7. Final Build Sequence

Adjustment from the minimum sequence: Wake/Session/Voice ID/Access is moved immediately after Provider Governance and repo activation. Repo truth shows real voice, wake, session, access, and Voice ID surfaces already exist, and the product spine depends on knowing whether to listen, who may be speaking, and what scope is allowed. Provider Governance still comes first because no provider-assisted semantic, voice, search, or media behavior may call OpenAI without model/cost/privacy/off/fake proof.

PHASE 0 - Architecture Docs and Index Complete
PHASE 1 - Repo-Truth Activation Pack
PHASE 2 - Provider Governance Baseline
PHASE 3 - Wake / Session / Voice ID / Access Posture Baseline
PHASE 4 - Semantic Meaning Proposal Baseline
PHASE 5 - PH1.X Deterministic Validation Against Identity/Access
PHASE 6 - PH1.WRITE Presentation + Quick Assist + Selene Emotional Intelligence Baseline
PHASE 7 - Web Search + Source Evidence + Source Chips
PHASE 8 - PH1.M Human Memory Core Lifecycle + Preference Boundary
PHASE 9 - Tool/File/Connector Scope
PHASE 10 - Voice/Wake/Session/Realtime + Quick Assist/Selene Route Proof
PHASE 11 - Visual Recognition + OCR + Media Evidence
PHASE 12 - Visual Rendering + Image Cards + Media Presentation
PHASE 13 - Broadcast/Delivery/Reminder/Messaging
PHASE 14 - Onboarding/Invite/Link/Enrollment
PHASE 15 - Master Access Template/Admin/Tenant/Governance
PHASE 16 - Platform Ops: Work/Lease/Scheduling/Health/KMS/Export
PHASE 17 - Artifact Trust/Document/Export/Provenance
PHASE 18 - Persona/Preference/Emotion/Feedback/Learning + Selene
PHASE 19 - Provider Assist/Cost/Prefetch/Arbitration
PHASE 20 - Deterministic Compute/Consensus/Calculation Authority
PHASE 21 - Video Recognition/Rendering/Generation
PHASE 22 - Client Route Presentation/App Open/Invite Rendering
PHASE 23 - Evaluation/JD Live Acceptance System
PHASE 24 - Old Compatibility Path Retirement

## 8. Phase Detail Template

| Phase | Purpose | Stacks covered | Canonical owners | Required repo-truth activation | Expected files/areas to inspect later | OpenAI role | Selene validation role | Provider-off/fake proof | Backend evidence | JD live proof | Old paths | Stop conditions | Acceptance target |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | Confirm docs/index complete. | All architecture docs. | Docs only. | None beyond clean tree. | `docs/SELENE_MASTER_ARCHITECTURE_BUILD_SET.md`. | None. | Docs law only. | Not applicable. | Commit proof. | No. | No deletion. | Missing docs. | CODEX_TESTED |
| 1 | Produce repo-truth activation packs. | All stacks. | Current repo owners. | Full owner/path/test/evidence map. | `crates`, `apple`, `docs`, migrations, reports. | None unless docs-only analysis. | Identify owners and gaps. | Plan only. | Activation report. | No. | Inventory only. | Repo truth incomplete. | CODEX_TESTED |
| 2 | Provider governance baseline. | T, Expansion 11, Expansion 15/16 dependencies. | PH1.D, PH1.PROVIDERCTL, PH1.COST, PH1.QUOTA. | Provider Governance Activation Pack. | `ph1d`, `ph1providerctl`, model policy, adapter route. | Governed/fake only, including future Quick Assist and Selene emotional presentation wording providers. | Model, budget, privacy, counters. | Mandatory. | Provider preflight/counter refs. | If visible. | No provider bypass deletion. | Missing off/fake proof. | CODEX_TESTED |
| 3 | Early human-interface permission spine. | K, L, Expansion 3/4/13/15/16. | Wake, PH1.C/L, Voice ID, Access/Gov/Policy. | VIA Activation Pack plus Quick Assist and Selene Emotional Intelligence + Relationship Presence activation packs before wording implementation. | `ph1w`, `ph1c`, `ph1l`, `ph1_voice_id`, `ph1access`, clients. | Natural wake/session/persona wording proposal only after state owner facts. | Evidence-only identity, access scope, defaults false; Quick Assist and Selene cannot grant scope. | Provider-independent for gates; provider-off/fake required for wording. | Wake/session/identity/access refs plus PH1.WRITE/PH1.TTS refs where wording appears. | Yes for voice/app paths. | No client authority cleanup yet. | Voice ID grants authority or hardcoded wake/persona language expands. | JD_LIVE where visible |
| 4 | Semantic proposal baseline. | A, T, Expansion 15/16. | Provider Governance, SemanticInterpreterProvider, PH1.X. | GHCS 0A/0C + model map + Quick Assist and Selene Emotional Intelligence + Relationship Presence activation packs. | PH1.D/Adapter/PH1.X contracts. | Schema-bound proposal, including Quick Assist and Selene emotional presentation/wording proposals. | Reject malformed/authority-granting output; PH1.X validates user/process state before Quick Assist or Selene emotional presentation output. | Mandatory. | Proposal/provider refs. | If visible. | No phrase path deletion. | Provider bypass. | CODEX_TESTED |
| 5 | PH1.X deterministic validation. | A, J, K. | PH1.X. | PH1.X activation pack. | `ph1x`, active frame, previous answer, risk ledgers. | Proposal only. | Directive, target, owner, access/risk validation. | Fake semantic provider. | HumanConversationDirective refs. | Required for conversation routes. | Old PH1.X compat retained. | Target/wrong-owner drift. | JD_LIVE where visible |
| 6 | PH1.WRITE presentation, Quick Assist, and Selene emotional presentation baseline. | E, F, N, M, Expansion 15/16. | PH1.WRITE, PH1.TTS, PH1.X where state/intent is needed, PH1.EMO, PH1.PERSONA, PH1.M. | PH1.WRITE activation pack + Quick Assist Activation Pack + Selene Emotional Intelligence + Relationship Presence Activation Pack. | `ph1write`, `ph1tts`, output contracts, wake/session/tool presentation surfaces, `ph1persona`, `ph1emocore`, `ph1emoguide`, `ph1m`. | Draft natural user guidance, clarification, formatting, emotional phrasing, persona tone, humor, and TTS-safe wording. | Final output/persona validation; no access/authority/memory/tool permission/protected execution/mutation. | Required where provider-assisted. | WriteOutput/TTS/persona/emotion refs plus deterministic state/fact owner refs. | Required. | No adapter rewrite deletion yet. | Raw provider output leak, deterministic language patch, or persona bypass. | JD_LIVE where visible |
| 7 | Web search and source chips. | B, C. | PH1.E, PH1.WRITE. | Web Search Source Evidence pack. | `ph1e`, `web_search_plan`, tools. | Search/synthesis. | Source acceptance, claim verification. | Mandatory. | Source/chip/claim refs. | Required. | No old search path deletion. | Source acceptance missing. | JD_LIVE |
| 8 | PH1.M human memory lifecycle and preferences: repo-truth audit, contracts, recall orchestrator, encoding/salience/consolidation, fresh/day/topic/deep/permanent memory, continuation gate, posture, freshness, trust, privacy, conflict/staleness, no-record handling, natural PH1.WRITE language, UI, and eval matrix. | G, Expansion 10. | PH1.M as single memory authority; PH1.X live context; PH1.L sleep/wake boundary; PH1.E tools/search/files; PH1.WRITE memory wording; Desktop/iPhone render only; Adapter transport only. | PH1.M Human Memory Core Activation Pack. | `ph1m`, PH1.X memory interactions, PH1.L sleep/wake, Adapter recall routes, storage digest/ledger/archive/audit surfaces, Desktop memory UI if any, `ph1persona`, `ph1learn`. | Salience, embedding, summary, consolidation, and topic proposal where governed. | Memory permission, scope, trust, freshness, conflict, staleness, privacy, continuation, update/forget, no session-search UX. | Required where provider-assisted. | MemoryEvidencePacket/equivalent, recall style, age label, trust/privacy/conflict/staleness refs, provenance/audit refs. | Required. | Adapter/Desktop/PH1.X/PH1.E/PH1.WRITE memory shortcuts retained until proof. | Memory implemented as session search, duplicate memory owner, unknown speaker private memory, stale context pollution. | JD_LIVE |
| 9 | Tool/file/connector scope. | H, I. | PH1.E, Access/Authority, SimulationExecutor. | Tool/File Connector Activation Pack. | `ph1e`, `ph1doc`, blueprints, tools. | Proposal/extraction. | Tool/file permission and prompt-injection defense. | Mandatory. | Tool/file decision refs. | Required. | No tool shortcut deletion. | Provider executes tool. | JD_LIVE |
| 10 | Voice/realtime, Quick Assist, and Selene route proof. | L, F, K, Expansion 15/16. | Wake/C/L/TTS/Voice ID, PH1.WRITE, PH1.EMO/PERSONA, Adapter transport, clients render. | Voice Runtime Activation Pack + Quick Assist Activation Pack + Selene Emotional Intelligence + Relationship Presence Activation Pack. | Desktop/iPhone, adapter bins, voice contracts, PH1.WRITE/TTS/persona output contracts. | STT/realtime/TTS plus natural wake/listening/session wording and Selene tone where governed. | Transcript admission, state validation, persona policy, TTS approval, no client brain. | Mandatory. | Transcript/voice/session/write/persona/client refs. | Required. | No client path deletion. | Desktop/Adapter meaning, local conversational brain, or local persona brain. | JD_LIVE |
| 11 | Visual recognition/media evidence. | Expansion 6, H, C. | PH1.VISION, PH1.E. | Visual/Media Activation Pack. | `ph1vision`, `ph1vision_media`, OCR routes. | Vision/OCR. | Media scope, visual evidence validation. | Mandatory. | Vision/OCR refs. | Required. | No visual old path deletion. | Private media leak. | JD_LIVE where visible |
| 12 | Visual rendering/media presentation. | Expansion 7, C, F. | PH1.WRITE, PH1.E, clients render. | Visual Presentation Pack. | PH1.E image cards, client render state. | Optional captions. | Approved cards only. | Required if provider-assisted. | Visual presentation refs. | Required. | No client rendering cleanup. | Raw URL/image leaks. | JD_LIVE |
| 13 | Broadcast/delivery/reminder/messaging. | Expansion 1. | PH1.BCAST, PH1.DELIVERY, PH1.REM. | Broadcast/Delivery Activation Pack. | `ph1bcast`, `ph1delivery`, `ph1rem`, blueprints. | Draft text only. | Recipient/timing/delivery validation. | Required if provider-assisted. | Delivery/audit refs. | Required. | Retain old delivery until proof. | Unauthorized delivery. | JD_LIVE |
| 14 | Onboarding/invite/link/enrollment. | Expansion 2, 13. | PH1.ONB, PH1.LINK, Access, clients render. | Onboarding/Access Activation Pack. | `ph1onb`, `ph1link`, iPhone routes. | Explain/draft only. | Invite/token/enrollment validation. | Provider not baseline. | Invite/onboarding refs. | Required. | No client invite parsing cleanup. | Client authority. | JD_LIVE |
| 15 | Access templates/admin/tenant/governance. | Expansion 3/4. | Access, Policy, Gov, Tenant, Quota. | Access/Tenant Activation Pack. | `ph1access`, `ph1policy`, `ph1gov`, `ph1tenant`, migrations. | Explain/draft only. | Role/template/tenant/quota validation. | Provider not baseline. | Access/gov/quota refs. | Required for admin UI. | No old access cleanup. | Access scope unclear. | JD_LIVE where visible |
| 16 | Platform ops. | Expansion 5. | Work/Lease/Sched/Health/KMS/Export. | Platform Ops Activation Pack. | `ph1work`, `ph1lease`, `ph1sched`, `ph1health`, `ph1kms`, `ph1export`. | Summaries only. | Deterministic ops/KMS/export gates. | Provider not baseline. | Ops evidence refs. | Required where visible. | No ops cleanup. | KMS/export authority drift. | CODEX/JD as visible |
| 17 | Artifact trust/document/export/provenance. | O, Expansion 9. | Artifact/doc/export, PH1.E, PH1.WRITE. | Artifact Trust Activation Pack. | `ph1art`, `ph1doc`, `ph1export`, storage. | Draft/transform. | Provenance/source/export validation. | Required if provider-assisted. | Artifact/export refs. | Required. | No artifact old cleanup. | Provenance missing. | JD_LIVE |
| 18 | Persona/preference/emotion/feedback/learning and Selene. | Expansion 10/16. | PH1.PERSONA, EMO, FEEDBACK, LEARN, PH1.M, PH1.WRITE, PH1.TTS. | Persona/Learning Activation Pack + Selene Emotional Intelligence + Relationship Presence Activation Pack. | `ph1persona`, `ph1emocore`, `ph1emoguide`, `ph1feedback`, `ph1learn`, `ph1m`, `ph1write`, `ph1tts`. | Suggest/adapt persona and emotional presentation only. | Memory/privacy/learning/persona validation; Selene cannot grant access, authority, or protected execution. | Required where provider-assisted. | Preference/emotion/persona/learning/write refs. | Required where visible. | No persona shortcut cleanup. | Hidden durable preference or persona safety bypass. | JD_LIVE |
| 19 | Provider assist/cost/prefetch/arbitration. | Expansion 11, T. | PAE, COST, PREFETCH, CACHE, Provider Governance. | Provider Assist Activation Pack. | `ph1pae`, `ph1cost`, `ph1prefetch`, `ph1cache`. | Governed capability. | Budget, cache, arbitration, no hidden calls. | Mandatory. | Cost/cache/provider refs. | If visible. | No prefetch cleanup yet. | Hidden provider dispatch. | CODEX_TESTED |
| 20 | Deterministic compute/consensus/calculation. | R, Expansion 12. | PH1.COMP, PH1.E, SimulationExecutor. | Compute Activation Pack. | `ph1comp`, analytics fixtures. | Explain/draft only unless validated. | Deterministic compute and official/protected split. | Provider-off deterministic baseline. | Calculation refs. | Required where visible. | No compute cleanup. | Provider-only official math. | JD_LIVE where visible |
| 21 | Video recognition/rendering/generation. | Q, Expansion 8. | Media/vision/artifact, Provider Governance. | Video model only after JD approval. | Safety, cost, provenance. | Mandatory. | Video evidence/provenance refs. | Required. | No video cleanup. | Exact model/policy missing. | JD_LIVE |
| 22 | Client route presentation/app-open/invite rendering. | Expansion 13. | PH1.LINK/ONB/runtime; clients render only. | None for route authority. | Runtime route validation. | Not applicable. | Client provenance refs. | Required. | No client route cleanup. | Client decides access. | JD_LIVE |
| 23 | Evaluation/JD live acceptance system. | U, Expansion 15/16. | Eval harness, Storage/Audit, JD live, PH1.WRITE/TTS/persona evidence. | Optional graders after governance. | Acceptance matrix and backend agreement, including Quick Assist and Selene visible/audible prompts. | Required if provider-assisted. | Eval/backend/JD refs. | Required. | No old eval deletion. | Backend evidence missing. | CODEX/JD |
| 24 | Old compatibility path retirement. | Expansion 14. | Correct owner per path. | None. | Replacement proof and active-caller checks. | Provider-off/fake as relevant. | Retirement evidence. | Required where visible. | Only retire after proof. | Active callers remain. | JD_LIVE where visible |

## 9. Stack-by-Stack Activation Packs Required

Required activation packs before implementation:

- Overall Repo-Truth Activation Pack
- Provider Governance Activation Pack
- GHCS Build 0A/0C Activation Pack
- Identity + Access + Authority VIA Activation Pack
- Wake/Session/Voice ID/Access Early Product Spine Activation Pack
- PH1.X Deterministic Validation Activation Pack
- PH1.WRITE Presentation Activation Pack
- Conversational Experience + Quick Assist Activation Pack
- Selene Emotional Intelligence + Relationship Presence Activation Pack
- Web Search Source Evidence Activation Pack
- PH1.M Human Memory Core Activation Pack
- Memory Scope and Preference Boundary Activation Pack
- Tool/File/Connector Scope Activation Pack
- Voice/Realtime Route Proof Activation Pack
- Visual/Media Recognition Activation Pack
- Visual Rendering/Image Cards Activation Pack
- Broadcast/Delivery/Reminder/Messaging Activation Pack
- Onboarding/Invite/Link/Enrollment Activation Pack
- Master Access Template/Admin/Tenant/Governance Activation Pack
- Platform Ops Activation Pack
- Artifact Trust/Document/Export/Provenance Activation Pack
- Persona/Preference/Emotion/Feedback/Learning Activation Pack
- Provider Assist/Cost/Prefetch/Arbitration Activation Pack
- Deterministic Compute/Consensus/Calculation Activation Pack
- Video Recognition/Rendering/Generation Activation Pack
- Client Route Presentation/App Open/Invite Rendering Activation Pack
- Evaluation/JD Live Acceptance Activation Pack
- Old Compatibility Retirement Activation Pack
- Model Governance Activation Pack for every `MODEL_POLICY_MISSING` row in Section 5

## 10. Presentation Excellence Plan

PH1.WRITE is the final human output boundary. Every stack must hand PH1.WRITE validated intent, accepted evidence, allowed sources, scope/access limits, risk status, presentation directive, language/style preference where lawful, TTS policy, and UI metadata.

Required presentation capabilities:

- short answers and one-line rewrites;
- long answers and explanatory paragraphs;
- bullet points;
- tables;
- reports;
- emails;
- source chips;
- image cards;
- video cards;
- artifact cards;
- multilingual output;
- TTS-safe text;
- GPT-5.5-assisted Quick Assist guidance, reassurance, clarification, wake acknowledgements, failed-step recovery, process help, result explanation, and weather/time presentation through PH1.WRITE validation;
- Selene emotional presentation and emotional presentation through PH1.WRITE, PH1.EMO, PH1.PERSONA, PH1.M where memory law allows, PH1.TTS approval, and Provider Governance;
- protected refusal wording;
- clarification wording;
- Desktop/iPhone render-only proof.

PH1.WRITE must prevent raw provider JSON, source dumps, unsupported claims, source metadata in TTS, protected action overclaim, private-memory leak, stale-topic output, and provider metadata leakage.

Desktop and iPhone may render approved packets, play approved audio, show approved source/media/artifact cards, and capture input. They must not rewrite, route, select targets, decide access, or call providers directly.

User communication, explanation, clarification, comfort, formatting, guidance, wake acknowledgement wording, weather/time presentation, and Selene emotional presentation/emotional phrasing are probabilistic-first wherever lawful. GPT-5.5 may propose personality wording, humor, warmth, reassurance, and multilingual TTS-safe tone, but Selene owns persona policy, emotional boundaries, PH1.WRITE approval, PH1.TTS approval, memory/privacy limits, access limits, protected-action seriousness, provider governance, and audit evidence. Deterministic gates remain mandatory for identity, access, provider governance, memory permission, source acceptance, tool permission, authority, simulation, audit, protected execution, and state mutation. Deterministic user-language behavior requires JD approval and may not become phrase-patch production law.

## 11. Evidence and Audit Plan

Every implementation slice must define backend evidence before editing.

Required evidence classes:

- semantic proposal evidence;
- PH1.X directive and candidate/rejection evidence;
- source/search/claim evidence;
- memory evidence and memory scope evidence;
- identity/access evidence;
- authority evidence;
- simulation evidence;
- provider model/cost/budget/privacy/counter evidence;
- presentation/write/TTS evidence;
- visual/media/artifact provenance evidence;
- Desktop/Adapter provenance evidence;
- protected fail-closed evidence;
- JD live acceptance evidence;
- old-path active-caller and retirement evidence.

User-visible behavior is not accepted until visible/audible output agrees with backend evidence.

## 12. Old Path Retirement Strategy

Old paths are not deleted by this plan. They retire only after canonical replacement, provider-off/fake-provider proof where relevant, backend evidence proving the correct owner, JD live acceptance where visible, old behavior regression proof, and no active caller remains.

| Current old/compatibility path | Correct canonical replacement owner | Proof before retirement | Tests before retirement | JD live proof | Backend evidence | Active-caller check | Lawful phase |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Large Adapter `lib.rs` semantic/follow-up compatibility surfaces | PH1.X for meaning/target, PH1.WRITE for output, Adapter transport only | PH1.X directive and PH1.WRITE output prove replacement | Adapter/PH1.X/PH1.WRITE regression and provider-off/fake | Yes for visible conversation | Adapter provenance only, PH1.X/WRITE refs | Required | Phase 24 |
| Adapter memory-adjacent synthetic actor/recent recall assertions | PH1.M + identity/access evidence | PH1.M reads canonical memory/speaker scope | Memory scope, unknown speaker denial, PH1.M tests | Yes for memory | PH1.M memory/access refs | Required | Phase 24 |
| Adapter payroll/protected shortcuts or fail-closed helpers | PH1.X + Authority + SimulationExecutor | Protected request fails closed or executes only through simulation | Protected fail-closed and simulation tests | Yes | ProtectedRisk/Authority/Simulation refs | Required | Phase 24 |
| PH1.D public-answer bypass risks | PH1.X claims follow-ups before PH1.D public answer | Live follow-up routes to PH1.X/WRITE when applicable | PH1.D/PH1.X routing tests | Yes | Provider evidence plus directive/output refs | Required | Phase 24 |
| PH1.X older phrase/vocabulary compatibility | Semantic proposal + PH1.X deterministic candidate ledger | No stale target or wrong owner across evals | PH1.X regression/eval pack | Yes | Candidate/rejection ledger | Required | Phase 24 |
| PH1.WRITE formatter shortcuts | PH1.WRITE directive-driven output | Formatting obeys validated directive | PH1.WRITE output tests | Yes | WriteOutput refs | Required | Phase 24 |
| Desktop/iPhone route parsing authority risks | Runtime/PH1.LINK/PH1.ONB server truth, clients render only | Client route events become runtime packets only | Client route/provenance tests | Yes | Desktop/iPhone provenance, runtime route refs | Required | Phase 24 |
| Provider routes without governance | Provider Governance / PH1.D | Provider-off zero attempts, fake provider, budget/model evidence | Provider governance tests | If visible | Provider preflight/counter refs | Required | Phase 24 |
| Phrase-patch shortcuts | Semantic proposal + canonical taxonomy + PH1.X validation | Paraphrase evals prove no exact phrase dependency | Eval pack negative/positive cases | Yes | Proposal/directive evidence | Required | Phase 24 |
| Search/tool/file logic outside PH1.E | PH1.E | Accepted evidence/tool decision only from PH1.E | PH1.E/source/tool/file tests | Yes where visible | PH1.E refs | Required | Phase 24 |
| Memory outside PH1.M | PH1.M | All private/recent recall through memory gateway | PH1.M/access tests | Yes | MemoryScope/MemoryEvidence refs | Required | Phase 24 |
| Protected execution bypass | SimulationExecutor + Authority | No protected action without authority/simulation/audit | Protected fail-closed tests | Yes | Authority/Simulation/Audit refs | Required | Phase 24 |

## 13. No-Skip Coverage Matrix

### Function Stack Map A-U

| Source stack | Included section | Phase | Status |
| --- | --- | --- | --- |
| A Global Human Interface / Semantic Intent | 4, 5, 6 | 4, 5 | PARTIAL |
| B Web Search + Source Evidence | 4, 5, 10 | 7 | PARTIAL |
| C Image-Backed Search + Visual Presentation | 4, 10 | 7, 12 | PARTIAL |
| D Deep Research | 4, 5 | 7/23 future gated | ARCHITECTURE_ONLY |
| E Writing + Transformation | 4, 10 | 6 | PARTIAL |
| F Presentation + TTS-Safe Output | 4, 10 | 6, 10 | PARTIAL |
| G Memory + Recall + Preference | 4, 11, PH1.M Human Memory Core Master Design | 8 | MASTER_DESIGN_ADDED / ACTIVATION_PACK_REQUIRED / NO_RUNTIME_IMPLEMENTATION_YET |
| H File QA + Document | 4, 5 | 9 | PARTIAL |
| I Tool / Connector / MCP | 4, 5 | 9 | PARTIAL |
| J Protected Action + Simulation | 4, 6, 11 | 5, 20 | PARTIAL |
| K Identity + Access + Authority | 4, 6 | 3, 15 | PARTIAL |
| L Voice / Wake / Session / Realtime | 4, 6 | 3, 10 | PARTIAL |
| M Translation + Language Adaptation | 4, 5, 10 | 6 | PARTIAL |
| N Summarization + Compression | 4, 10 | 6 | PARTIAL |
| O Artifact / Document / Slide / Spreadsheet | 4, 10 | 17 | PARTIAL |
| P Image Generation / Editing | 4, 5 | 21 future media lane | ARCHITECTURE_ONLY |
| Q Video Generation | 4, 5 | 21 | REPO_TRUTH_NEEDED |
| R Data Analysis + Report Drafting | 4, 5 | 20 | PARTIAL |
| S Code / Developer Assistance | 4, 5 | 23/supporting | PARTIAL |
| T Cost / Provider Governance / Observability | 4, 5, 6 | 2, 19 | PARTIAL |
| U Evaluation / Regression / JD Live Acceptance | 4, 11 | 23 | PARTIAL |

### Expansion Register 1-16

| Expansion stack | Included section | Phase | Status |
| --- | --- | --- | --- |
| Broadcast / Delivery / Reminder / Messaging | 4, 12 | 13 | PARTIAL |
| Onboarding / Invite / Link / Enrollment | 4, 12 | 14, 22 | PARTIAL |
| Master Access Template / Role / Permission / Admin Controls | 4, 6 | 15 | PARTIAL |
| Tenant / Workspace / Governance / Quota | 4, 6 | 15 | PARTIAL |
| Work / Lease / Scheduling / Health / KMS / Export Platform Ops | 4, 6 | 16 | PARTIAL |
| Visual Recognition / OCR / Media Ingestion / Multimodal Evidence | 4, 5 | 11 | PARTIAL |
| Visual Rendering / Image Cards / Media Presentation | 4, 10 | 12 | PARTIAL |
| Video Recognition / Video Rendering / Video Generation | 4, 5 | 21 | REPO_TRUTH_NEEDED |
| Artifact Trust / Document / Export / Provenance | 4, 11 | 17 | PARTIAL |
| Persona / Preference / Emotion / Feedback / Learning | 4, 11 | 18 | PARTIAL |
| Provider Assist / Cost / Prefetch / Arbitration | 4, 5 | 19 | PARTIAL |
| Deterministic Compute / Consensus / Calculation Authority | 4, 5 | 20 | PARTIAL |
| Client Route Presentation / App Open / Invite Rendering | 4, 12 | 22 | PARTIAL |
| Old Compatibility Path Retirement Register | 12, 13 | 24 | ACTIVATION_PACK_REQUIRED |
| Conversational Experience + Quick Assist | 4, 5, 8, 10, 13 | 2, 3, 4, 6, 10, 23 | ACTIVATION_PACK_REQUIRED |
| Selene Emotional Intelligence + Relationship Presence | 4, 5, 6, 8, 10, 13 | 2, 3, 4, 6, 10, 18, 23 | ACTIVATION_PACK_REQUIRED |

### Repo Engine Inventory Families

| Repo family | Included section | Phase | Status |
| --- | --- | --- | --- |
| PH1.X | 4, 6, 13 | 5 | PARTIAL |
| PH1.WRITE | 4, 6, 10 | 6 | PARTIAL |
| PH1.M | 4, 6, 11, PH1.M Human Memory Core Master Design | 8 | MASTER_DESIGN_ADDED / ACTIVATION_PACK_REQUIRED / NO_RUNTIME_IMPLEMENTATION_YET |
| PH1.E/search/tools/files | 4, 6 | 7, 9 | PARTIAL |
| PH1.W/C/L/TTS/STT/Voice ID | 4, 6 | 3, 10 | PARTIAL |
| PH1.D/Provider Governance/OpenAI | 4, 5, 6 | 2, 4, 19 | PARTIAL |
| PH1.BCAST/DELIVERY/REM | 4, 6 | 13 | PARTIAL |
| PH1.ONB/LINK | 4, 6 | 14, 22 | PARTIAL |
| PH1.ACCESS/POLICY/GOV/TENANT/QUOTA | 4, 6 | 15 | PARTIAL |
| PH1.WORK/LEASE/SCHED/HEALTH/KMS/EXPORT | 4, 6 | 16 | PARTIAL |
| PH1.VISION/OCR/media | 4, 5, 6 | 11, 12, 21 | PARTIAL |
| PH1.PERSONA/EMO/FEEDBACK/LEARN | 4, 6 | 18 | PARTIAL |
| PH1.COST/PREFETCH/CACHE/PAE | 4, 5, 6 | 19 | PARTIAL |
| PH1.COMP | 4, 6 | 20 | PARTIAL |
| SimulationExecutor | 4, 6, 11 | 5, 20 | PARTIAL |
| Desktop/iPhone | 4, 6, 10, 12 | 10, 22 | PARTIAL |
| Adapter | 6, 12 | All transport lanes, retirement in 24 | PARTIAL |
| Artifact/doc/export/provenance | 4, 6 | 17 | PARTIAL |
| Old compatibility/phrase/shortcut paths | 12, 13 | 24 | ACTIVATION_PACK_REQUIRED |

### OpenAI Capability Surfaces

| Capability | Included section | Status |
| --- | --- | --- |
| semantic interpretation | 5 | PARTIAL |
| structured outputs | 5 | PARTIAL |
| tool/function proposals | 5 | PARTIAL |
| web search | 5 | PARTIAL |
| file search | 5 | MODEL_POLICY_MISSING for exact provider/model |
| deep research | 5 | APPROVAL_REQUIRED/PARTIAL |
| embeddings | 5 | PARTIAL |
| STT | 5 | PARTIAL |
| TTS | 5 | PARTIAL |
| realtime voice | 5 | PARTIAL |
| image understanding | 5 | PARTIAL |
| image generation | 5 | MODEL_POLICY_MISSING for exact model ID |
| video generation | 5 | MODEL_POLICY_MISSING for exact model ID |
| writing | 5 | PARTIAL |
| summarization | 5 | PARTIAL |
| translation | 5 | PARTIAL |
| code assistance | 5 | APPROVAL_REQUIRED/PARTIAL |
| evals/graders | 5 | MODEL_POLICY_MISSING for exact grader model |
| provider governance/cost | 5 | PARTIAL |

## 14. Recommended First Implementation Build

Recommended first implementation build:

Provider Governance Baseline + SemanticInterpreterProvider interface skeleton + fake provider + provider-off zero-attempt proof + malformed provider output rejection + minimal SemanticMeaningProposalPacket or repo-equivalent skeleton.

Repo truth supports this ordering because:

- provider-assisted semantic, writing, search, voice, media, and eval behavior cannot lawfully call OpenAI without Provider Governance;
- PH1.D model evidence already requires expected/actual model agreement and blocks fallback/cheaper/unapproved model evidence;
- the model policy exists and pins several approved model routes while marking image/video/eval/file-search gaps for future activation;
- provider-off/fake-provider proof is required before any live/provider-dependent stack can become product behavior.

Wake/Session/Voice ID/Access must be the next early product spine and may receive its activation pack in parallel with provider governance planning, but code-changing provider work should not be bypassed by live OpenAI calls before governance is proven.

The first build instruction must remain narrow and must not implement full conversation, memory, search, Desktop semantics, protected execution, or old-path retirement.

## 15. Stop Conditions

Future builds must stop if any of these occur:

- AGENTS.md not read in the current run;
- clean tree is dirty before editing;
- required activation pack missing;
- approved file scope missing or exceeded;
- current canonical owner cannot be identified from repo truth;
- wrong-owner risk would move meaning/access/memory/provider/protected authority into Adapter or Desktop;
- duplicate owner risk appears;
- model policy missing for the requested provider/model and JD has not approved it;
- fallback/cheaper/unapproved model appears;
- provider-off proof missing where provider involved;
- fake-provider proof missing where provider behavior involved;
- malformed provider output rejection missing;
- prompt-injection defense missing where external evidence involved;
- backend evidence missing;
- JD live failure for visible behavior;
- visible behavior and backend evidence disagree;
- protected action lacks identity/access/authority/simulation/audit;
- Voice ID, wake, memory, provider confidence, Desktop, Adapter, or source text attempts to grant authority;
- phrase patch detected as production law;
- old-path deletion attempted before canonical replacement proof and active-caller check.

## 16. Final Build Plan Status

This is a planning document, not runtime implementation. It does not create provider behavior, packet schemas, runtime routes, Desktop/iPhone behavior, Adapter behavior, PH1 behavior, protected execution, or old-path deletion.

Implementation requires explicit next build instructions, activation packs, approved file scope, tests, backend evidence, provider-off/fake-provider proof where relevant, and JD live acceptance where visible.

### Future Build Instruction Governance

Future implementation instructions must be derived from this plan one slice at a time.

- JD approves scope and priority.
- ChatGPT/Monday may draft build instructions for JD review.
- Codex executes only after receiving explicit build instruction and approved file scope.
- Codex may propose next build instructions, but proposals do not authorize edits.
- Every future slice must obey AGENTS.md, clean tree proof, repo-truth activation, canonical owner boundaries, provider-off/fake-provider proof, exact tests, backend evidence, JD live proof where applicable, docs-only/runtime scope limits, and old-path retirement law.
- Commit/push occurs only after allowed files changed, verification passed, and final tree is clean.

Final docs-task commit and push status for this plan is recorded in the Codex final report for the docs publication slice, not as runtime acceptance.
