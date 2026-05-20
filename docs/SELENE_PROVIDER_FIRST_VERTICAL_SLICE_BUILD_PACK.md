Selene Provider-First Vertical Slice Build Pack

DOCUMENT TYPE:
VERTICAL SLICE BUILD PACK

CONTROLLING DOCUMENTS:
1. Selene Provider-First OpenAI Assisted Pivot Master Build Plan
2. Selene Provider-First Function Architecture Cards
3. AGENTS.md

PURPOSE:
Define the first executable vertical slice for each major Selene function so Codex can build one narrow, provable slice at a time inside existing canonical engines.

0. Controlling Rule

This document does not replace the master build plan or the function architecture cards.

It exists to answer:

What is the first smallest useful thing Codex should build for each function?
What must be tested?
What must JD test live?
What backend evidence must prove it worked?
What old path may or may not be removed?
What is the next slice after it passes?

The controlling approach remains:

1. One global provider-first architecture.
2. One architecture card per major function.
3. One vertical slice per function.
4. Live proof and backend evidence.
5. Remove old paths only after proof.
6. Repeat until every canonical engine is clean.

0A. AGENTS Law Requirement

Before Codex executes any slice in this pack, Codex must read:

AGENTS.md
docs/CORE_ARCHITECTURE.md
docs/SELENE_BUILD_EXECUTION_ORDER.md
docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md
relevant section docs for the specific function

If Codex has not read AGENTS.md in the current run, Codex must stop with:

AGENTS_LAW_NOT_READ_FOR_CURRENT_RUN

No slice may override AGENTS law.

0B. Slice Execution Rule

Each slice must be treated as a separate implementation run unless JD explicitly approves batching.

Each slice must include:

lane declaration
canonical owner
existing-owner discovery
file-scope approval
provider coverage classification
build-specific test plan
backend evidence requirement
JD live test where user-visible
provider-off test where relevant
fake-provider test where relevant
old-path classification
old-path removal only after proof
commit/push proof
final clean tree proof

Codex must not implement future slices early.

0C. Acceptance Rule

Cargo is a safety gate.

Backend evidence proves the route.

JD live testing is product acceptance for user-visible/runtime builds.

A user-visible/runtime slice is not passed until:

JD live behavior is correct
backend evidence proves the correct owner handled it
visible/audible result matches expected behavior
old behavior regression tests pass
final tree is clean

If cargo passes but JD live testing fails:

JD_LIVE_ACCEPTANCE_FAILED

Codex must repair the correct owner and retest.

0D. Smoke Test Definition and Acceptance Hierarchy

A smoke test is a quick real-path proof that the changed path is alive and not obviously broken.

A smoke test is not final product acceptance.

Acceptance hierarchy:

Cargo / unit / mocked / fake-provider / xcodebuild tests
= safety gates

Codex smoke test
= first real-path check

Backend evidence
= proof the correct owner handled the route

JD live test
= product acceptance for user-visible/runtime behavior

For Selene, a user-visible/runtime slice only passes when:

JD live behavior is correct
backend evidence proves the correct canonical owner handled it
visible/audible result matches expected behavior
old working behavior still passes
provider-off/fake-provider checks pass where relevant
final tree is clean

Codex must not report a user-visible/runtime slice as complete only because:

cargo passed
unit tests passed
mocked tests passed
fake-provider tests passed
xcodebuild passed
endpoint test passed
Codex smoke passed
/healthz passed
app launched

Those are required gates, but not product acceptance.

If cargo passes but JD live testing fails, the slice status is:

JD_LIVE_ACCEPTANCE_FAILED

If visible behavior appears correct but backend evidence is missing or points to the wrong owner, the slice status is:

BACKEND_EVIDENCE_VERIFICATION_FAILED

Final rule:

Smoke proves the path is alive.
Backend evidence proves the owner route.
JD live testing proves the product works.

0E. Root Owner Algorithmic Repair Rule

Codex must not patch symptoms.

Codex must not fix behavior in the nearest visible file unless that file is proven to be the canonical owner of the behavior.

Forbidden repair patterns:

nearest-visible-file patch
Adapter shortcut
Desktop workaround
phrase-specific production branch
example-specific logic
hardcoded JD prompt behavior
one-city / one-person / one-topic logic
parallel route
parallel brain
helper shim that bypasses the owner
fixture-driven production behavior
provider proposal blindly trusted

Required repair process:

1. Identify the observed failure.
2. Identify the expected behavior.
3. Identify the root canonical owner.
4. Prove why the owner owns the behavior.
5. Repair only at the owner level.
6. Use real architecture logic.
7. Prove the repaired route with backend evidence.
8. Re-run JD live test where user-visible.
9. Re-run old behavior regression tests.

Allowed implementation patterns:

real algorithms
canonical packets
state machines
candidate generation
candidate scoring
hard disqualifiers
validation layers
rejection ledgers
owner directives
evidence packets
provider governance
policy gates
source verification
memory scope checks
protected simulation gates

Example of wrong repair:

User says: “make it shorter”
Adapter checks contains("shorter")
Adapter sends shortcut rewrite response

Example of correct repair:

User says: “make it shorter”
SemanticInterpreterProvider proposes rewrite operation
PH1.X validates target and current active frame
PH1.X emits HumanConversationDirective
PH1.WRITE performs rewrite
Backend evidence proves the route

If Codex cannot identify the canonical owner, it must stop with:

ROOT_OWNER_NOT_PROVEN

If the correct owner is outside approved file scope, Codex must stop with:

CORRECT_OWNER_SCOPE_APPROVAL_REQUIRED

Final rule:

No patching.
No shortcuts.
No phrase fixes.
Fix the root canonical owner with real algorithms and evidence.

0F. Dead Code / Bad Patch Removal Rule

Selene must not carry old rubbish forward into the provider-first architecture.

Once a new canonical path is proven, Codex must identify and remove old code that is no longer active, no longer required, or not part of the new architecture.

Code that must be removed once proven safe:

unused code
bad old patches
phrase patches
duplicate owners
wrong-owner shortcuts
stale fallback routes
obsolete tests
unreachable helper functions
dead provider paths
old architecture surfaces
Desktop semantic workarounds
Adapter semantic shortcuts
untracked provider callers
raw source dump paths
fake acceptance reports
failed WIP code

Do not quarantine rubbish by default.

Do not leave dead paths around “just in case.”

If Selene needs something later, it must be rebuilt cleanly inside the correct canonical owner.

Deletion rule:

Protect active working code.
Delete proven dead rubbish.
Do not preserve patch junk.

Before deleting, Codex must classify each old path as exactly one of:

CURRENT_ACTIVE_REQUIRED
RETAINED_COMPATIBILITY_PATH with retirement condition
MIGRATE_TO_CANONICAL_OWNER
DEAD_UNREACHABLE
STALE_DANGEROUS
WRONG_OWNER_SURFACE
REPO_TRUTH_CONFLICT

Action rules:

CURRENT_ACTIVE_REQUIRED
→ keep and regression-test

RETAINED_COMPATIBILITY_PATH
→ keep temporarily with exact retirement condition

MIGRATE_TO_CANONICAL_OWNER
→ migrate only with approval and correct-owner proof

DEAD_UNREACHABLE
→ remove when inside approved scope

STALE_DANGEROUS
→ remove, block, or escalate before more wiring

WRONG_OWNER_SURFACE
→ move/remove/escalate; do not keep behavior in wrong layer

REPO_TRUTH_CONFLICT
→ stop for JD decision

Codex must not delete active working capability blindly.

Codex must not keep bad patches merely because they compile.

Codex must not layer new code on top of failed or obsolete patches.

If Codex cannot prove a path is dead, stale, wrong-owner, or superseded, it must not delete it and must report:

OLD_PATH_DELETION_PROOF_REQUIRED

If cleanup requires files outside approved scope, Codex must stop with:

OLD_PATH_CLEANUP_SCOPE_APPROVAL_REQUIRED

Every implementation slice must include a cleanup report:

old paths inspected
old paths classified
old paths removed
old paths retained
retirement condition for retained paths
proof no duplicate owner remains
proof old accepted behavior still works

Final rule:

New canonical path proven first.
Then remove old rubbish.
No dead patch junk survives without a justified retirement condition.

1. Standard Vertical Slice Template

Each slice must use this structure:

Slice ID
Function card linked
Purpose
Canonical owner
Secondary owners
Forbidden owners
Lane declaration
What this slice builds
What this slice must not build
Provider interface involved
Canonical packet involved
Old paths to inspect
Old paths allowed to remove now
Old paths not allowed to remove yet
Required tests
Provider-off test
Fake-provider test
JD live test
Expected visible/audible result
Backend evidence to inspect
Pass condition
Fail condition
Repair loop
Commit/push requirement
Next slice after pass

Slice 0A — Repo Truth + Baseline Proof

Function card linked

All cards

Purpose

Establish current repo truth before any provider-first implementation.

This slice does not implement OpenAI.

Canonical owner

Repository truth / docs / ledger / current runtime evidence

Lane declaration

selected lane: READ_ONLY_AUDIT
simulation required: no
authority required: no
state mutation allowed: no
protected execution allowed: no
provider degradation allowed: yes
normal answer allowed: no runtime answer change
fail-closed required: yes for protected behavior proof

What this slice builds

Nothing.
It audits current repo state and current canonical owners.

What this slice must not build

No provider code.
No runtime code.
No Desktop code.
No Adapter code.
No PH1.X / PH1.M / PH1.WRITE / PH1.E edits.
No tests added.
No cleanup.
No old path deletion.

Provider interface involved

None yet.
Discovery only.

Canonical packet involved

None yet.
Discovery only.

Old paths to inspect

existing provider paths
OpenAI call paths
Brave/search provider paths
STT/TTS paths
Realtime paths
PH1.X active-context paths
PH1.WRITE formatter paths
PH1.M memory paths
PH1.E search/tool paths
Desktop semantic paths
Adapter semantic paths
protected execution paths

Old paths allowed to remove now

None.

Old paths not allowed to remove yet

All old paths.
Classification only.

Required tests

No code tests required unless repo law requires baseline commands.
Must identify existing test suites and exact runnable test names where possible.

Provider-off test

Discovery only.
Identify whether provider-off tests already exist.

Fake-provider test

Discovery only.
Identify whether fake-provider harness already exists.

JD live test

Not applicable unless Codex discovers a required baseline live smoke is already mandated by repo law.

Expected visible/audible result

No user-visible behavior change.

Backend evidence to inspect

current repo HEAD
origin/main posture
clean tree
existing owners
existing provider surfaces
existing tests
existing proof reports
current live proof status

Pass condition

AGENTS.md read.
Required docs read.
Clean tree proven.
Current HEAD and origin/main proven.
Existing owners discovered.
Existing provider surfaces discovered.
Current baseline status reported honestly.
No files changed.

Fail condition

dirty tree
missing required docs
repo truth conflict
unable to identify owners
Codex attempts implementation

Repair loop

Stop and report blocker.
Do not guess.
Do not implement.

Commit/push requirement

No commit unless docs/report file is explicitly authorized.

Next slice after pass

Slice 0C — Provider Coverage Normalization

Slice 0C — Provider Coverage Normalization

Function card linked

All cards
Provider Governance Architecture Card

Purpose

Normalize the provider coverage map before implementation.

This prevents Codex from building a partial OpenAI provider layer that misses packet, interface, or capability alignment.

Canonical owner

Provider coverage map / docs / architecture reference

Lane declaration

selected lane: READ_ONLY_AUDIT or DOCS-ONLY if JD authorizes docs update
simulation required: no
authority required: no
state mutation allowed: no runtime mutation
protected execution allowed: no
provider degradation allowed: yes
normal answer allowed: no runtime answer change
fail-closed required: yes for protected boundary classification

What this slice builds

Provider coverage table from current repo truth and master plan.

If JD authorizes docs update, Codex may add the coverage table to docs.

What this slice must not build

No provider implementation.
No OpenAI calls.
No runtime edits.
No Desktop edits.
No Adapter edits.
No PH1.X / PH1.M / PH1.WRITE / PH1.E edits.

Provider interface involved

All listed provider interfaces in master plan Section 6.

Canonical packet involved

All listed packet families in master plan Section 7.

Old paths to inspect

all existing provider/search/STT/TTS/OpenAI/Brave/fake-provider paths

Old paths allowed to remove now

None.

Old paths not allowed to remove yet

All old paths.

Required tests

No runtime tests unless repo law requires.
Must prove coverage table completeness.

Provider-off test

Identify existing provider-off tests.
Do not create or run live provider calls.

Fake-provider test

Identify existing fake-provider tests.

JD live test

JD_LIVE_ACCEPTANCE_NOT_APPLICABLE unless runtime behavior changes.

Expected visible/audible result

No behavior change.

Backend evidence to inspect

coverage map
provider interfaces found
packet contracts found
capability keys found
deferred services identified
initial implementation services identified

Pass condition

Every service has:
service name
category
canonical owner
provider interface
canonical packet
capability key
implementation status
first allowable phase
JD approval requirement

Fail condition

missing service classification
provider surface in one section but not another
unclear owner
Codex tries to implement early

Repair loop

Stop and report PROVIDER_COVERAGE_NORMALIZATION_REQUIRED.

Commit/push requirement

Only if JD authorizes docs update.

Next slice after pass

Slice 1 — Provider Governance Foundation

Slice 1 — Provider Governance Foundation

Function card linked

Card 1 — Provider Governance Architecture Card

Purpose

Build the minimum provider governance foundation inside existing canonical owner surfaces.

This is the first real implementation slice.

Canonical owner

Existing provider governance / provider registry / provider routing owner discovered by repo truth

Secondary owners

Storage / Audit
Adapter only if transport evidence is required

Forbidden owners

Desktop
iPhone
PH1.X as provider registry
PH1.WRITE as provider registry
ad hoc helper brain
new parallel provider engine

Lane declaration

selected lane: PROBABILISTIC_PUBLIC_ANSWER infrastructure / provider governance
simulation required: no
authority required: no
state mutation allowed: only internal provider evidence if repo owner allows
protected execution allowed: no
provider degradation allowed: yes
normal answer allowed: no behavior change unless explicitly scoped
fail-closed required: yes for protected execution impact

What this slice builds

provider registry
provider capability map
provider enable/disable gate
provider call request/result evidence envelope
provider-off result shape
basic budget/counter fields if not already present
fake provider seam if required

What this slice must not build

No live OpenAI calls.
No model reasoning feature.
No STT/TTS change.
No PH1.X semantic change.
No PH1.WRITE behavior change.
No search behavior change.
No memory behavior change.
No Desktop UI change.
No old path deletion unless proven dead and explicitly in scope.

Provider interface involved

ProviderRegistry
ProviderTokenBudgetPolicy
ProviderTraceProvider
ProviderHealthProvider
ProviderContractVersioningPolicy

Canonical packet involved

ProviderCallRequestPacket
ProviderCallResultPacket
ProviderCostEvidencePacket
ProviderLatencyEvidencePacket
ProviderFailurePacket
ProviderTokenBudgetPacket
ProviderTracePacket
ProviderHealthPacket

Old paths to inspect

existing OpenAI provider callers
Brave/search provider callers
STT/TTS provider callers
provider config
provider env flags
provider budget/counter code
existing fake provider harness
startup provider probes

Old paths allowed to remove now

Only dead local provider scaffolding proven unreachable and inside approved file scope.

Old paths not allowed to remove yet

active provider callers
STT/TTS paths
PH1.E provider paths
PH1.X provider-adjacent paths
Desktop/Adapter paths

Required tests

provider registry lists fake provider
provider disabled returns disabled result
provider disabled produces zero attempt
provider disabled produces zero network dispatch
budget exceeded blocks dispatch
malformed provider result rejected
provider evidence recorded
protected execution cannot proceed from provider result

Provider-off test

Required.
Provider disabled must prove zero call attempts and zero network dispatch.

Fake-provider test

Required.
Fake provider success/failure/malformed/timeout/budget exceeded.

JD live test

JD_LIVE_ACCEPTANCE_NOT_APPLICABLE if no user-visible behavior changes.

Expected visible/audible result

No visible/audible change.

Backend evidence to inspect

ProviderCallRequestPacket
ProviderCallResultPacket
ProviderFailurePacket
ProviderTokenBudgetPacket
provider counter evidence
provider-off zero-attempt proof

Pass condition

All targeted tests pass.
No live provider calls.
No user-visible behavior changed.
No protected path weakened.
No duplicate provider brain created.
Final tree clean.

Fail condition

provider disabled still attempts call
live provider called in normal test
raw provider output leaks
duplicate provider registry created
protected path weakened

Repair loop

Repair correct provider governance owner.
Rerun provider-off and fake-provider tests.
Do not patch Desktop/Adapter.

Commit/push requirement

Commit and push if implementation changed files.

Next slice after pass

Slice 2 — Fake Provider + Provider-Off Proof Pack

Slice 2 — Fake Provider + Provider-Off Proof Pack

Function card linked

Card 1 — Provider Governance
Card 9 — Eval / Regression / Optimization

Purpose

Make fake-provider and provider-off behavior reusable across future slices.

Canonical owner

Provider governance test harness / fake provider harness discovered by repo truth

Lane declaration

selected lane: infrastructure / test harness
simulation required: no
authority required: no
state mutation allowed: no protected mutation
protected execution allowed: no
provider degradation allowed: yes
normal answer allowed: no behavior change unless scoped
fail-closed required: yes for protected provider proposal tests

What this slice builds

reusable fake provider cases
provider-off proof helpers
provider-malformed proof
provider-timeout proof
provider-budget-exceeded proof
protected proposal cannot execute proof

What this slice must not build

No live OpenAI calls.
No new semantic behavior.
No user-visible feature.
No Desktop edits unless test harness proves current-app posture and JD approves.

Provider interface involved

ProviderRegistry
FakeProvider
ProviderHealthProvider
ProviderCircuitBreakerPolicy if present

Canonical packet involved

ProviderCallResultPacket
ProviderFailurePacket
ProviderHealthPacket
ProviderCircuitBreakerPacket
BackendEvidenceVerificationPacket

Old paths to inspect

existing fake providers
mock providers
fixture-only providers
test-only shortcuts
vacuous tests

Old paths allowed to remove now

Only duplicate/dead fake provider helpers inside approved scope after replacement proof.

Old paths not allowed to remove yet

Any active test harness used by current suites.

Required tests

fake provider success
fake provider timeout
fake provider malformed
provider disabled
provider budget exceeded
provider unsupported capability
protected proposal blocked
zero-test/vacuous proof rejected

Provider-off test

Required.

Fake-provider test

Required.

JD live test

JD_LIVE_ACCEPTANCE_NOT_APPLICABLE unless user-visible behavior changes.

Expected visible/audible result

No visible/audible change.

Backend evidence to inspect

test output showing nonzero tests
provider-off zero attempt
fake provider result packets
failure packets

Pass condition

Reusable provider proof pack exists.
Normal tests never call live provider.
Provider-off proof is reusable by later slices.

Fail condition

fake provider only tests helper not runtime path
provider-off still dispatches
tests run zero cases

Repair loop

Fix harness owner.
Rerun exact nonzero tests.

Commit/push requirement

Commit and push if implementation changed files.

Next slice after pass

Slice 3 — PH1.X → PH1.WRITE “One Line” Vertical Slice

Slice 3 — PH1.X → PH1.WRITE “One Line” Vertical Slice

Function card linked

Card 3 — PH1.X Current Turn Understanding
Card 4 — PH1.WRITE Writing / Presentation
Card 10 — Desktop / iPhone Boundary if UI rendering is touched

Purpose

Prove the first end-to-end provider-first behavior:

User asks for prior answer to be rewritten as one line.
OpenAI/fake provider proposes meaning.
PH1.X validates.
PH1.WRITE owns final output.
Desktop renders only.
Backend evidence proves route.

Canonical owner

PH1.X for target/operation validation
PH1.WRITE for output

Secondary owners

Provider governance
Adapter transport
Desktop renderer only if user-visible app proof is in scope
Storage / Audit

Forbidden owners

Desktop deciding “one line”
Adapter deciding previous answer target
OpenAI directly controlling route
phrase patch in PH1.X or Adapter

Lane declaration

selected lane: PROBABILISTIC_PUBLIC_ANSWER
simulation required: no
authority required: no
state mutation allowed: only evidence/logging if current architecture allows
protected execution allowed: no
provider degradation allowed: yes
normal answer allowed: yes
fail-closed required: yes for protected negative cases

What this slice builds

SemanticInterpreterProvider fake/provider-backed proposal for one_line operation
CurrentTurnInterpretationPacket
PH1.X validation of target = previous answer
HumanConversationDirective to PH1.WRITE
WriteRequestPacket
WriteOutputPacket
backend route evidence
provider-off degraded behavior

What this slice must not build

No full PH1.X rewrite.
No full PH1.WRITE rewrite.
No memory recall.
No search.
No voice requirement unless chosen as smoke path.
No phrase-specific production branch.
No Desktop/Adapter semantics.
No old shortcut deletion until proof.

Provider interface involved

SemanticInterpreterProvider
WritingProvider

Canonical packet involved

CurrentTurnInterpretationPacket
HumanConversationDirective
WriteRequestPacket
WriteOutputPacket
ProviderDecisionTracePacket
BackendEvidenceVerificationPacket

Old paths to inspect

make it shorter phrase patches
one-line formatter shortcuts
adapter active context shortcuts
Desktop formatting logic
PH1.WRITE plain formatter path

Old paths allowed to remove now

None unless repo truth proves dead local helper and JD approves scope.

Old paths not allowed to remove yet

current active PH1.X fallback
current PH1.WRITE fallback
adapter active context fallback
Desktop rendering path

Required tests

one-line request targets previous answer
make it shorter targets previous answer
make it warmer targets previous answer
unrelated “what is your name” does not target previous answer
protected payroll request does not route to PH1.WRITE rewrite
provider disabled safe-degrades
fake provider malformed output rejected
no phrase-patch scan violations

Provider-off test

Required.
Expected: no provider call attempt, safe fallback or clarification/degraded response.

Fake-provider test

Required for normal tests.

JD live test

JD live required if user-visible runtime path is changed.

Prompt sequence:
1. JD asks Selene a normal question that produces a multi-line or normal answer.
2. JD says: “Can you give me one line?”

Expected:
Selene rewrites the previous answer into one line.
Selene does not switch topics.
Selene does not say it cannot find context if prior answer exists.
Selene does not use Desktop/Adapter logic to decide meaning.

Expected visible/audible result

A clean one-line answer.
If TTS enabled, TTS speaks the one-line tts_text only.

Backend evidence to inspect

CurrentTurnInterpretationPacket
HumanConversationDirective
selected target = previous answer
rejected stale/wrong targets if any
WriteRequestPacket
WriteOutputPacket
provider evidence fake/off/live depending on mode
Desktop render packet if app tested
TTS text hash if voice tested

Pass condition

JD live result correct.
Backend evidence proves PH1.X selected target and PH1.WRITE produced output.
No Desktop/Adapter semantic authority.
Provider-off test passes.
Fake-provider tests pass.
Protected negative case fail-closed.

Fail condition

wrong target
stale topic
Desktop/Adapter decides meaning
provider proposal blindly trusted
no backend evidence
cargo passes but JD live fails

Repair loop

Classify failure owner.
Repair PH1.X if target/routing wrong.
Repair PH1.WRITE if output wrong.
Repair Adapter only if transport lost packet.
Repair Desktop only if rendering wrong.
Rerun failed JD live scenario and backend evidence check.

Commit/push requirement

Commit and push only after JD live acceptance if user-visible behavior changed.

Next slice after pass

Slice 4 — Voice Wake → STT → Answer → TTS → Re-arm

Slice 4 — Voice Wake → STT → Answer → TTS → Re-arm

Function card linked

Card 2 — Voice Runtime
Card 10 — Desktop / iPhone Boundary

Purpose

Prove the core voice loop with provider governance:

wake → transcript → answer → approved TTS → playback completion → re-arm

Canonical owner

PH1.W
PH1.C
PH1.TTS
PH1.L

Secondary owners

PH1.X
PH1.WRITE
Adapter
Desktop shell
Storage / Audit

Forbidden owners

Desktop semantic wake
Desktop transcript decision
Adapter semantic transcript decision
OpenAI protected execution

Lane declaration

selected lane: PROBABILISTIC_PUBLIC_ANSWER voice runtime
simulation required: no
authority required: no
state mutation allowed: only evidence/session state as current architecture allows
protected execution allowed: no
provider degradation allowed: yes
normal answer allowed: yes
fail-closed required: yes for protected voice requests

What this slice builds

lawful wake to STT provider path
TranscriptPacket admission
answer path to PH1.WRITE
approved_tts_text to TTS provider
VoiceOutputPacket
playback completion evidence
re-arm evidence
provider-off degraded voice state

What this slice must not build

No full realtime duplex.
No barge-in.
No Voice ID authority.
No memory recall.
No protected execution.
No Desktop semantic logic.

Provider interface involved

SpeechToTextProvider
TextToSpeechProvider
ProviderGovernance

Canonical packet involved

TranscriptPacket
VoiceOutputPacket
ProviderCallResultPacket
HumanConversationDirective
WriteOutputPacket

Old paths to inspect

old Apple STT/TTS remnants
old wake/listening loops
Desktop transcript decisions
adapter transcript shortcuts
TTS fallback paths
pre-wake provider calls

Old paths allowed to remove now

Only dead local voice remnants proven unused and explicitly approved.

Old paths not allowed to remove yet

current active voice fallback paths until new loop passes JD live.

Required tests

wake word not committed as prompt
cough before wake rejected
question after wake committed
TTS receives approved text only
TTS completion evidence
re-arm evidence
provider-off degraded state
protected voice request fail-closed

Provider-off test

Required.
No STT/TTS network dispatch when disabled.

Fake-provider test

Required if provider call path is implemented.

JD live test

JD says:
“Selene, what time is it in Sydney?”

Expected:
Selene wakes, captures the question, answers, speaks if TTS enabled, and re-arms.

Expected visible/audible result

Transcript appears/captured correctly.
Answer visible.
TTS heard if enabled.
Selene returns to listening/re-arm state.

Backend evidence to inspect

wake evidence
TranscriptPacket
STT provider evidence
HumanConversationDirective
WriteOutputPacket
VoiceOutputPacket
TTS provider evidence
playback completion evidence
session/re-arm evidence

Pass condition

JD live voice works.
Backend evidence agrees.
No stale app.
One app / one adapter.
No Desktop/Adapter semantic authority.

Fail condition

missed speech
wake word becomes prompt
TTS text differs from approved text
no re-arm
backend evidence missing
cargo passes but JD live fails

Repair loop

Repair correct owner:
PH1.W wake
PH1.C transcript
PH1.TTS output
PH1.L session/re-arm
Adapter transport
Desktop capture/playback only
Then rerun JD live test.

Commit/push requirement

Commit and push only after JD live acceptance and backend evidence pass.

Next slice after pass

Slice 5 — PH1.E Search With Accepted Source Chip

Slice 5 — PH1.E Search With Accepted Source Chip

Function card linked

Card 5 — PH1.E Search / Tools / Evidence
Card 4 — PH1.WRITE
Card 10 — Desktop / iPhone Boundary if source chips render in app

Purpose

Prove provider-first search uses PH1.E evidence discipline and PH1.WRITE presentation.

Canonical owner

PH1.E

Secondary owners

PH1.X
PH1.WRITE
Provider governance
Desktop renderer if source chips visible
Storage / Audit

Forbidden owners

OpenAI accepted-source authority
Desktop source ranking
Adapter search brain
raw provider source dump

Lane declaration

selected lane: PROBABILISTIC_PUBLIC_ANSWER / public search
simulation required: no
authority required: no
state mutation allowed: evidence only if current architecture allows
protected execution allowed: no
provider degradation allowed: yes
normal answer allowed: yes
fail-closed required: yes for mixed protected prompt

What this slice builds

SearchProvider fake/provider route
SearchEvidencePacket
accepted/rejected source separation
SourceChipPacket
PH1.WRITE source-backed answer
provider-off search degrade
prompt-injection source defense

What this slice must not build

No live provider unless explicitly enabled.
No deep research.
No file search.
No image cards.
No broad PH1.E rewrite.
No raw source dumps.

Provider interface involved

SearchProvider
CitationFormattingProvider
PromptInjectionDefensePolicy

Canonical packet involved

SearchEvidencePacket
SourceAcceptancePacket
SourceChipPacket
CitationPresentationPacket
WriteOutputPacket
PromptInjectionDefensePacket

Old paths to inspect

raw source dumps
Brave/OpenAI/GDELT direct callers
source chip bypasses
wrong-source acceptance
provider shortcuts without budget
PH1.E stale context paths

Old paths allowed to remove now

Only old source dump fallback if proven superseded and approved.

Old paths not allowed to remove yet

current active search provider paths until new PH1.E route passes proof.

Required tests

accepted source displayed as chip
rejected source not displayed
wrong-source rejected
source prompt injection ignored
provider-off safe degrade
mixed search + protected action split
no raw URL/source dump in response_text
tts_text excludes source dump

Provider-off test

Required.
No provider attempt/dispatch.

Fake-provider test

Required for normal tests.

JD live test

JD asks:
“Search public news about a synthetic topic and show sources.”

Expected:
Clean answer with source chip.
No raw dump.
No rejected source shown.

Expected visible/audible result

Short answer.
Small source chip if Desktop supports it.
TTS speaks clean answer only.

Backend evidence to inspect

SearchEvidencePacket
accepted source IDs
rejected source IDs
SourceAcceptancePacket
SourceChipPacket
WriteOutputPacket
provider counters
prompt-injection defense status

Pass condition

JD visible answer correct.
Backend PH1.E evidence proves accepted/rejected separation.
Provider-off passes.
No raw dumps.

Fail condition

wrong source accepted
source dump appears
TTS speaks source dump
provider-off still calls network
backend evidence missing

Repair loop

Repair PH1.E for search/source discipline.
Repair PH1.WRITE for presentation.
Repair Desktop only for chip rendering.

Commit/push requirement

Commit and push after tests + JD live if user-visible.

Next slice after pass

Slice 6 — PH1.M Fresh Recall Without Session Wording

Slice 6 — PH1.M Fresh Recall Without Session Wording

Function card linked

Card 6 — PH1.M Memory / Recall
Card 4 — PH1.WRITE

Purpose

Prove PH1.M can answer a fresh recall question naturally with evidence, without archive/session wording.

Canonical owner

PH1.M

Secondary owners

PH1.X
PH1.WRITE
Storage / Audit
Voice ID speaker evidence if voice-tested

Forbidden owners

Adapter memory shortcut
Desktop memory brain
OpenAI memory as truth
PH1.X durable memory brain

Lane declaration

selected lane: PROBABILISTIC_PUBLIC_ANSWER with governed memory retrieval
simulation required: no
authority required: no unless private/company memory policy requires access
state mutation allowed: memory read/evidence only
protected execution allowed: no
provider degradation allowed: yes
normal answer allowed: yes
fail-closed required: yes for private/unknown-speaker memory

What this slice builds

fresh recall route
MemoryEvidencePacket
PH1.M speaker/scope check
PH1.WRITE natural memory answer
provider-off behavior

What this slice must not build

No deep lifelong memory.
No full topic graph.
No memory write UX.
No embedding provider unless already governed.
No adapter shortcut.

Provider interface involved

EmbeddingProvider only if already governed
ContextCompactionProvider only if already governed
otherwise no provider required for first slice

Canonical packet involved

MemoryEvidencePacket
WriteOutputPacket
ProviderDataEgressPacket if provider used

Old paths to inspect

adapter memory shortcuts
session-search wording
old 72-hour-only path
unscoped recall
duplicate memory helpers

Old paths allowed to remove now

Only session-search wording path if directly superseded and approved.

Old paths not allowed to remove yet

active recall/index/archive paths until new PH1.M path passes.

Required tests

fresh recall works
no session-search wording
unknown speaker cannot access JD private memory
conflicting old/new memory prefers newer or asks
provider-off recall behavior
protected request through memory fails closed

Provider-off test

Required if provider memory assistance is used.

Fake-provider test

Required if provider summarisation/embedding is used.

JD live test

JD asks:
“What did we decide about the provider-first plan?”

Expected:
Natural memory answer with evidence.
No “I searched sessions/archive” style wording.

Expected visible/audible result

Clear natural answer.
If voice, TTS speaks natural memory answer.

Backend evidence to inspect

MemoryEvidencePacket
speaker scope
memory source refs
PH1.M decision trace
WriteOutputPacket

Pass condition

JD live answer feels like memory.
Backend evidence proves PH1.M owner.
No Adapter/Desktop memory brain.

Fail condition

session/archive wording
wrong speaker memory leak
adapter did recall
no backend evidence
old memory beats newer decision incorrectly

Repair loop

Repair PH1.M for memory owner/scope.
Repair PH1.WRITE for wording.
Do not patch Adapter/Desktop.

Commit/push requirement

Commit and push after JD live and backend evidence if user-visible.

Next slice after pass

Slice 7 — File/Image Evidence Summary

Slice 7 — File/Image Evidence Summary

Function card linked

Card 7 — Vision / Files / Artifact
Card 4 — PH1.WRITE
Card 5 — PH1.E if evidence/source handling used

Purpose

Prove file/image input becomes bounded evidence and clean summary without raw dumps or automatic memory.

Canonical owner

PH1.E for evidence
PH1.WRITE for summary
Storage for file lifecycle
PH1.M only if memory permission is explicitly involved

Secondary owners

Provider governance
Desktop renderer if file/image cards are displayed

Forbidden owners

Desktop image/file intelligence
OpenAI truth authority
automatic memory write
generated artifact as official record

Lane declaration

selected lane: PROBABILISTIC_PUBLIC_ANSWER / advisory file or image summary
simulation required: no
authority required: no unless private/company file policy requires access
state mutation allowed: file/evidence metadata only if current architecture allows
protected execution allowed: no
provider degradation allowed: yes
normal answer allowed: yes
fail-closed required: yes for official-record/protected requests

What this slice builds

file/image input policy check
bounded FileEvidencePacket or VisualEvidencePacket
prompt-injection defense over file/OCR text
PH1.WRITE summary with limitations
no memory write unless permitted

What this slice must not build

No image generation.
No video generation.
No code interpreter.
No official document execution.
No automatic memory storage.

Provider interface involved

VisionProvider if image understanding is included
FileLifecycleProvider
ProviderFileInputPolicy
FileSearchProvider only if file Q&A retrieval is included

Canonical packet involved

ProviderFileInputPacket
ProviderFileLifecyclePacket
FileEvidencePacket
VisualEvidencePacket
PromptInjectionDefensePacket
WriteOutputPacket

Old paths to inspect

raw file dump to UI/TTS
file upload automatically memory
image card without provenance
Desktop image selection logic
file provider submission without permission

Old paths allowed to remove now

None unless direct dead raw-dump helper is proven unused and approved.

Old paths not allowed to remove yet

active file upload/render paths.

Required tests

file permission checked
file input blocked without permission
bounded summary
no raw full file dump
prompt injection inside file ignored
image uncertainty stated
official-record attempt fail-closed
no memory write without permission

Provider-off test

Required if vision/file provider is used.

Fake-provider test

Required for normal provider tests.

JD live test

JD uploads a document/image and asks:
“Summarize this briefly.”

Expected:
Clean bounded summary.
No raw dump.
No automatic memory claim.

Expected visible/audible result

Short summary with limitations where appropriate.
If TTS, clean summary only.

Backend evidence to inspect

ProviderFileInputPacket
FileEvidencePacket or VisualEvidencePacket
PromptInjectionDefensePacket
WriteOutputPacket
memory not written unless explicitly allowed

Pass condition

File/image summary works.
Backend evidence proves bounded file/visual evidence.
No raw dump.
No memory leak.

Fail condition

full file dumped
prompt injection followed
file became memory automatically
Desktop chose meaning
provider-off still called network

Repair loop

Repair PH1.E/file evidence or PH1.WRITE summary.
Repair Desktop only if render-only failure.

Commit/push requirement

Commit and push after tests + JD live if user-visible.

Next slice after pass

Slice 8 — Read-Only Tool Proposal

Slice 8 — Read-Only Tool Proposal

Function card linked

Card 8 — Tool / MCP / Connector
Card 5 — PH1.E
Card 11 — Protected Simulation Execution for negative tests

Purpose

Prove OpenAI/fake provider can propose a read-only tool, but PH1.E decides whether it executes.

Canonical owner

PH1.E

Secondary owners

PH1.X
PH1.WRITE
Access/Governance if permissioned data
SimulationExecutor for protected negative tests

Forbidden owners

OpenAI direct tool execution
Adapter tool brain
Desktop tool routing
MCP output authority

Lane declaration

selected lane: PROBABILISTIC_PUBLIC_ANSWER read-only tool proposal
simulation required: no for read-only public tool
authority required: depends on data source
state mutation allowed: no
protected execution allowed: no
provider degradation allowed: yes
normal answer allowed: yes
fail-closed required: yes for protected write attempts

What this slice builds

ToolProposalPacket from fake/provider
PH1.E tool permission check
read-only tool execution decision
PH1.WRITE summary of result
protected write proposal rejected

What this slice must not build

No connector writes.
No MCP write actions.
No external message sending.
No protected tool execution.
No broad tool catalog.

Provider interface involved

ToolProposalProvider
ToolSearchProvider only if needed
PromptInjectionDefensePolicy

Canonical packet involved

ToolProposalPacket
ToolExecutionDecisionPacket
WriteOutputPacket
PromptInjectionDefensePacket

Old paths to inspect

adapter tool routing
direct tool execution helpers
provider-driven tool calls
untracked tool schemas
protected write shortcuts

Old paths allowed to remove now

Only dead direct tool shortcut if proven unreachable and approved.

Old paths not allowed to remove yet

active tool routing until PH1.E path is proven.

Required tests

read-only tool allowed
malformed tool args rejected
provider-off tool proposal safe-degrades
protected write denied
simulation missing fail-closed
prompt injection in tool output blocked

Provider-off test

Required.

Fake-provider test

Required.

JD live test

JD asks a read-only lookup where tool is available.

Expected:
Selene retrieves/answers read-only.
Protected write attempt fails closed.

Expected visible/audible result

Clean answer from read-only result.
No protected execution.

Backend evidence to inspect

ToolProposalPacket
ToolExecutionDecisionPacket
PH1.E decision trace
Access/Gov result if permissioned
SimulationExecutor fail-closed if protected
WriteOutputPacket

Pass condition

PH1.E owns tool decision.
Read-only allowed.
Protected write denied.
Backend evidence proves route.

Fail condition

provider directly executes
Adapter routes tool
protected write executes
tool output prompt injection followed

Repair loop

Repair PH1.E tool policy.
Repair Access/Gov if permission issue.
Repair SimulationExecutor boundary only with explicit approval.

Commit/push requirement

Commit and push after tests + JD live if user-visible.

Next slice after pass

Slice 9 — Eval Harness for “One Line” Slice

Slice 9 — Eval Harness for “One Line” Slice

Function card linked

Card 9 — Eval / Regression / Optimization
Card 3 — PH1.X
Card 4 — PH1.WRITE

Purpose

Create the first provider-first eval/regression pack around the first behavior slice.

Canonical owner

Eval harness / acceptance matrix discovered by repo truth

Lane declaration

selected lane: EVAL_OPTIMIZATION / test harness
simulation required: no
authority required: no
state mutation allowed: no runtime mutation
protected execution allowed: no
provider degradation allowed: yes
normal answer allowed: no user behavior change unless tests exercise endpoint
fail-closed required: yes for protected negative cases

What this slice builds

eval corpus skeleton for one-line operation
positive cases
unseen paraphrases
negative hijack cases
protected fail-closed cases
backend evidence check expectation

What this slice must not build

No live eval provider unless JD approves.
No fine-tuning.
No prompt optimizer.
No production behavior changes unless explicitly scoped.

Provider interface involved

EvalProvider only fake/local unless JD approves live
GraderProvider deferred
PromptOptimizationProvider deferred

Canonical packet involved

EvalResultPacket
BackendEvidenceVerificationPacket
JDLiveTestTracePacket if live-tested

Old paths to inspect

vacuous tests
zero-test commands
mock-only acceptance reports
old proof language claiming live pass without live evidence

Old paths allowed to remove now

Obsolete/vacuous tests only if directly superseded and approved.

Old paths not allowed to remove yet

active regression tests.

Required tests

one-line positive
make shorter paraphrase
make concise paraphrase
unrelated question negative
protected payroll negative
provider malformed negative
Desktop/Adapter no-brain proof where applicable
nonzero exact test execution

Provider-off test

Required if provider path is exercised.

Fake-provider test

Required.

JD live test

Not required if eval-only and no runtime change.
But eval pack must reference JD live scenario required by Slice 3.

Expected visible/audible result

No user-visible behavior change unless running live scenario.

Backend evidence to inspect

EvalResultPacket
test count output
BackendEvidenceVerificationPacket if integrated

Pass condition

Eval pack tests the intended behavior and negative cases.
No zero-test pass.
No live provider call in normal tests.

Fail condition

tests only assert helpers
zero tests run
real searched names hardcoded
eval replaces JD live acceptance

Repair loop

Fix eval harness.
Rerun exact tests and prove nonzero execution.

Commit/push requirement

Commit and push after tests.

Next slice after pass

Slice 10 — Desktop Render-Only Proof

Slice 10 — Desktop Render-Only Proof

Function card linked

Card 10 — Desktop / iPhone Boundary
Card 4 — PH1.WRITE
Card 2 — Voice Runtime if TTS/playback included

Purpose

Prove Desktop renders accepted runtime packets only and does not become a semantic brain.

Canonical owner

Desktop/iPhone shell for rendering only
Adapter transport
Runtime owners for meaning

Lane declaration

selected lane: DISTRIBUTION_SURFACE / renderer proof
simulation required: no
authority required: no
state mutation allowed: no protected mutation
protected execution allowed: no
provider degradation allowed: yes
normal answer allowed: yes if UI behavior tested
fail-closed required: yes for protected negative display

What this slice builds

Desktop render-only proof for WriteOutputPacket
source chip/image/artifact render boundary if in scope
TTS playback evidence if in scope
current-app provenance proof

What this slice must not build

No Desktop meaning logic.
No provider calls in Desktop.
No provider secrets in Desktop.
No Desktop memory.
No Desktop tool routing.

Provider interface involved

None directly.

Canonical packet involved

WriteOutputPacket
VoiceOutputPacket if TTS
SourceChipPacket if source chips
GeneratedArtifactPacket if artifacts

Old paths to inspect

Desktop semantic intent logic
Desktop formatting brain
Desktop memory/search/tool routing
Desktop TTS provider choice
provider secrets in client
stale app paths

Old paths allowed to remove now

Only stale render helpers directly superseded and approved.

Old paths not allowed to remove yet

active app shell paths.

Required tests

one app instance
one adapter/runtime owner
current HEAD provenance
Desktop renders runtime output
Desktop does not call provider
Desktop does not decide meaning
TTS playback evidence if TTS included

Provider-off test

Required only if provider degraded state is rendered.

Fake-provider test

Not required unless render path uses fake provider output packets.

JD live test

JD tests latest app only after Codex proves current app provenance.

Prompt:
Use the Slice 3 one-line scenario or Slice 4 voice scenario.

Expected:
Desktop displays exactly runtime-approved output.

Expected visible/audible result

Correct output visible.
TTS speaks approved text only if enabled.
No duplicate/stale app.

Backend evidence to inspect

current HEAD
bundle path
process count
adapter owner
health/provenance
runtime packet refs
visible UI result
TTS playback evidence if relevant

Pass condition

Desktop proves renderer-only.
JD live visible result matches backend packet.
No Desktop provider/semantic authority.

Fail condition

stale app
multiple app/adapter owners
Desktop changes meaning
Desktop calls provider
backend evidence mismatch

Repair loop

Repair Desktop only if render/playback/provenance issue.
Repair runtime owner if meaning/output wrong.
Retest latest app.

Commit/push requirement

Commit and push after JD live proof if Desktop changed.

Next slice after pass

Slice 11 — Protected Payroll Fail-Closed Proof

Slice 11 — Protected Payroll Fail-Closed Proof

Function card linked

Card 11 — Protected Simulation Execution
Card 3 — PH1.X
Card 4 — PH1.WRITE

Purpose

Prove provider-first intelligence does not weaken protected simulation law.

Canonical owner

PH1.X for protected classification
SimulationExecutor / Access-Gov for protected execution gate
PH1.WRITE for final wording

Lane declaration

selected lane: MIXED_REQUEST / DETERMINISTIC_PROTECTED_EXECUTION for protected part
simulation required: yes for protected action
authority required: yes for protected action
state mutation allowed: no unless approved simulation executes
protected execution allowed: only through approved simulation + authority
provider degradation allowed: yes for public/advisory part
normal answer allowed: yes for public explanation
fail-closed required: yes for protected action

What this slice builds

protected payroll classification proof
public payroll explanation allowed
approve payroll fail-closed
provider proposal cannot execute
PH1.WRITE protected wording
backend fail-closed evidence

What this slice must not build

No real payroll execution.
No salary changes.
No database mutation.
No new simulation unless JD explicitly approves.
No authority bypass.
No Voice ID authority.

Provider interface involved

SemanticInterpreterProvider as proposal only
WritingProvider for wording only

Canonical packet involved

CurrentTurnInterpretationPacket
HumanConversationDirective
ToolProposalPacket if involved
ToolExecutionDecisionPacket if involved
WriteOutputPacket
SimulationExecutionPacket/AuditEvidencePacket only where repo owner defines existing surfaces

Old paths to inspect

adapter payroll classification helpers
protected action shortcuts
business mutation outside SimulationExecutor
Voice ID authority shortcut
Desktop/Adapter protected execution path

Old paths allowed to remove now

Only dead protected shortcut proven unreachable and approved.

Old paths not allowed to remove yet

active protected fail-closed path.

Required tests

Tell me about payroll → public/advisory answer
Approve payroll for Tim → fail closed
Increase Tim salary → fail closed
Search salary trends and increase Tim salary → public part allowed, protected part fail-closed
provider says protected action is allowed → reject
Voice ID not authority

Provider-off test

Required for provider-assisted interpretation if used.
Protected fail-closed must still work provider-off.

Fake-provider test

Required.
Fake provider must not be able to authorize protected execution.

JD live test

JD says:
“Organize payroll for Tim.”

Expected:
Selene identifies protected/business intent and fails closed unless approved simulation + authority exists.

Expected visible/audible result

Clear protected fail-closed wording.
No claim that payroll was approved, submitted, changed, or completed.

Backend evidence to inspect

PH1.X protected classification
HumanConversationDirective blocked action
Simulation lookup result
Authority result
fail-closed reason
WriteOutputPacket protected wording
audit/evidence refs if available

Pass condition

Protected action does not execute.
Public/advisory part is not wrongly blocked.
Backend evidence proves fail-closed.
Provider cannot authorize execution.

Fail condition

protected action executes
Selene claims action completed
provider proposal bypasses simulation
public/advisory answer blocked incorrectly
backend evidence missing

Repair loop

Repair PH1.X protected classification if classification wrong.
Repair SimulationExecutor/Access-Gov only with explicit scope if gate wrong.
Repair PH1.WRITE if wording overclaims.
Retest JD live protected prompt.

Commit/push requirement

Commit and push after tests + JD live proof if behavior changed.

Next slice after pass

Return to next function card activation based on repo truth:
Search deeper, memory deeper, voice duplex, or old-path retirement.

12. Old Path Retirement Mini-Slice

This mini-slice repeats after each function slice passes.

Purpose

Remove old rubbish only after the replacement path is proven.

When allowed

Only after:

new canonical path proven
old accepted behavior re-proven
JD live acceptance passed where user-visible
backend evidence proves new owner
provider-off/fake-provider tests pass where relevant
phrase-patch scan done
scope approval clear

Classification

Every old path must be classified:

CURRENT_ACTIVE_REQUIRED
RETAINED_COMPATIBILITY_PATH with retirement condition
MIGRATE_TO_CANONICAL_OWNER
DEAD_UNREACHABLE
STALE_DANGEROUS
WRONG_OWNER_SURFACE
REPO_TRUTH_CONFLICT

Pass condition

dead/rubbish path removed only if safe
retained path has retirement condition
no duplicate owner remains
tests still pass
JD live behavior still passes where relevant
final clean tree

Stop conditions

old path may still be active
owner conflict
scope unclear
protected behavior risk
Codex cannot prove path is dead

13. Recommended Slice Order

0A. Repo Truth + Baseline Proof
0C. Provider Coverage Normalization
1. Provider Governance Foundation
2. Fake Provider + Provider-Off Proof Pack
3. PH1.X → PH1.WRITE “One Line” Vertical Slice
4. Voice Wake → STT → Answer → TTS → Re-arm
5. PH1.E Search With Accepted Source Chip
6. PH1.M Fresh Recall Without Session Wording
7. File/Image Evidence Summary
8. Read-Only Tool Proposal
9. Eval Harness for “One Line” Slice
10. Desktop Render-Only Proof
11. Protected Payroll Fail-Closed Proof
12. Old Path Retirement Mini-Slice after each proven replacement

14. Final Rule

Do not build everything at once.
Do not treat this as one implementation prompt.
Do not skip AGENTS.md.
Do not skip repo-truth discovery.
Do not skip JD live testing where user-visible.
Do not call cargo pass a product pass.
Do not delete old paths before proof.

Build one slice.
Test exactly what was built.
JD live-tests where user-visible.
Inspect backend evidence.
Repair until it works.
Commit/push clean.
Then move to the next slice.
