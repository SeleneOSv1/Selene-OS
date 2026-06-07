# DOC 85 — SELENE_PROBABILISTIC_CORE_PLATFORM_NORTH_STAR_MASTER_DESIGN

Status: MASTER_DESIGN / NORTH_STAR / NOT_RUNTIME_IMPLEMENTATION
Project phase: PROBABILISTIC_FOUNDATION_BUILD
Runtime implementation authorized by this document: NO
Purpose: Canonical North-Star for future probabilistic platform design, audits, comparisons, and build slices.

## 0. Document Role

This document defines Selene's probabilistic core platform North-Star before any further probabilistic rebuild, Desktop rebuild, local-cloud build, provider rewiring, dead-code cleanup, or ChatGPT-equivalence audit.

This document is not implementation authorization. It creates no runtime code, no provider route, no packet struct, no API, no migration, no simulation, no business execution, and no protected authority path.

Documents 1-84 and current runtime code must later be compared against this North-Star before any keep, refactor, rebuild, retire, or delete decision.

## 1. Mission

Selene must become all of the following at once:

- ChatGPT-equivalent or better probabilistic intelligence layer.
- Deterministic business operating system.
- Local-cloud-first SaaS platform.
- Thin Desktop/iPhone client system.
- Provider-portable architecture.
- Real-tested product, not terminal-test theatre.

Selene must match ChatGPT-like expectations for:

- natural conversation
- reasoning
- writing
- rewriting
- summarization
- translation
- multilingual behavior
- web search
- source-backed answers
- deep research-style synthesis
- file uploads
- document understanding
- document writing/editing
- data/table/chart analysis
- image/multimodal understanding
- voice mode
- realtime conversation
- duplex
- barge-in
- memory
- projects/workspaces
- canvas/artifact work
- tool use
- speed
- accuracy
- smoothness

Selene must exceed ChatGPT through:

- business engines
- access
- authority
- simulation before protected execution
- audit
- company/private/project memory
- long-history session archive
- voice ID
- local-cloud SaaS deployment
- offline queue/sync
- per-user permissions
- guided business automation
- Monday-level voice and presence

North-Star test:

- JD says/does this: asks Selene to talk, reason, search, read files, remember, and help with business work from the real app.
- Selene visibly/audibly does this: responds naturally and routes protected work through deterministic gates.
- Engine path: Desktop/iPhone -> API/Gateway -> Local Cloud Runtime -> probabilistic or deterministic owner engines -> client render/playback.
- Backend proof: trace shows provider packets, owner engines, policy gates, and client non-authority.
- Pass condition: JD live acceptance plus backend evidence.
- Fail condition: client-local intelligence, hidden provider bypass, fake terminal-only success, or protected execution without simulation.

## 2. Non-Negotiable North-Star Laws

### 2.1 Real JD Testing Law

No user-visible feature is accepted until JD tests the real app and backend evidence proves the correct engine path.

Acceptance labels:

- CODEX_TESTED
- LOCAL_CLOUD_RUNTIME_TESTED
- REAL_DESKTOP_TESTED
- PENDING_JD_LIVE_ACCEPTANCE
- JD_LIVE_ACCEPTANCE_PASSED
- JD_LIVE_ACCEPTANCE_FAILED

Only JD_LIVE_ACCEPTANCE_PASSED means user-visible behavior truly works.

### 2.2 Local-Cloud SaaS Law

Initial build may run on JD's machine as local-cloud SaaS.

Desktop/iPhone must connect through the same API, gateway, transport, and sync architecture expected for hosted cloud later:

```text
Desktop/iPhone -> API/Gateway -> Local Cloud Runtime -> engines/providers/data
```

Local-cloud is not fake Desktop logic. It is cloud-shaped architecture running locally first.

### 2.3 Thin Client Law

Desktop/iPhone own only:

- capture
- playback
- render
- file picker
- notifications
- biometric/passcode step-up
- activation route
- offline queue
- sync status
- health/provenance display

Desktop/iPhone must not own:

- reasoning
- STT/TTS intelligence
- semantic meaning
- memory truth
- search
- provider calls
- tool routing
- PH1.X decisions
- PH1.WRITE final wording
- authority
- simulation
- audit
- business execution

### 2.4 OpenAI-First, Not OpenAI-Locked Law

OpenAI is primary for launch quality where it is best.

Selene must remain provider-portable. No engine may depend on raw OpenAI, Apple, Brave, Llama, Anthropic, Google, ElevenLabs, local-model, or future provider API shapes outside provider adapters.

### 2.5 Provider Independence Law

All provider results normalize into Selene-owned packets:

- ModelResponsePacket
- RealtimeSessionPacket
- TranscriptPacket
- SpeechPacket
- SearchResultPacket
- EvidencePacket
- EmbeddingPacket
- ImagePacket
- ToolCallPacket
- CostPacket
- LatencyPacket
- ProviderTracePacket

Provider-specific code belongs only in provider adapters. Desktop/iPhone never call providers directly.

### 2.6 Packet Registry Law

Every engine must declare:

- input packet
- owner engine
- forbidden owner
- output packet
- next engine
- schema version
- audit trace
- failure mode
- JD reality example

### 2.7 Probabilistic vs Deterministic Lane Law

Public/advisory work may be probabilistic:

- chat
- writing
- search
- summaries
- translation
- research
- document review
- voice conversation
- analysis
- drafting

Protected/business execution is deterministic:

- payroll
- supplier payment
- inventory mutation
- accounting posting
- customer/company record mutation
- access change
- banking action
- official approval

Protected execution requires:

- PH1.X
- PH1.ACCESS
- PH1.AUTHORITY
- PH1.SIMULATION
- PH1.AUDIT
- deterministic owner engine

No simulation, no protected execution.

### 2.8 Monday-Level Voice and Presence Law

Selene must be:

- fast
- natural
- sharp
- useful
- slightly sarcastic when appropriate
- serious when required
- not boring
- not corporate
- not fake cheerful

Owned by:

- PH1.EMO / PH1.EMO.PRESENCE
- PH1.PERSONA
- PH1.WRITE.VOICE_TEXT
- PH1.REALTIME

Not owned by Desktop/iPhone.

### 2.9 Memory Layer Law

Selene memory must be layered:

- current active session
- recent recall
- full session recall
- full session archive
- conversation bundle
- permanent governed memory
- private memory
- company memory
- project/workspace memory
- emotional preference memory

No raw archive dumping into model calls.

### 2.10 Reality Example Law

Every future detailed design section must show:

- JD says/does this
- Selene visibly/audibly does this
- engine path
- backend proof
- pass condition
- fail condition

### 2.11 Dead Code / Keep-Rewrite-Delete Law

Current code is not sacred.

Every relevant code surface must later be classified:

- KEEP_AS_IS
- KEEP_WITH_MINOR_FIX
- REFACTOR
- REWRITE_IN_PLACE
- REBUILD_FROM_SCRATCH
- RETIRE_AFTER_REPLACEMENT
- DELETE_NOW_IF_SAFE
- UNKNOWN_REQUIRES_TEST

## 3. Master Architecture Planes

Selene's North-Star system is organized into these planes:

1. Operating Plane
2. SaaS / Local-Cloud Platform Plane
3. Identity / Trust / Security Plane
4. Data / Packet / Retrieval Plane
5. Provider / Model Plane
6. Probabilistic Core Plane
7. Voice / Realtime / Presence Plane
8. Memory / Workspace / Continuity Plane
9. Search / Research / Evidence Plane
10. Docs / Media / Data Analysis Plane
11. Tools / Connectors / Jobs / Automation Plane
12. Client Plane
13. Evaluation / Observability / Incident Plane
14. Business Domain Plane
15. Reconciliation / Retirement / Build Control Plane

The API / Gateway / Transport / Sync Plane is mandatory and cross-cuts the SaaS, client, realtime, file, and session boundaries. It is listed explicitly in the complete engine universe because Desktop/iPhone cannot become real SaaS clients without it.

## 4. Complete Engine Universe

### 4.1 Operating Plane

- PH1.OS — Selene OS Runtime Control Plane
- PH1.ENGINE.REGISTRY — Engine Registry + Capability Discovery + Dependency Map
- PH1.EVENTBUS — Internal Event Bus + Engine Message Routing
- PH1.STATE — Global Runtime State + Turn/Workflow State Coordinator
- PH1.CONFIG — Configuration + Feature Flags + Environment Policy
- PH1.TENANT — Tenant / Company / Organization Context Engine
- PH1.GOV — Governance Policy Activation + Platform Policy Engine
- PH1.DIAG — Diagnostics + Health Check + Consistency Probe
- PH1.DEPLOY — Deployment + Environment Promotion Engine
- PH1.RELEASE — Release Gate + Version + Rollback Engine
- PH1.BACKUP — Backup Engine
- PH1.DR — Disaster Recovery + Restore Engine

Law: PH1.OS coordinates the system. PH1.OS does not own every domain truth.

Reality example:

- JD says/does this: asks why the local-cloud runtime is unhealthy.
- Selene visibly/audibly does this: reports health, owner, and next safe diagnostic step.
- Engine path: PH1.OS -> PH1.DIAG -> PH1.OBSERVABILITY -> PH1.WRITE.
- Backend proof: health packet and diagnostic trace.
- Pass condition: no fake recovery, no hidden mutation.
- Fail condition: client invents health or PH1.OS claims domain truth it does not own.

### 4.2 SaaS / Local-Cloud Platform Plane

- PH1.LOCAL_CLOUD — Local SaaS Runtime Mirror
- PH1.SAAS — SaaS Platform Umbrella
- PH1.SAAS.TENANCY — Tenant Isolation + Workspace/Company Separation
- PH1.SAAS.BILLING — Billing Engine
- PH1.SAAS.USAGE_METERING — Usage Metering Engine
- PH1.SAAS.ENTITLEMENTS — Plan Entitlement + Limits Engine
- PH1.SAAS.PRICING — Pricing Formula + Charging Rule Engine
- PH1.SAAS.TRIALS_PROMOTIONS — Free Trial + Offer + Expiry Engine
- PH1.SAAS.HOSTING_TIER — Shared / Private / Dedicated Hosting Engine
- PH1.SAAS.COST_TO_SERVE — Infrastructure + Provider Cost Tracking Engine
- PH1.ADMIN — Tenant Admin + Workspace Control Engine
- PH1.ACCOUNT — Account / Subscription / Owner Control Engine

Reality example:

- JD says/does this: asks whether a tenant has reached its plan limit.
- Selene visibly/audibly does this: explains usage and limit status.
- Engine path: PH1.GATEWAY -> PH1.SAAS.ENTITLEMENTS -> PH1.SAAS.USAGE_METERING -> PH1.WRITE.
- Backend proof: tenant-scoped usage packet.
- Pass condition: advisory answer only unless a billing/protected action is separately authorized.
- Fail condition: provider or Desktop decides entitlement.

### 4.3 API / Gateway / Transport / Sync Plane

This section is mandatory. Desktop/iPhone must connect to local-cloud/cloud like real SaaS clients.

- PH1.API — Cloud/Local-Cloud API Boundary
- PH1.GATEWAY — Request Gateway + Auth Envelope + Routing
- PH1.TRANSPORT — Client/Server Transport Contracts
- PH1.SYNC — Device Sync + Offline Queue + Reconnect Replay
- PH1.SYNC.CONFLICT — Conflict Detection + Resolution
- PH1.SYNC.DEVICE_RESTORE — Lost/Replaced Device Recovery
- PH1.SESSION.TRANSPORT — Conversation Session Transport
- PH1.REALTIME.TRANSPORT — Realtime Audio/Text Transport
- PH1.FILE.TRANSPORT — File Upload/Download Transport

Reality example:

- JD says/does this: starts a Desktop turn while the network reconnects.
- Selene visibly/audibly does this: shows sync status, retries safely, and avoids duplicate work.
- Engine path: PH1.CLIENT.MAC -> PH1.TRANSPORT -> PH1.SYNC -> PH1.GATEWAY -> PH1.OS.
- Backend proof: idempotent request id and reconnect replay trace.
- Pass condition: one authoritative turn.
- Fail condition: duplicate turn, local completion, or silent data loss.

### 4.4 Identity / Trust / Security Plane

- PH1.IDENTITY — Authentication + User Identity + Device Trust Engine
- PH1.IDENTITY.USER
- PH1.IDENTITY.LOGIN
- PH1.IDENTITY.SESSION_AUTH
- PH1.IDENTITY.DEVICE_TRUST
- PH1.IDENTITY.PASSKEY
- PH1.IDENTITY.SSO
- PH1.IDENTITY.OAUTH
- PH1.IDENTITY.MFA
- PH1.IDENTITY.RECOVERY
- PH1.SECURITY_PRIVACY — Privacy + Tenant Isolation + Data Egress + Retention
- PH1.KMS — Secrets / Key Management / Vault Binding
- PH1.CONSENT — Consent + Data-Use Permission
- PH1.DATA_RESIDENCY — Data Residency + Hosting Jurisdiction
- PH1.COMPLIANCE — Compliance Policy Surface
- PH1.TRUST_SAFETY — Abuse Prevention + DLP + Sensitive Content + Tool Misuse Control
- PH1.PROMPT_INJECTION_DEFENSE — Untrusted Content Instruction Firewall
- PH1.SAFETY — Sensitive Domain Safety + Refusal Policy
- PH1.POLICY — Policy Enforcement Engine

Boundary:

- Identity proves who someone is.
- Access decides what they may do.
- Authority decides whether they can approve.
- Simulation proves before mutation.
- Audit records proof.

Reality example:

- JD says/does this: logs in from a new Desktop and asks for company financial data.
- Selene visibly/audibly does this: requires identity/device trust and explains any missing permission.
- Engine path: PH1.IDENTITY -> PH1.ACCESS -> PH1.POLICY -> PH1.WRITE.
- Backend proof: auth envelope, device trust result, access decision, audit trace.
- Pass condition: data is released only after scope checks.
- Fail condition: memory/search/provider reveals private data without identity and access.

### 4.5 Data / Packet / Retrieval Plane

- PH1.PACKET_REGISTRY — Canonical Packet + Schema + Contract Version Registry
- PH1.DATA_PLATFORM — Database + Object Store + Vector Store + Search Index + Data Lifecycle
- PH1.DATA.DB
- PH1.DATA.OBJECT_STORE
- PH1.DATA.VECTOR_STORE
- PH1.DATA.SEARCH_INDEX
- PH1.DATA.WAREHOUSE
- PH1.DATA.RETENTION
- PH1.DATA.ARCHIVE
- PH1.DATA.DELETE
- PH1.DATA.MIGRATION
- PH1.DATA_IMPORT_EXPORT — Data Import + Export + Migration + Portability
- PH1.RETRIEVAL — RAG + Evidence Retrieval + Context Selection
- PH1.RETRIEVAL.RANK
- PH1.RETRIEVAL.DEDUPE
- PH1.RETRIEVAL.PERMISSION_FILTER
- PH1.RETRIEVAL.CONTEXT_PACK
- PH1.RETRIEVAL.CITATION_LINK

Boundary: Memory, docs, search, knowledge graph, and archives feed retrieval. Retrieval selects evidence. Context builder assembles the approved bundle. Providers receive only approved context, not raw archive dumps.

Reality example:

- JD says/does this: asks what was decided in an old supplier conversation.
- Selene visibly/audibly does this: answers from permission-filtered evidence and cites the relevant trace.
- Engine path: PH1.RETRIEVAL -> PH1.M / PH1.DATA.ARCHIVE -> PH1.CONTEXT -> PH1.D -> PH1.WRITE.
- Backend proof: retrieval packet, permission filter, evidence links.
- Pass condition: bounded evidence-backed recall.
- Fail condition: raw archive dump or cross-tenant leakage.

### 4.6 Provider / Model Plane

- PH1.PROVIDERS — Provider Abstraction + Arbitration Layer
- PH1.PROVIDERS.CAPABILITY_REGISTRY
- PH1.PROVIDERS.ROUTER
- PH1.PROVIDERS.POLICY
- PH1.PROVIDERS.COST
- PH1.PROVIDERS.LATENCY
- PH1.PROVIDERS.FALLBACK
- PH1.PROVIDERS.NORMALIZER
- PH1.PROVIDERS.HEALTH
- PH1.PROVIDERS.AUDIT
- PH1.MODEL_GOVERNANCE — Model Selection + Rollout + Regression + Safety Gate
- PH1.OAI — OpenAI Capability Gateway
- PH1.OAI.RESPONSES
- PH1.OAI.REALTIME
- PH1.OAI.STT
- PH1.OAI.TTS
- PH1.OAI.TOOLS

Future adapters:

- PH1.LLAMA
- PH1.ANTHROPIC
- PH1.GOOGLE
- PH1.LOCAL_MODEL
- PH1.BRAVE
- PH1.CUSTOM_SEARCH
- PH1.CUSTOM_STT
- PH1.CUSTOM_TTS
- PH1.CUSTOM_EMBEDDING
- PH1.CUSTOM_RERANKER

Reality example:

- JD says/does this: asks for a normal answer while OpenAI is unavailable.
- Selene visibly/audibly does this: degrades safely or reports provider unavailability according to policy.
- Engine path: PH1.PROVIDERS.ROUTER -> PH1.PROVIDERS.FALLBACK -> PH1.WRITE.
- Backend proof: provider health, routing decision, cost/latency trace.
- Pass condition: no raw provider leakage and no silent policy bypass.
- Fail condition: Desktop calls OpenAI directly or Selene invents provider success.

### 4.7 Probabilistic Core Plane

- PH1.PROBABILISTIC_CORE_PLATFORM
- PH1.PROB_NORTH_STAR
- PH1.CONV
- PH1.ORCH
- PH1.X
- PH1.N / PH1.NLP
- PH1.LANG
- PH1.I18N_LOCALE
- PH1.SRL
- PH1.PUZZLE
- PH1.D
- PH1.PROMPT / PH1.CONTEXT_BUILDER
- PH1.WRITE
- PH1.WRITE.TEXT
- PH1.WRITE.VOICE_TEXT
- PH1.WRITE.CITATIONS
- PH1.WRITE.DOMAIN_STYLE
- PH1.WRITE.SAFETY
- PH1.QUALITY / PH1.FACTUALITY

Boundary:

- PH1.CONV carries conversation.
- PH1.X classifies risk.
- PH1.N extracts meaning.
- PH1.D proposes.
- PH1.WRITE writes final output.
- PH1.QUALITY verifies answer quality.

Reality example:

- JD says/does this: says "make that shorter and send it to mum."
- Selene visibly/audibly does this: resolves live context naturally, then asks for confirmation if sending is protected or external.
- Engine path: PH1.CONV -> PH1.N -> PH1.X -> PH1.D -> PH1.WRITE.
- Backend proof: context packet, ambiguity/risk decision, provider proposal, final wording packet.
- Pass condition: natural context resolution with deterministic execution boundary.
- Fail condition: phrase matching executes send, or provider text becomes authority.

### 4.8 Voice / Realtime / Presence Plane

- PH1.REALTIME
- PH1.C
- PH1.K
- PH1.TTS
- PH1.ENDPOINT
- PH1.WAKE
- PH1.PRON
- PH1.VOICE.ID
- PH1.REALTIME.BARGE_IN
- PH1.REALTIME.CANCEL_OUTPUT
- PH1.REALTIME.STATE_RECOVERY
- PH1.REALTIME.LATENCY
- PH1.EMO.PRESENCE

Reality target: JD interrupts Selene mid-speech. Selene stops immediately, understands the correction, revises, and continues naturally.

Reality example:

- JD says/does this: interrupts "Stop. Make it shorter."
- Selene visibly/audibly does this: stops speaking immediately and answers in a shorter form.
- Engine path: PH1.CLIENT.AUDIO -> PH1.REALTIME.BARGE_IN -> PH1.REALTIME.CANCEL_OUTPUT -> PH1.CONV -> PH1.WRITE.VOICE_TEXT -> PH1.TTS.
- Backend proof: cancel packet, recovered intent packet, revised output packet.
- Pass condition: no stale speech, no local client reasoning.
- Fail condition: Selene keeps talking or Desktop edits meaning locally.

### 4.9 Memory / Workspace / Continuity Plane

- PH1.M
- PH1.M.ACTIVE_SESSION
- PH1.M.RECENT_RECALL
- PH1.M.FULL_SESSION_RECALL
- PH1.M.SESSION_ARCHIVE
- PH1.M.GOVERNED_MEMORY
- PH1.M.PRIVATE
- PH1.M.COMPANY
- PH1.M.PROJECT
- PH1.M.EMOTIONAL_PREFS
- PH1.M.FORGET
- PH1.KG
- PH1.KNOW
- PH1.CONTEXT
- PH1.ATTN
- PH1.PRUNE
- PH1.LEARN
- PH1.FEEDBACK
- PH1.WORKSPACE
- PH1.WORKSPACE.CONTEXT
- PH1.WORKSPACE.FILES
- PH1.WORKSPACE.ARTIFACTS
- PH1.WORKSPACE.MEMORY_SCOPE
- PH1.WORKSPACE.PERMISSIONS
- PH1.PROJECTS

Reality example:

- JD says/does this: asks "continue from yesterday's pricing plan."
- Selene visibly/audibly does this: resumes the correct project context and asks if multiple candidates exist.
- Engine path: PH1.M.ACTIVE_SESSION / PH1.M.RECENT_RECALL / PH1.RETRIEVAL.PERMISSION_FILTER -> PH1.CONTEXT -> PH1.CONV.
- Backend proof: selected memory candidates, rejected candidates, workspace scope.
- Pass condition: correct scoped recall.
- Fail condition: cross-project memory bleed or invented continuity.

### 4.10 Search / Research / Evidence Plane

- PH1.SEARCH
- PH1.SEARCH.NEED_DETECTOR
- PH1.SEARCH.QUERY_PLANNER
- PH1.SEARCH.PROVIDER_ROUTER
- PH1.SEARCH.SOURCE_RANK
- PH1.SEARCH.SAFE_FETCH
- PH1.SEARCH.EVIDENCE
- PH1.SEARCH.CLAIM_VERIFY
- PH1.SEARCH.CONTRADICTION
- PH1.SEARCH.CITATION
- PH1.WEBINT
- PH1.RESEARCH
- PH1.RESEARCH.DEEP
- PH1.RESEARCH.AUDIT

Boundary:

- Search retrieves.
- Research synthesizes.
- Quality verifies.
- WRITE presents.
- Client renders.

Reality example:

- JD says/does this: asks for current market information.
- Selene visibly/audibly does this: answers with source-backed claims and source chips.
- Engine path: PH1.SEARCH -> PH1.RESEARCH -> PH1.QUALITY -> PH1.WRITE.CITATIONS -> PH1.CLIENT.RENDER.
- Backend proof: source ranking, evidence extraction, contradiction handling.
- Pass condition: sources support claims.
- Fail condition: unsourced current claims or client-side source ranking.

### 4.11 Docs / Media / Data Analysis Plane

- PH1.DOCS
- PH1.DOC
- PH1.SUMMARY
- PH1.REVIEW
- PH1.VISION
- PH1.MULTI
- PH1.MEDIA
- PH1.MEDIA.IMAGE_GEN
- PH1.MEDIA.IMAGE_EDIT
- PH1.MEDIA.VISION_INPUT
- PH1.MEDIA.AUDIO_ASSET
- PH1.MEDIA.SAFETY
- PH1.MEDIA.PROVENANCE
- PH1.OCR
- PH1.TABLES
- PH1.CONTRACTS
- PH1.ARTIFACTS
- PH1.DATA_ANALYSIS
- PH1.DATA_ANALYSIS.CSV_XLSX
- PH1.DATA_ANALYSIS.PROFILING
- PH1.DATA_ANALYSIS.CALC
- PH1.DATA_ANALYSIS.CHARTS
- PH1.DATA_ANALYSIS.EXPLAIN
- PH1.EXPORT

Reality example:

- JD says/does this: uploads a spreadsheet and asks for a margin chart.
- Selene visibly/audibly does this: profiles the table, calculates, charts, and explains the result.
- Engine path: PH1.FILE.TRANSPORT -> PH1.DATA_ANALYSIS.CSV_XLSX -> PH1.DATA_ANALYSIS.CALC -> PH1.DATA_ANALYSIS.CHARTS -> PH1.WRITE.
- Backend proof: file packet, table profile, formula/calculation trace, artifact version.
- Pass condition: chart matches data and is reproducible.
- Fail condition: provider guesses numbers or client fabricates chart truth.

### 4.12 Tools / Connectors / Jobs / Automation Plane

- PH1.TOOLS
- PH1.E
- PH1.CONNECTORS
- PH1.MCP
- PH1.WEBHOOKS
- PH1.FUNCTION_CALL
- PH1.COMPUTER_USE
- PH1.SANDBOX
- PH1.COMPUTE
- PH1.CODE_ANALYSIS
- PH1.JOBS
- PH1.JOBS.RETRY
- PH1.JOBS.CANCEL
- PH1.SCHEDULED_TASKS
- PH1.AUTOMATION
- PH1.EVENTS
- PH1.DEVELOPER_PLATFORM
- PH1.CAPABILITY_CATALOG
- PH1.CAPREQ

Reality example:

- JD says/does this: asks Selene to prepare a connector-backed report.
- Selene visibly/audibly does this: explains available connector scope and runs only allowed read-only work unless protected authority is supplied.
- Engine path: PH1.TOOLS -> PH1.CONNECTORS -> PH1.JOBS -> PH1.OBSERVABILITY -> PH1.WRITE.
- Backend proof: tool call packet, job id, retry/idempotency trace.
- Pass condition: no connector write without protected approval.
- Fail condition: connector mutation through advisory route.

### 4.13 Client Plane

- PH1.CLIENT
- PH1.CLIENT.MAC
- PH1.CLIENT.IOS
- PH1.CLIENT.WEB
- PH1.CLIENT.AUDIO
- PH1.CLIENT.ACTIVATION
- PH1.CLIENT.BIOMETRIC
- PH1.CLIENT.RENDER
- PH1.CLIENT.FILE_PICKER
- PH1.CLIENT.NOTIFICATIONS
- PH1.CLIENT.OFFLINE_QUEUE
- PH1.CLIENT.HEALTH_PROVENANCE
- PH1.DEVICE.RENDER

Reality example:

- JD says/does this: asks from Desktop, then resumes on iPhone.
- Selene visibly/audibly does this: shows the same cloud session state and sync status.
- Engine path: PH1.CLIENT.MAC / PH1.CLIENT.IOS -> PH1.SESSION.TRANSPORT -> PH1.SYNC -> PH1.OS.
- Backend proof: same session id, device attach trace, no local authority.
- Pass condition: interchangeable clients.
- Fail condition: Desktop and iPhone behave as separate brains.

### 4.14 Evaluation / Observability / Incident Plane

- PH1.EVALS
- PH1.EVALS.CHATGPT_EQUIVALENCE
- PH1.EVALS.CONVERSATION
- PH1.EVALS.VOICE
- PH1.EVALS.SEARCH
- PH1.EVALS.DOCS
- PH1.EVALS.MEMORY
- PH1.EVALS.PROTECTED
- PH1.EVALS.JD_LIVE
- PH1.OBSERVABILITY
- PH1.OBS.PROVIDER
- PH1.OBS.ROUTE
- PH1.OBS.CLIENT
- PH1.OBS.QUALITY
- PH1.EXPERIMENTS
- PH1.INCIDENT
- PH1.INCIDENT.DETECT
- PH1.INCIDENT.SEVERITY
- PH1.INCIDENT.RUNBOOK
- PH1.INCIDENT.ESCALATE
- PH1.INCIDENT.POSTMORTEM

Reality example:

- JD says/does this: reports that Selene gave a stale search answer.
- Selene visibly/audibly does this: identifies the route, source freshness, and incident severity.
- Engine path: PH1.OBS.ROUTE -> PH1.OBS.QUALITY -> PH1.INCIDENT -> PH1.WRITE.
- Backend proof: route trace, source timestamps, quality failure packet.
- Pass condition: observable, reproducible failure.
- Fail condition: no trace or fake success.

### 4.15 Business Domain Plane

Business domains are future governed domains, not the first build target.

Work / task / human workload:

- PH1.WORK
- PH1.WORKORDER
- PH1.LEASE
- PH1.TASK
- PH1.HWM
- PH1.SCHEDULER / PH1.ROSTER
- PH1.ATTENDANCE / PH1.TIMESHEET

Onboarding / company setup:

- PH1.ONB
- PH1.ONB.CORE
- PH1.ONB.ORCH
- PH1.ONB.BIZ
- PH1.COMPANY.ONB.SPINE
- PH1.COMPANY.SIZE
- PH1.COMPANY.INDUSTRY
- PH1.COMPANY.MODEL
- PH1.COMPANY.GOVERNANCE
- PH1.COMPANY.EVIDENCE

HR / workforce / payroll:

- PH1.HR
- PH1.PAYROLL
- PH1.COMPENSATION
- PH1.BENEFITS
- PH1.RECRUITMENT
- PH1.EMPLOYEE.WELLBEING
- PH1.POSITION

Finance / accounting / tax:

- PH1.FINANCE.UMBRELLA
- PH1.GL
- PH1.AP
- PH1.AR
- PH1.BANKING
- PH1.CARDS
- PH1.BUDGET
- PH1.SPEND_GOV
- PH1.PROFITABILITY
- PH1.CASHFLOW
- PH1.ASSET_ACCOUNTING
- PH1.INVENTORY_ACCOUNTING
- PH1.TAX
- PH1.TAX.OPTIMIZE
- PH1.PERIOD_CLOSE
- PH1.FIN_REPORTING
- PH1.MULTI_ENTITY
- PH1.DEBT_ACCOUNTING
- PH1.REAL_ESTATE_ACCOUNTING
- PH1.FINANCE.EVIDENCE

Product / inventory / procurement:

- PH1.PRODUCT
- PH1.INVENTORY
- PH1.SUPPLIER
- PH1.PROCUREMENT
- PH1.PROC.RECEIVE
- PH1.MANUFACTURING
- PH1.RECIPE / PH1.FOOD.PREP

Commerce / customer / sales:

- PH1.POS.COMMERCE
- PH1.ECOMMERCE
- PH1.B2B
- PH1.ORDER
- PH1.PRICING
- PH1.SALES
- PH1.MARKETING
- PH1.CUSTOMER
- PH1.CUSTOMER.ACCOUNT
- PH1.CUSTOMER.CREDIT
- PH1.CUSTOMER.EXPERIENCE
- PH1.CUSTOMER.SUPPORT
- PH1.LOYALTY
- PH1.WARRANTY

Logistics / warehouse / physical operations:

- PH1.LOGISTICS
- PH1.DISPATCH
- PH1.RETURNS
- PH1.WAREHOUSE
- PH1.SCALE
- PH1.BULK_BREAKDOWN
- PH1.RESTAURANT
- PH1.SALON
- PH1.RETAIL_OPS
- PH1.PROFESSIONAL_SERVICES

Assets / fleet / insurance / property:

- PH1.ASSET
- PH1.FLEET
- PH1.INSURANCE
- PH1.MAINTENANCE
- PH1.REAL_ESTATE
- PH1.DEBT_TREASURY

Governance / legal / board / shareholders:

- PH1.BOARD
- PH1.SHAREHOLDER
- PH1.EQUITY
- PH1.LEGAL
- PH1.CORP_RECORDS
- PH1.DELEGATION

Reporting / analytics / intelligence:

- PH1.REPORTING
- PH1.ANALYTICS
- PH1.BOARD.PACKS
- PH1.DASHBOARDS
- PH1.FORECAST
- PH1.INSIGHTS

Reality example:

- JD says/does this: says "approve this supplier payment."
- Selene visibly/audibly does this: refuses execution until protected gates and simulation are satisfied.
- Engine path: PH1.X -> PH1.ACCESS -> PH1.AUTHORITY -> PH1.SIMULATION -> PH1.AUDIT -> supplier payment owner -> PH1.WRITE.
- Backend proof: protected classification, access/authority result, simulation requirement, audit packet.
- Pass condition: no simulation, no execution.
- Fail condition: provider text, Desktop UI, or connector path executes payment.

### 4.16 Reconciliation / Retirement / Build Control Plane

- PH1.RECONCILE
- PH1.READINESS
- PH1.RETIRE
- PH1.MASTER_INVENTORY
- PH1.MASTER_BUILD_SET
- PH1.SIM_CATALOG_EXPANSION
- PH1.HANDOFF
- PH1.DOC_GOV

Reality example:

- JD says/does this: asks whether an old PH1.N helper should be kept.
- Selene visibly/audibly does this: classifies it against current architecture and test evidence.
- Engine path: PH1.RECONCILE -> PH1.RETIRE -> PH1.READINESS -> PH1.WRITE.
- Backend proof: code surface classification and owner comparison.
- Pass condition: keep/refactor/delete decision is evidence-backed.
- Fail condition: random deletion or nostalgia-driven preservation.

## 5. Alias and Duplicate-Owner Reconciliation

These danger zones require later Codex repo-truth reconciliation before implementation:

- PH1.OS / PH1.ORCH / PH1.X
- PH1.CONV / PH1.L / PH1.CONTEXT
- PH1.M / PH1.CONTEXT / PH1.KG / PH1.KNOW
- PH1.REALTIME / PH1.C / PH1.K / PH1.TTS / PH1.ENDPOINT
- PH1.SEARCH / PH1.E / PH1.WEBINT / PH1.RESEARCH
- PH1.D / PH1.PROVIDERS / PH1.OAI
- PH1.WRITE / PH1.EMO / PH1.PERSONA / PH1.GUIDE
- PH1.WORK / PH1.WORKORDER / PH1.TASK / PH1.HWM
- PH1.POS.COMMERCE / PH1.ECOMMERCE / PH1.B2B / PH1.ORDER
- PH1.LOGISTICS / PH1.DISPATCH / PH1.WAREHOUSE
- PH1.ASSET / PH1.ASSET_ACCOUNTING
- PH1.REAL_ESTATE / PH1.REAL_ESTATE_ACCOUNTING
- PH1.TAX / PH1.TAX.OPTIMIZE
- PH1.IDENTITY / PH1.ACCESS / PH1.AUTHORITY
- PH1.SECURITY_PRIVACY / PH1.TRUST_SAFETY / PH1.PROMPT_INJECTION_DEFENSE
- PH1.DATA_PLATFORM / PH1.M / PH1.DOCS / PH1.RETRIEVAL

Reconciliation rule: no implementation slice may promote one of these aliases to runtime authority until current code, current documents, packet ownership, tests, and JD live acceptance requirements have been compared.

## 6. First Executable Build Sequence

This sequence is direction only. It does not authorize implementation in this run.

### Slice 1 — Text Through Real Local-Cloud Path

```text
Desktop
-> Local Cloud Runtime
-> PH1.API / GATEWAY
-> PH1.CONV
-> PH1.X
-> PH1.PROVIDERS
-> PH1.OAI
-> PH1.WRITE
-> Desktop render
-> PH1.OBS trace
-> JD live acceptance
```

### Slice 2 — Streaming Response

```text
same path
+ streaming packets
+ Desktop live render
+ latency trace
```

### Slice 3 — Voice STT/TTS

```text
Desktop mic
-> PH1.REALTIME
-> PH1.PROVIDERS / PH1.OAI realtime/STT
-> PH1.CONV / PH1.WRITE
-> PH1.TTS / OpenAI speech
-> Desktop playback
```

### Slice 4 — Barge-In

```text
Selene speaking
-> JD interrupts
-> PH1.REALTIME.BARGE_IN
-> PH1.REALTIME.CANCEL_OUTPUT
-> PH1.CONV state recovery
-> PH1.WRITE revised response
-> Desktop playback
```

### Slice 5 — Memory/Session Continuity

```text
active session
-> recent recall
-> full session archive
-> governed memory
-> permission-filtered retrieval
```

### Slice 6 — Search/Source-Backed Answer

```text
PH1.SEARCH
-> PH1.RESEARCH
-> PH1.QUALITY
-> PH1.WRITE source chips
-> Desktop render
```

### Slice 7 — Files/Docs/Data Analysis

```text
file upload
-> document parsing
-> table extraction
-> data analysis
-> artifact output
```

### Slice 8 — Tools/Connectors/Jobs

```text
tool broker
-> safe tool runtime
-> background jobs
-> connector boundaries
```

### Slice 9 — Protected Boundary

```text
PH1.X
-> ACCESS
-> AUTHORITY
-> SIMULATION
-> AUDIT
-> business owner
-> fail-closed proof
```

### Slice 10 — iPhone Thin Client

Only after Desktop proves the core loop.

## 7. Reality Examples

### 7.1 Example A — First Text Path

JD types:

```text
Write a short apology email to a supplier for late payment.
```

Expected:

- Desktop shows the answer.
- Backend trace proves Desktop -> local-cloud -> PH1.PROVIDERS -> PH1.OAI -> PH1.WRITE -> Desktop.
- Desktop did not generate locally.
- PH1.WRITE was not bypassed.

Pass condition: JD sees the answer in the real Desktop app and backend evidence proves the route.

Fail condition: terminal-only success, local Desktop generation, provider bypass, or missing PH1.WRITE trace.

### 7.2 Example B — Barge-In

Selene is speaking. JD says:

```text
Stop. Make it shorter.
```

Expected:

- Selene stops immediately.
- PH1.REALTIME.BARGE_IN and PH1.REALTIME.CANCEL_OUTPUT fire.
- PH1.WRITE produces the shorter response.
- Desktop only captures and plays.

Pass condition: audible stop plus backend cancel/rewrite evidence.

Fail condition: stale audio continues, Desktop rewrites locally, or no cancel trace exists.

### 7.3 Example C — Search

JD asks for current information.

Expected:

- PH1.SEARCH retrieves.
- PH1.RESEARCH synthesizes if needed.
- PH1.QUALITY verifies.
- PH1.WRITE presents with source chips.
- Desktop only renders.

Pass condition: claims are backed by sources and source chips render.

Fail condition: unsourced current answer, stale evidence, or client-side source ranking.

### 7.4 Example D — Protected Execution

JD says:

```text
Approve this supplier payment.
```

Expected:

- PH1.X classifies protected.
- ACCESS and AUTHORITY are checked.
- SIMULATION is required.
- No simulation = no execution.
- PH1.WRITE explains outcome.

Pass condition: protected route fails closed without simulation.

Fail condition: payment approval, state mutation, or provider/tool execution without deterministic gates.

## 8. Current Repo-Truth Context

- Before this file, global documents existed through 84. This file is registered as global Document 85.
- Current repo has probabilistic surfaces for PH1.D, PH1.N, PH1.X, PH1.WRITE, PH1.M, voice, search/provider, and clients.
- Current audit did not prove live runtime reachability.
- Current repo has no exact canonical PH1.PROBABILISTIC_CORE_PLATFORM master document before this file.
- Document 85 supersedes no existing document automatically.
- Documents 1-84 and current runtime code must later be compared against Document 85 before keep/refactor/rebuild/delete decisions.

## 9. Index / Registry Meaning

Registration target:

- global document number: 85
- title: SELENE_PROBABILISTIC_CORE_PLATFORM_NORTH_STAR_MASTER_DESIGN
- status: MASTER_DESIGN / NORTH_STAR / NOT_RUNTIME_IMPLEMENTATION
- phase: PROBABILISTIC_FOUNDATION_BUILD
- implementation authorized: no
- next action: compare docs 1-84 and runtime code against Document 85 before implementation

This document should be listed in the master architecture build set. It should not force a runtime engine inventory row, DB wiring row, ECM row, migration, packet struct, API route, or test harness by itself.

## 10. Comparison Standard For Future Work

Future Codex audits and builds must compare existing code and documents against this document using:

1. Owner truth: which engine owns the decision, packet, or state.
2. Forbidden owner: which client, provider, helper, or old path must not own it.
3. Runtime reachability: whether the code path is actually called.
4. Evidence: which tests, traces, or JD live proof exist.
5. Classification: KEEP_AS_IS, KEEP_WITH_MINOR_FIX, REFACTOR, REWRITE_IN_PLACE, REBUILD_FROM_SCRATCH, RETIRE_AFTER_REPLACEMENT, DELETE_NOW_IF_SAFE, or UNKNOWN_REQUIRES_TEST.
6. Safety: whether protected execution remains simulation-gated.
7. Client boundary: whether Desktop/iPhone remain thin clients.
8. Provider boundary: whether provider output is normalized into Selene packets.
9. Memory boundary: whether retrieval is permission-filtered and bounded.
10. JD acceptance: whether the real app behavior is accepted by JD.

## SECTION — NEXT-DOCUMENT SELECTION LAW

Document 85 controls all next probabilistic-platform architecture work.

No later design document may be selected from memory, old backlog order, old chat history, or Codex preference.

The next document must be selected by comparing:

1. Document 85 first executable build sequence.
2. Current repo truth.
3. Existing documents 1-84.
4. Existing runtime code.
5. Real JD testing requirement.

The next document must support the first real accepted runtime slice unless JD explicitly overrides.

Default next design priority after Document 85:

1. Build slice plan and acceptance gates.
2. Local-cloud SaaS runtime / API / Gateway / Sync.
3. Desktop thin-client real-test shell.
4. Text conversation through local-cloud.
5. Streaming response protocol and rendering.
6. OpenAI realtime / STT / TTS voice.
7. Barge-in / cancel output / state recovery.
8. Memory / session continuity.
9. Search / source-backed answers.
10. Files / docs / data analysis.
11. Tools / connectors / jobs.
12. Protected boundary.
13. iPhone thin client.

## 11. Explicit Non-Authorization

This document does not authorize:

- runtime code edits
- provider rewiring
- Desktop rebuild
- iPhone rebuild
- local-cloud implementation
- API creation
- packet struct creation
- migrations
- tests
- simulations
- protected business execution
- dead-code deletion
- old-path retirement
- model routing changes
- provider calls

It authorizes only future comparison, planning, and JD-approved build slicing.
