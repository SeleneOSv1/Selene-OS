# Selene Governance Source Pack Onboarding Master Design

```text
DOCUMENT TYPE:
MASTER DESIGN / COMPANY ONBOARDING / GOVERNANCE SOURCE PACK

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION
PENDING_CODEX_REPO_MAPPING

PURPOSE:
Define how Selene collects, reads, source-references, reviews, and routes company governance evidence such as constitution/articles, shareholder agreements, directors, board structure, shareholders, voting rights, approval matrices, bank signing rules, payment/budget/dividend rules, delegated authority, ownership structure, and legal entity structure.
```

## 0. Authority And Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

This document is part of the Company Onboarding Master Design Set. It defines future governance source onboarding only. It does not implement PH1.ONB.GOVSOURCE, PH1.BOARD, PH1.SHAREHOLDER, Access, Finance, Banking, Payroll, HR, Equity, Debt, Audit, legal review, packets, schemas, APIs, migrations, tests, or activation code.

Current repo truth does not prove complete runtime Governance Source Pack Onboarding or governance-rule activation. This document is future architecture pending Grand Architecture Reconciliation and Codex repo mapping.

## 1. Purpose

This document defines how Selene collects and structures company governance sources.

Governance onboarding matters when a company has:

```text
directors
shareholders
board rules
approval limits
bank signing rules
delegated authority
executive roles
share classes
voting rights
dividend rules
debt/loan approvals
budget approvals
```

Governance documents feed:

```text
PH1.BOARD
PH1.SHAREHOLDER
Access/Governance
Finance/Budget
Banking/Payments
AP/AR approvals
Payroll authority
HR authority
Audit
Equity/Dividends
Debt/Loans
```

## 2. Executive Target

Selene must read governance documents, propose a structured authority setup, and ask the right humans, legal reviewer, board authority, or shareholder authority to confirm.

Selene must not invent legal governance.

Selene must not turn uploaded documents into active law without review.

Selene must preserve source references.

## 3. Master Law

```text
Selene reads governance documents.
Selene extracts possible rules.
Selene preserves source references.
Governance documents are source evidence, not automatic legal truth.
Human/legal/board/shareholder authority confirms.
Access/Governance activates permissions and authority.
Board/shareholder engines govern votes and resolutions.
Source engines still validate execution.
No governance rule becomes active without review and audit.
```

## 4. Owner Split

| Area | Owner |
| --- | --- |
| Governance intake | PH1.ONB.GOVSOURCE / Company Onboarding |
| Legal interpretation approval | Legal / human authority |
| Board decisions | PH1.BOARD |
| Shareholder rights/votes | PH1.SHAREHOLDER |
| Permission grants | Access/Governance |
| Budget/spend authority | Finance/Budget + Access |
| Payment authority | Banking/Payments + Access |
| AP/AR approval routing | AP / AR + Access |
| Payroll authority | Payroll + Access |
| HR authority | HR + Access |
| Equity/dividends | Shareholder / Equity / Accounting |
| Debt/loans | Finance / Treasury / Legal / Accounting |
| Audit | Audit |

## 5. Governance Sources Selene Collects

```text
company constitution / articles
shareholder agreement
directors list
board structure
shareholders
cap table
share classes
voting rights
delegation authority
approval matrix
bank signing rules
payment approval rules
budget approval rules
dividend policy
board meeting rules
quorum
conflict rules
delegated authority
legal entity structure
ownership structure
executive roles
existing resolutions
loan/security documents
company policies
```

Selene says:

```text
Upload your company constitution, shareholder agreement, or board rules if you have them. I'll read them and prepare the approval and authority structure for your review.
```

## 6. Governance Extraction

Selene extracts candidate rules:

```text
board quorum
shareholder quorum
ordinary resolution threshold
special resolution threshold
director appointment/removal rules
reserved matters
share transfer restrictions
payment approval limits
bank signatory rules
budget approval levels
dividend approval rules
conflict rules
executive authority
delegated authority
share class voting rights
director roles
officer roles
committee authority
debt/loan approval rules
related-party approval rules
```

Every extracted rule has:

```text
source document
page/clause/reference
confidence score
status: Extracted / NeedsReview / Approved / Active / Rejected / Superseded
review owner
audit reference
```

## 7. Governance Review

If confidence is high:

```text
I found a rule that payments above $50,000 need two directors. Please confirm if this should become active.
```

If conflict exists:

```text
The constitution and shareholder agreement appear to conflict on share transfer approval. I will mark this for legal review before activating any rule.
```

If source evidence is weak:

```text
I found a possible approval rule, but the source is unclear. I will keep it as NeedsReview until an authorized reviewer confirms it.
```

## 8. Governance Feeding Access

Approved rules feed:

```text
role templates
approval chains
board voting thresholds
shareholder vote thresholds
bank/payment authority
purchase authority
refund authority
pricing override authority
budget authority
HR/payroll authority
field-level access
legal entity scope
country/region scope
temporary authority rules
fallback approver rules
```

Access grants.

Onboarding does not grant.

PH1.D/GPT-5.5 does not grant.

## 9. Governance Feeding Finance / Banking / Payroll / HR

Governance source pack must connect to:

```text
Finance/Budget approval thresholds
cash reserve override authority
budget approval rules
bank signing rules
payment approval rules
refund approval rules
AP supplier payment approvals
AR write-off/refund approvals
payroll approval authority
HR hire/termination authority
debt/loan approval authority
dividend approval authority
board pack approval rules
```

Finance, Banking, Payroll, HR, Equity, Debt, AP, and AR retain their source truth.

Governance evidence informs authority, but does not execute the downstream action by itself.

## 10. Board And Shareholder Boundaries

PH1.BOARD owns:

```text
board structure
board meetings
quorum
resolutions
director voting
reserved matters
board approvals
board minutes evidence
```

PH1.SHAREHOLDER owns:

```text
shareholders
share classes
voting rights
class rights
shareholder resolutions
shareholder approvals
ownership changes
shareholder notices
```

Onboarding may ingest and propose.

Board/shareholder owners and authorized humans confirm.

## 11. Governance Example

User uploads constitution and shareholder agreement.

Selene says:

```text
I found three governance rules that affect setup: board quorum is two directors, ordinary shareholder resolutions require more than 50%, and issuing new shares requires investor consent. I'll prepare these for review before activation.
```

If confirmed:

```text
PH1.BOARD gets quorum rule
PH1.SHAREHOLDER gets shareholder vote rule
Access gets authority mappings
Finance gets reserved matter warning
Audit records all proof
```

## 12. Bank Signing And Payment Approval Example

Selene extracts:

```text
bank payments above AUD 25,000 require two directors
new supplier bank accounts require CFO approval
budget increases above AUD 100,000 require board approval
```

Selene says:

```text
These rules affect payment authority and supplier bank changes. I will keep them in review until an authorized governance reviewer confirms them.
```

After confirmation:

```text
Access/Governance receives authority scope
Banking/Payments receives payment approval requirements
AP receives supplier bank-change payment hold rule
Finance/Budget receives budget approval threshold
Audit records source references
```

## 13. Required Logical Packets

Future mapping targets:

```text
GovernanceSourceIntakePacket
GovernanceDocumentEvidencePacket
GovernanceRuleExtractionPacket
GovernanceRuleReviewPacket
BoardAuthorityProposalPacket
ShareholderRightsProposalPacket
DelegationAuthorityPacket
ApprovalMatrixProposalPacket
BankSigningRulePacket
PaymentApprovalRulePacket
BudgetApprovalRulePacket
DividendPolicyRulePacket
GovernanceConflictPacket
GovernanceActivationPacket
GovernanceSourceReferencePacket
```

These are logical design packets only. They are not claimed runtime packets.

## 14. What Must Not Happen

```text
no invented board/shareholder rules
no legal governance activated without review
no bypassed board/shareholder approval
no ignored source references
no board approval treated as execution authority
no GPT-5.5 granted authority
no Onboarding granted access directly
no bank/payment approval rules invented
no dividend rules invented
no delegated authority invented
no conflicting governance sources silently resolved
no implementation claims
```

## 15. Future Simulation Targets

```text
SIM_COMPANY_GOV_001_upload_constitution
SIM_COMPANY_GOV_002_extract_board_quorum
SIM_COMPANY_GOV_003_extract_shareholder_voting_rights
SIM_COMPANY_GOV_004_detect_governance_conflict
SIM_COMPANY_GOV_005_approve_governance_rule
SIM_COMPANY_GOV_006_reject_governance_rule
SIM_COMPANY_GOV_007_activate_approval_matrix
SIM_COMPANY_GOV_008_route_board_approval_into_access
SIM_COMPANY_GOV_009_route_shareholder_rule_into_shareholder_engine
SIM_COMPANY_GOV_010_block_payment_setup_pending_governance_approval
```

## 16. Final Architecture Sentence

Selene Governance Source Pack Onboarding turns company governance documents into reviewable, source-referenced authority proposals that feed Board, Shareholder, Access, Finance, Banking, Payroll, HR, Equity, Debt, and Audit without inventing legal truth or bypassing required human, legal, board, or shareholder confirmation.
