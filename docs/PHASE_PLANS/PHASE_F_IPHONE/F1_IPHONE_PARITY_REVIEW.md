PHASE F1 - IPHONE PARITY REVIEW

A) REPO TRUTH CHECK
- `git status --short`: empty
- current branch: `main`
- repo-truth snapshot HEAD at review start: `1ade48d018d90876f96ce21a193c6a1d7a0b9904`
- baseline command run before authoring:
  - `bash scripts/selene_design_readiness_audit.sh`
  - result: `pass` (with a pre-existing unused-import warning in `crates/selene_os/src/ph1j.rs`; not changed in this phase)
- target file creation proof:
  - `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md`
- exact files reviewed:
  - [CORE_ARCHITECTURE](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/CORE_ARCHITECTURE.md)
  - [SELENE_BUILD_EXECUTION_ORDER](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_BUILD_EXECUTION_ORDER.md)
  - [SELENE_AUTHORITATIVE_ENGINE_INVENTORY](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md)
  - [E1_PERSONALITY_ARCHITECTURE_REVIEW](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_E_PERSONALITY/E1_PERSONALITY_ARCHITECTURE_REVIEW.md)
  - [E2_BOUNDED_ADAPTIVE_BEHAVIOR_MODEL_BUILD_PLAN](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_E_PERSONALITY/E2_BOUNDED_ADAPTIVE_BEHAVIOR_MODEL_BUILD_PLAN.md)
  - [E3_TONE_VS_LONG_TERM_BEHAVIOR_SEPARATION_BUILD_PLAN](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_E_PERSONALITY/E3_TONE_VS_LONG_TERM_BEHAVIOR_SEPARATION_BUILD_PLAN.md)
  - [E4_SAFETY_LAW_MEMORY_CONTROLS_BUILD_PLAN](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_E_PERSONALITY/E4_SAFETY_LAW_MEMORY_CONTROLS_BUILD_PLAN.md)
  - [E5_TESTS_DOCS_VERIFICATION_BUILD_PLAN](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_E_PERSONALITY/E5_TESTS_DOCS_VERIFICATION_BUILD_PLAN.md)
  - [SECTION_03](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md)
  - [SECTION_04](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md)
  - [SECTION_05](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md)
  - [SECTION_06](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_06.md)
  - [SECTION_07](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_07.md)
  - [SECTION_08](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md)
  - [SECTION_09](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md)
  - [SECTION_11](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md)
  - [PH1_W](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_W.md)
  - [PH1_ONB](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_ONB.md)
  - [PH1_LINK](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_LINK.md)
  - [PH1_PERSONA](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_PERSONA.md)
  - [PH1_CONTEXT](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_CONTEXT.md)
  - [PH1_M](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_M.md)
  - [PH1_LAW](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_LAW.md)
  - [runtime_execution.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs)
  - [ph1link.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1link.rs)
  - [ph1onb.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1onb.rs)
  - [ph1persona.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1persona.rs)
  - [ph1context.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1context.rs)
  - [selene_adapter/lib.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_adapter/src/lib.rs)
  - [app_ingress.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs)
  - [ph1os.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/ph1os.rs)
  - [device_artifact_sync.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/device_artifact_sync.rs)
  - [ph1f.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs)
- whether iPhone-specific code/docs were found and where:
  - iPhone-specific platform/runtime surfaces were found in [SECTION_08](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md), [PH1_W](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_W.md), [PH1_ONB](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_ONB.md), [PH1_LINK](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_LINK.md), [selene_adapter/lib.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_adapter/src/lib.rs), [runtime_execution.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs), and [ph1f.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs).
  - No native iPhone app/module code was found. A targeted tracked-file search found no `.swift`, `.m`, `.mm`, `Package.swift`, or `.xcodeproj` paths in current repo truth.

B) PURPOSE
- Freeze the actual iPhone parity baseline without inventing an iPhone client that does not exist.
- Consume frozen Phase E1-E5 as authoritative law for personality, adaptation, tone-vs-long-term behavior, memory controls, and runtime-law posture.
- Define lawful iPhone parity as:
  - same canonical cloud-authoritative ingress, session, personality, memory, governance, and law surfaces
  - platform-specific entry mechanics only where Section 08 explicitly allows them
- Prevent future Phase F work from misreading "parity" as "copy Android wake behavior onto iPhone".

C) CURRENT IPHONE PARITY STATE
- platform identity and trigger policy
  - CURRENT: [SECTION_08](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md) and [selene_adapter/lib.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_adapter/src/lib.rs) treat `IOS` as a first-class platform with `EXPLICIT_ONLY` trigger policy while preserving one universal runtime model.
  - TARGET: iPhone remains explicit-entry by platform law, but once a turn enters Selene it uses the same cloud session and canonical runtime path as every other client.
  - GAP: no native iPhone runtime exists in repo to emit live client version, integrity, capability, and platform posture facts from an actual phone app.
- canonical ingress convergence
  - CURRENT: [SECTION_03](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md), [runtime_execution.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs), and [app_ingress.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/app_ingress.rs) keep iPhone turns inside the same `RuntimeExecutionEnvelope` and session/turn alignment rules as other platforms.
  - TARGET: iPhone requests converge into canonical `/v1/voice/turn` behavior with no mobile-only execution shortcut.
  - GAP: no iPhone app or end-to-end iPhone ingress harness exists in repo to prove the path from real iPhone capture to canonical cloud ingress.
- invite-open / onboarding app-open binding
  - CURRENT: [PH1_LINK](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_LINK.md), [PH1_ONB](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_ONB.md), [ph1link.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1link.rs), [ph1onb.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/ph1onb.rs), and [ph1f.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs) require replay-safe app-open context (`app_platform`, `app_instance_id`, `deep_link_nonce`, `link_opened_at`, device fingerprint binding) for `IOS|ANDROID`.
  - TARGET: iPhone deep-link open/activate and onboarding start remain phone-app-bound, replay-safe, idempotent, and device-bound.
  - GAP: no iPhone app exists to produce these fields or to prove live deep-link/open activation on device.
- iPhone platform setup receipts
  - CURRENT: [ph1f.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs) defines required iPhone setup receipts:
    - `install_launch_handshake`
    - `push_permission_granted`
    - `notification_token_bound`
    - `ios_side_button_configured`
    - signer policy `selene_mobile_app`
  - TARGET: onboarding completion for iPhone depends on these receipts being emitted and bound deterministically by the mobile app.
  - GAP: no iPhone receipt producer, signer implementation, or client proof capture surface exists in repo.
- wake versus explicit-entry posture
  - CURRENT: [SECTION_08](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md), [PH1_W](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_W.md), [selene_adapter/lib.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_adapter/src/lib.rs), and [ph1f.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs) keep iPhone `explicit_trigger_only` by default, reject iPhone wake evaluation in the live adapter, and block iPhone wake enrollment unless an explicit override is admitted.
  - TARGET: iPhone parity is explicit-entry parity first. Any future iPhone wake path must be separately governed and must never be smuggled in as Android-style parity.
  - GAP: there is no iPhone explicit side-button/app-open producer implementation and no governed criteria yet defining when, if ever, an iPhone wake override is lawful.
- microphone readiness and capture session
  - CURRENT: [SECTION_08](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md) and [runtime_execution.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_kernel_contracts/src/runtime_execution.rs) model microphone capability generically, but repo truth does not define an iPhone-specific microphone permission, interruption, route-change, or capture-session contract.
  - TARGET: iPhone explicit-turn capture must produce canonical voice-turn inputs and must not become a local authority path.
  - GAP: no iPhone microphone readiness/capture lifecycle, permission receipt, or interruption model exists in repo truth.
- device artifact sync and phone-first local custody
  - CURRENT: [PH1_W](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_W.md), [SECTION_05](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md), and [device_artifact_sync.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_os/src/device_artifact_sync.rs) define phone-first local artifact custody plus canonical sync, ack, replay, and pull/apply mechanics.
  - TARGET: iPhone must participate in the same device-vault/outbox/artifact-sync discipline as other phone-class clients.
  - GAP: no native iPhone device vault, outbox, artifact sync sender, or artifact pull/apply client exists in repo.
- frozen Phase E personality parity
  - CURRENT: [PH1_PERSONA](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_PERSONA.md), [PH1_CONTEXT](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_CONTEXT.md), [PH1_M](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_M.md), and [PH1_LAW](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_LAW.md) keep personality/context as non-authoritative cloud outputs, memory as cloud-authoritative truth, and runtime law as final cloud posture.
  - TARGET: iPhone must consume those outputs only as a terminal surface and must never locally author persona core, long-term behavior, memory truth, governance posture, or law posture.
  - GAP: no iPhone app implementation exists to prove lawful Phase E consumption or to prove that local drift cannot happen in future client code.
- cross-device continuity and retry/recovery parity
  - CURRENT: [SECTION_05](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md), [PH1_LINK](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_LINK.md), [PH1_ONB](/Users/xiamo/Documents/A-Selene/Selene-OS/docs/DB_WIRING/PH1_ONB.md), and [ph1f.rs](/Users/xiamo/Documents/A-Selene/Selene-OS/crates/selene_storage/src/ph1f.rs) define idempotent app-open binding, durable outbox expectations, sync receipts, and replay-safe onboarding/wake artifact flow.
  - TARGET: iPhone retry, reconnect, and resume behavior must reuse the same session-first idempotent recovery model as all other clients.
  - GAP: no iPhone durable outbox, retry journal, or reconnect implementation exists in repo.
- native iPhone client implementation presence
  - CURRENT: repo truth contains cloud/runtime contracts and adapter/storage handling for `IOS`, but no native iPhone application project.
  - TARGET: real iPhone parity requires a concrete client/runtime implementation.
  - GAP: missing.

D) GAP TAXONOMY
- `NATIVE_IOS_CLIENT_GAP`: no Swift/Objective-C/Xcode iPhone app surface exists in repo truth.
- `EXPLICIT_ENTRY_IMPLEMENTATION_GAP`: iPhone explicit-trigger-only posture is documented and normalized, but no side-button/app-open client implementation exists.
- `IOS_PLATFORM_RECEIPT_GAP`: required iPhone onboarding/setup receipts exist in storage truth only; no client producer exists.
- `IOS_MIC_READINESS_GAP`: no iPhone microphone permission/readiness/capture-session contract exists.
- `IOS_ARTIFACT_SYNC_GAP`: no iPhone vault/outbox/sync/pull/apply implementation exists for phone-first artifact custody.
- `IOS_PERSONALITY_CONSUMPTION_GAP`: frozen Phase E boundaries exist, but there is no iPhone client proving lawful consumption-only behavior.
- `IOS_RECOVERY_PARITY_GAP`: no iPhone reconnect/retry/replay implementation exists for Section 05 parity.
- `IOS_WAKE_OVERRIDE_GOVERNANCE_GAP`: future iPhone wake override posture is not formally governed yet and must not be inferred from Android/Desktop behavior.
- `IOS_CLIENT_COMPATIBILITY_GAP`: platform versioning and compatibility policy exist in adapter normalization, but no native iPhone release/client bundle surface exists.

E) PHASE E NON-REGRESSION CHECK
- E1 persona / tone / emotion / memory / learning separation: current live violation `NO`.
  - Repo truth keeps these boundaries cloud-side and contract-bounded. No iPhone-side writer exists in repo.
- E2 bounded adaptive behavior law: current live violation `NO`.
  - No iPhone client exists that can self-activate bounded adaptation, rewrite persona core, or widen adaptive outputs.
- E3 tone-vs-long-term behavior separation: current live violation `NO`.
  - No iPhone persistence path exists that could currently persist transient tone as stable behavior.
- E4 safety / law / memory controls: current live violation `NO`.
  - Memory suppression, policy, governance, and runtime-law posture remain cloud-authoritative.
- E5 verification closure expectations: current live violation `NO`.
  - The frozen verification law remains intact, but there is no iPhone verification harness yet to prove client-side compliance.
- no device-local persona authority: current live violation `NO`.
- no device-local memory authority: current live violation `NO`.
- no client-side governance or final law authority: current live violation `NO`.

F) F1 FINDINGS
- P0 blockers
  - NONE
- P1 serious gaps
  - No native iPhone app/module exists in the repo, so iPhone parity cannot currently be claimed as live implementation truth.
  - iPhone explicit-entry-only policy exists in contracts and storage/runtime normalization, but there is no client implementation for side-button/app-open entry.
  - iPhone onboarding/setup receipt requirements exist only as cloud-side contract/storage truth; there is no mobile producer.
  - No iPhone microphone readiness or capture-session contract exists, despite generic platform capability handling.
  - No iPhone device vault/outbox/artifact sync implementation exists to satisfy phone-first local custody and replay-safe sync.
  - Frozen Phase E personality parity is protected only indirectly today; there is no iPhone client proving consumption-only behavior.
- P2 normal gaps
  - Future iPhone wake override posture is underdefined and must not be inferred from Android/Desktop wake behavior.
  - Client compatibility/version policy exists in adapter normalization, but there is no native iPhone release/bundle surface to enforce it.
  - The Phase F doc tree did not exist before this review.

G) WHAT F2 MUST DEFINE
- iPhone explicit trigger / app-open ingress contract
  - exact side-button, app-open, and user-triggered entry paths that are lawful for iPhone and how they converge into canonical Selene ingress
- iPhone microphone readiness and capture-session contract
  - permission posture, interruption posture, route-change handling, capture session lifecycle, and canonical voice-turn packaging
- iPhone platform setup receipt contract
  - exact production of `install_launch_handshake`, `push_permission_granted`, `notification_token_bound`, and `ios_side_button_configured`
- iPhone device vault / outbox / artifact sync contract
  - local artifact custody, ack-gated deletion, retry/replay, artifact pull/apply, and rollback posture
- iPhone Phase E consumption contract
  - exact rule that iPhone consumes persona/context/memory/law outputs only and never becomes a personality, memory, governance, or law authority
- iPhone parity / fallback contract
  - explicit-only lawful parity, optional future wake override governance, and fail-closed fallback when required client posture is missing

H) BUILD ORDER RECOMMENDATION
- whether F2 should proceed next: `YES`
- whether any prerequisite repo cleanup is required first: `NO`
- whether any native iPhone implementation should start before F2: `NO`

I) FINAL VERDICT
- PROCEED TO F2
