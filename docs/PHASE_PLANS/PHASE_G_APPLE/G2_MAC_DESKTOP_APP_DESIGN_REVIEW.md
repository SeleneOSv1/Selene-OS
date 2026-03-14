PHASE G2 — MAC DESKTOP APP DESIGN REVIEW

A) REVIEW SCOPE
- G2 designs the Mac Desktop app only as the lawful Selene desktop session interface, lawful desktop wake-word / explicit-entry interface where repo truth allows, and governed rendering surface for conversation, alerts, artifacts, history, system activity, and protected outputs.
- G2 does not redesign iPhone, Android, Tablet, or Windows, and it does not create a standalone desktop product shell.
- G2 does not implement code, create tickets, reopen frozen Phase F or frozen G1 law, or claim that a native Mac Desktop client already exists in-tree.

B) FROZEN UPSTREAM DESIGN POSITION
- The Mac Desktop app is a first-class platform surface, but it is not an authority source.
- G2 inherits frozen G1 law and state model as the Apple design anchor: session-bound posture, transcript vs `resume context` distinction, separate `System Activity` / `Needs Attention`, cloud-owned authority / governance / law / proof boundaries, PH1.BCAST-derived alert posture, PH1.K / PH1.X interruption posture, and tone-only personality presentation.
- G2 is cloud-authoritative, session-bound, and non-authoritative for identity, session truth, memory truth, artifact truth, proof, governance, and runtime law.
- Frozen upstream law consumed here is Phase `A1-A5`, `B1-B3`, `C1/C3/C4`, `D1-D4`, `E1-E4`, `F1-F5`, frozen `G1`, and Build Sections `01-11`.

C) G1 INHERITANCE + DESKTOP POSTURE
- The Mac Desktop app is not a free-floating desktop home shell. It is the desktop session interface for the same Selene cloud runtime already frozen for iPhone in G1.
- G2 inherits identically in law from G1:
  - one dominant responsive `session surface`
  - session-bound and onboarding-bound state derivation from cloud truth
  - transcript history distinct from PH1.M memory
  - bounded memory-derived `resume context`
  - separate `System Activity` / `Needs Attention`
  - cloud-owned authority / governance / law / proof posture
  - no local authority, no local proof, no local governance, no fake emotional UI
- What may differ from iPhone is presentation only:
  - wider window layout, denser sidebar usage, multi-pane detail presentation, keyboard-oriented affordances, and desktop window chrome
  - desktop alert chrome and notification placement where capability and product policy allow
  - desktop wake-word-first entry posture where frozen repo truth supports it
- One dominant responsive `session surface` remains the primary model. Bounded adjunct surfaces are limited to:
  - a collapsible history sidebar for explicit history access
  - a separate operational `System Activity` surface and `Needs Attention` list
  - bounded artifact/report/detail panes or windows tied to the current session
  - onboarding takeover surfaces while required onboarding gates remain active
  - inline restriction banners/cards or hard full-window takeover states when runtime posture requires them
- The desktop app follows session and onboarding law; session and onboarding law do not follow the desktop app.
- The UI should remain highly readable and conversation-centered, but it should be adapted for desktop layout rather than copied mechanically from iPhone.
- Wake-word-first is the intended Mac primary entry posture because repo truth already freezes Desktop as `wake word or explicit entry`, but explicit entry, app-open, and invite-open remain lawful canonical routes. G2 therefore treats wake-word-first as supported desktop posture, not as a bypass around canonical ingress.
- Visual adaptation does not change runtime law. The app may render, request, synchronize, and acknowledge; it may not become local simulation, local authority, local proof, local governance, or local law.

D) DEPENDENCY LAW
- Build Section `01`: cloud runtime is the only authority; the desktop app is a terminal and synchronized visibility surface only.
- Build Section `02`: the app is session-first; all lawful desktop states derive from cloud-owned session truth rather than local invention.
- Build Section `03`: the desktop app may enter only through canonical ingress routes and canonical runtime-envelope creation.
- Build Section `04`: onboarding progression, authority, artifact activation, and protected execution remain cloud-authoritative.
- Build Section `05`: retry, replay, outbox, reconciliation, and sync posture must remain envelope-driven, idempotent, and auditable.
- Build Section `06`: history and memory views are cached windows over cloud truth, never local memory authority.
- Build Section `07`: identity, voice, and biometric decisions remain cloud-owned; the desktop app may display status only.
- Build Section `08`: Desktop supports wake or explicit trigger, but platform entry differences affect session entry only, never runtime execution law.
- Build Section `09`: governance posture is visible in the envelope and may affect desktop UI posture, quarantine posture, and operator-visible severity.
- Build Section `10`: any quantitative thresholds rendered by the desktop UI are derived read-only outputs, never local judgment.
- Build Section `11`: final `BLOCK`, `DEGRADE`, `QUARANTINE`, or `SAFE_MODE` runtime posture remains cloud-authored.
- Frozen Phase `A1-A5` consumed: the desktop app may surface artifact refs, setup receipts, trust visibility, proof status, and protected failure posture only through canonical cloud-authoritative trust, proof, governance, and law transport.
- Frozen Phase `B1-B3` consumed: Mac wake or microphone posture may not create a second trust / proof / enforcement path or a weaker session / capture / enforcement model than the canonical mobile/desktop parity law already frozen upstream.
- Frozen Phase `C1/C3/C4` consumed: receipts are evidence only; lifecycle, memory, restore, archive, purge, and protected-complete posture remain cloud-owned and may not be authored by the desktop app.
- Frozen Phase `D1-D4` consumed: attach, resume, recover, detach, stale, retry, lease, ordering, reconciliation, and protected session posture remain cloud-decided.
- Frozen Phase `E1-E4` consumed: presentation and personalization may shape delivery only; adaptive behavior, emotional guidance, safety, identity, and runtime law remain constraint layers rather than UI-authored behavior.
- Frozen Phase `F1-F5` consumed: G2 may inherit the frozen Apple/iPhone ingress, continuity, governance/law/proof, and evidence-driven closure law, but it may not redefine any frozen `F` boundary.
- Frozen `G1` consumed: G2 must reuse the same Apple session law, state model, separation rules, and protected-boundary vocabulary while adapting presentation only for desktop.

E) CURRENT / TARGET / GAP
- `CURRENT`: repo truth already provides canonical session states (`Closed`, `Open`, `Active`, `SoftClosed`, `Suspended`), canonical session attach / recovery / reconciliation posture in the `RuntimeExecutionEnvelope`, canonical onboarding next-step and status contracts, append-only conversation storage distinct from PH1.M memory, bounded PH1.M `resume context` output, durable sync queue / ack / retry / dead-letter mechanics, `PH1.BCAST` urgency / waiting / followup / fallback mechanics, PH1.K / PH1.X interruption continuity posture, and desktop-specific onboarding/setup evidence such as `desktop_wakeword_configured`, `desktop_pairing_bound`, and wake runtime readiness receipts. Repo truth also already freezes Desktop as `wake word or explicit entry` with persistent wake listeners allowed. Repo truth does not provide a native Mac Desktop app implementation or a separate desktop authority path today.
- `TARGET`: G2 freezes one lawful Mac Desktop app design with one dominant responsive desktop `session surface`, wake-word-first entry where lawful, explicit entry still canonical, bounded lawful adjunct surfaces, separate history / `resume context` / `System Activity`, lazy detail panes for heavy governed content, PH1.BCAST-derived alerts, PH1.K / PH1.X interruption posture, and no widening of authority beyond cloud-owned Selene runtime truth.
- `GAP`: G2 does not claim a native desktop client, a local authority cache, local proof writer, local governance engine, local memory truth, or any desktop-specific implementation already present in-tree. Exact macOS notification-center chrome and exact profile-selection UX remain bounded presentation/product decisions where repo truth does not freeze them as current implementation fact.

F) LAWFUL DESKTOP APP-STATE MODEL
- The desktop app-state model is derived directly from `SessionState`, `SessionAttachOutcome`, `PersistenceRecoveryMode`, `ReconciliationDecision`, `OnboardingNextStep`, and `OnboardingStatus`.
- The desktop app must always open into one lawful state derived from canonical cloud truth; it may not open into an unbound generic home shell.
- Rendering carriers are fixed:
  - dominant desktop `session surface` for normal session-open, active, soft-closed, and bounded resume posture
  - collapsible history sidebar for explicit history recall only
  - inline restriction card/banner for soft restrictions while the main session surface remains lawful
  - full-window takeover for hard restrictions or gated onboarding states when normal session interaction is not lawful
  - separate operational surface for `System Activity` and `Needs Attention`
  - bounded detail pane or detached detail window for artifacts, reports, charts, and alert detail tied to the current session
- `SessionAttachOutcome::NewSessionCreated|ExistingSessionReused|ExistingSessionAttached|RetryReusedResult` changes the inline continuity label and recovery explanation inside the main desktop session surface; it does not create separate behavioral models.
- `PersistenceRecoveryMode::Recovering|DegradedRecovery|QuarantinedLocalState` and `ReconciliationDecision::RetrySameOperation|ReusePriorAuthoritativeOutcome|RejectStaleOperation|RequestFreshSessionState|QuarantineLocalState` determine whether the user sees a soft inline restriction, an operational `System Activity` item, or a hard full-window takeover state.
- Canonical base states:
  - `WAKE_READY`: desktop wake runtime is configured and the app is ready for wake-word-first entry while remaining attached to canonical ingress law.
  - `EXPLICIT_ENTRY_READY`: wake is unavailable, disabled, or not currently used; the desktop app is ready for explicit keyboard/UI entry, app-open, invite-open, or explicit history recall only.
  - `ONBOARDING_ENTRY_ACTIVE`: app-open or deep-link flow created an onboarding session and the next step is `Install`, `Terms`, `LoadPrefilled`, `AskMissing`, or platform setup; onboarding renders as a bounded takeover shell rather than a separate product shell.
  - `SESSION_OPEN_VISIBLE`: cloud session exists and is open but not currently in active spoken turn posture.
  - `SESSION_ACTIVE_VISIBLE`: cloud session exists and is active; the conversation view is live and dual transcript is visible.
  - `SESSION_SOFT_CLOSED_VISIBLE`: the recoverable session remains cloud-owned, and any resume affordance or `resume context` card must read from the authoritative session container rather than local draft truth.
  - `SESSION_SUSPENDED_VISIBLE`: the session is suspended cloud-side; the app may render status and lawful next action only, and this posture may require a hard full-window takeover.
- Canonical overlay states:
  - `RECOVERING`
  - `DEGRADED_RECOVERY`
  - `QUARANTINED_LOCAL_STATE`
  - `PENDING_CONFIRMATION`
  - `STEP_UP_REQUIRED`
  - `NEEDS_ATTENTION`
- Turn-level governed adjunct postures:
  - `RESUME_CONTEXT_VISIBLE`: derived from PH1.M bounded resume output and rendered as a short desktop catch-up card with thread title, pending work, and 1-3 summary bullets only
  - `INTERRUPT_VISIBLE`: derived from PH1.K / PH1.X interruption truth when accepted interrupt cancels TTS immediately and continuity requires clarify, continue, or resume-later posture inside the current session surface
- Soft vs hard restriction rendering law:
  - soft restriction = strong inline restriction card/banner while the main session surface remains visible
  - hard restriction = full-window takeover when suspended posture, blocked onboarding, quarantine, or other protected runtime posture makes normal interaction unlawful

G) desktop entry + session-state matrix
| app-visible state | canonical cloud source | default desktop posture | lawful user controls | forbidden local inference | notes |
| --- | --- | --- | --- | --- | --- |
| `WAKE_READY` | Desktop wake runtime configured and current session/onboarding truth allows wake entry | dominant desktop `session surface` with wake-listening-ready status and recent thread window | wake-word entry, explicit keyboard/UI entry, invite open, history recall | no local session invention, no bypass around canonical ingress | wake-word-first is supported on desktop, but still terminates into the same ingress law |
| `EXPLICIT_ENTRY_READY` | `SessionState::Closed` or no active turn with no lawful wake-ready posture | dominant desktop `session surface` in explicit-entry posture | explicit voice/text turn, invite open, app-open continue, history recall | no synthetic active session | there is still no generic standalone dashboard |
| `ONBOARDING_ENTRY_ACTIVE` | invite/app-open outcome plus onboarding session record and `OnboardingNextStep` / `OnboardingStatus` | bounded onboarding takeover shell inside the desktop window | continue onboarding, submit required fields, provide setup evidence, confirm device, continue verification | no skipping gates, no local completion, no fake readiness | desktop-specific setup evidence includes wake readiness and pairing receipts where required |
| `SESSION_OPEN_VISIBLE` | `SessionState::Open` plus `SessionAttachOutcome` | dominant desktop conversation surface, ready for next turn | wake entry where lawful, explicit entry, inspect recent history sidebar, inspect current detail panes | no local promotion to `Active`, no hidden new session | attach outcome changes inline continuity labeling only |
| `SESSION_ACTIVE_VISIBLE` | `SessionState::Active` plus current `turn_id` / envelope | live desktop conversational thread with both directions visible in real time | speak, type, interrupt lawfully, inspect current governed outputs and detail panes | no local turn authority, no local decision shortcut | live transcript remains text-visible even when spoken |
| `SESSION_SOFT_CLOSED_VISIBLE` | `SessionState::SoftClosed` plus optional presence nudge and bounded PH1.M output | visually quiet session surface with explicit resume affordance and optional bounded `resume context` card | resume explicitly, inspect recent archived slice, dismiss screen | no auto-reopen from local cache alone | visual reset may quiet the surface, but archive truth remains durable |
| `SESSION_SUSPENDED_VISIBLE` | `SessionState::Suspended` | hard full-window takeover with explanation and allowed next step only | read status, perform only lawful recovery or exit actions | no local unsuspend, no silent continuation | suspended posture is cloud-authored |
| `RECOVERING` / `DEGRADED_RECOVERY` / `QUARANTINED_LOCAL_STATE` | `persistence_state.recovery_mode` and `reconciliation_decision` | inline restriction when session remains visible; escalate to hard takeover when quarantine removes lawful normal interaction | reread authoritative state, retry only through canonical path, inspect failure details | no local override, no stale-cache trust, no hidden replay | overlays change posture, not ownership |

H) wake-word / explicit-entry matrix
| entry route | repo truth support | desktop app posture | setup evidence or gate surfaced | forbidden behavior | notes |
| --- | --- | --- | --- | --- | --- |
| wake-word entry | Desktop supports wake word; persistent wake listeners are allowed | wake-word-first when wake readiness is configured and current runtime posture allows it | `desktop_wakeword_configured`, wake runtime readiness receipt, current device/platform status | no wake bypass around canonical ingress, no local authority, no fake wake support when receipts are absent | wake entry changes entry posture only, not execution law |
| explicit keyboard / UI entry | Desktop supports explicit entry alongside wake | explicit keyboard shortcut, click, or UI action opens the same canonical session-entry path | current session state, onboarding state, and runtime legality | no alternate execution path, no local shortcut into protected action | explicit entry remains lawful even when wake is primary |
| app-open / relaunch | canonical client open remains lawful platform ingress | reopen directly into current lawful session or onboarding posture | app instance, session attach outcome, recovery posture | no local assumption that wake had to be used | app-open is still canonical even for wake-first desktop posture |
| invite-open / deep link | canonical invite/app-open law reused from G1/F2 | open into onboarding session context or current tenant/user context | token, tenant hint, deep-link nonce, onboarding session id | no local invite activation without canonical request | no desktop-specific ingress fork |
| wake unavailable / disabled / degraded | wake runtime not configured, temporarily disabled, or disallowed by posture | fall back to explicit entry without changing session law | missing or degraded wake receipt / runtime readiness state | no silent wake simulation, no pretending wake exists | unsupported or degraded wake remains visible state, not hidden failure |
| onboarding wake setup | desktop onboarding requires wake enrollment / readiness receipts | onboarding takeover shows exact remaining desktop wake setup tasks only when lawful | `desktop_wakeword_configured`, `desktop_pairing_bound`, voice enrollment, platform readiness receipts | no local completion claim before canonical receipt commit | desktop onboarding differs from iPhone only in platform setup evidence, not in overall state model |

I) conversation + history + system-activity matrix
- `transcript history` is append-only conversation recall for both directions of the session and appears in the main session surface plus explicit history sidebar recall.
- `resume context` is PH1.M-derived bounded catch-up output only: selected thread, pending work, resume tier, and 1-3 summary bullets.
- `System Activity` is the operational surface for sync / replay / reconciliation / delivery status that Selene manages cloud-side.
- `Needs Attention` is the human-actionable subset of `System Activity` or protected runtime posture; only real human action belongs there.
| surface | canonical backing truth | default desktop load posture | explicit user actions | forbidden behavior | notes |
| --- | --- | --- | --- | --- | --- |
| live dual transcript | typed input enters the same pipeline as voice; `conversation_ledger` stores user and Selene turns append-only | current turn plus recent slice visible by default in the dominant desktop `session surface` | speak, type, inspect per-turn details | no one-way transcript, no local-only fork, no hidden spoken-only output | both Selene output and user speech / typed input appear as live text |
| history sidebar | append-only `conversation_ledger` and archived session recall | collapsible sidebar with recent visible window only | open prior thread, `Load older messages`, `Show more history`, collapse sidebar | no eager full-history load, no cross-session blending, no raw memory ledger dump | desktop uses sidebar density instead of iPhone drawer, but the law is identical |
| `resume context` | PH1.M `selected_thread_id`, `selected_thread_title`, `pending_work_order_id`, `resume_tier`, and `resume_summary_bullets` | bounded inline catch-up card in the main session surface; 1-3 bullets only | resume selected thread, start fresh, open named thread explicitly | no raw memory dump, no 72h/30d transcript preload, no full archive dump | `72h` / `30d` govern resume surfacing tiers, not transcript loading |
| `System Activity` | persistence acknowledgement state, reconciliation decision, broadcast waiting/followup/reminder state, sync queue/dead-letter posture, and recovery posture | separate operational list/pane from history and transcript | inspect status, reread authoritative state, continue canonical flow, open linked detail | no manual resend console, no local transport repair authority, no hidden auto-heal claim | user sees state, but Selene manages sync/retry/dedupe/reconcile |
| `Needs Attention` | unresolved protected prompts, blocked onboarding, stale/recovery warnings, dead-letter sync, law/governance failure posture, or broadcast state that now requires human action | separate actionable list from normal thread and plain history | open item, acknowledge, retry through canonical path, inspect reason, complete required human action | no mixing unresolved operations into normal scrollback, no non-actionable clutter | `Needs Attention` is operational, not conversational |
| `Pending` / `Failed` operational queues | `PendingState`, pending confirmation, step-up, tool wait, verification pending, dead-letter sync, denied authority, failed delivery visibility | separate operational queues from history | resolve confirmation, finish step-up, continue onboarding, inspect failure, retry lawfully | no local completion of pending work, no silent disappearance of failed work | desktop may show them with more list density than iPhone, but not different law |

J) artifact / chart / long-content matrix
| content class | canonical backing truth | default desktop presentation | explicit user actions | forbidden behavior | notes |
| --- | --- | --- | --- | --- | --- |
| governed artifacts | cloud artifact refs and current thread linkage | bounded detail pane or detached detail window from the current session surface | open artifact, switch back to thread, inspect metadata | no local activation, no local truth fork | artifacts remain thread-bound, not separate app homes |
| charts and dashboards | governed report/output refs and read-only derived values | desktop reading pane optimized for width, inspection, and pagination | open chart, resize pane, open next page, collapse pane | no local recomputation, no free-form judgment layer | presentation may use desktop width; law remains Section `10` read-only |
| long written outputs | current thread outputs and report artifacts | ChatGPT-like reading posture adapted for desktop column width and scrolling density | expand, continue reading, detach detail view, return to thread | no eager hydration of all historical long outputs | readability may differ from iPhone through layout only |
| report previews | artifact/report summary refs plus current session context | summary card first, full content lazy-loads on explicit open | open report, jump back to source turn, open linked operational detail | no preloading every report in memory | lazy-load remains required |
| operational detail linkage | `System Activity` item or `Needs Attention` item linked to artifact/report/output | open linked detail beside the session surface, not as a second behavioral model | inspect detail, resolve action, close detail | no direct mutation or repair authority from the detail pane | detail surfaces are adjunct panes, not alternate app shells |

K) alert / interruption / blocked-posture matrix
- `PH1.BCAST` defines phone-first lifecycle, urgency mapping, `WAITING -> FOLLOWUP -> REMINDER` logic, bounded attempts, and fallback only when the Selene App is unavailable.
- G2 therefore reuses the same alert law as G1 while adapting desktop presentation only. The desktop app may render synchronized alert state, but it may not redefine phone-first lifecycle ownership or fallback order.
- Exact macOS notification-center chrome is still a presentation/product surface, not runtime law. The desktop app may use in-app banners/cards and capability-gated OS notification chrome where lawful.
| case | governing truth | default desktop display posture | sound / interruption posture | capability gate | notes |
| --- | --- | --- | --- | --- | --- |
| urgent governed alert | `PH1.BCAST` classification and runtime law posture | visible urgent banner/card in the desktop session surface and linked operational item | stronger alert posture; desktop notification sound only where platform policy/capability allows | capability-gated | urgency comes from cloud classification, not app guesswork |
| non-urgent delivery or followup | `PH1.BCAST` waiting window and non-urgent followup policy | quieter banner/list item with deferred emphasis | lighter or no sound by default; still capability-gated | capability-gated | non-urgent followup may wait before escalation |
| waiting / reminder handoff | `PH1.BCAST` `WAITING`, `FOLLOWUP`, `REMINDER_SET`, and `REMINDER_FIRED` lifecycle | operational state visible in `System Activity` until it becomes user-facing again | no local timer invention; any voice interruption remains cloud/policy-owned | capability-gated | timing comes from cloud lifecycle, not desktop timers |
| accepted `interrupt` during speech | PH1.K / PH1.X accepted interruption truth | current session surface shows speech stopped and the continuity outcome inline | TTS stops immediately; clarify/continue/resume-later posture is rendered exactly when carried by the response | no local interrupt invention | interruption is rendered, not authored |
| soft restriction | governance/law/pending/recovery posture that still permits visible session | strong inline restriction card/banner while session remains visible | no sound unless alert policy separately allows it | capability-gated | same law as G1, different desktop density only |
| hard restriction / suspended / quarantine | suspended posture, blocked onboarding, quarantine, or other protected runtime posture | hard full-window takeover with explanation and allowed next action only | no local override, no background continuation | capability-gated | normal interaction is not lawful in this posture |
| persona / emotional presentation | PH1.PERSONA / PH1.EMO bounded tone-only outputs | no dedicated emotion panel; tone is expressed through delivery style only | no sound or alert behavior may be invented from emotional posture | capability-gated | no theatrical emotion UI |

L) G1-inheritance + freeze-boundary matrix
| concern | what G2 inherits identically from G1 | what may differ only in desktop presentation | what G2 explicitly does not reopen | notes |
| --- | --- | --- | --- | --- |
| app-law anchor | session-bound, non-authority, cloud-authoritative, one dominant session surface, transcript/history/resume/system-activity separation | wider layout, sidebar density, panes, keyboard affordances, window chrome | no redesign of iPhone law or state semantics | G1 remains the Apple law anchor |
| entry semantics | canonical ingress, app-open/invite-open legality, explicit entry legality | Desktop may be wake-word-first because repo truth supports wake or explicit trigger | no alternate ingress family and no bypass around canonical entry | entry posture may differ; ingress law does not |
| continuity and sync | same session attach/recovery/reconcile semantics, same Selene-managed sync/retry/dedupe/reconcile posture | more visible operational density on desktop | no local transport repair authority, no alternate outbox law | desktop shows more, but owns no more |
| alerts and interruption | same PH1.BCAST lifecycle ownership and same PH1.K / PH1.X interruption posture | desktop banners, panes, and sound posture may differ within capability gates | no new alert state machine, no local interrupt law | desktop presentation adapts; runtime law does not |
| personality and emotional model | same tone-only, no-meaning-drift, no-execution-authority model | no theatrical UI; only delivery style can vary | no emotional dashboard, no execution-authority persona controls | onboarding/config-driven profile UX remains bounded design surface only |
| phase boundary | G2 is Mac Desktop design law only | none | no Android/Windows design, no implementation plan, no reopening of frozen F or G1 boundaries | this document ends at G2 |

M) COMPLETION CRITERIA
- G2 is complete only if reviewers can read one lawful Mac Desktop app design from this file without inventing local authority, local proof, local governance, or local law.
- The desktop app must preserve first-class/non-authority posture and cloud-authoritative parity.
- The desktop app must inherit frozen G1 law and state model identically wherever platform entry mechanics do not require a frozen desktop-specific difference.
- The desktop app must read as one dominant responsive desktop session surface plus bounded lawful adjunct surfaces only when runtime posture requires them.
- The desktop app must keep transcript history, bounded PH1.M `resume context`, `System Activity`, and `Needs Attention` explicitly separate.
- The desktop app must keep history windowed and incremental and must not reinterpret `72h` / `30d` memory tiering as transcript preload.
- The desktop app must surface alerts, urgency, waiting, followup, reminder, fallback, and sound posture from `PH1.BCAST` and must not invent unsupported local alert law.
- The desktop app must render PH1.K / PH1.X `interrupt` posture, clarify/continue/resume-later outcomes, and immediate TTS cancel semantics without inventing local interrupt law.
- The desktop app must preserve tone-only / non-theatrical personality presentation and must not create fake emotional UI.
