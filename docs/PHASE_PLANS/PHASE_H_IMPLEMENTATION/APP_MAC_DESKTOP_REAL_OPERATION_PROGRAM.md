# APP_MAC_DESKTOP Real-Operation Program

## Status
- Umbrella planning document only.
- This document does not override [MASTER_BUILD_COMPLETION_PLAN.md](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md).
- This document does not itself authorize code changes.
- This document defines the end-to-end desktop-first operational target and the execution program that later strict H-build slices must follow.

## Purpose
- Make the macOS app a real Selene client, not a bounded evidence shell.
- Reuse the existing Selene runtime, onboarding, access, wake, search, delivery, and law stack already present in the repo.
- Avoid fake local reasoning, fake search, fake wake, or any alternate Swift-side assistant architecture.
- Define the full desktop-first destination before selecting bounded implementation slices underneath it.

## Governing Law
- Build must proceed from the runtime kernel outward, not app-first, feature-first, or UI-first: [SELENE_BUILD_EXECUTION_ORDER.md#L11](/Users/selene/Documents/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L11)
- H1 requires strict dependency order, first-class but non-authority clients, and no Apple workaround architecture: [H1_BUILD_METHOD_AND_IMPLEMENTATION_SEQUENCE.md#L17](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H1_BUILD_METHOD_AND_IMPLEMENTATION_SEQUENCE.md#L17)
- Current master truth remains authoritative for landed state and still says APP_MAC_DESKTOP is partial and post-H236 remains `NOT_EXPLICIT`: [MASTER_BUILD_COMPLETION_PLAN.md#L96](/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md#L96)
- Current architecture-owned wiring truth still marks `PH1.J`, `PH1.M`, `PH1.OS`, and `PH1.COMP` as partially wired, with `PH1.LAW` fully wired: [COVERAGE_MATRIX.md#L7](/Users/selene/Documents/Selene-OS/docs/COVERAGE_MATRIX.md#L7)
- Exact executable-unit and anti-loop law remain governed by H104, H112, and H237:
  - [H104_POST_H103_TERMINAL_NOT_EXPLICIT_COMPLETION_MODE_OVERRIDE_SELECTION_LAW_CHANGE_BUILD_PLAN.md](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H104_POST_H103_TERMINAL_NOT_EXPLICIT_COMPLETION_MODE_OVERRIDE_SELECTION_LAW_CHANGE_BUILD_PLAN.md)
  - [H112_POST_H111_PERMANENT_EXECUTABLE_UNIT_ANTI_LOOP_SELECTION_LAW_CHANGE_AND_SECTION05_RESIDUAL_FRONTIER_RECLASSIFICATION_BUILD_PLAN.md](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H112_POST_H111_PERMANENT_EXECUTABLE_UNIT_ANTI_LOOP_SELECTION_LAW_CHANGE_AND_SECTION05_RESIDUAL_FRONTIER_RECLASSIFICATION_BUILD_PLAN.md)
  - [H237_POST_H236_PERMANENT_SAME_FRONTIER_DOC_ONLY_PUBLICATION_LOOP_FREEZE_SELECTION_LAW_CHANGE_BUILD_PLAN.md](/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H237_POST_H236_PERMANENT_SAME_FRONTIER_DOC_ONLY_PUBLICATION_LOOP_FREEZE_SELECTION_LAW_CHANGE_BUILD_PLAN.md)

## Product Target
Desktop Selene is considered real when all of the following are true:
- the macOS app can onboard a real user and device through the existing cloud-authoritative path
- employee / position / access and per-user access posture work through the existing PH1 stacks
- the app can obtain lawful microphone and speech permissions
- the app can capture audio and produce a lawful desktop `AppVoiceIngressRequest`
- the app can dispatch turns into the canonical desktop runtime path
- the app can route web search and other tool answers through the real tool lane
- the app can render authoritative responses in a conversation-first desktop UI
- the app can speak authoritative reply text aloud
- the app can lawfully configure and complete desktop wake enrollment
- the app can run a real native wake listener and hand wake detections into the canonical runtime path
- the app can send an invite link to iPhone through the existing link and delivery stack
- the iPhone can receive that link and continue through the existing onboarding/open path

## Current Repo Baseline
- The native macOS app exists in-tree:
  - [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop.xcodeproj](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop.xcodeproj)
  - [/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift)
- The current Mac shell is still evidence-first and explicitly non-authority:
  - wake evidence surface: [DesktopSessionShellView.swift#L2324](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L2324)
  - read-only onboarding receipt posture: [DesktopSessionShellView.swift#L2221](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift#L2221)
- Canonical desktop runtime entry already exists:
  - [app_ingress.rs#L3186](/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs#L3186)
- Desktop send-link dispatch is already proven in backend tests:
  - [app_ingress.rs#L19038](/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs#L19038)
- Desktop web search dispatch with provenance is already proven in backend tests:
  - [app_ingress.rs#L19121](/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs#L19121)
- Desktop onboarding already requires platform receipts:
  - `install_launch_handshake`
  - `mic_permission_granted`
  - `desktop_wakeword_configured`
  - `desktop_pairing_bound`
  at [ph1f.rs#L11571](/Users/selene/Documents/Selene-OS/crates/selene_storage/src/ph1f.rs#L11571)
- Wake enrollment is already required for desktop:
  - [app_ingress.rs#L6457](/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs#L6457)
- The current Mac app still lacks:
  - microphone / speech permission keys in [Info.plist](/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/Info.plist)
  - local mic capture implementation
  - speech recognition implementation
  - TTS playback implementation
  - native canonical runtime bridge from the app shell
  - native onboarding mutation flow
  - native wake listener integration
  - wake-to-turn handoff
  - conversation-first ChatGPT-style app layout

## End-to-End Desktop Runtime Chain
1. A user and device enter through the existing link and onboarding path.
2. Desktop platform receipts, pairing, voice enrollment, and wake enrollment complete through the existing PH1.ONB / PH1.VOICE.ID / PH1.W / PH1.ACCESS path.
3. The desktop app obtains microphone and speech permissions.
4. A local entry path starts:
   - either explicit capture
   - or wake-word activation once the wake listener is real
5. Local audio enters the voice intake substrate.
6. The app constructs a lawful desktop `AppVoiceIngressRequest`.
7. The app calls the canonical desktop runtime path in [app_ingress.rs#L3186](/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs#L3186).
8. Runtime intent resolution may produce:
   - respond
   - clarify
   - refuse
   - search/tool response
   - simulation-gated dispatch such as send-link
9. PH1.X returns the authoritative output posture.
10. The desktop app renders the response in the conversation UI.
11. The desktop app requests TTS playback for authoritative spoken output when lawful.
12. If the request is "send a link to my iPhone", the runtime uses PH1.LINK + PH1.BCAST + PH1.DELIVERY and the iPhone receives the invite link.

## Full Engine Inventory And Desktop Role Map

### 1. Runtime Foundation, Session, Proof, Governance, and Law
- `PH1.F`: foundational persistence and shared runtime storage substrate
- `PH1.J`: proof / audit / protected-path evidence carriage
- `PH1.L`: canonical session engine and session authority container
- `PH1.GOV`: governance decisions and runtime governance integration
- `PH1.QUOTA`: quota policy and bounded consumption decisions
- `PH1.WORK`: work order / work execution support
- `PH1.LEASE`: lease ownership and single-writer discipline
- `PH1.OS`: OS orchestration and canonical turn-level operating boundary
- `PH1.HEALTH`: runtime health and display-only health reporting
- `PH1.SCHED`: scheduling support
- `PH1.EXPORT`: export artifact boundary
- `PH1.KMS`: key and protected material handling
- `PH1.LAW`: final runtime law and independent-verification support

Desktop role:
- These engines are the substrate the Mac app must consume.
- The Mac app must remain downstream of these engines and must not replace them locally.

### 2. Voice Intake, Recognition, Decision, Response, and Speech
- `PH1.K`: voice runtime I/O and pre-roll / timing / stream contracts
- `PH1.LISTEN`: signal collection and filtering
- `PH1.C`: transcript quality gate and trusted transcript-or-reject boundary
- `PH1.NLP`: intent draft, clarify, and no-guess normalization
- `PH1.D`: deterministic decision layer
- `PH1.X`: response / directive / dispatch shaping
- `PH1.WRITE`: presentation-only formatting with no meaning drift
- `PH1.TTS`: authoritative speech output boundary
- `PH1.ENDPOINT`: endpointing and turn-edge support
- `PH1.LANG`: language handling
- `PH1.PRON`: pronunciation packs and pronunciation-aware delivery
- `PH1.SRL`: semantic role and repair support
- `PH1.DIAG`: diagnostics and consistency checking
- `PH1.COST`: cost / pacing / bounded degrade-only control

Desktop role:
- These engines form the real voice hot path for desktop Selene.
- The Mac app must bridge into this path, not create a second assistant stack.

### 3. Identity, Onboarding, Wake, Position, and Access
- `PH1.LINK`: invite generation, invite open, and token-driven entry
- `PH1.ONB`: onboarding session orchestration
- `PH1.POSITION`: employee / role / position schema and requirements
- `PH1.VOICE.ID`: voice identity enrollment and voice identity posture
- `PH1.W`: wake enrollment and wake profile runtime
- `PH1.ACCESS.001_PH2.ACCESS.002`: access and step-up authority
- `PH1.CAPREQ`: capability request lifecycle
- `PH1.TENANT`: tenant context and tenant policy boundary
- `PH1.POLICY`: additional policy control surface preserved in the repo inventory

Desktop role:
- This is how employee setup, user access, pairing, and wake readiness become real on desktop.
- This is also the lawful path for invite-to-onboarding and per-user access creation.

### 4. Tools, Search, Evidence, and Research
- `PH1.E`: tool lane and read-only tool dispatch
- `PH1.SEARCH`: search planning, rewrite, and evidence path
- `PH1.DOC`: document understanding
- `PH1.SUMMARY`: evidence-backed summary building
- `PH1.VISION`: image / video / visual understanding
- `PH1.PRUNE`: ambiguity and candidate narrowing
- `PH1.PREFETCH`: read-only prefetch support
- `PH1.EXPLAIN`: explanation and reason rendering support

Desktop role:
- These engines power real web search, deep research, and evidence-backed answers.
- Desktop must surface them through the canonical runtime and provenance-rich UI.

### 5. Memory, Context, Knowledge, Learning, and Multimodal Support
- `PH1.M`: memory ledger and retrieval
- `PH1.FEEDBACK`: feedback collection
- `PH1.LEARN`: learning artifact building
- `PH1.PAE`: provider and adaptation policy
- `PH1.CACHE`: cache hinting and bounded advisory reuse
- `PH1.KNOW`: knowledge packs and dictionaries
- `PH1.MULTI`: multimodal bundle composition
- `PH1.CONTEXT`: context bundle building
- `PH1.KG`: knowledge graph linkage
- `PH1.PATTERN`: offline pattern mining
- `PH1.RLL`: offline ranking / recommendation learning

Desktop role:
- These engines improve continuity, recall, personalization, and advisory behavior without granting local authority to the app.

### 6. Emotion, Persona, and Delivery Style
- `PH1.EMO.GUIDE`: emotional guidance profile building
- `PH1.EMO.CORE`: emotional state core
- `PH1.PERSONA`: persona profile and bounded delivery behavior

Desktop role:
- These shape how Selene sounds and feels.
- They must remain tone/delivery-only and may not become truth or execution authority.

### 7. Delivery, Broadcast, Reminder, and User Follow-Through
- `PH1.REM`: reminder lifecycle
- `PH1.BCAST`: broadcast and staged outbound communication
- `PH1.DELIVERY`: actual send / cancel delivery boundary

Desktop role:
- These make the desktop-first invite flow real.
- They also support follow-through operations such as reminders and outbound communications.

### 8. Quantitative / Computation Support
- `PH1.COMP`: deterministic quantitative computation and structured numeric output

Desktop role:
- Not the first desktop hot path, but part of the full system and relevant for structured evidence, analytics, or calculation-heavy responses.

### 9. Supporting Non-Engine Storage / Registry Groups Present In Repo Truth
- `PBS_TABLES`
- `SIMULATION_CATALOG_TABLES`
- `ENGINE_CAPABILITY_MAPS_TABLES`
- `ARTIFACTS_LEDGER_TABLES`
- `SELENE_OS_CORE_TABLES`
- `PH1.LEARN_FEEDBACK_KNOW (STORAGE_GROUP_ONLY)`

Desktop role:
- These are not separate user-facing engines, but they remain part of the real operational substrate for simulation, capability, audit, and storage correctness.

## Desktop Real-Operation Lanes

### Lane A. Native Desktop Audio, Permission, Recognition, and Playback Foundation
Goal:
- make the Mac app a lawful voice client

Includes:
- macOS microphone permission posture
- speech recognition permission posture
- local audio capture
- transcript preparation
- authoritative TTS playback
- local voice-state handling

Primary engines:
- `PH1.K`
- `PH1.LISTEN`
- `PH1.C`
- `PH1.TTS`
- `PH1.OS`

Outcome:
- the app can hear and speak lawfully

### Lane B. Canonical Runtime Bridge
Goal:
- make every desktop turn a real Selene runtime turn

Includes:
- desktop `AppVoiceIngressRequest` construction
- canonical runtime bridge
- authoritative response rendering
- clarify / refuse / respond / dispatch handling

Primary engines:
- `PH1.L`
- `PH1.C`
- `PH1.NLP`
- `PH1.D`
- `PH1.X`
- `PH1.WRITE`
- `PH1.DIAG`
- `PH1.OS`

Outcome:
- the desktop app becomes a real client of the existing runtime

### Lane C. Real Search, Evidence, and Tool Use
Goal:
- make desktop Selene useful beyond plain conversation

Includes:
- web search
- provenance rendering
- deep research and other tool-lane responses where already supported

Primary engines:
- `PH1.E`
- `PH1.SEARCH`
- `PH1.DOC`
- `PH1.SUMMARY`
- `PH1.VISION`
- `PH1.PRUNE`
- `PH1.PREFETCH`
- `PH1.EXPLAIN`
- `PH1.CONTEXT`

Outcome:
- desktop can ask the web and receive evidence-backed answers through the real Selene tool lane

### Lane D. Employee, User, Pairing, and Access Readiness
Goal:
- make user and employee identity/access actually work

Includes:
- invite entry
- onboarding session progression
- employee / role / position requirements
- per-user access creation
- pairing completion

Primary engines:
- `PH1.LINK`
- `PH1.ONB`
- `PH1.POSITION`
- `PH1.ACCESS.001_PH2.ACCESS.002`
- `PH1.CAPREQ`
- `PH1.TENANT`
- `PH1.POLICY`

Outcome:
- desktop user readiness becomes real and lawful

### Lane E. Voice Identity and Wake Readiness
Goal:
- make wake-word desktop Selene lawful and real

Includes:
- voice identity enrollment
- wake enrollment
- wake profile readiness
- wake suppression posture

Primary engines:
- `PH1.VOICE.ID`
- `PH1.W`
- `PH1.PRON`
- `PH1.K`
- `PH1.OS`
- `PH1.J`
- `PH1.GOV`

Outcome:
- the desktop app can move from explicit-only entry to wake-capable entry

### Lane F. Memory, Context, Persona, and Adaptive Quality
Goal:
- make desktop Selene feel persistent and personal without local authority drift

Includes:
- memory-backed continuity
- context bundling
- knowledge packs
- adaptive but bounded provider behavior
- persona and tone shaping
- learning and feedback capture

Primary engines:
- `PH1.M`
- `PH1.FEEDBACK`
- `PH1.LEARN`
- `PH1.PAE`
- `PH1.CACHE`
- `PH1.KNOW`
- `PH1.MULTI`
- `PH1.CONTEXT`
- `PH1.KG`
- `PH1.PATTERN`
- `PH1.RLL`
- `PH1.EMO.GUIDE`
- `PH1.EMO.CORE`
- `PH1.PERSONA`

Outcome:
- desktop Selene feels coherent over time while still staying cloud-authoritative

### Lane G. Delivery, Reminder, and Cross-Device Handoff
Goal:
- make desktop the launch surface for real cross-device workflows

Includes:
- send-link to iPhone
- reminder and follow-through operations
- delivery state rendering

Primary engines:
- `PH1.LINK`
- `PH1.BCAST`
- `PH1.DELIVERY`
- `PH1.REM`
- `PH1.X`

Outcome:
- desktop Selene can hand the user into the iPhone path through the real delivery stack

### Lane H. Governance, Proof, Quota, Law, and Hardening
Goal:
- make desktop operation real without relaxing system law

Includes:
- proof / audit carriage
- governance and step-up alignment
- quota and cost discipline
- health surfaces
- final law enforcement
- structured computation where required

Primary engines:
- `PH1.J`
- `PH1.GOV`
- `PH1.QUOTA`
- `PH1.COST`
- `PH1.HEALTH`
- `PH1.COMP`
- `PH1.LAW`

Outcome:
- desktop remains lawful, auditable, bounded, and cloud-authoritative even when made more real

## ChatGPT-Style Desktop Product Goal
The desktop app should become conversation-first and ChatGPT-like in layout and usability, while staying faithful to Selene law.

Primary UX targets:
- conversation list / session list
- main transcript pane
- clear voice-state indicator
- visible source/provenance treatment for search/tool answers
- settings / status area for permissions, onboarding, pairing, wake, and device readiness
- explicit voice entry remains available as fallback
- wake-word entry becomes the real hands-free target once implemented

This program does not authorize a UI-only redesign that ignores the runtime. The conversation-first UI must sit on top of the real runtime path.

## Program Phases

### Phase 1. Desktop Client Foundation
- app permission posture
- native audio capture substrate
- speech recognition substrate
- TTS playback substrate
- local voice state handling

### Phase 2. Canonical Desktop Runtime Bridge
- build the app-to-runtime bridge
- render authoritative outcomes
- support respond / clarify / refuse / dispatch

### Phase 3. Conversation-First Desktop Shell
- redesign the desktop shell around the transcript and conversation experience
- keep evidence/status surfaces as secondary support panes

### Phase 4. Search and Tool Completion
- wire web search and tool responses into the new desktop shell
- preserve provenance and retrieval metadata

### Phase 5. Onboarding, Employee, Pairing, and Access
- real invite handling
- onboarding mutation flow
- position / employee readiness flow
- per-user access creation
- pairing completion

### Phase 6. Voice Identity and Wake Enrollment
- voice enrollment
- wake enrollment
- wake profile readiness
- wake configuration completion

### Phase 7. Native Wake Listener
- local wake listener integration
- suppression handling
- wake-to-turn handoff into the canonical runtime

### Phase 8. Memory, Persona, and Adaptive Quality
- memory continuity
- context and knowledge assist
- bounded persona/tone integration
- feedback and learning capture

### Phase 9. Cross-Device Desktop-First Handoff
- desktop sends iPhone invite link
- iPhone receives and continues onboarding

### Phase 10. Hardening, Proof, Governance, and Law
- final audit, quota, health, and law verification across the desktop path

## Verification Journeys

### Journey 1. Explicit Desktop Talk
- open Mac app
- grant permissions
- speak
- runtime responds
- UI renders response
- TTS plays response

### Journey 2. Desktop Web Search
- ask a search question
- runtime routes to the existing search/tool lane
- response includes sources and retrieval provenance
- desktop speaks answer if lawful

### Journey 3. Employee / User Readiness
- open invite
- complete onboarding
- satisfy employee / position / access requirements
- reach ready posture

### Journey 4. Desktop Pairing and iPhone Handoff
- ask Selene on desktop to send the iPhone link
- runtime dispatches `LINK_INVITE` + `LINK_DELIVER_INVITE`
- iPhone receives link
- iPhone opens and continues onboarding

### Journey 5. Wake-Word Desktop Operation
- desktop wake enrollment is complete
- wake listener is active
- user speaks wake word
- wake handoff starts a lawful runtime turn
- Selene responds and speaks back

## Non-Negotiable Non-Claims
- no fake local assistant brain in Swift
- no desktop-local fake search stack
- no fake wake-word claim before native wake-listener integration is real
- no local authority over access, governance, law, or memory truth
- no UI-first architecture fork
- no generalized completion claim for `PH1.J`, `PH1.M`, `PH1.OS`, or `PH1.COMP` while repo truth still marks them partial
- no autonomous unlock claim until the system is truly built and verified for that posture

## Completion Definition
This program is complete when:
- desktop onboarding is real
- employee / user access is real
- pairing is real
- speech recognition is real
- canonical runtime dispatch is real
- web search is real
- spoken reply playback is real
- wake enrollment is real
- wake listener is real
- wake-to-turn handoff is real
- iPhone invite handoff is real
- the desktop app is conversation-first rather than evidence-first

## Execution Method
- This umbrella program should be followed by strict H-build slices.
- Do not attempt to implement the entire program in one giant build.
- Do not treat this program doc as current-landed truth.
- Use this program as the destination map.
- Use later strict H-build instructions for:
  - selection
  - bounded implementation
  - verification
  - master-doc truth catch-up

## Recommended Next Step
- Use this document as the umbrella program.
- Next, mint the first strict APP_MAC_DESKTOP real-operation selection slice under it.
- The first selection target should be the smallest coherent desktop foundation cluster that lawfully unlocks:
  - permission posture
  - voice capture and speech recognition substrate
  - canonical desktop runtime bridge
  - authoritative reply rendering
  - authoritative reply playback
