#![forbid(unsafe_code)]

use crate::ph1_voice_id::Ph1VoiceIdResponse;
use crate::ph1d::PolicyContextRef;
use crate::ph1e::ToolResponse;
use crate::ph1k::InterruptCandidate;
use crate::ph1l::SessionId;
use crate::ph1m::MemoryCandidate;
use crate::ph1n::Ph1nResponse;
use crate::ph1x::{ConfirmAnswer, ThreadState};
use crate::{
    ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, SessionState, Validate,
};

pub const PH1AGENT_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, PartialEq)]
pub struct AgentInputPacket {
    pub schema_version: SchemaVersion,
    pub correlation_id: u128,
    pub turn_id: u64,
    pub now: MonotonicTimeNs,
    pub trace_id: String,
    pub packet_hash: String,
    pub transcript_text: Option<String>,
    pub language_hint: Option<String>,
    pub srl_repaired_transcript: Option<String>,
    pub voice_identity_assertion: Ph1VoiceIdResponse,
    pub identity_prompt_scope_key: Option<String>,
    pub session_id: Option<SessionId>,
    pub session_state: SessionState,
    pub thread_key: Option<String>,
    pub thread_state: ThreadState,
    pub policy_context_ref: PolicyContextRef,
    pub memory_candidates: Vec<MemoryCandidate>,
    pub confirm_answer: Option<ConfirmAnswer>,
    pub nlp_output: Option<Ph1nResponse>,
    pub tool_response: Option<ToolResponse>,
    pub interruption: Option<InterruptCandidate>,
    pub last_failure_reason_code: Option<ReasonCodeId>,
    pub tenant_vocab_pack_ref: Option<String>,
    pub gold_mapping_pack_ref: Option<String>,
    pub sim_catalog_snapshot_hash: String,
    pub sim_catalog_snapshot_version: u64,
}

impl AgentInputPacket {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: u128,
        turn_id: u64,
        now: MonotonicTimeNs,
        trace_id: String,
        packet_hash: String,
        transcript_text: Option<String>,
        language_hint: Option<String>,
        srl_repaired_transcript: Option<String>,
        voice_identity_assertion: Ph1VoiceIdResponse,
        identity_prompt_scope_key: Option<String>,
        session_id: Option<SessionId>,
        session_state: SessionState,
        thread_key: Option<String>,
        thread_state: ThreadState,
        policy_context_ref: PolicyContextRef,
        memory_candidates: Vec<MemoryCandidate>,
        confirm_answer: Option<ConfirmAnswer>,
        nlp_output: Option<Ph1nResponse>,
        tool_response: Option<ToolResponse>,
        interruption: Option<InterruptCandidate>,
        last_failure_reason_code: Option<ReasonCodeId>,
        tenant_vocab_pack_ref: Option<String>,
        gold_mapping_pack_ref: Option<String>,
        sim_catalog_snapshot_hash: String,
        sim_catalog_snapshot_version: u64,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            schema_version: PH1AGENT_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            trace_id,
            packet_hash,
            transcript_text,
            language_hint,
            srl_repaired_transcript,
            voice_identity_assertion,
            identity_prompt_scope_key,
            session_id,
            session_state,
            thread_key,
            thread_state,
            policy_context_ref,
            memory_candidates,
            confirm_answer,
            nlp_output,
            tool_response,
            interruption,
            last_failure_reason_code,
            tenant_vocab_pack_ref,
            gold_mapping_pack_ref,
            sim_catalog_snapshot_hash,
            sim_catalog_snapshot_version,
        };
        packet.validate()?;
        Ok(packet)
    }
}

fn validate_optional_text(
    field: &'static str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(text) = value {
        if text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "must not be empty when provided",
            });
        }
        if text.len() > max_len {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "exceeds max length",
            });
        }
    }
    Ok(())
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

impl Validate for AgentInputPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1AGENT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "agent_input_packet.schema_version",
                reason: "must match PH1AGENT_CONTRACT_VERSION",
            });
        }
        if self.correlation_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "agent_input_packet.correlation_id",
                reason: "must be > 0",
            });
        }
        if self.turn_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "agent_input_packet.turn_id",
                reason: "must be > 0",
            });
        }
        if self.now.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "agent_input_packet.now",
                reason: "must be > 0",
            });
        }
        validate_required_text("agent_input_packet.trace_id", &self.trace_id, 128)?;
        validate_required_text("agent_input_packet.packet_hash", &self.packet_hash, 128)?;
        validate_optional_text(
            "agent_input_packet.transcript_text",
            &self.transcript_text,
            32_768,
        )?;
        validate_optional_text("agent_input_packet.language_hint", &self.language_hint, 64)?;
        validate_optional_text(
            "agent_input_packet.srl_repaired_transcript",
            &self.srl_repaired_transcript,
            32_768,
        )?;
        validate_optional_text(
            "agent_input_packet.identity_prompt_scope_key",
            &self.identity_prompt_scope_key,
            128,
        )?;
        validate_optional_text("agent_input_packet.thread_key", &self.thread_key, 128)?;
        validate_optional_text(
            "agent_input_packet.tenant_vocab_pack_ref",
            &self.tenant_vocab_pack_ref,
            128,
        )?;
        validate_optional_text(
            "agent_input_packet.gold_mapping_pack_ref",
            &self.gold_mapping_pack_ref,
            128,
        )?;
        validate_required_text(
            "agent_input_packet.sim_catalog_snapshot_hash",
            &self.sim_catalog_snapshot_hash,
            128,
        )?;
        self.voice_identity_assertion.validate()?;
        self.thread_state.validate()?;
        self.policy_context_ref.validate()?;
        let _ = &self.nlp_output;
        let _ = &self.tool_response;
        let _ = &self.interruption;
        for candidate in &self.memory_candidates {
            candidate.validate()?;
        }
        Ok(())
    }
}
