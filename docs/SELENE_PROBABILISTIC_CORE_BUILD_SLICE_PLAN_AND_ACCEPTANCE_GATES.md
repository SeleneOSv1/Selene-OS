# DOC 86 — SELENE_PROBABILISTIC_CORE_BUILD_SLICE_PLAN_AND_ACCEPTANCE_GATES

Status: MASTER_DESIGN / ACCEPTANCE_PLAN / NOT_RUNTIME_IMPLEMENTATION
Project phase: PROBABILISTIC_FOUNDATION_BUILD
Runtime implementation authorized by this document: NO
Purpose: Convert DOC 85 into ordered, real-testable build slices and acceptance gates before implementation.

## 1. Purpose

DOC 85 is Selene's probabilistic core platform North-Star constitution.

DOC 86 is the real build-slice and acceptance-gate plan that turns DOC 85 into a concrete sequence of future implementation work.

DOC 86 does not authorize implementation by itself.

Each later implementation slice requires its own JD-approved Codex instruction, file scope, baseline proof, design reference, code-owner map, test target, local-cloud runtime proof, and real acceptance proof.

No build slice is accepted until JD tests the real app and backend evidence agrees.

DOC 86 exists to prevent:

- implementation before architecture acceptance
- Desktop or iPhone becoming intelligence owners
- provider calls bypassing provider adapters
- terminal-only fake success
- code preservation without proof
- code deletion without proof
- protected execution slipping into probabilistic/public lanes
- moving to later capabilities before the first real runtime loop is proven

## 2. Governing Laws Inherited From DOC 85

DOC 86 inherits these DOC 85 laws as controlling requirements:

1. Real JD Testing Law: no user-visible feature is accepted until JD tests the real app and backend evidence proves the correct engine path.
2. Local-Cloud SaaS Law: the first build may run locally, but it must be cloud-shaped and API/gateway/sync compatible.
3. Thin Desktop/iPhone Client Law: clients capture, render, play, queue, and show status; they do not reason, decide, call providers, own memory truth, or execute authority.
4. OpenAI-First, Not OpenAI-Locked Law: OpenAI is primary where best for launch quality, but provider shapes must remain behind Selene adapters.
5. Provider Independence Law: all provider output normalizes into Selene-owned packets and traces.
6. Packet Registry Law: every engine path must declare inputs, owner, forbidden owner, outputs, schema version, audit trace, and failure mode.
7. Probabilistic vs Deterministic Lane Law: public/advisory work may be probabilistic; protected business execution is deterministic, authority-gated, simulation-gated, and auditable.
8. Monday-Level Voice/Presence Law: Selene must feel fast, natural, useful, sharp, and present while PH1.WRITE, PH1.EMO, PH1.PERSONA, PH1.M, PH1.TTS, and PH1.REALTIME preserve boundaries.
9. Memory Layer Law: memory is layered, scoped, permission-filtered, auditable, and never raw archive dumping.
10. Reality Example Law: every later detailed design section must show JD action, visible behavior, engine path, backend proof, pass condition, and fail condition.
11. Dead-Code Keep-Rewrite-Delete Law: current code is not sacred and must be classified by proof before keep, refactor, rebuild, retire, or delete.
12. Next-Document Selection Law: later work must be selected from DOC 85 first executable build sequence, current repo truth, existing docs, existing runtime code, and real JD testing requirements.

## 3. Build-Slice Acceptance Standard

Every future implementation slice must define:

- slice name
- purpose
- why it comes now
- engines involved
- owner engine for each step
- forbidden owner for each step
- existing docs to inspect first
- existing code to inspect first
- likely code surfaces, if known
- required design document dependency
- required packet contracts
- automated tests required
- local-cloud runtime proof required
- real Desktop proof required
- JD live acceptance required
- backend evidence required
- pass condition
- fail condition
- cleanup required
- accepted status labels

Acceptance labels:

- NOT_STARTED
- DESIGN_REQUIRED
- DESIGN_ACCEPTED
- IMPLEMENTATION_READY
- CODEX_TESTED
- LOCAL_CLOUD_RUNTIME_TESTED
- REAL_DESKTOP_TESTED
- PENDING_JD_LIVE_ACCEPTANCE
- JD_LIVE_ACCEPTANCE_PASSED
- JD_LIVE_ACCEPTANCE_FAILED
- ACCEPTED
- BLOCKED

Only JD_LIVE_ACCEPTANCE_PASSED plus backend evidence agreement allows user-visible ACCEPTED status.

Automated tests alone may never mark a user-visible slice as ACCEPTED.

Terminal-only proof may never mark a user-visible slice as ACCEPTED.

## Universal Runtime Slice Live Acceptance Operator Protocol

This protocol applies to every future runtime implementation slice, including but not limited to:

- Desktop text path
- streaming text
- voice
- realtime
- duplex
- barge-in
- memory/session continuity
- search
- files/docs
- tools
- iPhone
- protected actions
- business-engine runtime slices

This protocol does not apply to docs-only tasks.

### Master Law

Every runtime implementation slice must include live acceptance operation after build.

Codex must not stop at code completion, compile success, automated tests, commit, or push.

Codex must guide JD through live use of the latest app/runtime path, observe or request backend evidence, classify failures, perform narrow in-scope fixes where authorized, rerun tests, and continue until JD confirms the slice works according to the accepted design or Codex reports an out-of-scope blocker.

### Runtime Slice Acceptance States

Required runtime slice acceptance states:

- BUILD_NOT_STARTED
- BUILD_IN_PROGRESS
- BUILD_COMPLETE
- AUTOMATED_PROOF_PASSED
- LIVE_TEST_READY
- JD_LIVE_ACCEPTANCE_PENDING
- JD_LIVE_ACCEPTANCE_PASSED
- JD_LIVE_ACCEPTANCE_FAILED
- BLOCKED_OUT_OF_SCOPE
- ACCEPTED

Rules:

- BUILD_COMPLETE does not mean accepted.
- AUTOMATED_PROOF_PASSED does not mean accepted.
- A runtime slice cannot reach ACCEPTED unless JD_LIVE_ACCEPTANCE_PASSED is true.
- If Codex cannot perform GUI actions directly, Codex must tell JD exactly what to click/type and what evidence to report.
- If JD has not confirmed live success, final status must remain JD_LIVE_ACCEPTANCE_PENDING.

### Codex Live Acceptance Operator Duties

After every runtime build, Codex must:

- start or instruct JD to start the required backend/local-cloud runtime
- confirm required environment variables are present without revealing secrets
- build/open the latest Desktop/iPhone app if applicable
- identify the exact app/version/path being tested
- tell JD exactly what to click/type/say/upload
- provide the exact test prompt or action
- explain expected visible result
- explain expected backend trace/proof
- watch logs/traces where possible
- identify whether the correct runtime path was used
- identify whether forbidden legacy paths were avoided
- classify any failure
- make only narrow in-scope fixes if allowed
- rerun tests after fixes
- update JD with simple step-by-step instructions
- stop only when JD confirms success or an out-of-scope blocker exists

### Required Live Test Script Section

Every future runtime implementation instruction must include:

- exact backend command or discovery requirement
- exact app launch/open steps
- exact user action for JD
- exact expected result
- exact backend trace fields
- exact acceptance gates
- exact failure categories
- exact narrow-fix scope
- exact out-of-scope stop conditions

### Evidence Requirements

Every runtime slice final report must include:

- backend command used
- app/client build status
- app/client launch status if applicable
- exact JD test action
- visible user result
- backend trace/proof
- endpoint/path used
- owner engine sequence used
- forbidden legacy path checks
- provider/tool/execution boundary proof where applicable
- PH1.WRITE final-output proof where applicable
- PH1.X validation proof where applicable
- Access/Authority/Simulation proof where applicable
- tests run
- test results
- manual JD acceptance status
- final acceptance state

### Narrow Fix Loop

If a live test fails inside the approved slice scope, Codex may perform narrow fixes only within the approved file scope, then rerun required tests and live acceptance.

Codex must not broaden scope, rewrite unrelated systems, delete legacy code, or implement future slices to make the current slice pass.

### Stop Conditions

Codex must stop and report a blocker if:

- live acceptance requires files outside approved scope
- runtime secrets would be exposed
- a forbidden legacy path is required
- the slice cannot be tested through the actual app/runtime path
- JD cannot perform required live step and no alternative proof exists
- fixing requires implementing a future slice
- fixing requires broad architecture rewrite
- acceptance trace/proof cannot be produced

### Future Instruction Law

Every future runtime implementation Codex instruction must reference this DOC 86 Universal Runtime Slice Live Acceptance Operator Protocol instead of rewriting the full protocol manually.

If a future instruction omits live acceptance language, DOC 86 still controls and Codex must apply this protocol.

### Codex Final Report Law

Every runtime slice final report must clearly state one of:

- JD_LIVE_ACCEPTANCE_PASSED
- JD_LIVE_ACCEPTANCE_PENDING
- JD_LIVE_ACCEPTANCE_FAILED
- BLOCKED_OUT_OF_SCOPE

Codex must not use vague phrases like:

- "should work"
- "ready for testing"
- "implemented"
- "looks good"

as acceptance status.

### Slice 1 Retrospective Note

Slice 1 established the need for this protocol because implementation and automated proof were completed before JD live acceptance.

Future slices must include live acceptance operation as part of the build workflow.

## Universal Interactive Live Test Observation + Repair Protocol

This protocol applies to every future runtime implementation slice involving:

- Desktop
- iPhone
- mobile client
- web client
- voice UI
- text UI
- rendering
- audio capture/playback
- uploads
- local-cloud runtime testing
- provider-backed responses
- memory/search/docs/tools workflows
- protected action testing
- JD live acceptance

This protocol does not apply to docs-only tasks.

### Master Law

Every runtime live acceptance test must be operated as an interactive observation and repair loop.

Codex must not merely tell JD to test and wait for a vague result.

Codex must:

- prepare the latest app/runtime
- tell JD exactly what to type/say/click/upload
- observe or capture the actual JD input where possible
- observe or capture the visible/audible result where possible
- verify backend trace/proof for that exact action
- compare visible result against expected design
- classify every failure precisely
- repair only narrow in-scope defects where authorized
- rerun automated tests after repair
- rerun live test after repair
- repeat until JD and backend evidence agree, or until an out-of-scope blocker is reported

A slice is not accepted because Codex thinks it probably worked.

A slice is accepted only when:

- JD confirms the visible/audible result is correct
- Codex verifies backend evidence for the same action
- the correct engine path was used
- forbidden legacy paths were not used
- no out-of-scope or unproven behavior remains

### Input/Output Capture Law

For every JD live test, Codex must record or request:

- exact JD input typed/spoken/clicked/uploaded
- timestamp or test run marker
- app/client used
- screen/app state before action, where available
- visible response text, screenshot, audio confirmation, or UI result
- backend request ID
- session ID
- turn ID
- endpoint/path used
- engine sequence used
- output packet/render packet ID
- error/degraded state if any

If Codex has direct GUI/screenshot/app automation access, Codex must use it where lawful and practical.

If Codex cannot directly observe the GUI, Codex must explicitly ask JD for:

- exact visible response text
- screenshot
- exact error message
- exact audible result for voice tests

Codex must not infer visible Desktop/iPhone behavior only from backend success.

### Step-by-Step Operator Duty

For every live test, Codex must guide JD one step at a time.

Codex must say:

- backend/local-cloud runtime is ready or not ready
- latest app is launched or exact manual launch path
- exact prompt/action JD must perform
- expected visible/audible result
- what evidence Codex is watching
- what JD should report back
- whether the step passed or failed
- what the next step is

Codex must not ask JD to "test generally."

Codex must not ask JD to guess what matters.

Codex must not move to the next step until the current step is classified.

### Per-Step PASS/FAIL Classification

Every live test step must be classified as exactly one of:

- STEP_NOT_READY
- STEP_RUNNING
- STEP_PASSED
- STEP_FAILED
- STEP_BLOCKED_OUT_OF_SCOPE
- STEP_REQUIRES_JD_VISIBLE_CONFIRMATION
- STEP_REQUIRES_BACKEND_EVIDENCE
- STEP_REPAIRED_AND_RETEST_REQUIRED

Codex must explain the reason for each failed or blocked step.

### Automated Evidence Correlation

Codex must correlate JD-visible behavior with backend proof.

For every successful step, Codex must prove:

- the visible/audible result came from the expected endpoint/path
- the expected engine sequence ran
- PH1.X ran before provider when required
- PH1.WRITE finalized output when required
- PH1.PROVIDERS was used before provider calls when required
- Desktop/iPhone did not call providers directly
- old legacy paths did not own the turn
- adapter monolith did not execute forbidden provider logic
- trace/proof belongs to the same JD action
- no stale app/runtime instance produced the result

### Failure Diagnosis Matrix

Codex must classify failures into exact categories before fixing.

Standing categories:

- APP_NOT_LAUNCHED
- STALE_APP_UNPROVEN
- WRONG_APP_BUNDLE
- BACKEND_NOT_RUNNING
- CLIENT_BACKEND_CONNECTION_FAILED
- WRONG_ENDPOINT_USED
- LEGACY_ENDPOINT_USED
- PROVIDER_KEY_MISSING
- PROVIDER_CALL_FAILED
- VAULT_READ_FAILED
- PH1X_NOT_RUN
- PH1WRITE_BYPASSED
- PROVIDER_BYPASS_DETECTED
- DESKTOP_DIRECT_PROVIDER_CALL_DETECTED
- ADAPTER_MONOLITH_EXECUTION_DETECTED
- OLD_PH1OS_PATH_USED
- TRACE_MISSING
- TRACE_DOES_NOT_MATCH_VISIBLE_OUTPUT
- UI_RENDER_FAILED
- AUDIO_CAPTURE_FAILED
- AUDIO_PLAYBACK_FAILED
- BARge_IN_NOT_DETECTED
- CANCEL_OUTPUT_FAILED
- MEMORY_RETRIEVAL_UNPROVEN
- SEARCH_EVIDENCE_MISSING
- FILE_UPLOAD_FAILED
- PROTECTED_GATE_BYPASSED
- JD_VISIBLE_RESULT_INCORRECT
- UNKNOWN

UNKNOWN is not acceptable as a final state unless Codex explains what evidence is missing and what would be needed to classify it.

### Narrow Repair Loop Law

If a failure is inside the approved slice scope, Codex may repair narrowly only inside the approved file scope for that slice.

After any repair, Codex must:

- show changed files
- prove no forbidden files changed
- rerun required automated tests
- rebuild latest app if client behavior changed
- relaunch latest app or give JD exact new launch path
- rerun the same JD live test
- verify backend evidence again
- ask JD to confirm the visible/audible result again

Codex must not:

- broaden scope
- implement future slices
- rewrite unrelated systems
- delete legacy code broadly
- hide failures behind mocks/tests
- claim acceptance without retesting the real app

### JD Reconciliation Law

Before a slice can reach JD_LIVE_ACCEPTANCE_PASSED, Codex must explicitly reconcile with JD.

Codex must ask JD a direct confirmation question:

```text
Do you confirm the visible/audible result in the real app matches the expected behavior for this step?
```

JD must answer yes/confirmed or provide the actual visible/audible problem.

If JD confirms and backend evidence agrees, Codex may mark:

```text
JD_LIVE_ACCEPTANCE_PASSED
```

If backend evidence passes but JD has not confirmed, status remains:

```text
JD_LIVE_ACCEPTANCE_PENDING
```

If JD says the result is wrong, status is:

```text
JD_LIVE_ACCEPTANCE_FAILED
```

until repaired and retested.

### Cannot-See-GUI Rule

If Codex cannot directly see the GUI, Codex must not pretend it can.

Codex must state:

```text
GUI_DIRECT_OBSERVATION_UNAVAILABLE
```

Then Codex must rely on:

- backend trace/proof it can inspect
- exact visible response text from JD
- screenshot from JD where possible
- exact error messages from JD
- app logs where available

Codex may not mark the step passed from backend evidence alone if the requirement is user-visible Desktop/iPhone behavior.

### Final Report Law

Every runtime live acceptance final report must include:

- exact JD input/action
- whether Codex directly observed GUI or JD reported result
- visible response text or screenshot status
- backend request ID
- endpoint/path used
- engine sequence
- output/render packet ID
- forbidden path checks
- app provenance proof
- backend provenance proof
- step-by-step PASS/FAIL table
- failures found
- repairs made
- tests rerun
- live retest result
- JD reconciliation statement
- final acceptance state

Final acceptance state must be one of:

- JD_LIVE_ACCEPTANCE_PASSED
- JD_LIVE_ACCEPTANCE_PENDING
- JD_LIVE_ACCEPTANCE_FAILED
- BLOCKED_OUT_OF_SCOPE

Codex must not use vague wording such as:

- "seems to work"
- "should be fine"
- "ready for JD"
- "implemented"
- "likely working"

as acceptance.

### Future Instruction Law

Every future runtime implementation or live acceptance Codex instruction must reference:

- DOC 86 Universal Runtime Slice Live Acceptance Operator Protocol
- DOC 86 Universal Latest Client App Provenance + Launch Readiness Protocol
- DOC 86 Universal Interactive Live Test Observation + Repair Protocol

If a future instruction omits these phrases, DOC 86 still controls and Codex must apply all three protocols.

## 4. First Executable Build Sequence

DOC 86 defines these slices in this order unless JD explicitly overrides later.

### Slice 1 — Local-Cloud Text Conversation Path

Path:

```text
Desktop -> Local Cloud Runtime -> PH1.API / PH1.GATEWAY -> PH1.CONV -> PH1.X -> PH1.PROVIDERS -> PH1.OAI -> PH1.WRITE -> Desktop render -> PH1.OBS trace -> JD live acceptance
```

Purpose: prove the core thin-client SaaS loop.

Why it comes now: every later probabilistic capability depends on one real Desktop-to-local-cloud-to-provider-to-WRITE-to-Desktop path.

Must design: request envelope, client handoff, API/gateway boundary, conversation turn ownership, PH1.X lane classification, provider routing, OpenAI adapter boundary, PH1.WRITE final output, render packet, observability trace, and JD acceptance record.

May build later: minimal local-cloud text endpoint, Desktop submit/render path, provider adapter path, trace capture, and acceptance display.

Engines involved: PH1.CLIENT.MAC, PH1.LOCAL_CLOUD, PH1.API, PH1.GATEWAY, PH1.CONV, PH1.X, PH1.PROVIDERS, PH1.OAI, PH1.WRITE, PH1.OBSERVABILITY.

Owner engine for each step:

- PH1.CLIENT.MAC owns capture and render only.
- PH1.LOCAL_CLOUD owns runtime session boundary.
- PH1.API / PH1.GATEWAY own request admission, envelope validation, and routing.
- PH1.CONV owns conversation turn lifecycle.
- PH1.X owns lane classification and next conversational directive.
- PH1.PROVIDERS owns provider route selection.
- PH1.OAI owns OpenAI adapter call boundary.
- PH1.WRITE owns final wording.
- PH1.OBSERVABILITY owns trace proof.

Forbidden owner for each step:

- Desktop must not reason, call OpenAI, classify lane, or write final wording.
- PH1.OAI must not bypass PH1.PROVIDERS or PH1.WRITE.
- PH1.D must not become authority for execution.
- PH1.WRITE must not claim protected execution.

Existing docs to inspect first: DOC 85, CORE_ARCHITECTURE, SELENE_BUILD_EXECUTION_ORDER, SELENE_AUTHORITATIVE_ENGINE_INVENTORY, Global Human Conversation Spine, PH1.X design, PH1.D design, PH1.WRITE design, Provider-First pivot docs, and Search/Conversation spine docs if turn routing touches them.

Existing code to inspect first: Desktop submit/render code, local runtime/server entry points, API/gateway routes, provider adapter code, OpenAI client wrappers, conversation turn pipeline, PH1.X owner code, PH1.WRITE output path, observability/logging code, and old direct-provider call paths.

Likely code surfaces, if known: `apps/`, `clients/`, `desktop/`, `src/`, `crates/`, `server/`, `services/`, `runtime/`, `providers/`, `openai/`, `observability/`, and any existing Selene Desktop bridge path discovered by repo-truth audit.

Required design document dependency: DOC 85 and DOC 86 accepted; later implementation instruction must also cite the exact slice-specific activation design.

Required packet contracts: TurnRequestPacket, ClientContextPacket, GatewayDecisionPacket, ConversationTurnPacket, LaneClassificationPacket, ProviderRoutePacket, ModelResponsePacket, WriteOutputPacket, RenderPacket, ProviderTracePacket, LatencyPacket.

Automated tests required: request-envelope tests, route-order tests, provider-adapter mock tests, PH1.WRITE not-bypassed tests, no direct Desktop provider-call scan, trace-presence tests, failure-mode tests.

Local-cloud runtime proof required: real local-cloud process receives a Desktop-originated request and records API/gateway, PH1.CONV, PH1.X, provider, PH1.WRITE, render, and trace IDs.

Real Desktop proof required: JD can type in the real Desktop app and see the returned answer rendered by Desktop.

JD live acceptance required: yes.

Backend evidence required: request ID, session ID, turn ID, client ID, lane classification, provider route, PH1.WRITE output packet, render packet ID, latency, and trace path.

Reality example:

```text
JD types: "Write a short apology email to a supplier for late payment."
Expected: Desktop shows a short useful email draft.
```

Pass condition: JD sees the answer in Desktop and backend evidence proves the full path exactly.

Fail condition: Desktop generates the answer locally, Desktop calls OpenAI directly, PH1.WRITE is bypassed, backend trace is missing, or only unit tests pass while real Desktop fails.

Cleanup required: classify and remove or quarantine duplicate direct-provider paths only after replacement proof; retire fake terminal-only demo paths only after JD-approved proof; preserve old code only with KEEP proof.

Accepted status labels: NOT_STARTED -> DESIGN_REQUIRED -> DESIGN_ACCEPTED -> IMPLEMENTATION_READY -> CODEX_TESTED -> LOCAL_CLOUD_RUNTIME_TESTED -> REAL_DESKTOP_TESTED -> PENDING_JD_LIVE_ACCEPTANCE -> JD_LIVE_ACCEPTANCE_PASSED -> ACCEPTED.

### Slice 2 — Streaming Response Path

Path:

```text
Desktop -> Local Cloud Runtime -> PH1.API / PH1.GATEWAY -> PH1.CONV -> PH1.X -> PH1.PROVIDERS -> PH1.OAI -> streaming packets -> PH1.WRITE final packet -> Desktop live render -> PH1.OBS trace -> JD live acceptance
```

Purpose: prove ChatGPT-like visible streaming without making Desktop an output owner.

Why it comes now: streaming is the first UX upgrade after the core text path and must be proven before voice.

Must design: streaming packet schema, chunk provenance, partial/final output distinction, latency tracking, cancellation compatibility, Desktop live render, and final PH1.WRITE closure.

May build later: minimal streaming endpoint, Desktop incremental render, trace events, stream close handling, and stream failure recovery.

Engines involved: PH1.CLIENT.MAC, PH1.LOCAL_CLOUD, PH1.API, PH1.GATEWAY, PH1.CONV, PH1.X, PH1.PROVIDERS, PH1.OAI, PH1.WRITE, PH1.TRANSPORT, PH1.OBSERVABILITY.

Owner engine for each step:

- PH1.TRANSPORT owns stream delivery semantics.
- PH1.OAI owns provider streaming adapter only.
- PH1.WRITE owns final answer boundary and final output packet.
- Desktop owns live render only.

Forbidden owner for each step:

- Desktop must not fake streaming from a final blob without trace.
- Provider adapter must not stream directly to Desktop.
- PH1.WRITE must not disappear from final output.

Existing docs to inspect first: DOC 85, DOC 86 Slice 1 acceptance proof, Full Duplex and Barge-In Enterprise Voice Architecture, PH1.WRITE design, Provider-First function cards, and transport/sync docs.

Existing code to inspect first: streaming handlers, web socket/SSE/IPC bridges, Desktop render loops, provider streaming wrappers, cancellation code, and trace code.

Likely code surfaces, if known: transport modules, gateway stream routes, provider adapter stream code, Desktop message renderer, and observability spans.

Required design document dependency: accepted Slice 1 design and proof; later streaming implementation instruction.

Required packet contracts: StreamingChunkPacket, StreamingStartPacket, StreamingEndPacket, WriteOutputPacket, RenderPatchPacket, LatencyPacket, ProviderTracePacket.

Automated tests required: chunk order tests, final packet tests, trace-presence tests, Desktop no-rewrite tests, stream failure tests, latency recording tests.

Local-cloud runtime proof required: streaming packets originate from local-cloud runtime with request/session/turn IDs.

Real Desktop proof required: Desktop visibly streams the response and closes cleanly.

JD live acceptance required: yes.

Backend evidence required: stream start, chunk IDs, final PH1.WRITE packet, Desktop render IDs, latency timing, and error/degraded-state trace.

Reality example:

```text
JD asks: "Explain why Selene needs simulation before supplier payments. Keep it short."
Expected: Desktop starts showing the answer quickly and finishes cleanly.
```

Pass condition: visible stream plus trace-backed final output.

Fail condition: response appears only after completion when streaming was expected, Desktop creates fake streaming from one final blob without trace, or PH1.WRITE final output is missing.

Cleanup required: classify old stream shims, fake typing effects, direct provider-to-client streams, and stale renderer paths before acceptance.

Accepted status labels: same label ladder as Slice 1; ACCEPTED requires JD_LIVE_ACCEPTANCE_PASSED plus backend evidence.

### Slice 3 — Backend Trace / Observability Proof

Path:

```text
Every request -> PH1.OBSERVABILITY -> trace bundle -> evidence review -> JD/debug visibility
```

Purpose: make proof visible and reliable before voice, search, memory, tools, and protected boundaries add complexity.

Why it comes now: if proof is missing now, later slice claims become theatre.

Must design: trace identity model, trace bundle schema, evidence retention, degraded-state reporting, request/session/turn correlation, and user-visible provenance where safe.

May build later: trace capture, trace viewer/report, per-slice evidence pack, and failure triage output.

Engines involved: PH1.OBSERVABILITY, PH1.LOCAL_CLOUD, PH1.API, PH1.GATEWAY, PH1.CONV, PH1.X, PH1.PROVIDERS, PH1.OAI, PH1.WRITE, PH1.CLIENT.MAC.

Owner engine for each step:

- PH1.OBSERVABILITY owns trace schema, correlation, and evidence bundle.
- Each engine owns its own emitted trace facts.
- Desktop may display provenance but must not fabricate proof.

Forbidden owner for each step:

- Desktop must not be the source of backend truth.
- Provider logs must not replace Selene-owned trace packets.
- PH1.WRITE must not invent proof language.

Existing docs to inspect first: DOC 85, DOC 86 Slices 1-2, CORE_ARCHITECTURE, SELENE_BUILD_EXECUTION_ORDER, Overall Repo-Truth Activation Pack, and existing observability/audit docs.

Existing code to inspect first: logging, tracing, request IDs, session IDs, Desktop render metadata, provider trace wrappers, audit/proof ledgers, and health/status code.

Likely code surfaces, if known: observability modules, logging middleware, trace collectors, Desktop provenance display, API middleware, and provider adapters.

Required design document dependency: accepted Slice 1 design; Slice 3 detailed observability design if repo truth requires it.

Required packet contracts: RequestTracePacket, TurnTracePacket, ProviderTracePacket, RenderTracePacket, LatencyPacket, ErrorTracePacket, DegradedStatePacket.

Automated tests required: required-field trace tests, missing-trace failure tests, correlation tests, degraded-state tests, no-client-fabrication tests.

Local-cloud runtime proof required: every request has request ID, session ID, turn ID, client ID, route decision, lane classification, provider route, PH1.WRITE output packet, Desktop render packet ID, latency timings, and error/degraded-state trace.

Real Desktop proof required: JD-visible answer can be matched to backend trace.

JD live acceptance required: yes for any user-visible proof display; otherwise JD acceptance of evidence pack.

Backend evidence required: trace bundle with all mandatory fields.

Reality example:

```text
JD asks a simple text question. Selene answers. Backend trace shows the exact engine path.
```

Pass condition: JD sees output and backend proof distinguishes Desktop, local-cloud, provider, WRITE, and render steps.

Fail condition: JD sees output but backend proof is missing or ambiguous.

Cleanup required: classify stale logs, duplicate request IDs, old debug-only traces, and any proof path that cannot link to real Desktop render.

Accepted status labels: same label ladder; ACCEPTED requires backend evidence agreement and JD acceptance.

### Slice 4 — OpenAI STT/TTS Voice Path

Path:

```text
Desktop mic -> PH1.REALTIME / PH1.C -> PH1.PROVIDERS / PH1.OAI realtime/STT -> PH1.CONV / PH1.WRITE -> PH1.TTS / OpenAI speech -> Desktop playback -> PH1.OBS trace -> JD live acceptance
```

Purpose: prove real voice path with OpenAI primary while preserving provider portability.

Why it comes now: voice depends on text, streaming, and trace proof.

Must design: microphone capture boundary, realtime/STT route, transcript packet, conversation handoff, PH1.WRITE spoken wording, TTS packet, playback packet, latency, and transcript/audio trace.

May build later: real mic capture, OpenAI realtime/STT/TTS adapter path, Desktop playback, transcript display, voice trace, and failure fallback.

Engines involved: PH1.CLIENT.MAC, PH1.REALTIME, PH1.C, PH1.PROVIDERS, PH1.OAI, PH1.CONV, PH1.X, PH1.WRITE, PH1.TTS, PH1.OBSERVABILITY.

Owner engine for each step:

- Desktop owns microphone capture and playback only.
- PH1.REALTIME owns realtime voice session boundary.
- PH1.C owns transcript quality gate.
- PH1.PROVIDERS / PH1.OAI own provider route and OpenAI adapter.
- PH1.WRITE owns spoken wording.
- PH1.TTS owns speech rendering handoff.

Forbidden owner for each step:

- Desktop must not own STT intelligence, meaning, final wording, or provider calls.
- Apple STT/TTS must not become primary without explicit approval.
- PH1.TTS must not rewrite response meaning.

Existing docs to inspect first: DOC 85, DOC 86 Slices 1-3, Universal Language Intelligence + Voice Capture, Full Duplex and Barge-In, Voice Identity + Human Presence, PH1.WRITE, PH1.C, PH1.TTS, and provider docs.

Existing code to inspect first: microphone capture, audio device permissions, STT/TTS wrappers, realtime provider client code, Desktop playback, transcript handling, PH1.C, PH1.TTS, and voice trace code.

Likely code surfaces, if known: Desktop audio modules, realtime services, provider adapters, TTS modules, transcript models, and observability spans.

Required design document dependency: accepted Slice 1-3; later voice implementation instruction.

Required packet contracts: AudioCapturePacket, RealtimeSessionPacket, TranscriptPacket, TranscriptQualityPacket, ConversationTurnPacket, WriteOutputPacket, SpeechPacket, PlaybackPacket, ProviderTracePacket, LatencyPacket.

Automated tests required: audio packet boundary tests, provider mock tests, transcript quality tests, PH1.WRITE ownership tests, playback packet tests, no Desktop STT/TTS intelligence scan.

Local-cloud runtime proof required: transcript and TTS trace exist from local-cloud/provider path.

Real Desktop proof required: JD speaks through Desktop and hears a response.

JD live acceptance required: yes.

Backend evidence required: audio capture ID, transcript packet, provider route, PH1.WRITE output, TTS packet, playback ID, latency, and error/degraded-state trace.

Reality example:

```text
JD says: "Selene, explain what you can do in one sentence."
Expected: Selene responds by voice, naturally and briefly.
```

Pass condition: JD hears the answer and backend evidence proves provider/STT/TTS/WRITE/playback boundaries.

Fail condition: Apple STT/TTS becomes primary without approval, Desktop decides meaning, Desktop generates voice wording, or no backend transcript/voice trace exists.

Cleanup required: classify old local voice helpers, direct STT/TTS calls, duplicate transcript paths, and non-traced playback code.

Accepted status labels: same label ladder; ACCEPTED requires JD_LIVE_ACCEPTANCE_PASSED plus backend evidence.

### Slice 5 — Duplex / Barge-In / Cancel Output

Path:

```text
Selene speaking -> JD interrupts -> PH1.REALTIME.BARGE_IN -> PH1.REALTIME.CANCEL_OUTPUT -> PH1.CONV state recovery -> PH1.WRITE revised response -> Desktop playback -> PH1.OBS trace -> JD live acceptance
```

Purpose: prove ChatGPT-like realtime voice interruption.

Why it comes now: barge-in requires working voice, streaming, cancellation, trace, and conversation state recovery.

Must design: interruption detection, cancel-output packet, playback stop, conversation state recovery, revised prompt/turn packet, PH1.WRITE revised response, and trace proof.

May build later: interrupt listener, output cancel path, state recovery path, revised response path, and playback restart.

Engines involved: PH1.CLIENT.MAC, PH1.REALTIME, PH1.TTS, PH1.CONV, PH1.X, PH1.WRITE, PH1.OBSERVABILITY.

Owner engine for each step:

- PH1.REALTIME owns barge-in detection and cancel-output event.
- PH1.TTS owns stoppable playback handoff.
- PH1.CONV owns state recovery.
- PH1.WRITE owns revised answer.
- Desktop owns audio stop/playback only.

Forbidden owner for each step:

- Desktop must not semantically interpret "make it shorter."
- Desktop must not rewrite the answer locally.
- PH1.TTS must not decide revised content.

Existing docs to inspect first: DOC 85, DOC 86 Slices 1-4, Full Duplex and Barge-In, PH1.TTS, PH1.WRITE, PH1.CONV, PH1.X.

Existing code to inspect first: audio playback cancellation, realtime session state, interruption handling, conversation state recovery, TTS stop controls, Desktop audio loop, and trace code.

Likely code surfaces, if known: realtime voice manager, TTS playback manager, Desktop audio player, conversation state machine, and observability events.

Required design document dependency: accepted voice path design and proof.

Required packet contracts: BargeInPacket, CancelOutputPacket, ConversationRecoveryPacket, WriteRevisionPacket, PlaybackStopPacket, SpeechPacket, TracePacket.

Automated tests required: cancellation tests, state recovery tests, no local rewrite tests, stale audio tests, revised response route tests, trace tests.

Local-cloud runtime proof required: backend records barge-in, cancel-output, state recovery, PH1.WRITE revision, and playback update.

Real Desktop proof required: JD interrupts while audio is playing and hears the revised answer.

JD live acceptance required: yes.

Backend evidence required: barge-in timestamp, cancel event, original output ID, recovered turn ID, revised PH1.WRITE output ID, playback stop/start IDs.

Reality example:

```text
Selene is speaking. JD says: "Stop. Make it shorter."
Expected: Selene stops and gives a shorter answer.
```

Pass condition: audible stop plus backend cancel/rewrite evidence.

Fail condition: Selene keeps talking, Desktop interprets "make it shorter" locally, or backend barge-in evidence is missing.

Cleanup required: classify old playback-only stop paths, fake interruption handlers, duplicate cancel fields, and untraced audio state.

Accepted status labels: same label ladder; ACCEPTED requires JD_LIVE_ACCEPTANCE_PASSED plus backend evidence.

### Slice 6 — Session Continuity + Local-Cloud Sync

Path:

```text
Desktop session -> Local Cloud Runtime session truth -> PH1.SYNC -> Desktop close/reopen -> session restore -> PH1.OBS trace -> JD live acceptance
```

Purpose: prove cloud/local-cloud truth and device replacement model.

Why it comes now: memory, iPhone, offline, and protected work require durable session truth before expansion.

Must design: server-side session state, client cache boundary, sync status, close/reopen restore, session archive pointer, and conflict-safe reconnect.

May build later: session restore path, Desktop sync status, local cache invalidation, backend archive pointer, and reconnect handling.

Engines involved: PH1.LOCAL_CLOUD, PH1.SYNC, PH1.CLIENT.MAC, PH1.CONV, PH1.M, PH1.OBSERVABILITY.

Owner engine for each step:

- PH1.LOCAL_CLOUD owns session truth.
- PH1.SYNC owns sync state and replay/reconnect posture.
- Desktop owns cache and status display only.
- PH1.M may store eligible memory later, not active session truth.

Forbidden owner for each step:

- Desktop local files must not be the only source of session truth.
- PH1.M must not replace live session lifecycle ownership.
- Sync must not silently overwrite conflicts.

Existing docs to inspect first: CORE_ARCHITECTURE, SELENE_BUILD_EXECUTION_ORDER, DOC 85, DOC 86 Slices 1-5, PH1.M design, session architecture docs, persistence/sync docs.

Existing code to inspect first: session storage, local cache, Desktop persistence, sync queue, reconnect code, archive code, and session IDs.

Likely code surfaces, if known: session service, local store, sync manager, Desktop state restoration, backend persistence, and observability.

Required design document dependency: accepted text/trace path; later session continuity design if repo truth requires it.

Required packet contracts: SessionSnapshotPacket, SyncStatusPacket, RestoreRequestPacket, RestoreResultPacket, ConflictPacket, TracePacket.

Automated tests required: close/reopen restore tests, no-client-truth tests, replay/idempotency tests, conflict tests, trace tests.

Local-cloud runtime proof required: backend session archive exists and restores the conversation after Desktop close/reopen.

Real Desktop proof required: JD closes and reopens Desktop and sees restored session context.

JD live acceptance required: yes.

Backend evidence required: session ID, snapshot ID, restore event, sync status, cache posture, and trace.

Reality example:

```text
JD has a conversation, closes Desktop, reopens Desktop.
Expected: Selene resumes the session from local-cloud truth.
```

Pass condition: session resumes from backend/local-cloud truth with visible sync status.

Fail condition: Desktop local files are the only truth, session cannot restore, or backend session evidence is missing.

Cleanup required: classify old local-only session files, duplicate session stores, stale cache rules, and untraced restore paths.

Accepted status labels: same label ladder; ACCEPTED requires JD_LIVE_ACCEPTANCE_PASSED plus backend evidence.

### Slice 7 — Memory + Long-History Recall

Path:

```text
active session -> recent recall -> full session recall -> session archive -> governed memory -> permission-filtered retrieval -> PH1.WRITE answer -> Desktop render -> PH1.OBS trace -> JD live acceptance
```

Purpose: prove layered memory and long-history recall.

Why it comes now: memory is useful only after session truth and trace proof exist.

Must design: active-session context, recent recall, full session recall, archive lookup, governed permanent memory, private/company/project scopes, permission filtering, source display, forget/hide/delete path.

May build later: retrieval pipeline, memory source chips, archive search, scoped recall, forget/hide/delete controls, and memory trace.

Engines involved: PH1.M, PH1.RETRIEVAL, PH1.CONTEXT, PH1.CONV, PH1.X, PH1.WRITE, PH1.CLIENT.MAC, PH1.OBSERVABILITY.

Owner engine for each step:

- PH1.M owns durable memory truth.
- PH1.RETRIEVAL owns evidence selection.
- PH1.CONTEXT owns bounded context bundle assembly.
- PH1.WRITE owns final wording.
- Desktop renders source/provenance only.

Forbidden owner for each step:

- Desktop must not own memory truth.
- Provider must not receive raw archive dumps.
- PH1.WRITE must not invent memory.
- PH1.X must not bypass permission filtering.

Existing docs to inspect first: DOC 85, PH1.M Human Memory Core, CORE_ARCHITECTURE, memory ledger docs, Global Human Conversation Spine, PH1.WRITE, and access/privacy docs.

Existing code to inspect first: memory store, retrieval code, archive/session search, context assembly, privacy scopes, Desktop memory display, forget/delete paths, and old local memory caches.

Likely code surfaces, if known: memory engine, retrieval service, archive store, context builder, Desktop memory UI, and permission filters.

Required design document dependency: accepted session continuity; later memory implementation instruction.

Required packet contracts: MemoryQueryPacket, MemoryEvidencePacket, RetrievalBundlePacket, ContextBundlePacket, MemoryScopePacket, ForgetHideDeletePacket, WriteOutputPacket, TracePacket.

Automated tests required: scope filter tests, no raw archive dump tests, source evidence tests, forget/hide/delete tests, hallucinated-memory refusal tests, trace tests.

Local-cloud runtime proof required: backend retrieval bundle shows sources and scope filtering.

Real Desktop proof required: JD sees a response grounded in shown memory evidence.

JD live acceptance required: yes.

Backend evidence required: retrieval query, evidence IDs, scope decisions, context bundle, PH1.WRITE output, source/provenance render.

Reality example:

```text
JD says: "Continue the Desktop design from yesterday."
Expected: Selene retrieves the relevant session/memory and continues with evidence.
```

Pass condition: relevant memory is retrieved with evidence and permission filtering.

Fail condition: Selene guesses, raw archive is dumped into a model call, permission filtering is missing, or memory source cannot be shown.

Cleanup required: classify local memory shims, stale summary caches, duplicate retrieval helpers, and ungoverned archive injection paths.

Accepted status labels: same label ladder; ACCEPTED requires JD_LIVE_ACCEPTANCE_PASSED plus backend evidence.

### Slice 8 — Search / Source-Backed Answers

Path:

```text
PH1.SEARCH -> PH1.RESEARCH -> PH1.QUALITY -> PH1.WRITE source chips -> Desktop render -> PH1.OBS trace -> JD live acceptance
```

Purpose: prove source-backed public research.

Why it comes now: search must be source-backed and provider-brokered before broader file/tool work.

Must design: search-need detection, provider routing through PH1.PROVIDERS, source ranking, evidence validation, PH1.RESEARCH synthesis, PH1.QUALITY verification, source chips, and Desktop render.

May build later: search route, evidence bundle, source chip render, current-info refusal/degrade path, and trace.

Engines involved: PH1.SEARCH, PH1.RESEARCH, PH1.QUALITY, PH1.PROVIDERS, PH1.WRITE, PH1.CLIENT.MAC, PH1.OBSERVABILITY.

Owner engine for each step:

- PH1.SEARCH owns query/evidence retrieval assistance.
- PH1.RESEARCH owns synthesis from evidence.
- PH1.QUALITY owns support/claim verification.
- PH1.WRITE owns presentation with source chips.
- Desktop renders only.

Forbidden owner for each step:

- Desktop must not call search providers directly.
- Provider raw JSON must not go straight to user.
- PH1.WRITE must not present unsupported current claims.

Existing docs to inspect first: Search Intelligence Lane, PH1.WRITE, Provider-First docs, DOC 85, DOC 86 prior slices, PH1.SEARCH registry rows, and PH1.QUALITY/RESEARCH related docs if present.

Existing code to inspect first: search wrappers, provider adapters, source ranking, citation/source-chip render, current-info prompts, PH1.WRITE presentation, and direct web/search calls.

Likely code surfaces, if known: search provider adapter, research synthesis, quality checker, Desktop source chip UI, and observability trace.

Required design document dependency: accepted trace path and provider route; later search implementation instruction.

Required packet contracts: SearchRequestPacket, SearchResultPacket, EvidencePacket, ResearchSynthesisPacket, QualityCheckPacket, SourceChipPacket, WriteOutputPacket, TracePacket.

Automated tests required: source support tests, stale source tests, no direct Desktop provider calls, raw JSON block tests, source chip render tests, trace tests.

Local-cloud runtime proof required: search/provider route and source evidence recorded in local-cloud trace.

Real Desktop proof required: Desktop shows source-backed answer and source chips.

JD live acceptance required: yes.

Backend evidence required: query, sources, ranking, evidence bundle, quality result, PH1.WRITE output, source chips, trace.

Reality example:

```text
JD asks for current OpenAI realtime voice capability.
Expected: Selene answers with source-backed summary and source chips.
```

Pass condition: claims are supported and source chips render.

Fail condition: unsupported claims appear, raw provider JSON appears, Desktop calls search provider directly, or source evidence is missing.

Cleanup required: classify old direct search calls, citation formatting hacks, stale source ranking, and unverified answer paths.

Accepted status labels: same label ladder; ACCEPTED requires JD_LIVE_ACCEPTANCE_PASSED plus backend evidence.

### Slice 9 — Files / Docs / Data Analysis

Path:

```text
Desktop file picker -> Local Cloud Runtime object/evidence store -> PH1.DOCS / PH1.DATA_ANALYSIS -> PH1.WRITE artifact answer -> Desktop render/export -> PH1.OBS trace -> JD live acceptance
```

Purpose: prove ChatGPT-like file, document, and data workflow.

Why it comes now: file/data work requires text, trace, session, memory boundaries, and source/evidence discipline.

Must design: upload handoff, object/evidence storage, document parsing, table extraction, data analysis, artifact output, evidence packet, export handoff, and Desktop render.

May build later: file upload, parser route, data/table analysis, artifact response, export, and trace.

Engines involved: PH1.CLIENT.MAC, PH1.LOCAL_CLOUD, PH1.DOCS, PH1.DATA_ANALYSIS, PH1.PROVIDERS, PH1.WRITE, PH1.OBSERVABILITY.

Owner engine for each step:

- Desktop owns file picker and render/export only.
- Local-cloud owns file/object/evidence storage.
- PH1.DOCS owns document evidence extraction.
- PH1.DATA_ANALYSIS owns table/data analysis.
- PH1.WRITE owns final explanation/artifact wording.

Forbidden owner for each step:

- Desktop must not parse authoritative document truth.
- Provider must not receive files outside approved evidence packets.
- Tool output must not bypass PH1.WRITE.

Existing docs to inspect first: DOC 85, DOC 86 prior slices, PH1.DOC, PH1.SUMMARY, PH1.MULTI, PH1.WRITE, data/artifact docs, and access/privacy docs.

Existing code to inspect first: file picker, upload route, object store, parsers, spreadsheet/data analysis helpers, document readers, artifact/export code, and Desktop render.

Likely code surfaces, if known: Desktop upload UI, backend file service, document parser, data analysis service, artifact writer, export code, and trace.

Required design document dependency: accepted local-cloud text/trace/session path; later files/data implementation instruction.

Required packet contracts: FileUploadPacket, FileEvidencePacket, DocumentParsePacket, TableExtractionPacket, DataAnalysisPacket, ArtifactPacket, ExportHandoffPacket, WriteOutputPacket, TracePacket.

Automated tests required: upload validation tests, evidence packet tests, parser mock tests, table extraction tests, no Desktop authority tests, artifact output tests, trace tests.

Local-cloud runtime proof required: uploaded file is handled through local-cloud/object/evidence store and analyzed by owner engine.

Real Desktop proof required: JD uploads a file and sees analysis/artifact in Desktop.

JD live acceptance required: yes.

Backend evidence required: file ID, storage evidence, parser result, analysis packet, PH1.WRITE output, artifact/export ID, trace.

Reality example:

```text
JD uploads a small spreadsheet and asks: "Summarize this and show the key numbers."
Expected: Selene analyzes the file and returns summary/artifact through Desktop.
```

Pass condition: analysis is grounded in the uploaded file and rendered in Desktop.

Fail condition: Desktop parses authoritative document truth, file is not stored in local-cloud/object store, or PH1.DOCS / PH1.DATA_ANALYSIS evidence is missing.

Cleanup required: classify local parser shortcuts, stale upload code, duplicate artifact paths, and untraced export helpers.

Accepted status labels: same label ladder; ACCEPTED requires JD_LIVE_ACCEPTANCE_PASSED plus backend evidence.

### Slice 10 — Tool Broker / Connectors / Jobs

Path:

```text
PH1.X tool need -> PH1.TOOLS broker -> connector/job boundary -> normalized tool result -> PH1.WRITE report/draft -> Desktop render -> PH1.OBS trace -> JD live acceptance
```

Purpose: prove tools are brokered safely before protected execution.

Why it comes now: tools/connectors/jobs increase capability and risk; broker boundaries must exist before protected business action.

Must design: read-only tool broker, function-call broker, background job queue, connector boundary, tool result normalization, policy/degrade posture, and tool audit trace.

May build later: read-only tool call path, connector normalization, background job status, result packet, report presentation, and trace.

Engines involved: PH1.X, PH1.TOOLS, PH1.PROVIDERS, PH1.WRITE, PH1.OBSERVABILITY, PH1.CLIENT.MAC.

Owner engine for each step:

- PH1.X owns tool-needed classification.
- PH1.TOOLS owns broker boundary and tool result normalization.
- Background job owner owns job lifecycle.
- PH1.WRITE owns presentation.
- Desktop renders only.

Forbidden owner for each step:

- Desktop must not call tools directly.
- Tool result must not go directly to user without PH1.WRITE.
- Tool broker must not execute protected action without protected boundary.

Existing docs to inspect first: DOC 85, DOC 86 prior slices, PH1.E/tool docs, connector docs, provider docs, PH1.WRITE, access/authority docs.

Existing code to inspect first: tool invocation paths, connector wrappers, job queue, result normalization, policy checks, Desktop tool UI, and old direct tool calls.

Likely code surfaces, if known: tool broker, connector adapters, job queue, result normalizer, Desktop status panel, and observability.

Required design document dependency: accepted trace/search/file boundaries; later tool implementation instruction.

Required packet contracts: ToolRequestPacket, ToolPolicyPacket, ToolCallPacket, ToolResultPacket, JobStatusPacket, ConnectorTracePacket, WriteOutputPacket, TracePacket.

Automated tests required: broker route tests, read-only permission tests, no Desktop direct tool calls, result normalization tests, job lifecycle tests, trace tests.

Local-cloud runtime proof required: local-cloud broker invokes allowed tool/connector and records normalized result.

Real Desktop proof required: JD sees a drafted/read-only report result through Desktop.

JD live acceptance required: yes for user-visible workflows.

Backend evidence required: tool request, policy posture, tool call ID, connector result, normalized result, PH1.WRITE output, trace.

Reality example:

```text
JD asks Selene to prepare a read-only report using allowed data.
Expected: Tool call is brokered, result normalized, PH1.WRITE presents report draft.
```

Pass condition: tool broker path is traced and result is presented through PH1.WRITE.

Fail condition: tool is called directly from Desktop, tool bypasses policy, or tool result goes directly to user without PH1.WRITE.

Cleanup required: classify direct connector calls, duplicate job runners, unnormalized tool outputs, and stale tool UI routes.

Accepted status labels: same label ladder; ACCEPTED requires JD_LIVE_ACCEPTANCE_PASSED plus backend evidence where user-visible.

### Slice 11 — Protected Boundary

Path:

```text
PH1.X -> PH1.ACCESS -> PH1.AUTHORITY -> PH1.SIMULATION -> PH1.AUDIT -> deterministic business owner -> PH1.WRITE explanation -> Desktop render -> JD live acceptance
```

Purpose: prove protected execution fails closed.

Why it comes now: protected/business execution must be fenced after public/tool capability exists and before any real protected automation.

Must design: protected intent classification, access check, authority check, simulation requirement, audit record, deterministic owner handoff, refusal/explanation wording, and fail-closed proof.

May build later: protected-route classifier, fail-closed response, simulation-required posture, audit proof, and deterministic owner stub/handoff only under later approved scope.

Engines involved: PH1.X, PH1.ACCESS, PH1.AUTHORITY, PH1.SIMULATION, PH1.AUDIT, deterministic business owner, PH1.WRITE, PH1.OBSERVABILITY, PH1.CLIENT.MAC.

Owner engine for each step:

- PH1.X owns protected classification.
- PH1.ACCESS owns access decision.
- PH1.AUTHORITY owns authority decision.
- PH1.SIMULATION owns simulation gate.
- PH1.AUDIT owns audit proof.
- Deterministic business owner owns domain execution only after all gates pass.
- PH1.WRITE owns explanation.

Forbidden owner for each step:

- PH1.D must not approve action.
- PH1.WRITE must not claim execution without proof.
- Public lane must not execute protected action.
- Desktop must not mutate business state.

Existing docs to inspect first: Identity + Access + Authority Spine, Global Request Decision Lattice, SELENE_AUTHORITATIVE_ENGINE_INVENTORY, simulation catalog docs, audit docs, finance/supplier payment docs if payment example is used, DOC 85, DOC 86 prior slices.

Existing code to inspect first: PH1.X classification, access checks, authority checks, simulation engine, audit ledger, business owner stubs, refusal wording, and any direct mutation path.

Likely code surfaces, if known: access/authority modules, simulation catalog, audit ledger, business engine handlers, Desktop action buttons, and observability.

Required design document dependency: accepted tool broker and trace; later protected boundary implementation instruction.

Required packet contracts: ProtectedActionCandidatePacket, AccessDecisionPacket, AuthorityDecisionPacket, SimulationRequirementPacket, SimulationResultPacket, AuditRecordPacket, BusinessOwnerHandoffPacket, WriteOutputPacket, TracePacket.

Automated tests required: no-simulation fail-closed tests, no-authority fail-closed tests, PH1.D cannot approve tests, PH1.WRITE no-fake-execution tests, public lane block tests, audit tests.

Local-cloud runtime proof required: protected request fails closed without simulation/authority and records audit trace.

Real Desktop proof required: JD sees refusal/requirement explanation in Desktop.

JD live acceptance required: yes.

Backend evidence required: protected classification, access decision, authority decision, simulation posture, audit record, PH1.WRITE explanation, trace.

Reality example:

```text
JD says: "Approve this supplier payment."
Expected: Selene identifies protected action. If simulation/authority is missing, no execution occurs. PH1.WRITE explains what is required.
```

Pass condition: protected route fails closed without unauthorized mutation and evidence proves why.

Fail condition: any payment/action executes without simulation, PH1.D approves action, PH1.WRITE claims execution without proof, or protected action goes through public lane.

Cleanup required: classify old direct mutation paths, fake approval wording, missing audit branches, bypassed simulation calls, and stale protected UI affordances.

Accepted status labels: same label ladder; ACCEPTED requires JD_LIVE_ACCEPTANCE_PASSED plus backend evidence and no protected mutation without lawful gates.

### Slice 12 — Offline Queue + Reconnect Sync

Path:

```text
Desktop offline status -> safe local queue -> reconnect -> PH1.SYNC replay/reconcile -> Local Cloud Runtime truth -> Desktop status -> PH1.OBS trace -> JD live acceptance
```

Purpose: prove SaaS device model during unreliable connectivity.

Why it comes now: offline/reconnect must preserve thin-client boundaries before iPhone and broader device support.

Must design: offline banner/status, safe local queue, protected/cloud-required wait rules, reconnect replay, idempotency, conflict resolution, and backend source-of-truth restoration.

May build later: offline banner, draft queue, reconnect replay, conflict display, backend reconciliation, and trace.

Engines involved: PH1.CLIENT.MAC, PH1.SYNC, PH1.LOCAL_CLOUD, PH1.GATEWAY, PH1.CONV, PH1.X, PH1.OBSERVABILITY.

Owner engine for each step:

- Desktop owns offline display and safe queue only.
- PH1.SYNC owns replay, reconciliation, idempotency, and conflict posture.
- Local-cloud owns authoritative state after reconnect.
- PH1.X classifies queued instruction lane after reconnect.

Forbidden owner for each step:

- Desktop must not pretend full cloud intelligence is available offline.
- Protected action must not execute offline.
- Conflicts must not be silently overwritten.

Existing docs to inspect first: CORE_ARCHITECTURE, DOC 85, DOC 86 session/sync slices, persistence/sync docs, access/protected boundary docs.

Existing code to inspect first: offline detection, local queue, retry/replay, conflict handling, session restore, Desktop status UI, and protected action blocks.

Likely code surfaces, if known: sync manager, local queue store, network status, gateway replay handling, conflict resolver, Desktop banners, and trace.

Required design document dependency: accepted session continuity and protected boundary.

Required packet contracts: OfflineQueuePacket, ReconnectReplayPacket, IdempotencyKeyPacket, ConflictResolutionPacket, SyncStatusPacket, TracePacket.

Automated tests required: offline queue tests, protected offline block tests, replay idempotency tests, conflict tests, reconnect trace tests, no local intelligence tests.

Local-cloud runtime proof required: queued allowed work replays after reconnect and backend becomes source of truth.

Real Desktop proof required: JD sees offline status, reconnect status, and safe sync result.

JD live acceptance required: yes.

Backend evidence required: queued item IDs, replay IDs, conflict posture, sync status, trace.

Reality example:

```text
JD goes offline, writes a draft instruction, reconnects.
Expected: Selene syncs allowed queued work.
```

Pass condition: allowed queued work syncs safely and protected/cloud-required work waits.

Fail condition: offline mode pretends full cloud intelligence is available, protected action is executed offline, or conflicts are silently overwritten.

Cleanup required: classify local offline intelligence paths, stale queue formats, direct offline mutation code, and missing conflict UI.

Accepted status labels: same label ladder; ACCEPTED requires JD_LIVE_ACCEPTANCE_PASSED plus backend evidence.

### Slice 13 — iPhone Thin Client

Path:

```text
iPhone client -> PH1.API / PH1.GATEWAY -> Local Cloud Runtime -> shared session truth -> owner engines -> iPhone render/playback -> PH1.OBS trace -> JD live acceptance
```

Purpose: bring iPhone after Desktop proves the core loop.

Why it comes now: iPhone must inherit the same cloud/local-cloud truth instead of inventing a parallel mobile architecture.

Must design: activation route, audio capture/playback, biometric/passcode step-up, notifications, file/photo picker, render-only UI, sync with local-cloud/cloud, shared session, and no intelligence ownership.

May build later: iPhone activation, submit/render, audio capture/playback, sync status, biometric/passcode step-up, notifications, file/photo picker, and trace.

Engines involved: PH1.CLIENT.IOS, PH1.API, PH1.GATEWAY, PH1.LOCAL_CLOUD, PH1.SYNC, PH1.CONV, PH1.X, PH1.PROVIDERS, PH1.WRITE, PH1.TTS, PH1.OBSERVABILITY.

Owner engine for each step:

- PH1.CLIENT.IOS owns capture, playback, render, picker, notifications, and step-up UX only.
- PH1.API / PH1.GATEWAY own request admission.
- PH1.LOCAL_CLOUD owns shared session truth.
- PH1.SYNC owns cross-device consistency.
- Existing owner engines keep the same ownership as Desktop slices.

Forbidden owner for each step:

- iPhone must not own STT/TTS intelligence without explicit approval.
- iPhone must not own separate memory truth.
- iPhone must not bypass PH1.API / PH1.GATEWAY.
- iPhone must not execute protected action locally.

Existing docs to inspect first: CORE_ARCHITECTURE, DOC 85, DOC 86 prior slices, Desktop proof packs, iPhone/mobile client docs if present, access/biometric docs, sync docs.

Existing code to inspect first: iOS client code, activation route, mobile audio capture/playback, mobile sync, notifications, file/photo picker, mobile auth/biometric step-up, and shared session code.

Likely code surfaces, if known: iOS app, mobile bridge, activation API, notification service, mobile sync manager, audio modules, and trace.

Required design document dependency: accepted Desktop text/voice/session/sync/protected boundaries.

Required packet contracts: MobileActivationPacket, MobileClientContextPacket, SyncStatusPacket, BiometricStepUpPacket, NotificationPacket, AudioCapturePacket, PlaybackPacket, RenderPacket, TracePacket.

Automated tests required: activation tests, no mobile provider-call tests, shared-session tests, sync consistency tests, biometric step-up tests, render-only tests, trace tests.

Local-cloud runtime proof required: iPhone connects to same local-cloud/cloud truth and shares session state.

Real Desktop proof required: Desktop/iPhone consistency must be visible where cross-device session is tested.

Real iPhone proof required: JD asks the same simple question from iPhone after Desktop session exists.

JD live acceptance required: yes.

Backend evidence required: mobile client ID, shared session ID, route path, owner engine path, sync status, render/playback IDs, trace.

Reality example:

```text
JD asks the same simple question from iPhone after Desktop session exists.
Expected: iPhone connects to same cloud/local-cloud truth and renders response.
```

Pass condition: iPhone behaves as a thin terminal into the same session truth.

Fail condition: iPhone owns STT/TTS intelligence without approval, iPhone has separate memory truth, or sync mismatch occurs.

Cleanup required: classify mobile-only logic, duplicate session truth, direct mobile provider calls, and untraced mobile playback/render paths.

Accepted status labels: same label ladder; ACCEPTED requires JD_LIVE_ACCEPTANCE_PASSED plus backend evidence.

## 5. Slice Dependency Matrix

| Slice # | Slice Name | Depends On | Required Prior Design Docs | Implementation May Begin Only After | JD Live Proof Required |
|---:|---|---|---|---|---|
| 1 | Local-Cloud Text Conversation Path | DOC 85, DOC 86 | DOC 85, DOC 86 | JD-approved implementation instruction + repo-truth audit | Yes |
| 2 | Streaming Response Path | Slice 1 | DOC 85, DOC 86, Slice 1 proof | Slice 1 accepted or JD explicitly scopes a controlled parallel proof | Yes |
| 3 | Backend Trace / Observability Proof | Slice 1, Slice 2 where streaming applies | DOC 85, DOC 86, observability docs | Trace design accepted | Yes for visible proof |
| 4 | OpenAI STT/TTS Voice Path | Slices 1-3 | DOC 85, DOC 86, voice/language docs | Text, streaming, and trace proof accepted | Yes |
| 5 | Duplex / Barge-In / Cancel Output | Slices 1-4 | DOC 85, DOC 86, barge-in/voice docs | Voice path accepted | Yes |
| 6 | Session Continuity + Local-Cloud Sync | Slices 1-3 | DOC 85, DOC 86, session/sync docs | Text and trace path accepted | Yes |
| 7 | Memory + Long-History Recall | Slices 1, 3, 6 | DOC 85, DOC 86, PH1.M docs | Session continuity accepted | Yes |
| 8 | Search / Source-Backed Answers | Slices 1-3 | DOC 85, DOC 86, search/write/provider docs | Provider and trace path accepted | Yes |
| 9 | Files / Docs / Data Analysis | Slices 1, 3, 6, 8 | DOC 85, DOC 86, docs/data designs | File/evidence design accepted | Yes |
| 10 | Tool Broker / Connectors / Jobs | Slices 1, 3, 8, 9 | DOC 85, DOC 86, tool/connector docs | Tool broker design accepted | Yes for visible workflows |
| 11 | Protected Boundary | Slices 1, 3, 10 | DOC 85, DOC 86, access/authority/simulation/audit docs | Protected boundary design accepted | Yes |
| 12 | Offline Queue + Reconnect Sync | Slices 1, 3, 6, 11 | DOC 85, DOC 86, sync/protected docs | Sync/protected rules accepted | Yes |
| 13 | iPhone Thin Client | Slices 1-6, 11-12 | DOC 85, DOC 86, client/mobile/sync docs | Desktop core loop accepted | Yes |

## 6. Engine Ownership Matrix

| Engine | Owns | Must Not Own | First Slice Where It Appears | Proof Required |
|---|---|---|---:|---|
| PH1.OS | top-level orchestration gate and legal next-move posture | provider calls, client rendering, protected execution bypass | 1 | orchestration trace when implementation uses OS gate |
| PH1.LOCAL_CLOUD | cloud-shaped local runtime, session boundary, backend truth | Desktop UI behavior or provider-specific logic | 1 | local-cloud receives request and emits trace |
| PH1.API | request admission and API contract boundary | reasoning, provider selection, final wording | 1 | request envelope validation trace |
| PH1.GATEWAY | routing, gateway policy, transport handoff | semantic rewriting or business action | 1 | gateway decision packet |
| PH1.TRANSPORT | stream/reconnect delivery semantics | content meaning or final wording | 2 | streaming/replay packet trace |
| PH1.SYNC | sync state, replay, reconnect, conflict posture | session truth, memory truth, protected execution | 6 | sync status and replay trace |
| PH1.CLIENT.MAC | Desktop capture, render, playback, picker, status | reasoning, STT/TTS intelligence, provider calls, memory truth, authority | 1 | no direct provider/authority path plus real render proof |
| PH1.CONV | conversation turn lifecycle and state recovery | provider adapter calls or business execution | 1 | conversation turn packet |
| PH1.X | lane classification and next conversational directive | final wording, simulation execution, provider calls | 1 | lane classification packet |
| PH1.PROVIDERS | provider routing and provider abstraction | final answer wording or direct client rendering | 1 | provider route packet |
| PH1.OAI | OpenAI adapter call boundary | route selection outside PH1.PROVIDERS or client call path | 1 | provider trace packet |
| PH1.WRITE | final user-facing wording and presentation | authority, execution, fake proof, provider routing | 1 | WriteOutputPacket ID in trace |
| PH1.OBSERVABILITY | trace schema, proof bundle, evidence correlation | user-facing semantics or business execution | 1 | complete trace bundle |
| PH1.REALTIME | realtime voice session, barge-in, cancel output | final meaning or protected execution | 4 | realtime/cancel trace |
| PH1.TTS | speech rendering handoff and playback safety | response meaning or authority | 4 | speech/playback packet |
| PH1.M | governed memory truth | live session lifecycle, raw archive dump, client cache | 7 | memory evidence packet |
| PH1.RETRIEVAL | permission-filtered evidence selection | final wording or unbounded archive injection | 7 | retrieval bundle |
| PH1.SEARCH | search query/evidence assist | direct client search or unsupported final claims | 8 | source evidence packet |
| PH1.RESEARCH | evidence-backed synthesis | source retrieval authority without evidence | 8 | research synthesis packet |
| PH1.QUALITY | claim/source support verification | final presentation without PH1.WRITE | 8 | quality check packet |
| PH1.DOCS | document evidence extraction | Desktop authoritative parsing or business mutation | 9 | document parse evidence |
| PH1.DATA_ANALYSIS | table/data analysis | file storage truth or final wording bypass | 9 | data analysis packet |
| PH1.TOOLS | tool broker, result normalization, job/connector boundary | direct protected execution or final presentation | 10 | tool result normalization trace |
| PH1.ACCESS | access decision | final wording or simulation result | 11 | access decision packet |
| PH1.AUTHORITY | role/authority decision | probabilistic approval or client-local authority | 11 | authority decision packet |
| PH1.SIMULATION | simulation requirement and simulation result gate | public chat or provider calls | 11 | simulation posture/result packet |
| PH1.AUDIT | tamper-evident protected action proof | action approval or client UI | 11 | audit record |
| PH1.CLIENT.IOS | iPhone capture, render, playback, notifications, picker, step-up UX | reasoning, memory truth, STT/TTS intelligence without approval, protected execution | 13 | mobile route trace and shared session proof |

## 7. Existing Code Review Rule Before Each Slice

Before implementation of any slice, Codex must perform a slice-specific repo-truth audit and classify current code as:

- KEEP_AS_IS
- KEEP_WITH_MINOR_FIX
- REFACTOR
- REWRITE_IN_PLACE
- REBUILD_FROM_SCRATCH
- RETIRE_AFTER_REPLACEMENT
- DELETE_NOW_IF_SAFE
- UNKNOWN_REQUIRES_TEST

Old code is not preserved because it exists.

Old code is not deleted blindly.

Every keep/delete decision must be based on DOC 85, DOC 86, repo truth, and real path proof.

Each implementation instruction must list:

- current code surfaces inspected
- current owners found
- conflicting owners found
- direct provider/client bypasses found
- stale helper paths found
- duplicate runtime paths found
- exact classification and reason
- replacement proof needed before retirement

## 8. Required Proof Pack for Every Future Implementation Slice

Each implementation slice must produce:

- lane declaration
- file-scope declaration
- existing capability reuse proof
- correct owner map
- baseline proof
- targeted tests
- local-cloud runtime proof
- real Desktop/iPhone proof where applicable
- JD live acceptance status
- backend evidence references
- dead/duplicate cleanup proof
- no runtime/client authority drift proof
- final clean tree proof

The proof pack must explicitly state whether ACCEPTED is allowed.

If JD_LIVE_ACCEPTANCE_PASSED is absent for a user-visible slice, the slice may be CODEX_TESTED, LOCAL_CLOUD_RUNTIME_TESTED, REAL_DESKTOP_TESTED, or PENDING_JD_LIVE_ACCEPTANCE, but it must not be ACCEPTED.

## 9. No-Implementation Warning

DOC 86 does not authorize implementation.

DOC 86 authorizes only future planning and selection of the correct implementation slice.

Every implementation requires a separate JD-approved Codex instruction.

This document does not authorize:

- runtime code edits
- Desktop rebuild
- iPhone build
- local-cloud implementation
- provider rewiring
- packet struct creation
- API creation
- migrations
- test harness changes
- app builds
- xcodebuild
- cargo build/test
- live provider calls
- protected business execution
- code cleanup
- file deletion

## 10. Current Repo-Truth Context

- DOC 85 is accepted and registered.
- DOC 85 controls next-document selection.
- DOC 86 is selected because DOC 85's priority list names Build Slice Plan and Acceptance Gates first.
- Local `main` may be ahead of `origin/main` if DOC 85 commits are not pushed; do not claim remote equality unless same-run fetch proves it.
- DOC 86 is a master design / acceptance plan only.
- DOC 86 adds no runtime engine row by itself.
- DOC 86 does not change current runtime reachability claims.
- DOC 86 does not prove any slice implemented.

## 11. Index / Registry Update

Required registration:

- global document number: 86
- title: SELENE_PROBABILISTIC_CORE_BUILD_SLICE_PLAN_AND_ACCEPTANCE_GATES
- status: MASTER_DESIGN / ACCEPTANCE_PLAN / NOT_RUNTIME_IMPLEMENTATION
- phase: PROBABILISTIC_FOUNDATION_BUILD
- implementation authorized: no
- depends on: DOC 85
- next action: audit DOC 86 acceptance before DOC 87

DOC 86 should be registered in `docs/SELENE_MASTER_ARCHITECTURE_BUILD_SET.md`.

DOC 86 does not require a runtime engine inventory row because it creates no engine and authorizes no implementation.

DOC 86 does not require a build execution order edit because it does not change the runtime build order; it defines acceptance gates for future probabilistic foundation slices.

## 12. Acceptance Summary

DOC 86 is acceptance-clean only if an audit proves:

- the title and status block are present
- all inherited DOC 85 laws are summarized
- the build-slice acceptance standard is present
- all 13 slices are present in order
- Slice 1 path is present exactly
- each slice states purpose, owner engines, forbidden owners, proof requirements, pass/fail, and cleanup requirements
- the dependency matrix is present
- the engine ownership matrix is present
- code review classification rules are present
- proof pack rules are present
- No-Implementation Warning is present
- index registration is present
- no runtime file is changed by this authoring run

If any of those are missing, DOC 86 must be patched before DOC 87 selection or implementation planning proceeds.
