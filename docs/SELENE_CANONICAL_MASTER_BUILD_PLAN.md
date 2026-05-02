# Selene Canonical Master Build Plan

Status: CANONICAL_BUILD_ROADMAP
Created: 2026-05-02
Repo Root: `/Users/selene/Documents/Selene-OS`
Current Next Build: Stage 1 - Canonical Engine Inventory And Wiring Map

## Purpose

This document is the build plan Selene must follow so implementation does not drift.

It exists to keep every future build aligned to one target:

1. Build one Selene pipeline, not a second brain.
2. Reuse already-built engines where repo truth proves they exist.
3. Finish and wire partial engines before rebuilding them.
4. Keep public chat/search fast, useful, source-backed, and read-only.
5. Keep protected execution simulation/authority gated.
6. Build presentation contracts before Desktop/iPhone visual polish.
7. Keep source chips, image cards, TTS, provider gates, and trace behavior governed.

This plan must be updated after every completed build so the next stage is always explicit.

## Governing Rules

- `AGENTS.md` wins over this file when there is any conflict.
- Python is disallowed in this repository.
- Do not hardcode real searched names in code, tests, fixtures, mocks, corpora, sample data, or proof hooks.
- Do not run live providers unless the build instruction explicitly allows it and provider-off proof has passed first.
- Do not weaken provider gates, budget counters, startup-probe blocks, or protected execution gates.
- Public websearch is read-only and does not require simulation.
- Protected execution requires authority and simulation.
- Desktop and iPhone are renderers. They must not become search brains, provider callers, source rankers, image choosers, or execution authorities.
- Search must not be rebuilt from scratch. Existing search/source/image work should be finished, wired, and proven.
- Rich Desktop/iPhone UI must be built through presentation contracts, adapter transport, and renderer support.
- TTS must speak only clean approved `tts_text`, not source chips, image metadata, debug traces, raw URLs, provider JSON, or internal classes.

## Current Repo Truth Summary

This plan is based on repo checks and docs review through local HEAD:

- Branch: `main`
- Local HEAD at planning time: `351d15dd44813ad5a625e96b0cd8a432623703f8`
- `origin/main` at planning time: `75ac6c48ae34d791e5a86dafb4fd3755b2fda051`
- Local branch was ahead of `origin/main` by 2 commits during planning.
- Current repo has a large PH1 system. It is not an empty rebuild.
- Existing docs include `SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`, `SELENE_BUILD_EXECUTION_ORDER.md`, `COVERAGE_MATRIX.md`, `MASTER_BUILD_COMPLETION_PLAN.md`, and `MASTER_BUILD_COMPLETION_LEDGER.md`.
- Existing Desktop and adapter code already transport/render some source chips and image cards.
- Existing search/source/image systems are substantial but still need quality, wiring, and proof closure.
- Older references to an 86-engine plan must be reconciled in Stage 1 before being treated as build authority.
- The current target architecture is organized around four core stacks:
  1. Continuous Conversation
  2. Universal Understanding
  3. Write / Response
  4. Search / Tool Intelligence

## Build Tracking Rule

After every build, update this section before final reporting.

| Field | Current Value |
|---|---|
| Current active stage | Stage 1 |
| Current active build | Canonical Engine Inventory And Wiring Map |
| Next build after current stage passes | Stage 2 - Conversation Packet And State Foundation |
| Last completed stage | None under this canonical plan |
| Stages blocked | None yet |
| Plan drift allowed | No |

## Status Legend

- `EXISTS_BUT_NEEDS_RECONCILIATION`: code/docs exist, but ownership/status/wiring must be audited.
- `PARTIALLY_BUILT`: capability exists in some form but needs finishing or wiring.
- `NEEDS_BUILDING`: capability is missing or only conceptual.
- `NEEDS_FINISHING`: capability is implemented enough to reuse but still fails required proof.
- `PROVEN_COMPLETE`: stage passed required tests/proofs and docs were updated.
- `BLOCKED`: stage cannot proceed until an explicit blocker is fixed.

## Canonical Build Order

The correct order is:

`inventory -> conversation packets -> runtime turn spine -> conversation control -> understanding plus early language governance -> voice proof -> orchestrator plus language-aware routing -> safety/routing -> search evidence quality -> write plus Write/TTS language continuity -> presentation contracts -> adapter transport plus language/display transport -> Desktop -> iPhone -> advanced language profiles/certification -> product capability families -> final certification`

## Continuous Proof Gate

Certification does not wait until the final stage.

Every stage must include:

- targeted tests for the capability being built;
- regression tests for provider gates, protected execution, TTS/display separation, and no real searched-name hardcoding when relevant;
- no unrelated broad rewrites;
- `git diff --check`;
- docs or ledger update when the build instruction requires it;
- final proof of the next stage pointer in this document.

Stage 26 is the full unified certification harness. It is not the first proof stage.

## Stage 1 - Canonical Engine Inventory And Wiring Map

Status: NEEDS_BUILDING

Build:

- One repo-truth inventory of all PH1 engines/modules.
- One wiring map from ingress to conversation, understanding, routing, execution, write, adapter, Desktop, iPhone, and TTS.
- One status table for every engine: complete, partial, standalone, duplicated, unwired, missing, or deprecated.
- One reconciliation between:
  - current repo modules;
  - `SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`;
  - newer four-stack master plan;
  - older 86-engine references;
  - Desktop and iPhone surfaces;
  - adapter/runtime routes;
  - tests/proofs.

Must identify:

- all existing contracts;
- all existing engine implementations;
- all OS orchestration modules;
- all adapter response/transport packet shapes;
- all Desktop render paths;
- all iPhone render paths;
- all provider/search gates;
- all protected execution gates;
- all duplicated or legacy concepts;
- every stage that is already partly built.

Proof:

- No code behavior change required unless a doc-only inventory build explicitly allows it.
- Inventory must be based on exact file paths and repo evidence.
- Final output must state the next stage clearly.

Next if passed:

- Stage 2 - Conversation Packet And State Foundation.

## Stage 2 - Conversation Packet And State Foundation

Status: PARTIALLY_BUILT

Build:

- conversation packet contracts;
- listen-state packet shape;
- committed-turn packet shape;
- interruption packet shape;
- barge-in cancellation packet shape;
- speech-output packet shape;
- clarification/correction/recovery packet shapes;
- deterministic trace fields;
- stable `session_id` and `turn_id` propagation.

Rules:

- Partial speech must not become a committed turn.
- Abandoned speech must not trigger understanding, tools, search, write, simulation, or execution.
- Packet contracts must be deterministic and trace-ready.
- This stage should be mostly contracts and boundaries, not broad behavior changes.

Proof:

- packet compile tests;
- non-committed turn rejection tests;
- stale/cancelled turn packet tests;
- no tool/search/execution route from packet creation alone.

Next if passed:

- Stage 3 - Runtime Session / Turn Spine Closure.

## Stage 3 - Runtime Session / Turn Spine Closure

Status: PARTIALLY_BUILT

Build:

- ingress to session wiring;
- session to turn wiring;
- turn candidate to committed turn promotion;
- committed turn to transcript/understanding handoff;
- stale turn quarantine;
- abandoned turn safe-degrade;
- interrupted turn cancellation;
- superseded turn invalidation.

Rules:

- Only a committed current turn can enter Universal Understanding.
- Old tool results must not render as current answers.
- Old speech output must not continue after cancellation.
- Runtime spine closure must not bypass protected execution gates.

Proof:

- stale turn blocked;
- cancelled turn blocked;
- superseded turn blocked;
- committed turn accepted;
- no direct raw text to tool route.

Next if passed:

- Stage 4 - Continuous Conversation Control Stack.

## Stage 4 - Continuous Conversation Control Stack

Status: PARTIALLY_BUILT

Build:

- `PH1.CONVERSATION.CONTROL`;
- `PH1.LISTEN.STATE`;
- `PH1.TURN.STATE`;
- `PH1.INTERRUPT.CONTROL`;
- `PH1.BARGEIN.CANCEL`;
- `PH1.SPEECH.OUTPUT.CONTROL`;
- `PH1.DISCOURSE.FRAME`;
- `PH1.CORRECTION.CONTROL`;
- `PH1.CLARIFICATION.LOOP`;
- `PH1.CONVERSATION.TIMING`;
- `PH1.CONVERSATION.RECOVERY`;
- `PH1.CONVERSATION.SAFETY.GATE`;
- `PH1.CONVERSATION.TRACE`;
- `PH1.CONVERSATION.EVAL`.

Rules:

- Stop-speaking is a control action, not a normal chat request.
- Barge-in cancels current output and makes the new user speech authoritative.
- Conversation recovery must fail safe, not force execution.
- TTS must speak only approved TTS-safe text.

Proof:

- interruption proof;
- barge-in proof;
- short pause does not commit a low-confidence turn;
- stale answer cannot render after interruption;
- recovery does not invent a user turn;
- trace records state transitions without leaking raw audio or secrets.

Next if passed:

- Stage 5 - Transcript, Spelling, Grammar, Language, And Understanding Spine.

## Stage 5 - Transcript, Spelling, Grammar, Language, And Understanding Spine

Status: PARTIALLY_BUILT

Build:

- `PH1.TRANSCRIPT`;
- early `PH1.LANG.PACKET`;
- early `PH1.LANG.SAFETY`;
- `PH1.LANG.NORMALIZE`;
- `PH1.SPELL.PHONETIC.RESOLVE`;
- `PH1.GRAMMAR.MEANING.REPAIR`;
- `PH1.SEMANTIC.PARSE`;
- `PH1.ENTITY.RESOLVE`;
- `PH1.CONTEXT.RETRIEVE`;
- `PH1.HYPOTHESIS.GENERATE`;
- `PH1.HYPOTHESIS.RANK`;
- `PH1.INTENT.RESOLVE`;
- `PH1.UNDERSTANDING.TRACE`;
- `PH1.CORRECTION.LEARN`;
- `PH1.UNDERSTANDING.EVAL`.

Rules:

- Original transcript and repaired meaning must both remain traceable.
- Language packet and language safety are core understanding foundations, not late product polish.
- Language metadata must not infer protected identity such as nationality, race, ethnicity, citizenship, or protected class.
- Wrong-language STT capture must be blocked or clarified before routing.
- Spelling/phonetic repair must be generic, not hardcoded to real names.
- Grammar repair must not invent missing protected slots.
- Entity resolution must not accept partial-name overlap as exact identity.
- Low confidence must clarify instead of pretending certainty.

Proof:

- broken English repair;
- spelling/phonetic alternatives;
- language packet creation;
- wrong-language STT mismatch blocking;
- assistant-name ambiguity handling;
- entity ambiguity handling;
- wrong-entity rejection;
- no protected-slot guessing.

Next if passed:

- Stage 6 - Early Voice Proof Repair.

## Stage 6 - Early Voice Proof Repair

Status: NEEDS_FINISHING

Build:

- real English Desktop voice proof path;
- real Chinese Desktop voice proof path;
- mixed-language voice proof path;
- assistant-name disambiguation for `Selene` vs similar-sounding words;
- wrong-language STT mismatch blocking;
- transcript confidence and clarification behavior.

Rules:

- Voice proof must use real captured transcript when required.
- Typed proof cannot replace voice proof.
- Assistant-name repair must be generic app-name/context lexicon behavior, not public-search hardcoding.
- Protected voice ambiguity must fail closed or clarify.

Proof:

- English no-search prompt;
- English public search prompt;
- Chinese no-search prompt;
- Chinese public search prompt;
- mixed-language prompt;
- assistant-name prompt;
- protected command fail-closed by voice.

Next if passed:

- Stage 7 - Reasoning Orchestrator And Capability Registry.

## Stage 7 - Reasoning Orchestrator And Capability Registry

Status: NEEDS_BUILDING

Build:

- `PH1.REASON.ORCH`;
- `PH1.CAPABILITY.REGISTRY`;
- early `PH1.LANG.TOOL.ROUTE`;
- capability ownership registry;
- route ownership table;
- direct-answer vs search vs tool vs file vs data vs canvas vs task vs protected route decision.

Rules:

- This is not a second brain.
- It must route resolved intent through existing Selene engines.
- Language-aware routing is part of capability routing, not a late renderer feature.
- It must not bypass risk, authority, simulation, provider, or budget gates.
- Every capability must have one clear owner.

Proof:

- no capability can run from raw text;
- resolved public question can route to direct answer/search;
- non-English and mixed-language requests route to the correct capability;
- file/doc/data/canvas/task routes choose correct owners;
- protected action still requires authority and simulation.

Next if passed:

- Stage 8 - Intent / Risk / Authority / Simulation / Tool Route Closure.

## Stage 8 - Intent / Risk / Authority / Simulation / Tool Route Closure

Status: PARTIALLY_BUILT

Build:

- `PH1.RISK.CLASSIFY`;
- `PH1.AUTHORITY.GATE`;
- `PH1.PLAN.VERIFY`;
- `PH1.TOOL.ROUTE`;
- `PH1.SIMULATION`;
- `PH1.EXECUTION_GATE`;
- mixed public/protected command separation.

Rules:

- Public chat, public search, weather, time, reading, and normal answers do not require simulation.
- Protected mutation requires authority and matching simulation.
- No simulation means no protected execution.
- No authority means no protected execution.
- Protected ambiguity must clarify or fail closed.

Proof:

- public business-word query answers normally;
- payroll/HR/finance/access/inventory/approval mutation fails closed without authority/simulation;
- mixed search + payroll separates public read-only search from protected blocked action;
- no mutation from best-available guessing.

Next if passed:

- Stage 9 - Search / Source / Image Evidence Quality Finish.

## Stage 9 - Search / Source / Image Evidence Quality Finish

Status: NEEDS_FINISHING

Build:

- finish existing `PH1.SEARCH`;
- finish existing `PH1.E`;
- finish source verification;
- finish best-available answer selection;
- finish accepted/rejected source ranking;
- finish wrong-source rejection;
- finish weak-source rejection;
- finish entity-role search quality;
- finish closest-supported answer behavior;
- finish source chips;
- finish approved image packet creation;
- finish provider-off zero-call proof;
- finish controlled live-provider proof where explicitly allowed.

Rules:

- Do not rebuild search from scratch.
- Do not hardcode real companies, people, products, wineries, or one-off searched names.
- This stage produces accepted evidence, rejected evidence, source-chip packets, image-card packets, provider traces, and best-available answer decisions.
- Final screenshot-level user-facing quality depends on later PH1.WRITE, presentation contracts, adapter transport, and Desktop/iPhone renderers.
- Public websearch must have enough evidence data for best available source-backed answers when accepted evidence exists.
- Source chips must show accepted supporting sources only.
- Rejected sources remain trace/debug only.
- Image cards come only from approved metadata and do not prove factual claims.
- Provider calls must obey global gate, paid-provider gate, budget, counter, and no-startup-probe law.

Proof:

- exact role answer;
- closest supported role answer;
- official source wins over weak directory;
- wrong entity rejected;
- weak source rejected;
- conflicting accepted sources resolved naturally;
- source chips attached;
- image packet approved/blocked correctly;
- provider-off zero attempts;
- protected mixed command fail-closed.

Next if passed:

- Stage 10 - PH1.WRITE Evidence Input.

## Stage 10 - PH1.WRITE Evidence Input

Status: NEEDS_BUILDING

Build:

- `PH1.WRITE.EVIDENCE.INPUT`;
- accepted evidence packet;
- rejected evidence packet;
- claim-to-source map input;
- freshness input;
- contradiction input;
- internal confidence input;
- source-chip metadata input.

Rules:

- Rejected evidence cannot become source chips.
- Unsupported claims cannot become confident facts.
- Internal confidence remains trace/internal unless explicitly allowed.
- PH1.WRITE must not invent facts, sources, citations, titles, roles, dates, prices, images, or people.

Proof:

- accepted vs rejected separation;
- unsupported claim removal;
- contradiction handling;
- no raw provider/source/debug leakage in `response_text` or `tts_text`.

Next if passed:

- Stage 11 - PH1.WRITE Image Evidence Input.

## Stage 11 - PH1.WRITE Image Evidence Input

Status: NEEDS_BUILDING

Build:

- `PH1.WRITE.IMAGE.EVIDENCE.INPUT`;
- approved image input;
- blocked image input;
- deferred image input;
- metadata-only image state;
- image relevance reason;
- image source-page relation;
- image display policy state.

Rules:

- Images are display evidence, not claim proof.
- Blocked images cannot render as successful image cards.
- Metadata-only image state must not pretend a real image rendered.
- Raw unsafe image URLs must not enter `response_text`, `tts_text`, or normal UI.

Proof:

- approved image renders as card downstream;
- blocked image does not render as success;
- unsafe image URL blocked;
- rejected-source image blocked;
- wrong-entity image blocked.

Next if passed:

- Stage 12 - PH1.WRITE Spelling / Grammar / Style Bridge.

## Stage 12 - PH1.WRITE Spelling / Grammar / Style Bridge

Status: NEEDS_BUILDING

Build:

- `PH1.WRITE.SPELL.GRAMMAR.STYLE`;
- early `PH1.LANG.WRITE.TTS.CONTINUITY`;
- output spelling cleanup;
- output grammar cleanup;
- same-language output polish;
- professional style pass;
- protected-slot preservation.

Rules:

- Polish must not change factual meaning.
- Write may polish output, but it must not reinterpret evidence.
- Write must not become a second reasoning layer.
- Write must not change protected slots.
- Write must not convert uncertainty into certainty.
- Polish must not hide uncertainty.
- Polish must not turn weak evidence into certainty.
- Protected required fields must not be silently guessed.

Proof:

- messy input produces clean output;
- same-language continuity preserved;
- display/TTS language continuity preserved;
- factual meaning preserved;
- protected ambiguity remains blocked/clarified.

Next if passed:

- Stage 13 - PH1.WRITE Structured Writing Engine.

## Stage 13 - PH1.WRITE Structured Writing Engine

Status: NEEDS_BUILDING

Build:

- `PH1.WRITE.STRUCTURED.WRITING`;
- direct answer mode;
- explainer mode;
- research answer mode;
- comparison mode;
- business writing mode;
- build instruction mode;
- table answer mode;
- clarification mode;
- limitation answer mode.

Must support:

- titles;
- headings;
- subheadings;
- paragraphs;
- bullet lists;
- numbered steps;
- tables;
- callout/code-style blocks;
- bottom-line sections;
- warnings and limitations;
- source-aware claims.

Rules:

- Simple questions stay concise.
- Complex answers can use structure.
- Source-backed answers remain source-backed.
- TTS path remains clean and natural.

Proof:

- screenshot-style headings/paragraphs/bullets;
- Codex-ready build instruction output;
- concise direct answer output;
- comparison output;
- no debug/source/image metadata in spoken text.

Next if passed:

- Stage 14 - Presentation Contracts.

## Stage 14 - Presentation Contracts

Status: NEEDS_BUILDING

Build:

- `PH1.PRESENTATION.CONTRACTS`;
- `PresentationPacket`;
- block order contract;
- heading block;
- paragraph block;
- bullet list block;
- numbered list block;
- table block;
- source chip block;
- image card block;
- quote block;
- warning block;
- divider block;
- display-only metadata;
- TTS-only text.

Rules:

- Display answer and spoken answer may differ.
- Display may include structure, source chips, images, tables, and metadata.
- TTS speaks only approved natural text.
- No raw HTML, provider metadata, trace metadata, secrets, or fake citations.

Proof:

- block order preserved;
- source chips remain accepted-only;
- image cards remain approved-only;
- display-only metadata not spoken;
- TTS-only text not rendered as debug.

Next if passed:

- Stage 15 - Adapter Rich Transport.

## Stage 15 - Adapter Rich Transport

Status: PARTIALLY_BUILT

Build:

- `PH1.ADAPTER.TRANSPORT`;
- structured block transport;
- source chip transport;
- image card transport;
- language packet transport;
- language safety state transport;
- display/TTS language continuity transport;
- safety state transport;
- display hash;
- TTS hash;
- trace ID transport.

Rules:

- Adapter must not flatten rich answers into weak plain text.
- Adapter must not rewrite captions into facts.
- Adapter must not drop approved source chips or approved image cards.
- Adapter must not expose blocked image candidates or rejected sources in normal output.
- Adapter must preserve language packet, language safety, display language, and TTS language without silent rewrite.

Proof:

- structured blocks preserved;
- source chips preserved;
- image cards preserved;
- TTS/display separation preserved;
- language/display/TTS continuity preserved;
- no provider/debug/raw URL leakage.

Next if passed:

- Stage 16 - Desktop Rich Renderer.

## Stage 16 - Desktop Rich Renderer

Status: PARTIALLY_BUILT

Build:

- `PH1.DESKTOP.RICH.RENDERER`;
- large readable headers;
- subheaders;
- paragraph spacing;
- bullet lists;
- numbered lists;
- tables;
- callout/code-style blocks;
- compact source pills;
- image cards;
- dividers;
- loading states;
- blocked/deferred image states;
- safe fallback for unsupported blocks.

Rules:

- Desktop is only renderer.
- Desktop must not search, call providers, rank sources, choose images, hold provider keys, infer photo identity, or authorize protected execution.
- Source pills must be compact, clickable, rounded, subtle gray/shadow style, near the answer.
- Approved image cards must render actual approved image/thumbnail, not placeholder-only success.

Proof:

- screenshot-level structured answer rendering;
- source pill visual parity;
- real approved image rendering;
- placeholder only while loading/failure;
- no raw URL/source dump/provider JSON/unix_ms/rejected source details;
- TTS clean.

Next if passed:

- Stage 17 - iPhone Rich Renderer.

## Stage 17 - iPhone Rich Renderer

Status: NEEDS_BUILDING

Build:

- iPhone presentation contract decoder;
- mobile heading rendering;
- mobile paragraphs and lists;
- mobile table fallback;
- mobile source pills;
- mobile image cards;
- mobile TTS/display separation;
- mobile safe fallback states.

Rules:

- iPhone follows the same backend presentation contract.
- iPhone must not become a search brain or execution authority.
- Mobile layout may differ, but proof semantics must match Desktop.

Proof:

- iPhone renders structured answer blocks;
- source chips clickable and compact;
- image cards approved-only;
- protected fail-closed visible;
- no raw debug/provider/source/image leakage.

Next if passed:

- Stage 18 - Advanced Language Profiles And Language Certification Foundation.

## Stage 18 - Advanced Language Profiles And Language Certification Foundation

Status: PARTIALLY_BUILT

Build:

- `PH1.LANG.PROFILE.REGISTRY`;
- `PH1.LANG.TRACE.EVAL`;
- advanced language profile fixtures;
- multilingual certification fixtures;
- provider-off language fallback proof;
- language trace certification.

Rules:

- Core language packet, safety, tool routing, and Write/TTS continuity must already be built earlier in Stages 5, 7, 12, and 15.
- This stage expands profiles, fixtures, trace evaluation, and certification coverage.
- OpenAI or any provider may supply multilingual intelligence where configured, but Selene governs profiles, trace, certification, safety proof, and provider-off fallback.
- Language profiles cannot override truth, source verification, provider gates, authority gates, simulation gates, or protected execution law.

Proof:

- English typed and voice;
- Chinese typed and voice;
- mixed English/Chinese;
- non-English search/tool routing;
- non-English protected command fail-closed;
- language profile fixture coverage;
- provider-off language fallback;
- TTS/display language hash proof.

Next if passed:

- Stage 19 - Product Capability Family: Project / Memory / Persona / Workspace.

## Stage 19 - Product Capability Family: Project / Memory / Persona / Workspace

Status: PARTIALLY_BUILT

Build:

- project context;
- workspace boundaries;
- custom instructions;
- persona;
- memory retrieve/store policy;
- project-only memory;
- tenant/workspace governance;
- sharing/collaboration boundaries.

Rules:

- Memory and persona affect usefulness/style, not truth or safety.
- Project-only context must stay inside project.
- Workspace rules must not override authority, simulation, source-truth, or provider law.

Proof:

- project context retrieval;
- project boundary enforcement;
- persona style without factual distortion;
- memory cannot authorize protected execution;
- cross-tenant leakage blocked.

Next if passed:

- Stage 20 - Product Capability Family: File / Doc / Data.

## Stage 20 - Product Capability Family: File / Doc / Data

Status: PARTIALLY_BUILT

Build:

- `PH1.FILE`;
- `PH1.DOC`;
- `PH1.DATA`;
- file intake;
- document classification;
- PDF/document/text extraction;
- CSV/JSON/spreadsheet analysis;
- table output;
- chart-ready output.

Rules:

- Uploaded artifacts become evidence/work objects with provenance.
- Unsupported/unsafe files safe-degrade.
- Data analysis must not fake calculations.
- Document/data outputs must route through Write and presentation contracts.

Proof:

- file intake fixtures;
- document summary;
- document extraction;
- CSV/table analysis;
- JSON analysis;
- chart-ready structured output;
- no cross-project leakage.

Next if passed:

- Stage 21 - Product Capability Family: Canvas / Artifacts.

## Stage 21 - Product Capability Family: Canvas / Artifacts

Status: NEEDS_BUILDING

Build:

- `PH1.CANVAS`;
- versioned artifact workspace;
- document artifact editing;
- build-plan artifact editing;
- code-plan artifact editing;
- revision history;
- artifact-to-chat handoff;
- chat-to-artifact handoff.

Rules:

- Canvas is an editing workspace, not an authority engine.
- Canvas cannot trigger protected execution by itself.
- Canvas state must obey workspace/project boundaries.

Proof:

- artifact open/create/update;
- versioned edits;
- artifact rendering;
- no execution from canvas alone;
- project boundary preserved.

Next if passed:

- Stage 22 - Product Capability Family: Agent / Apps / Connectors / Tasks.

## Stage 22 - Product Capability Family: Agent / Apps / Connectors / Tasks

Status: PARTIALLY_BUILT

Build:

- `PH1.AGENT`;
- `PH1.APPS.CONNECTORS`;
- `PH1.TASKS`;
- connector registry;
- read/write connector separation;
- scheduled tasks;
- reminders;
- recurring checks;
- external app routing;
- authority-gated connector writes.

Rules:

- Agent cannot run from raw text.
- Connector access requires workspace and authority checks.
- Read-only connector work is separate from mutation.
- Protected connector actions require simulation/execution gates.
- Scheduled tasks re-enter through ingress and do not bypass gates.

Proof:

- read connector route;
- write connector blocked without authority;
- reminder scheduling;
- recurring task policy;
- scheduled web check respects provider budgets;
- protected scheduled mutation fails closed.

Next if passed:

- Stage 23 - Product Capability Family: Meeting Recording / Record Mode.

## Stage 23 - Product Capability Family: Meeting Recording / Record Mode

Status: PARTIALLY_BUILT

Build:

- `PH1.RECORD`;
- record-mode capture;
- meeting/voice-note artifact path;
- post-recording summary path.

Rules:

- Core live voice runtime belongs earlier in Stages 2-6.
- This stage is for product-level meeting recording and voice-note capture.
- Record mode captures and processes after completion.
- Record mode must not answer live like assistant chat.
- Record mode must not trigger tools from partial recording.

Proof:

- record mode transcript artifact;
- meeting summary through doc/write path;
- no tool route from partial recording;
- no permanent raw audio retention by default unless policy allows.

Next if passed:

- Stage 24 - Product Capability Family: Image Generation / Editing.

## Stage 24 - Product Capability Family: Image Generation / Editing

Status: NEEDS_BUILDING

Build:

- `PH1.IMAGE.GEN`;
- image generation route;
- image editing route;
- generated image packet;
- edited image packet;
- generated-vs-sourced-image separation.

Rules:

- Generated images are creative outputs, not factual source evidence.
- Image editing requires user-provided or approved input image.
- Generated images must not be displayed as real source photos.
- Image generation must obey safety/policy gates.

Proof:

- generation intent route;
- edit intent route;
- generated image not used as evidence;
- sourced image evidence remains separate;
- unsafe generation blocked.

Next if passed:

- Stage 25 - Product Capability Family: Codex / Dev Lane.

## Stage 25 - Product Capability Family: Codex / Dev Lane

Status: NEEDS_BUILDING

Build:

- `PH1.CODEX`;
- `PH1.DEV`;
- repo analysis workflow;
- code review workflow;
- build instruction workflow;
- test execution workflow;
- worktree policy;
- developer tool execution gates;
- clean-tree discipline.

Rules:

- Code tasks route through Codex/Dev lane.
- Repo reads and code edits are separate.
- Tool execution requires allowed path.
- Protected/business execution remains separate.
- No secret leakage.
- No uncontrolled shell/app actions.

Proof:

- repo audit;
- code review;
- build instruction generation;
- safe test command execution;
- dirty tree stop behavior;
- no destructive git unless explicitly requested.

Next if passed:

- Stage 26 - Full Certification Harness.

## Stage 26 - Full Certification Harness

Status: NEEDS_BUILDING

Build:

- full text certification;
- full voice certification;
- full search certification;
- full write certification;
- Desktop certification;
- iPhone certification;
- file/doc/data certification;
- task/agent/connector certification;
- language certification;
- protected execution certification;
- provider-off certification;
- controlled live-provider certification;
- TTS/display hash proof;
- source/image/presentation leak scans.

Rules:

- Certification must prove the whole pipeline, not isolated helper functions only.
- Live provider certification must be opt-in, capped, and provider-off proven first.
- Voice certification must use exact captured transcript when required.
- Protected execution must remain fail-closed.

Proof:

- all targeted tests;
- all full crate tests;
- Desktop build;
- iPhone build where applicable;
- provider-off runtime proof;
- source/image/debug leak scans;
- forbidden real-name scan;
- final clean tree proof.

Next if passed:

- Mark canonical roadmap implementation complete, then continue only with explicitly approved product expansion or bugfix builds.

## Current Next Build Instruction Seed

The next Codex build should be:

```text
TASK: CANONICAL_ENGINE_INVENTORY_AND_WIRING_MAP_REPAIR

Goal:
Create the repo-truth inventory and wiring map that reconciles existing PH1 modules, docs, adapter routes, Desktop/iPhone renderers, old 86-engine references, and the current four-stack master plan.

Do not implement feature behavior yet.
Do not rebuild search.
Do not redesign Desktop.
Do not run live providers.
Do not use Python.

The output must update this document's Build Tracking Rule and clearly state whether Stage 2 is ready to start.
```

## Build Completion Update Rule

At the end of every future build:

1. Update the stage status in this document.
2. Update the `Build Tracking Rule` table.
3. State the next exact stage and build name.
4. If the build discovers this plan is wrong, do not silently drift. Add a `Plan Reconciliation Note` with file/line evidence and JD approval requirement.
5. Update `MASTER_BUILD_COMPLETION_PLAN.md` and `MASTER_BUILD_COMPLETION_LEDGER.md` only when the build instruction requires it.
6. Keep final reports aligned to this plan.

## Plan Reconciliation Notes

- Older 86-engine references are not deleted by this plan. They must be reconciled in Stage 1.
- The four-stack architecture is the current build target unless Stage 1 proves a more accurate repo-truth structure.
- Existing search work is preserved and finished later in Stage 9. It is not rebuilt from zero.
- Stage 9 is evidence-quality closure, not final ChatGPT-style visual presentation. Final user-facing visual quality is completed through Write, presentation contracts, adapter transport, Desktop, and iPhone.
- Desktop visuals are intentionally after presentation contracts and adapter transport.
- Voice proof is intentionally early because bad transcript capture poisons understanding, routing, search, and protected execution.
- Core language governance is intentionally split into early stages: language packet/safety in Stage 5, language-aware routing in Stage 7, Write/TTS continuity in Stage 12, and language transport in Stage 15.
- Stage 18 is advanced language profile and certification work, not the first language-governance build.
