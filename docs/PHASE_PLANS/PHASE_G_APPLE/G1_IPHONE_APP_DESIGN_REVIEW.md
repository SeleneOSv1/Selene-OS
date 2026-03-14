PHASE G1 — IPHONE APP DESIGN REVIEW

A) REVIEW SCOPE
- G1 designs the iPhone app only as the lawful Selene session interface, onboarding-entry interface, explicit-interaction interface, and governed rendering surface.
- G1 does not design Android, Mac, Windows, or a standalone consumer shell.
- G1 does not implement code, create tickets, reopen frozen Phase F semantics, or claim that a native iPhone client already exists in-tree.

B) FROZEN UPSTREAM DESIGN POSITION
- The iPhone app is a first-class platform surface, but it is not an authority source.
- The app remains `EXPLICIT_ONLY`, cloud-authoritative, session-bound, and non-authoritative for identity, session truth, memory truth, artifact truth, proof, governance, and runtime law.
- No lawful G1 reading may widen iPhone into wake parity. If repo truth does not establish a hardware-trigger behavior as implemented fact, G1 may describe it only as a capability-gated explicit-entry surface.
- Frozen upstream law consumed here is Phase `F1-F5` plus frozen Phase `A-E` dependency law and Build Sections `01-11`.

C) CANONICAL APP POSTURE
- The iPhone app is not a free-floating home shell. It is the iPhone session interface for the same Selene cloud runtime described in `CORE_ARCHITECTURE.md` and frozen across Phase `F`.
- G1 freezes one dominant responsive `session surface` as the default runtime posture. The main conversation thread is the primary app surface whenever cloud session or onboarding law does not require a bounded adjunct surface.
- Bounded lawful adjunct surfaces are limited to:
  - a `side drawer` for explicit history access
  - an onboarding takeover shell while required onboarding gates remain active
  - strong inline restriction cards/banners while the main session surface may remain visible
  - full-takeover states when normal interaction is not lawful
  - bounded artifact/report/detail surfaces and a separate operational `System Activity` surface tied to current cloud posture
- The app follows session and onboarding law; session and onboarding law do not follow the app.
- The visual treatment may be almost identical to ChatGPT on iPhone for readability: a conversational thread, ChatGPT-like reading posture for long outputs, `side drawer` history access, live text for Selene output, and live text for user speech or typed input.
- Primary navigation model:
  - the default surface is the current session surface / thread
  - the `side drawer` holds recent history recall and archived history recall only
  - `System Activity` and `Needs Attention` are separate operational surfaces, not part of transcript history and not mixed into the normal scrollback
  - governed artifacts, charts, long written outputs, and reports open as bounded detail surfaces from the current session surface rather than as independent app homes
- Side-button-first is the intended iPhone primary explicit-entry posture, but repo truth only freezes canonical app-open / invite-open ingress and setup evidence such as `ios_side_button_configured`. G1 therefore treats side-button as intended explicit-entry posture, not as a proven live hardware producer, and preserves app-open / invite-open as canonical ingress.
- That visual resemblance does not change runtime law. The app may render, request, synchronize, and acknowledge; it may not become local simulation, local authority, local proof, local governance, or local law.
- The screen is a governed rendering surface, not the primary legality surface. G1 therefore keeps a screenless-first posture: most interactions must remain lawful even when the screen is not the primary interaction medium, and the app mainly renders current session state, governed outputs, alerts, windowed history, `resume context`, artifacts, and recovery posture.
- Device `unlock` remains user/OS-auth controlled. G1 does not claim autonomous unlock or OS-auth bypass.
- Side-button or other hardware-entry affordances may only be described as explicit-entry surfaces where repo truth already freezes them as platform receipt or trigger posture. G1 does not claim any unsupported hardware binding as live implementation fact.

D) DEPENDENCY LAW
- Build Section `01`: cloud runtime is the only authority; the app is a terminal and synchronized visibility surface only.
- Build Section `02`: the app is session-first; all lawful app states are derived from cloud-owned session truth rather than local invention.
- Build Section `03`: the app may enter only through canonical ingress routes and canonical runtime-envelope creation.
- Build Section `04`: onboarding progression, authority, artifact activation, and protected execution remain cloud-authoritative.
- Build Section `05`: retry, replay, outbox, reconciliation, and sync posture must remain envelope-driven, idempotent, and auditable.
- Build Section `06`: history and memory views are cached windows over cloud truth, never local memory authority.
- Build Section `07`: identity, voice, and biometric decisions remain cloud-owned; the app may display status only.
- Build Section `08`: iPhone remains explicit-trigger-only and platform-specific differences affect session entry only, never runtime execution law.
- Build Section `09`: governance posture is visible in the envelope and may affect UI posture, quarantine posture, and operator-visible severity.
- Build Section `10`: any quantitative thresholds rendered by the UI are derived read-only outputs, never free-form local judgment.
- Build Section `11`: final `BLOCK`, `DEGRADE`, `QUARANTINE`, or `SAFE_MODE` runtime posture remains cloud-authored.
- Frozen Phase `A1-A5` consumed: the app may surface artifact refs, setup receipts, trust visibility, proof status, and protected failure posture only through canonical cloud-authoritative trust, proof, governance, and law transport.
- Frozen Phase `B1-B3` consumed: iPhone may not define weaker session, continuity, capture, or enforcement behavior than the frozen mobile parity baseline, and it may not create a second trust/proof/enforcement path.
- Frozen Phase `C1/C3/C4` consumed: receipts are evidence only; lifecycle, memory, restore, archive, purge, and protected-complete posture remain cloud-owned and may not be authored by the app.
- Frozen Phase `D1-D4` consumed: attach, resume, recover, detach, stale, retry, lease, ordering, reconciliation, and protected session posture remain cloud-decided.
- Frozen Phase `E1/E3/E4` consumed: presentation and personalization may shape delivery only; identity, safety, governance, and runtime law remain constraint layers rather than UI-authored behavior.
- Frozen Phase `F1-F5` consumed: G1 may surface ingress, continuity, governance/law/proof, and rollout truth, but it may not redefine any frozen `F` boundary.

E) CURRENT / TARGET / GAP
- `CURRENT`: repo truth already provides canonical iPhone explicit-entry ingress (`AppVoiceIngressRequest`, invite-link/app-open, onboarding continue, and setup receipts such as `ios_side_button_configured`), canonical session states (`Closed`, `Open`, `Active`, `SoftClosed`, `Suspended`), canonical session attach/recovery and persistence posture in the `RuntimeExecutionEnvelope`, append-only conversation storage distinct from PH1.M memory, bounded PH1.M `resume context` output, mobile artifact sync queue/pull/ack/retry/dead-letter mechanics, `PH1.BCAST` urgency/waiting/followup/fallback mechanics, and governance/law/proof visibility in the envelope. Repo truth does not provide a native iPhone app implementation, a proven live side-button producer, an autonomous unlock path, or a fully frozen iPhone lock-screen redaction / sound taxonomy today.
- `TARGET`: G1 freezes one lawful iPhone app design with one dominant responsive session surface, bounded lawful adjunct surfaces, dual live transcript, `side drawer` history access, separate `System Activity` / `Needs Attention`, bounded memory-derived `resume context`, lazy governed detail surfaces, and no widening of authority beyond cloud-owned Selene runtime truth.
- `GAP`: G1 does not claim a native client, a local authority cache, local proof writer, local governance engine, local memory truth, unlawful wake parity, unsupported hardware-trigger implementation, or autonomous unlock. If a capability is not established by repo truth, G1 leaves it capability-gated, explicitly product-decided later, or outside current implementation truth rather than inventing it.

F) LAWFUL APP-STATE MODEL
- The app-state model is derived directly from `SessionState`, `SessionAttachOutcome`, `PersistenceRecoveryMode`, `ReconciliationDecision`, `OnboardingNextStep`, and `OnboardingStatus`.
- The app must always open into one lawful state derived from canonical cloud truth; it may not open into an unbound generic home shell.
- Rendering carriers are fixed:
  - dominant `session surface` for normal session-open, active, soft-closed, and bounded resume posture
  - `side drawer` for explicit history recall only
  - inline restriction card/banner for soft restrictions while the main session surface remains lawful
  - full takeover for hard restrictions or gated onboarding states when normal session interaction is not lawful
  - separate operational surface for `System Activity` and `Needs Attention`
- `SessionAttachOutcome::NewSessionCreated|ExistingSessionReused|ExistingSessionAttached|RetryReusedResult` changes the inline session banner, continuity label, and recovery explanation inside the main session surface; it does not create separate app navigation.
- `PersistenceRecoveryMode::Recovering|DegradedRecovery|QuarantinedLocalState` and `ReconciliationDecision::RetrySameOperation|ReusePriorAuthoritativeOutcome|RejectStaleOperation|RequestFreshSessionState|QuarantineLocalState` determine whether the user sees a soft inline restriction, an operational `System Activity` item, or a hard full-takeover state.
- Canonical base states:
  - `EXPLICIT_ENTRY_READY`: `SessionState::Closed` with no active onboarding draft; the app is ready for explicit voice/text entry, invite open, or explicit history recall only.
  - `ONBOARDING_ENTRY_ACTIVE`: app-open or deep-link flow created an onboarding session and the next step is `Install`, `Terms`, `LoadPrefilled`, or `AskMissing`; onboarding renders as a bounded takeover shell rather than a separate app home.
  - `SESSION_OPEN_VISIBLE`: cloud session exists and is open but not currently in active spoken turn posture.
  - `SESSION_ACTIVE_VISIBLE`: cloud session exists and is active; thread view is live and dual transcript is visible.
  - `SESSION_SOFT_CLOSED_VISIBLE`: the recoverable session remains cloud-owned, the screen may be visually reset, and any resume affordance or `resume context` card must read from the authoritative session container rather than local draft truth.
  - `SESSION_SUSPENDED_VISIBLE`: the session is suspended cloud-side; the app may render status and lawful next action only, and this posture may require a hard full-takeover state.
- Canonical overlay states:
  - `RECOVERING`: derived from `PersistenceRecoveryMode::Recovering`; the app may show continuity in progress, but it may not trust local state as final.
  - `DEGRADED_RECOVERY`: derived from `PersistenceRecoveryMode::DegradedRecovery`; the app must favor explicit reread and guarded UI posture.
  - `QUARANTINED_LOCAL_STATE`: derived from `PersistenceRecoveryMode::QuarantinedLocalState` or `ReconciliationDecision::QuarantineLocalState`; the app may show the problem, but it may not self-heal authority.
  - `PENDING_CONFIRMATION`: derived from `AuthorityPolicyDecision::PendingConfirmation` or `PendingState::Confirm`; the app routes the user to confirmation, not silent execution.
  - `STEP_UP_REQUIRED`: derived from `AuthorityPolicyDecision::StepUpRequired` or `PendingState::StepUp`; the app displays the challenge path without owning the security decision.
  - `NEEDS_ATTENTION`: aggregate display posture for blocked onboarding, dead-letter sync, stale/recovery issues, proof/governance/law failure posture, or unresolved protected prompts.
- Turn-level governed adjunct postures:
  - `RESUME_CONTEXT_VISIBLE`: derived from PH1.M bounded resume output and rendered as a short session-surface catch-up card with thread title, pending work, and 1-3 summary bullets only.
  - `INTERRUPT_VISIBLE`: derived from PH1.K / PH1.X interruption truth when accepted `interrupt` cancels TTS immediately and continuity requires a clarify, continue, or resume-later posture inside the current session surface.
- Onboarding-specific visible sub-states must follow canonical onboarding truth:
  - verification pending / confirmed / rejected
  - primary device confirm pending
  - voice enroll pending
  - wake enroll pending or deferred where lawful
  - access provision pending
  - complete / ready
  - blocked
- Soft vs hard restriction rendering law:
  - soft restriction = strong inline restriction card/banner while the main session surface remains visible
  - hard restriction = full takeover when suspended posture, blocked onboarding, quarantine, or other protected runtime posture makes normal interaction unlawful

G) app entry + session-state matrix
| app-visible state | canonical cloud source | default screen posture | lawful user controls | forbidden local inference | notes |
| --- | --- | --- | --- | --- | --- |
| `EXPLICIT_ENTRY_READY` | `SessionState::Closed` with no active onboarding draft | dominant session surface in explicit-entry posture with recent thread window, typed input affordance, and lawful explicit voice entry affordance | start typed turn, explicit voice turn, open invite link, recall recent history window | no local session resurrection, no synthetic active session | the app is session-bound even when closed; there is no generic standalone dashboard |
| `ONBOARDING_ENTRY_ACTIVE` | invite/app-open outcome plus onboarding session record and `OnboardingNextStep` / `OnboardingStatus` | bounded onboarding takeover shell with current required field, required verification gates, and current receipt/task status | continue onboarding, submit required field, provide setup evidence, confirm device, continue verification | no skipping gates, no local completion, no fake readiness | app-open is a lawful entry path, not a separate app mode |
| `SESSION_OPEN_VISIBLE` | `SessionState::Open` plus `SessionAttachOutcome` | dominant session surface visible, ready for next explicit turn, with current session banner | type, explicit voice entry, view recent history window, inspect current artifacts | no local promotion to `Active`, no hidden new session | attach outcome changes inline continuity labeling only |
| `SESSION_ACTIVE_VISIBLE` | `SessionState::Active` plus current `turn_id` / envelope | live conversational thread with both directions visible in real time | speak explicitly, type, interrupt lawfully, inspect current governed outputs | no local turn authority, no local decision shortcuts | live transcript remains text-visible even when spoken |
| `SESSION_SOFT_CLOSED_VISIBLE` | `SessionState::SoftClosed` plus optional presence nudge and bounded PH1.M output | visually quiet session surface with explicit resume affordance, archived recent slice, and optional bounded `resume context` card | resume explicitly, inspect recent archived slice, dismiss screen | no auto-reopen from local cache alone | visual reset may clear the screen, but archive truth remains durable |
| `SESSION_SUSPENDED_VISIBLE` | `SessionState::Suspended` | hard full takeover of the main session surface with explanation and allowed next step only | read status, perform only lawful recovery or exit actions | no local unsuspend, no silent continuation | suspended posture is cloud-authored |
| `RECOVERING` / `DEGRADED_RECOVERY` / `QUARANTINED_LOCAL_STATE` | `persistence_state.recovery_mode` and `reconciliation_decision` | inline restriction card when the session remains visible; escalate to hard takeover when quarantine or reread posture removes lawful normal interaction | reread authoritative state, retry only through canonical entry path, inspect failure details | no local override, no trust in stale cache, no hidden replay | overlays change posture, not ownership |

H) onboarding + app-open flow matrix
| flow step | canonical source | app posture | evidence or gate surfaced | forbidden behavior | notes |
| --- | --- | --- | --- | --- | --- |
| invite / app-open entry | `AppInviteLinkOpenRequest` plus `AppInviteLinkOpenOutcome` | open directly into onboarding session context, not a generic shell | token, tenant hint, device fingerprint, app instance, deep-link nonce | no local invite activation without canonical request | app-open is part of Selene ingress law |
| draft creation | `OnbSessionStartDraftRequest` and `OnbSessionStartResult` | show current onboarding step: `Install`, `Terms`, `LoadPrefilled`, or `AskMissing` | required verification gates and missing fields | no local draft fabrication | onboarding session id is cloud-authored |
| platform setup | `AppOnboardingContinueAction::PlatformSetupReceipt` plus platform receipt commit outcome | show the exact remaining iOS setup steps and current receipt completion state | `install_launch_handshake`, `push_permission_granted`, `notification_token_bound`, `ios_side_button_configured` | no fuzzy equivalent receipt family, no local readiness shortcut | side-button posture is evidence-backed setup state, not a free-form hardware claim |
| terms / verification | onboarding continue plus sender verification outcomes | show terms, verification pending, verification confirmed, verification rejected, or blocked | current verification status and required next action | no local override of verification result | rejected verification must surface blocked posture |
| primary device / voice / wake | `PrimaryDeviceConfirm`, `VoiceEnrollLock`, wake enroll actions, latest locked receipt refs | show current gated enrollment surface only when lawful | current device id, proof status, voice artifact sync receipt ref, wake defer/complete posture | no auto-advance around required evidence | wake remains explicit-entry constrained on iPhone |
| access provision / complete | access provision and complete commit outcomes | show readiness, completion, or remaining blockers | access engine instance id and final onboarding status | no client-side completion claim before canonical completion | ready means cloud-side onboarding truth, not local optimism |

I) conversation + history + system-activity matrix
- `transcript history` is append-only conversation recall for both directions of the session and appears in the main session surface plus explicit history recall.
- `resume context` is PH1.M-derived bounded catch-up output only: selected thread, pending work, resume tier, and 1-3 summary bullets.
- `System Activity` is the operational surface for sync/replay/reconciliation/delivery status that Selene manages cloud-side.
- `Needs Attention` is the human-actionable subset of `System Activity` or protected runtime posture; only real human action belongs there.
| surface | canonical backing truth | default load posture | explicit user actions | forbidden behavior | notes |
| --- | --- | --- | --- | --- | --- |
| live dual transcript | typed input enters the same pipeline as voice; `conversation_ledger` stores user and Selene turns append-only | current turn plus recent slice is visible by default | type, explicit voice entry, inspect per-turn details | no one-way transcript, no local-only text fork, no hidden spoken-only output | both Selene output and user speech / typed input appear as text |
| `resume context` | PH1.M `selected_thread_id`, `selected_thread_title`, `pending_work_order_id`, `resume_tier`, and `resume_summary_bullets` | bounded inline catch-up card in the main session surface; 1-3 bullets only | resume the selected thread, start fresh, open the named thread explicitly | no raw memory dump, no 72h/30d transcript preload, no full archive dump | `72h` / `30d` govern resume surfacing tiers, not transcript loading |
| recent conversation history | append-only `conversation_ledger` and archived session recall | recent visible window in the main session surface; older history loads only through explicit `side drawer` recall and incremental expansion | `Load older messages`, `Show more history`, recall archived session explicitly from the `side drawer` | no full-history eager load, no silent mutation, no cross-session blending, no raw memory ledger dump | history is reconstructible, but the UI loads it incrementally and keeps history distinct from PH1.M memory |
| history after session close | archived conversation truth with visual reset on close | screen may clear while preserving archived recall | reopen recent slice explicitly, recall a prior session explicitly | no automatic post-close resurfacing | close is visual reset, not deletion |
| `System Activity` | persistence acknowledgement state, reconciliation decision, broadcast waiting/followup/reminder state, sync queue/dead-letter posture, and recovery posture | separate operational list/sheet from history and transcript | inspect status, reread authoritative state, continue canonical flow, open linked operational detail | no manual resend console, no local transport repair authority, no hidden auto-heal claim | user sees state, but Selene manages sync/retry/dedupe/reconcile |
| `Needs Attention` | unresolved protected prompts, blocked onboarding, stale/recovery warnings, dead-letter sync, law/governance failure posture, or broadcast state that now requires human action | separate actionable list from normal thread and from plain history | open item, acknowledge, retry through canonical path, inspect reason, complete the required human action | no mixing unresolved operations into normal scrollback, no non-actionable clutter | `Needs Attention` is operational, not conversational, and only real human action belongs here |
| `Pending` | `PendingState`, pending confirmation, step-up, tool wait, onboarding verification pending, broadcast waiting/followup | separate operational queue from history | resolve confirmation, finish step-up, continue onboarding, wait for followup window | no local completion of pending work | `Pending` is explicit and user-visible |
| `Failed` | dead-letter sync, rejected verification, denied authority, failed protected posture, failed delivery visibility | separate operational queue from history | inspect failure, retry lawfully, dismiss once resolved | no silent disappearance of failed work | `Failed` stays visible until the cloud-visible issue is resolved or explicitly cleared |
| heavy governed content | artifacts, charts, long written outputs, reports, and previews tied to current thread or operational items | collapsed summary card first; heavy content lazy-loads only on explicit open | `Show more history`, open report, open artifact, open chart, load next page | no eager full-content hydration, no local truth fork | heavy content opens with a ChatGPT-like reading posture but remains cloud-authoritative and lazy-loaded |

J) PH1.BCAST alerts + haptics + screenless-display matrix
- `PH1.BCAST` defines phone-first Selene App delivery, urgency mapping, `WAITING -> FOLLOWUP -> REMINDER` logic, bounded attempts, and fallback only when the Selene App is unavailable.
- Exact iPhone lock-screen redaction and the full sound taxonomy are not fully frozen in repo truth. G1 therefore treats them as capability-gated / product-decision surfaces rather than current implementation fact.
- Device `unlock` remains user/OS-auth controlled. Urgent surfacing may strongly alert the phone when lawful, but G1 does not claim autonomous unlock.
| case | governing truth | default display posture | haptic / vibration posture | capability gate | notes |
| --- | --- | --- | --- | --- | --- |
| urgent governed alert | `PH1.BCAST` classification and runtime law posture | visible urgent banner/card with immediate attention posture in-app, with stronger delivery salience when lawful | policy-gated and platform-capability-gated urgent haptic only when lawful | no unsupported hardware assumption and no autonomous unlock | urgent display follows cloud classification, not app guesswork |
| non-urgent delivery or followup | `PH1.BCAST` waiting window and non-urgent followup policy | quieter banner/list item with deferred emphasis | lighter or no haptic by default; still policy-gated | capability-gated | non-urgent followup may wait before escalation |
| waiting / reminder handoff | `PH1.BCAST` `WAITING`, `FOLLOWUP`, `REMINDER_SET`, and `REMINDER_FIRED` lifecycle | operational state visible in `System Activity` until it becomes user-facing again | no extra haptic unless policy/classification allows | capability-gated | waiting/followup timing comes from cloud lifecycle, not local timers |
| `screenless` continuity reminder | cloud session and broadcast posture, not local timer invention | render a catch-up card when the app becomes foreground-visible again | optional minimal haptic only if policy and capability allow | capability-gated | screenless-first means the app renders governed outputs when needed, not that the screen owns the interaction |
| proof / governance / law block | envelope-carried governance / proof / law posture | soft inline restriction when the session remains lawful, or hard takeover when normal interaction is blocked | haptic only if the governing policy allows it | capability-gated | the app may explain posture; it may not downgrade it |
| phone-app delivery request | broadcast-to-SeleneApp path | render incoming governed output in the app UI, not SMS | haptic posture follows delivery priority and policy | capability-gated | “display it on my phone” is Selene app delivery, not SMS |
| side-button / hardware explicit entry | platform receipt and explicit-trigger posture only | show as explicit-entry affordance only where setup evidence and platform policy allow | no haptic claim beyond platform capability | capability-gated | any unsupported or unproven hardware behavior remains capability-gated rather than asserted as fact |

K) authority / governance / law / proof boundary matrix
- Personality / emotional posture is a governed design surface only.
- Onboarding-driven or config-driven demeanor/profile selection may influence tone refs or delivery-policy refs where repo truth supports them, but the exact onboarding profile UX is not frozen as current implementation fact.
- App-visible effect is limited to tone, pacing, warmth/directness, and explanation style. No emotional meter, mood panel, or theatrical persona UI is lawful here.
| concern | app may display | app may request | authoritative owner | app may not do | notes |
| --- | --- | --- | --- | --- | --- |
| session truth | current session state, attach outcome, recovery posture | explicit entry, resume request, reread request | cloud Session Engine / Section `02` | invent, mutate, or reconcile final session truth locally | session-facing UI is read-only over cloud truth |
| onboarding progression | current onboarding status, next step, missing fields, remaining receipts | continue current onboarding step | cloud authority / Section `04` | locally mark onboarding complete | onboarding UI follows canonical state only |
| identity / voice | verification status, step-up requirement, allowed next action | explicit user participation in verification flows | cloud identity authority / Section `07` | claim local biometric authority | device posture may assist only |
| device unlock / local auth | current proof requirement, step-up requirement, and OS-auth prompt status | participate in OS-auth or step-up only when cloud posture requires it | user/OS auth plus cloud identity authority | unlock the device autonomously, treat local unlock as session authority, or bypass OS auth | `unlock` remains user/OS-auth controlled |
| persistence / sync | queued, in-flight, replay-due, acked, dead-letter, recovery posture | retry through canonical path, reread state | cloud persistence / sync law / Section `05` | silently drop, auto-heal, or mark success locally | outbox and sync are operational visibility surfaces |
| governance | governance version, certification/quarantine posture, governed severity | acknowledge or inspect governed posture | PH1.GOV / Section `09` | rewrite severity or quarantine | governance remains visibility and decision law |
| proof | protected proof status and evidence linkage refs | trigger only the user-facing step that causes canonical protected action | PH1.J plus canonical storage | write proof locally or use proof as substitute for authority | proof is visibility after authoritative action |
| runtime law | final `ALLOWED`, `DENIED`, `STEP_UP_REQUIRED`, `PENDING_CONFIRMATION`, `BLOCK`, `QUARANTINE`, or degraded posture as rendered outcome | obey or re-enter through lawful explicit interaction | PH1.LAW / Section `11` | override or reinterpret final runtime posture | the UI explains posture; it never authors it |
| interruption posture | TTS stopped, clarify request, continue request, or resume-later hint | accept clarify answer, continue previous topic, switch topic, or resume later when the cloud offers it | PH1.K / PH1.X | invent local interrupt law, fake resume authority, or keep speaking after canonical cancel | interrupt posture is rendered, not authored |
| persona / delivery profile | tone/delivery posture only, including pacing, warmth/directness, and explanation style | later bounded settings/profile actions only where repo truth supports them | PH1.PERSONA / PH1.EMO | display emotional theater, mood meters, or execution-authority persona controls | tone-only / no-meaning-drift / no-execution-authority remains locked |
| memory and artifacts | bounded history, artifact/report previews, memory-backed response visibility | explicit open, explicit recall, explicit restore request where later supported | cloud memory and artifact authority | treat local cache or preview as authoritative memory/artifact truth | cached views remain non-authoritative |

L) freeze-boundary + future-portability matrix
| concern | what G1 freezes | what G1 explicitly does not reopen | portability constraint for later Apple surfaces | notes |
| --- | --- | --- | --- | --- |
| Phase `F` ingress law | iPhone app must terminate into frozen `F2` explicit entry, app-open, and setup receipt law | no redesign of canonical ingress payloads or receipt family | later Apple surfaces must reuse the same ingress law | G1 surfaces frozen Phase `F`, it does not replace it |
| Phase `F` continuity law | iPhone UI states must follow frozen `F3` continuity/session/artifact-sync law | no redefinition of attach/resume/recover/detach, retry, outbox, or apply authority | later Apple surfaces must display the same continuity truth with form-factor-specific chrome only | continuity semantics stay cloud-owned |
| Phase `F` governance / law / proof law | protected posture must follow frozen `F4` visibility and enforcement law | no second governance, proof, or runtime-law path | later Apple surfaces may reuse the same protected posture vocabulary | UI vocabulary must stay aligned to canonical envelope posture |
| Phase `F` closure law | readiness is evidence-driven, not file-driven | no reopening of Phase `F` closure criteria | later Apple surfaces may add shell-specific evidence, not new law | G1 assumes Phase `F` is frozen upstream law |
| future Apple portability | one cloud-authoritative app law across iPhone now and later Apple surfaces | no Mac-specific redesign inside G1 | future Apple form factors may change layout, navigation density, and window chrome only after reusing the same non-authority, `EXPLICIT_ONLY`, cloud-authoritative law | portability here is legal alignment, not Mac design detail |
| phase boundary | G1 is design law for the iPhone app only | no G2, no implementation plan, no platform widening | later work must consume G1 rather than reopen it casually | this document ends at G1 |

M) COMPLETION CRITERIA
- G1 is complete only if reviewers can read one lawful iPhone app design from this file without inventing local authority, local proof, local governance, or local law.
- The app must remain explicitly session-bound, not standalone.
- The app must preserve first-class/non-authority posture, `EXPLICIT_ONLY`, cloud-authoritative parity, no native client claim, no wake parity claim, and no autonomous unlock claim.
- The app must read as one dominant responsive session surface plus bounded lawful adjunct surfaces only when runtime posture requires them.
- The app must show both directions of conversation, keep history in the `side drawer`, use windowed incremental history loading, keep `System Activity` / `Needs Attention` separate from transcript history, and lazy-load heavy content.
- The app must treat `resume context` as bounded PH1.M output only; `72h` and `30d` are resume-tiering rules, not transcript-loading rules.
- The app must surface alerts, urgency, followup, reminder, fallback, and haptic posture from `PH1.BCAST` and must not invent unsupported lock-screen, sound, or unlock behavior.
- The app must render PH1.K / PH1.X `interrupt` posture, clarify/continue/resume-later outcomes, and immediate TTS cancel semantics without inventing local interrupt law.
- The app must preserve screenless-first interaction law by acting as a governed rendering surface for current session truth, governed outputs, alerts, artifacts, and recovery posture.
