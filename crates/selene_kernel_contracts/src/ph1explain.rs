#![forbid(unsafe_code)]

use crate::ph1d::PolicyContextRef;
use crate::ph1x::Ph1xDirective;
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1EXPLAIN_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExplainRequestType {
    Why,
    WhyNot,
    HowKnow,
    WhatNext,
    WhatHappened,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainRequest {
    pub schema_version: SchemaVersion,
    pub request_type: ExplainRequestType,
    /// Optional raw user utterance (e.g., "why?", "what happened?"). Kept short.
    pub utterance: Option<String>,
}

impl ExplainRequest {
    pub fn v1(
        request_type: ExplainRequestType,
        utterance: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1EXPLAIN_CONTRACT_VERSION,
            request_type,
            utterance,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for ExplainRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EXPLAIN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "explain_request.schema_version",
                reason: "must match PH1EXPLAIN_CONTRACT_VERSION",
            });
        }
        if let Some(u) = &self.utterance {
            if u.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "explain_request.utterance",
                    reason: "must not be empty when provided",
                });
            }
            if u.len() > 256 {
                return Err(ContractViolation::InvalidValue {
                    field: "explain_request.utterance",
                    reason: "must be <= 256 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventContextRef {
    pub schema_version: SchemaVersion,
    pub primary_reason_code: ReasonCodeId,
    pub related_reason_codes: Vec<ReasonCodeId>,
    /// Optional recent directive (e.g., Clarify/Confirm) so EXPLAIN can be specific without guessing.
    pub conversation_directive: Option<Ph1xDirective>,
    /// Optional short user phrase that triggered the event (e.g., "wait"). Used for barge-in explanations only.
    pub verbatim_trigger: Option<String>,
}

impl EventContextRef {
    pub fn v1(
        primary_reason_code: ReasonCodeId,
        related_reason_codes: Vec<ReasonCodeId>,
        conversation_directive: Option<Ph1xDirective>,
        verbatim_trigger: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let c = Self {
            schema_version: PH1EXPLAIN_CONTRACT_VERSION,
            primary_reason_code,
            related_reason_codes,
            conversation_directive,
            verbatim_trigger,
        };
        c.validate()?;
        Ok(c)
    }
}

impl Validate for EventContextRef {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EXPLAIN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "event_context_ref.schema_version",
                reason: "must match PH1EXPLAIN_CONTRACT_VERSION",
            });
        }
        if self.related_reason_codes.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "event_context_ref.related_reason_codes",
                reason: "must be <= 16 entries",
            });
        }
        if let Some(t) = &self.verbatim_trigger {
            if t.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "event_context_ref.verbatim_trigger",
                    reason: "must not be empty when provided",
                });
            }
            if t.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "event_context_ref.verbatim_trigger",
                    reason: "must be <= 128 chars",
                });
            }
            if t.chars().any(|c| c.is_control()) {
                return Err(ContractViolation::InvalidValue {
                    field: "event_context_ref.verbatim_trigger",
                    reason: "must not contain control characters",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryCandidateRef {
    pub schema_version: SchemaVersion,
    pub evidence_quote: String,
    pub provenance: Option<String>,
    pub is_sensitive: bool,
}

impl MemoryCandidateRef {
    pub fn v1(
        evidence_quote: String,
        provenance: Option<String>,
        is_sensitive: bool,
    ) -> Result<Self, ContractViolation> {
        let m = Self {
            schema_version: PH1EXPLAIN_CONTRACT_VERSION,
            evidence_quote,
            provenance,
            is_sensitive,
        };
        m.validate()?;
        Ok(m)
    }
}

impl Validate for MemoryCandidateRef {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EXPLAIN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "memory_candidate_ref.schema_version",
                reason: "must match PH1EXPLAIN_CONTRACT_VERSION",
            });
        }
        if self.evidence_quote.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_candidate_ref.evidence_quote",
                reason: "must not be empty",
            });
        }
        if self.evidence_quote.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_candidate_ref.evidence_quote",
                reason: "must be <= 256 chars",
            });
        }
        if let Some(p) = &self.provenance {
            if p.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "memory_candidate_ref.provenance",
                    reason: "must not be empty when provided",
                });
            }
            if p.len() > 256 {
                return Err(ContractViolation::InvalidValue {
                    field: "memory_candidate_ref.provenance",
                    reason: "must be <= 256 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1ExplainInput {
    pub schema_version: SchemaVersion,
    pub explain_request: ExplainRequest,
    pub event_context_ref: EventContextRef,
    pub memory_candidate_ref: Option<MemoryCandidateRef>,
    pub policy_context_ref: PolicyContextRef,
}

impl Ph1ExplainInput {
    pub fn v1(
        explain_request: ExplainRequest,
        event_context_ref: EventContextRef,
        memory_candidate_ref: Option<MemoryCandidateRef>,
        policy_context_ref: PolicyContextRef,
    ) -> Result<Self, ContractViolation> {
        let i = Self {
            schema_version: PH1EXPLAIN_CONTRACT_VERSION,
            explain_request,
            event_context_ref,
            memory_candidate_ref,
            policy_context_ref,
        };
        i.validate()?;
        Ok(i)
    }
}

impl Validate for Ph1ExplainInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EXPLAIN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1_explain_input.schema_version",
                reason: "must match PH1EXPLAIN_CONTRACT_VERSION",
            });
        }
        self.explain_request.validate()?;
        self.event_context_ref.validate()?;
        if let Some(m) = &self.memory_candidate_ref {
            m.validate()?;
        }
        self.policy_context_ref.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExplanationType {
    Why,
    WhyNot,
    HowKnow,
    WhatNext,
    WhatHappened,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplanationOk {
    pub schema_version: SchemaVersion,
    pub explanation_text: String,
    pub explanation_type: ExplanationType,
    pub evidence_quote: Option<String>,
}

impl ExplanationOk {
    pub fn v1(
        explanation_text: String,
        explanation_type: ExplanationType,
        evidence_quote: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1EXPLAIN_CONTRACT_VERSION,
            explanation_text,
            explanation_type,
            evidence_quote,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for ExplanationOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EXPLAIN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "explanation_ok.schema_version",
                reason: "must match PH1EXPLAIN_CONTRACT_VERSION",
            });
        }
        if self.explanation_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "explanation_ok.explanation_text",
                reason: "must not be empty",
            });
        }
        if self.explanation_text.len() > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "explanation_ok.explanation_text",
                reason: "must be <= 512 chars",
            });
        }
        // Enforce "1-2 sentences max" with a deterministic, conservative heuristic.
        if sentence_terminator_count(&self.explanation_text) > 2 {
            return Err(ContractViolation::InvalidValue {
                field: "explanation_ok.explanation_text",
                reason: "must be <= 2 sentences",
            });
        }
        if let Some(q) = &self.evidence_quote {
            if q.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "explanation_ok.evidence_quote",
                    reason: "must not be empty when provided",
                });
            }
            if q.len() > 256 {
                return Err(ContractViolation::InvalidValue {
                    field: "explanation_ok.evidence_quote",
                    reason: "must be <= 256 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplanationRefuse {
    pub schema_version: SchemaVersion,
    pub reason_code: ReasonCodeId,
    pub refusal_text: String,
}

impl ExplanationRefuse {
    pub fn v1(reason_code: ReasonCodeId, refusal_text: String) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1EXPLAIN_CONTRACT_VERSION,
            reason_code,
            refusal_text,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for ExplanationRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EXPLAIN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "explanation_refuse.schema_version",
                reason: "must match PH1EXPLAIN_CONTRACT_VERSION",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "explanation_refuse.reason_code",
                reason: "must be > 0",
            });
        }
        if self.refusal_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "explanation_refuse.refusal_text",
                reason: "must not be empty",
            });
        }
        if self.refusal_text.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "explanation_refuse.refusal_text",
                reason: "must be <= 256 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ph1ExplainResponse {
    Explanation(ExplanationOk),
    ExplanationRefuse(ExplanationRefuse),
}

fn sentence_terminator_count(s: &str) -> usize {
    s.chars().filter(|c| matches!(c, '.' | '!' | '?')).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ph1d::{PolicyContextRef, SafetyTier};

    #[test]
    fn explanation_text_must_be_one_or_two_sentences() {
        let ok = ExplanationOk::v1("One. Two. Three.".to_string(), ExplanationType::Why, None);
        assert!(ok.is_err());
    }

    #[test]
    fn memory_candidate_requires_evidence_quote() {
        let m = MemoryCandidateRef::v1("  ".to_string(), None, false);
        assert!(m.is_err());
    }

    #[test]
    fn input_validates_nested_contracts() {
        let req = ExplainRequest::v1(ExplainRequestType::Why, Some("why?".to_string())).unwrap();
        let ctx = EventContextRef::v1(ReasonCodeId(1), vec![], None, None).unwrap();
        let policy = PolicyContextRef::v1(false, false, SafetyTier::Standard);
        let input = Ph1ExplainInput::v1(req, ctx, None, policy).unwrap();
        assert_eq!(input.schema_version, PH1EXPLAIN_CONTRACT_VERSION);
    }
}
