#![forbid(unsafe_code)]

use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1SIMFINDER_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

pub mod reason_codes {
    use crate::ReasonCodeId;

    pub const SIM_FINDER_MATCH_OK: ReasonCodeId = ReasonCodeId(0x5346_0001);
    pub const SIM_FINDER_MATCH_OK_GOLD_BOOSTED: ReasonCodeId = ReasonCodeId(0x5346_0002);
    pub const SIM_FINDER_MATCH_OK_CATALOG_ACTIVE: ReasonCodeId = ReasonCodeId(0x5346_0003);

    pub const SIM_FINDER_CLARIFY_MISSING_FIELD: ReasonCodeId = ReasonCodeId(0x5346_0011);
    pub const SIM_FINDER_CLARIFY_AMBIGUOUS: ReasonCodeId = ReasonCodeId(0x5346_0012);
    pub const SIM_FINDER_CLARIFY_LOW_CONFIDENCE_TIE: ReasonCodeId = ReasonCodeId(0x5346_0013);
    pub const SIM_FINDER_ABSTAIN_LOW_CALIBRATED_CONFIDENCE: ReasonCodeId =
        ReasonCodeId(0x5346_0014);

    pub const SIM_FINDER_REFUSE_ACCESS_DENIED: ReasonCodeId = ReasonCodeId(0x5346_0021);
    pub const SIM_FINDER_REFUSE_ACCESS_AP_REQUIRED: ReasonCodeId = ReasonCodeId(0x5346_0022);
    pub const SIM_FINDER_REFUSE_UNSAFE_REQUEST: ReasonCodeId = ReasonCodeId(0x5346_0023);
    pub const SIM_FINDER_REFUSE_AMBIGUOUS: ReasonCodeId = ReasonCodeId(0x5346_0024);
    pub const SIM_FINDER_REFUSE_POLICY_BLOCKED: ReasonCodeId = ReasonCodeId(0x5346_0025);
    pub const SIM_FINDER_SIMULATION_INACTIVE: ReasonCodeId = ReasonCodeId(0x5346_0026);
    pub const SIM_FINDER_REPLAY_ARTIFACT_MISSING: ReasonCodeId = ReasonCodeId(0x5346_0027);

    pub const SIM_FINDER_MISSING_SIMULATION: ReasonCodeId = ReasonCodeId(0x5346_0031);
    pub const SIM_FINDER_MISSING_SIMULATION_DECLINED_LOW_VALUE_HIGH_RISK: ReasonCodeId =
        ReasonCodeId(0x5346_0032);
    pub const SIM_FINDER_MISSING_SIMULATION_RATE_LIMITED: ReasonCodeId = ReasonCodeId(0x5346_0033);
    pub const SIM_FINDER_MISSING_SIMULATION_DAILY_CAP_REACHED: ReasonCodeId =
        ReasonCodeId(0x5346_0034);
}

pub fn reason_code_label(code: ReasonCodeId) -> Option<&'static str> {
    match code {
        reason_codes::SIM_FINDER_MATCH_OK => Some("SIM_FINDER_MATCH_OK"),
        reason_codes::SIM_FINDER_MATCH_OK_GOLD_BOOSTED => Some("SIM_FINDER_MATCH_OK_GOLD_BOOSTED"),
        reason_codes::SIM_FINDER_MATCH_OK_CATALOG_ACTIVE => {
            Some("SIM_FINDER_MATCH_OK_CATALOG_ACTIVE")
        }
        reason_codes::SIM_FINDER_CLARIFY_MISSING_FIELD => Some("SIM_FINDER_CLARIFY_MISSING_FIELD"),
        reason_codes::SIM_FINDER_CLARIFY_AMBIGUOUS => Some("SIM_FINDER_CLARIFY_AMBIGUOUS"),
        reason_codes::SIM_FINDER_CLARIFY_LOW_CONFIDENCE_TIE => {
            Some("SIM_FINDER_CLARIFY_LOW_CONFIDENCE_TIE")
        }
        reason_codes::SIM_FINDER_ABSTAIN_LOW_CALIBRATED_CONFIDENCE => {
            Some("SIM_FINDER_ABSTAIN_LOW_CALIBRATED_CONFIDENCE")
        }
        reason_codes::SIM_FINDER_REFUSE_ACCESS_DENIED => Some("SIM_FINDER_REFUSE_ACCESS_DENIED"),
        reason_codes::SIM_FINDER_REFUSE_ACCESS_AP_REQUIRED => {
            Some("SIM_FINDER_REFUSE_ACCESS_AP_REQUIRED")
        }
        reason_codes::SIM_FINDER_REFUSE_UNSAFE_REQUEST => Some("SIM_FINDER_REFUSE_UNSAFE_REQUEST"),
        reason_codes::SIM_FINDER_REFUSE_AMBIGUOUS => Some("SIM_FINDER_REFUSE_AMBIGUOUS"),
        reason_codes::SIM_FINDER_REFUSE_POLICY_BLOCKED => Some("SIM_FINDER_REFUSE_POLICY_BLOCKED"),
        reason_codes::SIM_FINDER_SIMULATION_INACTIVE => Some("SIM_FINDER_SIMULATION_INACTIVE"),
        reason_codes::SIM_FINDER_REPLAY_ARTIFACT_MISSING => {
            Some("SIM_FINDER_REPLAY_ARTIFACT_MISSING")
        }
        reason_codes::SIM_FINDER_MISSING_SIMULATION => Some("SIM_FINDER_MISSING_SIMULATION"),
        reason_codes::SIM_FINDER_MISSING_SIMULATION_DECLINED_LOW_VALUE_HIGH_RISK => {
            Some("SIM_FINDER_MISSING_SIMULATION_DECLINED_LOW_VALUE_HIGH_RISK")
        }
        reason_codes::SIM_FINDER_MISSING_SIMULATION_RATE_LIMITED => {
            Some("SIM_FINDER_MISSING_SIMULATION_RATE_LIMITED")
        }
        reason_codes::SIM_FINDER_MISSING_SIMULATION_DAILY_CAP_REACHED => {
            Some("SIM_FINDER_MISSING_SIMULATION_DAILY_CAP_REACHED")
        }
        _ => None,
    }
}

fn validate_required_text(
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

fn validate_optional_text(
    field: &'static str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(v) = value {
        validate_required_text(field, v, max_len)?;
    }
    Ok(())
}

fn validate_bp(field: &'static str, value: u16) -> Result<(), ContractViolation> {
    if value > 10_000 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 10000 basis points",
        });
    }
    Ok(())
}

fn validate_list_non_empty(
    field: &'static str,
    values: &[String],
    max_items: usize,
    max_item_len: usize,
) -> Result<(), ContractViolation> {
    if values.len() > max_items {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max items",
        });
    }
    for value in values {
        if value.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "must not contain empty entries",
            });
        }
        if value.len() > max_item_len {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "contains oversized entry",
            });
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FinderPacketType {
    SimulationMatch,
    Clarify,
    Refuse,
    MissingSimulation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FinderRiskTier {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FinderFallbackPolicy {
    Clarify,
    MissingSimulation,
    Refuse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClarifyOnExceedPolicy {
    MissingSimulation,
    Refuse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogCheckKind {
    ActiveCheck,
    DraftCheck,
    NoneFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogCheckTraceEntry {
    pub kind: CatalogCheckKind,
    pub checked_at: MonotonicTimeNs,
    pub proof_ref: String,
}

impl Validate for CatalogCheckTraceEntry {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.checked_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "catalog_check_trace_entry.checked_at",
                reason: "must be > 0",
            });
        }
        validate_required_text("catalog_check_trace_entry.proof_ref", &self.proof_ref, 256)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationMatchPacket {
    pub packet_type: FinderPacketType,
    pub schema_version: SchemaVersion,
    pub tenant_id: String,
    pub user_id: String,
    pub correlation_id: u128,
    pub turn_id: u64,
    pub intent_family: String,
    pub simulation_id: String,
    pub candidate_rank: u8,
    pub confidence_bp: u16,
    pub required_fields_present: Vec<String>,
    pub required_fields_missing: Vec<String>,
    pub evidence_spans: Vec<String>,
    pub risk_tier: FinderRiskTier,
    pub confirm_required: bool,
    pub access_actions_required: Vec<String>,
    pub idempotency_key: String,
    pub idempotency_recipe_ref: String,
    pub fallback_if_inactive_or_missing: FinderFallbackPolicy,
    pub reason_code: ReasonCodeId,
}

impl SimulationMatchPacket {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        tenant_id: String,
        user_id: String,
        correlation_id: u128,
        turn_id: u64,
        intent_family: String,
        simulation_id: String,
        confidence_bp: u16,
        required_fields_present: Vec<String>,
        required_fields_missing: Vec<String>,
        evidence_spans: Vec<String>,
        risk_tier: FinderRiskTier,
        confirm_required: bool,
        access_actions_required: Vec<String>,
        idempotency_key: String,
        idempotency_recipe_ref: String,
        fallback_if_inactive_or_missing: FinderFallbackPolicy,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            packet_type: FinderPacketType::SimulationMatch,
            schema_version: PH1SIMFINDER_CONTRACT_VERSION,
            tenant_id,
            user_id,
            correlation_id,
            turn_id,
            intent_family,
            simulation_id,
            candidate_rank: 1,
            confidence_bp,
            required_fields_present,
            required_fields_missing,
            evidence_spans,
            risk_tier,
            confirm_required,
            access_actions_required,
            idempotency_key,
            idempotency_recipe_ref,
            fallback_if_inactive_or_missing,
            reason_code,
        };
        packet.validate()?;
        Ok(packet)
    }
}

impl Validate for SimulationMatchPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.packet_type != FinderPacketType::SimulationMatch {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_match_packet.packet_type",
                reason: "must be SimulationMatch",
            });
        }
        if self.schema_version != PH1SIMFINDER_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_match_packet.schema_version",
                reason: "must match PH1SIMFINDER_CONTRACT_VERSION",
            });
        }
        validate_required_text("simulation_match_packet.tenant_id", &self.tenant_id, 128)?;
        validate_required_text("simulation_match_packet.user_id", &self.user_id, 128)?;
        if self.correlation_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_match_packet.correlation_id",
                reason: "must be > 0",
            });
        }
        if self.turn_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_match_packet.turn_id",
                reason: "must be > 0",
            });
        }
        validate_required_text(
            "simulation_match_packet.intent_family",
            &self.intent_family,
            128,
        )?;
        validate_required_text(
            "simulation_match_packet.simulation_id",
            &self.simulation_id,
            128,
        )?;
        if self.candidate_rank != 1 {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_match_packet.candidate_rank",
                reason: "must equal 1",
            });
        }
        validate_bp("simulation_match_packet.confidence_bp", self.confidence_bp)?;
        validate_list_non_empty(
            "simulation_match_packet.required_fields_present",
            &self.required_fields_present,
            64,
            128,
        )?;
        validate_list_non_empty(
            "simulation_match_packet.required_fields_missing",
            &self.required_fields_missing,
            64,
            128,
        )?;
        validate_list_non_empty(
            "simulation_match_packet.evidence_spans",
            &self.evidence_spans,
            256,
            512,
        )?;
        validate_list_non_empty(
            "simulation_match_packet.access_actions_required",
            &self.access_actions_required,
            32,
            128,
        )?;
        validate_required_text(
            "simulation_match_packet.idempotency_key",
            &self.idempotency_key,
            256,
        )?;
        validate_required_text(
            "simulation_match_packet.idempotency_recipe_ref",
            &self.idempotency_recipe_ref,
            256,
        )?;
        if !is_reason_code_allowed_for_match(self.reason_code) {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_match_packet.reason_code",
                reason: "reason_code is not allowed for SimulationMatchPacket",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClarifyPacket {
    pub packet_type: FinderPacketType,
    pub schema_version: SchemaVersion,
    pub tenant_id: String,
    pub user_id: String,
    pub correlation_id: u128,
    pub turn_id: u64,
    pub question: String,
    pub missing_field: String,
    pub allowed_answer_formats: Vec<String>,
    pub attempt_index: u8,
    pub max_attempts: u8,
    pub on_exceed: ClarifyOnExceedPolicy,
    pub candidate_context_ref: String,
    pub idempotency_key: String,
    pub reason_code: ReasonCodeId,
}

impl ClarifyPacket {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        tenant_id: String,
        user_id: String,
        correlation_id: u128,
        turn_id: u64,
        question: String,
        missing_field: String,
        allowed_answer_formats: Vec<String>,
        attempt_index: u8,
        max_attempts: u8,
        on_exceed: ClarifyOnExceedPolicy,
        candidate_context_ref: String,
        idempotency_key: String,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            packet_type: FinderPacketType::Clarify,
            schema_version: PH1SIMFINDER_CONTRACT_VERSION,
            tenant_id,
            user_id,
            correlation_id,
            turn_id,
            question,
            missing_field,
            allowed_answer_formats,
            attempt_index,
            max_attempts,
            on_exceed,
            candidate_context_ref,
            idempotency_key,
            reason_code,
        };
        packet.validate()?;
        Ok(packet)
    }
}

impl Validate for ClarifyPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.packet_type != FinderPacketType::Clarify {
            return Err(ContractViolation::InvalidValue {
                field: "clarify_packet.packet_type",
                reason: "must be Clarify",
            });
        }
        if self.schema_version != PH1SIMFINDER_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "clarify_packet.schema_version",
                reason: "must match PH1SIMFINDER_CONTRACT_VERSION",
            });
        }
        validate_required_text("clarify_packet.tenant_id", &self.tenant_id, 128)?;
        validate_required_text("clarify_packet.user_id", &self.user_id, 128)?;
        if self.correlation_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "clarify_packet.correlation_id",
                reason: "must be > 0",
            });
        }
        if self.turn_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "clarify_packet.turn_id",
                reason: "must be > 0",
            });
        }
        validate_required_text("clarify_packet.question", &self.question, 240)?;
        validate_required_text("clarify_packet.missing_field", &self.missing_field, 128)?;
        if !(2..=3).contains(&self.allowed_answer_formats.len()) {
            return Err(ContractViolation::InvalidValue {
                field: "clarify_packet.allowed_answer_formats",
                reason: "must contain 2-3 entries",
            });
        }
        validate_list_non_empty(
            "clarify_packet.allowed_answer_formats",
            &self.allowed_answer_formats,
            3,
            128,
        )?;
        if self.attempt_index == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "clarify_packet.attempt_index",
                reason: "must be > 0",
            });
        }
        if self.max_attempts == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "clarify_packet.max_attempts",
                reason: "must be > 0",
            });
        }
        if self.attempt_index > self.max_attempts {
            return Err(ContractViolation::InvalidValue {
                field: "clarify_packet.attempt_index",
                reason: "must be <= max_attempts",
            });
        }
        validate_required_text(
            "clarify_packet.candidate_context_ref",
            &self.candidate_context_ref,
            256,
        )?;
        validate_required_text("clarify_packet.idempotency_key", &self.idempotency_key, 256)?;
        if !is_reason_code_allowed_for_clarify(self.reason_code) {
            return Err(ContractViolation::InvalidValue {
                field: "clarify_packet.reason_code",
                reason: "reason_code is not allowed for ClarifyPacket",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefusePacket {
    pub packet_type: FinderPacketType,
    pub schema_version: SchemaVersion,
    pub tenant_id: String,
    pub user_id: String,
    pub correlation_id: u128,
    pub turn_id: u64,
    pub reason_code: ReasonCodeId,
    pub message: String,
    pub evidence_refs: Vec<String>,
    pub existing_draft_ref: Option<String>,
}

impl RefusePacket {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        tenant_id: String,
        user_id: String,
        correlation_id: u128,
        turn_id: u64,
        reason_code: ReasonCodeId,
        message: String,
        evidence_refs: Vec<String>,
        existing_draft_ref: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            packet_type: FinderPacketType::Refuse,
            schema_version: PH1SIMFINDER_CONTRACT_VERSION,
            tenant_id,
            user_id,
            correlation_id,
            turn_id,
            reason_code,
            message,
            evidence_refs,
            existing_draft_ref,
        };
        packet.validate()?;
        Ok(packet)
    }
}

impl Validate for RefusePacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.packet_type != FinderPacketType::Refuse {
            return Err(ContractViolation::InvalidValue {
                field: "refuse_packet.packet_type",
                reason: "must be Refuse",
            });
        }
        if self.schema_version != PH1SIMFINDER_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "refuse_packet.schema_version",
                reason: "must match PH1SIMFINDER_CONTRACT_VERSION",
            });
        }
        validate_required_text("refuse_packet.tenant_id", &self.tenant_id, 128)?;
        validate_required_text("refuse_packet.user_id", &self.user_id, 128)?;
        if self.correlation_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "refuse_packet.correlation_id",
                reason: "must be > 0",
            });
        }
        if self.turn_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "refuse_packet.turn_id",
                reason: "must be > 0",
            });
        }
        if !is_reason_code_allowed_for_refuse(self.reason_code) {
            return Err(ContractViolation::InvalidValue {
                field: "refuse_packet.reason_code",
                reason: "reason_code is not allowed for RefusePacket",
            });
        }
        validate_required_text("refuse_packet.message", &self.message, 512)?;
        validate_list_non_empty("refuse_packet.evidence_refs", &self.evidence_refs, 64, 256)?;
        validate_optional_text(
            "refuse_packet.existing_draft_ref",
            &self.existing_draft_ref,
            256,
        )?;

        if self.reason_code == reason_codes::SIM_FINDER_SIMULATION_INACTIVE
            && self.existing_draft_ref.is_none()
        {
            return Err(ContractViolation::InvalidValue {
                field: "refuse_packet.existing_draft_ref",
                reason: "must be present when reason_code=SIM_FINDER_SIMULATION_INACTIVE",
            });
        }
        if self.reason_code != reason_codes::SIM_FINDER_SIMULATION_INACTIVE
            && self.existing_draft_ref.is_some()
        {
            return Err(ContractViolation::InvalidValue {
                field: "refuse_packet.existing_draft_ref",
                reason: "must be empty unless reason_code=SIM_FINDER_SIMULATION_INACTIVE",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingSimulationPacket {
    pub packet_type: FinderPacketType,
    pub schema_version: SchemaVersion,
    pub tenant_id: String,
    pub user_id: String,
    pub correlation_id: u128,
    pub turn_id: u64,
    pub requested_capability_name_normalized: String,
    pub raw_user_utterance: String,
    pub cleaned_paraphrase: String,
    pub category: String,
    pub estimated_frequency_score_bp: u16,
    pub estimated_value_score_bp: u16,
    pub estimated_roi_score_bp: u16,
    pub estimated_feasibility_score_bp: u16,
    pub estimated_risk_score_bp: u16,
    pub worthiness_score_bp: u16,
    pub scope_class: String,
    pub required_integrations: Vec<String>,
    pub proposed_simulation_family: String,
    pub required_fields_schema_json: String,
    pub acceptance_test_suggestion: Vec<String>,
    pub dedupe_fingerprint: String,
    pub catalog_check_trace: Vec<CatalogCheckTraceEntry>,
    pub active_check_proof_ref: String,
    pub draft_check_proof_ref: String,
    pub no_match_proof_ref: String,
    pub existing_draft_ref: Option<String>,
    pub idempotency_key: String,
    pub reason_code: ReasonCodeId,
}

impl MissingSimulationPacket {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        tenant_id: String,
        user_id: String,
        correlation_id: u128,
        turn_id: u64,
        requested_capability_name_normalized: String,
        raw_user_utterance: String,
        cleaned_paraphrase: String,
        category: String,
        estimated_frequency_score_bp: u16,
        estimated_value_score_bp: u16,
        estimated_roi_score_bp: u16,
        estimated_feasibility_score_bp: u16,
        estimated_risk_score_bp: u16,
        worthiness_score_bp: u16,
        scope_class: String,
        required_integrations: Vec<String>,
        proposed_simulation_family: String,
        required_fields_schema_json: String,
        acceptance_test_suggestion: Vec<String>,
        dedupe_fingerprint: String,
        catalog_check_trace: Vec<CatalogCheckTraceEntry>,
        active_check_proof_ref: String,
        draft_check_proof_ref: String,
        no_match_proof_ref: String,
        idempotency_key: String,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            packet_type: FinderPacketType::MissingSimulation,
            schema_version: PH1SIMFINDER_CONTRACT_VERSION,
            tenant_id,
            user_id,
            correlation_id,
            turn_id,
            requested_capability_name_normalized,
            raw_user_utterance,
            cleaned_paraphrase,
            category,
            estimated_frequency_score_bp,
            estimated_value_score_bp,
            estimated_roi_score_bp,
            estimated_feasibility_score_bp,
            estimated_risk_score_bp,
            worthiness_score_bp,
            scope_class,
            required_integrations,
            proposed_simulation_family,
            required_fields_schema_json,
            acceptance_test_suggestion,
            dedupe_fingerprint,
            catalog_check_trace,
            active_check_proof_ref,
            draft_check_proof_ref,
            no_match_proof_ref,
            existing_draft_ref: None,
            idempotency_key,
            reason_code,
        };
        packet.validate()?;
        Ok(packet)
    }
}

impl Validate for MissingSimulationPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.packet_type != FinderPacketType::MissingSimulation {
            return Err(ContractViolation::InvalidValue {
                field: "missing_simulation_packet.packet_type",
                reason: "must be MissingSimulation",
            });
        }
        if self.schema_version != PH1SIMFINDER_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "missing_simulation_packet.schema_version",
                reason: "must match PH1SIMFINDER_CONTRACT_VERSION",
            });
        }
        validate_required_text("missing_simulation_packet.tenant_id", &self.tenant_id, 128)?;
        validate_required_text("missing_simulation_packet.user_id", &self.user_id, 128)?;
        if self.correlation_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "missing_simulation_packet.correlation_id",
                reason: "must be > 0",
            });
        }
        if self.turn_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "missing_simulation_packet.turn_id",
                reason: "must be > 0",
            });
        }
        validate_required_text(
            "missing_simulation_packet.requested_capability_name_normalized",
            &self.requested_capability_name_normalized,
            128,
        )?;
        validate_required_text(
            "missing_simulation_packet.raw_user_utterance",
            &self.raw_user_utterance,
            32_768,
        )?;
        validate_required_text(
            "missing_simulation_packet.cleaned_paraphrase",
            &self.cleaned_paraphrase,
            32_768,
        )?;
        validate_required_text("missing_simulation_packet.category", &self.category, 128)?;
        validate_bp(
            "missing_simulation_packet.estimated_frequency_score_bp",
            self.estimated_frequency_score_bp,
        )?;
        validate_bp(
            "missing_simulation_packet.estimated_value_score_bp",
            self.estimated_value_score_bp,
        )?;
        validate_bp(
            "missing_simulation_packet.estimated_roi_score_bp",
            self.estimated_roi_score_bp,
        )?;
        validate_bp(
            "missing_simulation_packet.estimated_feasibility_score_bp",
            self.estimated_feasibility_score_bp,
        )?;
        validate_bp(
            "missing_simulation_packet.estimated_risk_score_bp",
            self.estimated_risk_score_bp,
        )?;
        validate_bp(
            "missing_simulation_packet.worthiness_score_bp",
            self.worthiness_score_bp,
        )?;
        validate_required_text(
            "missing_simulation_packet.scope_class",
            &self.scope_class,
            64,
        )?;
        validate_list_non_empty(
            "missing_simulation_packet.required_integrations",
            &self.required_integrations,
            64,
            128,
        )?;
        validate_required_text(
            "missing_simulation_packet.proposed_simulation_family",
            &self.proposed_simulation_family,
            128,
        )?;
        validate_required_text(
            "missing_simulation_packet.required_fields_schema_json",
            &self.required_fields_schema_json,
            65_536,
        )?;
        validate_list_non_empty(
            "missing_simulation_packet.acceptance_test_suggestion",
            &self.acceptance_test_suggestion,
            64,
            256,
        )?;
        validate_required_text(
            "missing_simulation_packet.dedupe_fingerprint",
            &self.dedupe_fingerprint,
            128,
        )?;
        if self.catalog_check_trace.is_empty() || self.catalog_check_trace.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "missing_simulation_packet.catalog_check_trace",
                reason: "must contain 1-8 entries",
            });
        }
        for entry in &self.catalog_check_trace {
            entry.validate()?;
        }
        validate_required_text(
            "missing_simulation_packet.active_check_proof_ref",
            &self.active_check_proof_ref,
            256,
        )?;
        validate_required_text(
            "missing_simulation_packet.draft_check_proof_ref",
            &self.draft_check_proof_ref,
            256,
        )?;
        validate_required_text(
            "missing_simulation_packet.no_match_proof_ref",
            &self.no_match_proof_ref,
            256,
        )?;
        if self.existing_draft_ref.is_some() {
            return Err(ContractViolation::InvalidValue {
                field: "missing_simulation_packet.existing_draft_ref",
                reason: "must be None for canonical MissingSimulationPacket flow",
            });
        }
        validate_required_text(
            "missing_simulation_packet.idempotency_key",
            &self.idempotency_key,
            256,
        )?;
        if !is_reason_code_allowed_for_missing_simulation(self.reason_code) {
            return Err(ContractViolation::InvalidValue {
                field: "missing_simulation_packet.reason_code",
                reason: "reason_code is not allowed for MissingSimulationPacket",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinderTerminalPacket {
    SimulationMatch(SimulationMatchPacket),
    Clarify(ClarifyPacket),
    Refuse(RefusePacket),
    MissingSimulation(MissingSimulationPacket),
}

impl Validate for FinderTerminalPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            FinderTerminalPacket::SimulationMatch(packet) => packet.validate(),
            FinderTerminalPacket::Clarify(packet) => packet.validate(),
            FinderTerminalPacket::Refuse(packet) => packet.validate(),
            FinderTerminalPacket::MissingSimulation(packet) => packet.validate(),
        }
    }
}

pub fn is_reason_code_allowed_for_match(reason_code: ReasonCodeId) -> bool {
    matches!(
        reason_code,
        reason_codes::SIM_FINDER_MATCH_OK
            | reason_codes::SIM_FINDER_MATCH_OK_GOLD_BOOSTED
            | reason_codes::SIM_FINDER_MATCH_OK_CATALOG_ACTIVE
    )
}

pub fn is_reason_code_allowed_for_clarify(reason_code: ReasonCodeId) -> bool {
    matches!(
        reason_code,
        reason_codes::SIM_FINDER_CLARIFY_MISSING_FIELD
            | reason_codes::SIM_FINDER_CLARIFY_AMBIGUOUS
            | reason_codes::SIM_FINDER_CLARIFY_LOW_CONFIDENCE_TIE
            | reason_codes::SIM_FINDER_ABSTAIN_LOW_CALIBRATED_CONFIDENCE
    )
}

pub fn is_reason_code_allowed_for_refuse(reason_code: ReasonCodeId) -> bool {
    matches!(
        reason_code,
        reason_codes::SIM_FINDER_REFUSE_ACCESS_DENIED
            | reason_codes::SIM_FINDER_REFUSE_ACCESS_AP_REQUIRED
            | reason_codes::SIM_FINDER_REFUSE_UNSAFE_REQUEST
            | reason_codes::SIM_FINDER_REFUSE_AMBIGUOUS
            | reason_codes::SIM_FINDER_REFUSE_POLICY_BLOCKED
            | reason_codes::SIM_FINDER_SIMULATION_INACTIVE
            | reason_codes::SIM_FINDER_REPLAY_ARTIFACT_MISSING
    )
}

pub fn is_reason_code_allowed_for_missing_simulation(reason_code: ReasonCodeId) -> bool {
    matches!(
        reason_code,
        reason_codes::SIM_FINDER_MISSING_SIMULATION
            | reason_codes::SIM_FINDER_MISSING_SIMULATION_DECLINED_LOW_VALUE_HIGH_RISK
            | reason_codes::SIM_FINDER_MISSING_SIMULATION_RATE_LIMITED
            | reason_codes::SIM_FINDER_MISSING_SIMULATION_DAILY_CAP_REACHED
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_simfinder_01_match_packet_v1_validates() {
        let packet = SimulationMatchPacket::v1(
            "tenant_1".to_string(),
            "user_1".to_string(),
            11,
            22,
            "scheduling".to_string(),
            "PH1.REM.001".to_string(),
            9_001,
            vec!["when".to_string()],
            Vec::new(),
            vec!["span:when".to_string()],
            FinderRiskTier::Low,
            false,
            vec!["REMINDER_SCHEDULE".to_string()],
            "idemp_key".to_string(),
            "sim_match:recipe".to_string(),
            FinderFallbackPolicy::Clarify,
            reason_codes::SIM_FINDER_MATCH_OK,
        )
        .expect("packet should validate");
        packet.validate().expect("packet must stay valid");
    }

    #[test]
    fn at_simfinder_02_clarify_packet_requires_allowed_reason_code() {
        let err = ClarifyPacket::v1(
            "tenant_1".to_string(),
            "user_1".to_string(),
            11,
            22,
            "What time should I use?".to_string(),
            "when".to_string(),
            vec![
                "at 3 PM".to_string(),
                "tomorrow morning".to_string(),
                "2026-03-01T15:00:00+08:00".to_string(),
            ],
            1,
            2,
            ClarifyOnExceedPolicy::MissingSimulation,
            "cand_ctx_1".to_string(),
            "idemp".to_string(),
            reason_codes::SIM_FINDER_MISSING_SIMULATION,
        )
        .expect_err("wrong reason code must fail");
        assert!(matches!(
            err,
            ContractViolation::InvalidValue {
                field: "clarify_packet.reason_code",
                ..
            }
        ));
    }

    #[test]
    fn at_simfinder_03_refuse_packet_requires_draft_ref_on_inactive() {
        let err = RefusePacket::v1(
            "tenant_1".to_string(),
            "user_1".to_string(),
            11,
            22,
            reason_codes::SIM_FINDER_SIMULATION_INACTIVE,
            "draft exists".to_string(),
            vec!["draft:ref".to_string()],
            None,
        )
        .expect_err("inactive require draft ref");
        assert!(matches!(
            err,
            ContractViolation::InvalidValue {
                field: "refuse_packet.existing_draft_ref",
                ..
            }
        ));
    }

    #[test]
    fn at_simfinder_04_missing_sim_packet_forbids_existing_draft_ref() {
        let mut packet = MissingSimulationPacket::v1(
            "tenant_1".to_string(),
            "user_1".to_string(),
            11,
            22,
            "order_pizza".to_string(),
            "order pizza please".to_string(),
            "order pizza".to_string(),
            "food_order".to_string(),
            4_000,
            6_000,
            5_000,
            4_500,
            2_000,
            5_000,
            "tenant_only".to_string(),
            vec!["dominos".to_string()],
            "food_ordering".to_string(),
            "{\"required\":[\"store\",\"item\"]}".to_string(),
            vec!["AT-FOOD-ORDER-01".to_string()],
            "dedupe_hash".to_string(),
            vec![
                CatalogCheckTraceEntry {
                    kind: CatalogCheckKind::ActiveCheck,
                    checked_at: MonotonicTimeNs(1),
                    proof_ref: "active:proof".to_string(),
                },
                CatalogCheckTraceEntry {
                    kind: CatalogCheckKind::DraftCheck,
                    checked_at: MonotonicTimeNs(2),
                    proof_ref: "draft:proof".to_string(),
                },
                CatalogCheckTraceEntry {
                    kind: CatalogCheckKind::NoneFound,
                    checked_at: MonotonicTimeNs(3),
                    proof_ref: "none:proof".to_string(),
                },
            ],
            "active:proof".to_string(),
            "draft:proof".to_string(),
            "none:proof".to_string(),
            "idemp".to_string(),
            reason_codes::SIM_FINDER_MISSING_SIMULATION,
        )
        .expect("valid missing packet");
        packet.existing_draft_ref = Some("draft_should_not_exist".to_string());
        let err = packet
            .validate()
            .expect_err("missing packet must reject existing_draft_ref");
        assert!(matches!(
            err,
            ContractViolation::InvalidValue {
                field: "missing_simulation_packet.existing_draft_ref",
                ..
            }
        ));
    }

    #[test]
    fn at_simfinder_05_reason_code_registry_is_complete_for_known_codes() {
        let codes = [
            reason_codes::SIM_FINDER_MATCH_OK,
            reason_codes::SIM_FINDER_MATCH_OK_GOLD_BOOSTED,
            reason_codes::SIM_FINDER_MATCH_OK_CATALOG_ACTIVE,
            reason_codes::SIM_FINDER_CLARIFY_MISSING_FIELD,
            reason_codes::SIM_FINDER_CLARIFY_AMBIGUOUS,
            reason_codes::SIM_FINDER_CLARIFY_LOW_CONFIDENCE_TIE,
            reason_codes::SIM_FINDER_ABSTAIN_LOW_CALIBRATED_CONFIDENCE,
            reason_codes::SIM_FINDER_REFUSE_ACCESS_DENIED,
            reason_codes::SIM_FINDER_REFUSE_ACCESS_AP_REQUIRED,
            reason_codes::SIM_FINDER_REFUSE_UNSAFE_REQUEST,
            reason_codes::SIM_FINDER_REFUSE_AMBIGUOUS,
            reason_codes::SIM_FINDER_REFUSE_POLICY_BLOCKED,
            reason_codes::SIM_FINDER_SIMULATION_INACTIVE,
            reason_codes::SIM_FINDER_REPLAY_ARTIFACT_MISSING,
            reason_codes::SIM_FINDER_MISSING_SIMULATION,
            reason_codes::SIM_FINDER_MISSING_SIMULATION_DECLINED_LOW_VALUE_HIGH_RISK,
            reason_codes::SIM_FINDER_MISSING_SIMULATION_RATE_LIMITED,
            reason_codes::SIM_FINDER_MISSING_SIMULATION_DAILY_CAP_REACHED,
        ];
        for code in codes {
            assert!(reason_code_label(code).is_some());
        }
        assert!(reason_code_label(ReasonCodeId(0xDEAD_BEEF)).is_none());
    }
}
