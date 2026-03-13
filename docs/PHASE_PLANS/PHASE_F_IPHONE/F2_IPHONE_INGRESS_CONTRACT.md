PHASE F2 — IPHONE INGRESS CONTRACT

A) REPO TRUTH CHECK
- task-state mode for this pass: `FRESH_AUTHORING`
- repo root: `/Users/xiamo/Documents/A-Selene/Selene-OS`
- branch at authoring start: `main`
- clean-tree truth at authoring start: `git status --short` empty
- target-file truth at authoring start: `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F2_IPHONE_INGRESS_CONTRACT.md` did not exist
- baseline proof before authoring: `bash scripts/selene_design_readiness_audit.sh` passed on clean tree
- anchor bundle reviewed for this design:
  - core authority and platform law: [CORE_ARCHITECTURE.md#L17](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md#L17), [CORE_ARCHITECTURE.md#L40](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md#L40), [CORE_ARCHITECTURE.md#L78](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md#L78), [CORE_ARCHITECTURE.md#L118](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md#L118), [CORE_ARCHITECTURE.md#L1996](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md#L1996), [CORE_ARCHITECTURE.md#L2568](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md#L2568)
  - build-order and section anchors: [SELENE_BUILD_EXECUTION_ORDER.md#L13](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L13), [SELENE_BUILD_EXECUTION_ORDER.md#L31](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L31), [SELENE_BUILD_EXECUTION_ORDER.md#L69](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L69), [SELENE_BUILD_EXECUTION_ORDER.md#L101](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L101), [SELENE_BUILD_EXECUTION_ORDER.md#L201](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L201), [SELENE_BUILD_EXECUTION_ORDER.md#L233](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L233), [SELENE_BUILD_EXECUTION_ORDER.md#L265](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L265), [SELENE_BUILD_EXECUTION_ORDER.md#L295](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L295), [SELENE_BUILD_EXECUTION_ORDER.md#L329](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L329), [SELENE_BUILD_EXECUTION_ORDER.md#L359](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L359)
  - direct section law: [SELENE_BUILD_SECTION_01.md#L39](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md#L39), [SELENE_BUILD_SECTION_02.md#L21](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_02.md#L21), [SELENE_BUILD_SECTION_03.md#L21](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md#L21), [SELENE_BUILD_SECTION_04.md#L21](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md#L21), [SELENE_BUILD_SECTION_05.md#L19](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md#L19), [SELENE_BUILD_SECTION_07.md#L27](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_07.md#L27), [SELENE_BUILD_SECTION_08.md#L47](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md#L47), [SELENE_BUILD_SECTION_09.md#L19](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md#L19), [SELENE_BUILD_SECTION_10.md#L19](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_10.md#L19), [SELENE_BUILD_SECTION_11.md#L19](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md#L19)
  - repo-truth ingress/runtime anchors: [runtime_execution.rs#L220](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs#L220), [runtime_execution.rs#L236](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs#L236), [runtime_execution.rs#L251](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs#L251), [app_ingress.rs#L127](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L127), [app_ingress.rs#L180](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L180), [app_ingress.rs#L258](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L258), [app_ingress.rs#L281](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L281), [app_ingress.rs#L901](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L901), [ph1link.rs#L1466](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1link.rs#L1466), [ph1onb.rs#L239](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1onb.rs#L239), [ph1f.rs#L11503](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs#L11503), [device_artifact_sync.rs#L61](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/device_artifact_sync.rs#L61), [device_artifact_sync.rs#L91](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/device_artifact_sync.rs#L91), [ph1os.rs#L608](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs#L608), [PH1_W.md#L15](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_W.md#L15)
  - frozen upstream boundaries consumed unchanged: [F1_IPHONE_PARITY_REVIEW.md#L112](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L112), [F1_IPHONE_PARITY_REVIEW.md#L121](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L121), [F1_IPHONE_PARITY_REVIEW.md#L259](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L259), [C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L63](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L63), [D1_CROSS_DEVICE_SESSION_CONSISTENCY_REVIEW.md#L100](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D1_CROSS_DEVICE_SESSION_CONSISTENCY_REVIEW.md#L100), [D3_RUNTIME_AND_PERSISTENCE_WIRING_BUILD_PLAN.md#L62](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D3_RUNTIME_AND_PERSISTENCE_WIRING_BUILD_PLAN.md#L62), [D4_LAW_GOVERNANCE_ALIGNMENT_BUILD_PLAN.md#L76](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D4_LAW_GOVERNANCE_ALIGNMENT_BUILD_PLAN.md#L76), [E1_PERSONALITY_ARCHITECTURE_REVIEW.md#L61](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_E_PERSONALITY/E1_PERSONALITY_ARCHITECTURE_REVIEW.md#L61), [E4_SAFETY_LAW_MEMORY_CONTROLS_BUILD_PLAN.md#L66](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_E_PERSONALITY/E4_SAFETY_LAW_MEMORY_CONTROLS_BUILD_PLAN.md#L66)

B) PURPOSE
- freeze the iPhone ingress/runtime contract boundary for explicit-entry, app-open, capture, and setup-receipt surfaces only
- define how an eventual iPhone client is allowed to enter the existing cloud/runtime model without becoming a new authority surface
- preserve F1 truth that current parity is cloud/runtime parity only and that no native iPhone client exists in-tree today: [F1_IPHONE_PARITY_REVIEW.md#L119](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L119), [F1_IPHONE_PARITY_REVIEW.md#L239](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L239), [F1_IPHONE_PARITY_REVIEW.md#L274](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L274)
- stop before F3, F4, and F5

C) DEPENDENCY RULE
- Section `01` remains the authority boundary; F2 may design only client-to-cloud entry surfaces into that boundary, never a new authority path: [SELENE_BUILD_SECTION_01.md#L41](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md#L41)
- Section `02` and PH1.L remain the only session owner; F2 may reference `session_id`, `turn_id`, and session discovery/attach implications, but may not redefine session truth: [SELENE_BUILD_SECTION_02.md#L23](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_02.md#L23), [SELENE_BUILD_SECTION_02.md#L149](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_02.md#L149)
- Section `03` remains the canonical ingress family and runtime-envelope seam; F2 must terminate into that family and must not introduce an iPhone-only execution path: [SELENE_BUILD_SECTION_03.md#L21](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md#L21), [SELENE_BUILD_SECTION_03.md#L35](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md#L35), [SELENE_BUILD_SECTION_03.md#L49](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md#L49), [SELENE_BUILD_SECTION_03.md#L91](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md#L91)
- Sections `04` and `05` remain protected-authority and persistence law; F2 may design bounded evidence and idempotent request surfaces only: [SELENE_BUILD_SECTION_04.md#L21](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md#L21), [SELENE_BUILD_SECTION_05.md#L21](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md#L21), [SELENE_BUILD_SECTION_05.md#L35](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md#L35), [SELENE_BUILD_SECTION_05.md#L87](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md#L87)
- Sections `06` and `07` remain memory and identity boundaries; F2 must carry identity-scoped input into the envelope but may not create local identity or memory authority: [SELENE_BUILD_EXECUTION_ORDER.md#L201](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md#L201), [SELENE_BUILD_SECTION_07.md#L27](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_07.md#L27), [SELENE_BUILD_SECTION_07.md#L43](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_07.md#L43), [SELENE_BUILD_SECTION_07.md#L159](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_07.md#L159)
- Section `08` remains the iPhone posture anchor: first-class platform, explicit-entry only, no wake widening here: [SELENE_BUILD_SECTION_08.md#L47](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md#L47), [SELENE_BUILD_SECTION_08.md#L63](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md#L63), [SELENE_BUILD_SECTION_08.md#L119](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md#L119)
- Sections `09`, `10`, and `11` remain downstream authority consumers for governance, deterministic scoring, and final law posture: [SELENE_BUILD_SECTION_09.md#L19](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md#L19), [SELENE_BUILD_SECTION_10.md#L19](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_10.md#L19), [SELENE_BUILD_SECTION_11.md#L21](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md#L21)
- frozen C/D/E rules remain in force:
  - C4: proof and law visibility must not become alternate authority, and replay may not invent a second mutation: [C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L65](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L65), [C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L95](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L95)
  - D1/D3/D4: ingress normalization, runtime-envelope materialization, session-authority order, and governance/law visibility order stay frozen: [D1_CROSS_DEVICE_SESSION_CONSISTENCY_REVIEW.md#L101](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D1_CROSS_DEVICE_SESSION_CONSISTENCY_REVIEW.md#L101), [D3_RUNTIME_AND_PERSISTENCE_WIRING_BUILD_PLAN.md#L64](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D3_RUNTIME_AND_PERSISTENCE_WIRING_BUILD_PLAN.md#L64), [D4_LAW_GOVERNANCE_ALIGNMENT_BUILD_PLAN.md#L78](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D4_LAW_GOVERNANCE_ALIGNMENT_BUILD_PLAN.md#L78)
  - E1/E4: personality, memory, governance, and law remain downstream bounded consumers, never ingress authority: [E1_PERSONALITY_ARCHITECTURE_REVIEW.md#L61](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_E_PERSONALITY/E1_PERSONALITY_ARCHITECTURE_REVIEW.md#L61), [E4_SAFETY_LAW_MEMORY_CONTROLS_BUILD_PLAN.md#L66](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_E_PERSONALITY/E4_SAFETY_LAW_MEMORY_CONTROLS_BUILD_PLAN.md#L66)
- F2 scope is locked by F1: explicit-entry ingress contract, app-open/deep-link production, microphone/capture/session-start client contract, platform-context emission, and setup receipt production only: [F1_IPHONE_PARITY_REVIEW.md#L259](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L259)

D) ARCHITECTURAL POSITION
- iPhone remains a first-class platform surface, but never an authority source: [CORE_ARCHITECTURE.md#L17](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md#L17), [CORE_ARCHITECTURE.md#L40](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md#L40), [F1_IPHONE_PARITY_REVIEW.md#L112](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L112)
- F2 sits at the client-to-cloud seam only: it defines how iPhone-originated explicit entry, app-open, capture posture, and setup receipts must materialize lawful cloud/runtime requests.
- F2 does not create an iPhone-local session model, memory model, governance model, or law model.
- iPhone remains `EXPLICIT_ONLY`, so F2 covers explicit-entry parity only and must not claim wake parity: [SELENE_BUILD_SECTION_08.md#L119](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md#L119), [PH1_W.md#L15](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_W.md#L15), [F1_IPHONE_PARITY_REVIEW.md#L121](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L121)
- F2 is cloud/runtime parity design only. No native iPhone client is claimed or implemented here: [F1_IPHONE_PARITY_REVIEW.md#L123](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L123), [F1_IPHONE_PARITY_REVIEW.md#L239](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L239)

E) CURRENT REPO SURFACES IN SCOPE
- current F2 scope is limited to cloud/runtime surfaces that already exist in repo truth:
  - `RuntimeExecutionEnvelope` and `PlatformRuntimeContext`
  - `AppVoiceIngressRequest`
  - `AppInviteLinkOpenRequest`
  - `InviteOpenActivateCommitRequest`
  - `OnbSessionStartDraftRequest`
  - iOS setup receipt requirements in storage truth
  - `DeviceArtifactPullRequest`, `DeviceArtifactPullResponse`, and `DeviceArtifactSyncEnvelope`
  - existing microphone capability gate on live voice turns
- current F2 scope explicitly excludes:
  - any native iPhone capture-session implementation
  - any iPhone continuity/outbox/apply worker mechanics
  - any wake-path widening
  - any governance/law/proof execution redesign

Coverage matrix

| current repo surface | current role | F2 relevance | frozen note | current gap |
| --- | --- | --- | --- | --- |
| `RuntimeExecutionEnvelope` and `PlatformRuntimeContext` | canonical runtime-carried execution object | F2 must require every iPhone entry surface to produce or validate this envelope shape | Section `03` ingress and Section `08` platform posture remain canonical | no native iPhone emitter today |
| `AppVoiceIngressRequest` | explicit voice-entry request surface | F2 defines the iPhone explicit-entry contract against this shape | Section `02` session and Section `07` identity binding remain unchanged | no iPhone client request producer |
| `AppInviteLinkOpenRequest` | app-open/deep-link runtime entry surface | F2 defines the iPhone app-open contract against this shape | PH1.LINK and PH1.ONB own activation/onboarding outcomes | no iPhone deep-link/open producer |
| `InviteOpenActivateCommitRequest` and `OnbSessionStartDraftRequest` | bounded link activation and onboarding draft requests | F2 preserves device/app-instance/deep-link binding fields | client may not bypass activation or onboarding gates | no native iPhone caller |
| iOS setup receipt kinds in `ph1f.rs` | platform-specific onboarding completion evidence | F2 defines receipt production contract only | receipts are evidence, not authority | no iPhone receipt producer |
| `DeviceArtifactPullRequest`, `DeviceArtifactPullResponse`, `DeviceArtifactSyncEnvelope` | cloud/device artifact continuity surfaces | F2 may reference setup-linked sync surfaces but may not implement client queue/apply | Section `05` outbox/ack and F3 continuity remain downstream | no iPhone sync client |
| `PH1.W` explicit-trigger lock and wake suppression | current iPhone trigger posture | F2 must preserve `EXPLICIT_ONLY` and no wake claim | wake widening is out of scope here | no lawful wake parity |
| `ph1os` microphone capability gate | existing live voice entry guard | F2 capture contract must carry microphone capability when voice entry is attempted | live entry still fails closed without negotiated capability | no iPhone capture-session implementation |

F) CANONICAL IPHONE INGRESS CONTRACT MODEL
- F2 freezes the following ingress/runtime model:
  1. iPhone explicit entry is a lawful client-to-cloud request shape, not a new runtime path.
  2. iPhone app-open/deep-link is a lawful activation/onboarding entry shape, not a new onboarding authority.
  3. iPhone capture posture is upstream evidence only; capture metadata never becomes Section `04` authority.
  4. iPhone setup receipts are bounded platform evidence only; protected completion remains cloud-decided.
  5. iPhone artifact sync references may be minted or consumed by cloud/runtime surfaces, but client queue/apply mechanics remain outside F2.
  6. All iPhone entry surfaces remain `EXPLICIT_ONLY` unless a future governed phase changes that posture explicitly.
- explicit review frame for this freeze:
  - `CURRENT`: repo truth already provides `IOS` platform classification, explicit-trigger policy, `AppVoiceIngressRequest`, `AppInviteLinkOpenRequest`, activation/onboarding request fields, iOS setup receipt kinds, and artifact-sync envelope shapes: [runtime_execution.rs#L236](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs#L236), [app_ingress.rs#L127](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L127), [app_ingress.rs#L281](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L281), [ph1link.rs#L1466](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1link.rs#L1466), [ph1onb.rs#L239](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1onb.rs#L239), [ph1f.rs#L11503](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs#L11503), [device_artifact_sync.rs#L91](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/device_artifact_sync.rs#L91)
  - `TARGET`: F2 freeze certifies one enterprise-grade iPhone ingress contract boundary for explicit-entry, app-open, capture posture, and setup receipt production, all terminating into the existing cloud-authoritative runtime model without redefining session, authority, governance, or law.
  - `GAP`: F2 does not create a native iPhone client, microphone/capture-session implementation, durable retry queue, artifact apply worker, or protected-action enforcement client. Those remain explicitly deferred to F3-F5 by F1 boundary law: [F1_IPHONE_PARITY_REVIEW.md#L260](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L260), [F1_IPHONE_PARITY_REVIEW.md#L261](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L261), [F1_IPHONE_PARITY_REVIEW.md#L262](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L262)

Session / Runtime Envelope matrix

| contract field family | current repo truth | F2 freeze rule | fail-closed condition |
| --- | --- | --- | --- |
| `platform`, `platform_type` | request platform must match envelope platform | every iPhone entry surface must identify `IOS` consistently across request and envelope | mismatched platform blocks ingress |
| `requested_trigger`, `trigger_policy`, `trigger_allowed` | trigger policy is `EXPLICIT_ONLY` for iPhone | iPhone explicit-entry must declare `EXPLICIT` and must not present wake as lawful entry | wake or mismatched trigger posture blocks |
| `actor_identity`, `device_identity` | request actor and device must align to the envelope | iPhone request fields must bind actor and device scope before execution | actor/device mismatch blocks |
| `session_id`, `turn_id` | envelope/session refs must match request/session state refs | iPhone explicit entry may reuse or attach only through canonical session law | session mismatch or missing canonical session state blocks |
| `claimed_capabilities`, `negotiated_capabilities` | platform context carries capability posture | voice entry must carry negotiated `MICROPHONE` capability when live voice path is used | voice-live request without negotiated microphone blocks |
| `integrity_status`, `compatibility_status`, `attestation_ref` | platform context already carries integrity/compatibility fields | F2 preserves these as cloud-consumed posture inputs only | invalid or missing required posture may degrade or block later |
| capture posture metadata | capture artifact trust flags exist only as non-authoritative upstream posture | F2 may carry capture posture metadata, but it must never be treated as artifact authority | any design that upgrades capture posture into authority is unlawful |

G) EXPLICIT-ENTRY / APP-OPEN / CAPTURE / SETUP-RECEIPT SURFACES
- explicit-entry surface:
  - `AppVoiceIngressRequest` already defines the cloud-side entry shape for voice turns and can materialize a fallback runtime envelope for an app request: [app_ingress.rs#L141](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L141), [app_ingress.rs#L155](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L155)
  - request validation already enforces device, actor, platform, turn, and session alignment against the envelope: [app_ingress.rs#L193](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L193)
- app-open surface:
  - `AppInviteLinkOpenRequest` already defines token, tenant, platform, device fingerprint, app instance, deep-link nonce, and idempotency fields: [app_ingress.rs#L281](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L281), [app_ingress.rs#L293](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L293)
  - current runtime flow already activates the link, enforces tenant scope, requires active simulation chain, and then starts onboarding draft: [app_ingress.rs#L913](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L913), [app_ingress.rs#L948](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L948), [app_ingress.rs#L957](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L957), [app_ingress.rs#L999](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L999)
- capture surface:
  - live voice entry already requires negotiated `MICROPHONE` capability in the envelope: [ph1os.rs#L608](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs#L608)
  - `PlatformRuntimeContext` already carries non-authoritative capture posture metadata and must keep it non-authoritative: [runtime_execution.rs#L265](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs#L265)
  - no native iPhone microphone permission, interruption, route-change, or capture-session contract exists in repo truth today; F2 must name that gap rather than invent it: [F1_IPHONE_PARITY_REVIEW.md#L242](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L242), [F1_IPHONE_PARITY_REVIEW.md#L243](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L243)
- setup-receipt surface:
  - iOS requires `install_launch_handshake`, `push_permission_granted`, `notification_token_bound`, and `ios_side_button_configured`: [ph1f.rs#L11503](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs#L11503)
  - mobile signer scope is `selene_mobile_app`: [ph1f.rs#L11546](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs#L11546)
  - iOS wake override remains blocked unless explicitly allowed; F2 cannot weaken that: [ph1f.rs#L14469](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs#L14469)

Ingress Surface matrix

| ingress surface | current cloud-side request shape | F2 design freeze | explicit deferred gap |
| --- | --- | --- | --- |
| explicit voice entry | `AppVoiceIngressRequest` plus runtime envelope | iPhone voice entry must use this shape or an additive equivalent that preserves identical envelope alignment rules | no native iPhone request producer |
| app-open / deep-link entry | `AppInviteLinkOpenRequest` plus downstream PH1.LINK and PH1.ONB requests | iPhone app-open must remain device-bound, replay-safe, tenant-scoped, and idempotent | no native deep-link/open caller |
| invite activation commit | `InviteOpenActivateCommitRequest` fields | token, signature, device fingerprint, platform, app instance, deep-link nonce, and idempotency remain the canonical bounded inputs | no iPhone activation caller |
| onboarding session start draft | `OnbSessionStartDraftRequest` fields | device fingerprint, platform, app instance, deep-link nonce, and link-open time remain the canonical bounded inputs | no iPhone onboarding-draft caller |
| live capture entry | existing voice-live runtime path with negotiated microphone capability | capture entry must advertise lawful capability posture before voice execution starts | no iPhone capture-session contract |
| setup receipt production | storage truth for iOS receipt kinds and mobile signer | receipt production may prove platform setup only; it may not mint onboarding authority or wake authority | no iPhone receipt producer |

H) ARTIFACT SYNC / GOVERNANCE / LAW / PROOF PARTICIPATION
- artifact-sync participation:
  - `DeviceArtifactPullRequest` and `DeviceArtifactPullResponse` already define a bounded pull/update contract for device-side artifact versions and payload refs: [device_artifact_sync.rs#L61](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/device_artifact_sync.rs#L61), [device_artifact_sync.rs#L76](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/device_artifact_sync.rs#L76)
  - `DeviceArtifactSyncEnvelope` already carries `receipt_ref`, `artifact_profile_id`, optional onboarding session, user, device, attempt count, and idempotency key: [device_artifact_sync.rs#L91](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/device_artifact_sync.rs#L91)
  - sync kinds already include voice and wake artifacts, but F2 may only define the ingress/setup contract seam that could mint or consume these refs; F3 owns client continuity mechanics: [device_artifact_sync.rs#L149](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/device_artifact_sync.rs#L149), [F1_IPHONE_PARITY_REVIEW.md#L260](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L260)
- governance/law/proof participation:
  - governance attaches execution posture to the envelope and remains downstream of runtime sections `01-08`: [SELENE_BUILD_SECTION_09.md#L19](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md#L19), [SELENE_BUILD_SECTION_09.md#L45](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md#L45)
  - deterministic computation remains cloud-side whenever thresholds or consensus matter: [SELENE_BUILD_SECTION_10.md#L19](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_10.md#L19)
  - PH1.LAW remains the final runtime-law judge, and C4 requires proof/law visibility before protected completion can be considered complete: [SELENE_BUILD_SECTION_11.md#L21](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md#L21), [C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L99](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L99)

Artifact Sync matrix

| artifact/sync surface | current repo truth | F2 design implication | deferred boundary |
| --- | --- | --- | --- |
| `DeviceArtifactPullRequest` | device reports current active versions | F2 may require iPhone setup/entry flows to reference pull eligibility only as cloud/runtime contract | F3 owns pull/apply worker behavior |
| `DeviceArtifactPullResponse` | cloud returns payload refs, hashes, and optional idempotency keys | F2 may preserve this as downstream artifact delivery contract only | F3 owns actual device apply semantics |
| `DeviceArtifactSyncEnvelope` | sync job carries receipt ref, artifact profile id, onboarding session id, user id, device id, and idempotency key | F2 may define how setup-receipt surfaces and app-open surfaces point into sync job creation | F3 owns queue, retry, and ack mechanics |
| `VoiceProfile` / `VoiceArtifactManifest` | voice identity artifacts already have sync kinds | F2 may require ingress/setup to remain compatible with phone-first identity custody | F3 owns on-phone custody and continuity |
| `WakeProfile` / `WakeArtifactManifest` | wake artifact sync kinds exist cloud-side | F2 must not claim iPhone wake parity or widen entry posture from their presence | wake behavior remains out of scope here |

Governance / Law / Proof matrix

| concern or action | source of authority | governance/law/proof participation | iPhone F2 role |
| --- | --- | --- | --- |
| ordinary explicit voice ingress | PH1.L plus ingress/runtime envelope discipline | later governance/law may consume the envelope; ingress itself does not become authority | propose bounded input only |
| app-open activation | PH1.LINK and PH1.ONB | tenant-scope validation, active simulation-chain enforcement, and later protected-completion law remain cloud-side | produce bounded open request only |
| setup receipt acceptance | storage/onboarding authority | proof and law may be required before protected completion | produce bounded receipt evidence only |
| artifact sync job creation from setup outcome | artifact/persistence authority | proof/governance/law may apply if protected completion depends on visibility | no client-local artifact authority |
| platform mismatch, integrity mismatch, or invalid posture | cloud validation and downstream law | governance may escalate; PH1.LAW may block/quarantine/safe-mode | client cannot self-clear |

I) FAILURE / ESCALATION / SAFE-FAIL MODEL
- missing or malformed envelope fields fail closed before execution begins: [SELENE_BUILD_SECTION_03.md#L113](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md#L113)
- actor, device, platform, turn, or session mismatch against the envelope blocks ingress: [app_ingress.rs#L193](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L193)
- live voice entry without negotiated `MICROPHONE` capability blocks: [ph1os.rs#L608](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs#L608)
- tenant mismatch or missing tenant scope on app-open blocks invite activation: [app_ingress.rs#L932](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L932), [app_ingress.rs#L942](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L942)
- non-activated link result blocks onboarding draft start: [app_ingress.rs#L990](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L990)
- missing required iOS setup receipt blocks protected onboarding completion rather than being guessed locally: [ph1f.rs#L11503](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs#L11503), [CORE_ARCHITECTURE.md#L2568](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md#L2568)
- any proof/governance/law visibility failure after authoritative truth exists preserves truth but withholds protected completion: [C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L64](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L64), [C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L99](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L99)
- F2 may not infer wake parity, synthesize missing iPhone capture behavior, or allow local continuation on authority ambiguity

J) IDEMPOTENCY / REPLAY / TENANT / DEVICE BINDING CONSTRAINTS
- idempotency remains mandatory for app-open and downstream persistence surfaces: [app_ingress.rs#L297](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L297), [SELENE_BUILD_SECTION_05.md#L35](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md#L35), [C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L95](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md#L95)
- device fingerprint, app instance id, deep-link nonce, and link-open time remain the canonical binding tuple for app-open and onboarding draft surfaces: [ph1link.rs#L1466](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1link.rs#L1466), [ph1onb.rs#L239](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1onb.rs#L239)
- D1/D3 remain binding on ordering, retries, and replay-safe reuse; F2 may not redefine them into a mobile-only rule family: [D1_CROSS_DEVICE_SESSION_CONSISTENCY_REVIEW.md#L101](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D1_CROSS_DEVICE_SESSION_CONSISTENCY_REVIEW.md#L101), [D3_RUNTIME_AND_PERSISTENCE_WIRING_BUILD_PLAN.md#L64](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D3_RUNTIME_AND_PERSISTENCE_WIRING_BUILD_PLAN.md#L64)
- tenant scope remains explicit and must match activation scope on app-open: [app_ingress.rs#L926](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs#L926)
- setup receipts and sync references remain evidence and visibility inputs only; they must not reverse-authorize onboarding, session, or artifact truth

K) CURRENT CONFLICTS / GAPS
- `P1` no native iPhone client exists in repo truth today: [F1_IPHONE_PARITY_REVIEW.md#L239](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L239)
- `P1` no native iPhone producer exists yet for `AppVoiceIngressRequest`, `AppInviteLinkOpenRequest`, invite activation, onboarding draft start, or iOS setup receipts
- `P1` no iPhone microphone permission, interruption, route-change, or capture-session contract exists in repo truth; only the cloud/runtime acceptance boundary exists: [F1_IPHONE_PARITY_REVIEW.md#L242](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L242)
- `P1` no iPhone artifact pull/apply/outbox client exists, even though cloud-side sync envelopes and kinds exist: [F1_IPHONE_PARITY_REVIEW.md#L241](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L241)
- `P2` platform compatibility, integrity, and attestation fields already exist in runtime context, but no iPhone emitter exists yet to make them live: [runtime_execution.rs#L261](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs#L261)
- `P2` wake artifacts and wake sync kinds exist cloud-side, but F2 must not interpret that as lawful iPhone wake parity while `EXPLICIT_ONLY` remains frozen: [PH1_W.md#L16](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_W.md#L16), [F1_IPHONE_PARITY_REVIEW.md#L245](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md#L245)

L) F2 -> F3 / F4 / F5 FREEZE BOUNDARY
- F2 freezes ingress/runtime contract design only.
- F2 may define entry-shape, field, binding, and fail-closed rules for explicit-entry, app-open, capture posture, and setup receipts.
- F2 may not define continuity workers, device retry/outbox/apply mechanics, protected-action enforcement, or closure proof.

F2 -> F3 / F4 / F5 Boundary matrix

| downstream phase | may define | must not redefine | frozen upstream it must consume | freeze-boundary result |
| --- | --- | --- | --- | --- |
| `F3` | iPhone continuity, attach/resume/recover client behavior, durable retry/outbox, artifact pull/apply, device sync, client-local continuity mechanics | Section `02` session authority, Section `05` idempotent/ack law, F2 ingress field semantics, any wake widening | Sections `02`, `05`, `07`, C4, D1-D4, F1, F2 | F3 owns continuity mechanics only |
| `F4` | iPhone governance/law/proof enforcement, protected completion participation, compatibility/integrity feed, quarantine/safe-fail client wiring | PH1.GOV authority, PH1.LAW final posture, PH1.J proof meaning, PH1.COMP numeric authority, F2 ingress semantics | Sections `04`, `09`, `10`, `11`, C4, D4, E4, F1, F2 | F4 owns protected-participation wiring only |
| `F5` | tests, docs, traceability, evidence pack, final freeze gate for the iPhone slice | any new runtime semantics or weakening of F1/F2/F3/F4 obligations | all prior F phases plus C5, D5, E5 | F5 owns closure proof only |

M) COMPLETION CRITERIA
- F2 is complete only if all of the following are true:
  - iPhone remains first-class platform, non-authority source
  - `EXPLICIT_ONLY` posture remains explicit and unchanged
  - explicit-entry, app-open, capture posture, and setup-receipt surfaces are all defined against current repo truth
  - `CURRENT`, `TARGET`, and `GAP` are explicit
  - all 6 required matrix sections are present
  - no native iPhone client is claimed as present in repo truth
  - no wake parity claim is made
  - F3, F4, and F5 boundaries remain explicit and frozen downstream only
- F2 final truth:
  - design freeze-ready: `YES`
  - native iPhone implementation present in repo today: `NO`
  - remaining work beyond F2: continuity/runtime mechanics, governance/law/proof client wiring, and closure evidence remain deferred to `F3-F5`
