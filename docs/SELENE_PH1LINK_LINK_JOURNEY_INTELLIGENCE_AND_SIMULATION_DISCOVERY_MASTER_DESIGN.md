# Selene PH1.LINK Link Journey Intelligence + Simulation Discovery Master Design

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design. No runtime code was changed. This document does not authorize implementation.

The PH1.LINK repo-truth extraction remains the factual base:

- `docs/SELENE_PH1LINK_LINK_ENGINE_REPO_TRUTH_FUNCTIONALITY_EXTRACTION_MASTER_DESIGN.md`

This document defines future upgrade architecture pending Grand Architecture Reconciliation. It does not replace current repo truth, implement a new runtime path, delete old paths, or authorize PH1.LINK to own delivery, onboarding, access, authority, or simulation execution.

## 1. Executive Target

PH1.LINK currently handles link lifecycle mechanics: generate invite/onboarding links, store token-to-draft mappings, validate token signatures, activate links from app-open context, expire/revoke/block tokens, bind device/app-open evidence, and hand activated context to PH1.ONB.

Selene now needs a higher-level human link journey layer around that mechanical boundary.

The target future layer must understand messy human requests, determine what type of link is needed, identify the recipient, resolve role/access template implications, check permission, discover a valid simulation/capability, confirm the resolved action with the requester, generate the link through PH1.LINK, hand delivery to Broadcast/Delivery, track receiver activation, hand onboarding to PH1.ONB, and help sender or receiver troubleshoot when the journey goes wrong.

The governing product standard is:

Selene should let a human say something natural like "Recruit Tom as CEO" and safely turn that into a governed, confirmed, simulation-backed, SMS-first link journey, without letting fuzzy language, raw links, clients, Adapter, or PH1.LINK itself grant authority.

## 2. Current Repo-Truth Foundation

The repo-truth extraction establishes the current foundation:

- PH1.LINK owns link lifecycle.
- PH1.ONB owns onboarding after activation.
- PH1.BCAST / PH1.DELIVERY own sending and delivery.
- Access/Governance own access, role, permission, tenant, and future workspace scope.
- Authority + Simulation own protected execution.
- Desktop/iPhone render/open only.
- Adapter transports only.

Current PH1.LINK repo truth supports:

- invite/onboarding draft link generation,
- token/signature validation,
- app-open activation,
- device fingerprint binding,
- forwarded-link blocking,
- expiry,
- revocation,
- expired recovery,
- role proposal draft evidence,
- dual-role conflict escalation draft evidence,
- PH1.ONB handoff after activation.

This document does not duplicate the full extraction. It builds the future human journey layer that should sit above the current link lifecycle mechanics and route into the existing canonical owners.

## 3. Human Request To Link Journey

The standard future flow is:

User messy request
→ PH1.D / GPT-5.5 semantic proposal
→ PH1.N meaning candidates
→ PH1.X Request Decision Lattice
→ link intent resolved
→ link type resolved
→ recipient resolved
→ role/access template resolved
→ delivery method resolved
→ permission checked
→ simulation discovered
→ confirmation requested
→ execution only after confirmation
→ PH1.LINK generates link
→ PH1.BCAST / PH1.DELIVERY sends link
→ PH1.LINK tracks activation
→ PH1.ONB takes over onboarding

This journey has two different kinds of intelligence:

- Probabilistic understanding for what the human probably means.
- Deterministic execution for what Selene is allowed to do.

PH1.D, GPT-5.5, and PH1.N may help understand the request. PH1.X validates the route and gates. Access/Governance validates scope. Simulation discovery proves the capability exists. Confirmation proves the user accepts the resolved action. PH1.LINK generates only the approved link. PH1.BCAST / PH1.DELIVERY sends only after the delivery owner is invoked. PH1.ONB owns onboarding after activation.

## 4. Probabilistic Understanding Layer

Users may request links in endless ways:

- "Add my friend Tom."
- "Send Tom a link."
- "Help my friend join us."
- "Recruit Tom."
- "Onboard Tim as CEO."
- "Invite Sarah to the company."
- "Send the new CFO invite."

GPT-5.5/OpenAI through PH1.D may propose the likely meaning.

PH1.N extracts candidates:

- link intent candidate,
- recipient candidate,
- recipient type candidate,
- role/access candidate,
- tenant/workspace candidate,
- delivery channel candidate,
- urgency candidate,
- missing field candidate,
- protected-risk hint,
- ambiguity reason.

PH1.X validates:

- lane,
- risk,
- recipient,
- link type,
- access implications,
- required gates,
- simulation need,
- clarification need,
- confirmation need.

Important law:

Understanding is probabilistic.

Execution is deterministic.

A model may propose that "Recruit Tom as CEO" means "create an executive onboarding link for Tom as CEO." That proposal cannot generate or send a link until deterministic permission, simulation, owner, and confirmation gates pass.

## 5. Link Intent Classes

| Link Intent Class | Likely Owner | Permission Level | Risk Level | Simulation Required | Confirmation Required |
| --- | --- | --- | --- | --- | --- |
| friend / personal connection link | PH1.LINK for lifecycle; Access/Governance for allowed relationship scope | Low to moderate | Low if no company/private access is granted | Yes if link generation mutates link state or sends delivery | Yes before generation/sending |
| customer link | PH1.LINK lifecycle; CRM/customer onboarding owner if present; Access/Governance | Business-scoped | Moderate | Yes | Yes |
| supplier link | PH1.LINK lifecycle; supplier onboarding/compliance owner if present; Access/Governance | Business-scoped | Moderate to high | Yes | Yes |
| employee onboarding link | PH1.LINK lifecycle; PH1.ONB onboarding; Access/Governance | Manager/admin/HR or approved equivalent | High | Yes | Yes |
| contractor onboarding link | PH1.LINK lifecycle; PH1.ONB onboarding; Access/Governance | Manager/admin/HR/vendor-owner scope | High | Yes | Yes |
| executive onboarding link | PH1.LINK lifecycle; PH1.ONB onboarding; Access/Governance; Authority if approval needed | Elevated authority | Very high | Yes | Yes, serious confirmation |
| workspace invite link | PH1.LINK lifecycle if approved; Access/Governance/workspace owner | Workspace admin or approved role | Moderate to high | Yes | Yes |
| tenant invite link | PH1.LINK lifecycle if approved; Access/Governance/tenant owner | Tenant admin or approved role | High | Yes | Yes |
| device/app link | PH1.LINK lifecycle or device-link owner if split later | User/session/device scope | Moderate | Yes if state changes | Yes if delivery or binding occurs |
| voice enrollment link, if later approved | PH1.LINK lifecycle; Voice Identity/Onboarding/Access owner | Identity/onboarding scope | High | Yes | Yes |
| access/role proposal link | PH1.LINK proposal/lifecycle only; Access/Governance owns role | Elevated depending role | High to very high | Yes | Yes, serious confirmation |
| unknown / unsupported link type | PH1.X + capability request owner if present | Unknown | Unknown | No execution without valid simulation | Ask clarification or missing simulation protocol |

Rules:

- Link type determines the required access gate.
- Link lifecycle does not imply access authority.
- PH1.LINK must not grant access.
- PH1.LINK must not decide the final role or permission template.
- Unsupported link types must not be executed by analogy.

## 6. Permission And Access Matrix

### Friend Link

Friend links are usually low-risk when they do not grant company access, private memory, tenant scope, role scope, workspace membership, or protected execution rights.

Expected controls:

- many users may be allowed to create a public-safe friend/personal connection link,
- no company access is granted,
- no role is granted,
- no protected authority is granted,
- recipient/contact permission is still required for delivery,
- confirmation is required before generation/sending.

### Employee Onboarding Link

Employee onboarding links are business-scoped and can create downstream access implications.

Expected controls:

- tenant/workspace permission required,
- manager/admin/HR or approved equivalent role may be required,
- approved onboarding simulation required,
- role/access template must be resolved,
- missing fields must be explicit,
- confirmation is required before generation/sending.

### CEO / CFO / Executive Link

Executive links carry high access implications.

Expected controls:

- access template resolution required,
- elevated authority or approval may be required,
- role must be confirmed explicitly,
- delivery channel must be confirmed explicitly,
- protected serious wording is required,
- no silent CEO/CFO correction is allowed,
- no execution from fuzzy role guesses.

Example:

"Recruit Tom as CEO" and "send Tom the CFO invite" are not casual synonyms. If Selene is uncertain, she must ask.

### Supplier / Customer Link

Supplier/customer links may affect CRM, supplier onboarding, compliance, contracts, account records, or service access.

Expected controls:

- business permission may be required,
- correct owner stack must be selected,
- approved simulation required,
- delivery permission and channel validation required,
- access and data scope must be separated from link lifecycle.

### Global Permission Rules

Raw user phrase does not grant permission.

Raw link does not grant authority.

Link type determines the required access gate.

Role/access template must be validated by Access/Governance.

PH1.LINK does not grant access.

PH1.LINK does not grant authority.

PH1.LINK does not decide tenant/workspace permission.

Desktop/iPhone do not decide access.

Adapter does not decide access.

## 7. Simulation Discovery / Capability Finder

Before Selene creates or sends a link, she must locate an approved simulation/capability.

Simulation discovery must check:

- exact simulation name,
- aliases/synonyms,
- stack owner,
- active simulations,
- retired simulations,
- pending simulations,
- similar supported flows,
- delivery dependency,
- onboarding dependency,
- access/role dependency.

The Simulation Finder must answer:

- Is there an approved simulation for this link journey?
- Which owner owns it?
- Is it active?
- Is it retired?
- Is it pending?
- Is there a similar supported flow?
- Does delivery require a separate simulation?
- Does onboarding require a separate simulation?
- Does access/role assignment require a separate simulation?
- Does this journey require user confirmation?
- Does this journey require elevated authority?

If a valid simulation exists:

- continue to confirmation.

If no valid simulation exists:

- run Missing Simulation Protocol.

No simulation means no execution.

Simulation discovery is not model guessing. GPT-5.5/OpenAI may propose likely capability names or synonyms, but Selene must verify against the canonical simulation/capability registry.

## 8. Confirmation Gate Before Execution

After Selene resolves the user request and finds a simulation, Selene must confirm with the requester before generating or sending.

Example:

User:
"Recruit Tom as CEO."

Selene:
"Just confirming: do you want me to create an onboarding link for Tom as CEO and send it by SMS?"

If user says yes:

- proceed through the approved deterministic path.

If user says no:

- ask clarification,
- do not generate,
- do not send.

If user changes details:

- update candidates,
- re-run permission/simulation check where needed,
- confirm again.

Confirmation must be clear, short, and human.

PH1.WRITE and Selene emotional presentation may phrase it naturally.

Protected or high-access roles require serious confirmation:

- recipient,
- role,
- tenant/workspace,
- delivery method,
- link type,
- authority implication,
- action effect.

Confirmation must not hide material risk behind friendly wording.

## 9. Missing Simulation Protocol

If no valid simulation/capability exists, Selene must not merely say no.

Selene must triple-check:

1. exact simulation registry,
2. alias/synonym match,
3. similar supported capability,
4. pending capability backlog,
5. retired/legacy flow,
6. correct stack owner.

If still not found, Selene must create a capability request ticket, if the ticket/workflow owner exists.

Ticket must include:

- original user request,
- normalized meaning,
- requested link type,
- requested recipient type,
- requested role/access type,
- missing simulation/capability,
- business reason,
- requester id/scope,
- tenant/workspace if applicable,
- priority,
- suggested owner stack,
- risk notes,
- required review team/member,
- source conversation refs,
- audit refs.

If no ticket/workflow engine exists, mark:

CAPABILITY_REQUEST_OWNER_NEEDED

User-facing behavior:

- PH1.WRITE uses GPT-5.5 to explain politely.
- Do not give robotic deterministic failure.
- Tell the user a request has been created or that the capability is unavailable.
- Notify requester later when approved, built, or rejected if notification permission exists.

Example:

"I don't have an approved simulation for that link type yet. I've created a capability request for the technical team to review. I'll let you know when Selene can support it."

Missing simulation does not authorize a shortcut.

Missing simulation does not authorize PH1.LINK to invent a link type.

Missing simulation does not authorize delivery.

## 10. Capability Review And Notification Flow

Future process:

Capability request created
→ assigned to technical/product owner/team
→ reviewed
→ accepted / rejected / needs more info
→ if accepted, build simulation/capability
→ once available, notify original requester
→ if rejected, notify requester politely

OpenAI/GPT-5.5 may draft:

- user notification,
- technical ticket summary,
- rejection explanation,
- capability-available announcement.

Selene owns:

- ticket state,
- audit,
- owner routing,
- notification permission,
- final PH1.WRITE wording.

Capability request review must not become hidden execution. A capability request is a request for future system support, not an approval to perform the unsupported action now.

## 11. Delivery Handoff

PH1.LINK does not send.

After link generation:

- PH1.LINK returns approved link/token/context.
- PH1.BCAST / PH1.DELIVERY sends the link.

Default delivery:

- SMS.

Other possible methods:

- email,
- WhatsApp,
- in-app notification,
- QR code,
- copy link,
- future approved channels.

Delivery engine owns:

- recipient channel validation,
- delivery status,
- retry policy,
- failed delivery,
- sent/delivered/clicked proof where available,
- channel-specific audit.

PH1.LINK owns:

- link lifecycle,
- token,
- validation,
- activation,
- expiry,
- revocation,
- status.

Delivery handoff must preserve:

- requester identity,
- recipient contact,
- delivery channel,
- generated link ref,
- tenant/workspace scope where applicable,
- confirmation ref,
- simulation ref,
- audit refs.

PH1.LINK must never absorb delivery because sending a message is a separate side effect with separate provider, channel, consent, retry, failure, and audit semantics.

## 12. Receiver Activation And Onboarding Handoff

Once receiver opens link:

PH1.LINK:

- validates token,
- validates signature,
- validates status,
- validates device/app-open context,
- validates tenant context,
- activates link or denies.

PH1.ONB:

- starts onboarding,
- asks missing fields,
- resolves onboarding steps,
- hands to Access/Governance where needed.

After activation, Selene must treat onboarding as the active journey.

PH1.LINK is not responsible for completing onboarding.

If onboarding requires access templates, identity proof, voice enrollment, role confirmation, compliance checks, or manager approval, those steps belong to their canonical owners.

## 13. Troubleshooting And Correction Flow

Selene must help the sender or receiver troubleshoot.

### Wrong Role Before Activation

Expected behavior:

- inspect link status,
- revoke wrong link if policy allows,
- generate corrected link if authorized,
- send corrected link through delivery owner,
- audit old and new paths.

### Wrong Role After Activation But Onboarding Incomplete

Expected behavior:

- inspect activation and onboarding state,
- pause/correct onboarding context where policy allows,
- or revoke/reissue depending policy,
- PH1.ONB / Access decides.

### Wrong Role After Onboarding Completed

Expected behavior:

- PH1.LINK no longer owns correction,
- route to Access/Governance / role-change simulation,
- authority may be required.

### Wrong Phone Number

Expected behavior:

- route delivery correction through PH1.BCAST / PH1.DELIVERY,
- revoke/reissue link if exposed,
- require confirmation before sending corrected link.

### Expired Link

Expected behavior:

- use expired recovery if allowed,
- or generate replacement if authorized,
- send through delivery owner.

### Forwarded Link / Device Mismatch

Expected behavior:

- block or deny activation,
- explain safely,
- generate new link only if authorized.

### Example

User:
"I sent Tom the CFO link but he should be CEO."

Selene must determine:

- activated or not,
- onboarding started or not,
- access granted or not,
- safe correction path,
- required authority/simulation.

Selene must not silently edit the role. CEO/CFO mistakes are high-risk access/authority differences.

## 14. Link Status Assistant

Selene should be able to answer:

- Was the link created?
- Was it sent?
- Was it delivered?
- Did receiver click it?
- Was it activated?
- Did onboarding start?
- Did it expire?
- Was it revoked?
- Was it blocked?
- Did delivery fail?
- What should I do next?

Owner map:

- PH1.LINK provides lifecycle status.
- PH1.BCAST / PH1.DELIVERY provide delivery status.
- PH1.ONB provides onboarding status.
- Access/Governance provide access/role status where relevant.
- PH1.WRITE explains clearly.

Status answers must not expose private recipient data to unauthorized users.

Status answers must distinguish:

- link generated,
- link sent,
- link delivered,
- link clicked/opened,
- link activated,
- onboarding started,
- onboarding completed,
- access granted,
- blocked/expired/revoked.

These are not the same state.

## 15. Required Packets / Logical Contracts

These are logical future packets. Codex must later map them to repo equivalents. Do not claim they currently exist.

| Logical Contract | Purpose | Current Existence Claim |
| --- | --- | --- |
| `LinkJourneyRequestPacket` | Captures human request, requester, tenant/workspace, and source refs for a link journey. | FUTURE_LOGICAL_CONTRACT |
| `LinkIntentCandidatePacket` | Carries PH1.D/PH1.N candidate link intents and ambiguity. | FUTURE_LOGICAL_CONTRACT |
| `LinkRecipientCandidatePacket` | Carries recipient candidate, contact candidate, recipient type, and confidence. | FUTURE_LOGICAL_CONTRACT |
| `LinkTypeResolutionPacket` | Records resolved link type and rejected link-type candidates. | FUTURE_LOGICAL_CONTRACT |
| `LinkPermissionDecisionPacket` | Records Access/Governance permission decision and required gate refs. | FUTURE_LOGICAL_CONTRACT |
| `LinkSimulationDiscoveryPacket` | Records exact/alias/similar/pending/retired simulation discovery results. | FUTURE_LOGICAL_CONTRACT |
| `LinkConfirmationPacket` | Records the confirmation prompt, user response, confirmed fields, and changed details. | FUTURE_LOGICAL_CONTRACT |
| `LinkGenerationRequestPacket` | Request from journey layer to PH1.LINK after permission/simulation/confirmation pass. | FUTURE_LOGICAL_CONTRACT |
| `LinkDeliveryHandoffPacket` | Handoff from generated link to PH1.BCAST / PH1.DELIVERY. | FUTURE_LOGICAL_CONTRACT |
| `LinkActivationStatusPacket` | Normalized lifecycle status from PH1.LINK for status assistant and troubleshooting. | FUTURE_LOGICAL_CONTRACT |
| `LinkTroubleshootingPacket` | Captures wrong-link, wrong-role, expired, blocked, delivery failure, and correction context. | FUTURE_LOGICAL_CONTRACT |
| `MissingSimulationCapabilityRequestPacket` | Captures unsupported capability request after Simulation Finder fails. | FUTURE_LOGICAL_CONTRACT |
| `CapabilityReviewNotificationPacket` | Captures review outcome and user notification refs. | FUTURE_LOGICAL_CONTRACT |

Mapping law:

- Reuse current repo contracts when they exist.
- Extend canonical owners rather than duplicate them.
- Do not put these contracts in Desktop/iPhone.
- Do not put execution authority in Adapter.
- Do not make PH1.LINK own the whole journey.

## 16. Example End-To-End Flows

### Example A — Friend Link

User:
"Send Tom a link to connect with Selene."

Flow:

- PH1.D/GPT-5.5 proposes friend/personal connection link meaning.
- PH1.N extracts Tom as recipient candidate and SMS/contact candidate if present.
- PH1.X validates low-risk public/friend link lane.
- Access/Governance confirms requester may create this link type.
- Simulation Finder locates approved friend-link generation and delivery capability.
- PH1.WRITE confirms: "Just confirming: should I create a friend connection link for Tom and send it by SMS?"
- User confirms.
- PH1.LINK creates the link.
- PH1.BCAST / PH1.DELIVERY sends SMS.
- PH1.LINK tracks activation.

### Example B — Employee Onboarding

User:
"Onboard Tim as staff."

Flow:

- PH1.D/GPT-5.5 proposes employee onboarding link.
- PH1.N extracts Tim as recipient and staff role candidate.
- PH1.X marks business onboarding risk.
- Access/Governance checks tenant/admin/manager/HR permission.
- Simulation Finder locates approved employee onboarding link simulation and delivery dependency.
- PH1.WRITE confirms role and delivery method.
- User confirms.
- PH1.LINK creates link.
- PH1.BCAST / PH1.DELIVERY sends link.
- Receiver opens link.
- PH1.LINK activates.
- PH1.ONB starts onboarding after activation.

### Example C — Executive Link

User:
"Recruit Tom as CEO."

Flow:

- PH1.D/GPT-5.5 proposes executive onboarding link.
- PH1.N extracts Tom and CEO as high-access role candidate.
- PH1.X marks high-risk access implication.
- Access/Governance resolves executive access template.
- Authority or elevated approval may be required.
- Simulation Finder must locate approved executive onboarding capability.
- PH1.WRITE asks serious confirmation: "Just confirming: do you want me to create an executive onboarding link for Tom as CEO and send it by SMS?"
- User confirms.
- Execution proceeds only if all gates pass.

### Example D — Missing Simulation

User:
"Send supplier compliance onboarding link."

Flow:

- PH1.D/GPT-5.5 proposes supplier compliance onboarding link.
- PH1.N extracts supplier/compliance link candidates.
- PH1.X validates likely business onboarding request.
- Simulation Finder checks exact simulation, aliases, similar flows, pending backlog, retired flows, and owner stack.
- No valid simulation is found.
- Missing Simulation Protocol creates a capability request ticket if owner exists.
- If no owner exists, mark `CAPABILITY_REQUEST_OWNER_NEEDED`.
- PH1.WRITE explains politely.

### Example E — Wrong Link

User:
"I sent Tom CFO but meant CEO."

Flow:

- PH1.X classifies correction/troubleshooting.
- Link Status Assistant checks link lifecycle.
- Delivery status is checked through PH1.BCAST / PH1.DELIVERY.
- Onboarding status is checked through PH1.ONB.
- Access status is checked through Access/Governance if onboarding completed.
- If not activated, revoke/reissue may be available.
- If activated but onboarding incomplete, PH1.ONB / Access policy decides correction path.
- If onboarding completed or access granted, route to Access/Governance role-change simulation.
- PH1.WRITE explains the safe next step.

## 17. What Must Not Happen

No raw link grants authority.

No link generation without valid simulation/capability.

No sending without confirmation.

No delivery from PH1.LINK.

No role/access grant by PH1.LINK.

No Desktop/iPhone access decision.

No Adapter access decision.

No unsupported link type execution.

No guessing recipient/role/channel for high-risk links.

No silent correction of CEO/CFO-style role mistakes.

No phrase-patch intent handling.

No implementation from this document alone.

No OpenAI/GPT-5.5 proposal may execute PH1.LINK directly.

No PH1.N candidate may bypass PH1.X.

No capability request may be treated as approval to run unsupported work.

## 18. Recommended Future Build Slices

1. PH1.LINK Link Journey Intelligence Activation Pack
2. Link Intent Understanding / PH1.D + PH1.N Proposal Shell
3. Link Permission Matrix + Access/Governance Map
4. Simulation Discovery / Capability Finder
5. Confirmation Gate Before Link Generation
6. PH1.LINK Generate Link Integration
7. Broadcast/Delivery SMS Handoff
8. Receiver Activation / ONB Handoff Proof
9. Link Status Assistant
10. Wrong Link / Wrong Role Correction Flow
11. Missing Simulation Capability Request Flow
12. Capability Review + Requester Notification Flow
13. JD Live Link Journey Acceptance Pack

Build sequencing law:

- Start with repo-truth activation.
- Map current contracts before creating new contracts.
- Prove provider-off and fake-provider behavior for PH1.D assistance.
- Prove PH1.N proposals cannot execute.
- Prove PH1.X remains final route validator.
- Prove PH1.LINK does not send.
- Prove PH1.BCAST / PH1.DELIVERY own SMS delivery.
- Prove PH1.ONB owns onboarding after activation.
- Prove missing simulation creates a request or fails with `CAPABILITY_REQUEST_OWNER_NEEDED`.

## 19. Grand Architecture Reconciliation Note

This document must later be reconciled into:

- PH1.LINK extraction,
- Global Request Decision Lattice,
- PH1.D Proposal Gateway,
- PH1.N Meaning Unravelling,
- PH1.X routing,
- PH1.WRITE presentation,
- Access/Governance,
- SimulationExecutor,
- Broadcast/Delivery,
- Onboarding,
- Voice Identity if voice enrollment links are added,
- Desktop/iPhone render-only proof,
- Adapter transport-only proof.

Future reconciliation must ensure:

- current PH1.LINK repo truth stays the factual base,
- this journey layer does not become a duplicate link engine,
- Simulation Finder uses canonical simulation/capability registry truth,
- missing-simulation tickets route to a canonical capability request owner if one exists,
- delivery remains SMS-first by default but delivery-owner controlled,
- wrong-link correction routes by current lifecycle state,
- high-access role mistakes require serious confirmation and correct owner routing.

## 20. Final Architecture Sentence

"PH1.LINK creates and governs link lifecycle; Selene Link Journey Intelligence understands what link the human wants, confirms the resolved action, finds the lawful simulation, hands delivery to Broadcast/Delivery, tracks activation, and hands onboarding/access correction to the correct canonical owners."
