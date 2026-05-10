SELENE ACTIVE SESSION CONTEXT AND SEGMENT BOUNDARY REPAIR

TASK:
SELENE_ACTIVE_SESSION_CONTEXT_AND_SEGMENT_BOUNDARY_REPAIR

BUILD CLASS:
IMPLEMENTATION / FIX BUILD

PRIMARY GOAL:
Make Selene behave like a continuous conversation inside the current live session by repairing active session context, pending clarification, short follow-up resolution, and session no-leak boundaries.

SECONDARY GOAL:
Confirm an additive runtime-owned active conversation boundary using existing identifiers first — preferably `session_id + thread_key + turn_id/correlation_id` where repo truth supports them — so later 72-hour memory and archive recall can be built without being tangled into active session context.

Only add a new PH1.X contract field or explicit segment field if same-task repo truth proves existing identifiers cannot prevent context leaks.

1. Why this build exists

Codex read-only audits already confirmed enough repo truth to proceed.

Current state:

Continuous conversation is PARTIAL.
Memory is structurally built but not the owner of short same-session follow-ups.
PH1.X and PH1.M are only partially connected.
There is no clear internal conversation segment model.
72-hour HOT memory is resume/retention logic only, not general searchable recall.

The immediate user-visible failure is active conversation continuity:

User: What’s the weather like in Australia?
Selene: Which city?
User: Sydney.
Expected: Selene fills the pending city/location slot with Sydney and routes the prior weather request to Sydney weather.

User: What time is it in New York?
Selene: It is ...
User: What about London?
Expected: Selene inherits the time intent and answers London time.

New session:
User: Sydney.
Expected: Selene must not inherit old Australia/weather context.

This is not durable memory.

This is active session context.

2. Architecture decision

The correct boundary is:

PH1.L Session Engine = session container / lifetime / close / timeout / no-leak boundary.
PH1.X = active conversation context brain: pending slot, last intent, follow-up, correction, current topic.
Adapter runtime bridge = carries same session/thread/turn context through live typed/voice turns.
PH1.M = governed memory, recall, proposal, archive, 72-hour memory, medium-term memory, permanent memory.
Desktop/iPhone = capture, playback, transport, render, runtime bridge only.

Therefore:

“Sydney” after “weather in Australia” belongs to PH1.X active session context.
“What about London?” after “New York time” belongs to PH1.X active session context.
“What did we discuss yesterday?” belongs to PH1.M/storage/archive recall.

This build must not solve active conversation using PH1.M durable memory.

3. Codex upgrade changes incorporated

The following upgrades are now mandatory in this build instruction:

1. Segment boundary is additive only.
2. Prefer existing session_id + thread_key + turn_id/correlation_id before adding any PH1.X field.
3. PH1.M inspection is boundary-only, not a broad memory audit.
4. Pass condition includes no MemoryOperation::Propose, no PH1.M thread digest, and no recall candidate required for Sydney or What about London.
5. Provider-off/fake-tool safe-degrade may pass if routing/context is proven.
6. PH1.X test filters must be discovered and run if nonzero.
7. Every filtered test requires list-first proof and nonzero execution proof.
8. Sydney acceptance means fills pending city/location slot, not merely mentions Sydney.
9. Pending slot clearing must be proven after resolution.
10. Memory candidate cannot hijack active context if the harness allows seeding/passing candidates.
11. Typed real adapter/runtime endpoint smoke is acceptable fallback if native app/voice smoke is blocked.
12. Commit message should be `Runtime: repair active session context` unless a real segment field is added.

4. Lane declaration required before editing

Codex must declare this before any edit:

LANE DECLARATION:
current project phase: PROBABILISTIC_FOUNDATION_BUILD
selected lane: PROBABILISTIC_PUBLIC_ANSWER
simulation required: no
authority required: no
state mutation allowed: repo files only during implementation/test
protected execution allowed: no
provider degradation allowed: yes for public read-only tool behavior, if existing gates allow
normal answer allowed: yes
fail-closed required: yes for protected/business actions only; yes for any accidental protected execution path

This build is public conversational behavior.

It must not add deterministic protected execution.

It must not weaken protected fail-closed behavior.

5. Required AGENTS law constraints

Codex must follow the active SELENE AGENTS LAW.

Non-negotiable constraints:

No Python.
No worktrees.
Work only in /Users/selene/Documents/Selene-OS.
Clean tree required before starting.
Fetch required before claiming origin/main equality.
Existing files are read-only by default unless explicitly approved by this instruction.
No real searched-name hardcoding.
No provider calls unless existing provider gates allow them.
Normal tests must not call live providers.
Desktop must not gain semantic decision authority.
No surprise refactors.
No broad rewrites.
No docs-only build.
No vacuous test passes.
Exact runnable tests must execute nonzero tests.
Voice-first smoke is required where practical.
Typed real adapter/runtime endpoint smoke is acceptable fallback if native app/voice smoke is blocked and the blocker is reported clearly.
Stale/dead surfaces directly related to this repair must be classified and removed if in scope.

6. Approved implementation scope

Codex is approved to inspect and edit only the minimum required files inside this scope.

Primary likely files:

crates/selene_adapter/src/lib.rs
crates/selene_os/src/ph1x.rs
crates/selene_engines/src/ph1x.rs
crates/selene_kernel_contracts/src/ph1x.rs

Conditional files, only if repo truth proves they are required:

crates/selene_os/src/app_ingress.rs
crates/selene_os/src/ph1l.rs
crates/selene_kernel_contracts/src/ph1l.rs

Test-only files may be edited or added only where the repo already places related tests.

Desktop/iPhone files are not approved for semantic context repairs.

Desktop/iPhone may only be touched if Codex proves a renderer/bridge test must be updated and gets explicit approval before editing them.

PH1.M files are not approved for this build except read-only boundary proof.

PH1.M inspection must stay narrow. Do not repeat a broad memory audit. Do not inspect PH1.M beyond what is needed to prove:

No MemoryOperation::Propose is required for “Sydney”.
No PH1.M thread digest is required for “Sydney”.
No PH1.M recall candidate is required for “Sydney”.
No MemoryOperation::Propose is required for “What about London?”.
No PH1.M thread digest is required for “What about London?”.
No PH1.M recall candidate is required for “What about London?”.

PH1.M must remain read-only for this build.

7. Hard stop conditions

Codex must stop and report instead of editing if any of these are true:

START_TREE_DIRTY
WRONG_REPO
REMOTE_TRUTH_UNVERIFIED when remote equality is required
PYTHON_REQUIRED
PH1.M_REQUIRED_FOR_ACTIVE_CONTEXT
DESKTOP_SEMANTIC_CONTEXT_REQUIRED
PH1.L_SPINE_CHANGE_REQUIRED_BEYOND_SESSION_BOUNDARY
CONTRACT_SEMANTIC_CHANGE_REQUIRED_WITHOUT_EXPLICIT_APPROVAL
NO_LAWFUL_IMPLEMENTATION_SEAM_FOUND
BASELINE_TEST_RED_BEFORE_EDITS
PROVIDER_CALL_REQUIRED_FOR_NORMAL_TEST

If the repair requires changing protected execution, simulation dispatch, authority gates, PH1.M durable memory semantics, or Desktop semantic behavior, stop.

8. Required preflight

Before editing, Codex must run shell-only inspection.

Required commands:

git status --short
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git fetch origin main
git rev-parse origin/main

Then confirm:

repo path = /Users/selene/Documents/Selene-OS
branch = main
working tree = clean
HEAD == origin/main, if fetch succeeds

If xcodebuild will be run later, remove repo-local build artifact trees before final cleanliness proof:

apple/mac_desktop/build
apple/iphone/build

9. Required first-read files

Before editing, Codex must read the repo architecture files required by AGENTS law:

docs/CORE_ARCHITECTURE.md
docs/SELENE_BUILD_EXECUTION_ORDER.md
docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md

Codex must also inspect the current owners for this specific build:

crates/selene_kernel_contracts/src/ph1x.rs
crates/selene_os/src/ph1x.rs
crates/selene_engines/src/ph1x.rs
crates/selene_adapter/src/lib.rs
crates/selene_os/src/ph1l.rs
crates/selene_os/src/app_ingress.rs
crates/selene_kernel_contracts/src/ph1m.rs
crates/selene_os/src/ph1m.rs

PH1.M inspection is read-only boundary verification only.

10. Ownership map required before edits

Codex must produce this ownership map before editing:

Observed bug:
Same-session short follow-ups and pending clarification are not proven live end-to-end.

Owning engine/module:
PH1.X active conversation context plus adapter runtime bridge.

Session boundary owner:
PH1.L only for session open/resume/close/timeout/no-leak boundary.

Memory owner:
PH1.M for governed durable memory, 72-hour recall, archive, medium-term memory, permanent memory.
PH1.M is excluded from solving “Sydney” and “What about London?” in this build.

Client owner:
Desktop/iPhone remain capture/playback/transport/render/runtime bridge only.

Files allowed:
Only approved implementation scope.

Proof path:
Typed/runtime tests, adapter tests, PH1.X tests, no-leak tests, Desktop non-authority proof, voice-first smoke or fallback smoke.

If current repo truth contradicts this map, Codex must stop and report the exact file/line conflict.

11. Baseline tests before edits

Codex must list exact runnable tests first using runner-native listing where names may be ambiguous.

Before every filtered baseline test and before every filtered post-edit verification test, Codex must run the appropriate cargo test -p ... -- --list or equivalent runner-native listing command, confirm the exact runnable test/filter exists, and prove the later execution ran a nonzero number of tests.

Any zero-filter command must be called out and must not be counted as proof.

Baseline must include related existing tests where available:

cargo test -p selene_adapter desktop_continuous_session -- --test-threads=1
cargo test -p selene_adapter memory -- --test-threads=1
cargo test -p selene_adapter voice_id -- --test-threads=1
cargo test -p selene_adapter false_transcript -- --test-threads=1
cargo test -p selene_adapter post_tts -- --test-threads=1
cargo test -p selene_os ph1_voice_id -- --test-threads=1

PH1.X-specific filters must also be discovered and run if nonzero:

cargo test -p selene_os ph1x -- --test-threads=1
cargo test -p selene_engines ph1x -- --test-threads=1

These PH1.X filters must first be checked with -- --list. If they run zero tests, Codex must report them as zero-filter/unproven and not count them as proof.

If an exact filter runs zero tests, it does not count.

If baseline is red, stop before edits and report.

12. Implementation requirements

12.1 Active segment / boundary key

This boundary must be additive only.

Codex must prefer existing identifiers first:

session_id
thread_key
turn_id
correlation_id

Only add a new PH1.X contract field or explicit segment field if repo truth proves existing identifiers cannot prevent same-session/no-leak failures.

Add or confirm a runtime-owned active conversation boundary sufficient to distinguish:

same live session / same active thread / same pending context
versus
new session / closed session / expired context / unrelated topic

This can use existing session/thread/turn/correlation identifiers if repo truth supports them.

Do not invent a broad new memory system.

If a new additive packet/field is required, it must be minimal, backward-compatible, and justified in the final report with exact repo evidence showing why existing session_id + thread_key + turn_id/correlation_id were insufficient.

Required behavior:

A pending clarification belongs only to the active session/thread/segment.
A short follow-up belongs only to the active session/thread/segment.
A new session must not inherit old pending weather/time context.
Recalled PH1.M memory must not become active pending intent unless explicitly routed by a future memory build.

12.2 Pending clarification slot repair

Repair the path:

User: What’s the weather like in Australia?
Selene: Which city?
User: Sydney.
Selene: fills the pending city/location slot with Sydney and routes the prior weather request to Sydney, Australia, in the same active session.

Required internal behavior:

weather request with missing city creates/keeps active pending clarification.
next short location answer fills the pending city/location slot.
pass condition is slot-fill + route correctness, not merely mentioning Sydney.
filled slot routes to the weather path.
pending slot clears after successful answer.
a later short phrase in the same session must not keep using the old pending city slot after it has been resolved.
pending slot does not become durable memory.

12.3 “What about X?” follow-up repair

Repair the path:

User: What time is it in New York?
Selene: answers New York time.
User: What about London?
Selene: answers London time.

Required internal behavior:

last answer type / intent / tool = time.
follow-up phrase “What about London?” inherits time intent.
location/entity becomes London.
resolved query routes to time tool/path.
last context updates to London after answer.

Also preserve weather follow-up where existing repo truth already supports it.

12.4 No-leak boundary

Add proof that context clears or is not reused across closed/new session boundaries.

Required scenario:

Session A:
Weather in Australia → city clarification → Sydney → answer.
Session A closes or new session starts.
Session B:
User: Sydney.
Expected: Selene does not assume old Australia/weather pending context.

Correct output may be a clarification, generic answer, or safe “what would you like to know about Sydney?” style response depending on existing PH1.X behavior.

Incorrect output:

Selene answers Sydney weather because of stale Session A context.

12.5 Topic switch behavior

If the user starts a clearly new topic, active context should not force old tool intent.

Example:

User: What time is it in New York?
Selene: answers.
User: Explain payroll rules.
Expected: normal new topic, not time follow-up.

This build does not need full advanced topic detection, but it must not make old context sticky in obvious new-topic cases.

12.6 Correction behavior, minimal only

Add minimal support if naturally adjacent and within the same owner path:

No, I meant Sydney.
Not weather, time.
Same question for London.

If full correction support requires broader work, preserve existing behavior and report deferred correction expansion.

12.7 Memory exclusion proof

This build must prove:

No PH1.M durable memory write is needed for “Sydney”.
No MemoryOperation::Propose is required for “Sydney”.
No PH1.M thread digest is required for “Sydney”.
No PH1.M recall candidate is required for “Sydney”.
No PH1.M durable memory write is needed for “What about London?”.
No MemoryOperation::Propose is required for “What about London?”.
No PH1.M thread digest is required for “What about London?”.
No PH1.M recall candidate is required for “What about London?”.
PH1.M memory_candidates, if present, do not fill active pending clarification by accident.
72-hour HOT memory is not used as active context.

If an existing harness allows memory candidates to be seeded or passed, add a test proving a PH1.M memory candidate cannot hijack active pending context. If the harness does not allow this, Codex must report the gap and prove the closest available boundary.

12.8 Desktop non-authority proof

If Desktop is not touched, prove with read-only grep that Desktop does not own:

pending slot resolution
weather/time intent rewrite
semantic memory write
identity decision
PH1.M proposal
provider routing
protected execution

If Desktop is touched, Codex must get explicit approval first and then prove Desktop remains renderer/bridge only.

13. Required tests to add or repair

Codex must add targeted tests that prove real behavior, not just helper existence.

13.1 Pending clarification test

Required test behavior:

Turn 1: “What’s the weather like in Australia?”
Expected: clean city clarification / pending place slot.

Turn 2 same session: “Sydney.”
Expected: pending city/location slot is filled by Sydney.
Expected: resolved weather route for Sydney / no generic Sydney answer.
Expected: pass is based on route/slot correctness, not merely mentioning Sydney.
Expected: pending slot cleared after answer.
Expected: a later short phrase does not reuse the old resolved pending slot.
Expected: no PH1.M durable memory write.

13.2 Time follow-up test

Required test behavior:

Turn 1: “What time is it in New York?”
Expected: time answer path.

Turn 2 same session: “What about London?”
Expected: resolved time request for London.
Expected: not generic London search/chat.
Expected: not weather.

13.3 No-leak test

Required test behavior:

Session A completes Australia → Sydney.
Session B/new session receives “Sydney.”
Expected: no old weather intent inherited.

13.4 Topic switch test

Required test behavior:

Time/weather answer occurs.
Next user asks an unrelated topic.
Expected: old tool intent does not hijack new topic.

13.5 Memory boundary test

Required test behavior:

Same-session short follow-up resolution succeeds with PH1.X/adapter state.
PH1.M durable memory write/proposal is not required.
MemoryOperation::Propose is not required.
PH1.M thread digest is not required.
PH1.M recall candidate is not required.
PH1.M recall candidates do not mutate pending slot state.
If the harness supports seeded memory_candidates, seed/pass one and prove active pending context still wins only when same-session context is valid.

13.6 Desktop boundary test or proof

Required behavior:

Desktop does not resolve “Sydney”.
Desktop does not rewrite “What about London?”.
Desktop does not write PH1.M memory.
Desktop only transports/renders runtime output.

A read-only proof is acceptable if Desktop is not edited.

14. Required verification commands

After implementation, Codex must run the strongest applicable proof pack.

Before each filtered command below, Codex must run the runner-native listing command and prove nonzero matching tests exist. After execution, Codex must prove nonzero tests actually ran.

Minimum required:

cargo check -p selene_os -p selene_adapter -p selene_engines -p selene_kernel_contracts
cargo test -p selene_adapter desktop_continuous_session -- --test-threads=1
cargo test -p selene_adapter memory -- --test-threads=1
cargo test -p selene_adapter voice_id -- --test-threads=1
cargo test -p selene_adapter false_transcript -- --test-threads=1
cargo test -p selene_adapter post_tts -- --test-threads=1
cargo test -p selene_os ph1_voice_id -- --test-threads=1

Explicit PH1.X filters to discover and run if nonzero:

cargo test -p selene_os ph1x -- --test-threads=1
cargo test -p selene_engines ph1x -- --test-threads=1

Add all new exact targeted tests created for this build.

If this build touches PH1.X engine/OS behavior, also run relevant PH1.X/PH1.L tests discovered from repo truth.

Full suite requirement if scope touches central adapter/OS behavior:

cargo test -p selene_adapter -- --test-threads=1
cargo test -p selene_os -- --test-threads=1
cargo test -p selene_engines -- --test-threads=1

If full suite is too slow or blocked by existing unrelated repo truth, Codex must state exactly what ran, what did not run, and why.

No zero-test command counts as proof.

15. Smoke proof requirement

Voice-first smoke is required if technically practical.

Preferred smoke path:

Real Desktop app voice/microphone smoke.

Required spoken sequence if voice is available:

1. “What’s the weather like in Australia?”
2. “Sydney.”
3. “What time is it in New York?”
4. “What about London?”
5. Start/force new session.
6. “Sydney.”

Codex must record:

exact captured transcript
normalized intent/route
session/thread/segment key evidence
answer class/path
TTS text if voice-originated
whether PH1.M was not used for active resolution
whether Desktop only transported/rendered

If voice smoke is not practical, Codex must run typed app smoke through the real app UI if practical.

If native app smoke is blocked, typed real adapter/runtime endpoint smoke is acceptable for this build because the core behavior is context routing. Codex must clearly report the blocker, the fallback smoke path, and exactly what the fallback proves.

If app UI smoke is not practical, Codex must run the closest authoritative runtime/adapter endpoint smoke and state the blocker plainly.

Unit tests and cargo check do not replace smoke.

16. Provider/tool rules

Weather/time may use existing read-only public tool/provider behavior if repo truth already supports it.

However:

Do not enable live paid providers.
Do not add provider calls to normal tests.
Do not bypass provider kill switches.
Do not call Brave for this build.
Do not let provider failure become protected execution failure.
Use existing fake/tool fixtures where tests require weather/time proof.

This build is about context routing, not provider quality.

Provider-off acceptance rule:

Tests may pass by proving that the utterance resolved into the correct weather/time route with Sydney/London, even if provider/search/tool execution is disabled and the final answer safe-degrades.

Do not require live weather/time quality to pass this build.

Acceptance is route/context correctness, pending-slot correctness, no-leak correctness, and memory-boundary correctness.

17. Stale/dead surface cleanup requirement

Before commit, Codex must search and classify directly related stale surfaces.

At minimum inspect symbols/paths related to:

deterministic_public_clarification_followup_query
deterministic_weather_context_followup_query
weather_context_state
public_discourse_frame
LastTurnContext
PendingState::Clarify
ResumeBuffer
ThreadState
thread_key
session_id
memory_candidates
MemoryOperation::Propose
thread_digest

Classify each as exactly one:

STILL_ACTIVE_REQUIRED
DEAD_LOCAL_SURFACE
WRONG_OWNER_SURFACE
LEGACY_COMPATIBILITY_REQUIRED

Remove DEAD_LOCAL_SURFACE only if removal stays inside approved scope.

If wrong-owner cleanup requires broader scope, stop and report:

STALE_SURFACE_OWNER_SCOPE_REQUIRED

Final report must list stale symbols found, classifications, removals, kept-with-reason, and proof.

18. Documentation and ledger

Default is no docs change.

Docs/ledger should be updated only if repo law requires it for this implementation class or if the implementation materially changes current repo truth.

If docs/ledger are updated, update only minimum necessary truth.

Do not turn this into a docs-only reconciliation.

19. Commit rules

Commit only if all are true:

baseline passed before edits
implementation tests passed
new targeted tests passed with nonzero execution
smoke proof passed or environmental blocker documented with strongest fallback smoke
no PH1.M durable memory misuse
no MemoryOperation::Propose required for Sydney or What about London
no PH1.M thread digest required for Sydney or What about London
no PH1.M recall candidate required for Sydney or What about London
no Desktop semantic authority added
protected fail-closed behavior preserved
git diff --check passed
no unrelated files changed
stale/dead surface cleanup completed or blocker reported
final tree can be cleaned after commit/push

Suggested commit message:

Runtime: repair active session context

Use a segment-specific commit title only if a real additive segment field is added.

Push to origin after commit, then prove:

git status --short
git rev-parse HEAD
git rev-parse origin/main

Final state must be clean and HEAD == origin/main if push succeeds.

20. Final report required format

Codex final report must include:

A) Clean start proof
B) Fresh remote truth proof
C) Lane declaration
D) Ownership map
E) Current / target / gap
F) Files changed
G) Implementation summary
H) Active segment/session boundary proof, including whether existing IDs were sufficient or whether a new additive field was required
I) Pending clarification proof: Australia → Sydney fills the pending city/location slot, not merely mentions Sydney
J) Follow-up proof: New York time → What about London
K) No-leak proof: new session → Sydney does not inherit old context
L) Pending-clears proof: after Australia → Sydney resolves, a later short phrase does not reuse the old pending city slot
M) Topic-switch proof
N) Memory boundary proof: PH1.M not used for active context, including no MemoryOperation::Propose / PH1.M thread digest / PH1.M recall candidate required for Sydney or What about London
O) Desktop boundary proof
P) Tests run with list-first proof and nonzero execution counts
Q) Smoke path used and result, including voice/native-app blocker and typed runtime fallback if applicable
R) Provider/tool disclosure: real/fake/off, including provider-off route-correctness acceptance if used
S) Harness/mock/fixture disclosure
T) Stale/dead surface classification and cleanup
U) Protected fail-closed proof
V) What remains unproven/deferred
W) Commit hash and push status
X) Final clean tree proof

21. Explicit deferred items

This build must not attempt these:

72-hour searchable archive recall
medium-term project memory
permanent memory UX
governed voice memory proposal/write path
Voice ID enrollment/learning
name/preference/favorite memory UX
full correction engine
full topic-switch classifier
provider/search quality repair
Desktop redesign
iPhone redesign
PH1.M durable write expansion
PH1.M semantic search
DeepResearch
Brave live proof
image/provider work

These belong to later builds.

22. Next build after this one, if this passes

After this build passes, the next logical build is:

SELENE_72_HOUR_RECENT_MEMORY_AND_ARCHIVE_RECALL_FOUNDATION

Purpose:

Make the one visible long chat internally searchable by recent conversation segments without loading the entire chat into active context.

That later build should belong to:

PH1.M + storage/archive/search

Not PH1.X active session context.

23. Final instruction to Codex

Proceed with implementation only if repo truth confirms a lawful seam inside PH1.X + adapter runtime bridge, with PH1.L used only for session boundary/no-leak behavior.

Do not use PH1.M to solve same-session short follow-ups.

Do not put semantic context logic into Desktop/iPhone.

Do not perform another broad audit.

Do not drift into memory architecture implementation.

Build the smallest complete active-session repair that proves:

Australia weather → Sydney fills the pending city/location slot and routes to weather.
Pending weather slot clears after Sydney resolves.
New York time → What about London routes to London time.
Provider-off/fake-tool safe-degrade is acceptable if route/context proof is clear.
New session → Sydney does not leak old context.
PH1.M is not required for Sydney or What about London.
Desktop/iPhone do not gain semantic authority.

That is the acceptance gate for this build.
