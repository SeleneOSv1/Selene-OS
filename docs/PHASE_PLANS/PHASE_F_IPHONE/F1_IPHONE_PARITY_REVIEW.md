PHASE F1 — IPHONE PARITY REVIEW

A) REPO TRUTH CHECK
- task-state mode for this pass: `PARTIAL_AUTHORING_RECOVERY`
- reason for mode: the target file already existed, but repo truth showed it did not satisfy the required F1 headings, matrices, and review depth
- repo root: `/Users/xiamo/Documents/A-Selene/Selene-OS`
- branch: `main`
- repo-truth snapshot at review start: `df1a2780fb8ef68102e4a3155a174cd901d10e7a`
- preflight truth:
  - `git status --short`: empty
  - `git branch --show-current`: `main`
  - `git ls-remote origin HEAD`: reachable and equal to local HEAD at review start
- target file truth:
  - tracked file already existed at `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_F_IPHONE/F1_IPHONE_PARITY_REVIEW.md`
- source sets inspected for this review:
  - architectural law: `docs/CORE_ARCHITECTURE.md`, `docs/SELENE_BUILD_EXECUTION_ORDER.md`, `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`
  - frozen upstream review/plan baselines: Phase C `C1/C3/C4/C5`, Phase D `D1/D2/D3/D4/D5`, Phase E `E1/E2/E3/E4/E5`
  - build sections: `01` through `11`
  - DB wiring and runtime surfaces: `PH1.L`, `PH1.LINK`, `PH1.ONB`, `PH1.W`, `PH1.VOICE.ID`, `PH1.M`, `PH1.PERSONA`, `PH1.CONTEXT`, `PH1.OS`, `PH1.GOV`, `PH1.LAW`, `PH1.J`, `PH1.COMP`, `ARTIFACTS_LEDGER_TABLES`, `runtime_execution.rs`, `app_ingress.rs`, `device_artifact_sync.rs`, `runtime_governance.rs`, `runtime_law.rs`
- repo-truth search result that controls this review:
  - cloud/runtime support for `IOS` exists
  - a native iPhone client does not exist in this repo
  - no tracked `.swift`, `.m`, `.mm`, `Package.swift`, or `.xcodeproj` paths were found

B) PURPOSE
- freeze the actual iPhone parity baseline against current repo truth rather than assumed mobile-client intent
- define what lawful iPhone parity means under Sections `01-11`, PH1 runtime boundaries, and frozen Phase C/D/E constraints
- identify every current in-scope surface, every current gap, and the exact downstream boundary for `F2-F5`

C) DEPENDENCY RULE
- F1 consumes frozen authority. It does not reopen or reinterpret it.
- frozen upstream surfaces consumed by this review:
  - Phase C:
    - `C1` authority vs receipt vs projection vs evidence model
    - `C3` cloud-authoritative memory retention and deletion law
    - `C4` storage-before-visibility, proof, and law completion rule
    - `C5` verification and closure model
  - Phase D:
    - `D1` cross-device session consistency law
    - `D2` attach / recover / detach contract law
    - `D3` runtime and persistence wiring law
    - `D4` governance / runtime-law / proof alignment law
    - `D5` verification and closure law
  - Phase E:
    - `E1` personality architecture law
    - `E2` bounded adaptation law
    - `E3` tone vs long-term behavior separation law
    - `E4` safety, law, and memory control law
    - `E5` verification and closure law
- build-section dependency rule:
  - Section `01` gives the cloud-authoritative runtime kernel
  - Section `02` gives PH1.L session authority and cross-device sequencing
  - Section `03` gives canonical ingress and runtime-envelope discipline
  - Section `04` gives verification-before-authority and artifact trust boundaries
  - Section `05` gives persistence, sync, replay, and idempotent recovery law
  - Section `06` gives context and memory participation boundaries
  - Section `07` gives identity and voice binding law
  - Section `08` gives platform-runtime normalization, including iPhone posture
  - Section `09` gives runtime-governance participation
  - Section `10` gives deterministic numeric and consensus computation
  - Section `11` gives final runtime-law posture and fail-closed response classes
- F1 is review-only. It does not start `F2-F5`, implementation, or runtime redesign.

D) ARCHITECTURAL POSITION
- iPhone is a terminal, not an authority source.
- canonical Selene law from Section `01` and `CORE_ARCHITECTURE.md` remains unchanged:
  - cloud runtime owns identity, session lifecycle, memory truth, artifact activation, governance, proof, and law posture
  - clients capture input, render output, and sync with the cloud runtime
  - clients do not author canonical state
- lawful iPhone parity therefore means:
  - same cloud-authoritative session, ingress, identity, artifact, memory, governance, proof, and law model as every other platform
  - platform-specific entry mechanics only where Section `08` allows them
  - no client-local exception path for iPhone
- current repo truth is stricter than generic "mobile parity":
  - iPhone is a first-class platform class
  - iPhone trigger posture is currently `EXPLICIT_ONLY`
  - iPhone parity is therefore explicit-entry parity first, not wake parity

E) CURRENT REPO SURFACES IN SCOPE
- current F1 scope includes the complete canonical surface that an iPhone client must eventually terminate into:
  - runtime kernel and authoritative boundary
  - session creation, resume, attach, recover, replay, stale rejection, and idempotency
  - ingress normalization and runtime execution envelope
  - artifact trust, activation, and proof linkage
  - persistence, sync, outbox, and device artifact continuity
  - identity, voice identity, and platform/runtime posture
  - memory, personality, tone, and bounded adaptation consumption
  - governance, runtime law, proof, audit, and deterministic scoring
- current F1 scope explicitly excludes:
  - implementation of an iPhone client
  - design work for `F2-F5`
  - any reinterpretation of frozen C/D/E law

Current Repo Surface → F1 iPhone Parity Scope Mapping

| current repo surface | canonical role | iPhone parity relevance | current repo truth | F1 freeze note |
| --- | --- | --- | --- | --- |
| Section `01` core runtime | cloud-authoritative runtime kernel | iPhone must terminate into the same kernel as every other client | present | no client-side authority allowed |
| Section `02` / `PH1.L` | canonical session authority and cross-device sequencing | iPhone turns must use the same `session_id`, `turn_id`, single-writer, and per-device sequencing model | present | iPhone does not get a parallel session model |
| Section `03` / runtime ingress | canonical request normalization and envelope discipline | iPhone ingress must produce the same runtime envelope and alignment guarantees | present | no iPhone-only shortcut ingress |
| Section `04` / artifact trust | verification-before-authority, artifact trust, proof linkage | iPhone may consume artifacts; it does not bypass artifact trust or protected completion | present | client posture is upstream only |
| Section `05` / persistence and sync | replay, outbox, reconciliation, idempotency, device artifact sync | iPhone must obey the same retry, stale, dedupe, and sync rules | present cloud-side; no iPhone client | parity not yet live on device |
| Section `06` / memory-context layer | context and memory participation boundaries | iPhone must consume memory/context outputs only | present | no device-local memory authority |
| Section `07` / identity and voice | actor/device binding, voice identity, phone-first artifact custody | iPhone must obey the same actor/device identity law | present cloud-side | no native iPhone producer yet |
| Section `08` / platform runtime | platform normalization and trigger policy | iPhone-specific posture is defined here | present | iPhone is explicit-only today |
| Section `09` / governance | runtime-governance visibility and protected-action classification | iPhone parity must participate in governance decisions without becoming governance authority | present | downstream-only authority |
| Section `10` / `PH1.COMP` | deterministic scoring, thresholds, quotas, consensus | iPhone cannot locally decide numeric thresholds or consensus for protected runtime behavior | present | deterministic math remains cloud-side |
| Section `11` / `PH1.LAW` | final runtime posture and fail-closed responses | iPhone parity must end under the same `ALLOW/BLOCK/QUARANTINE/SAFE_MODE` law posture | present | final law remains cloud-side |

F) CANONICAL IPHONE PARITY MODEL
- F1 freezes the following canonical parity model:
  1. iPhone is a first-class platform, not a first-class authority.
  2. Every lawful iPhone turn enters through the canonical runtime envelope and PH1.L session boundary.
  3. iPhone may vary only in entry mechanics, capability reporting, and device/platform posture.
  4. iPhone must not locally author session truth, memory truth, persona truth, governance truth, proof truth, or runtime-law truth.
  5. iPhone must obey frozen Phase D attach / resume / recover / detach, replay, handoff, stale, and idempotency law without reinterpretation.
  6. iPhone must obey frozen Phase E personality and memory limits without creating local long-term behavioral authority.
  7. iPhone must consume artifact, governance, proof, and law outcomes already decided by the cloud runtime.
  8. iPhone parity is incomplete until a native client exists that emits the required authoritative inputs and receipts into the existing cloud-side model.
- current repo truth makes one additional rule explicit:
  - iPhone parity currently means `EXPLICIT_ONLY` entry unless a future governed change explicitly widens that posture
- explicit review frame for this freeze:
  - `CURRENT`: repo truth already provides cloud-authoritative `IOS` platform classification, canonical runtime-envelope/session law, onboarding/setup receipt law, artifact authority and sync surfaces, and governance/law/proof participation; repo truth does not provide a native iPhone client, iPhone receipt producer, iPhone artifact consumer, or lawful wake path.
  - `TARGET`: F1 freeze certifies the review boundary only: iPhone is a first-class platform surface inside the canonical cloud/runtime model, not an authority source; current parity is cloud/runtime parity only; all lawful gaps are explicit; and downstream client-definition work remains deferred to `F2-F5`.
  - `GAP`: F1 closes review ambiguity by naming the current cloud-side parity surface and freezing the downstream boundary. F1 does not close the still-missing native iPhone client/runtime surfaces, which remain explicitly deferred to `F2-F5` and are not claimed as live parity in this phase.

Current Repo Surface to Canonical F1 Mapping

| current repo surface | canonical F1 rule it supports | current truth | parity status | gap statement |
| --- | --- | --- | --- | --- |
| `runtime_execution.rs` / `app_ingress.rs` | all iPhone turns must enter via canonical runtime envelope discipline | present | partial | no native iPhone request producer |
| `PH1.L` and frozen Phase D | iPhone must use the same session, attach, resume, recover, retry, and stale law as every other client | present | partial | no iPhone client to exercise the law |
| `PH1.LINK` and `PH1.ONB` | iPhone onboarding and app-open flows must remain device-bound, replay-safe, and idempotent | present | partial | no iPhone deep-link/open implementation |
| `PH1.W` plus adapter normalization | iPhone trigger posture is explicit-only today | present | current | no lawful basis to claim iPhone wake parity |
| `PH1.VOICE.ID` and `device_artifact_sync.rs` | iPhone must participate in phone-first identity artifact custody and sync | present cloud-side | partial | no iPhone vault/outbox/sync client |
| `PH1.PERSONA`, `PH1.CONTEXT`, `PH1.M`, Phase E | iPhone consumes cloud personality, context, and memory outputs only | present | current | no client proof yet, but no local authority exists in repo |
| `PH1.GOV`, `PH1.LAW`, `PH1.J` | protected iPhone actions must end under governance, proof, and law posture | present | partial | no iPhone protected-action implementation path |
| `ARTIFACTS_LEDGER_TABLES`, `ph1art.rs`, and `device_artifact_sync.rs` | iPhone artifacts must remain trust-bound, contract-bound, and proof-linked | present cloud-side | partial | no native iPhone artifact consumer/installer |
| `PH1.COMP` | thresholds, scoring, and consensus decisions stay deterministic and cloud-side | present | current | no client-local override path exists |

G) CURRENT RUNTIME / SESSION / IDENTITY / ARTIFACT / PLATFORM SURFACES
- runtime and session truth:
  - `RuntimeExecutionEnvelope` already carries `platform`, `platform_context`, `session_id`, `turn_id`, `device_identity`, `device_turn_sequence`, `governance_state`, `proof_state`, and `law_state`
  - PH1.L and frozen Phase D already define single-writer session authority, cross-device attach/recover law, stale rejection, retry reuse, and ownership-uncertainty fail-closed behavior
- identity truth:
  - Section `07`, `PH1.VOICE.ID`, and current contract/runtime surfaces bind actor identity, device identity, and voice-identity posture inside the same canonical execution model
  - voice-identity artifacts are phone-first in custody but sync back to Selene for continuity; this still does not create device authority
- artifact truth:
  - current artifact custody, append-only ledgers, trust-root registry, and proof linkage are cloud-side and replay-safe
  - device artifact sync already exists as a general mobile/device pattern
- platform truth:
  - Section `08` and adapter normalization treat `IOS` as first-class
  - iPhone trigger policy is currently `EXPLICIT_ONLY`
  - onboarding/setup receipts already define iPhone-specific setup prerequisites
- frozen Phase E participation:
  - iPhone can only consume the outputs of `PH1.PERSONA`, `PH1.CONTEXT`, `PH1.M`, and `PH1.LAW`
  - there is no in-repo iPhone path that can locally rewrite those surfaces

Device / Runtime / Session / Identity / Artifact Matrix

| surface | cloud-authoritative truth today | current iPhone-specific repo truth | parity judgment | explicit gap |
| --- | --- | --- | --- | --- |
| runtime class | cloud runtime kernel from Section `01` | `IOS` recognized as first-class platform in platform context | aligned | no native iPhone runtime client |
| entry trigger | Section `08` platform trigger policy | `EXPLICIT_ONLY` for iPhone | aligned | no live side-button/app-open producer |
| session container | PH1.L cloud-authoritative session container | iPhone must use the same `session_id` / `turn_id` model | aligned in design | no native iPhone attach/resume client |
| cross-device session ordering | PH1.L plus Phase D per-device sequence law | iPhone has no special exemption | aligned in design | no iPhone replay/handoff implementation |
| actor identity binding | Section `07` and runtime envelope actor/device binding | iPhone must bind into same actor/device scope | aligned in design | no native client attestation/binding path |
| voice identity artifacts | `PH1.VOICE.ID` phone-first custody plus sync | iPhone eligible as phone-class device in architecture | partial | no iPhone vault/profile/sync client |
| wake artifacts | `PH1.W` plus cloud sync surfaces | iPhone wake path is blocked by default today | not target scope for parity | no governed iPhone wake contract yet |
| onboarding app-open and receipts | `PH1.LINK`, `PH1.ONB`, storage receipt kinds | iPhone requires `install_launch_handshake`, `push_permission_granted`, `notification_token_bound`, `ios_side_button_configured` | aligned in design | no iPhone receipt producer |
| device artifact sync | Section `05` plus `device_artifact_sync.rs` | generic mobile/device sync exists | partial | no iPhone queue/outbox/pull/apply client |
| client compatibility and integrity | platform context supports compatibility/integrity posture | no real iPhone client version/integrity feed exists | not yet live | missing native app |
| memory/persona/law consumption | frozen Phase E cloud outputs | no in-repo iPhone local authority path exists | aligned | no client proof harness yet |

H) GOVERNANCE / LAW / PROOF PARTICIPATION
- iPhone parity must participate in the same protected-action closure model as every other client-facing surface.
- authoritative split remains frozen:
  - PH1.GOV classifies and constrains protected actions
  - PH1.LAW emits final runtime posture
  - PH1.J emits audit and proof linkage
  - PH1.COMP computes deterministic threshold, budget, or consensus math when a protected path needs numeric authority
  - none of the above makes the iPhone client authoritative
- F1 freezes one core rule from C4 and D4:
  - authoritative session or artifact truth may exist before downstream visibility succeeds, but protected completion is not considered complete until required proof/governance/law visibility is present

Authority / Governance / Law / Proof Participation Matrix

| concern or action | source of authority | governance participation | runtime-law participation | proof / audit participation | iPhone client role |
| --- | --- | --- | --- | --- | --- |
| ordinary iPhone turn ingress | PH1.L plus runtime envelope discipline | none unless protected posture applies | final law posture still applies | PH1.J audit lineage may apply | propose input only |
| attach / resume / recover under cross-device posture | PH1.L plus frozen Phase D | governed when protected, uncertain, degraded, or role-sensitive | always final posture source | audit required; proof required when policy marks action protected | observe result only |
| onboarding completion | PH1.ONB, receipt validation, artifact sync receipts | may classify protected completion | final completion posture | proof/audit required for protected completion chain | produce bounded receipts only |
| artifact activation / sync acceptance | artifact trust and activation surfaces | governed when activation or rollback is protected | final activation posture | proof linkage mandatory on protected actions | receive/apply only |
| identity-sensitive recovery or role elevation | Section `07`, PH1.L, Phase D | governed | law-sensitive and often protected | audit and possible proof | provide bounded identity evidence only |
| platform mismatch or invalid iPhone posture | cloud validation path | may escalate or block | final `BLOCK/QUARANTINE/SAFE_MODE` posture | audit required | cannot self-clear |
| deterministic threshold or budget decision | `PH1.COMP` | may consume result | may rely on result for final posture | auditable via envelope and downstream audit | no client-local numeric authority |

I) FAILURE / ESCALATION / QUARANTINE / SAFE-FAIL MODEL
- iPhone parity inherits the frozen fail-closed model from C4, D4, and Section `11`.
- required failure behavior:
  - missing required iPhone setup receipt: block protected completion
  - actor/device/platform mismatch: block
  - stale `device_turn_sequence`: reject stale request
  - ownership uncertainty, transfer pending, or degraded recovery: degrade, block, quarantine, or safe-mode according to final law posture
  - proof/governance/law visibility failure after authoritative truth exists: preserve truth, but withhold protected completion
  - missing native-client capability proof: do not invent parity; record the gap
- what iPhone parity may not do:
  - continue locally on ambiguous authority
  - replay local state as authoritative truth
  - self-certify compatibility, integrity, governance, or law posture
  - infer wake parity from Android or desktop behavior

J) CROSS-DEVICE / HANDOFF / REPLAY / IDEMPOTENCY CONSTRAINTS
- frozen Phase D law applies to iPhone unchanged:
  - `attach`, `resume`, `recover`, and `detach` remain distinct
  - same-sequence retry reuses the authoritative result
  - lower sequence is stale and is rejected
  - one active mutation owner exists at a time
  - persistence and sync repair visibility; they do not become alternate session authority
- this matters more for iPhone because the repo currently has only cloud-side parity surfaces:
  - the lawful handoff/replay model already exists
  - the iPhone client that must obey it does not yet exist

Replay / Handoff / Continuity / Idempotency Matrix

| case | authoritative anchor | lawful outcome | fail-closed outcome | iPhone-specific note |
| --- | --- | --- | --- | --- |
| same-device retry | PH1.L session truth plus `device_turn_sequence` | reuse prior authoritative result | duplicate mutation refused | client must be durable enough to resend without re-authoring |
| stale device turn | PH1.L per-device sequence map | reject stale request | stale replay stays blocked | no local recovery shortcut allowed |
| cross-device attach | Phase D attach law plus PH1.L | attach to same canonical session when lawful | block on identity/platform/ownership mismatch | iPhone must use the same attach law as every other device |
| reconnect after partial success | C4 plus Section `05` | reread authoritative truth, then repair visibility only | do not rerun authoritative mutation | requires device-side journal/outbox not yet implemented on iPhone |
| degraded recovery | Phase D recover law plus persistence posture | explicit degraded recovery under governance/law posture | block/quarantine/safe-mode when unsafe | no iPhone client path exists yet |
| transfer pending | Phase D coordination state | later lawful resume/attach after transfer resolves | block conflicting admission | iPhone has no exemption |
| ownership uncertainty | PH1.L plus PH1.LEASE / persistence posture | explicit governed recovery only | quarantine or safe-mode | iPhone may not self-elect ownership |
| artifact sync replay | Section `05` plus device artifact sync worker model | replay until acked, then apply/activate or rollback deterministically | dead-letter / retry / block on invalid payload or hash | no iPhone client queue yet |
| continuity across devices | PH1.L session continuity plus cloud memory continuity | same cloud session or lawful new session with continuity context | no client-local reconstruction of truth | iPhone continuity is architecture-ready, not implementation-ready |

K) CURRENT CONFLICTS / GAPS
- `P1` no native iPhone client exists in this repo.
  - consequence: iPhone parity cannot honestly be claimed as live implementation truth.
- `P1` no iPhone app currently emits the canonical ingress payloads, app-open fields, or setup receipts already required by `PH1.LINK`, `PH1.ONB`, and storage truth.
- `P1` no iPhone device-vault, outbox, artifact pull/apply, or sync-ack client exists even though cloud/device sync law exists.
- `P1` no native iPhone microphone permission, interruption, route-change, or capture-session contract is present in repo truth.
- `P1` no native iPhone compatibility/integrity feed exists for the platform context that runtime law could rely on.
- `P2` iPhone wake parity is not in scope for current lawful parity because current repo truth explicitly keeps iPhone `EXPLICIT_ONLY`.
- `P2` protected-action participation for iPhone is defined cloud-side, but there is no iPhone implementation path yet to prove end-to-end governance/proof/law closure.
- `P2` frozen Phase E consumption boundaries are safe by absence of client authority today, not by an implemented iPhone client proof harness.
- `P2` current repo truth already contains Phase D session/handoff law, but F1 must not pretend that cloud-side law alone equals complete iPhone parity.

L) F1 → F2 / F3 / F4 / F5 FREEZE BOUNDARY
- F1 freezes review truth only.
- downstream phases may define or implement only inside the boundary below.
- none of `F2-F5` may reopen frozen C/D/E authority, session, personality, proof, governance, or law semantics.

F1 → F2 / F3 / F4 / F5 Boundary Matrix

| downstream phase | may define | must not redefine | frozen upstream it must consume | freeze-boundary result |
| --- | --- | --- | --- | --- |
| `F2` | iPhone explicit-entry ingress contract, app-open/deep-link production, microphone/capture/session-start client contract, platform-context emission, setup receipt production | Section `01-04` authority, Section `08` explicit-only posture, any wake widening without explicit approval | Sections `01-04`, `08`, C1/C4, D1-D3 | F2 owns client ingress and setup contract only |
| `F3` | iPhone session continuity, attach/resume/recover client behavior, durable retry/outbox, device artifact sync, identity/artifact continuity on phone | Phase D attach/recover semantics, Phase E authority split, any client-local session or memory authority | Sections `02`, `05`, `07`, D1-D4, C4, E1-E4 | F3 owns continuity and device-runtime mechanics only |
| `F4` | iPhone governance/law/proof enforcement surfaces, protected completion, quarantine and safe-fail client participation, compatibility/integrity enforcement feed | PH1.GOV authority, PH1.LAW response classes, PH1.J proof path, PH1.COMP numeric authority | Sections `04`, `09`, `10`, `11`, D4, C4, E4 | F4 owns protected-participation wiring only |
| `F5` | tests, docs, traceability, evidence pack, residual risks, freeze gate for the iPhone slice | any new runtime semantics, any weakening of prior phase proof obligations | all prior F phases plus C5, D5, E5 | F5 owns closure proof only |

M) COMPLETION CRITERIA
- F1 is complete only if all of the following are true:
  - current repo truth is captured without inventing a native iPhone client
  - Sections `01-11` are mapped into iPhone parity scope
  - frozen Phase C/D/E constraints are explicitly consumed and preserved
  - runtime, session, identity, artifact, platform, governance, law, proof, and deterministic-scoring surfaces are all covered
  - all required matrices are present
  - all current gaps are explicit
  - downstream `F2-F5` boundaries are frozen without starting them
- F1 final truth:
  - review freeze-ready: `YES`
  - live iPhone implementation parity present in repo today: `NO`
  - blocker to claiming live parity: native iPhone client and its bounded client-runtime surfaces do not yet exist in repo truth
