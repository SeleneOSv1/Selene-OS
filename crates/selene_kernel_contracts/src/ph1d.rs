#![forbid(unsafe_code)]

use crate::ph1c::{SessionStateRef, TranscriptOk};
use crate::ph1e::ToolCatalogRef;
use crate::ph1n::{EvidenceSpan, FieldKey, FieldValue, IntentType, Ph1nResponse, TranscriptHash};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1D_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);

impl Validate for RequestId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "request_id",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SchemaHash(pub u64);

impl Validate for SchemaHash {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "schema_hash",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SafetyTier {
    Standard,
    Strict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PolicyContextRef {
    pub schema_version: SchemaVersion,
    pub privacy_mode: bool,
    pub do_not_disturb: bool,
    pub safety_tier: SafetyTier,
}

impl PolicyContextRef {
    pub fn v1(privacy_mode: bool, do_not_disturb: bool, safety_tier: SafetyTier) -> Self {
        Self {
            schema_version: PH1D_CONTRACT_VERSION,
            privacy_mode,
            do_not_disturb,
            safety_tier,
        }
    }
}

impl Validate for PolicyContextRef {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1D_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "policy_context_ref.schema_version",
                reason: "must match PH1D_CONTRACT_VERSION",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1dRequest {
    pub schema_version: SchemaVersion,
    // Deterministic request envelope (audit-grade).
    pub request_id: RequestId,
    pub prompt_template_version: SchemaVersion,
    pub output_schema_hash: SchemaHash,
    pub tool_catalog_hash: SchemaHash,
    pub policy_context_hash: SchemaHash,
    pub transcript_hash: TranscriptHash,
    pub transcript_ok: TranscriptOk,
    pub nlp_output: Ph1nResponse,
    pub session_state_ref: SessionStateRef,
    pub policy_context_ref: PolicyContextRef,
    pub tool_catalog_ref: ToolCatalogRef,
}

impl Ph1dRequest {
    pub fn v1(
        transcript_ok: TranscriptOk,
        nlp_output: Ph1nResponse,
        session_state_ref: SessionStateRef,
        policy_context_ref: PolicyContextRef,
        tool_catalog_ref: ToolCatalogRef,
    ) -> Result<Self, ContractViolation> {
        // Stable, deterministic hashes for audit/replay (not crypto).
        let transcript_hash = TranscriptHash(nonzero_u64(fnv1a64(
            transcript_ok.transcript_text.as_bytes(),
        )));
        let tool_catalog_hash =
            SchemaHash(nonzero_u64(fnv1a64(&tool_catalog_bytes(&tool_catalog_ref))));
        let policy_context_hash = SchemaHash(nonzero_u64(fnv1a64(&policy_context_bytes(
            &policy_context_ref,
        ))));
        let output_schema_hash = SchemaHash(nonzero_u64(fnv1a64(b"ph1d_output_schema_v1")));
        let request_id = RequestId(nonzero_u64(fnv1a64(&request_id_bytes(
            transcript_hash,
            tool_catalog_hash,
            policy_context_hash,
            output_schema_hash,
        ))));
        let r = Self {
            schema_version: PH1D_CONTRACT_VERSION,
            request_id,
            prompt_template_version: SchemaVersion(1),
            output_schema_hash,
            tool_catalog_hash,
            policy_context_hash,
            transcript_hash,
            transcript_ok,
            nlp_output,
            session_state_ref,
            policy_context_ref,
            tool_catalog_ref,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1dRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1D_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_request.schema_version",
                reason: "must match PH1D_CONTRACT_VERSION",
            });
        }
        if self.prompt_template_version != SchemaVersion(1) {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_request.prompt_template_version",
                reason: "unsupported prompt_template_version",
            });
        }
        self.request_id.validate()?;
        self.output_schema_hash.validate()?;
        self.tool_catalog_hash.validate()?;
        self.policy_context_hash.validate()?;
        self.transcript_ok.validate()?;
        self.policy_context_ref.validate()?;
        self.tool_catalog_ref.validate()?;

        // Envelope integrity: recompute and compare (fail closed on mismatch).
        let expected_transcript_hash = TranscriptHash(nonzero_u64(fnv1a64(
            self.transcript_ok.transcript_text.as_bytes(),
        )));
        if self.transcript_hash != expected_transcript_hash {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_request.transcript_hash",
                reason: "must match stable hash of transcript_ok.transcript_text",
            });
        }

        let expected_tool_catalog_hash = SchemaHash(nonzero_u64(fnv1a64(&tool_catalog_bytes(
            &self.tool_catalog_ref,
        ))));
        if self.tool_catalog_hash != expected_tool_catalog_hash {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_request.tool_catalog_hash",
                reason: "must match stable hash of tool_catalog_ref",
            });
        }

        let expected_policy_context_hash = SchemaHash(nonzero_u64(fnv1a64(&policy_context_bytes(
            &self.policy_context_ref,
        ))));
        if self.policy_context_hash != expected_policy_context_hash {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_request.policy_context_hash",
                reason: "must match stable hash of policy_context_ref",
            });
        }

        let expected_output_schema_hash =
            SchemaHash(nonzero_u64(fnv1a64(b"ph1d_output_schema_v1")));
        if self.output_schema_hash != expected_output_schema_hash {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_request.output_schema_hash",
                reason: "must match expected schema hash for PH1.D output schema v1",
            });
        }

        let expected_request_id = RequestId(nonzero_u64(fnv1a64(&request_id_bytes(
            expected_transcript_hash,
            expected_tool_catalog_hash,
            expected_policy_context_hash,
            expected_output_schema_hash,
        ))));
        if self.request_id != expected_request_id {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_request.request_id",
                reason: "must match deterministic request_id derivation",
            });
        }

        Ok(())
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut h = OFFSET;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(PRIME);
    }
    h
}

fn nonzero_u64(h: u64) -> u64 {
    if h == 0 {
        1
    } else {
        h
    }
}

fn tool_catalog_bytes(c: &ToolCatalogRef) -> Vec<u8> {
    // Stable order for hashing.
    let mut names = c.tools.iter().map(|t| t.as_str()).collect::<Vec<_>>();
    names.sort();
    let mut out: Vec<u8> = Vec::new();
    for n in names {
        out.extend_from_slice(n.as_bytes());
        out.push(0);
    }
    out
}

fn policy_context_bytes(p: &PolicyContextRef) -> Vec<u8> {
    let mut out = Vec::with_capacity(3);
    out.push(p.privacy_mode as u8);
    out.push(p.do_not_disturb as u8);
    out.push(match p.safety_tier {
        SafetyTier::Standard => 0,
        SafetyTier::Strict => 1,
    });
    out
}

fn request_id_bytes(
    transcript_hash: TranscriptHash,
    tool_catalog_hash: SchemaHash,
    policy_context_hash: SchemaHash,
    output_schema_hash: SchemaHash,
) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(32);
    out.extend_from_slice(&transcript_hash.0.to_le_bytes());
    out.extend_from_slice(&tool_catalog_hash.0.to_le_bytes());
    out.extend_from_slice(&policy_context_hash.0.to_le_bytes());
    out.extend_from_slice(&output_schema_hash.0.to_le_bytes());
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1dChat {
    pub schema_version: SchemaVersion,
    pub response_text: String,
    pub reason_code: ReasonCodeId,
}

impl Ph1dChat {
    pub fn v1(response_text: String, reason_code: ReasonCodeId) -> Result<Self, ContractViolation> {
        let c = Self {
            schema_version: PH1D_CONTRACT_VERSION,
            response_text,
            reason_code,
        };
        c.validate()?;
        Ok(c)
    }
}

impl Validate for Ph1dChat {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.response_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_chat.response_text",
                reason: "must not be empty",
            });
        }
        if self.response_text.len() > 8_192 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_chat.response_text",
                reason: "must be <= 8192 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1dClarify {
    pub schema_version: SchemaVersion,
    pub question: String,
    pub what_is_missing: Vec<FieldKey>,
    pub accepted_answer_formats: Vec<String>,
    pub reason_code: ReasonCodeId,
}

impl Ph1dClarify {
    pub fn v1(
        question: String,
        what_is_missing: Vec<FieldKey>,
        accepted_answer_formats: Vec<String>,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let c = Self {
            schema_version: PH1D_CONTRACT_VERSION,
            question,
            what_is_missing,
            accepted_answer_formats,
            reason_code,
        };
        c.validate()?;
        Ok(c)
    }
}

impl Validate for Ph1dClarify {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.question.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_clarify.question",
                reason: "must not be empty",
            });
        }
        if self.what_is_missing.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_clarify.what_is_missing",
                reason: "must not be empty",
            });
        }
        // Hard rule: one question => one missing field.
        if self.what_is_missing.len() != 1 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_clarify.what_is_missing",
                reason: "must contain exactly 1 entry",
            });
        }
        if !(2..=3).contains(&self.accepted_answer_formats.len()) {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_clarify.accepted_answer_formats",
                reason: "must contain 2–3 entries",
            });
        }
        for f in &self.accepted_answer_formats {
            if f.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_clarify.accepted_answer_formats[]",
                    reason: "must not contain empty strings",
                });
            }
            if f.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_clarify.accepted_answer_formats[]",
                    reason: "must be <= 128 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1dAnalysis {
    pub schema_version: SchemaVersion,
    /// Internal-only. Must never be spoken to the user.
    pub short_analysis: String,
    pub reason_code: ReasonCodeId,
}

impl Ph1dAnalysis {
    pub fn v1(
        short_analysis: String,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let a = Self {
            schema_version: PH1D_CONTRACT_VERSION,
            short_analysis,
            reason_code,
        };
        a.validate()?;
        Ok(a)
    }
}

impl Validate for Ph1dAnalysis {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.short_analysis.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_analysis.short_analysis",
                reason: "must not be empty",
            });
        }
        if self.short_analysis.len() > 2_048 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_analysis.short_analysis",
                reason: "must be <= 2048 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1dFieldRefinement {
    pub field: FieldKey,
    pub value: FieldValue,
    pub evidence_span: EvidenceSpan,
}

impl Validate for Ph1dFieldRefinement {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.evidence_span.validate()?;
        if self.evidence_span.field != self.field {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_field_refinement.evidence_span.field",
                reason: "must match field",
            });
        }
        if self.evidence_span.verbatim_excerpt != self.value.original_span {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_field_refinement.evidence_span.verbatim_excerpt",
                reason: "must match value.original_span exactly",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1dIntent {
    pub schema_version: SchemaVersion,
    pub refined_intent_type: IntentType,
    pub field_refinements: Vec<Ph1dFieldRefinement>,
    pub missing_fields: Vec<FieldKey>,
    pub reason_code: ReasonCodeId,
}

impl Ph1dIntent {
    pub fn v1(
        refined_intent_type: IntentType,
        field_refinements: Vec<Ph1dFieldRefinement>,
        missing_fields: Vec<FieldKey>,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let i = Self {
            schema_version: PH1D_CONTRACT_VERSION,
            refined_intent_type,
            field_refinements,
            missing_fields,
            reason_code,
        };
        i.validate()?;
        Ok(i)
    }
}

impl Validate for Ph1dIntent {
    fn validate(&self) -> Result<(), ContractViolation> {
        for r in &self.field_refinements {
            r.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ph1dOk {
    Chat(Ph1dChat),
    Intent(Ph1dIntent),
    Clarify(Ph1dClarify),
    Analysis(Ph1dAnalysis),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ph1dFailureKind {
    InvalidSchema,
    ForbiddenOutput,
    SafetyBlock,
    Timeout,
    BudgetExceeded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1dFail {
    pub schema_version: SchemaVersion,
    pub reason_code: ReasonCodeId,
    pub kind: Ph1dFailureKind,
}

impl Ph1dFail {
    pub fn v1(reason_code: ReasonCodeId, kind: Ph1dFailureKind) -> Self {
        Self {
            schema_version: PH1D_CONTRACT_VERSION,
            reason_code,
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ph1dResponse {
    Ok(Ph1dOk),
    Fail(Ph1dFail),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ph1c::{ConfidenceBucket, LanguageTag};
    use crate::ph1e::ToolName;
    use crate::ph1n::{Chat, Ph1nResponse};
    use crate::ph1w::SessionState;

    #[test]
    fn ph1d_request_envelope_integrity_is_enforced() {
        let ok = TranscriptOk::v1(
            "hello".to_string(),
            LanguageTag::new("en").unwrap(),
            ConfidenceBucket::High,
        )
        .unwrap();

        let mut r = Ph1dRequest::v1(
            ok,
            Ph1nResponse::Chat(Chat::v1("hi".to_string(), ReasonCodeId(1)).unwrap()),
            SessionStateRef::v1(SessionState::Active, false),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
            ToolCatalogRef::v1(vec![ToolName::Time, ToolName::Weather]).unwrap(),
        )
        .unwrap();

        // Tamper with a derived field; validation must fail closed.
        r.transcript_hash = TranscriptHash(r.transcript_hash.0.wrapping_add(1));
        assert!(r.validate().is_err());
    }
}
