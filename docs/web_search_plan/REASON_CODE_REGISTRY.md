# Reason Code Registry (Run 1 Foundation Lock)

`REASON_CODES.json` is the only valid reason-code authority for this lane.

## Hard Rules
- No ad-hoc reason codes.
- Unknown reason code reference fails closed.
- User message templates must not leak secrets or internal infrastructure details.

## Required Foundation Codes
- Input/session: `invalid_session`, `identity_not_verified`, `input_unparseable`
- Policy/access/quota: `access_denied`, `policy_violation`, `quota_exceeded`, `budget_exhausted`
- Retrieval: `provider_unconfigured`, `provider_upstream_failed`, `proxy_misconfigured`, `timeout_exceeded`, `empty_results`
- Evidence/grounding: `insufficient_evidence`, `conflicting_evidence_detected`, `citation_mismatch`, `unsupported_claim`, `evidence_truncated`, `hash_collision_detected`
- Freshness: `stale_data`, `freshness_policy_unmet`
- Compliance: `insufficient_regulatory_evidence`, `jurisdiction_mismatch`, `compliance_confidence_low`
