#![forbid(unsafe_code)]

use crate::ph1c::{SessionStateRef, TranscriptOk};
use crate::ph1e::ToolCatalogRef;
use crate::ph1n::{EvidenceSpan, FieldKey, FieldValue, IntentType, Ph1nResponse, TranscriptHash};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1D_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);
pub const PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_VERSION: SchemaVersion = SchemaVersion(1);
pub const PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1: SchemaHash =
    SchemaHash(0xD001_0001_0000_0001);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ph1dProviderTask {
    OcrTextExtract,
    SttTranscribe,
    TtsSynthesize,
}

impl Ph1dProviderTask {
    pub const fn as_str(self) -> &'static str {
        match self {
            Ph1dProviderTask::OcrTextExtract => "OCR_TEXT_EXTRACT",
            Ph1dProviderTask::SttTranscribe => "STT_TRANSCRIBE",
            Ph1dProviderTask::TtsSynthesize => "TTS_SYNTHESIZE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ph1dProviderRouteClass {
    Primary,
    Secondary,
    Tertiary,
}

impl Ph1dProviderRouteClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Ph1dProviderRouteClass::Primary => "PRIMARY",
            Ph1dProviderRouteClass::Secondary => "SECONDARY",
            Ph1dProviderRouteClass::Tertiary => "TERTIARY",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ph1dProviderInputPayloadKind {
    Image,
    Document,
    Audio,
    Text,
}

impl Ph1dProviderInputPayloadKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Ph1dProviderInputPayloadKind::Image => "IMAGE",
            Ph1dProviderInputPayloadKind::Document => "DOCUMENT",
            Ph1dProviderInputPayloadKind::Audio => "AUDIO",
            Ph1dProviderInputPayloadKind::Text => "TEXT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ph1dProviderStatus {
    Ok,
    Error,
}

impl Ph1dProviderStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Ph1dProviderStatus::Ok => "OK",
            Ph1dProviderStatus::Error => "ERROR",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ph1dProviderValidationStatus {
    SchemaOk,
    SchemaFail,
}

impl Ph1dProviderValidationStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Ph1dProviderValidationStatus::SchemaOk => "SCHEMA_OK",
            Ph1dProviderValidationStatus::SchemaFail => "SCHEMA_FAIL",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1dProviderNormalizedOutput {
    pub schema_version: SchemaVersion,
    pub provider_task: Ph1dProviderTask,
    pub text_output: Option<String>,
    pub language_tag: Option<String>,
    pub confidence_bp: Option<u16>,
    pub stable: Option<bool>,
    pub audio_output_ref: Option<String>,
    pub audio_content_type: Option<String>,
    pub estimated_duration_ms: Option<u32>,
}

impl Ph1dProviderNormalizedOutput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        provider_task: Ph1dProviderTask,
        text_output: Option<String>,
        language_tag: Option<String>,
        confidence_bp: Option<u16>,
        stable: Option<bool>,
        audio_output_ref: Option<String>,
        audio_content_type: Option<String>,
        estimated_duration_ms: Option<u32>,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_VERSION,
            provider_task,
            text_output,
            language_tag,
            confidence_bp,
            stable,
            audio_output_ref,
            audio_content_type,
            estimated_duration_ms,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for Ph1dProviderNormalizedOutput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_provider_normalized_output.schema_version",
                reason: "must match PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_VERSION",
            });
        }
        if let Some(text_output) = &self.text_output {
            if text_output.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_provider_normalized_output.text_output",
                    reason: "must not be empty when present",
                });
            }
            if text_output.len() > 65_536 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_provider_normalized_output.text_output",
                    reason: "must be <= 65536 chars",
                });
            }
        }
        if let Some(language_tag) = &self.language_tag {
            validate_language_tag("ph1d_provider_normalized_output.language_tag", language_tag)?;
        }
        if let Some(confidence_bp) = self.confidence_bp {
            if confidence_bp > 10_000 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_provider_normalized_output.confidence_bp",
                    reason: "must be <= 10000",
                });
            }
        }
        validate_opt_provider_token(
            "ph1d_provider_normalized_output.audio_output_ref",
            &self.audio_output_ref,
            256,
        )?;
        if let Some(audio_content_type) = &self.audio_content_type {
            if audio_content_type.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_provider_normalized_output.audio_content_type",
                    reason: "must not be empty when present",
                });
            }
            if audio_content_type.len() > 64 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_provider_normalized_output.audio_content_type",
                    reason: "must be <= 64 chars",
                });
            }
            if !audio_content_type
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '+' | '.'))
            {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_provider_normalized_output.audio_content_type",
                    reason: "contains unsupported characters",
                });
            }
        }
        if let Some(estimated_duration_ms) = self.estimated_duration_ms {
            if estimated_duration_ms == 0 || estimated_duration_ms > 600_000 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_provider_normalized_output.estimated_duration_ms",
                    reason: "must be within 1..=600000",
                });
            }
        }

        match self.provider_task {
            Ph1dProviderTask::OcrTextExtract => {
                if self.text_output.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "ph1d_provider_normalized_output.text_output",
                        reason: "must be Some(...) for OCR_TEXT_EXTRACT",
                    });
                }
                if self.audio_output_ref.is_some()
                    || self.audio_content_type.is_some()
                    || self.estimated_duration_ms.is_some()
                    || self.stable.is_some()
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "ph1d_provider_normalized_output.provider_task",
                        reason: "OCR_TEXT_EXTRACT must not include audio/stable fields",
                    });
                }
            }
            Ph1dProviderTask::SttTranscribe => {
                if self.text_output.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "ph1d_provider_normalized_output.text_output",
                        reason: "must be Some(...) for STT_TRANSCRIBE",
                    });
                }
                if self.language_tag.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "ph1d_provider_normalized_output.language_tag",
                        reason: "must be Some(...) for STT_TRANSCRIBE",
                    });
                }
                if self.confidence_bp.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "ph1d_provider_normalized_output.confidence_bp",
                        reason: "must be Some(...) for STT_TRANSCRIBE",
                    });
                }
                if self.stable.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "ph1d_provider_normalized_output.stable",
                        reason: "must be Some(...) for STT_TRANSCRIBE",
                    });
                }
                if self.audio_output_ref.is_some()
                    || self.audio_content_type.is_some()
                    || self.estimated_duration_ms.is_some()
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "ph1d_provider_normalized_output.provider_task",
                        reason: "STT_TRANSCRIBE must not include audio output fields",
                    });
                }
            }
            Ph1dProviderTask::TtsSynthesize => {
                if self.text_output.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "ph1d_provider_normalized_output.text_output",
                        reason: "must be Some(...) for TTS_SYNTHESIZE",
                    });
                }
                if self.audio_output_ref.is_none()
                    || self.audio_content_type.is_none()
                    || self.estimated_duration_ms.is_none()
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "ph1d_provider_normalized_output.provider_task",
                        reason: "TTS_SYNTHESIZE requires audio_output_ref/audio_content_type/estimated_duration_ms",
                    });
                }
                if self.stable.is_some() || self.confidence_bp.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "ph1d_provider_normalized_output.provider_task",
                        reason: "TTS_SYNTHESIZE must not include stable/confidence fields",
                    });
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1dProviderCallRequest {
    pub schema_version: SchemaVersion,
    pub correlation_id: u64,
    pub turn_id: u64,
    pub tenant_id: String,
    pub request_id: RequestId,
    pub idempotency_key: String,
    pub provider_task: Ph1dProviderTask,
    pub provider_route_class: Ph1dProviderRouteClass,
    pub provider_id: String,
    pub model_id: String,
    pub timeout_ms: u32,
    pub retry_budget: u8,
    pub prompt_template_ref: Option<String>,
    pub transcript_ref: Option<String>,
    pub output_schema_version: SchemaVersion,
    pub output_schema_hash: SchemaHash,
    pub tool_catalog_hash: SchemaHash,
    pub policy_context_hash: SchemaHash,
    pub transcript_hash: Option<TranscriptHash>,
    pub input_payload_ref: String,
    pub input_payload_kind: Ph1dProviderInputPayloadKind,
    pub input_payload_hash: SchemaHash,
    pub input_payload_inline: Option<String>,
    pub input_content_type: Option<String>,
    pub safety_tier: SafetyTier,
    pub privacy_mode: bool,
    pub do_not_disturb: bool,
}

impl Ph1dProviderCallRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: u64,
        turn_id: u64,
        tenant_id: String,
        request_id: RequestId,
        idempotency_key: String,
        provider_task: Ph1dProviderTask,
        provider_route_class: Ph1dProviderRouteClass,
        provider_id: String,
        model_id: String,
        timeout_ms: u32,
        retry_budget: u8,
        prompt_template_ref: Option<String>,
        transcript_ref: Option<String>,
        output_schema_version: SchemaVersion,
        output_schema_hash: SchemaHash,
        tool_catalog_hash: SchemaHash,
        policy_context_hash: SchemaHash,
        transcript_hash: Option<TranscriptHash>,
        input_payload_ref: String,
        input_payload_kind: Ph1dProviderInputPayloadKind,
        input_payload_hash: SchemaHash,
        input_payload_inline: Option<String>,
        input_content_type: Option<String>,
        safety_tier: SafetyTier,
        privacy_mode: bool,
        do_not_disturb: bool,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1D_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            tenant_id,
            request_id,
            idempotency_key,
            provider_task,
            provider_route_class,
            provider_id,
            model_id,
            timeout_ms,
            retry_budget,
            prompt_template_ref,
            transcript_ref,
            output_schema_version,
            output_schema_hash,
            tool_catalog_hash,
            policy_context_hash,
            transcript_hash,
            input_payload_ref,
            input_payload_kind,
            input_payload_hash,
            input_payload_inline,
            input_content_type,
            safety_tier,
            privacy_mode,
            do_not_disturb,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for Ph1dProviderCallRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1D_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_provider_call_request.schema_version",
                reason: "must match PH1D_CONTRACT_VERSION",
            });
        }
        if self.correlation_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_provider_call_request.correlation_id",
                reason: "must be > 0",
            });
        }
        if self.turn_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_provider_call_request.turn_id",
                reason: "must be > 0",
            });
        }
        self.request_id.validate()?;
        validate_provider_token("ph1d_provider_call_request.tenant_id", &self.tenant_id, 128)?;
        validate_provider_token(
            "ph1d_provider_call_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        validate_provider_token(
            "ph1d_provider_call_request.provider_id",
            &self.provider_id,
            64,
        )?;
        validate_provider_token("ph1d_provider_call_request.model_id", &self.model_id, 128)?;
        validate_opt_provider_token(
            "ph1d_provider_call_request.prompt_template_ref",
            &self.prompt_template_ref,
            128,
        )?;
        validate_opt_provider_token(
            "ph1d_provider_call_request.transcript_ref",
            &self.transcript_ref,
            128,
        )?;
        validate_provider_token(
            "ph1d_provider_call_request.input_payload_ref",
            &self.input_payload_ref,
            256,
        )?;
        validate_opt_provider_token(
            "ph1d_provider_call_request.input_content_type",
            &self.input_content_type,
            64,
        )?;
        if !(100..=120_000).contains(&self.timeout_ms) {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_provider_call_request.timeout_ms",
                reason: "must be within 100..=120000",
            });
        }
        if self.retry_budget > 10 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_provider_call_request.retry_budget",
                reason: "must be <= 10",
            });
        }
        if self.output_schema_version.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_provider_call_request.output_schema_version",
                reason: "must be > 0",
            });
        }
        self.output_schema_hash.validate()?;
        self.tool_catalog_hash.validate()?;
        self.policy_context_hash.validate()?;
        self.input_payload_hash.validate()?;
        if let Some(input_payload_inline) = &self.input_payload_inline {
            if input_payload_inline.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_provider_call_request.input_payload_inline",
                    reason: "must not be empty when present",
                });
            }
            if input_payload_inline.len() > 262_144 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_provider_call_request.input_payload_inline",
                    reason: "must be <= 262144 chars",
                });
            }
            if self.input_content_type.is_none() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_provider_call_request.input_content_type",
                    reason: "must be Some(...) when input_payload_inline is present",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1dProviderCallResponse {
    pub schema_version: SchemaVersion,
    pub correlation_id: u64,
    pub turn_id: u64,
    pub request_id: RequestId,
    pub idempotency_key: String,
    pub provider_call_id: Option<String>,
    pub provider_id: String,
    pub provider_task: Ph1dProviderTask,
    pub model_id: String,
    pub provider_status: Ph1dProviderStatus,
    pub provider_latency_ms: u32,
    pub provider_cost_microunits: u64,
    pub provider_confidence_bp: Option<u16>,
    pub normalized_output_schema_hash: Option<SchemaHash>,
    pub normalized_output_json: Option<String>,
    pub validation_status: Ph1dProviderValidationStatus,
    pub reason_code: ReasonCodeId,
}

impl Ph1dProviderCallResponse {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: u64,
        turn_id: u64,
        request_id: RequestId,
        idempotency_key: String,
        provider_call_id: Option<String>,
        provider_id: String,
        provider_task: Ph1dProviderTask,
        model_id: String,
        provider_status: Ph1dProviderStatus,
        provider_latency_ms: u32,
        provider_cost_microunits: u64,
        provider_confidence_bp: Option<u16>,
        normalized_output_schema_hash: Option<SchemaHash>,
        normalized_output_json: Option<String>,
        validation_status: Ph1dProviderValidationStatus,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1D_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            request_id,
            idempotency_key,
            provider_call_id,
            provider_id,
            provider_task,
            model_id,
            provider_status,
            provider_latency_ms,
            provider_cost_microunits,
            provider_confidence_bp,
            normalized_output_schema_hash,
            normalized_output_json,
            validation_status,
            reason_code,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for Ph1dProviderCallResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1D_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_provider_call_response.schema_version",
                reason: "must match PH1D_CONTRACT_VERSION",
            });
        }
        if self.correlation_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_provider_call_response.correlation_id",
                reason: "must be > 0",
            });
        }
        if self.turn_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_provider_call_response.turn_id",
                reason: "must be > 0",
            });
        }
        self.request_id.validate()?;
        validate_provider_token(
            "ph1d_provider_call_response.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        validate_opt_provider_token(
            "ph1d_provider_call_response.provider_call_id",
            &self.provider_call_id,
            128,
        )?;
        validate_provider_token(
            "ph1d_provider_call_response.provider_id",
            &self.provider_id,
            64,
        )?;
        validate_provider_token("ph1d_provider_call_response.model_id", &self.model_id, 128)?;
        if self.provider_latency_ms > 120_000 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_provider_call_response.provider_latency_ms",
                reason: "must be <= 120000",
            });
        }
        if let Some(provider_confidence_bp) = self.provider_confidence_bp {
            if provider_confidence_bp > 10_000 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_provider_call_response.provider_confidence_bp",
                    reason: "must be <= 10000",
                });
            }
        }
        if let Some(normalized_output_schema_hash) = self.normalized_output_schema_hash {
            normalized_output_schema_hash.validate()?;
        }
        if let Some(normalized_output_json) = &self.normalized_output_json {
            if normalized_output_json.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_provider_call_response.normalized_output_json",
                    reason: "must not be empty when present",
                });
            }
            if normalized_output_json.len() > 262_144 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_provider_call_response.normalized_output_json",
                    reason: "must be <= 262144 chars",
                });
            }
        }
        if self.validation_status == Ph1dProviderValidationStatus::SchemaOk {
            if self.normalized_output_schema_hash.is_none() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_provider_call_response.normalized_output_schema_hash",
                    reason: "must be Some(...) when validation_status=SCHEMA_OK",
                });
            }
            if self.normalized_output_json.is_none() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_provider_call_response.normalized_output_json",
                    reason: "must be Some(...) when validation_status=SCHEMA_OK",
                });
            }
            if self.normalized_output_schema_hash
                != Some(PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1)
            {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_provider_call_response.normalized_output_schema_hash",
                    reason: "must match PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1 when validation_status=SCHEMA_OK",
                });
            }
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_provider_call_response.reason_code",
                reason: "must be > 0",
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

fn validate_provider_token(
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
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':' | '/'))
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "contains unsupported characters",
        });
    }
    Ok(())
}

fn validate_opt_provider_token(
    field: &'static str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(v) = value {
        validate_provider_token(field, v, max_len)?;
    }
    Ok(())
}

fn validate_language_tag(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > 32 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 32 chars",
        });
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "contains unsupported characters",
        });
    }
    Ok(())
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
        if self.schema_version != PH1D_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_chat.schema_version",
                reason: "must match PH1D_CONTRACT_VERSION",
            });
        }
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
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_chat.reason_code",
                reason: "must be > 0",
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
        if self.schema_version != PH1D_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_clarify.schema_version",
                reason: "must match PH1D_CONTRACT_VERSION",
            });
        }
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
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_clarify.reason_code",
                reason: "must be > 0",
            });
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
        if self.schema_version != PH1D_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_analysis.schema_version",
                reason: "must match PH1D_CONTRACT_VERSION",
            });
        }
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
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_analysis.reason_code",
                reason: "must be > 0",
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
        if self.schema_version != PH1D_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_intent.schema_version",
                reason: "must match PH1D_CONTRACT_VERSION",
            });
        }
        for r in &self.field_refinements {
            r.validate()?;
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_intent.reason_code",
                reason: "must be > 0",
            });
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

    #[test]
    fn ph1d_provider_normalized_output_accepts_stt_schema_v1() {
        let out = Ph1dProviderNormalizedOutput::v1(
            Ph1dProviderTask::SttTranscribe,
            Some("invoice total due one hundred".to_string()),
            Some("en-US".to_string()),
            Some(9400),
            Some(true),
            None,
            None,
            None,
        )
        .expect("stt normalized output must validate");
        assert_eq!(out.provider_task, Ph1dProviderTask::SttTranscribe);
    }

    #[test]
    fn ph1d_provider_normalized_output_rejects_tts_without_audio_fields() {
        let err = Ph1dProviderNormalizedOutput::v1(
            Ph1dProviderTask::TtsSynthesize,
            Some("hello world".to_string()),
            Some("en-US".to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect_err("tts output must fail closed when audio fields are missing");
        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(field, "ph1d_provider_normalized_output.provider_task")
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn ph1d_provider_call_response_schema_ok_requires_normalized_schema_hash_v1() {
        let err = Ph1dProviderCallResponse::v1(
            9,
            11,
            RequestId(7),
            "idem_7".to_string(),
            Some("call_7".to_string()),
            "openai".to_string(),
            Ph1dProviderTask::SttTranscribe,
            "gpt-4o-mini-transcribe".to_string(),
            Ph1dProviderStatus::Ok,
            44,
            12,
            Some(9300),
            Some(SchemaHash(7001)),
            Some("{\"schema_version\":1}".to_string()),
            Ph1dProviderValidationStatus::SchemaOk,
            ReasonCodeId(1),
        )
        .expect_err("schema-ok response must require normalized schema hash v1");
        match err {
            ContractViolation::InvalidValue { field, .. } => assert_eq!(
                field,
                "ph1d_provider_call_response.normalized_output_schema_hash"
            ),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn ph1d_provider_call_response_schema_ok_accepts_normalized_schema_hash_v1() {
        let out = Ph1dProviderCallResponse::v1(
            9,
            11,
            RequestId(7),
            "idem_7".to_string(),
            Some("call_7".to_string()),
            "openai".to_string(),
            Ph1dProviderTask::SttTranscribe,
            "gpt-4o-mini-transcribe".to_string(),
            Ph1dProviderStatus::Ok,
            44,
            12,
            Some(9300),
            Some(PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1),
            Some("{\"schema_version\":1}".to_string()),
            Ph1dProviderValidationStatus::SchemaOk,
            ReasonCodeId(1),
        )
        .expect("schema-ok response should validate with normalized schema hash v1");
        assert_eq!(
            out.normalized_output_schema_hash,
            Some(PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1)
        );
    }
}
