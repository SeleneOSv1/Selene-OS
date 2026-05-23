# Selene PH1.BCAST / PH1.DELIVERY / PH1.REM — Broadcast Journey Intelligence + Delivery Orchestration Master Design

DOCUMENT STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

The repo-truth extraction remains the factual base:

- `docs/SELENE_PH1BCAST_BROADCAST_DELIVERY_REMINDER_REPO_TRUTH_FUNCTIONALITY_EXTRACTION_MASTER_DESIGN.md`

This document defines future upgrade architecture pending Grand Architecture Reconciliation.

Future implementation requires repo-truth activation, approved file scope, tests, backend evidence, provider-off/fake-provider proof, and JD live proof where visible.

## 1. Executive Target

PH1.BCAST, PH1.DELIVERY, and PH1.REM already provide strong mechanical foundations.

PH1.BCAST owns the broadcast/message lifecycle.

PH1.DELIVERY owns delivery provider-attempt truth.

PH1.REM owns reminder timing.

Selene OS owns orchestration bridges between the owners.

The future target is a higher-level human communication journey layer above those mechanical owners.

Selene should understand messy human communication requests, resolve recipients and channels, check permissions and consent, discover simulations, confirm before sending, draft high-quality messages through PH1.WRITE and OpenAI through PH1.D, send through the correct delivery owner, track status, troubleshoot failures, and schedule or follow up through reminders.

This must happen without letting clients, Adapter, PH1.BCAST, PH1.DELIVERY, PH1.REM, or OpenAI grant access, authority, or protected execution.

The governing product standard is:

Selene may understand communication requests probabilistically.

Selene may only send, schedule, retry, or broadcast through deterministic owner gates.

## 2. Current Repo-Truth Foundation

The repo-truth extraction establishes the current foundation:

- PH1.BCAST owns broadcast/message lifecycle.
- PH1.DELIVERY owns delivery provider-attempt truth.
- PH1.REM owns reminder timing.
- Selene OS owns orchestration bridges.
- PH1.LINK creates links; PH1.BCAST and PH1.DELIVERY send links.
- PH1.ONB owns onboarding after activation.
- PH1.WRITE boundary is underdefined.
- Live provider proof is missing.
- Recipient/audience resolver is partial.
- Contact consent/unsubscribe is missing.
- Privacy handshake is partial/design-only.
- Audit is partial.
- JD live acceptance is missing.

This document does not duplicate the full extraction. It defines the future journey intelligence that should sit above those owners and route into them without changing current repo truth.

## 3. Human Request To Communication Journey

The standard future flow is:

User messy request
→ PH1.D / GPT-5.5 semantic proposal
→ PH1.N meaning candidates
→ PH1.X Request Decision Lattice
→ communication intent resolved
→ recipient/audience resolved
→ message type resolved
→ channel resolved
→ content sensitivity resolved
→ access/permission/consent checked
→ simulation/capability discovered
→ PH1.WRITE drafts/validates message content
→ confirmation requested
→ execution only after confirmation
→ PH1.BCAST creates/updates lifecycle
→ PH1.DELIVERY sends or tracks provider attempt
→ PH1.REM schedules/fires follow-up if needed
→ PH1.WRITE explains status/failure/follow-up
→ audit evidence recorded

Understanding is probabilistic.

External sending is deterministic-gated.

Protected/private content requires access, authority, and policy where applicable.

## 4. Probabilistic Understanding Layer

Users may request communication in endless ways:

- "Tell Tom I sent the link."
- "Send everyone the roster update."
- "Message staff about tomorrow."
- "Ping Sarah next week."
- "Follow up with Tim if he doesn't answer."
- "Send my brother a soft message."
- "Let the new employee know the onboarding link is ready."
- "Broadcast this to all managers."
- "Tell customers the delivery is delayed."
- "Remind me to chase Tom tomorrow."

GPT-5.5/OpenAI through PH1.D may propose likely meaning.

PH1.N extracts candidates:

- communication intent,
- recipient / audience,
- message type,
- delivery channel,
- timing,
- urgency,
- sensitivity,
- protected/private risk,
- missing fields,
- clarification need.

PH1.X validates:

- lane,
- risk,
- required gates,
- owner,
- simulation need,
- confirmation need.

No phrase patches.

No keyword routing as architecture.

## 5. Communication Intent Classes

| Communication Intent Class | Likely Owner | Risk Level | Required Permission | Simulation Required | PH1.WRITE Required | Confirmation Required | Delivery Provider Required |
| --- | --- | --- | --- | --- | --- | --- | --- |
| direct personal message | PH1.X route, PH1.WRITE content, PH1.BCAST lifecycle, PH1.DELIVERY send | Low to moderate | Sender/contact permission | Yes for external send | Yes | Yes | Yes if external |
| friend/personal outreach draft | PH1.WRITE | Low | Usually none beyond memory/privacy scope | No unless state is mutated | Yes | No for draft only | No |
| friend/personal outreach send | PH1.WRITE + PH1.BCAST/PH1.DELIVERY | Moderate | Recipient/contact permission | Yes | Yes | Yes | Yes |
| invite/link delivery | PH1.LINK creates; PH1.BCAST/PH1.DELIVERY sends | Moderate to high depending link | Link permission + recipient/contact permission | Yes | Yes for message wrapper | Yes | Yes unless copy-only |
| employee/staff notification | PH1.BCAST + PH1.DELIVERY | High | Tenant/workspace role permission | Yes | Yes | Yes | Yes |
| customer notification | PH1.BCAST + PH1.DELIVERY | High | Business permission + contact consent | Yes | Yes | Yes | Yes |
| supplier notification | PH1.BCAST + PH1.DELIVERY | High | Business permission + contact consent | Yes | Yes | Yes | Yes |
| broadcast announcement | PH1.BCAST + PH1.DELIVERY | High blast radius | Audience and sender permission | Yes | Yes | Yes | Yes if external |
| workspace/team announcement | PH1.BCAST + PH1.DELIVERY | Moderate to high | Workspace/team scope permission | Yes | Yes | Yes | Channel-dependent |
| scheduled reminder | PH1.REM | Low to moderate | User/session permission | Yes if state changes | Yes for wording | Yes | No unless external notification |
| follow-up reminder | PH1.REM + PH1.BCAST/PH1.DELIVERY if external | Moderate | User/contact/audience permission | Yes | Yes | Yes | Channel-dependent |
| failed-delivery retry | PH1.DELIVERY + PH1.BCAST/PH1.REM | Moderate | Original send permission or renewed permission | Yes | Maybe for explanation | Yes if user-triggered | Yes |
| emotional outreach draft | PH1.WRITE + Selene Emotional Intelligence | Low to moderate | Memory/privacy scope if used | No unless state is mutated | Yes | No for draft only | No |
| emotional outreach send | PH1.WRITE + PH1.BCAST/PH1.DELIVERY | Moderate | Recipient/contact permission and privacy scope | Yes | Yes | Yes | Yes |
| protected/private content send | PH1.X + Access/Governance + Authority where needed + PH1.WRITE + PH1.BCAST/PH1.DELIVERY | High to very high | Access and possibly authority | Yes | Yes | Yes, serious | Yes |
| system notification | Owning system stack + PH1.BCAST/PH1.DELIVERY or client notification owner | Variable | Owning stack permission | Yes if state changes | Usually templated/PH1.WRITE-reviewed | Context-dependent | Channel-dependent |
| unknown/unsupported communication type | PH1.X + capability request owner if present | Unknown | Unknown | No execution without capability | PH1.WRITE explains | Clarify or decline | No until supported |

## 6. Recipient / Audience Resolver

The future recipient/audience resolver must resolve:

- one named person,
- contact candidate,
- all staff,
- managers,
- employees,
- contractors,
- customers,
- suppliers,
- workspace members,
- tenant members,
- people with pending onboarding,
- people with failed delivery,
- custom audience,
- unknown/ambiguous audience.

Rules:

- audience resolution must be access-scoped,
- sender must have permission to message the audience,
- private/company audience lists must not be exposed to unauthorized users,
- broad broadcasts require stronger confirmation,
- high-risk audiences require serious wording,
- PH1.BCAST must not invent audiences from vague language,
- PH1.N may propose audience candidates,
- PH1.X validates,
- Access/Governance authorizes.

Examples:

"Tell staff the roster changed" requires staff audience resolution.

"Message managers" requires role/audience resolution.

"Send customers an update" requires customer audience resolution and business permission.

## 7. Permission / Consent / Channel Matrix

Communication permissions must be risk-based.

Personal/friend message:

- lower risk if no company data is included,
- still requires recipient/contact and confirmation before send.

Employee/staff notification:

- tenant/workspace permission required,
- may require manager/admin/HR role,
- private content check required.

Customer/supplier notification:

- business permission required,
- contact consent/channel rules required,
- message content classification required.

Broadcast announcement:

- high blast-radius risk,
- audience resolution required,
- confirmation required,
- may require approval depending audience and content.

Protected/private content:

- access and possibly authority required,
- PH1.WRITE must not include protected data unless allowed,
- no send if permission/consent is missing.

Channel rules:

- SMS-first default where allowed,
- email, WhatsApp, WeChat, in-app, and push are optional future channels,
- channel availability and consent must be validated,
- unsubscribe/stop handling must be designed before live SMS/email production,
- delivery provider/cost policy required.

## 8. PH1.WRITE Message Content Boundary

PH1.WRITE owns final message content.

Rules:

- GPT-5.5 may draft message wording through PH1.D.
- PH1.WRITE owns final user-facing/message wording.
- PH1.BCAST must carry approved content/payload refs.
- PH1.DELIVERY must send only approved content refs.
- PH1.REM must not invent reminder message content.
- PH1.BCAST, PH1.DELIVERY, and PH1.REM must not become message writers.
- Sensitive/private/protected content must be classified before sending.

Example:

User:
"Send Tom something soft."

Expected flow:

GPT-5.5 drafts.
PH1.WRITE validates tone/content.
Selene confirms with the requester.
PH1.DELIVERY sends if permitted.

Example:

User:
"Send salary file to Tim."

Expected flow:

PH1.X marks private/protected risk.
Access/Authority gates are required.
PH1.WRITE cannot casually include protected content.
PH1.DELIVERY must not send blocked content.

## 9. OpenAI / GPT-5.5 Interconnection

All OpenAI usage must flow through PH1.D / Provider Governance.

OpenAI may assist with:

- understanding messy communication requests,
- drafting message content,
- rewriting in a tone,
- summarizing status for the user,
- drafting failure explanations,
- drafting reminder wording,
- drafting capability-request tickets,
- multilingual message wording,
- emotional outreach drafts.

OpenAI must not:

- choose final recipient,
- approve audience,
- approve delivery,
- send messages,
- read private contacts without access,
- include private content without permission,
- bypass PH1.WRITE,
- bypass Access/Governance,
- bypass Simulation.

Required proof later:

- provider-off zero attempt,
- fake-provider deterministic drafts,
- malformed provider output rejection,
- no raw provider JSON in message,
- no hidden provider fallback,
- cost/counter evidence.

## 10. Simulation Discovery / Capability Finder

Before external send or schedule, Selene must locate an approved simulation/capability.

Simulation discovery must check:

- exact simulation name,
- aliases/synonyms,
- active simulations,
- retired simulations,
- pending simulations,
- similar supported flows,
- stack owner,
- delivery dependency,
- reminder dependency,
- access/permission dependency,
- provider/channel dependency.

If simulation exists:

- continue to confirmation.

If no simulation exists:

- run Missing Simulation Protocol.

No simulation means no execution.

OpenAI may suggest likely capability names.

Selene must verify against the canonical registry.

## 11. Confirmation Gate Before External Send

External send requires confirmation.

After Selene resolves:

- recipient/audience,
- content summary,
- channel,
- timing,
- sensitivity,
- simulation,
- permission,

Selene must confirm.

Examples:

"Just confirming: do you want me to send Tom the onboarding link by SMS?"

"Just confirming: do you want me to message all staff about the roster change?"

"Just confirming: do you want me to schedule a reminder to follow up with Sarah tomorrow morning?"

Rules:

- broad broadcast confirmation must name audience,
- high-risk/private confirmation must name risk,
- delivery method must be visible,
- changed details require re-check and reconfirmation,
- user can cancel or correct before send,
- no confirmation means no send.

## 12. Missing Simulation Protocol

If no valid simulation/capability exists, Selene must not merely say no.

Selene must triple-check:

1. exact simulation registry
2. alias/synonym match
3. similar supported capability
4. pending capability backlog
5. retired/legacy flow
6. correct stack owner
7. channel/provider availability

If still not found:

- create capability request ticket if a capability-request owner exists,
- otherwise mark `CAPABILITY_REQUEST_OWNER_NEEDED`.

Ticket must include:

- original user request,
- normalized meaning,
- communication type,
- recipient/audience type,
- delivery channel,
- timing/schedule if any,
- missing capability,
- business reason,
- requester id/scope,
- tenant/workspace if applicable,
- risk notes,
- suggested owner,
- source conversation refs,
- audit refs.

PH1.WRITE/OpenAI may draft user-friendly notification:

"I don't have an approved simulation for that communication flow yet. I've created a capability request for review."

Capability request is not approval to execute.

## 13. Delivery Channel Strategy

Default future delivery strategy:

- SMS-first where recipient/contact/consent/provider policy allows.

Other channels:

- email,
- WhatsApp,
- WeChat,
- in-app,
- push,
- QR/copy link if link-related,
- future approved channels.

Rules:

- PH1.DELIVERY owns provider-attempt truth,
- Provider Governance owns live provider/cost policy,
- no live provider claim until proven,
- no hidden provider sends,
- no startup probes,
- provider-off means zero attempts,
- fake provider must be testable,
- live provider requires explicit activation and JD proof,
- channel failures must be visible to status assistant.

## 14. Reminder Natural Language Journey

PH1.REM already owns timing mechanics.

Future journey must allow users to say:

- "Remind me tomorrow morning."
- "Ping Tom if he doesn't reply."
- "Follow up next week."
- "Remind Sarah before the meeting."
- "Nudge the team in two hours."

Flow:

GPT-5.5/PH1.N interprets time and intent.

PH1.X validates schedule/reminder lane.

PH1.REM owns timing.

PH1.WRITE owns reminder wording.

PH1.BCAST/PH1.DELIVERY owns message delivery if the reminder sends externally.

Ambiguous time must clarify:

"Do you mean tomorrow morning at 9 AM?"

Protected/private reminder content must remain gated.

## 15. Status Assistant

Selene must be able to answer:

- Was the message drafted?
- Was it confirmed?
- Was it sent?
- Was it delivered?
- Did it fail?
- Was it retried?
- Was it cancelled?
- Did the reminder fire?
- Did the follow-up happen?
- Was the link delivered?
- Did the receiver open/activate the link?
- What should I do next?

Owner map:

- PH1.BCAST provides lifecycle state.
- PH1.DELIVERY provides provider-attempt state.
- PH1.REM provides timing/reminder state.
- PH1.LINK provides activation state for links.
- PH1.ONB provides onboarding state where relevant.
- PH1.WRITE explains clearly.

Status answers must be access-scoped.

## 16. Troubleshooting / Retry / Failure Flow

Selene must help troubleshoot:

- wrong recipient,
- wrong audience,
- wrong channel,
- missing phone/email,
- SMS failed,
- provider unavailable,
- delivery timed out,
- reminder time ambiguous,
- reminder not fired,
- link expired before send,
- link sent but not activated,
- message content wrong before send,
- message content wrong after send,
- broad broadcast accidentally prepared,
- user changed mind.

State-based correction rules:

- before send: edit/cancel draft,
- after send but before delivery confirmed: cancel if provider supports it,
- after delivered: cannot unsend unless channel supports recall; route to correction/follow-up,
- wrong link: PH1.LINK/PH1.ONB/Access determines state,
- failed delivery: retry or alternate channel if allowed,
- ambiguous content/audience: clarify before action.

## 17. Privacy / Sensitive Content Guard

Before sending, Selene must classify message content:

- public-safe,
- personal/private,
- company private,
- sensitive HR/payroll/finance,
- protected action-related,
- regulated/legal/medical/high-risk,
- unknown.

Rules:

- private/company/protected content requires access,
- protected action content may require authority,
- salary/payroll/HR/customer financial information must not be sent casually,
- third-party personal information must be scoped,
- emotional outreach must not reveal private emotional memory without permission,
- PH1.WRITE must phrase sensitive messages carefully,
- PH1.DELIVERY must not send blocked content.

## 18. Broadcast / Delivery / Reminder Status Packets

Logical future packets:

- `CommunicationJourneyRequestPacket`
- `CommunicationIntentCandidatePacket`
- `RecipientCandidatePacket`
- `AudienceResolutionPacket`
- `ChannelResolutionPacket`
- `MessageContentApprovalPacket`
- `CommunicationPermissionDecisionPacket`
- `CommunicationSimulationDiscoveryPacket`
- `CommunicationConfirmationPacket`
- `BroadcastLifecycleRequestPacket`
- `DeliveryHandoffPacket`
- `DeliveryAttemptStatusPacket`
- `ReminderJourneyPacket`
- `CommunicationTroubleshootingPacket`
- `MissingCommunicationCapabilityRequestPacket`

Codex must later map these to repo equivalents.

Do not claim these currently exist.

## 19. Example End-To-End Flows

### Example A — Send invite link by SMS

User:
"Send Tom the invite link."

Flow:

resolve Tom
→ confirm SMS
→ PH1.LINK link ref
→ PH1.BCAST lifecycle
→ PH1.DELIVERY SMS
→ status assistant tracks delivery and activation

### Example B — Broadcast to staff

User:
"Tell all staff the roster changed."

Flow:

resolve staff audience
→ check permission
→ PH1.WRITE drafts message
→ confirm audience/channel/content
→ PH1.BCAST creates lifecycle
→ PH1.DELIVERY sends
→ status assistant tracks

### Example C — Reminder

User:
"Remind Sarah tomorrow morning."

Flow:

resolve Sarah and time
→ clarify if ambiguous
→ PH1.REM schedules
→ if external notification is needed, PH1.BCAST/PH1.DELIVERY handoff

### Example D — Emotional outreach

User:
"Send my brother something gentle."

Flow:

PH1.WRITE/OpenAI drafts
→ recipient/contact/permission
→ confirmation
→ PH1.DELIVERY sends if allowed

### Example E — Missing capability

User:
"WhatsApp all suppliers a compliance reminder."

Flow:

simulation/channel/capability not found
→ Missing Simulation Protocol
→ capability request ticket
→ user notification

### Example F — Failed delivery

User:
"Why didn't Tom get the message?"

Flow:

PH1.DELIVERY status
→ PH1.BCAST lifecycle
→ contact/channel/provider check
→ PH1.WRITE explains and suggests retry/alternate channel

## 20. What Must Not Happen

Codex must not allow:

- no message send without confirmation,
- no PH1.LINK sending,
- no PH1.BCAST creating links,
- no PH1.DELIVERY creating message content,
- no PH1.REM inventing reminder content,
- no Desktop/iPhone external send authority,
- no Adapter provider/business delivery authority,
- no OpenAI direct send,
- no PH1.N candidate execution,
- no PH1.D provider output becoming final message,
- no private/protected content without access/authority,
- no unsupported channel execution,
- no live provider claim from deterministic refs,
- no hidden fallback provider,
- no broad audience send from vague language,
- no phrase-patch communication routing,
- no implementation from this document alone.

## 21. Recommended Future Build Slices

1. Broadcast Journey Intelligence Activation Pack
2. Communication Intent Understanding / PH1.D + PH1.N Proposal Shell
3. Recipient / Audience Resolver
4. PH1.WRITE Message Content Boundary
5. Permission / Consent / Channel Matrix
6. Simulation Discovery / Capability Finder
7. Confirmation Before External Send
8. Provider-Off / Fake Delivery Provider Proof
9. SMS-First Delivery Handoff
10. Link Delivery Journey Integration
11. Reminder Natural Language Scheduling
12. Delivery Status Assistant
13. Troubleshooting / Retry / Failure Flow
14. Privacy / Sensitive Content Guard
15. Live Provider Activation Pack, if approved
16. JD Live Broadcast/Delivery/Reminder Acceptance Pack

## 22. Grand Architecture Reconciliation Note

This document must later be reconciled into:

- PH1.BCAST / PH1.DELIVERY / PH1.REM extraction,
- PH1.D Proposal Gateway,
- PH1.N Meaning Unravelling,
- PH1.X Request Decision Lattice,
- PH1.WRITE Human Presentation,
- PH1.LINK Link Journey,
- Access/Governance,
- SimulationExecutor,
- Provider Governance,
- Desktop/iPhone render-only proof,
- Adapter transport-only proof,
- Selene Emotional Intelligence / outreach drafts,
- Old Compatibility Path Retirement.

This reconciliation must preserve current repo-truth ownership and must not create duplicate broadcast, delivery, reminder, message-writing, provider, or communication-routing brains.

## 23. Final Architecture Sentence

PH1.BCAST, PH1.DELIVERY, and PH1.REM provide Selene's outbound communication mechanics; Broadcast Journey Intelligence turns messy human communication requests into confirmed, permissioned, simulation-backed, PH1.WRITE-approved, provider-governed delivery journeys with status, troubleshooting, reminders, and audit.
