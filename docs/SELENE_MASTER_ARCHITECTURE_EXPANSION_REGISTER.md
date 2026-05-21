Selene Master Architecture Expansion Register

DOCUMENT TYPE:
MASTER ARCHITECTURE EXPANSION REGISTER / REPO-TRUTH GAP CAPTURE

BUILD CLASS:
DOCS-ONLY / ARCHITECTURE EXPANSION REGISTER

PURPOSE:
Capture active or likely Selene function stacks discovered by repo-truth audit that are missing, underdefined, or not yet fully connected in the first six master architecture documents.

This document is not an implementation plan.

It does not authorize runtime edits, provider implementation, packet/schema implementation, Desktop authority, Adapter authority, PH1 runtime changes, old-path deletion, or protected execution changes.

Every stack below requires repo-truth activation before implementation.

GLOBAL RULE:
OpenAI may assist only where governed. Selene remains the owner of validation, routing, authority, protected execution, presentation, and audit.

## 1. Broadcast / Delivery / Reminder / Messaging Stack

- Intent: Define the governed stack for broadcast lifecycle, delivery attempts, reminders, notification-like messaging, and external-message side effects.
- Repo evidence from the audit / current repo truth: `PH1.BCAST`, `PH1.DELIVERY`, and `PH1.REM` are listed in `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`; code exists in `crates/selene_kernel_contracts/src/ph1bcast.rs`, `crates/selene_engines/src/ph1bcast.rs`, `crates/selene_os/src/ph1bcast.rs`, `crates/selene_kernel_contracts/src/ph1delivery.rs`, `crates/selene_engines/src/ph1delivery.rs`, `crates/selene_os/src/ph1delivery.rs`, `crates/selene_kernel_contracts/src/ph1rem.rs`, and `crates/selene_os/src/ph1rem.rs`; persistence contracts appear in `docs/04_KERNEL_CONTRACTS.md` KC.17 and KC.18.
- Current six-document coverage gap: Messaging and delivery are covered only generically through tools, protected execution, provider governance, and presentation. There is no first-class broadcast/delivery/reminder stack.
- Likely canonical owner or owner family: PH1.BCAST, PH1.DELIVERY, PH1.REM, PH1.E where read-only tool classification is involved, SimulationExecutor and Access/Authority where external side effects are protected.
- OpenAI role: May draft message text, classify user intent, propose delivery wording, or summarize delivery status when governed; must not send, acknowledge, retry, or complete delivery.
- Selene-owned validation role: Validate recipient scope, delivery channel, side-effect class, simulation requirement, idempotency, delivery state, retry/defer/expire rules, and auditability.
- Identity/access/authority/protected-risk implications: External message sending and broadcast delivery are side-effecting and may require access, authority, simulation, and audit; public advisory messaging drafts remain non-mutating.
- Required presentation/audit implications: PH1.WRITE should present draft/send/fail/defer/ack states clearly; TTS must not imply delivery success without proof; audit must record delivery attempts and protected denials.
- Recommended next architecture/build action: Add a dedicated Broadcast / Delivery / Reminder / Messaging architecture slice with owner map, route law, proof law, and old-path classification. ACTIVATION_PACK_REQUIRED.

## 2. Onboarding / Invite / Link / Enrollment Stack

- Intent: Define the governed stack for invitation links, onboarding sessions, user/company/employee onboarding, wake or voice enrollment handoffs, and onboarding requirement backfill.
- Repo evidence from the audit / current repo truth: `PH1.ONB`, `PH1.LINK`, `PH1.POSITION`, `PH1.CAPREQ`, `PH1.W`, and `PH1.VOICE.ID` appear in `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`; code exists in `crates/selene_os/src/ph1onb.rs`, `crates/selene_os/src/ph1link.rs`, `crates/selene_os/src/ph1position.rs`, `crates/selene_os/src/ph1capreq.rs`, `crates/selene_os/src/ph1w.rs`, and `crates/selene_os/src/ph1_voice_id.rs`; onboarding and wake enrollment persistence appear in `docs/04_KERNEL_CONTRACTS.md` KC.16, KC.20, KC.21, and KC.25; iPhone and Desktop shells render onboarding/invite postures.
- Current six-document coverage gap: IAA covers identity/access generally, but there is no first-class onboarding/invite/enrollment journey stack connecting link activation, requirements, access instance creation, device/wake enrollment, and client render-only boundaries.
- Likely canonical owner or owner family: PH1.ONB, PH1.LINK, PH1.POSITION, PH1.CAPREQ, PH1.W, PH1.VOICE.ID, Access/Governance, PH1.F/PH1.J for persistence and audit.
- OpenAI role: May explain onboarding requirements or help draft user-facing onboarding copy when governed; must not activate links, bind identities, grant access, complete onboarding, or enroll voice/wake artifacts.
- Selene-owned validation role: Validate link state, invite scope, requirements schema, identity/access posture, enrollment evidence, dedupe, tenant scope, and audit.
- Identity/access/authority/protected-risk implications: Onboarding can change access posture and identity-related state; it must remain deterministic, session-bound, and simulation/authority-gated where applicable.
- Required presentation/audit implications: Clients may render onboarding status only; audit must prove link activation, onboarding requirement decisions, enrollment state changes, and denied/expired/recovery paths.
- Recommended next architecture/build action: Add an Onboarding / Invite / Link / Enrollment architecture pack before implementation planning. ACTIVATION_PACK_REQUIRED.

## 3. Master Access Template / Role / Permission / Admin Controls Stack

- Intent: Define access-policy authoring, role templates, permission templates, board approval policies, overlays, overrides, and compiled access lineage.
- Repo evidence from the audit / current repo truth: `PH1.ACCESS.001_PH2.ACCESS.002` is authoritative in `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`; `docs/COVERAGE_MATRIX.md` lists access schema, overlay, board policy, vote, and access instance compile simulations; `docs/04_KERNEL_CONTRACTS.md` includes role catalog and KC.26 schema-driven Master Access objects.
- Current six-document coverage gap: IAA defines access and authority concepts, but access-template authoring and compiled lineage are not first-class expansion stacks.
- Likely canonical owner or owner family: Access/Governance, PH1.POLICY, PH1.GOV, PH1.TENANT, PH1.F/PH1.J, SimulationExecutor for protected access changes.
- OpenAI role: May explain policies, propose natural-language summaries, or draft admin-facing review text; must not grant access, author active policy, vote, override, compile, or activate permission templates.
- Selene-owned validation role: Validate role/template schema, overlay operations, tenant scope, approval policy, board thresholds, authority, simulation, idempotency, and audit lineage.
- Identity/access/authority/protected-risk implications: This stack is authority-sensitive by default; unknown identity or missing authority must fail closed.
- Required presentation/audit implications: PH1.WRITE must distinguish draft/review/active/denied access states; audit must capture template version, compiled lineage, approvals, overrides, and denial reasons.
- Recommended next architecture/build action: Add Master Access Template architecture before final overall build planning. ACTIVATION_PACK_REQUIRED.

## 4. Tenant / Workspace / Governance / Quota Stack

- Intent: Define tenant/workspace context resolution, policy pointers, governance decisions, quota lanes, provider/resource budgets, and admin-visible control state.
- Repo evidence from the audit / current repo truth: `PH1.TENANT`, `PH1.GOV`, and `PH1.QUOTA` are listed in `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`; code exists in `crates/selene_kernel_contracts/src/ph1tenant.rs`, `crates/selene_engines/src/ph1tenant.rs`, `crates/selene_os/src/ph1tenant.rs`, `ph1gov.rs`, and `ph1quota.rs`; DB/ECM docs exist for these engines.
- Current six-document coverage gap: Provider governance and IAA mention policy, budget, and access, but tenant/workspace governance is not explicit as a stack.
- Likely canonical owner or owner family: PH1.TENANT, PH1.GOV, PH1.QUOTA, PH1.OS, PH1.POLICY, PH1.F/PH1.J.
- OpenAI role: May summarize governance posture or help explain quota/admin status; must not choose tenant, change policy, override quota, or grant budget.
- Selene-owned validation role: Validate tenant scope, workspace context, governance policy version, quota lane, provider budget gates, and audit.
- Identity/access/authority/protected-risk implications: Tenant/workspace routing affects private data and protected access; missing tenant evidence must fail closed for protected/private operations.
- Required presentation/audit implications: PH1.WRITE should show bounded governance/quota status without secret or internal policy dumps; audit should record tenant/policy/quota decisions and denials.
- Recommended next architecture/build action: Add Tenant / Workspace / Governance / Quota stack section or expansion pack. ACTIVATION_PACK_REQUIRED.

## 5. Work / Lease / Scheduling / Health / KMS / Export Platform Ops Stack

- Intent: Define platform operations for work-order ledgers, leases, deterministic scheduling, health visibility, key/secret handling, and compliance export.
- Repo evidence from the audit / current repo truth: `PH1.WORK`, `PH1.LEASE`, `PH1.SCHED`, `PH1.HEALTH`, `PH1.KMS`, and `PH1.EXPORT` are listed in `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`; code exists in matching `crates/selene_kernel_contracts/src`, `crates/selene_engines/src`, and `crates/selene_os/src` files; `docs/04_KERNEL_CONTRACTS.md` includes work, lease, capreq, and related persistence contracts.
- Current six-document coverage gap: Stack T covers provider governance/observability, but not the full Selene platform-ops family for work, lease, scheduler, health, KMS, and export.
- Likely canonical owner or owner family: PH1.WORK, PH1.LEASE, PH1.SCHED, PH1.HEALTH, PH1.KMS, PH1.EXPORT, PH1.OS, PH1.F/PH1.J.
- OpenAI role: May explain health status, draft export summaries, or assist with non-secret operational explanations; must not issue keys, acquire leases, mutate work ledgers, schedule protected retries, or export compliance data.
- Selene-owned validation role: Validate work-order append/no-op decisions, lease ownership, scheduling decisions, KMS opaque handle issuance, export authorization, redaction, and audit.
- Identity/access/authority/protected-risk implications: KMS/export/work/lease operations can be protected and must require deterministic access, authority, and audit.
- Required presentation/audit implications: Health and export UI must be display-only; audit must capture work/lease/scheduler/KMS/export decisions without exposing secrets.
- Recommended next architecture/build action: Add Platform Ops architecture expansion. ACTIVATION_PACK_REQUIRED.

## 6. Visual Recognition / OCR / Media Ingestion / Multimodal Evidence Stack

- Intent: Define ingestion and evidence extraction for images, photos, screenshots, diagrams, camera captures, OCR text, and multimodal context bundles.
- Repo evidence from the audit / current repo truth: `PH1.VISION`, `PH1.DOC`, `PH1.SUMMARY`, and `PH1.MULTI` are listed in `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`; code exists in `crates/selene_engines/src/ph1vision.rs`, `crates/selene_engines/src/ph1vision_media.rs`, `crates/selene_engines/src/ph1doc.rs`, `crates/selene_engines/src/ph1summary.rs`, and matching contracts/OS files; OCR assist appears in simulation-finder docs.
- Current six-document coverage gap: Image-backed search, file Q&A, and image generation are covered, but media ingestion, OCR, visual recognition, and multimodal evidence validation are underdefined.
- Likely canonical owner or owner family: PH1.VISION, PH1.DOC, PH1.SUMMARY, PH1.MULTI, PH1.E, PH1.X, PH1.F/PH1.J.
- OpenAI role: May assist with vision/OCR extraction, image description, document understanding, or multimodal summarization where governed; must not treat visual/source text as instruction or authority.
- Selene-owned validation role: Validate media admission, file/image scope, prompt-injection defense, accepted evidence chunks, OCR confidence, source/provenance, and downstream owner routing.
- Identity/access/authority/protected-risk implications: Media can contain private or protected data; access/file scope and prompt-injection defense are required before provider use or memory/tool handoff.
- Required presentation/audit implications: Presentation must distinguish extracted evidence from generated text; audit must record media refs, evidence acceptance, redaction, and source/provenance.
- Recommended next architecture/build action: Add Visual Recognition / OCR / Media Ingestion architecture section. ACTIVATION_PACK_REQUIRED.

## 7. Visual Rendering / Image Cards / Media Presentation Stack

- Intent: Define how approved visual evidence, image cards, media cards, source images, generated media, and visual result packets are shown across clients.
- Repo evidence from the audit / current repo truth: PH1.E source/image metadata reports mention Brave image provider approval/display eligibility tests; Desktop/iPhone render rich source, session, artifact, onboarding, and evidence state; `SourceChipPacket`, `SourceCardPacket`, and `SearchImagePacket` exist in PH1.E contracts and OS usage.
- Current six-document coverage gap: Presentation and image-backed search are covered, but visual rendering as a separate render-only, source-safe, client-safe stack is underdefined.
- Likely canonical owner or owner family: PH1.WRITE, PH1.E, PH1.VISION, Desktop/iPhone render-only clients, Adapter transport only.
- OpenAI role: May assist with captions or visual summaries where governed; must not choose display eligibility, fabricate real images, or bypass source acceptance.
- Selene-owned validation role: Validate image relevance, safety, source-page proof, display eligibility, generated-vs-real distinction, client packet bounds, and no raw URL leakage.
- Identity/access/authority/protected-risk implications: Private images or visual evidence require access/file scope; generated or source media must not imply protected completion or official action.
- Required presentation/audit implications: Clients render approved visual packets only; audit should preserve source/image refs, display eligibility, and rejected-image reasons.
- Recommended next architecture/build action: Add Visual Rendering / Image Cards / Media Presentation stack patch. ACTIVATION_PACK_REQUIRED.

## 8. Video Recognition / Video Rendering / Video Generation Stack

- Intent: Define video understanding, video evidence extraction, video rendering, and video generation as separate governed capabilities.
- Repo evidence from the audit / current repo truth: The six-doc set mentions video generation, but audit did not find a clear current runtime video owner comparable to PH1.VISION for images. Video recognition/rendering implementation evidence remains REPO_TRUTH_NEEDED.
- Current six-document coverage gap: Stack Q covers video generation only. Video recognition and video rendering are missing or underdefined.
- Likely canonical owner or owner family: REPO_TRUTH_NEEDED; likely PH1.VISION/PH1.MULTI/PH1.E for video evidence, PH1.WRITE/media artifact owner for presentation, provider governance for generation.
- OpenAI role: May assist video generation, storyboard, or video understanding only after provider governance and media safety gates exist.
- Selene-owned validation role: Validate media scope, safety, provenance, generated-vs-real labeling, cost/budget, privacy, and presentation eligibility.
- Identity/access/authority/protected-risk implications: Video can contain biometric/private/protected data and may require identity, file scope, consent, and data-egress checks.
- Required presentation/audit implications: Video outputs must be provenance-labeled; rendering must be approved packet only; audit must capture media refs, provider refs, and safety decisions.
- Recommended next architecture/build action: Add this as a future architecture gap entry with REPO_TRUTH_NEEDED before implementation.

## 9. Artifact Trust / Document / Export / Provenance Stack

- Intent: Define artifact authenticity, document generation, export, provenance, activation, trust-root checks, and derived artifact lifecycle.
- Repo evidence from the audit / current repo truth: `ARTIFACTS_LEDGER_TABLES` is in `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`; PH1.DOC and PH1.EXPORT exist; `docs/CORE_ARCHITECTURE.md` lists artifact authenticity and trust-root verification as a P0 gap; Desktop/iPhone render artifact-related surfaces.
- Current six-document coverage gap: Stack O covers artifact/document/slide/spreadsheet work, but trust-root verification, activation, provenance, and export lifecycle need stronger first-class architecture.
- Likely canonical owner or owner family: Artifact/storage owner, PH1.DOC, PH1.EXPORT, PH1.GOV, PH1.F/PH1.J, PH1.WRITE for final wording.
- OpenAI role: May draft or transform artifact content where governed; must not activate artifacts, assert authenticity, export protected artifacts, or create official record status.
- Selene-owned validation role: Validate artifact source evidence, provenance, hash/signature/trust-root where available, export access, redaction, official/advisory distinction, and audit.
- Identity/access/authority/protected-risk implications: Artifact activation/export may be protected or private; access and authority must be deterministic.
- Required presentation/audit implications: PH1.WRITE must distinguish draft/generated/verified/exported/official states; audit must record provenance and export proof.
- Recommended next architecture/build action: Expand Stack O into an Artifact Trust / Document / Export / Provenance stack. ACTIVATION_PACK_REQUIRED.

## 10. Persona / Preference / Emotion / Feedback / Learning Stack

- Intent: Define safe user adaptation, persona hints, emotional guidance, feedback capture, learning artifacts, preference application, and non-authoritative learning boundaries.
- Repo evidence from the audit / current repo truth: `PH1.PERSONA`, `PH1.EMO.GUIDE`, `PH1.EMO.CORE`, `PH1.FEEDBACK`, `PH1.LEARN`, `PH1.LISTEN`, `PH1.CACHE`, `PH1.KNOW`, `PH1.KG`, `PH1.PATTERN`, and `PH1.RLL` are listed in `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`; matching code exists across contracts, engines, and OS files.
- Current six-document coverage gap: Memory/preference and presentation are covered, but learning, emotion, persona, feedback, and adaptation are not a full stack.
- Likely canonical owner or owner family: PH1.M, PH1.PERSONA, PH1.EMO.GUIDE, PH1.EMO.CORE, PH1.FEEDBACK, PH1.LEARN, PH1.KNOW, PH1.KG, PH1.WRITE.
- OpenAI role: May suggest tone, summarize feedback, propose preference signals, or help draft adaptive wording; must not become memory truth, identity proof, authority, or behavior law.
- Selene-owned validation role: Validate identity scope, memory law, preference permission, tone-only boundaries, no meaning drift, learning artifact approval, and audit.
- Identity/access/authority/protected-risk implications: Preferences and emotional/persona signals can be private; unknown identity should block private personalization and persistent memory writes.
- Required presentation/audit implications: PH1.WRITE should apply allowed style hints without claiming durable preference unless memory law allows; audit should record feedback and adaptation evidence.
- Recommended next architecture/build action: Add Learning / Persona / Emotion / Feedback architecture pack. ACTIVATION_PACK_REQUIRED.

## 11. Provider Assist / Cost / Prefetch / Arbitration Stack

- Intent: Define provider assist selection, cost hints, prefetch candidates, provider arbitration, promotion/demotion, and degraded provider posture.
- Repo evidence from the audit / current repo truth: `PH1.COST`, `PH1.PREFETCH`, `PH1.PAE`, `PH1.D`, `PH1.SEARCH`, and provider governance docs/reports exist; live OpenAI and PH1.E provider tests are present as controlled/ignored/proof surfaces.
- Current six-document coverage gap: Provider governance is covered broadly, but assist arbitration, prefetch, promotion/demotion, and cost-adaptation are underdefined as a stack.
- Likely canonical owner or owner family: Provider Governance, PH1.COST, PH1.PREFETCH, PH1.PAE, PH1.D, PH1.E, PH1.J.
- OpenAI role: Provider itself may provide capability when allowed; it must not decide budget, routing authority, fallback policy, or promotion.
- Selene-owned validation role: Validate provider allowlist, capability key, model policy, provider-off/fake-provider behavior, budget, circuit breaker, data egress, prefetch TTL, and arbitration result.
- Identity/access/authority/protected-risk implications: Provider context must respect privacy and protected data rules; provider availability must not weaken protected fail-closed behavior.
- Required presentation/audit implications: User-visible answers should not expose raw provider metadata; audit must record provider attempts, dispatches, budgets, degradation, and model evidence.
- Recommended next architecture/build action: Expand provider governance into an Assist / Cost / Prefetch / Arbitration stack. ACTIVATION_PACK_REQUIRED.

## 12. Deterministic Compute / Consensus / Calculation Authority Stack

- Intent: Define deterministic computation authority for scoring, ranking, normalization, consensus, budget math, conflict resolution, and official calculations.
- Repo evidence from the audit / current repo truth: `PH1.COMP` is listed in `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md` as the deterministic compute engine; code exists in `crates/selene_kernel_contracts/src/ph1comp.rs`, `crates/selene_engines/src/ph1comp.rs`, and `crates/selene_os/src/ph1comp.rs`; `docs/SELENE_BUILD_EXECUTION_ORDER.md` has Build Section 10 for Numeric and Consensus Computation.
- Current six-document coverage gap: Data analysis/report drafting is covered, but deterministic compute authority is not fully separated from advisory provider analysis.
- Likely canonical owner or owner family: PH1.COMP, PH1.OS, PH1.X, PH1.E for evidence-backed calculations, SimulationExecutor for protected official calculations.
- OpenAI role: May explain calculations, draft advisory analysis, or propose formulas; must not own deterministic numeric authority or official computation.
- Selene-owned validation role: Validate input data, deterministic algorithm/version, replayability, consensus rules, official/advisory classification, and audit.
- Identity/access/authority/protected-risk implications: Official calculations over company records may be protected and require access, authority, and simulation; user-supplied advisory calculations remain public/advisory unless they mutate state or claim official status.
- Required presentation/audit implications: PH1.WRITE must label advisory vs official outputs; audit must capture computation packet/equivalent refs, inputs, versions, and official-status decisions.
- Recommended next architecture/build action: Add Deterministic Compute / Consensus architecture expansion. ACTIVATION_PACK_REQUIRED.

## 13. Client Route Presentation / App Open / Invite Rendering Stack

- Intent: Define client-side route rendering for app-open, invite, onboarding, session visibility, failed/needs-attention queues, source chips, interruption continuity, and current app provenance while preserving client no-authority.
- Repo evidence from the audit / current repo truth: `apple/iphone/SeleneIPhone/SessionShellView.swift`, `apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`, and `apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift` contain rich rendering and route-parsing surfaces for invite/open/onboarding/session/TTS/source/provenance states, with repeated no-local-authority wording.
- Current six-document coverage gap: Desktop/iPhone are stated as render-only, but app-open/invite rendering and client route presentation are not a dedicated stack.
- Likely canonical owner or owner family: Desktop/iPhone clients render only; Adapter transports only; cloud runtime owners remain PH1.L, PH1.X, PH1.WRITE, PH1.ONB, PH1.LINK, PH1.BCAST/DELIVERY, PH1.F/PH1.J as applicable.
- OpenAI role: None for client authority. May assist content generation upstream only when governed and approved by canonical owners.
- Selene-owned validation role: Validate cloud-authored route state, bounded metadata, session refs, provenance refs, and no local mutation/authority.
- Identity/access/authority/protected-risk implications: Client surfaces must not grant identity, access, authority, resend, retry, complete onboarding, or mutate session/protected state.
- Required presentation/audit implications: Clients should render approved packets and expose bounded proof/posture; audit must prove client-visible state corresponds to cloud-authoritative evidence.
- Recommended next architecture/build action: Add Client Route Presentation / App Open / Invite Rendering stack or patch IAA/client architecture. ACTIVATION_PACK_REQUIRED.

## 14. Old Compatibility Path Retirement Register

- Intent: Track retained compatibility paths, wrong-owner surfaces, phrase/vocabulary shortcuts, old deterministic context helpers, legacy provider/tool routes, and cleanup conditions.
- Repo evidence from the audit / current repo truth: Reports identify `crates/selene_adapter/src/lib.rs::deterministic_active_context_followup_query`, `deterministic_weather_context_followup_query`, adapter protected/payroll classification helpers, synthetic `ph1m_actor_recent_recall_assertion`, and retained PH1.X/Adapter compatibility paths. The six-doc set already requires old-path retirement only after proof.
- Current six-document coverage gap: Old-path retirement is required globally, but there is no master register enumerating the active compatibility surfaces discovered across repo truth.
- Likely canonical owner or owner family: Depends on path; PH1.X for meaning/target, PH1.M for memory, PH1.E for tools/search/files, PH1.WRITE for output, Adapter for transport-only cleanup, Desktop/iPhone for render-only cleanup.
- OpenAI role: None for retirement decisions. Provider-assisted semantic proposal may help future canonical replacements only after governance and PH1.X validation.
- Selene-owned validation role: Classify each path as retained compatibility, wrong-owner path, duplicate path, dead path, or retirement candidate; require replacement proof, backend evidence, JD live acceptance where visible, and no active caller before removal.
- Identity/access/authority/protected-risk implications: Old protected, memory, identity, delivery, or provider paths must remain fail-closed and must not be removed until canonical owners prove equivalent or stronger behavior.
- Required presentation/audit implications: Retirement reports must preserve evidence, route provenance, test proof, and final clean tree proof.
- Recommended next architecture/build action: Create an old-path retirement register after activation packs map active callers. ACTIVATION_PACK_REQUIRED.

## 15. Conversational Experience + Quick Assist Stack

- Intent: Use GPT-5.5 through Selene-owned provider interfaces to make Selene natural, helpful, comforting, clarifying, and adaptive during user interaction.
- Covers: wake acknowledgements; listening/session reassurance; quick clarify; quick confirm; quick suggest; “where am I?” process help; “what should I do next?” guidance; deterministic process explanation; failed-step recovery wording; time/weather/forecast presentation; tool/search/result explanation; short/long/bullet/paragraph/table formatting; multilingual guidance; TTS-safe friendly phrasing.
- Repo evidence from the audit / current repo truth: PH1.WRITE, PH1.X, PH1.W, PH1.C, PH1.L, PH1.TTS, PH1.E, PH1.M, Desktop/iPhone render surfaces, and Adapter transport surfaces are mapped in `docs/SELENE_OVERALL_REPO_TRUTH_ACTIVATION_PACK.md`; existing compatibility risks include deterministic wake/help/weather/follow-up wording and Adapter/PH1.X phrase-style helper paths.
- Current six-document coverage gap: Existing architecture covers writing, presentation, wake/session, search/tool answers, and memory preference boundaries, but it did not name normal guided conversational experience as a first-class cross-cutting stack.
- Likely canonical owner or owner family: PH1.WRITE for final wording; PH1.X for validated user/process intent; PH1.W / PH1.C / PH1.L for wake/session state; PH1.TTS for approved speech; PH1.E for facts/tools/search/weather/time; PH1.M for preference/personalization only where memory law allows; Desktop/iPhone render/play only; Adapter transport only.
- OpenAI role: GPT-5.5 may propose natural wording, clarification, comfort, explanation, options, next-step guidance, formatting, and TTS-safe phrasing.
- Selene-owned validation role: Selene validates state, scope, risk, evidence, and final output. GPT-5.5 does not grant access, authority, memory permission, tool permission, or protected execution.
- Identity/access/authority/protected-risk implications: Human communication may be probabilistic-first wherever lawful, but private personalization requires memory/access scope and protected or mutating behavior remains deterministic-gated.
- Required architecture rule: Human communication with Selene is probabilistic-first wherever lawful. Deterministic gates remain for identity, access, provider governance, memory scope, source acceptance, tool permission, authority, simulation, audit, and state mutation.
- Forbidden: hardcoded wake greetings; phrase-list clarification logic; hardcoded weather/time templates as primary UX; stack-local language understanding outside SemanticInterpreterProvider / PH1.X; Desktop or Adapter conversational brain; deterministic user-help behavior without JD approval.
- Required presentation/audit implications: PH1.WRITE must approve final `display_text` / `tts_text`; PH1.TTS may speak only approved text; backend evidence must show the deterministic state/fact owner, provider wording proposal where used, PH1.WRITE validation, and client render/play provenance.
- Recommended next architecture/build action: Create Quick Assist Activation Pack before implementation. First implementation must be provider-off/fake-provider safe and must route final wording through PH1.WRITE. ACTIVATION_PACK_REQUIRED.

## 16. Celine Persona + Emotional Presentation Stack

- Intent: Create Selene’s official conversational personality layer, called Celine, so user interaction feels warm, witty, emotionally intelligent, lightly playful, and human-guided instead of robotic, deterministic, or cold.
- Covers: personality style; humor/sarcasm level; emotional warmth; reassurance; user comfort; frustration handling; confusion handling; serious-mode behavior; protected-action serious wording; wake greeting personality; Quick Assist tone; TTS voice style; multilingual personality consistency; preference-aware tone where memory law allows; emotional engine integration; persona safety and boundary rules.
- Repo evidence from the audit / current repo truth: PH1.WRITE, PH1.X, PH1.TTS, PH1.PERSONA, PH1.EMO.GUIDE, PH1.EMO.CORE, PH1.FEEDBACK, PH1.LEARN, PH1.M, Provider Governance, Desktop/iPhone render surfaces, and Adapter transport surfaces are mapped in `docs/SELENE_OVERALL_REPO_TRUTH_ACTIVATION_PACK.md`; Expansion Register stack 10 already captures persona/preference/emotion/feedback/learning as a broader adaptation stack, while this stack makes Celine’s user-facing persona layer first-class.
- Current six-document coverage gap: Existing architecture covers persona, memory preference, emotional guidance, Quick Assist, and PH1.WRITE presentation, but it does not yet name Celine persona and emotional presentation as the official cross-cutting personality layer.
- Likely canonical owner or owner family: PH1.WRITE owns final persona wording; PH1.EMO / PH1.EMO.CORE / PH1.EMO.GUIDE own emotional-state assist surfaces where current repo truth supports them; PH1.PERSONA owns persona profile surfaces where current repo truth supports them; PH1.FEEDBACK / PH1.LEARN may assist adaptation only under memory/privacy law; PH1.M owns durable preference memory where allowed; PH1.X validates user intent/risk/state before persona output; PH1.TTS owns approved speech rendering; Provider Governance controls GPT-5.5/OpenAI usage; Desktop/iPhone render or speak only; Adapter transports only.
- OpenAI role: GPT-5.5 may propose natural wording, humor, emotional phrasing, comforting explanation, clarification wording, wake greetings, Quick Assist guidance, tone adaptation, TTS-safe phrasing, and multilingual personality-preserving wording.
- Selene-owned validation role: Selene validates persona policy, seriousness level, safety boundaries, emotional appropriateness, identity/access/privacy limits, memory permission, protected-risk context, final `display_text`, and final `tts_text`.
- Core personality: Celine should be witty, warm, emotionally perceptive, lightly sarcastic where appropriate, playful but useful, clear and direct, supportive when the user is confused, serious when stakes are high, never cruel, never manipulative, never blocking for comedy, and never overriding safety or authority gates.
- Tone modes: normal chat: playful, warm, clever; wake acknowledgement: short, varied, friendly; technical build work: sharp, practical, lightly funny; confused user: calm, guiding, low sarcasm; frustrated user: reassuring, direct, low sarcasm; emotional/stress context: gentle, no mockery; protected action: serious, clear, no jokes that imply approval; access denial: respectful and clear; error/failure: helpful, accountable, low humor; celebratory success: lively but concise.
- Identity/access/authority/protected-risk implications: Celine personality is probabilistic-first for communication but deterministic-gated for safety; private personalization requires identity/access and memory scope, and protected or mutating contexts require serious wording without implying approval.
- Required architecture rule: Celine personality is probabilistic-first for communication but deterministic-gated for safety. Correct wiring is user input / wake event / process state -> GPT-5.5 persona or wording proposal where applicable -> PH1.X or correct owner validates state, scope, risk, and action class -> PH1.WRITE applies persona policy and approves final `display_text` / `tts_text` -> PH1.TTS speaks approved text -> Desktop/iPhone render or play only.
- Forbidden: persona deciding access; persona deciding authority; persona changing state; persona executing tools; persona approving protected actions; persona bypassing PH1.X; persona bypassing PH1.WRITE; persona bypassing PH1.M memory law; persona hiding uncertainty; persona making unsupported factual claims; persona insulting the user harshly; persona using humor in sensitive protected/action-denial contexts; Desktop/Adapter persona brain; hardcoded personality phrase lists as the main UX.
- Required presentation/audit implications: PH1.WRITE must approve final persona wording and TTS-safe text; PH1.TTS may speak only approved text; backend evidence must show persona/writing proposal where used, relevant emotional/persona/preference refs, memory/access scope where personalization is durable, PH1.WRITE validation, and client render/play provenance.
- Recommended next architecture/build action: Create a future Celine Persona + Emotional Presentation Activation Pack before runtime implementation. First implementation must use provider-off/fake-provider safe behavior and PH1.WRITE final validation. No live OpenAI personality behavior is allowed until Provider Governance permits it. ACTIVATION_PACK_REQUIRED.

## Final Register Status

- Architecture reference: READY
- Runtime implementation authorization: NOT_GRANTED
- Packet/schema implementation authorization: NOT_GRANTED
- Old path deletion authorization: NOT_GRANTED
- Required before final overall build planning: YES
