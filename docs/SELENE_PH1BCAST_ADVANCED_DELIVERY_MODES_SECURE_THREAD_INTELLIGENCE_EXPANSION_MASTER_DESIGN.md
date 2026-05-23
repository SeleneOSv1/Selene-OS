# Selene PH1.BCAST / PH1.DELIVERY / PH1.REM — Advanced Delivery Modes + Secure Thread Intelligence Expansion Master Design

DOCUMENT STATUS:
MASTER_DESIGN_EXPANSION
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design expansion.

No runtime code was changed.

This document does not authorize implementation.

The repo-truth extraction remains the factual base.

The Broadcast Journey Intelligence document remains the baseline future journey design.

This document adds advanced privacy, threading, secure media, emergency, connector, and view-confirmation architecture.

Future implementation requires repo-truth activation, approved file scope, tests, backend evidence, provider-off/fake-provider proof, security/privacy proof, and JD live proof where visible.

## 1. Executive Target

Selene's outbound communication system must become more than "send a message."

The target is:

- human-aware delivery,
- privacy-aware delivery,
- interruption-aware delivery,
- auditable delivery,
- searchable delivery,
- secure media/document delivery,
- thread-based operational communication,
- status and view confirmation,
- safe OpenAI-assisted communication intelligence,
- deterministic access/authority/simulation governance.

PH1.BCAST, PH1.DELIVERY, and PH1.REM provide mechanics.

Broadcast Journey Intelligence provides the human request-to-send flow.

This expansion provides advanced delivery behavior and secure operational communication intelligence.

The governing product standard is:

Selene may help humans communicate with intelligence, privacy, timing, and care.

Selene must never turn delivery convenience into access, authority, protected execution, or client-side decision power.

## 2. Current Foundation

PH1.BCAST owns broadcast/message lifecycle.

PH1.DELIVERY owns provider-attempt truth.

PH1.REM owns reminder timing.

PH1.WRITE owns final wording.

PH1.D governs OpenAI proposals.

PH1.N extracts meaning candidates.

PH1.X validates routes and gates.

Access/Governance owns permission.

Authority/Simulation own protected execution.

Desktop/iPhone render only.

Adapter transports only.

Do not merge these owners.

Do not build a single mega broadcast brain.

The future stack must preserve the mechanical split:

- PH1.BCAST governs message/broadcast lifecycle state.
- PH1.DELIVERY governs provider-attempt evidence.
- PH1.REM governs timing/follow-up state.
- PH1.WRITE governs approved content.
- PH1.D governs provider proposals.
- PH1.X validates the route and risk.
- Access, Authority, and Simulation decide what is allowed.

## 3. Advanced Delivery Mode Matrix

| Delivery Mode | Meaning | When Used | Required Permission | Privacy Behavior | PH1.WRITE / TTS Behavior | Audit Requirement | Forbidden Behavior |
| --- | --- | --- | --- | --- | --- | --- | --- |
| OutLoud | Selene may speak the message audibly. | Public-safe or explicitly approved spoken delivery. | Speaker/session permission and content classification allowing audible output. | Content may be heard by nearby people; must not be default for private content. | PH1.WRITE provides spoken-safe wording; PH1.TTS speaks only approved text. | Record mode decision and content classification. | Speaking private/confidential content by default. |
| DeviceOnly | Deliver content silently or visually to a verified device. | Private payroll, HR, finance, personal, confidential, or crowded-room contexts. | Device/session/access scope. | Content is rendered privately on device; no audible disclosure by default. | PH1.WRITE may produce display-first wording; PH1.TTS remains silent or gives a generic notice. | Record device target, access posture, and privacy decision. | TTS reading full blocked content aloud. |
| EarOnly | Speak through approved private audio route only. | Personal earbuds/headset, approved accessibility mode, private voice channel. | Device route proof and policy permission. | Audio route must be private enough for policy. | PH1.WRITE creates concise spoken wording; PH1.TTS must verify approved route. | Record route proof and content class. | Assuming speakers are private without proof. |
| Hybrid | Speak a safe summary and deliver details privately. | Sensitive but useful conversational flow. | Access to details plus delivery target permission. | Summary is public-safe; details go DeviceOnly/SecureLinkOnly. | PH1.WRITE splits spoken summary from private detail text. | Record summary/detail split. | Letting summary reveal protected facts. |
| NoStore | Deliver transient content without durable conversational storage where policy allows. | Ephemeral low-risk notices or sensitive one-time codes where supported. | Policy allowing non-retention and delivery owner support. | Minimize retained body; keep required audit hashes/proof only. | PH1.WRITE must avoid durable context claims. | Retain required audit and proof refs. | Using NoStore to hide protected execution evidence. |
| StepUpRequired | Recipient must re-authenticate before viewing. | Confidential docs, payroll, HR, finance, legal, high-risk operations. | Recipient identity/access step-up. | Content hidden until step-up passes. | PH1.WRITE explains access requirement without exposing content. | Record step-up request/result. | Showing content before step-up. |
| SecureLinkOnly | Deliver a secure link instead of raw content. | Reports, PDFs, evidence bundles, confidential media. | Link/access/document permission. | Content remains behind governed link/access controls. | PH1.WRITE provides safe wrapper; no content dump in message. | Record link id, expiry, view/open evidence where available. | Sending confidential attachment as casual raw text/file. |
| SilentNotification | Notify without audible or intrusive content. | Low urgency, private context, after-hours quiet rules. | Notification permission and channel policy. | No sensitive preview unless allowed. | PH1.WRITE writes minimal notification text. | Record delivery and mode. | Exposing private message preview on lock screen where blocked. |
| AcknowledgementRequired | Recipient must explicitly acknowledge receipt. | Safety, operations, policy, incident, urgent follow-up. | Sender authority and recipient scope. | Acknowledgement state stored in thread/lifecycle. | PH1.WRITE makes acknowledgement request clear. | Record ack request and result. | Treating delivered/read as acknowledged. |
| ActionConfirmationRequired | Recipient must confirm a requested action separately from reading. | Operational tasks, incident instructions, delivery instructions, protected-adjacent workflows. | Action owner permission and simulation where action mutates state. | Reading does not equal action consent. | PH1.WRITE separates "read" from "confirm action." | Record action confirmation separately. | Executing a task/roster/schedule change from view receipt alone. |

Examples:

Private payroll message:

- `DeviceOnly` or `StepUpRequired`.

Emergency safety message:

- interruption allowed where policy permits,
- `AcknowledgementRequired`.

Confidential report:

- `SecureLinkOnly`,
- step-up,
- no casual forwarding.

## 4. Privacy Negotiation Before Content

Private/confidential content must use privacy negotiation before content exposure.

For private/confidential content, Selene must ask before exposing content:

"This is private. Should I send it to your device, or is it okay to say it out loud?"

Rules:

- private/confidential content is not spoken aloud by default,
- `DeviceOnly` is the safe default,
- `OutLoud` requires explicit recipient confirmation where policy allows,
- `EarOnly` is allowed only where policy permits,
- `Hybrid` may speak a summary but deliver details privately,
- PH1.TTS must not speak blocked content,
- clients render only approved mode.

Privacy negotiation must happen before content detail is emitted.

PH1.WRITE owns the wording of the privacy prompt.

PH1.X validates whether a privacy negotiation is required.

Access/Governance validates whether the requester and recipient may receive the content at all.

## 5. Sender Review And Message Approval

Before sending sensitive, private, confidential, broad, high-reach, external, or high-impact messages, Selene must show the sender what will be sent.

PH1.WRITE owns the reviewed message.

OpenAI/GPT-5.5 may draft through PH1.D.

Sender review must include:

- recipient/audience,
- channel,
- message preview,
- classification,
- attachments,
- delivery mode,
- expiry/ack requirement if any.

Rules:

- simple message may have lightweight confirmation,
- priority/private/confidential/company-wide requires mandatory review,
- modified/reconstructed message requires approval before send,
- no hidden rewriting after approval.

Approved content must be frozen by content hash or equivalent approved-content ref before PH1.BCAST or PH1.DELIVERY carries it.

If the sender changes wording, recipient, audience, delivery mode, attachment, timing, or channel, Selene must re-run the relevant permission, content classification, and confirmation gates.

## 6. Audience Preview And Reach Confirmation

Before group, segment, department, customer, supplier, or company-wide broadcast, Selene must preview audience scope.

Example:

"This will go to 184 employees in Production and Packaging. Continue?"

Rules:

- no blind mass broadcasts,
- show audience type and approximate/exact count where policy allows,
- show include/exclude filters,
- show classification,
- show channel,
- show reach cap status,
- require explicit confirmation,
- require access/authority where applicable.

Audience preview must not expose private audience lists to unauthorized senders.

For high-risk audiences, Selene should show enough to prevent mistakes without leaking protected data.

Examples:

- "all managers" should resolve to a governed role/audience selector.
- "customers in Sydney" should resolve through a business-permitted customer audience owner.
- "everyone" should never send until Selene clarifies the intended scope.

## 7. Abuse, Spam, Rate Limit, And Overload Protection

Advanced protections are required:

- rate limiting,
- duplicate detection,
- anti-spam controls,
- harassment prevention,
- audience overload protection,
- cooldowns for repeated broadcast,
- sender risk scoring,
- emergency misuse detection,
- repeated-contact protection,
- after-hours quiet rules where policy applies.

Rules:

- protections fail closed when high risk,
- OpenAI may help explain refusal,
- OpenAI may not override safety controls,
- PH1.BCAST/Access/Governance own policy enforcement.

Examples:

- repeated messages to the same recipient may require cooldown or confirmation,
- broad repeated broadcasts may require elevated approval,
- emotionally charged outreach may be slowed or redirected to draft-only mode,
- after-hours delivery may be deferred unless urgent/emergency policy permits.

Abuse controls must distinguish:

- user frustration,
- legitimate urgent operations,
- harassment risk,
- accidental duplicate sends,
- malicious or compromised-account behavior.

## 8. Emergency / Life-Critical Delivery

Emergency messages may need stronger delivery posture than normal communication.

Emergency messages:

- may override normal politeness,
- may interrupt more strongly,
- require explicit simulation/policy,
- require audit,
- require acknowledgement,
- may escalate across devices/channels/managers/emergency contacts if policy allows,
- must not be theatrical or manipulative,
- must be concise and clear.

Emergency classification must not be chosen casually by OpenAI.

PH1.X, Access, Authority, and Simulation must validate emergency classification and emergency delivery behavior.

Emergency delivery may use:

- `AcknowledgementRequired`,
- `ActionConfirmationRequired`,
- multi-channel escalation,
- louder/more visible client presentation where policy permits,
- manager escalation if acknowledgement fails,
- reminder/retry loops through PH1.REM.

Emergency delivery must not:

- invent danger,
- use fear to manipulate,
- bypass identity/access,
- bypass simulation,
- send to unapproved emergency contacts,
- overstate delivery/read/view certainty.

## 9. Threaded Bi-Directional Operational Conversations

Broadcasts may become structured threads.

Thread examples:

- sender sends incident photos,
- recipient asks for more information,
- sender adds PDF,
- department responds,
- thread closes when resolved.

Thread states:

- open,
- waiting for sender,
- waiting for recipient,
- waiting for info,
- blocked,
- escalated,
- resolved,
- expired,
- archived.

Rules:

- every message is recorded,
- every attachment is linked,
- replies remain in the same deterministic thread unless split,
- thread ownership must be access-scoped,
- PH1.M may support memory/search but does not own broadcast truth,
- PH1.WRITE phrases thread summaries.

Threading must preserve:

- participant list,
- tenant/workspace scope,
- access posture,
- content classification,
- attachment refs,
- delivery attempts,
- acknowledgement/action-confirmation state,
- timestamps,
- audit refs.

External replies must not automatically mutate operational state. They become admitted thread evidence until the correct owner validates any requested action.

## 10. Searchable Communication Memory

Selene must support future thread search and recall.

Searchable by:

- time,
- topic,
- participants,
- labels,
- attachments,
- classifications,
- unresolved status,
- department,
- customer/supplier,
- incident/project,
- delivery status.

Example:

"Find the incident thread from December 2019."

Rules:

- search is access-scoped,
- private/confidential threads require permission,
- Selene presents best matches,
- user confirms selected thread,
- PH1.M/Search may assist but BCAST/Thread owner keeps canonical communication evidence.

Search results should distinguish:

- exact match,
- likely match,
- related thread,
- archived thread,
- inaccessible/private thread,
- stale or unresolved thread.

Selene may summarize matching threads through PH1.WRITE only after access checks and evidence acceptance.

## 11. Secure Document / PDF / Media / Report Delivery

Selene must support secure delivery of:

- PDFs,
- reports,
- images,
- videos,
- meeting notes,
- sales materials,
- evidence bundles,
- operational documents.

Controls:

- secure preview vs download,
- document-level read confirmation,
- page/section-level view tracking where technically available,
- step-up for confidential content,
- watermarking where required,
- annotation/comment permissions,
- OCR/search where allowed,
- screenshot/download restrictions where policy allows,
- expiry/self-destruct rules where policy allows,
- audit logs for open/view/respond/forward/ignore.

Rules:

- PH1.ART / PH1.DOC / PH1.EXPORT / media owners own artifact/document truth where applicable.
- PH1.BCAST/DELIVERY owns delivery/thread state.
- PH1.WRITE owns summaries/explanations.
- PH1.D/OpenAI may summarize only after permission and data-egress checks.

Secure document delivery must not convert a protected document into casual message text.

Secure document delivery must not overclaim view tracking where the channel cannot prove it.

## 12. AI-Assisted Report Intelligence

OpenAI/GPT-5.5 through PH1.D may assist with:

- summarize PDF/report,
- executive summary,
- key findings,
- risk extraction,
- action items,
- compare reports,
- explain in simple terms,
- multilingual summaries,
- read-aloud summaries.

Rules:

- PH1.D governs provider/data egress,
- PH1.WRITE validates final summary,
- PH1.E/PH1.DOC/media owner provides document evidence,
- OpenAI may not fabricate findings,
- unauthorized users cannot summarize private documents,
- sensitive docs may require local/private model or denial.

Report intelligence output must preserve:

- document source refs,
- confidence/uncertainty,
- accepted evidence boundaries,
- redaction policy,
- access scope,
- delivery mode.

OpenAI may produce a draft summary, but Selene must own the accepted final summary.

## 13. Meeting Intelligence

Selene must support meeting-related communication content:

- meeting transcript,
- meeting minutes,
- decisions made,
- unresolved items,
- action owners,
- follow-up reminders,
- meeting summary broadcast,
- searchable meeting memory.

Rules:

- recording mode and meeting artifact pipeline own capture/artifact truth,
- PH1.WRITE writes minutes/summaries,
- PH1.REM schedules follow-up,
- PH1.BCAST/DELIVERY sends only approved outputs,
- protected action items remain to-dos unless authorized simulation executes.

Meeting intelligence must separate:

- what was said,
- what was decided,
- what is an action item,
- what requires authority,
- what is only a reminder,
- what may be broadcast.

Selene must not turn meeting notes into completed protected actions.

## 14. Marketing And Sales Communication Assistance

Selene may assist authorized marketing/sales workflows:

- rewrite promotions,
- generate customer-targeted messaging,
- multilingual campaigns,
- customer segment tone adaptation,
- social/email/SMS variants,
- campaign scheduling,
- approval before publishing/sending.

Rules:

- audience authority required,
- opt-out rules required,
- rate limits required,
- no sending without approval,
- no OpenAI direct send,
- PH1.WRITE validates final content,
- PH1.D governs provider use.

Marketing and sales communication must account for:

- customer consent,
- channel eligibility,
- campaign frequency,
- content claims,
- brand/legal review where required,
- unsubscribe/stop handling,
- segment privacy.

OpenAI may draft variants. Selene must validate audience, content, approval, and delivery gates.

## 15. External Channel Connectors

Future approved channels may include:

- email,
- SMS,
- WhatsApp,
- Slack,
- Microsoft Teams,
- customer portals,
- supplier portals,
- push notifications,
- in-app notifications,
- future approved connectors.

Rules:

- connector/channel owner must be explicit,
- provider governance required,
- no hidden provider calls,
- provider-off means zero attempts,
- fake-provider proof required,
- live provider proof required before claiming live sends,
- channel restrictions and consent apply.

Each connector must define:

- send capability,
- receive/intake capability,
- delivery status capability,
- read/view receipt capability,
- attachment support,
- threading support,
- rate limits,
- privacy posture,
- provider-off/fake-provider proof.

## 16. External Content Intake

Selene must support replies and files coming back from external channels.

Examples:

- email replies,
- SMS replies,
- uploaded PDFs,
- customer documents,
- supplier portal responses,
- photos/videos/files.

Rules:

- external content is untrusted until admitted,
- prompt injection defenses apply,
- attachments need artifact/media owners,
- replies join canonical thread only after validation,
- private/protected data is scoped,
- audit records source channel and content hash.

External intake must preserve:

- channel,
- sender identity posture,
- raw source refs where allowed,
- normalized content refs,
- attachment refs,
- thread association candidate,
- trust posture,
- security scan status,
- audit refs.

PH1.D/OpenAI may help summarize or classify admitted content only after provider governance and data-egress checks.

## 17. Forward / Modify / Reply / Redelivery Flow

Selene must support forwarding and redelivery where policy permits.

Forwarding:

- sender may forward permitted content/thread/report,
- sender may ask Selene to summarize/translate/clean before forwarding,
- PH1.WRITE shows modified version for approval,
- classification and retention rules travel with content,
- confidential content cannot be forwarded unless policy allows.

Reply:

- recipient may reply with text/files/questions,
- replies stay in thread,
- new authorized thread may be created only through approved flow.

Redelivery:

- retry failed delivery,
- alternate channel if allowed,
- reissue secure link if expired,
- preserve audit.

Modify:

- before send, sender may edit content, audience, channel, attachments, timing, or delivery mode,
- after approval, any change invalidates approval and requires re-review,
- after delivery, corrections become follow-up messages or owner-specific correction workflows.

## 18. View Confirmation And Read Receipts

Selene should support view confirmation where policy and channel allow.

Statuses:

- sent,
- delivered,
- opened,
- partially viewed,
- fully viewed,
- ignored,
- responded,
- requested more info,
- forwarded where permitted,
- acknowledged,
- action-confirmed.

Rules:

- do not overclaim if provider/channel cannot prove it,
- distinguish delivered from read,
- distinguish opened from fully viewed,
- document page/section view tracking only where technically available,
- sender visibility depends on policy.

Read receipts are evidence, not authority.

View confirmation does not mean:

- the recipient understood,
- the recipient approved,
- a protected action may execute,
- a task is completed,
- a roster/schedule/business state changed.

## 19. Task / Scheduler / Roster Handoff

Broadcast may create or interact with:

- tasks,
- reminders,
- scheduled work,
- roster-impacting instructions,
- operational actions.

Rules:

- PH1.BCAST may carry communication,
- TASK/SCHED/ROSTER owners handle their own state,
- no task/schedule/roster mutation without authority and simulation,
- PH1.X splits communication vs mutation,
- PH1.WRITE explains split.

Example:

"Ask Tom to pick up tissues on the way back."

Could be:

- message only,
- task suggestion,
- scheduled task,

depending on permission and confirmation.

Selene must not silently convert a message into a task, a reminder into a roster change, or a read receipt into operational completion.

## 20. Advanced Packets / Logical Contracts

Logical future packets:

- `DeliveryModeDecisionPacket`
- `PrivacyNegotiationPacket`
- `SenderReviewPacket`
- `AudiencePreviewPacket`
- `AbuseProtectionDecisionPacket`
- `EmergencyDeliveryPacket`
- `OperationalThreadPacket`
- `ThreadSearchRequestPacket`
- `SecureDocumentDeliveryPacket`
- `ViewConfirmationPacket`
- `ExternalConnectorSendPacket`
- `ExternalConnectorIntakePacket`
- `ForwardModifyApprovalPacket`
- `ReportIntelligenceRequestPacket`
- `MeetingIntelligenceBroadcastPacket`
- `MarketingCampaignCommunicationPacket`
- `TaskSchedulerHandoffPacket`

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 21. Example Advanced Flows

### Example A — Private DeviceOnly Message

User asks Selene to send private payroll note.

Flow:

content classified private
→ `DeviceOnly` required
→ sender review
→ recipient step-up if needed
→ delivery
→ read confirmation if allowed

### Example B — Company-Wide Broadcast

User says:

"Tell everyone production starts at 7."

Flow:

audience resolver
→ reach preview
→ sender authority
→ PH1.WRITE message
→ confirmation
→ delivery

### Example C — Confidential PDF

User sends confidential report.

Flow:

secure document delivery
→ step-up
→ view tracking
→ no casual download/forward

### Example D — Threaded Incident

Sender sends incident photo.

Receiver asks question.

Sender replies with extra evidence.

Thread remains open until resolved.

### Example E — External Reply Intake

Customer replies by email with PDF.

Flow:

connector intake
→ artifact admission
→ thread update
→ status assistant summarizes

### Example F — Emergency

Authorized manager sends emergency safety alert.

Flow:

emergency classification validated
→ interruption/ack required
→ escalation if not acknowledged

## 22. What Must Not Happen

Codex must include:

- no implementation from this document alone,
- no merging PH1.BCAST/PH1.DELIVERY/PH1.REM into one mega engine,
- no OpenAI direct send,
- no OpenAI audience expansion,
- no OpenAI classification upgrade to emergency/confidential without validation,
- no PH1.D provider output as final message,
- no PH1.WRITE bypass,
- no confidential content spoken out loud by default,
- no broad audience send without preview,
- no live provider claim without proof,
- no view confirmation overclaim,
- no external connector intake without admission/security,
- no task/scheduler/roster mutation from broadcast alone,
- no Desktop/iPhone authority,
- no Adapter business/provider ownership,
- no old invalid persona/product names.

## 23. Recommended Future Build Slices

1. Advanced Broadcast Expansion Activation Pack
2. Delivery Mode Matrix + Privacy Negotiation
3. Sender Review + Audience Preview
4. Abuse/Spam/Rate-Limit Protection
5. Emergency Delivery Rules
6. Operational Thread Model
7. Searchable Thread Memory
8. Secure Document/PDF/Media Delivery
9. View Confirmation / Read Receipt Proof
10. External Connector Send/Intake Proof
11. Forward/Modify/Reply/Redelivery Flow
12. AI Report Intelligence Boundary
13. Meeting Intelligence Broadcast Flow
14. Marketing/Sales Communication Flow
15. Task/Scheduler/Roster Handoff Boundary
16. JD Live Advanced Broadcast Acceptance Pack

## 24. Grand Architecture Reconciliation Note

This document must later be reconciled into:

- BCAST/DELIVERY/REM extraction,
- Broadcast Journey Intelligence,
- PH1.D Proposal Gateway,
- PH1.WRITE Human Presentation,
- PH1.N Meaning Unravelling,
- PH1.X Request Decision Lattice,
- PH1.M Human Memory,
- Search/Document/Artifact stacks,
- Task/Scheduler/Roster stacks,
- Provider Governance,
- Identity + Access + Authority,
- Full Duplex/Barge-In for interruptions,
- Universal Language Intelligence,
- Desktop/iPhone render-only proof,
- Adapter transport-only proof,
- Old Compatibility Path Retirement.

Grand Architecture Reconciliation must preserve the owner split and must not create duplicate delivery, thread, document, connector, writing, provider, memory, or routing brains.

## 25. Final Architecture Sentence

Broadcast Advanced Delivery Modes + Secure Thread Intelligence extends Selene's outbound communication system beyond simple sending: it governs privacy-aware delivery modes, audience preview, sender review, threaded operational conversations, secure document/media delivery, external connector intake, view confirmation, emergency delivery, and AI-assisted report/meeting/marketing communication while preserving PH1.WRITE, PH1.D, PH1.BCAST, PH1.DELIVERY, PH1.REM, Access, Authority, Simulation, Desktop, and Adapter boundaries.
