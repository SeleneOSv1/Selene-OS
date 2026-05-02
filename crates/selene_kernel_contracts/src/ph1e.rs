#![forbid(unsafe_code)]

use crate::ph1d::{PolicyContextRef, SafetyTier, PH1D_CONTRACT_VERSION};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1E_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ToolRequestId(pub u64);

impl Validate for ToolRequestId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "tool_request_id",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ToolQueryHash(pub u64);

impl Validate for ToolQueryHash {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "tool_query_hash",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    // FNV-1a 64-bit (stable across platforms, deterministic).
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut h = OFFSET;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(PRIME);
    }
    h
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ToolName {
    Time,
    Weather,
    WebSearch,
    News,
    UrlFetchAndCite,
    DocumentUnderstand,
    PhotoUnderstand,
    DataAnalysis,
    DeepResearch,
    RecordMode,
    ConnectorQuery,
    Other(String),
}

impl ToolName {
    pub fn other(name: impl Into<String>) -> Result<Self, ContractViolation> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "tool_name.other",
                reason: "must not be empty",
            });
        }
        if name.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "tool_name.other",
                reason: "must be <= 64 chars",
            });
        }
        Ok(ToolName::Other(name))
    }

    pub fn as_str(&self) -> &str {
        match self {
            ToolName::Time => "time",
            ToolName::Weather => "weather",
            ToolName::WebSearch => "web_search",
            ToolName::News => "news",
            ToolName::UrlFetchAndCite => "url_fetch_and_cite",
            ToolName::DocumentUnderstand => "document_understand",
            ToolName::PhotoUnderstand => "photo_understand",
            ToolName::DataAnalysis => "data_analysis",
            ToolName::DeepResearch => "deep_research",
            ToolName::RecordMode => "record_mode",
            ToolName::ConnectorQuery => "connector_query",
            ToolName::Other(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCatalogRef {
    pub schema_version: SchemaVersion,
    pub tools: Vec<ToolName>,
}

impl ToolCatalogRef {
    pub fn v1(tools: Vec<ToolName>) -> Result<Self, ContractViolation> {
        let c = Self {
            schema_version: PH1E_CONTRACT_VERSION,
            tools,
        };
        c.validate()?;
        Ok(c)
    }
}

impl Validate for ToolCatalogRef {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.tools.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "tool_catalog_ref.tools",
                reason: "must not be empty",
            });
        }
        for t in &self.tools {
            if matches!(t, ToolName::Other(_)) {
                return Err(ContractViolation::InvalidValue {
                    field: "tool_catalog_ref.tools",
                    reason: "must not contain ToolName::Other(...)",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ToolRequestOrigin {
    Ph1X,
    Other(String),
}

impl ToolRequestOrigin {
    pub fn other(name: impl Into<String>) -> Result<Self, ContractViolation> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "tool_request_origin.other",
                reason: "must not be empty",
            });
        }
        Ok(Self::Other(name))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StrictBudget {
    pub timeout_ms: u32,
    pub max_results: u8,
}

impl StrictBudget {
    pub fn new(timeout_ms: u32, max_results: u8) -> Result<Self, ContractViolation> {
        let b = Self {
            timeout_ms,
            max_results,
        };
        b.validate()?;
        Ok(b)
    }
}

impl Validate for StrictBudget {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.timeout_ms == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "strict_budget.timeout_ms",
                reason: "must be > 0",
            });
        }
        if self.max_results == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "strict_budget.max_results",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolRequest {
    pub schema_version: SchemaVersion,
    // Deterministic request envelope (audit-grade).
    pub request_id: ToolRequestId,
    pub query_hash: ToolQueryHash,
    pub origin: ToolRequestOrigin,
    pub tool_name: ToolName,
    pub query: String,
    pub locale: Option<String>,
    pub strict_budget: StrictBudget,
    pub policy_context_ref: PolicyContextRef,
}

impl ToolRequest {
    pub fn v1(
        origin: ToolRequestOrigin,
        tool_name: ToolName,
        query: String,
        locale: Option<String>,
        strict_budget: StrictBudget,
        policy_context_ref: PolicyContextRef,
    ) -> Result<Self, ContractViolation> {
        let query_hash = {
            let mut h = fnv1a64(query.as_bytes());
            if h == 0 {
                h = 1;
            }
            ToolQueryHash(h)
        };
        let request_id = {
            let mut b: Vec<u8> = Vec::new();
            b.extend_from_slice(tool_name.as_str().as_bytes());
            b.push(0);
            b.extend_from_slice(&query_hash.0.to_le_bytes());
            b.push(0);
            if let Some(loc) = &locale {
                b.extend_from_slice(loc.as_bytes());
            }
            b.push(0);
            b.extend_from_slice(&strict_budget.timeout_ms.to_le_bytes());
            b.push(strict_budget.max_results);
            b.push(policy_context_ref.privacy_mode as u8);
            b.push(policy_context_ref.do_not_disturb as u8);
            b.push(match policy_context_ref.safety_tier {
                SafetyTier::Standard => 0,
                SafetyTier::Strict => 1,
            });
            let mut h = fnv1a64(&b);
            if h == 0 {
                h = 1;
            }
            ToolRequestId(h)
        };
        let r = Self {
            schema_version: PH1E_CONTRACT_VERSION,
            request_id,
            query_hash,
            origin,
            tool_name,
            query,
            locale,
            strict_budget,
            policy_context_ref,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for ToolRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.request_id.validate()?;
        self.query_hash.validate()?;
        self.strict_budget.validate()?;
        if self.policy_context_ref.schema_version != PH1D_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "tool_request.policy_context_ref.schema_version",
                reason: "must match PH1D_CONTRACT_VERSION",
            });
        }
        if !matches!(self.origin, ToolRequestOrigin::Ph1X) {
            return Err(ContractViolation::InvalidValue {
                field: "tool_request.origin",
                reason: "must be ToolRequestOrigin::Ph1X",
            });
        }
        if matches!(self.tool_name, ToolName::Other(_)) {
            return Err(ContractViolation::InvalidValue {
                field: "tool_request.tool_name",
                reason: "must be an allowed tool",
            });
        }
        if self.query.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "tool_request.query",
                reason: "must not be empty",
            });
        }
        if self.query.len() > 2048 {
            return Err(ContractViolation::InvalidValue {
                field: "tool_request.query",
                reason: "must be <= 2048 chars",
            });
        }
        if let Some(loc) = &self.locale {
            if loc.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "tool_request.locale",
                    reason: "must not be empty when provided",
                });
            }
            if loc.len() > 32 {
                return Err(ContractViolation::InvalidValue {
                    field: "tool_request.locale",
                    reason: "must be <= 32 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRef {
    pub title: String,
    pub url: String,
}

impl Validate for SourceRef {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.title.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "source_ref.title",
                reason: "must not be empty",
            });
        }
        if self.title.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "source_ref.title",
                reason: "must be <= 256 chars",
            });
        }
        if self.url.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "source_ref.url",
                reason: "must not be empty",
            });
        }
        if self.url.len() > 2048 {
            return Err(ContractViolation::InvalidValue {
                field: "source_ref.url",
                reason: "must be <= 2048 chars",
            });
        }
        if self.url.chars().any(|c| c.is_whitespace()) {
            return Err(ContractViolation::InvalidValue {
                field: "source_ref.url",
                reason: "must not contain whitespace",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestedEntityPacket {
    pub requested_entity_id: String,
    pub captured_text: String,
    pub normalized_name: String,
    pub entity_type: String,
    pub known_entity_status: String,
    pub synthetic_allowed: bool,
    pub source_turn_id: String,
    pub language_hint: Option<String>,
    pub confidence: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceEvaluationPacket {
    pub source_id: String,
    pub requested_entity_id: String,
    pub title: String,
    pub domain: String,
    pub url: String,
    pub entity_match_result: String,
    pub claim_support_result: String,
    pub source_strength: String,
    pub accepted: bool,
    pub rejection_reasons: Vec<String>,
    pub claim_refs: Vec<String>,
    pub safe_for_user_display: bool,
    pub safe_for_tts: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedSourcePacket {
    pub source_id: String,
    pub label: String,
    pub domain: String,
    pub safe_click_url: String,
    pub source_type: String,
    pub supported_claim_refs: Vec<String>,
    pub entity_match_result: String,
    pub claim_support_result: String,
    pub accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedSourcePacket {
    pub source_id: String,
    pub domain: String,
    pub source_type: String,
    pub accepted: bool,
    pub rejection_reasons: Vec<String>,
    pub entity_match_result: String,
    pub claim_support_result: String,
    pub trace_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceChipPacket {
    pub source_id: String,
    pub label: String,
    pub domain: String,
    pub safe_click_url: String,
    pub source_type: String,
    pub accepted: bool,
    pub claim_refs: Vec<String>,
    pub icon_key: Option<String>,
    pub verified_for_claim: bool,
    pub display_rank: u16,
    pub tooltip_or_accessibility_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCardPacket {
    pub source_id: String,
    pub title: String,
    pub domain: String,
    pub safe_click_url: String,
    pub source_type: String,
    pub short_excerpt_or_summary: String,
    pub accepted: bool,
    pub claim_refs: Vec<String>,
    pub display_rank: u16,
    pub retrieved_at_human: Option<String>,
    pub metadata_safe_for_user: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchImagePacket {
    pub image_id: String,
    pub image_kind: String,
    pub approved_asset_ref: String,
    pub safe_image_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub source_page_url: String,
    pub source_page_domain: String,
    pub source_label: String,
    pub caption: String,
    pub alt_text: String,
    pub query_relevance_score: u16,
    pub entity_match_score: u16,
    pub source_id: String,
    pub claim_refs: Vec<String>,
    pub display_allowed: bool,
    pub display_denied_reason: Option<String>,
    pub provider: String,
    pub provider_tier: String,
    pub metadata_only: bool,
    pub rights_or_policy_status: String,
    pub retrieved_at: Option<String>,
    pub metadata_safe_for_user: bool,
    pub remote_image_load_allowed: bool,
    pub fixture_or_local_asset: bool,
    pub display_rank: u16,
    pub result_classes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentationPacket {
    pub display_text: String,
    pub response_text: String,
    pub tts_text: String,
    pub answer_class: String,
    pub language: String,
    pub source_chips: Vec<SourceChipPacket>,
    pub source_cards: Vec<SourceCardPacket>,
    pub image_cards: Vec<SearchImagePacket>,
    pub trace_id: String,
    pub metadata_safe_for_user: bool,
    pub response_style: String,
    pub expandable_available: bool,
    pub presentation_boundary_used: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimRequestPacket {
    pub request_id: String,
    pub turn_id: String,
    pub claim_id: String,
    pub claim_type: String,
    pub requested_entity: String,
    pub normalized_entity: String,
    pub claim_text: String,
    pub expected_answer_shape: String,
    pub freshness_required: bool,
    pub source_requirements: Vec<String>,
    pub generated_from_user_prompt: bool,
    pub protected_lane: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimEvidenceLink {
    pub claim_id: String,
    pub source_id: String,
    pub evidence_chunk_id: String,
    pub evidence_excerpt_hash: String,
    pub entity_match: String,
    pub claim_term_match: String,
    pub role_or_value_match: String,
    pub freshness_match: String,
    pub support_level: String,
    pub contradiction_level: String,
    pub confidence: u16,
    pub confidence_class: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimVerificationPacket {
    pub claim_id: String,
    pub claim_type: String,
    pub claim_text: String,
    pub requested_entity: String,
    pub verification_status: String,
    pub confidence: u16,
    pub confidence_class: String,
    pub supporting_sources: Vec<String>,
    pub contradicting_sources: Vec<String>,
    pub insufficient_sources: Vec<String>,
    pub rejected_sources: Vec<String>,
    pub evidence_links: Vec<ClaimEvidenceLink>,
    pub uncertainty_reason: Option<String>,
    pub selected_answer_value: Option<String>,
    pub source_hierarchy_reason: Option<String>,
    pub freshness_reason: Option<String>,
    pub safe_for_direct_answer: bool,
    pub user_visible_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebAnswerVerificationPacket {
    pub requested_entity: RequestedEntityPacket,
    pub normalized_entity: String,
    pub query: String,
    pub expanded_query: String,
    pub source_candidate_ids: Vec<String>,
    pub accepted_source_ids: Vec<String>,
    pub rejected_source_ids: Vec<String>,
    pub source_evaluations: Vec<SourceEvaluationPacket>,
    pub accepted_sources: Vec<AcceptedSourcePacket>,
    pub rejected_sources: Vec<RejectedSourcePacket>,
    pub source_chips: Vec<SourceChipPacket>,
    pub answer_claims: Vec<String>,
    pub claim_to_source_map: Vec<(String, String)>,
    pub claim_requests: Vec<ClaimRequestPacket>,
    pub claim_verifications: Vec<ClaimVerificationPacket>,
    pub unsupported_claims_removed: Vec<String>,
    pub contradiction_result: String,
    pub final_answer_class: String,
    pub presentation: PresentationPacket,
    pub response_text: String,
    pub source_dump_present: bool,
    pub rejected_sources_present_in_response_text: bool,
    pub debug_trace_present_in_response_text: bool,
    pub tts_input_text: String,
    pub displayed_response_text_sha256: String,
    pub tts_input_text_sha256: String,
    pub provider_call_count_when_disabled: u32,
}

impl Validate for RequestedEntityPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_bounded_nonempty(
            "requested_entity.requested_entity_id",
            &self.requested_entity_id,
            128,
        )?;
        validate_bounded_nonempty("requested_entity.captured_text", &self.captured_text, 512)?;
        validate_bounded_nonempty(
            "requested_entity.normalized_name",
            &self.normalized_name,
            256,
        )?;
        validate_bounded_nonempty("requested_entity.entity_type", &self.entity_type, 64)?;
        validate_bounded_nonempty(
            "requested_entity.known_entity_status",
            &self.known_entity_status,
            64,
        )?;
        validate_bounded_nonempty("requested_entity.source_turn_id", &self.source_turn_id, 128)?;
        if !matches!(
            self.known_entity_status.as_str(),
            "KNOWN" | "UNKNOWN" | "SYNTHETIC_OR_TEST" | "AMBIGUOUS"
        ) {
            return Err(ContractViolation::InvalidValue {
                field: "requested_entity.known_entity_status",
                reason: "must be an allowed status",
            });
        }
        if let Some(language_hint) = &self.language_hint {
            validate_bounded_nonempty("requested_entity.language_hint", language_hint, 32)?;
        }
        if self.confidence > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "requested_entity.confidence",
                reason: "must be <= 10000",
            });
        }
        Ok(())
    }
}

impl Validate for SourceEvaluationPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_bounded_nonempty("source_evaluation.source_id", &self.source_id, 128)?;
        validate_bounded_nonempty(
            "source_evaluation.requested_entity_id",
            &self.requested_entity_id,
            128,
        )?;
        validate_bounded_nonempty("source_evaluation.title", &self.title, 256)?;
        validate_bounded_nonempty("source_evaluation.domain", &self.domain, 256)?;
        validate_safe_url("source_evaluation.url", &self.url)?;
        validate_stage1_token(
            "source_evaluation.entity_match_result",
            &self.entity_match_result,
            &[
                "ENTITY_MATCH_STRONG",
                "ENTITY_MATCH_MEDIUM",
                "ENTITY_MATCH_WEAK",
                "ENTITY_MATCH_REJECT",
                "ENTITY_MATCH_UNKNOWN",
            ],
        )?;
        validate_stage1_token(
            "source_evaluation.claim_support_result",
            &self.claim_support_result,
            &[
                "CLAIM_SUPPORT_DIRECT",
                "CLAIM_SUPPORT_IMPLIED",
                "CLAIM_SUPPORT_ENTITY_ONLY",
                "CLAIM_SUPPORT_NONE",
            ],
        )?;
        validate_bounded_nonempty(
            "source_evaluation.source_strength",
            &self.source_strength,
            64,
        )?;
        if self.accepted && !self.rejection_reasons.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "source_evaluation.rejection_reasons",
                reason: "must be empty for accepted sources",
            });
        }
        if !self.accepted && self.rejection_reasons.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "source_evaluation.rejection_reasons",
                reason: "must not be empty for rejected sources",
            });
        }
        validate_reason_codes(&self.rejection_reasons)?;
        validate_short_vec("source_evaluation.claim_refs", &self.claim_refs, 20, 128)?;
        if self.accepted && (!self.safe_for_user_display || !self.safe_for_tts) {
            return Err(ContractViolation::InvalidValue {
                field: "source_evaluation.safe",
                reason: "accepted sources must be safe for display and tts metadata",
            });
        }
        Ok(())
    }
}

impl Validate for AcceptedSourcePacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_bounded_nonempty("accepted_source.source_id", &self.source_id, 128)?;
        validate_bounded_nonempty("accepted_source.label", &self.label, 256)?;
        validate_bounded_nonempty("accepted_source.domain", &self.domain, 256)?;
        validate_safe_url("accepted_source.safe_click_url", &self.safe_click_url)?;
        validate_bounded_nonempty("accepted_source.source_type", &self.source_type, 64)?;
        validate_short_vec(
            "accepted_source.supported_claim_refs",
            &self.supported_claim_refs,
            20,
            128,
        )?;
        if self.supported_claim_refs.is_empty() || !self.accepted {
            return Err(ContractViolation::InvalidValue {
                field: "accepted_source.accepted",
                reason: "accepted sources must be accepted and support at least one claim",
            });
        }
        validate_bounded_nonempty(
            "accepted_source.entity_match_result",
            &self.entity_match_result,
            64,
        )?;
        validate_bounded_nonempty(
            "accepted_source.claim_support_result",
            &self.claim_support_result,
            64,
        )?;
        Ok(())
    }
}

impl Validate for RejectedSourcePacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_bounded_nonempty("rejected_source.source_id", &self.source_id, 128)?;
        validate_bounded_nonempty("rejected_source.domain", &self.domain, 256)?;
        validate_bounded_nonempty("rejected_source.source_type", &self.source_type, 64)?;
        if self.accepted || !self.trace_only || self.rejection_reasons.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "rejected_source.accepted",
                reason: "rejected sources must be trace-only and include reasons",
            });
        }
        validate_reason_codes(&self.rejection_reasons)?;
        validate_bounded_nonempty(
            "rejected_source.entity_match_result",
            &self.entity_match_result,
            64,
        )?;
        validate_bounded_nonempty(
            "rejected_source.claim_support_result",
            &self.claim_support_result,
            64,
        )?;
        Ok(())
    }
}

impl Validate for SourceChipPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_bounded_nonempty("source_chip.source_id", &self.source_id, 128)?;
        validate_bounded_nonempty("source_chip.label", &self.label, 256)?;
        validate_bounded_nonempty("source_chip.domain", &self.domain, 256)?;
        validate_safe_url("source_chip.safe_click_url", &self.safe_click_url)?;
        validate_bounded_nonempty("source_chip.source_type", &self.source_type, 64)?;
        validate_short_vec("source_chip.claim_refs", &self.claim_refs, 20, 128)?;
        if let Some(icon_key) = &self.icon_key {
            validate_bounded_nonempty("source_chip.icon_key", icon_key, 64)?;
        }
        validate_bounded_nonempty(
            "source_chip.tooltip_or_accessibility_label",
            &self.tooltip_or_accessibility_label,
            256,
        )?;
        if !self.accepted || self.claim_refs.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "source_chip.accepted",
                reason: "source chips must derive from accepted claim-supporting sources",
            });
        }
        if !self.verified_for_claim {
            return Err(ContractViolation::InvalidValue {
                field: "source_chip.verified_for_claim",
                reason: "source chips must be verified for at least one claim",
            });
        }
        Ok(())
    }
}

impl Validate for SourceCardPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_bounded_nonempty("source_card.source_id", &self.source_id, 128)?;
        validate_bounded_nonempty("source_card.title", &self.title, 256)?;
        validate_bounded_nonempty("source_card.domain", &self.domain, 256)?;
        validate_safe_url("source_card.safe_click_url", &self.safe_click_url)?;
        validate_bounded_nonempty("source_card.source_type", &self.source_type, 64)?;
        validate_bounded_nonempty(
            "source_card.short_excerpt_or_summary",
            &self.short_excerpt_or_summary,
            500,
        )?;
        validate_short_vec("source_card.claim_refs", &self.claim_refs, 20, 128)?;
        if let Some(retrieved_at_human) = &self.retrieved_at_human {
            validate_bounded_nonempty("source_card.retrieved_at_human", retrieved_at_human, 128)?;
        }
        if !self.accepted || self.claim_refs.is_empty() || !self.metadata_safe_for_user {
            return Err(ContractViolation::InvalidValue {
                field: "source_card.accepted",
                reason: "source cards must be accepted, claim-linked, and metadata-safe",
            });
        }
        Ok(())
    }
}

impl Validate for SearchImagePacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_bounded_nonempty("search_image.image_id", &self.image_id, 128)?;
        validate_stage1_token(
            "search_image.image_kind",
            &self.image_kind,
            &[
                "logo",
                "person_photo",
                "building_or_place",
                "product",
                "brand_asset",
                "generic_entity_visual",
                "unknown",
            ],
        )?;
        validate_stage6_fixture_asset_ref(
            "search_image.approved_asset_ref",
            &self.approved_asset_ref,
        )?;
        if let Some(safe_image_url) = &self.safe_image_url {
            validate_safe_url("search_image.safe_image_url", safe_image_url)?;
            if !self.remote_image_load_allowed {
                return Err(ContractViolation::InvalidValue {
                    field: "search_image.safe_image_url",
                    reason: "remote image URLs require explicit remote image load approval",
                });
            }
        }
        if let Some(thumbnail_url) = &self.thumbnail_url {
            validate_safe_url("search_image.thumbnail_url", thumbnail_url)?;
            if !self.remote_image_load_allowed {
                return Err(ContractViolation::InvalidValue {
                    field: "search_image.thumbnail_url",
                    reason: "remote image thumbnails require explicit remote image load approval",
                });
            }
        }
        validate_safe_url("search_image.source_page_url", &self.source_page_url)?;
        validate_bounded_nonempty(
            "search_image.source_page_domain",
            &self.source_page_domain,
            256,
        )?;
        validate_bounded_nonempty("search_image.source_label", &self.source_label, 256)?;
        validate_bounded_nonempty("search_image.caption", &self.caption, 220)?;
        validate_bounded_nonempty("search_image.alt_text", &self.alt_text, 220)?;
        validate_bounded_nonempty("search_image.source_id", &self.source_id, 128)?;
        validate_short_vec("search_image.claim_refs", &self.claim_refs, 20, 128)?;
        if let Some(display_denied_reason) = &self.display_denied_reason {
            validate_bounded_nonempty(
                "search_image.display_denied_reason",
                display_denied_reason,
                160,
            )?;
        }
        validate_bounded_nonempty("search_image.provider", &self.provider, 64)?;
        validate_bounded_nonempty("search_image.provider_tier", &self.provider_tier, 64)?;
        validate_stage1_token(
            "search_image.rights_or_policy_status",
            &self.rights_or_policy_status,
            &["fixture_approved", "display_policy_approved"],
        )?;
        if let Some(retrieved_at) = &self.retrieved_at {
            validate_bounded_nonempty("search_image.retrieved_at", retrieved_at, 128)?;
        }
        validate_short_vec("search_image.result_classes", &self.result_classes, 32, 128)?;
        for required in [
            "STAGE6_IMAGE_PACKET_PASS",
            "STAGE6_IMAGE_DISPLAY_GATE_PASS",
            "STAGE6_IMAGE_URL_SAFETY_PASS",
            "STAGE6_SOURCE_PAGE_LINK_PASS",
            "STAGE6_QUERY_RELEVANCE_PASS",
        ] {
            if !self.result_classes.iter().any(|class| class == required) {
                return Err(ContractViolation::InvalidValue {
                    field: "search_image.result_classes",
                    reason: "approved image cards must include Stage 6 result-class proof",
                });
            }
        }
        if self.query_relevance_score > 10_000 || self.entity_match_score > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "search_image.relevance",
                reason: "scores must be <= 10000",
            });
        }
        if self.query_relevance_score < 7_500 || self.entity_match_score < 7_500 {
            return Err(ContractViolation::InvalidValue {
                field: "search_image.relevance",
                reason: "displayed image cards require strong query and entity relevance",
            });
        }
        if !self.display_allowed
            || self.display_denied_reason.is_some()
            || !self.metadata_safe_for_user
            || self.metadata_only
            || self.claim_refs.is_empty()
        {
            return Err(ContractViolation::InvalidValue {
                field: "search_image.display_allowed",
                reason: "normal image cards must be allowed, metadata-safe, and claim-linked",
            });
        }
        if self.remote_image_load_allowed && !self.fixture_or_local_asset {
            return Err(ContractViolation::InvalidValue {
                field: "search_image.remote_image_load_allowed",
                reason: "Stage 6 normal payload permits fixture/local rendering only",
            });
        }
        if !self.fixture_or_local_asset {
            return Err(ContractViolation::InvalidValue {
                field: "search_image.fixture_or_local_asset",
                reason: "Stage 6 displayed image cards must use approved fixture/local assets",
            });
        }
        Ok(())
    }
}

impl Validate for PresentationPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_bounded_nonempty("presentation.display_text", &self.display_text, 4096)?;
        validate_bounded_nonempty("presentation.response_text", &self.response_text, 4096)?;
        validate_bounded_nonempty("presentation.tts_text", &self.tts_text, 4096)?;
        validate_stage1_token(
            "presentation.answer_class",
            &self.answer_class,
            &[
                "VERIFIED_DIRECT_ANSWER",
                "SOURCE_DISCOVERY_ONLY",
                "PARTIAL_UNCERTAIN_ANSWER",
                "UNSUPPORTED_SAFE_DEGRADE",
                "CONTRADICTED_SAFE_DEGRADE",
                "STALE_UNCERTAIN_SAFE_DEGRADE",
                "PROTECTED_FAIL_CLOSED",
            ],
        )?;
        validate_bounded_nonempty("presentation.language", &self.language, 32)?;
        for chip in &self.source_chips {
            chip.validate()?;
        }
        for card in &self.source_cards {
            card.validate()?;
        }
        if self.image_cards.len() > 3 {
            return Err(ContractViolation::InvalidValue {
                field: "presentation.image_cards",
                reason: "Stage 6 presentation displays at most three image cards by default",
            });
        }
        for image_card in &self.image_cards {
            image_card.validate()?;
        }
        validate_bounded_nonempty("presentation.trace_id", &self.trace_id, 128)?;
        validate_stage1_token(
            "presentation.response_style",
            &self.response_style,
            &["concise_default"],
        )?;
        validate_bounded_nonempty(
            "presentation.presentation_boundary_used",
            &self.presentation_boundary_used,
            96,
        )?;
        if !self.metadata_safe_for_user {
            return Err(ContractViolation::InvalidValue {
                field: "presentation.metadata_safe_for_user",
                reason: "presentation metadata must be safe for user payload transport",
            });
        }
        if self.tts_text != self.response_text || self.display_text != self.response_text {
            return Err(ContractViolation::InvalidValue {
                field: "presentation.tts_text",
                reason: "Stage 5 TTS/display text must equal clean response_text",
            });
        }
        Ok(())
    }
}

impl Validate for ClaimRequestPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_bounded_nonempty("claim_request.request_id", &self.request_id, 128)?;
        validate_bounded_nonempty("claim_request.turn_id", &self.turn_id, 128)?;
        validate_bounded_nonempty("claim_request.claim_id", &self.claim_id, 128)?;
        validate_stage1_token(
            "claim_request.claim_type",
            &self.claim_type,
            &[
                "leadership_role",
                "identity_or_title",
                "company_fact",
                "current_status",
                "date_or_event",
                "numeric_value",
                "price_or_cost",
                "location_or_address",
                "ownership_or_affiliation",
                "policy_or_rule",
                "comparison",
                "list_or_ranking",
                "definition_or_explanation",
                "source_discovery",
                "unknown_factual",
            ],
        )?;
        validate_bounded_nonempty(
            "claim_request.requested_entity",
            &self.requested_entity,
            256,
        )?;
        validate_bounded_nonempty(
            "claim_request.normalized_entity",
            &self.normalized_entity,
            256,
        )?;
        validate_bounded_nonempty("claim_request.claim_text", &self.claim_text, 512)?;
        validate_bounded_nonempty(
            "claim_request.expected_answer_shape",
            &self.expected_answer_shape,
            128,
        )?;
        validate_short_vec(
            "claim_request.source_requirements",
            &self.source_requirements,
            20,
            128,
        )?;
        if self.protected_lane {
            return Err(ContractViolation::InvalidValue {
                field: "claim_request.protected_lane",
                reason: "public websearch claim verification must not be protected execution",
            });
        }
        Ok(())
    }
}

impl Validate for ClaimEvidenceLink {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_bounded_nonempty("claim_evidence_link.claim_id", &self.claim_id, 128)?;
        validate_bounded_nonempty("claim_evidence_link.source_id", &self.source_id, 128)?;
        validate_bounded_nonempty(
            "claim_evidence_link.evidence_chunk_id",
            &self.evidence_chunk_id,
            128,
        )?;
        validate_sha256_hex(
            "claim_evidence_link.evidence_excerpt_hash",
            &self.evidence_excerpt_hash,
        )?;
        validate_bounded_nonempty("claim_evidence_link.entity_match", &self.entity_match, 96)?;
        validate_bounded_nonempty(
            "claim_evidence_link.claim_term_match",
            &self.claim_term_match,
            96,
        )?;
        validate_bounded_nonempty(
            "claim_evidence_link.role_or_value_match",
            &self.role_or_value_match,
            96,
        )?;
        validate_bounded_nonempty(
            "claim_evidence_link.freshness_match",
            &self.freshness_match,
            96,
        )?;
        validate_stage1_token(
            "claim_evidence_link.support_level",
            &self.support_level,
            &[
                "DIRECT_SUPPORT",
                "INDIRECT_SUPPORT",
                "ENTITY_ONLY",
                "MENTION_ONLY",
                "NO_SUPPORT",
                "CONTRADICTS",
                "STALE_SUPPORT",
                "WRONG_ENTITY",
            ],
        )?;
        validate_stage1_token(
            "claim_evidence_link.contradiction_level",
            &self.contradiction_level,
            &[
                "resolved_by_higher_trust_source",
                "resolved_by_freshness",
                "unresolved_conflict",
                "low_confidence_conflict",
                "no_conflict",
            ],
        )?;
        if self.confidence > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "claim_evidence_link.confidence",
                reason: "must be <= 10000",
            });
        }
        validate_confidence_class(&self.confidence_class)?;
        validate_bounded_nonempty("claim_evidence_link.reason", &self.reason, 512)?;
        Ok(())
    }
}

impl Validate for ClaimVerificationPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_bounded_nonempty("claim_verification.claim_id", &self.claim_id, 128)?;
        validate_stage1_token(
            "claim_verification.claim_type",
            &self.claim_type,
            &[
                "leadership_role",
                "identity_or_title",
                "company_fact",
                "current_status",
                "date_or_event",
                "numeric_value",
                "price_or_cost",
                "location_or_address",
                "ownership_or_affiliation",
                "policy_or_rule",
                "comparison",
                "list_or_ranking",
                "definition_or_explanation",
                "source_discovery",
                "unknown_factual",
            ],
        )?;
        validate_bounded_nonempty("claim_verification.claim_text", &self.claim_text, 512)?;
        validate_bounded_nonempty(
            "claim_verification.requested_entity",
            &self.requested_entity,
            256,
        )?;
        validate_stage1_token(
            "claim_verification.verification_status",
            &self.verification_status,
            &[
                "SUPPORTED",
                "PARTIALLY_SUPPORTED",
                "UNSUPPORTED",
                "CONTRADICTED",
                "CONFLICT_UNRESOLVED",
                "STALE_UNCERTAIN",
                "ENTITY_MISMATCH",
                "INSUFFICIENT_EVIDENCE",
                "PROTECTED_FAIL_CLOSED",
            ],
        )?;
        if self.confidence > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "claim_verification.confidence",
                reason: "must be <= 10000",
            });
        }
        validate_confidence_class(&self.confidence_class)?;
        validate_short_vec(
            "claim_verification.supporting_sources",
            &self.supporting_sources,
            20,
            128,
        )?;
        validate_short_vec(
            "claim_verification.contradicting_sources",
            &self.contradicting_sources,
            20,
            128,
        )?;
        validate_short_vec(
            "claim_verification.insufficient_sources",
            &self.insufficient_sources,
            20,
            128,
        )?;
        validate_short_vec(
            "claim_verification.rejected_sources",
            &self.rejected_sources,
            20,
            128,
        )?;
        for link in &self.evidence_links {
            link.validate()?;
        }
        if let Some(reason) = &self.uncertainty_reason {
            validate_bounded_nonempty("claim_verification.uncertainty_reason", reason, 512)?;
        }
        if let Some(value) = &self.selected_answer_value {
            validate_bounded_nonempty("claim_verification.selected_answer_value", value, 256)?;
        }
        if let Some(reason) = &self.source_hierarchy_reason {
            validate_bounded_nonempty("claim_verification.source_hierarchy_reason", reason, 512)?;
        }
        if let Some(reason) = &self.freshness_reason {
            validate_bounded_nonempty("claim_verification.freshness_reason", reason, 512)?;
        }
        validate_bounded_nonempty(
            "claim_verification.user_visible_summary",
            &self.user_visible_summary,
            1024,
        )?;
        if self.safe_for_direct_answer
            && (self.verification_status != "SUPPORTED" || self.supporting_sources.is_empty())
        {
            return Err(ContractViolation::InvalidValue {
                field: "claim_verification.safe_for_direct_answer",
                reason: "direct answers require supported claim-linked evidence",
            });
        }
        Ok(())
    }
}

impl Validate for WebAnswerVerificationPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.requested_entity.validate()?;
        validate_bounded_nonempty(
            "web_answer_verification.normalized_entity",
            &self.normalized_entity,
            256,
        )?;
        validate_bounded_nonempty("web_answer_verification.query", &self.query, 2048)?;
        validate_bounded_nonempty(
            "web_answer_verification.expanded_query",
            &self.expanded_query,
            2048,
        )?;
        validate_short_vec(
            "web_answer_verification.source_candidate_ids",
            &self.source_candidate_ids,
            20,
            128,
        )?;
        validate_short_vec(
            "web_answer_verification.accepted_source_ids",
            &self.accepted_source_ids,
            20,
            128,
        )?;
        validate_short_vec(
            "web_answer_verification.rejected_source_ids",
            &self.rejected_source_ids,
            20,
            128,
        )?;
        for evaluation in &self.source_evaluations {
            evaluation.validate()?;
        }
        for accepted in &self.accepted_sources {
            accepted.validate()?;
        }
        for rejected in &self.rejected_sources {
            rejected.validate()?;
        }
        for chip in &self.source_chips {
            chip.validate()?;
            if !self.accepted_source_ids.contains(&chip.source_id) {
                return Err(ContractViolation::InvalidValue {
                    field: "web_answer_verification.source_chips",
                    reason: "source chips must come from accepted source ids",
                });
            }
        }
        validate_short_vec(
            "web_answer_verification.answer_claims",
            &self.answer_claims,
            20,
            512,
        )?;
        for request in &self.claim_requests {
            request.validate()?;
        }
        for verification in &self.claim_verifications {
            verification.validate()?;
        }
        validate_short_vec(
            "web_answer_verification.unsupported_claims_removed",
            &self.unsupported_claims_removed,
            20,
            512,
        )?;
        validate_stage1_token(
            "web_answer_verification.contradiction_result",
            &self.contradiction_result,
            &[
                "resolved_by_higher_trust_source",
                "resolved_by_freshness",
                "unresolved_conflict",
                "low_confidence_conflict",
                "no_conflict",
            ],
        )?;
        validate_stage1_token(
            "web_answer_verification.final_answer_class",
            &self.final_answer_class,
            &[
                "VERIFIED_DIRECT_ANSWER",
                "SOURCE_DISCOVERY_ONLY",
                "PARTIAL_UNCERTAIN_ANSWER",
                "UNSUPPORTED_SAFE_DEGRADE",
                "CONTRADICTED_SAFE_DEGRADE",
                "STALE_UNCERTAIN_SAFE_DEGRADE",
                "PROTECTED_FAIL_CLOSED",
            ],
        )?;
        self.presentation.validate()?;
        validate_bounded_nonempty(
            "web_answer_verification.response_text",
            &self.response_text,
            4096,
        )?;
        validate_bounded_nonempty(
            "web_answer_verification.tts_input_text",
            &self.tts_input_text,
            4096,
        )?;
        validate_sha256_hex(
            "web_answer_verification.displayed_response_text_sha256",
            &self.displayed_response_text_sha256,
        )?;
        validate_sha256_hex(
            "web_answer_verification.tts_input_text_sha256",
            &self.tts_input_text_sha256,
        )?;
        if self.tts_input_text != self.response_text {
            return Err(ContractViolation::InvalidValue {
                field: "web_answer_verification.tts_input_text",
                reason: "must equal clean response_text for Stage 1",
            });
        }
        if self.presentation.response_text != self.response_text
            || self.presentation.tts_text != self.tts_input_text
            || self.presentation.answer_class != self.final_answer_class
        {
            return Err(ContractViolation::InvalidValue {
                field: "web_answer_verification.presentation",
                reason: "presentation packet must preserve final answer text, TTS text, and answer class",
            });
        }
        if self.presentation.source_chips != self.source_chips {
            return Err(ContractViolation::InvalidValue {
                field: "web_answer_verification.presentation.source_chips",
                reason: "presentation source chips must match accepted verification source chips",
            });
        }
        for image_card in &self.presentation.image_cards {
            if !self.accepted_source_ids.contains(&image_card.source_id) {
                return Err(ContractViolation::InvalidValue {
                    field: "web_answer_verification.presentation.image_cards",
                    reason: "image cards must be linked to accepted source ids",
                });
            }
            if image_card.claim_refs.iter().any(|claim_ref| {
                !self
                    .claim_requests
                    .iter()
                    .any(|claim| &claim.claim_id == claim_ref)
            }) {
                return Err(ContractViolation::InvalidValue {
                    field: "web_answer_verification.presentation.image_cards",
                    reason: "image card claim refs must be covered by final answer claims",
                });
            }
        }
        if self.source_dump_present
            || self.rejected_sources_present_in_response_text
            || self.debug_trace_present_in_response_text
        {
            return Err(ContractViolation::InvalidValue {
                field: "web_answer_verification.response_text",
                reason: "must not contain source dumps, rejected sources, or debug trace",
            });
        }
        if self.final_answer_class == "VERIFIED_DIRECT_ANSWER"
            && (self.accepted_source_ids.is_empty() || self.answer_claims.is_empty())
        {
            return Err(ContractViolation::InvalidValue {
                field: "web_answer_verification.accepted_source_ids",
                reason: "verified answers require accepted source-backed claims",
            });
        }
        if self.final_answer_class == "VERIFIED_DIRECT_ANSWER"
            && !self
                .claim_verifications
                .iter()
                .any(|claim| claim.safe_for_direct_answer)
        {
            return Err(ContractViolation::InvalidValue {
                field: "web_answer_verification.claim_verifications",
                reason: "verified direct answers require a safe direct claim verification",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMetadata {
    pub schema_version: SchemaVersion,
    /// Optional, coarse hint only. Must not be required for upstream logic.
    pub provider_hint: Option<String>,
    pub retrieved_at_unix_ms: u64,
    /// Accepted/user-displayable sources only. Rejected sources live in trace-only verification.
    pub sources: Vec<SourceRef>,
    pub web_answer_verification: Option<WebAnswerVerificationPacket>,
}

impl Validate for SourceMetadata {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1E_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "source_metadata.schema_version",
                reason: "must match PH1E_CONTRACT_VERSION",
            });
        }
        if let Some(h) = &self.provider_hint {
            if h.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "source_metadata.provider_hint",
                    reason: "must not be empty when provided",
                });
            }
            if h.len() > 64 {
                return Err(ContractViolation::InvalidValue {
                    field: "source_metadata.provider_hint",
                    reason: "must be <= 64 chars",
                });
            }
        }
        if self.retrieved_at_unix_ms == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "source_metadata.retrieved_at_unix_ms",
                reason: "must be > 0",
            });
        }
        if self.sources.is_empty() && self.web_answer_verification.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "source_metadata.sources",
                reason: "must not be empty unless web_answer_verification carries an unsupported safe-degrade packet",
            });
        }
        if self.sources.len() > 20 {
            return Err(ContractViolation::InvalidValue {
                field: "source_metadata.sources",
                reason: "must be <= 20 entries",
            });
        }
        for s in &self.sources {
            s.validate()?;
        }
        if let Some(verification) = &self.web_answer_verification {
            verification.validate()?;
        }
        Ok(())
    }
}

fn validate_bounded_nonempty(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    Ok(())
}

fn validate_safe_url(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    validate_bounded_nonempty(field, value, 2048)?;
    if (!value.starts_with("https://") && !value.starts_with("http://"))
        || value.chars().any(|ch| ch.is_whitespace())
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be an http(s) URL without whitespace",
        });
    }
    Ok(())
}

fn validate_stage6_fixture_asset_ref(
    field: &'static str,
    value: &str,
) -> Result<(), ContractViolation> {
    validate_bounded_nonempty(field, value, 128)?;
    let allowed_chars = value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'));
    if !allowed_chars
        || value.contains("..")
        || !value.starts_with("fixture-image-")
        || !value.ends_with(".png")
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be an approved Stage 6 fixture/local image asset reference",
        });
    }
    Ok(())
}

fn validate_stage1_token(
    field: &'static str,
    value: &str,
    allowed: &[&str],
) -> Result<(), ContractViolation> {
    validate_bounded_nonempty(field, value, 96)?;
    if !allowed.contains(&value) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be an allowed Stage 1 token",
        });
    }
    Ok(())
}

fn validate_confidence_class(value: &str) -> Result<(), ContractViolation> {
    validate_stage1_token(
        "confidence_class",
        value,
        &["HIGH", "MEDIUM", "LOW", "UNKNOWN"],
    )
}

fn validate_reason_codes(codes: &[String]) -> Result<(), ContractViolation> {
    for code in codes {
        validate_stage1_token(
            "rejection_reasons[]",
            code,
            &[
                "ENTITY_MISMATCH",
                "PARTIAL_NAME_OVERLAP_ONLY",
                "SIMILAR_SOUNDING_NAME_ONLY",
                "MENTIONS_ENTITY_ONLY",
                "CLAIM_NOT_SUPPORTED",
                "TOPIC_SOURCE_OVERLAP_INSUFFICIENT",
                "WEAK_SEO_SOURCE",
                "SCRAPED_PROFILE_SOURCE",
                "THIN_DIRECTORY_SOURCE",
                "STALE_FOR_CURRENT_CLAIM",
                "UNVERIFIED_PROFILE_SOURCE",
                "LOW_TRUST_SOURCE",
                "RAW_PROVIDER_DUMP_BLOCKED",
                "UNSAFE_CLICK_URL",
                "DEBUG_TRACE_OUTPUT_BLOCKED",
                "PRIVATE_OR_PROTECTED_QUERY_BLOCKED",
                "PROVIDER_DISABLED_ZERO_CALL_REQUIRED",
            ],
        )?;
    }
    Ok(())
}

fn validate_short_vec(
    field: &'static str,
    values: &[String],
    max_items: usize,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if values.len() > max_items {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "too many entries",
        });
    }
    for value in values {
        validate_bounded_nonempty(field, value, max_len)?;
    }
    Ok(())
}

fn validate_sha256_hex(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.len() != 64 || !value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be a 64-char sha256 hex string",
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolTextSnippet {
    pub title: String,
    pub snippet: String,
    pub url: String,
}

impl Validate for ToolTextSnippet {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.title.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "tool_text_snippet.title",
                reason: "must not be empty",
            });
        }
        if self.title.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "tool_text_snippet.title",
                reason: "must be <= 256 chars",
            });
        }
        if self.snippet.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "tool_text_snippet.snippet",
                reason: "must not be empty",
            });
        }
        if self.snippet.len() > 2048 {
            return Err(ContractViolation::InvalidValue {
                field: "tool_text_snippet.snippet",
                reason: "must be <= 2048 chars",
            });
        }
        if self.url.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "tool_text_snippet.url",
                reason: "must not be empty",
            });
        }
        if self.url.len() > 2048 {
            return Err(ContractViolation::InvalidValue {
                field: "tool_text_snippet.url",
                reason: "must be <= 2048 chars",
            });
        }
        if self.url.chars().any(|c| c.is_whitespace()) {
            return Err(ContractViolation::InvalidValue {
                field: "tool_text_snippet.url",
                reason: "must not contain whitespace",
            });
        }
        if !(self.url.starts_with("https://") || self.url.starts_with("http://")) {
            return Err(ContractViolation::InvalidValue {
                field: "tool_text_snippet.url",
                reason: "must start with http:// or https://",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolStructuredField {
    pub key: String,
    pub value: String,
}

impl Validate for ToolStructuredField {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.key.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "tool_structured_field.key",
                reason: "must not be empty",
            });
        }
        if self.key.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "tool_structured_field.key",
                reason: "must be <= 128 chars",
            });
        }
        if self.value.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "tool_structured_field.value",
                reason: "must not be empty",
            });
        }
        if self.value.len() > 1024 {
            return Err(ContractViolation::InvalidValue {
                field: "tool_structured_field.value",
                reason: "must be <= 1024 chars",
            });
        }
        if self.key.chars().any(|c| c.is_control()) || self.value.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "tool_structured_field",
                reason: "must not contain control characters",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredAmbiguity {
    pub summary: String,
    pub alternatives: Vec<String>,
}

impl Validate for StructuredAmbiguity {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.summary.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "structured_ambiguity.summary",
                reason: "must not be empty",
            });
        }
        if self.summary.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "structured_ambiguity.summary",
                reason: "must be <= 256 chars",
            });
        }
        if self.alternatives.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "structured_ambiguity.alternatives",
                reason: "must not be empty",
            });
        }
        if self.alternatives.len() > 3 {
            return Err(ContractViolation::InvalidValue {
                field: "structured_ambiguity.alternatives",
                reason: "must be <= 3 entries",
            });
        }
        for a in &self.alternatives {
            if a.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "structured_ambiguity.alternatives[]",
                    reason: "must not contain empty strings",
                });
            }
            if a.len() > 256 {
                return Err(ContractViolation::InvalidValue {
                    field: "structured_ambiguity.alternatives[]",
                    reason: "must be <= 256 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolResult {
    Time {
        local_time_iso: String,
    },
    Weather {
        summary: String,
    },
    WebSearch {
        items: Vec<ToolTextSnippet>,
    },
    News {
        items: Vec<ToolTextSnippet>,
    },
    UrlFetchAndCite {
        citations: Vec<ToolTextSnippet>,
    },
    DocumentUnderstand {
        summary: String,
        extracted_fields: Vec<ToolStructuredField>,
        citations: Vec<ToolTextSnippet>,
    },
    PhotoUnderstand {
        summary: String,
        extracted_fields: Vec<ToolStructuredField>,
        citations: Vec<ToolTextSnippet>,
    },
    DataAnalysis {
        summary: String,
        extracted_fields: Vec<ToolStructuredField>,
        citations: Vec<ToolTextSnippet>,
    },
    DeepResearch {
        summary: String,
        extracted_fields: Vec<ToolStructuredField>,
        citations: Vec<ToolTextSnippet>,
    },
    RecordMode {
        summary: String,
        action_items: Vec<ToolStructuredField>,
        evidence_refs: Vec<ToolStructuredField>,
    },
    ConnectorQuery {
        summary: String,
        extracted_fields: Vec<ToolStructuredField>,
        citations: Vec<ToolTextSnippet>,
    },
}

impl Validate for ToolResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        fn validate_items(
            field_name: &'static str,
            items: &[ToolTextSnippet],
        ) -> Result<(), ContractViolation> {
            if items.is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: field_name,
                    reason: "must not be empty",
                });
            }
            if items.len() > 20 {
                return Err(ContractViolation::InvalidValue {
                    field: field_name,
                    reason: "must be <= 20 entries",
                });
            }
            for it in items {
                it.validate()?;
            }
            Ok(())
        }

        match self {
            ToolResult::Time { local_time_iso } => {
                if local_time_iso.trim().is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.time.local_time_iso",
                        reason: "must not be empty",
                    });
                }
                if local_time_iso.len() > 64 {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.time.local_time_iso",
                        reason: "must be <= 64 chars",
                    });
                }
            }
            ToolResult::Weather { summary } => {
                if summary.trim().is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.weather.summary",
                        reason: "must not be empty",
                    });
                }
                if summary.len() > 512 {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.weather.summary",
                        reason: "must be <= 512 chars",
                    });
                }
            }
            ToolResult::WebSearch { items } | ToolResult::News { items } => {
                validate_items("tool_result.items", items)?;
            }
            ToolResult::UrlFetchAndCite { citations } => {
                validate_items("tool_result.citations", citations)?;
            }
            ToolResult::DocumentUnderstand {
                summary,
                extracted_fields,
                citations,
            } => {
                if summary.trim().is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.document_understand.summary",
                        reason: "must not be empty",
                    });
                }
                if summary.len() > 1024 {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.document_understand.summary",
                        reason: "must be <= 1024 chars",
                    });
                }
                if extracted_fields.is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.document_understand.extracted_fields",
                        reason: "must not be empty",
                    });
                }
                if extracted_fields.len() > 20 {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.document_understand.extracted_fields",
                        reason: "must be <= 20 entries",
                    });
                }
                for field in extracted_fields {
                    field.validate()?;
                }
                validate_items("tool_result.document_understand.citations", citations)?;
            }
            ToolResult::PhotoUnderstand {
                summary,
                extracted_fields,
                citations,
            } => {
                if summary.trim().is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.photo_understand.summary",
                        reason: "must not be empty",
                    });
                }
                if summary.len() > 1024 {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.photo_understand.summary",
                        reason: "must be <= 1024 chars",
                    });
                }
                if extracted_fields.is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.photo_understand.extracted_fields",
                        reason: "must not be empty",
                    });
                }
                if extracted_fields.len() > 20 {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.photo_understand.extracted_fields",
                        reason: "must be <= 20 entries",
                    });
                }
                for field in extracted_fields {
                    field.validate()?;
                }
                validate_items("tool_result.photo_understand.citations", citations)?;
            }
            ToolResult::DataAnalysis {
                summary,
                extracted_fields,
                citations,
            } => {
                if summary.trim().is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.data_analysis.summary",
                        reason: "must not be empty",
                    });
                }
                if summary.len() > 1024 {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.data_analysis.summary",
                        reason: "must be <= 1024 chars",
                    });
                }
                if extracted_fields.is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.data_analysis.extracted_fields",
                        reason: "must not be empty",
                    });
                }
                if extracted_fields.len() > 20 {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.data_analysis.extracted_fields",
                        reason: "must be <= 20 entries",
                    });
                }
                for field in extracted_fields {
                    field.validate()?;
                }
                validate_items("tool_result.data_analysis.citations", citations)?;
            }
            ToolResult::DeepResearch {
                summary,
                extracted_fields,
                citations,
            } => {
                if summary.trim().is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.deep_research.summary",
                        reason: "must not be empty",
                    });
                }
                if summary.len() > 1024 {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.deep_research.summary",
                        reason: "must be <= 1024 chars",
                    });
                }
                if extracted_fields.is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.deep_research.extracted_fields",
                        reason: "must not be empty",
                    });
                }
                if extracted_fields.len() > 20 {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.deep_research.extracted_fields",
                        reason: "must be <= 20 entries",
                    });
                }
                for field in extracted_fields {
                    field.validate()?;
                }
                validate_items("tool_result.deep_research.citations", citations)?;
            }
            ToolResult::RecordMode {
                summary,
                action_items,
                evidence_refs,
            } => {
                if summary.trim().is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.record_mode.summary",
                        reason: "must not be empty",
                    });
                }
                if summary.len() > 1024 {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.record_mode.summary",
                        reason: "must be <= 1024 chars",
                    });
                }
                if action_items.is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.record_mode.action_items",
                        reason: "must not be empty",
                    });
                }
                if action_items.len() > 20 {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.record_mode.action_items",
                        reason: "must be <= 20 entries",
                    });
                }
                for field in action_items {
                    field.validate()?;
                }
                if evidence_refs.is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.record_mode.evidence_refs",
                        reason: "must not be empty",
                    });
                }
                if evidence_refs.len() > 20 {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.record_mode.evidence_refs",
                        reason: "must be <= 20 entries",
                    });
                }
                for field in evidence_refs {
                    field.validate()?;
                }
            }
            ToolResult::ConnectorQuery {
                summary,
                extracted_fields,
                citations,
            } => {
                if summary.trim().is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.connector_query.summary",
                        reason: "must not be empty",
                    });
                }
                if summary.len() > 1024 {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.connector_query.summary",
                        reason: "must be <= 1024 chars",
                    });
                }
                if extracted_fields.is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.connector_query.extracted_fields",
                        reason: "must not be empty",
                    });
                }
                if extracted_fields.len() > 20 {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_result.connector_query.extracted_fields",
                        reason: "must be <= 20 entries",
                    });
                }
                for field in extracted_fields {
                    field.validate()?;
                }
                validate_items("tool_result.connector_query.citations", citations)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheStatus {
    Hit,
    Miss,
    Bypassed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolResponse {
    pub schema_version: SchemaVersion,
    pub request_id: ToolRequestId,
    pub query_hash: ToolQueryHash,
    pub tool_status: ToolStatus,
    pub tool_result: Option<ToolResult>,
    pub source_metadata: Option<SourceMetadata>,
    pub reason_code: ReasonCodeId,
    pub fail_reason_code: Option<ReasonCodeId>,
    pub fail_detail: Option<String>,
    pub ambiguity: Option<StructuredAmbiguity>,
    pub cache_status: CacheStatus,
}

impl ToolResponse {
    pub fn ok_v1(
        request_id: ToolRequestId,
        query_hash: ToolQueryHash,
        tool_result: ToolResult,
        source_metadata: SourceMetadata,
        ambiguity: Option<StructuredAmbiguity>,
        reason_code: ReasonCodeId,
        cache_status: CacheStatus,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1E_CONTRACT_VERSION,
            request_id,
            query_hash,
            tool_status: ToolStatus::Ok,
            tool_result: Some(tool_result),
            source_metadata: Some(source_metadata),
            reason_code,
            fail_reason_code: None,
            fail_detail: None,
            ambiguity,
            cache_status,
        };
        r.validate()?;
        Ok(r)
    }

    pub fn fail_v1(
        request_id: ToolRequestId,
        query_hash: ToolQueryHash,
        fail_reason_code: ReasonCodeId,
        cache_status: CacheStatus,
    ) -> Result<Self, ContractViolation> {
        Self::fail_with_detail_v1(request_id, query_hash, fail_reason_code, None, cache_status)
    }

    pub fn fail_with_detail_v1(
        request_id: ToolRequestId,
        query_hash: ToolQueryHash,
        fail_reason_code: ReasonCodeId,
        fail_detail: Option<String>,
        cache_status: CacheStatus,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1E_CONTRACT_VERSION,
            request_id,
            query_hash,
            tool_status: ToolStatus::Fail,
            tool_result: None,
            source_metadata: None,
            reason_code: fail_reason_code,
            fail_reason_code: Some(fail_reason_code),
            fail_detail,
            ambiguity: None,
            cache_status,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for ToolResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.request_id.validate()?;
        self.query_hash.validate()?;
        match self.tool_status {
            ToolStatus::Ok => {
                if self.tool_result.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_response.tool_result",
                        reason: "must be Some(...) when tool_status=OK",
                    });
                }
                if self.source_metadata.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_response.source_metadata",
                        reason: "must be Some(...) when tool_status=OK",
                    });
                }
                if self.fail_reason_code.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_response.fail_reason_code",
                        reason: "must be None when tool_status=OK",
                    });
                }
                if self.fail_detail.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_response.fail_detail",
                        reason: "must be None when tool_status=OK",
                    });
                }
                self.tool_result.as_ref().unwrap().validate()?;
                self.source_metadata.as_ref().unwrap().validate()?;
                if let Some(a) = &self.ambiguity {
                    a.validate()?;
                }
            }
            ToolStatus::Fail => {
                if self.fail_reason_code.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_response.fail_reason_code",
                        reason: "must be Some(...) when tool_status=FAIL",
                    });
                }
                if self.tool_result.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_response.tool_result",
                        reason: "must be None when tool_status=FAIL",
                    });
                }
                if self.source_metadata.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_response.source_metadata",
                        reason: "must be None when tool_status=FAIL",
                    });
                }
                if self.fail_reason_code != Some(self.reason_code) {
                    return Err(ContractViolation::InvalidValue {
                        field: "tool_response.reason_code",
                        reason: "must equal fail_reason_code when tool_status=FAIL",
                    });
                }
                if let Some(detail) = &self.fail_detail {
                    let trimmed = detail.trim();
                    if trimmed.is_empty() {
                        return Err(ContractViolation::InvalidValue {
                            field: "tool_response.fail_detail",
                            reason: "must not be empty when present",
                        });
                    }
                    if trimmed.len() > 256 {
                        return Err(ContractViolation::InvalidValue {
                            field: "tool_response.fail_detail",
                            reason: "must be <= 256 chars when present",
                        });
                    }
                    if trimmed.chars().any(|c| c.is_control()) {
                        return Err(ContractViolation::InvalidValue {
                            field: "tool_response.fail_detail",
                            reason: "must not contain control characters",
                        });
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_request_rejects_non_x_origin() {
        let req = ToolRequest::v1(
            ToolRequestOrigin::other("something").unwrap(),
            ToolName::Time,
            "now".to_string(),
            None,
            StrictBudget::new(1000, 3).unwrap(),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
        );
        assert!(req.is_err());
    }

    #[test]
    fn tool_response_fail_requires_reason_match() {
        let mut resp = ToolResponse::fail_v1(
            ToolRequestId(1),
            ToolQueryHash(1),
            ReasonCodeId(42),
            CacheStatus::Miss,
        )
        .unwrap();
        resp.reason_code = ReasonCodeId(43);
        assert!(resp.validate().is_err());
    }

    #[test]
    fn tool_response_ok_requires_result_and_metadata() {
        let mut resp = ToolResponse::ok_v1(
            ToolRequestId(1),
            ToolQueryHash(1),
            ToolResult::Time {
                local_time_iso: "2026-01-01T00:00:00Z".to_string(),
            },
            SourceMetadata {
                schema_version: PH1E_CONTRACT_VERSION,
                provider_hint: Some("stub".to_string()),
                retrieved_at_unix_ms: 1_700_000_000_000,
                sources: vec![SourceRef {
                    title: "Example".to_string(),
                    url: "https://example.invalid".to_string(),
                }],
                web_answer_verification: None,
            },
            None,
            ReasonCodeId(1),
            CacheStatus::Hit,
        )
        .unwrap();
        resp.source_metadata = None;
        assert!(resp.validate().is_err());
    }

    #[test]
    fn tool_catalog_rejects_other_tool_entries() {
        let catalog = ToolCatalogRef::v1(vec![ToolName::other("custom").unwrap()]);
        assert!(catalog.is_err());
    }
}
