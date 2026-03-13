PHASE F4 — IPHONE GOVERNANCE / LAW / PROOF REVIEW

A) REPO TRUTH CHECK
- task-state mode for this pass: `FRESH_AUTHORING`
- repo root: `/Users/xiamo/Documents/A-Selene/Selene-OS`
- branch at authoring start: `main`
- clean-tree truth at authoring start: `git status --short` empty
- target-file truth at authoring start: `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F4_IPHONE_GOVLAW.md` did not exist
- baseline proof before authoring: `bash scripts/selene_design_readiness_audit.sh` passed on clean tree
- authoritative review bundle consumed for this freeze:
  - F1 boundary: [F1_IPHONE_PARITY_REVIEW.md#L112](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L112), [F1_IPHONE_PARITY_REVIEW.md#L121](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L121), [F1_IPHONE_PARITY_REVIEW.md#L250](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L250)
  - F2 boundary: [F2_IPHONE_INGRESS_CONTRACT.md#L81](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F2_IPHONE_INGRESS_CONTRACT.md#L81), [F2_IPHONE_INGRESS_CONTRACT.md#L88](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F2_IPHONE_INGRESS_CONTRACT.md#L88), [F2_IPHONE_INGRESS_CONTRACT.md#L188](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F2_IPHONE_INGRESS_CONTRACT.md#L188)
  - F3 boundary: [F3_IPHONE_CONTINUITY.md#L80](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F3_IPHONE_CONTINUITY.md#L80), [F3_IPHONE_CONTINUITY.md#L183](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F3_IPHONE_CONTINUITY.md#L183), [F3_IPHONE_CONTINUITY.md#L195](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F3_IPHONE_CONTINUITY.md#L195)
  - core and build-order law: [CORE_ARCHITECTURE.md#L17](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md#L17), [CORE_ARCHITECTURE.md#L40](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md#L40), [CORE_ARCHITECTURE.md#L220](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md#L220), [CORE_ARCHITECTURE.md#L1996](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md#L1996), [SELENE_BUILD_EXECUTION_ORDER.md#L13](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L13), [SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L81](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L81), [SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L82](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L82), [SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L100](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L100), [SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L104](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L104), [SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L109](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md#L109)
  - repo-truth enforcement surfaces: [runtime_execution.rs#L1134](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs#L1134), [runtime_execution.rs#L1184](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs#L1184), [runtime_governance.rs#L23](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_governance.rs#L23), [runtime_governance.rs#L300](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_governance.rs#L300), [runtime_governance.rs#L601](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_governance.rs#L601), [runtime_law.rs#L29](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_law.rs#L29), [runtime_law.rs#L210](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_law.rs#L210), [runtime_law.rs#L528](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_law.rs#L528), [ph1os.rs#L576](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs#L576), [ph1os.rs#L1664](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs#L1664), [ph1j.rs#L198](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1j.rs#L198), [ph1f.rs#L5944](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs#L5944), [ph1_voice_id.rs#L9](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1_voice_id.rs#L9), [ph1_voice_id.rs#L314](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1_voice_id.rs#L314), [PH1_W.md#L15](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_W.md#L15)

B) PURPOSE
- freeze the iPhone governance, runtime-law, and proof-enforcement boundary only.
- preserve F1 platform truth, F2 ingress truth, and F3 continuity truth without reopening them.
- define how an eventual iPhone client may participate in protected completion, safe-fail posture, and proof-linked enforcement while remaining a first-class platform terminal and never an authority source.
- stop before F5.

C) DEPENDENCY RULE
- Section `01` remains the cloud-authoritative root: iPhone may observe, cache, carry, and surface results, but it may not author session truth, identity truth, memory truth, governance truth, proof truth, or runtime-law truth: [SELENE_BUILD_SECTION_01.md#L41](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md#L41), [CORE_ARCHITECTURE.md#L17](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md#L17)
- Section `02` keeps PH1.L as the only session owner. F4 may gate protected continuation on lawful session posture, but it may not mutate or fork session state: [SELENE_BUILD_SECTION_02.md#L23](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_02.md#L23), [D4_LAW_GOVERNANCE_ALIGNMENT_BUILD_PLAN.md#L82](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D4_LAW_GOVERNANCE_ALIGNMENT_BUILD_PLAN.md#L82)
- Section `03` remains a frozen predecessor seam only. F4 consumes the exact F2 `AppVoiceIngressRequest` predecessor and the exact iOS setup-receipt family already frozen in F2; it may not reinterpret ingress, request shape, or setup receipts: [SELENE_BUILD_SECTION_03.md#L23](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md#L23), [SELENE_BUILD_SECTION_03.md#L49](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md#L49), [F2_IPHONE_INGRESS_CONTRACT.md#L81](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F2_IPHONE_INGRESS_CONTRACT.md#L81), [F2_IPHONE_INGRESS_CONTRACT.md#L84](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F2_IPHONE_INGRESS_CONTRACT.md#L84), [F2_IPHONE_INGRESS_CONTRACT.md#L108](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F2_IPHONE_INGRESS_CONTRACT.md#L108), [F2_IPHONE_INGRESS_CONTRACT.md#L120](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F2_IPHONE_INGRESS_CONTRACT.md#L120), [app_ingress.rs#L127](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L127), [ph1f.rs#L11503](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs#L11503)
- Section `04` remains the protected authority seam. F4 may consume canonical authority state, artifact trust state, proof refs, governance state, and law state only after those surfaces are carried in the runtime envelope; it may not invent a second authority or proof path: [SELENE_BUILD_SECTION_04.md#L23](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md#L23), [SELENE_BUILD_SECTION_04.md#L57](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md#L57), [runtime_execution.rs#L1184](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs#L1184)
- Section `05` keeps retry, replay, and outbox state non-authoritative. F4 may escalate stale or quarantined persistence posture, but it may not let replay invent a second mutation or proof bypass: [SELENE_BUILD_SECTION_05.md#L19](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md#L19), [SELENE_BUILD_SECTION_05.md#L35](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md#L35), [SELENE_BUILD_SECTION_05.md#L87](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md#L87)
- Section `06` keeps memory authority cloud-side. F4 may block protected completion on missing lawful memory posture, but it may not let client continuity or personality layers become memory authority: [SELENE_BUILD_SECTION_06.md#L21](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_06.md#L21), [runtime_law.rs#L347](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_law.rs#L347)
- Section `07` keeps identity enforcement cloud-side. F4 consumes PH1.VOICE.ID and session-bound identity posture only from canonical inputs and may not treat local client or wake posture as identity truth: [SELENE_BUILD_SECTION_07.md#L29](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_07.md#L29), [SELENE_BUILD_SECTION_07.md#L43](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_07.md#L43), [ph1_voice_id.rs#L9](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1_voice_id.rs#L9), [ph1_voice_id.rs#L314](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1_voice_id.rs#L314)
- Section `08` remains first-class-platform, `EXPLICIT_ONLY`, and no-wake-widening law for iPhone. F4 may enforce compatibility or trust posture but may not use governance/law/proof work to smuggle wake parity: [SELENE_BUILD_SECTION_08.md#L47](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md#L47), [SELENE_BUILD_SECTION_08.md#L119](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md#L119), [PH1_W.md#L15](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_W.md#L15), [ph1os.rs#L4678](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs#L4678)
- Section `09` keeps governance deterministic and envelope-bound. F4 may only use canonical governance rules, reason codes, certification state, drift signals, and quarantine posture already attached to the runtime envelope: [SELENE_BUILD_SECTION_09.md#L21](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md#L21), [SELENE_BUILD_SECTION_09.md#L45](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md#L45), [runtime_governance.rs#L23](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_governance.rs#L23), [runtime_governance.rs#L103](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_governance.rs#L103)
- Section `10` remains deterministic numeric/computation support only. F4 may consume carried computation or consensus posture if already lawful, but may not create a probabilistic or platform-local final judge: [SELENE_BUILD_SECTION_10.md#L19](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_10.md#L19)
- Section `11` remains the final runtime-law posture layer. F4 defines iPhone participation in that enforcement path, but PH1.LAW remains the only final response-class engine: [SELENE_BUILD_SECTION_11.md#L21](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md#L21), [SELENE_BUILD_SECTION_11.md#L53](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md#L53), [runtime_law.rs#L528](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_law.rs#L528)
- frozen A law remains in force:
  - A4 freezes PH1.J proof input law. F4 may consume only canonical proof state, ordered decision refs, session/turn identity, and proof linkage already produced by PH1.J. Raw client assertions, raw PH1.OS fields, or raw GOV fields are forbidden as proof truth: [A4_PH1J_PROOF_CAPTURE_TRUST_VERIFICATION_OUTCOMES_BUILD_PLAN.md#L68](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A4_PH1J_PROOF_CAPTURE_TRUST_VERIFICATION_OUTCOMES_BUILD_PLAN.md#L68), [A4_PH1J_PROOF_CAPTURE_TRUST_VERIFICATION_OUTCOMES_BUILD_PLAN.md#L75](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A4_PH1J_PROOF_CAPTURE_TRUST_VERIFICATION_OUTCOMES_BUILD_PLAN.md#L75), [A4_PH1J_PROOF_CAPTURE_TRUST_VERIFICATION_OUTCOMES_BUILD_PLAN.md#L214](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A4_PH1J_PROOF_CAPTURE_TRUST_VERIFICATION_OUTCOMES_BUILD_PLAN.md#L214)
  - A5 freezes GOV/LAW ownership. Section `04` decides, A3 transports, A4 proves, and A5 consumes. F4 therefore cannot introduce an alternate iPhone governance or law input path: [A5_GOV_LAW_ENFORCEMENT_TRUST_FAILURES_BUILD_PLAN.md#L47](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A5_GOV_LAW_ENFORCEMENT_TRUST_FAILURES_BUILD_PLAN.md#L47), [A5_GOV_LAW_ENFORCEMENT_TRUST_FAILURES_BUILD_PLAN.md#L166](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A5_GOV_LAW_ENFORCEMENT_TRUST_FAILURES_BUILD_PLAN.md#L166)
- frozen B law remains in force:
  - the mobile parity baseline may only feed canonical transport, proof, and enforcement surfaces and may not create weaker iPhone continuity, trust, or enforcement behavior than the frozen Android posture model: [B1_ANDROID_WAKE_MICROPHONE_GAP_REVIEW.md#L76](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B1_ANDROID_WAKE_MICROPHONE_GAP_REVIEW.md#L76), [B1_ANDROID_WAKE_MICROPHONE_GAP_REVIEW.md#L89](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B1_ANDROID_WAKE_MICROPHONE_GAP_REVIEW.md#L89), [B2_ANDROID_TRIGGER_CAPTURE_PARITY_CONTRACT_BUILD_PLAN.md#L648](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B2_ANDROID_TRIGGER_CAPTURE_PARITY_CONTRACT_BUILD_PLAN.md#L648), [B2_ANDROID_TRIGGER_CAPTURE_PARITY_CONTRACT_BUILD_PLAN.md#L660](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B2_ANDROID_TRIGGER_CAPTURE_PARITY_CONTRACT_BUILD_PLAN.md#L660), [B3_ANDROID_RUNTIME_ENFORCEMENT_WIRING_BUILD_PLAN.md#L333](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B3_ANDROID_RUNTIME_ENFORCEMENT_WIRING_BUILD_PLAN.md#L333)
- frozen C law remains in force:
  - C4 freezes proof/governance/law visibility as non-authoritative, makes proof-required actions incomplete without PH1.J success, and makes proof failure a runtime-law failure: [C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L63](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L63), [C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L95](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L95)
- frozen D law remains in force:
  - D4 keeps PH1.L as writer, PH1.GOV as governance visibility/decision, PH1.LAW as final posture, PH1.J as proof/evidence visibility, and PH1.OS as orchestration posture. F4 may not drift any of those roles: [D4_LAW_GOVERNANCE_ALIGNMENT_BUILD_PLAN.md#L76](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D4_LAW_GOVERNANCE_ALIGNMENT_BUILD_PLAN.md#L76)
- frozen E law remains in force:
  - personality, memory, governance, and law remain control or decision layers only. F4 may consume these controls but may not let them become alternate authority or personality writers: [E4_SAFETY_LAW_MEMORY_CONTROLS_BUILD_PLAN.md#L66](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_E_PERSONALITY/E4_SAFETY_LAW_MEMORY_CONTROLS_BUILD_PLAN.md#L66)

D) ARCHITECTURAL POSITION
- F4 sits after frozen F3 continuity mechanics and before F5 verification closure.
- the canonical order remains:
  - F1 platform truth and non-authority boundary
  - F2 canonical ingress and setup-receipt truth
  - F3 session, continuity, and artifact-sync mechanics
  - F4 governance visibility, proof-linked enforcement, runtime-law posture, and safe-fail client participation
  - F5 verification, test evidence, and closure proof
- PH1.L remains authoritative writer for session truth.
- PH1.VOICE.ID remains authoritative identity engine for cloud-bound speaker posture.
- PH1.M remains authoritative memory engine.
- PH1.GOV remains deterministic governance visibility and response selection.
- PH1.J remains proof/evidence capture and linkage.
- PH1.LAW remains final runtime-law posture layer.
- PH1.OS remains orchestration posture and platform-law visibility only.

E) CURRENT ENFORCEMENT SURFACES IN SCOPE
- repo-truth surfaces directly relevant to F4 are:
  - `RuntimeExecutionEnvelope` carries governance, proof, identity, memory, authority, artifact-trust, and law state as one canonical enforcement object: [runtime_execution.rs#L1184](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs#L1184)
  - `ProofExecutionState` carries proof record ref, write outcome, failure class, chain status, verification posture, and verifier metadata ref: [runtime_execution.rs#L1134](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs#L1134)
  - PH1.OS validates platform/trigger/capability posture and then chains proof -> governance -> law in that order for protected completion: [ph1os.rs#L576](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs#L576), [ph1os.rs#L1726](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs#L1726)
  - PH1.J writes canonical append-only proof records and derives proof state from the receipt or failure: [ph1j.rs#L198](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1j.rs#L198), [ph1j.rs#L339](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1j.rs#L339)
  - runtime governance uses canonical reason codes, rules, and governance_state, and blocks, degrades, or quarantines on missing session, sequence, stale replay, quarantine, or missing proof: [runtime_governance.rs#L23](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_governance.rs#L23), [runtime_governance.rs#L300](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_governance.rs#L300), [runtime_governance.rs#L601](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_governance.rs#L601)
  - runtime law evaluates the envelope after governance and returns `ALLOW`, `ALLOW_WITH_WARNING`, `DEGRADE`, `BLOCK`, `QUARANTINE`, or `SAFE_MODE`: [runtime_law.rs#L29](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_law.rs#L29), [runtime_law.rs#L210](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_law.rs#L210), [runtime_law.rs#L528](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_law.rs#L528)
  - PH1.F proof storage is append-only, idempotent, chain-linked, and session/turn-bound: [ph1f.rs#L5944](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs#L5944)
  - PH1.VOICE.ID binds identity evaluation to session snapshot, device, wake context, and device trust inputs, not to local iPhone authority: [ph1_voice_id.rs#L314](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1_voice_id.rs#L314)

F) CANONICAL F4 ENFORCEMENT MODEL
- iPhone remains a first-class platform surface and a non-authority source.
- F4 freezes cloud-authoritative enforcement only:
  - PH1.OS may validate platform posture and assemble the lawful chain.
  - PH1.J may write canonical proof and return proof state.
  - PH1.GOV may classify, certify, block, degrade, quarantine, or safe-mode based on canonical envelope inputs only.
  - PH1.LAW may return the final protected response class from canonical inputs plus governance outputs only.
  - iPhone may receive, surface, retry, cache, and obey those postures, but may not reinterpret or author them.
- F4 therefore defines how a future iPhone client participates in protected completion and safe-fail behavior, not how it becomes an enforcement engine.

G) CURRENT / TARGET / GAP
- CURRENT: repo truth already contains cloud-side authority enforcement, governance reason codes, runtime-law response classes, proof capture, proof-ledger idempotency, and explicit iPhone platform-trigger validation, but no native iPhone client-side enforcement implementation exists in-tree today: [runtime_governance.rs#L23](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_governance.rs#L23), [runtime_law.rs#L29](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_law.rs#L29), [ph1os.rs#L576](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs#L576), [ph1j.rs#L198](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1j.rs#L198), [ph1f.rs#L5944](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs#L5944), [F1_IPHONE_PARITY_REVIEW.md#L239](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L239)
- TARGET: F4 freeze means one iPhone enforcement model where protected completion participates only through canonical authority state, governance state, proof state, law state, and exact predecessor ingress/setup surfaces from F2 plus continuity outputs from F3, while preserving first-class but non-authority, `EXPLICIT_ONLY`, cloud-authoritative parity, and no-wake-widening law: [F1_IPHONE_PARITY_REVIEW.md#L112](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L112), [F2_IPHONE_INGRESS_CONTRACT.md#L81](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F2_IPHONE_INGRESS_CONTRACT.md#L81), [F3_IPHONE_CONTINUITY.md#L183](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F3_IPHONE_CONTINUITY.md#L183), [SELENE_BUILD_SECTION_11.md#L21](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md#L21)
- GAP: native iPhone protected-action UI, client-visible proof/governance/law receipts, rollout verification, and end-to-end evidence remain deferred to F5 only. F4 does not claim a native iPhone implementation is present and does not reopen F1-F3 behavior: [F1_IPHONE_PARITY_REVIEW.md#L260](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L260), [F2_IPHONE_INGRESS_CONTRACT.md#L188](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F2_IPHONE_INGRESS_CONTRACT.md#L188), [F3_IPHONE_CONTINUITY.md#L183](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F3_IPHONE_CONTINUITY.md#L183)

H) AUTHORITY / GOVERNANCE / LAW / PROOF COVERAGE
Authority Coverage matrix

| protected concern | canonical authority source | iPhone/F4 role | forbidden drift |
| --- | --- | --- | --- |
| session authority | `PH1.L` and Section `02` session state | obey carried session posture only | local session mutation or local session writer |
| identity authority | `PH1.VOICE.ID` plus session-bound identity inputs | surface blocked/degraded identity outcomes only | local speaker truth or local identity override |
| memory authority | `PH1.M` plus lawful memory posture | obey memory-required block/degrade posture only | client-local memory authority or retention override |
| artifact authority | Section `04` artifact trust state and canonical refs | obey artifact trust required/failed/degraded posture only | local artifact trust path or raw artifact claim promotion |
| completion authority | PH1.GOV + PH1.LAW carried in the envelope | consume final response class only | local final judge or alternate approval path |

Governance Coverage matrix

| governance concern | repo-truth source | F4 rule | safe-fail posture |
| --- | --- | --- | --- |
| missing session or device sequence | `runtime_governance.rs` missing-session and sequence rules | protected iPhone completion must fail closed on missing canonical session or device ordering | `BLOCK` |
| degraded persistence replay | `runtime_governance.rs` degraded persistence rule | iPhone may continue only under carried degraded posture; it may not clear or hide degradation | `DEGRADE` |
| quarantined persistence | `runtime_governance.rs` quarantine rule | iPhone must preserve quarantine and stop protected continuation | `QUARANTINE` |
| proof missing or broken | `runtime_governance.rs` proof rule | iPhone protected completion is refused until canonical proof is written and linked | `BLOCK` or `QUARANTINE` |
| certification drift or safe mode | governance policy window, certification, and safe-mode state | iPhone must obey carried certification and safe-mode posture without local reinterpretation | `SAFE_MODE` or governed block |

Law Enforcement Coverage matrix

| runtime-law concern | repo-truth source | F4 rule | final response class family |
| --- | --- | --- | --- |
| session or admission required | `runtime_law.rs` envelope checks | protected iPhone actions require lawful session and admission posture | `BLOCK` |
| identity required | `runtime_law.rs` identity posture rules | iPhone protected flows must fail closed without canonical identity posture | `BLOCK` |
| memory required | `runtime_law.rs` memory posture rules | iPhone may not bypass cloud memory eligibility or retention limits | `BLOCK` |
| platform compatibility or trust | `runtime_law.rs` platform rules plus Section `08` | iPhone may be degraded or blocked on incompatibility/trust posture while staying `EXPLICIT_ONLY` | `DEGRADE` or `BLOCK` |
| governance safe mode or divergence | governance state consumed by PH1.LAW | iPhone must obey final safe-mode or cluster-divergence posture without a local exception path | `SAFE_MODE`, `DEGRADE`, or `BLOCK` |

Proof Capture Coverage matrix

| proof concern | repo-truth source | F4 rule | forbidden drift |
| --- | --- | --- | --- |
| proof write request | PH1.J protected proof request | F4 consumes only canonical proof write requests originating from lawful authority/gov inputs | local proof writer or local proof schema |
| proof receipt to proof state | `proof_execution_state_from_receipt` | iPhone consumes proof outcome only through carried `proof_state` | local derivation of proof success |
| proof idempotency and chain continuity | PH1.F append-only proof ledger | same protected action must reuse canonical proof or fail; no second proof history | duplicate local proof chain |
| proof policy linkage | PH1.J proof policy identifiers and version from governance state | proof remains linked to canonical governance rule and policy version | client-generated policy or rule refs |
| proof-linked artifact trust refs | A4/A5 and PH1.F proof entries | artifact trust proof linkage remains read-only canonical evidence | raw client receipt as proof truth |

I) FAILURE / QUARANTINE / SAFE-FAIL MODEL
- missing canonical session, missing device sequence, missing proof, or denied authority are fail-closed conditions.
- proof-chain integrity failures and proof-signature failures are quarantine-class failures, not advisory warnings: [runtime_governance.rs#L627](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_governance.rs#L627), [runtime_law.rs#L310](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_law.rs#L310)
- stale or quarantined persistence posture may escalate protected iPhone participation into `BLOCK`, `QUARANTINE`, or `SAFE_MODE`; it may not degrade into silent best-effort continuation: [runtime_governance.rs#L365](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_governance.rs#L365), [runtime_law.rs#L288](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/runtime_law.rs#L288), [C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L95](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L95)
- iPhone platform mismatch, trigger mismatch, or missing microphone capability must still fail closed in PH1.OS, and iOS wake remains unlawful: [ph1os.rs#L576](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs#L576), [ph1os.rs#L4678](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs#L4678)
- proof -> governance -> law order remains mandatory for protected completion. Governance or law visibility may not become alternate authority, and proof loss is not optional logging loss: [ph1os.rs#L1726](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs#L1726), [D4_LAW_GOVERNANCE_ALIGNMENT_BUILD_PLAN.md#L76](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D4_LAW_GOVERNANCE_ALIGNMENT_BUILD_PLAN.md#L76), [C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L100](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L100)

J) CROSS-DEVICE ENFORCEMENT DEPENDENCIES
Cross-Device Enforcement Dependencies matrix

| cross-device seam | upstream owner | F4 role | downstream result |
| --- | --- | --- | --- |
| attach/resume/detach continuity | F3 mechanics plus PH1.L | consume continuity posture only | protected completion obeys carried session truth |
| durable retry/outbox and replay | Section `05` plus F3 sync mechanics | escalate stale, degraded, or quarantined persistence posture only | no replay shortcut or duplicate authority |
| artifact sync ack/apply/dead-letter | F3 sync worker plus artifact truth | classify protected outcomes only from canonical sync and trust refs | no local artifact activation authority |
| identity-bound resume or follow-up | PH1.VOICE.ID and PH1.M inputs | require lawful identity and memory posture before protected continuation | no local resume override |
| platform and trigger posture | Section `08`, PH1.OS, PH1.W | preserve `EXPLICIT_ONLY` and iPhone no-wake law while allowing cloud-side enforcement decisions | no wake widening and no alternate platform law |

K) CURRENT CONFLICTS / GAPS
- `P1` no native iPhone client exists in repo truth today, so F4 defines cloud-authoritative enforcement participation only: [F1_IPHONE_PARITY_REVIEW.md#L239](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L239)
- `P1` repo truth already contains cloud-side governance, proof, and runtime-law enforcement, but there is no native iPhone receipt, UI, or event-feed consumer for those postures today.
- `P1` F4 therefore cannot lawfully claim end-to-end protected-action UX parity on iPhone yet; it can freeze the enforcement contract only.
- `P2` iPhone remains `EXPLICIT_ONLY`, and F4 may not reinterpret governance or law posture as wake enablement or wake-equivalent behavior: [SELENE_BUILD_SECTION_08.md#L119](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md#L119), [PH1_W.md#L15](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_W.md#L15)

L) F4 -> F5 FREEZE BOUNDARY
- F4 freezes iPhone governance, runtime-law, proof, authority-enforcement, quarantine, and safe-fail participation only.
- F4 may define how iPhone consumes carried governance, law, proof, identity, authority, artifact-trust, and persistence posture.
- F4 may not redefine F1 parity law, F2 ingress law, F3 continuity mechanics, Section `04` authority origin, Section `09` governance meaning, or Section `11` final law ownership.

Downstream F5 Freeze Boundaries matrix

| downstream phase | may define | must not redefine | freeze-boundary result |
| --- | --- | --- | --- |
| `F5` | tests, traces, verification evidence, docs reconciliation, freeze-pack proof for F1-F4 | any new runtime semantics, any alternate authority path, any wake widening, any weakening of F1-F4 obligations | F5 owns closure proof only |

M) COMPLETION CRITERIA
- F4 is complete only if all of the following are true:
  - iPhone remains first-class platform and non-authority source.
  - `EXPLICIT_ONLY` remains explicit and unchanged.
  - cloud-authoritative parity remains explicit and unchanged.
  - no native iPhone client is claimed as present in repo truth.
  - no unlawful wake parity claim is made.
  - `CURRENT`, `TARGET`, and `GAP` are explicit.
  - Sections `01-11`, frozen F1/F2/F3, frozen A4/A5, frozen B1-B3, frozen C4, frozen D4, and frozen E4 are explicit and preserved.
  - the exact F2 `AppVoiceIngressRequest` predecessor and the exact iOS setup-receipt family remain explicit predecessor inputs, unchanged in this freeze.
  - PH1.GOV and PH1.LAW remain control/decision layers only, not alternate authority writers.
  - PH1.J remains proof/evidence linkage only, not alternate authority.
  - all 6 required tables are present.
  - F5 remains explicit and frozen downstream only.
- F4 final truth:
  - design freeze-ready: `YES`
  - native iPhone implementation present in repo today: `NO`
  - remaining work beyond F4: verification and closure evidence remain deferred to `F5`
