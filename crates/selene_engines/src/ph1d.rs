#![forbid(unsafe_code)]

use std::env;
use std::io::Read;
use std::time::{Duration, Instant};

use serde_json::Value;

use selene_kernel_contracts::ph1d::{
    Ph1dAnalysis, Ph1dChat, Ph1dClarify, Ph1dFail, Ph1dFailureKind, Ph1dFieldRefinement,
    Ph1dIntent, Ph1dOk, Ph1dProviderCallRequest, Ph1dProviderCallResponse,
    Ph1dProviderNormalizedOutput, Ph1dProviderStatus, Ph1dProviderTask,
    Ph1dProviderValidationStatus, Ph1dRequest, Ph1dResponse,
    PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1,
};
use selene_kernel_contracts::ph1n::{
    EvidenceSpan, FieldKey, FieldValue, IntentType, TranscriptHash,
};
use selene_kernel_contracts::{ContractViolation, ReasonCodeId, Validate};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1dProviderAdapterError {
    pub message: String,
    pub retryable: bool,
}

impl Ph1dProviderAdapterError {
    pub fn terminal(message: String) -> Self {
        Self {
            message,
            retryable: false,
        }
    }

    pub fn retryable(message: String) -> Self {
        Self {
            message,
            retryable: true,
        }
    }
}

pub trait Ph1dProviderAdapter {
    fn execute(
        &self,
        req: &Ph1dProviderCallRequest,
    ) -> Result<Ph1dProviderCallResponse, Ph1dProviderAdapterError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1dLiveProviderAdapterConfig {
    pub openai_api_key: Option<String>,
    pub google_api_key: Option<String>,
    pub openai_ocr_endpoint: String,
    pub openai_stt_endpoint: String,
    pub openai_tts_endpoint: String,
    pub google_ocr_endpoint: String,
    pub google_stt_endpoint: String,
    pub google_tts_endpoint: String,
    pub connect_timeout_ms: u64,
}

impl Ph1dLiveProviderAdapterConfig {
    pub fn from_env() -> Self {
        Self {
            openai_api_key: env::var("PH1D_OPENAI_API_KEY").ok(),
            google_api_key: env::var("PH1D_GOOGLE_API_KEY").ok(),
            openai_ocr_endpoint: env::var("PH1D_OPENAI_OCR_ENDPOINT")
                .unwrap_or_else(|_| "https://api.openai.com/v1/responses".to_string()),
            openai_stt_endpoint: env::var("PH1D_OPENAI_STT_ENDPOINT")
                .unwrap_or_else(|_| "https://api.openai.com/v1/audio/transcriptions".to_string()),
            openai_tts_endpoint: env::var("PH1D_OPENAI_TTS_ENDPOINT")
                .unwrap_or_else(|_| "https://api.openai.com/v1/audio/speech".to_string()),
            google_ocr_endpoint: env::var("PH1D_GOOGLE_OCR_ENDPOINT")
                .unwrap_or_else(|_| "https://vision.googleapis.com/v1/images:annotate".to_string()),
            google_stt_endpoint: env::var("PH1D_GOOGLE_STT_ENDPOINT").unwrap_or_else(|_| {
                "https://speech.googleapis.com/v1/speech:recognize".to_string()
            }),
            google_tts_endpoint: env::var("PH1D_GOOGLE_TTS_ENDPOINT").unwrap_or_else(|_| {
                "https://texttospeech.googleapis.com/v1/text:synthesize".to_string()
            }),
            connect_timeout_ms: env::var("PH1D_PROVIDER_CONNECT_TIMEOUT_MS")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .filter(|v| (100..=120_000).contains(v))
                .unwrap_or(5_000),
        }
    }

    pub fn validate(&self) -> Result<(), ContractViolation> {
        validate_endpoint_url(
            "ph1d_live_provider_adapter_config.openai_ocr_endpoint",
            &self.openai_ocr_endpoint,
        )?;
        validate_endpoint_url(
            "ph1d_live_provider_adapter_config.openai_stt_endpoint",
            &self.openai_stt_endpoint,
        )?;
        validate_endpoint_url(
            "ph1d_live_provider_adapter_config.openai_tts_endpoint",
            &self.openai_tts_endpoint,
        )?;
        validate_endpoint_url(
            "ph1d_live_provider_adapter_config.google_ocr_endpoint",
            &self.google_ocr_endpoint,
        )?;
        validate_endpoint_url(
            "ph1d_live_provider_adapter_config.google_stt_endpoint",
            &self.google_stt_endpoint,
        )?;
        validate_endpoint_url(
            "ph1d_live_provider_adapter_config.google_tts_endpoint",
            &self.google_tts_endpoint,
        )?;
        if !(100..=120_000).contains(&self.connect_timeout_ms) {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_live_provider_adapter_config.connect_timeout_ms",
                reason: "must be within 100..=120000",
            });
        }
        Ok(())
    }

    fn endpoint_for(&self, vendor: Ph1dSpeechProviderVendor, task: Ph1dProviderTask) -> &str {
        match (vendor, task) {
            (Ph1dSpeechProviderVendor::OpenAi, Ph1dProviderTask::OcrTextExtract) => {
                self.openai_ocr_endpoint.as_str()
            }
            (Ph1dSpeechProviderVendor::OpenAi, Ph1dProviderTask::SttTranscribe) => {
                self.openai_stt_endpoint.as_str()
            }
            (Ph1dSpeechProviderVendor::OpenAi, Ph1dProviderTask::TtsSynthesize) => {
                self.openai_tts_endpoint.as_str()
            }
            (Ph1dSpeechProviderVendor::Google, Ph1dProviderTask::OcrTextExtract) => {
                self.google_ocr_endpoint.as_str()
            }
            (Ph1dSpeechProviderVendor::Google, Ph1dProviderTask::SttTranscribe) => {
                self.google_stt_endpoint.as_str()
            }
            (Ph1dSpeechProviderVendor::Google, Ph1dProviderTask::TtsSynthesize) => {
                self.google_tts_endpoint.as_str()
            }
        }
    }

    fn api_key_for(&self, vendor: Ph1dSpeechProviderVendor) -> Option<&str> {
        match vendor {
            Ph1dSpeechProviderVendor::OpenAi => self.openai_api_key.as_deref(),
            Ph1dSpeechProviderVendor::Google => self.google_api_key.as_deref(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1dLiveProviderAdapter {
    config: Ph1dLiveProviderAdapterConfig,
}

impl Ph1dLiveProviderAdapter {
    pub fn new(config: Ph1dLiveProviderAdapterConfig) -> Result<Self, ContractViolation> {
        config.validate()?;
        Ok(Self { config })
    }

    pub fn from_env() -> Result<Self, ContractViolation> {
        Self::new(Ph1dLiveProviderAdapterConfig::from_env())
    }

    fn execute_http_call(
        &self,
        req: &Ph1dProviderCallRequest,
        vendor: Ph1dSpeechProviderVendor,
    ) -> Result<Ph1dProviderTransportOutcome, Ph1dProviderAdapterError> {
        let Some(api_key) = self.config.api_key_for(vendor) else {
            return Ok(Ph1dProviderTransportOutcome::ContractMismatch {
                provider_call_id: None,
                provider_latency_ms: 0,
            });
        };

        let endpoint = self.config.endpoint_for(vendor, req.provider_task);
        let payload_json = build_live_provider_payload(req, vendor).map_err(|e| {
            Ph1dProviderAdapterError::terminal(format!(
                "ph1d live provider payload build failed: {e:?}"
            ))
        })?;

        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_millis(self.config.connect_timeout_ms))
            .timeout_read(Duration::from_millis(u64::from(req.timeout_ms)))
            .timeout_write(Duration::from_millis(u64::from(req.timeout_ms)))
            .build();

        let started = Instant::now();
        let mut http = agent
            .post(endpoint)
            .set("content-type", "application/json")
            .set("idempotency-key", req.idempotency_key.as_str())
            .set("x-selene-provider-task", req.provider_task.as_str())
            .set("x-selene-provider-route", req.provider_route_class.as_str())
            .set("x-selene-request-id", &req.request_id.0.to_string());
        match vendor {
            Ph1dSpeechProviderVendor::OpenAi => {
                http = http.set("authorization", &format!("Bearer {}", api_key));
            }
            Ph1dSpeechProviderVendor::Google => {
                http = http.set("x-goog-api-key", api_key);
            }
        }

        match http.send_string(&payload_json) {
            Ok(resp) => {
                let provider_latency_ms = elapsed_ms(started);
                let provider_call_id = provider_call_id_from_response(&resp);
                let status = resp.status();
                let raw_payload_json = read_response_body(resp);
                if !(200..=299).contains(&status) {
                    return Ok(status_to_transport_failure(
                        status,
                        provider_call_id,
                        provider_latency_ms,
                    ));
                }
                Ok(Ph1dProviderTransportOutcome::Ok {
                    provider_call_id,
                    provider_confidence_bp: confidence_hint_from_json(&raw_payload_json),
                    raw_payload_json,
                    provider_latency_ms,
                    provider_cost_microunits: 0,
                })
            }
            Err(ureq::Error::Status(status, resp)) => {
                let provider_latency_ms = elapsed_ms(started);
                let provider_call_id = provider_call_id_from_response(&resp);
                let _ = read_response_body(resp);
                Ok(status_to_transport_failure(
                    status,
                    provider_call_id,
                    provider_latency_ms,
                ))
            }
            Err(ureq::Error::Transport(err)) => match err.kind() {
                ureq::ErrorKind::Io | ureq::ErrorKind::ConnectionFailed => {
                    Ok(Ph1dProviderTransportOutcome::Timeout {
                        provider_call_id: None,
                        provider_latency_ms: elapsed_ms(started),
                    })
                }
                _ => Err(Ph1dProviderAdapterError::retryable(format!(
                    "ph1d live provider transport error: {err}"
                ))),
            },
        }
    }
}

impl Ph1dProviderAdapter for Ph1dLiveProviderAdapter {
    fn execute(
        &self,
        req: &Ph1dProviderCallRequest,
    ) -> Result<Ph1dProviderCallResponse, Ph1dProviderAdapterError> {
        req.validate().map_err(|e| {
            Ph1dProviderAdapterError::terminal(format!("ph1d live provider request invalid: {e:?}"))
        })?;

        let vendor = provider_vendor_from_provider_id(&req.provider_id);
        let Some(vendor) = vendor else {
            return normalize_provider_transport_outcome(
                req,
                Ph1dSpeechProviderVendor::OpenAi,
                Ph1dProviderTransportOutcome::ContractMismatch {
                    provider_call_id: None,
                    provider_latency_ms: 0,
                },
            )
            .map_err(|e| {
                Ph1dProviderAdapterError::terminal(format!(
                    "ph1d live provider mismatch normalization failed: {e:?}"
                ))
            });
        };

        let transport = self.execute_http_call(req, vendor)?;
        normalize_provider_transport_outcome(req, vendor, transport).map_err(|e| {
            Ph1dProviderAdapterError::terminal(format!(
                "ph1d live provider normalization failed: {e:?}"
            ))
        })
    }
}

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.D reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const D_FAIL_INVALID_SCHEMA: ReasonCodeId = ReasonCodeId(0x4400_0001);
    pub const D_FAIL_FORBIDDEN_OUTPUT: ReasonCodeId = ReasonCodeId(0x4400_0002);
    pub const D_FAIL_SAFETY_BLOCK: ReasonCodeId = ReasonCodeId(0x4400_0003);
    pub const D_FAIL_TIMEOUT: ReasonCodeId = ReasonCodeId(0x4400_0004);
    pub const D_FAIL_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4400_0005);

    pub const D_CLARIFY_EVIDENCE_REQUIRED: ReasonCodeId = ReasonCodeId(0x4400_0100);

    pub const D_PROVIDER_SCHEMA_DRIFT: ReasonCodeId = ReasonCodeId(0x4400_0201);
    pub const D_PROVIDER_TIMEOUT: ReasonCodeId = ReasonCodeId(0x4400_0202);
    pub const D_PROVIDER_CONTRACT_MISMATCH: ReasonCodeId = ReasonCodeId(0x4400_0203);
    pub const D_PROVIDER_OK: ReasonCodeId = ReasonCodeId(0x4400_0204);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ph1dSpeechProviderVendor {
    OpenAi,
    Google,
}

impl Ph1dSpeechProviderVendor {
    pub const fn as_str(self) -> &'static str {
        match self {
            Ph1dSpeechProviderVendor::OpenAi => "OPENAI",
            Ph1dSpeechProviderVendor::Google => "GOOGLE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1dProviderTransportOutcome {
    Ok {
        provider_call_id: Option<String>,
        raw_payload_json: String,
        provider_latency_ms: u32,
        provider_cost_microunits: u64,
        provider_confidence_bp: Option<u16>,
    },
    Timeout {
        provider_call_id: Option<String>,
        provider_latency_ms: u32,
    },
    ContractMismatch {
        provider_call_id: Option<String>,
        provider_latency_ms: u32,
    },
}

pub fn normalize_provider_transport_outcome(
    req: &Ph1dProviderCallRequest,
    vendor: Ph1dSpeechProviderVendor,
    outcome: Ph1dProviderTransportOutcome,
) -> Result<Ph1dProviderCallResponse, ContractViolation> {
    req.validate()?;

    if !vendor_matches_provider_id(vendor, &req.provider_id) {
        return provider_error_response(
            req,
            None,
            Ph1dProviderValidationStatus::SchemaFail,
            reason_codes::D_PROVIDER_CONTRACT_MISMATCH,
            0,
            0,
            None,
        );
    }

    match outcome {
        Ph1dProviderTransportOutcome::Timeout {
            provider_call_id,
            provider_latency_ms,
        } => provider_error_response(
            req,
            provider_call_id,
            Ph1dProviderValidationStatus::SchemaFail,
            reason_codes::D_PROVIDER_TIMEOUT,
            provider_latency_ms,
            0,
            None,
        ),
        Ph1dProviderTransportOutcome::ContractMismatch {
            provider_call_id,
            provider_latency_ms,
        } => provider_error_response(
            req,
            provider_call_id,
            Ph1dProviderValidationStatus::SchemaFail,
            reason_codes::D_PROVIDER_CONTRACT_MISMATCH,
            provider_latency_ms,
            0,
            None,
        ),
        Ph1dProviderTransportOutcome::Ok {
            provider_call_id,
            raw_payload_json,
            provider_latency_ms,
            provider_cost_microunits,
            provider_confidence_bp,
        } => {
            let normalized =
                match normalize_vendor_payload(vendor, req.provider_task, &raw_payload_json) {
                    Ok(v) => v,
                    Err(_) => {
                        return provider_error_response(
                            req,
                            provider_call_id,
                            Ph1dProviderValidationStatus::SchemaFail,
                            reason_codes::D_PROVIDER_SCHEMA_DRIFT,
                            provider_latency_ms,
                            provider_cost_microunits,
                            provider_confidence_bp,
                        )
                    }
                };
            let normalized_output_json = normalized_output_to_json(&normalized).map_err(|_| {
                ContractViolation::InvalidValue {
                    field: "ph1d_provider_boundary.normalized_output_json",
                    reason: "failed to serialize normalized output",
                }
            })?;
            Ph1dProviderCallResponse::v1(
                req.correlation_id,
                req.turn_id,
                req.request_id,
                req.idempotency_key.clone(),
                provider_call_id,
                req.provider_id.clone(),
                req.provider_task,
                req.model_id.clone(),
                Ph1dProviderStatus::Ok,
                provider_latency_ms,
                provider_cost_microunits,
                provider_confidence_bp,
                Some(PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1),
                Some(normalized_output_json),
                Ph1dProviderValidationStatus::SchemaOk,
                reason_codes::D_PROVIDER_OK,
            )
        }
    }
}

fn provider_error_response(
    req: &Ph1dProviderCallRequest,
    provider_call_id: Option<String>,
    validation_status: Ph1dProviderValidationStatus,
    reason_code: ReasonCodeId,
    provider_latency_ms: u32,
    provider_cost_microunits: u64,
    provider_confidence_bp: Option<u16>,
) -> Result<Ph1dProviderCallResponse, ContractViolation> {
    Ph1dProviderCallResponse::v1(
        req.correlation_id,
        req.turn_id,
        req.request_id,
        req.idempotency_key.clone(),
        provider_call_id,
        req.provider_id.clone(),
        req.provider_task,
        req.model_id.clone(),
        Ph1dProviderStatus::Error,
        provider_latency_ms,
        provider_cost_microunits,
        provider_confidence_bp,
        None,
        None,
        validation_status,
        reason_code,
    )
}

fn validate_endpoint_url(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if trimmed.len() > 1_024 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 1024 chars",
        });
    }
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must start with http:// or https://",
        });
    }
    Ok(())
}

fn provider_vendor_from_provider_id(provider_id: &str) -> Option<Ph1dSpeechProviderVendor> {
    let normalized = provider_id.trim().to_ascii_lowercase();
    if normalized.contains("openai") {
        Some(Ph1dSpeechProviderVendor::OpenAi)
    } else if normalized.contains("google") || normalized.contains("gcp") {
        Some(Ph1dSpeechProviderVendor::Google)
    } else {
        None
    }
}

fn vendor_matches_provider_id(vendor: Ph1dSpeechProviderVendor, provider_id: &str) -> bool {
    let normalized = provider_id.trim().to_ascii_lowercase();
    match vendor {
        Ph1dSpeechProviderVendor::OpenAi => normalized.contains("openai"),
        Ph1dSpeechProviderVendor::Google => {
            normalized.contains("google") || normalized.contains("gcp")
        }
    }
}

fn build_live_provider_payload(
    req: &Ph1dProviderCallRequest,
    vendor: Ph1dSpeechProviderVendor,
) -> Result<String, ContractViolation> {
    let task_label = match (vendor, req.provider_task) {
        (Ph1dSpeechProviderVendor::OpenAi, Ph1dProviderTask::OcrTextExtract) => "ocr.extract",
        (Ph1dSpeechProviderVendor::OpenAi, Ph1dProviderTask::SttTranscribe) => "stt.transcribe",
        (Ph1dSpeechProviderVendor::OpenAi, Ph1dProviderTask::TtsSynthesize) => "tts.synthesize",
        (Ph1dSpeechProviderVendor::Google, Ph1dProviderTask::OcrTextExtract) => {
            "vision:text_extract"
        }
        (Ph1dSpeechProviderVendor::Google, Ph1dProviderTask::SttTranscribe) => "speech:recognize",
        (Ph1dSpeechProviderVendor::Google, Ph1dProviderTask::TtsSynthesize) => "text:synthesize",
    };

    let input_payload = match req.input_payload_inline.as_deref() {
        Some(inline) => {
            serde_json::from_str::<Value>(inline).unwrap_or(Value::String(inline.to_string()))
        }
        None => Value::Null,
    };
    let payload = serde_json::json!({
        "task": task_label,
        "model": req.model_id,
        "tenant_id": req.tenant_id,
        "request_id": req.request_id.0,
        "idempotency_key": req.idempotency_key,
        "provider_route_class": req.provider_route_class.as_str(),
        "input_payload_kind": req.input_payload_kind.as_str(),
        "input_payload_ref": req.input_payload_ref,
        "input_content_type": req.input_content_type,
        "input_payload": input_payload,
        "prompt_template_ref": req.prompt_template_ref,
        "transcript_ref": req.transcript_ref,
        "safety_tier": safety_tier_str(req.safety_tier),
        "privacy_mode": req.privacy_mode,
        "do_not_disturb": req.do_not_disturb,
    });
    serde_json::to_string(&payload).map_err(|_| ContractViolation::InvalidValue {
        field: "ph1d_live_provider.payload_json",
        reason: "failed to serialize outbound provider payload",
    })
}

fn elapsed_ms(started: Instant) -> u32 {
    let elapsed = started.elapsed().as_millis();
    if elapsed > u128::from(u32::MAX) {
        u32::MAX
    } else {
        elapsed as u32
    }
}

fn provider_call_id_from_response(resp: &ureq::Response) -> Option<String> {
    resp.header("x-request-id")
        .or_else(|| resp.header("x-goog-request-id"))
        .or_else(|| resp.header("x-cloud-trace-context"))
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn read_response_body(resp: ureq::Response) -> String {
    let mut out = String::new();
    // Bound body reads for fail-closed robustness.
    let _ = resp.into_reader().take(262_144).read_to_string(&mut out);
    out
}

fn status_to_transport_failure(
    status: u16,
    provider_call_id: Option<String>,
    provider_latency_ms: u32,
) -> Ph1dProviderTransportOutcome {
    if status == 408 || status == 429 || status >= 500 {
        Ph1dProviderTransportOutcome::Timeout {
            provider_call_id,
            provider_latency_ms,
        }
    } else {
        Ph1dProviderTransportOutcome::ContractMismatch {
            provider_call_id,
            provider_latency_ms,
        }
    }
}

fn confidence_hint_from_json(raw_payload_json: &str) -> Option<u16> {
    let value = serde_json::from_str::<Value>(raw_payload_json).ok()?;
    let obj = value.as_object()?;
    let raw = obj.get("confidence_bp")?.as_u64()?;
    u16::try_from(raw).ok()
}

fn safety_tier_str(tier: selene_kernel_contracts::ph1d::SafetyTier) -> &'static str {
    match tier {
        selene_kernel_contracts::ph1d::SafetyTier::Standard => "STANDARD",
        selene_kernel_contracts::ph1d::SafetyTier::Strict => "STRICT",
    }
}

fn normalize_vendor_payload(
    vendor: Ph1dSpeechProviderVendor,
    provider_task: Ph1dProviderTask,
    raw_payload_json: &str,
) -> Result<Ph1dProviderNormalizedOutput, ContractViolation> {
    let v: Value =
        serde_json::from_str(raw_payload_json).map_err(|_| ContractViolation::InvalidValue {
            field: "ph1d_provider_boundary.raw_payload_json",
            reason: "must be valid json object",
        })?;
    let obj = v.as_object().ok_or(ContractViolation::InvalidValue {
        field: "ph1d_provider_boundary.raw_payload_json",
        reason: "must be a json object",
    })?;

    let normalized = match vendor {
        Ph1dSpeechProviderVendor::OpenAi => normalize_openai_payload(provider_task, obj)?,
        Ph1dSpeechProviderVendor::Google => normalize_google_payload(provider_task, obj)?,
    };
    normalized.validate()?;
    Ok(normalized)
}

fn normalize_openai_payload(
    provider_task: Ph1dProviderTask,
    obj: &serde_json::Map<String, Value>,
) -> Result<Ph1dProviderNormalizedOutput, ContractViolation> {
    let task = required_string("ph1d_provider_boundary.openai.task", obj.get("task"))?;
    let expected_task = match provider_task {
        Ph1dProviderTask::OcrTextExtract => "ocr.extract",
        Ph1dProviderTask::SttTranscribe => "stt.transcribe",
        Ph1dProviderTask::TtsSynthesize => "tts.synthesize",
    };
    if task != expected_task {
        return Err(ContractViolation::InvalidValue {
            field: "ph1d_provider_boundary.openai.task",
            reason: "provider task mismatch",
        });
    }

    match provider_task {
        Ph1dProviderTask::OcrTextExtract => Ph1dProviderNormalizedOutput::v1(
            provider_task,
            Some(
                required_string("ph1d_provider_boundary.openai.text", obj.get("text"))?.to_string(),
            ),
            optional_string(obj.get("language")).map(|v| v.to_string()),
            optional_u16(
                obj.get("confidence_bp"),
                "ph1d_provider_boundary.openai.confidence_bp",
            )?,
            None,
            None,
            None,
            None,
        ),
        Ph1dProviderTask::SttTranscribe => Ph1dProviderNormalizedOutput::v1(
            provider_task,
            Some(
                required_string("ph1d_provider_boundary.openai.text", obj.get("text"))?.to_string(),
            ),
            Some(
                required_string(
                    "ph1d_provider_boundary.openai.language",
                    obj.get("language"),
                )?
                .to_string(),
            ),
            Some(required_u16(
                "ph1d_provider_boundary.openai.confidence_bp",
                obj.get("confidence_bp"),
            )?),
            Some(required_bool(
                "ph1d_provider_boundary.openai.stable",
                obj.get("stable"),
            )?),
            None,
            None,
            None,
        ),
        Ph1dProviderTask::TtsSynthesize => Ph1dProviderNormalizedOutput::v1(
            provider_task,
            Some(
                required_string(
                    "ph1d_provider_boundary.openai.render_text",
                    obj.get("render_text"),
                )?
                .to_string(),
            ),
            optional_string(obj.get("language")).map(|v| v.to_string()),
            None,
            None,
            Some(
                required_string(
                    "ph1d_provider_boundary.openai.audio_ref",
                    obj.get("audio_ref"),
                )?
                .to_string(),
            ),
            Some(
                required_string(
                    "ph1d_provider_boundary.openai.audio_content_type",
                    obj.get("audio_content_type"),
                )?
                .to_string(),
            ),
            Some(required_u32(
                "ph1d_provider_boundary.openai.estimated_duration_ms",
                obj.get("estimated_duration_ms"),
            )?),
        ),
    }
}

fn normalize_google_payload(
    provider_task: Ph1dProviderTask,
    obj: &serde_json::Map<String, Value>,
) -> Result<Ph1dProviderNormalizedOutput, ContractViolation> {
    let task = required_string("ph1d_provider_boundary.google.task", obj.get("task"))?;
    let expected_task = match provider_task {
        Ph1dProviderTask::OcrTextExtract => "vision:text_extract",
        Ph1dProviderTask::SttTranscribe => "speech:recognize",
        Ph1dProviderTask::TtsSynthesize => "text:synthesize",
    };
    if task != expected_task {
        return Err(ContractViolation::InvalidValue {
            field: "ph1d_provider_boundary.google.task",
            reason: "provider task mismatch",
        });
    }

    match provider_task {
        Ph1dProviderTask::OcrTextExtract => Ph1dProviderNormalizedOutput::v1(
            provider_task,
            Some(
                required_string(
                    "ph1d_provider_boundary.google.ocr_text",
                    obj.get("ocr_text"),
                )?
                .to_string(),
            ),
            optional_string(obj.get("lang")).map(|v| v.to_string()),
            optional_u16(
                obj.get("confidence_bp"),
                "ph1d_provider_boundary.google.confidence_bp",
            )?,
            None,
            None,
            None,
            None,
        ),
        Ph1dProviderTask::SttTranscribe => Ph1dProviderNormalizedOutput::v1(
            provider_task,
            Some(
                required_string(
                    "ph1d_provider_boundary.google.transcript",
                    obj.get("transcript"),
                )?
                .to_string(),
            ),
            Some(
                required_string("ph1d_provider_boundary.google.lang", obj.get("lang"))?.to_string(),
            ),
            Some(required_u16(
                "ph1d_provider_boundary.google.confidence_bp",
                obj.get("confidence_bp"),
            )?),
            Some(required_bool(
                "ph1d_provider_boundary.google.is_final",
                obj.get("is_final"),
            )?),
            None,
            None,
            None,
        ),
        Ph1dProviderTask::TtsSynthesize => Ph1dProviderNormalizedOutput::v1(
            provider_task,
            Some(
                required_string("ph1d_provider_boundary.google.text", obj.get("text"))?.to_string(),
            ),
            optional_string(obj.get("lang")).map(|v| v.to_string()),
            None,
            None,
            Some(
                required_string(
                    "ph1d_provider_boundary.google.audio_uri",
                    obj.get("audio_uri"),
                )?
                .to_string(),
            ),
            Some(
                required_string(
                    "ph1d_provider_boundary.google.mime_type",
                    obj.get("mime_type"),
                )?
                .to_string(),
            ),
            Some(required_u32(
                "ph1d_provider_boundary.google.duration_ms",
                obj.get("duration_ms"),
            )?),
        ),
    }
}

fn normalized_output_to_json(
    normalized: &Ph1dProviderNormalizedOutput,
) -> Result<String, serde_json::Error> {
    let payload = serde_json::json!({
        "schema_version": normalized.schema_version.0,
        "provider_task": normalized.provider_task.as_str(),
        "text_output": normalized.text_output,
        "language_tag": normalized.language_tag,
        "confidence_bp": normalized.confidence_bp,
        "stable": normalized.stable,
        "audio_output_ref": normalized.audio_output_ref,
        "audio_content_type": normalized.audio_content_type,
        "estimated_duration_ms": normalized.estimated_duration_ms,
    });
    serde_json::to_string(&payload)
}

pub fn decode_normalized_output_json(
    json_text: &str,
) -> Result<Ph1dProviderNormalizedOutput, ContractViolation> {
    let v: Value =
        serde_json::from_str(json_text).map_err(|_| ContractViolation::InvalidValue {
            field: "ph1d_provider_boundary.decode_normalized_output_json",
            reason: "must be valid json object",
        })?;
    let obj = v.as_object().ok_or(ContractViolation::InvalidValue {
        field: "ph1d_provider_boundary.decode_normalized_output_json",
        reason: "must be a json object",
    })?;

    let schema_version = required_u32(
        "ph1d_provider_boundary.decode.schema_version",
        obj.get("schema_version"),
    )?;
    let provider_task_str = required_string(
        "ph1d_provider_boundary.decode.provider_task",
        obj.get("provider_task"),
    )?;
    let provider_task = provider_task_from_str(provider_task_str)?;

    let text_output = optional_string(obj.get("text_output")).map(|v| v.to_string());
    let language_tag = optional_string(obj.get("language_tag")).map(|v| v.to_string());
    let confidence_bp = optional_u16(
        obj.get("confidence_bp"),
        "ph1d_provider_boundary.decode.confidence_bp",
    )?;
    let stable = optional_bool(obj.get("stable"), "ph1d_provider_boundary.decode.stable")?;
    let audio_output_ref = optional_string(obj.get("audio_output_ref")).map(|v| v.to_string());
    let audio_content_type = optional_string(obj.get("audio_content_type")).map(|v| v.to_string());
    let estimated_duration_ms = optional_u32(
        obj.get("estimated_duration_ms"),
        "ph1d_provider_boundary.decode.estimated_duration_ms",
    )?;

    let normalized = Ph1dProviderNormalizedOutput {
        schema_version: selene_kernel_contracts::SchemaVersion(schema_version),
        provider_task,
        text_output,
        language_tag,
        confidence_bp,
        stable,
        audio_output_ref,
        audio_content_type,
        estimated_duration_ms,
    };
    normalized.validate()?;
    Ok(normalized)
}

fn provider_task_from_str(task: &str) -> Result<Ph1dProviderTask, ContractViolation> {
    match task {
        "OCR_TEXT_EXTRACT" => Ok(Ph1dProviderTask::OcrTextExtract),
        "STT_TRANSCRIBE" => Ok(Ph1dProviderTask::SttTranscribe),
        "TTS_SYNTHESIZE" => Ok(Ph1dProviderTask::TtsSynthesize),
        _ => Err(ContractViolation::InvalidValue {
            field: "ph1d_provider_boundary.decode.provider_task",
            reason: "unsupported provider_task",
        }),
    }
}

fn required_string<'a>(
    field: &'static str,
    value: Option<&'a Value>,
) -> Result<&'a str, ContractViolation> {
    value
        .and_then(Value::as_str)
        .ok_or(ContractViolation::InvalidValue {
            field,
            reason: "must be a string",
        })
}

fn optional_string(value: Option<&Value>) -> Option<&str> {
    value.and_then(Value::as_str)
}

fn required_u32(field: &'static str, value: Option<&Value>) -> Result<u32, ContractViolation> {
    let raw = value
        .and_then(Value::as_u64)
        .ok_or(ContractViolation::InvalidValue {
            field,
            reason: "must be an unsigned integer",
        })?;
    u32::try_from(raw).map_err(|_| ContractViolation::InvalidValue {
        field,
        reason: "must fit in u32",
    })
}

fn optional_u32(
    value: Option<&Value>,
    field: &'static str,
) -> Result<Option<u32>, ContractViolation> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(_) => required_u32(field, value).map(Some),
    }
}

fn required_u16(field: &'static str, value: Option<&Value>) -> Result<u16, ContractViolation> {
    let raw = value
        .and_then(Value::as_u64)
        .ok_or(ContractViolation::InvalidValue {
            field,
            reason: "must be an unsigned integer",
        })?;
    u16::try_from(raw).map_err(|_| ContractViolation::InvalidValue {
        field,
        reason: "must fit in u16",
    })
}

fn optional_u16(
    value: Option<&Value>,
    field: &'static str,
) -> Result<Option<u16>, ContractViolation> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(_) => required_u16(field, value).map(Some),
    }
}

fn required_bool(field: &'static str, value: Option<&Value>) -> Result<bool, ContractViolation> {
    value
        .and_then(Value::as_bool)
        .ok_or(ContractViolation::InvalidValue {
            field,
            reason: "must be a boolean",
        })
}

fn optional_bool(
    value: Option<&Value>,
    field: &'static str,
) -> Result<Option<bool>, ContractViolation> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(_) => required_bool(field, value).map(Some),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1dConfig {
    pub max_chat_chars: usize,
}

impl Ph1dConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_chat_chars: 2_048,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModelCallOutcome {
    Ok { raw_json: String },
    Timeout,
    BudgetExceeded,
    SafetyBlock,
}

#[derive(Debug, Clone)]
pub struct Ph1dRuntime {
    config: Ph1dConfig,
}

impl Ph1dRuntime {
    pub fn new(config: Ph1dConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1dRequest, outcome: ModelCallOutcome) -> Ph1dResponse {
        if req.validate().is_err() {
            return Ph1dResponse::Fail(Ph1dFail::v1(
                reason_codes::D_FAIL_INVALID_SCHEMA,
                Ph1dFailureKind::InvalidSchema,
            ));
        }

        match outcome {
            ModelCallOutcome::Timeout => Ph1dResponse::Fail(Ph1dFail::v1(
                reason_codes::D_FAIL_TIMEOUT,
                Ph1dFailureKind::Timeout,
            )),
            ModelCallOutcome::BudgetExceeded => Ph1dResponse::Fail(Ph1dFail::v1(
                reason_codes::D_FAIL_BUDGET_EXCEEDED,
                Ph1dFailureKind::BudgetExceeded,
            )),
            ModelCallOutcome::SafetyBlock => Ph1dResponse::Fail(Ph1dFail::v1(
                reason_codes::D_FAIL_SAFETY_BLOCK,
                Ph1dFailureKind::SafetyBlock,
            )),
            ModelCallOutcome::Ok { raw_json } => self.parse_and_enforce(req, &raw_json),
        }
    }

    fn parse_and_enforce(&self, req: &Ph1dRequest, raw_json: &str) -> Ph1dResponse {
        let v: Value = match serde_json::from_str(raw_json) {
            Ok(v) => v,
            Err(_) => {
                return Ph1dResponse::Fail(Ph1dFail::v1(
                    reason_codes::D_FAIL_INVALID_SCHEMA,
                    Ph1dFailureKind::InvalidSchema,
                ));
            }
        };

        let obj = match v.as_object() {
            Some(o) => o,
            None => {
                return Ph1dResponse::Fail(Ph1dFail::v1(
                    reason_codes::D_FAIL_INVALID_SCHEMA,
                    Ph1dFailureKind::InvalidSchema,
                ));
            }
        };

        let mode = match obj.get("mode").and_then(|m| m.as_str()) {
            Some(s) => s,
            None => {
                return Ph1dResponse::Fail(Ph1dFail::v1(
                    reason_codes::D_FAIL_INVALID_SCHEMA,
                    Ph1dFailureKind::InvalidSchema,
                ));
            }
        };

        if let Some(rc) = detect_forbidden_keys(mode, obj.keys().map(|k| k.as_str())) {
            return Ph1dResponse::Fail(Ph1dFail::v1(rc, Ph1dFailureKind::ForbiddenOutput));
        }

        match mode {
            "chat" => self.parse_chat(obj),
            "clarify" => self.parse_clarify(obj),
            "analysis" => self.parse_analysis(obj),
            "intent" => self.parse_intent(req, obj),
            _ => Ph1dResponse::Fail(Ph1dFail::v1(
                reason_codes::D_FAIL_INVALID_SCHEMA,
                Ph1dFailureKind::InvalidSchema,
            )),
        }
    }

    fn parse_chat(&self, obj: &serde_json::Map<String, Value>) -> Ph1dResponse {
        let response_text = match obj.get("response_text").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => {
                return Ph1dResponse::Fail(Ph1dFail::v1(
                    reason_codes::D_FAIL_INVALID_SCHEMA,
                    Ph1dFailureKind::InvalidSchema,
                ));
            }
        };

        let reason_code = match parse_reason_code(obj) {
            Some(rc) => rc,
            None => {
                return Ph1dResponse::Fail(Ph1dFail::v1(
                    reason_codes::D_FAIL_INVALID_SCHEMA,
                    Ph1dFailureKind::InvalidSchema,
                ));
            }
        };

        if response_text.len() > self.config.max_chat_chars {
            return Ph1dResponse::Fail(Ph1dFail::v1(
                reason_codes::D_FAIL_INVALID_SCHEMA,
                Ph1dFailureKind::InvalidSchema,
            ));
        }

        if contains_authority_invention(response_text) {
            return Ph1dResponse::Fail(Ph1dFail::v1(
                reason_codes::D_FAIL_FORBIDDEN_OUTPUT,
                Ph1dFailureKind::ForbiddenOutput,
            ));
        }

        // No leakage: chat must not disclose tools/providers/prompts/system rules.
        if contains_internal_leakage(response_text) {
            return Ph1dResponse::Fail(Ph1dFail::v1(
                reason_codes::D_FAIL_FORBIDDEN_OUTPUT,
                Ph1dFailureKind::ForbiddenOutput,
            ));
        }

        match Ph1dChat::v1(response_text.to_string(), reason_code) {
            Ok(c) => Ph1dResponse::Ok(Ph1dOk::Chat(c)),
            Err(_) => Ph1dResponse::Fail(Ph1dFail::v1(
                reason_codes::D_FAIL_INVALID_SCHEMA,
                Ph1dFailureKind::InvalidSchema,
            )),
        }
    }

    fn parse_clarify(&self, obj: &serde_json::Map<String, Value>) -> Ph1dResponse {
        let question = match obj.get("question").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => {
                return Ph1dResponse::Fail(Ph1dFail::v1(
                    reason_codes::D_FAIL_INVALID_SCHEMA,
                    Ph1dFailureKind::InvalidSchema,
                ));
            }
        };
        let what_is_missing = match obj.get("what_is_missing").and_then(|v| v.as_array()) {
            Some(a) => a
                .iter()
                .filter_map(|x| x.as_str())
                .filter_map(field_key_from_str)
                .collect::<Vec<_>>(),
            None => {
                return Ph1dResponse::Fail(Ph1dFail::v1(
                    reason_codes::D_FAIL_INVALID_SCHEMA,
                    Ph1dFailureKind::InvalidSchema,
                ));
            }
        };
        let formats = match obj
            .get("accepted_answer_formats")
            .and_then(|v| v.as_array())
        {
            Some(a) => a
                .iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>(),
            None => {
                return Ph1dResponse::Fail(Ph1dFail::v1(
                    reason_codes::D_FAIL_INVALID_SCHEMA,
                    Ph1dFailureKind::InvalidSchema,
                ));
            }
        };

        let reason_code = match parse_reason_code(obj) {
            Some(rc) => rc,
            None => {
                return Ph1dResponse::Fail(Ph1dFail::v1(
                    reason_codes::D_FAIL_INVALID_SCHEMA,
                    Ph1dFailureKind::InvalidSchema,
                ));
            }
        };

        match Ph1dClarify::v1(question.to_string(), what_is_missing, formats, reason_code) {
            Ok(c) => Ph1dResponse::Ok(Ph1dOk::Clarify(c)),
            Err(_) => Ph1dResponse::Fail(Ph1dFail::v1(
                reason_codes::D_FAIL_INVALID_SCHEMA,
                Ph1dFailureKind::InvalidSchema,
            )),
        }
    }

    fn parse_analysis(&self, obj: &serde_json::Map<String, Value>) -> Ph1dResponse {
        let short_analysis = match obj.get("short_analysis").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => {
                return Ph1dResponse::Fail(Ph1dFail::v1(
                    reason_codes::D_FAIL_INVALID_SCHEMA,
                    Ph1dFailureKind::InvalidSchema,
                ));
            }
        };

        let reason_code = match parse_reason_code(obj) {
            Some(rc) => rc,
            None => {
                return Ph1dResponse::Fail(Ph1dFail::v1(
                    reason_codes::D_FAIL_INVALID_SCHEMA,
                    Ph1dFailureKind::InvalidSchema,
                ));
            }
        };

        match Ph1dAnalysis::v1(short_analysis.to_string(), reason_code) {
            Ok(a) => Ph1dResponse::Ok(Ph1dOk::Analysis(a)),
            Err(_) => Ph1dResponse::Fail(Ph1dFail::v1(
                reason_codes::D_FAIL_INVALID_SCHEMA,
                Ph1dFailureKind::InvalidSchema,
            )),
        }
    }

    fn parse_intent(
        &self,
        req: &Ph1dRequest,
        obj: &serde_json::Map<String, Value>,
    ) -> Ph1dResponse {
        let refined_intent_type = match obj.get("intent_type").and_then(|v| v.as_str()) {
            Some(s) => match intent_type_from_str(s) {
                Some(i) => i,
                None => {
                    return Ph1dResponse::Fail(Ph1dFail::v1(
                        reason_codes::D_FAIL_INVALID_SCHEMA,
                        Ph1dFailureKind::InvalidSchema,
                    ));
                }
            },
            None => {
                return Ph1dResponse::Fail(Ph1dFail::v1(
                    reason_codes::D_FAIL_INVALID_SCHEMA,
                    Ph1dFailureKind::InvalidSchema,
                ));
            }
        };

        let missing_fields = match obj.get("missing_fields") {
            Some(Value::Array(a)) => a
                .iter()
                .filter_map(|v| v.as_str())
                .filter_map(field_key_from_str)
                .collect::<Vec<_>>(),
            Some(_) => {
                return Ph1dResponse::Fail(Ph1dFail::v1(
                    reason_codes::D_FAIL_INVALID_SCHEMA,
                    Ph1dFailureKind::InvalidSchema,
                ));
            }
            None => vec![],
        };

        let reason_code = match parse_reason_code(obj) {
            Some(rc) => rc,
            None => {
                return Ph1dResponse::Fail(Ph1dFail::v1(
                    reason_codes::D_FAIL_INVALID_SCHEMA,
                    Ph1dFailureKind::InvalidSchema,
                ));
            }
        };

        let mut refinements: Vec<Ph1dFieldRefinement> = Vec::new();
        let mut evidence_violation = false;
        let mut violation_fields: Vec<FieldKey> = Vec::new();

        if let Some(Value::Array(arr)) = obj.get("field_refinements") {
            for item in arr {
                let Some(ro) = item.as_object() else {
                    return Ph1dResponse::Fail(Ph1dFail::v1(
                        reason_codes::D_FAIL_INVALID_SCHEMA,
                        Ph1dFailureKind::InvalidSchema,
                    ));
                };

                // Reject unexpected keys inside refinement objects.
                if let Some(rc) = detect_forbidden_refinement_keys(ro.keys().map(|k| k.as_str())) {
                    return Ph1dResponse::Fail(Ph1dFail::v1(rc, Ph1dFailureKind::ForbiddenOutput));
                }

                let field = match ro
                    .get("field")
                    .and_then(|v| v.as_str())
                    .and_then(field_key_from_str)
                {
                    Some(f) => f,
                    None => {
                        return Ph1dResponse::Fail(Ph1dFail::v1(
                            reason_codes::D_FAIL_INVALID_SCHEMA,
                            Ph1dFailureKind::InvalidSchema,
                        ));
                    }
                };

                let original_span = match ro.get("original_span").and_then(|v| v.as_str()) {
                    Some(s) => s.to_string(),
                    None => {
                        return Ph1dResponse::Fail(Ph1dFail::v1(
                            reason_codes::D_FAIL_INVALID_SCHEMA,
                            Ph1dFailureKind::InvalidSchema,
                        ));
                    }
                };

                let normalized_value = match ro.get("normalized_value") {
                    Some(Value::String(s)) => Some(s.to_string()),
                    Some(Value::Null) | None => None,
                    Some(_) => {
                        return Ph1dResponse::Fail(Ph1dFail::v1(
                            reason_codes::D_FAIL_INVALID_SCHEMA,
                            Ph1dFailureKind::InvalidSchema,
                        ));
                    }
                };

                let evidence_span = match ro.get("evidence_span").and_then(|v| v.as_object()) {
                    Some(o) => {
                        if let Some(rc) =
                            detect_forbidden_evidence_span_keys(o.keys().map(|k| k.as_str()))
                        {
                            return Ph1dResponse::Fail(Ph1dFail::v1(
                                rc,
                                Ph1dFailureKind::ForbiddenOutput,
                            ));
                        }

                        let span_field = match o.get("field").and_then(|v| v.as_str()) {
                            Some(s) => match field_key_from_str(s) {
                                Some(f) => f,
                                None => {
                                    return Ph1dResponse::Fail(Ph1dFail::v1(
                                        reason_codes::D_FAIL_INVALID_SCHEMA,
                                        Ph1dFailureKind::InvalidSchema,
                                    ));
                                }
                            },
                            None => {
                                return Ph1dResponse::Fail(Ph1dFail::v1(
                                    reason_codes::D_FAIL_INVALID_SCHEMA,
                                    Ph1dFailureKind::InvalidSchema,
                                ));
                            }
                        };
                        let transcript_hash =
                            match o.get("transcript_hash").and_then(|v| v.as_u64()) {
                                Some(n) => TranscriptHash(n),
                                None => {
                                    return Ph1dResponse::Fail(Ph1dFail::v1(
                                        reason_codes::D_FAIL_INVALID_SCHEMA,
                                        Ph1dFailureKind::InvalidSchema,
                                    ));
                                }
                            };
                        let start_byte = match o.get("start_byte").and_then(|v| v.as_u64()) {
                            Some(n) => n as u32,
                            None => {
                                return Ph1dResponse::Fail(Ph1dFail::v1(
                                    reason_codes::D_FAIL_INVALID_SCHEMA,
                                    Ph1dFailureKind::InvalidSchema,
                                ));
                            }
                        };
                        let end_byte = match o.get("end_byte").and_then(|v| v.as_u64()) {
                            Some(n) => n as u32,
                            None => {
                                return Ph1dResponse::Fail(Ph1dFail::v1(
                                    reason_codes::D_FAIL_INVALID_SCHEMA,
                                    Ph1dFailureKind::InvalidSchema,
                                ));
                            }
                        };
                        let verbatim_excerpt =
                            match o.get("verbatim_excerpt").and_then(|v| v.as_str()) {
                                Some(s) => s.to_string(),
                                None => {
                                    return Ph1dResponse::Fail(Ph1dFail::v1(
                                        reason_codes::D_FAIL_INVALID_SCHEMA,
                                        Ph1dFailureKind::InvalidSchema,
                                    ));
                                }
                            };

                        let span = EvidenceSpan {
                            field: span_field,
                            transcript_hash,
                            start_byte,
                            end_byte,
                            verbatim_excerpt,
                        };
                        match span.validate() {
                            Ok(()) => span,
                            Err(_) => {
                                return Ph1dResponse::Fail(Ph1dFail::v1(
                                    reason_codes::D_FAIL_INVALID_SCHEMA,
                                    Ph1dFailureKind::InvalidSchema,
                                ));
                            }
                        }
                    }
                    None => {
                        return Ph1dResponse::Fail(Ph1dFail::v1(
                            reason_codes::D_FAIL_INVALID_SCHEMA,
                            Ph1dFailureKind::InvalidSchema,
                        ));
                    }
                };

                // Evidence discipline: the span must prove THIS field and match original_span exactly.
                if evidence_span.field != field || evidence_span.verbatim_excerpt != original_span {
                    evidence_violation = true;
                    if !violation_fields.contains(&field) {
                        violation_fields.push(field);
                    }
                    continue;
                }

                if !is_evidence_span_exact(req, &evidence_span) {
                    evidence_violation = true;
                    if !violation_fields.contains(&field) {
                        violation_fields.push(field);
                    }
                    continue;
                }

                let value = match normalized_value {
                    Some(n) => FieldValue::normalized(original_span.clone(), n),
                    None => FieldValue::verbatim(original_span.clone()),
                };

                let value = match value {
                    Ok(v) => v,
                    Err(_) => {
                        return Ph1dResponse::Fail(Ph1dFail::v1(
                            reason_codes::D_FAIL_INVALID_SCHEMA,
                            Ph1dFailureKind::InvalidSchema,
                        ));
                    }
                };

                refinements.push(Ph1dFieldRefinement {
                    field,
                    value,
                    evidence_span,
                });
            }
        } else if obj.get("field_refinements").is_some() {
            return Ph1dResponse::Fail(Ph1dFail::v1(
                reason_codes::D_FAIL_INVALID_SCHEMA,
                Ph1dFailureKind::InvalidSchema,
            ));
        }

        // Evidence discipline: invented refinements must be converted into a clarify path.
        if evidence_violation {
            let ask = if !missing_fields.is_empty() {
                missing_fields.clone()
            } else if !violation_fields.is_empty() {
                violation_fields
            } else {
                vec![FieldKey::Task]
            };
            match clarify_for_missing_fields(&ask) {
                Ok(c) => return Ph1dResponse::Ok(Ph1dOk::Clarify(c)),
                Err(_) => {
                    return Ph1dResponse::Fail(Ph1dFail::v1(
                        reason_codes::D_FAIL_INVALID_SCHEMA,
                        Ph1dFailureKind::InvalidSchema,
                    ))
                }
            }
        }

        match Ph1dIntent::v1(
            refined_intent_type,
            refinements,
            missing_fields,
            reason_code,
        ) {
            Ok(i) => Ph1dResponse::Ok(Ph1dOk::Intent(i)),
            Err(_) => Ph1dResponse::Fail(Ph1dFail::v1(
                reason_codes::D_FAIL_INVALID_SCHEMA,
                Ph1dFailureKind::InvalidSchema,
            )),
        }
    }
}

fn detect_forbidden_keys<'a>(
    mode: &str,
    keys: impl Iterator<Item = &'a str>,
) -> Option<ReasonCodeId> {
    let allowed: &[&str] = match mode {
        "chat" => &["mode", "response_text", "reason_code"],
        "clarify" => &[
            "mode",
            "question",
            "what_is_missing",
            "accepted_answer_formats",
            "reason_code",
        ],
        "analysis" => &["mode", "short_analysis", "reason_code"],
        "intent" => &[
            "mode",
            "intent_type",
            "field_refinements",
            "missing_fields",
            "reason_code",
        ],
        _ => &["mode"],
    };

    for k in keys {
        if !allowed.contains(&k) {
            return Some(reason_codes::D_FAIL_FORBIDDEN_OUTPUT);
        }
    }
    None
}

fn detect_forbidden_refinement_keys<'a>(
    keys: impl Iterator<Item = &'a str>,
) -> Option<ReasonCodeId> {
    let allowed = [
        "field",
        "original_span",
        "normalized_value",
        "evidence_span",
    ];
    for k in keys {
        if !allowed.contains(&k) {
            return Some(reason_codes::D_FAIL_FORBIDDEN_OUTPUT);
        }
    }
    None
}

fn detect_forbidden_evidence_span_keys<'a>(
    keys: impl Iterator<Item = &'a str>,
) -> Option<ReasonCodeId> {
    let allowed = [
        "field",
        "transcript_hash",
        "start_byte",
        "end_byte",
        "verbatim_excerpt",
    ];
    for k in keys {
        if !allowed.contains(&k) {
            return Some(reason_codes::D_FAIL_FORBIDDEN_OUTPUT);
        }
    }
    None
}

fn field_key_from_str(s: &str) -> Option<FieldKey> {
    match s {
        "When" | "when" => Some(FieldKey::When),
        "Task" | "task" => Some(FieldKey::Task),
        "Person" | "person" => Some(FieldKey::Person),
        "Place" | "place" => Some(FieldKey::Place),
        "PartySize" | "party_size" | "partySize" | "party size" => Some(FieldKey::PartySize),
        "Amount" | "amount" => Some(FieldKey::Amount),
        "Recipient" | "recipient" => Some(FieldKey::Recipient),
        "InviteeType" | "invitee_type" | "inviteeType" => Some(FieldKey::InviteeType),
        "DeliveryMethod" | "delivery_method" | "deliveryMethod" => Some(FieldKey::DeliveryMethod),
        "RecipientContact" | "recipient_contact" | "recipientContact" => {
            Some(FieldKey::RecipientContact)
        }
        "TenantId" | "tenant_id" | "tenantId" => Some(FieldKey::TenantId),
        "RequestedCapabilityId" | "requested_capability_id" | "requestedCapabilityId" => {
            Some(FieldKey::RequestedCapabilityId)
        }
        "TargetScopeRef" | "target_scope_ref" | "targetScopeRef" => Some(FieldKey::TargetScopeRef),
        "Justification" | "justification" => Some(FieldKey::Justification),
        "CapreqAction" | "capreq_action" | "capreqAction" => Some(FieldKey::CapreqAction),
        "CapreqId" | "capreq_id" | "capreqId" => Some(FieldKey::CapreqId),
        "IntentChoice" | "intent_choice" | "intentChoice" => Some(FieldKey::IntentChoice),
        "ReferenceTarget" | "reference_target" | "referenceTarget" => {
            Some(FieldKey::ReferenceTarget)
        }
        _ => None,
    }
}

fn intent_type_from_str(s: &str) -> Option<IntentType> {
    match s {
        "CreateCalendarEvent" | "create_calendar_event" => Some(IntentType::CreateCalendarEvent),
        "SetReminder" | "set_reminder" => Some(IntentType::SetReminder),
        "BookTable" | "book_table" => Some(IntentType::BookTable),
        "SendMoney" | "send_money" => Some(IntentType::SendMoney),
        "CreateInviteLink" | "create_invite_link" | "createInviteLink" => {
            Some(IntentType::CreateInviteLink)
        }
        "CapreqManage" | "capreq_manage" | "capreqManage" => Some(IntentType::CapreqManage),
        "TimeQuery" | "time_query" => Some(IntentType::TimeQuery),
        "WeatherQuery" | "weather_query" => Some(IntentType::WeatherQuery),
        "Continue" | "continue" => Some(IntentType::Continue),
        "MoreDetail" | "more_detail" | "moreDetail" => Some(IntentType::MoreDetail),
        _ => None,
    }
}

fn is_evidence_span_exact(req: &Ph1dRequest, span: &EvidenceSpan) -> bool {
    if span.transcript_hash != req.transcript_hash {
        return false;
    }
    let t = &req.transcript_ok.transcript_text;
    let start = span.start_byte as usize;
    let end = span.end_byte as usize;
    match t.get(start..end) {
        Some(s) => s == span.verbatim_excerpt,
        None => false,
    }
}

fn contains_authority_invention(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    lower.contains("permission granted")
        || lower.contains("approved")
        || lower.contains("i approve")
        || lower.contains("authorization granted")
}

fn contains_internal_leakage(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    lower.contains("web_search")
        || lower.contains("web search")
        || lower.contains("tool")
        || lower.contains("provider")
        || lower.contains("openai")
        || lower.contains("google")
        || lower.contains("prompt")
        || lower.contains("system policy")
        || lower.contains("schema")
        || lower.contains("i can browse")
}

fn clarify_for_missing_fields(
    missing_fields: &[FieldKey],
) -> Result<Ph1dClarify, ContractViolation> {
    let primary = select_primary_missing(missing_fields);
    let (q, formats) = match primary {
        FieldKey::When => (
            "What day and time do you mean?".to_string(),
            vec![
                "Tomorrow at 3pm".to_string(),
                "Friday 10am".to_string(),
                "2026-02-10 15:00".to_string(),
            ],
        ),
        FieldKey::Amount => (
            "How much should I send?".to_string(),
            vec![
                "$20".to_string(),
                "100 dollars".to_string(),
                "15".to_string(),
            ],
        ),
        FieldKey::Task => (
            "What exactly should I do?".to_string(),
            vec![
                "Remind me to call mom".to_string(),
                "Schedule a meeting".to_string(),
            ],
        ),
        FieldKey::Recipient => (
            "Who should I send it to?".to_string(),
            vec!["To Alex".to_string(), "To John".to_string()],
        ),
        FieldKey::RequestedCapabilityId => (
            "Which capability should this request include?".to_string(),
            vec![
                "position.activate".to_string(),
                "access.override.create".to_string(),
                "payroll.approve".to_string(),
            ],
        ),
        FieldKey::CapreqAction => (
            "Which capability-request action should I run?".to_string(),
            vec![
                "create_draft".to_string(),
                "submit_for_approval".to_string(),
                "approve".to_string(),
            ],
        ),
        FieldKey::CapreqId => (
            "Which capability request ID is this for?".to_string(),
            vec![
                "capreq_abc123".to_string(),
                "capreq_tenant_1_payroll".to_string(),
                "capreq_store_17_mgr".to_string(),
            ],
        ),
        FieldKey::TargetScopeRef => (
            "What target scope should this apply to?".to_string(),
            vec![
                "store_17".to_string(),
                "team.finance".to_string(),
                "tenant_default".to_string(),
            ],
        ),
        FieldKey::Justification => (
            "What is the justification?".to_string(),
            vec![
                "Monthly payroll processing".to_string(),
                "Need temporary manager coverage".to_string(),
                "Required for onboarding completion".to_string(),
            ],
        ),
        _ => (
            "Can you clarify that?".to_string(),
            vec![
                "One short sentence".to_string(),
                "A few keywords".to_string(),
            ],
        ),
    };
    Ph1dClarify::v1(
        q,
        vec![primary],
        formats,
        reason_codes::D_CLARIFY_EVIDENCE_REQUIRED,
    )
}

fn select_primary_missing(missing: &[FieldKey]) -> FieldKey {
    // Same deterministic priority as PH1.NLP skeleton.
    for k in [
        FieldKey::IntentChoice,
        FieldKey::ReferenceTarget,
        FieldKey::CapreqAction,
        FieldKey::CapreqId,
        FieldKey::RequestedCapabilityId,
        FieldKey::TargetScopeRef,
        FieldKey::Justification,
        FieldKey::TenantId,
        FieldKey::Amount,
        FieldKey::Recipient,
        FieldKey::Task,
        FieldKey::When,
    ] {
        if missing.contains(&k) {
            return k;
        }
    }
    missing.first().copied().unwrap_or(FieldKey::Task)
}

fn parse_reason_code(obj: &serde_json::Map<String, Value>) -> Option<ReasonCodeId> {
    let n = obj.get("reason_code")?.as_u64()?;
    if n == 0 || n > u32::MAX as u64 {
        return None;
    }
    Some(ReasonCodeId(n as u32))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;

    use selene_kernel_contracts::ph1c::LanguageTag;
    use selene_kernel_contracts::ph1c::{ConfidenceBucket, SessionStateRef, TranscriptOk};
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1e::{ToolCatalogRef, ToolName};
    use selene_kernel_contracts::ph1n::{Chat, Ph1nResponse};
    use selene_kernel_contracts::ph1w::SessionState;

    fn req(transcript: &str) -> Ph1dRequest {
        let ok = TranscriptOk::v1(
            transcript.to_string(),
            LanguageTag::new("en").unwrap(),
            ConfidenceBucket::High,
        )
        .unwrap();
        Ph1dRequest::v1(
            ok,
            Ph1nResponse::Chat(
                Chat::v1("hi".to_string(), selene_kernel_contracts::ReasonCodeId(1)).unwrap(),
            ),
            SessionStateRef::v1(SessionState::Active, false),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
            ToolCatalogRef::v1(vec![ToolName::Time, ToolName::Weather]).unwrap(),
        )
        .unwrap()
    }

    fn provider_req(provider_task: Ph1dProviderTask, provider_id: &str) -> Ph1dProviderCallRequest {
        Ph1dProviderCallRequest::v1(
            4101,
            7101,
            "tenant_1".to_string(),
            selene_kernel_contracts::ph1d::RequestId(9201),
            "idem_9201".to_string(),
            provider_task,
            selene_kernel_contracts::ph1d::Ph1dProviderRouteClass::Primary,
            provider_id.to_string(),
            "model_primary".to_string(),
            4_000,
            2,
            None,
            None,
            selene_kernel_contracts::SchemaVersion(1),
            PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1,
            selene_kernel_contracts::ph1d::SchemaHash(8101),
            selene_kernel_contracts::ph1d::SchemaHash(8102),
            None,
            "audio_ref_1".to_string(),
            selene_kernel_contracts::ph1d::Ph1dProviderInputPayloadKind::Audio,
            selene_kernel_contracts::ph1d::SchemaHash(8103),
            Some("{\"audio_ref\":\"audio_ref_1\"}".to_string()),
            Some("application/json".to_string()),
            SafetyTier::Standard,
            false,
            false,
        )
        .unwrap()
    }

    fn spawn_one_shot_http_server(
        status: u16,
        response_body: &'static str,
    ) -> (String, Arc<Mutex<String>>, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let request_capture = Arc::new(Mutex::new(String::new()));
        let request_capture_thread = Arc::clone(&request_capture);
        let handle = thread::spawn(move || {
            if let Ok((mut stream, _peer)) = listener.accept() {
                let mut header_buf = vec![0_u8; 0];
                let mut chunk = [0_u8; 1024];
                let mut header_end = None;
                loop {
                    let n = stream.read(&mut chunk).unwrap_or(0);
                    if n == 0 {
                        break;
                    }
                    header_buf.extend_from_slice(&chunk[..n]);
                    if let Some(pos) = header_buf.windows(4).position(|w| w == b"\r\n\r\n") {
                        header_end = Some(pos + 4);
                        break;
                    }
                    if header_buf.len() > 32 * 1024 {
                        break;
                    }
                }
                let (captured, content_len, consumed_after_header) = if let Some(end) = header_end {
                    let captured = String::from_utf8_lossy(&header_buf[..end]).to_string();
                    let content_len = captured
                        .lines()
                        .find_map(|line| {
                            let lower = line.to_ascii_lowercase();
                            lower
                                .strip_prefix("content-length:")
                                .map(|v| v.trim().parse::<usize>().ok())
                                .flatten()
                        })
                        .unwrap_or(0);
                    let consumed_after_header = header_buf.len().saturating_sub(end);
                    (captured, content_len, consumed_after_header)
                } else {
                    (String::from_utf8_lossy(&header_buf).to_string(), 0, 0)
                };
                *request_capture_thread.lock().unwrap() = captured;

                if content_len > consumed_after_header {
                    let mut remaining = content_len - consumed_after_header;
                    while remaining > 0 {
                        let mut body_chunk = [0_u8; 1024];
                        let n = stream.read(&mut body_chunk).unwrap_or(0);
                        if n == 0 {
                            break;
                        }
                        remaining = remaining.saturating_sub(n);
                    }
                }

                let status_line = match status {
                    200 => "HTTP/1.1 200 OK",
                    503 => "HTTP/1.1 503 Service Unavailable",
                    429 => "HTTP/1.1 429 Too Many Requests",
                    _ => "HTTP/1.1 400 Bad Request",
                };
                let resp = format!(
                    "{status_line}\r\ncontent-type: application/json\r\nx-request-id: req_live_1\r\ncontent-length: {}\r\n\r\n{}",
                    response_body.len(),
                    response_body
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        });
        (format!("http://{}", addr), request_capture, handle)
    }

    fn live_config_for_endpoint(endpoint: String) -> Ph1dLiveProviderAdapterConfig {
        Ph1dLiveProviderAdapterConfig {
            openai_api_key: Some("test-openai-key".to_string()),
            google_api_key: Some("test-google-key".to_string()),
            openai_ocr_endpoint: endpoint.clone(),
            openai_stt_endpoint: endpoint.clone(),
            openai_tts_endpoint: endpoint.clone(),
            google_ocr_endpoint: endpoint.clone(),
            google_stt_endpoint: endpoint.clone(),
            google_tts_endpoint: endpoint,
            connect_timeout_ms: 1_000,
        }
    }

    #[test]
    fn at_d_01_one_mode_only_rejects_mixed_output() {
        let rt = Ph1dRuntime::new(Ph1dConfig::mvp_v1());
        let out = rt.run(
            &req("hello"),
            ModelCallOutcome::Ok {
                raw_json: r#"{"mode":"chat","response_text":"hi","reason_code":1,"tool_name":"web_search"}"#
                    .to_string(),
            },
        );
        assert!(
            matches!(out, Ph1dResponse::Fail(f) if f.reason_code == reason_codes::D_FAIL_FORBIDDEN_OUTPUT)
        );
    }

    #[test]
    fn at_d_02_schema_or_nothing_rejects_malformed_json() {
        let rt = Ph1dRuntime::new(Ph1dConfig::mvp_v1());
        let out = rt.run(
            &req("hello"),
            ModelCallOutcome::Ok {
                raw_json: "not json".to_string(),
            },
        );
        assert!(
            matches!(out, Ph1dResponse::Fail(f) if f.reason_code == reason_codes::D_FAIL_INVALID_SCHEMA)
        );
    }

    #[test]
    fn at_d_03_no_tool_injection_rejects_extra_keys() {
        let rt = Ph1dRuntime::new(Ph1dConfig::mvp_v1());
        let out = rt.run(
            &req("hello"),
            ModelCallOutcome::Ok {
                raw_json: r#"{"mode":"intent","intent_type":"SetReminder","field_refinements":[],"missing_fields":["When"],"reason_code":1,"tool_name":"time"}"#.to_string(),
            },
        );
        assert!(
            matches!(out, Ph1dResponse::Fail(f) if f.reason_code == reason_codes::D_FAIL_FORBIDDEN_OUTPUT)
        );
    }

    #[test]
    fn at_d_04_no_silent_assumptions_evidence_violation_forces_clarify() {
        let rt = Ph1dRuntime::new(Ph1dConfig::mvp_v1());
        let out = rt.run(
            &req("schedule it"),
            ModelCallOutcome::Ok {
                raw_json: r#"{
                  "mode":"intent",
                  "intent_type":"CreateCalendarEvent",
                  "field_refinements":[{"field":"When","original_span":"tomorrow 3pm","normalized_value":null,
                    "evidence_span":{"field":"When","transcript_hash":123,"start_byte":0,"end_byte":3,"verbatim_excerpt":"tomorrow 3pm"}}],
                  "missing_fields":["When"],
                  "reason_code":1
                }"#.to_string(),
            },
        );
        assert!(matches!(out, Ph1dResponse::Ok(Ph1dOk::Clarify(_))));
    }

    #[test]
    fn at_d_05_no_authority_invention_rejected() {
        let rt = Ph1dRuntime::new(Ph1dConfig::mvp_v1());
        let out = rt.run(
            &req("hello"),
            ModelCallOutcome::Ok {
                raw_json: r#"{"mode":"chat","response_text":"Approved. Permission granted.","reason_code":1}"#
                    .to_string(),
            },
        );
        assert!(
            matches!(out, Ph1dResponse::Fail(f) if f.reason_code == reason_codes::D_FAIL_FORBIDDEN_OUTPUT)
        );
    }

    #[test]
    fn at_d_06_timeout_returns_d_fail_timeout() {
        let rt = Ph1dRuntime::new(Ph1dConfig::mvp_v1());
        let out = rt.run(&req("hello"), ModelCallOutcome::Timeout);
        assert!(
            matches!(out, Ph1dResponse::Fail(f) if f.reason_code == reason_codes::D_FAIL_TIMEOUT)
        );
    }

    #[test]
    fn at_d_07_reason_code_required_rejects_missing() {
        let rt = Ph1dRuntime::new(Ph1dConfig::mvp_v1());
        let out = rt.run(
            &req("hello"),
            ModelCallOutcome::Ok {
                raw_json: r#"{"mode":"chat","response_text":"hi"}"#.to_string(),
            },
        );
        assert!(
            matches!(out, Ph1dResponse::Fail(f) if f.reason_code == reason_codes::D_FAIL_INVALID_SCHEMA)
        );
    }

    #[test]
    fn at_d_07b_reason_code_zero_is_rejected() {
        let rt = Ph1dRuntime::new(Ph1dConfig::mvp_v1());
        let out = rt.run(
            &req("hello"),
            ModelCallOutcome::Ok {
                raw_json: r#"{"mode":"chat","response_text":"hi","reason_code":0}"#.to_string(),
            },
        );
        assert!(
            matches!(out, Ph1dResponse::Fail(f) if f.reason_code == reason_codes::D_FAIL_INVALID_SCHEMA)
        );
    }

    #[test]
    fn at_d_10_no_internal_leakage_in_chat() {
        let rt = Ph1dRuntime::new(Ph1dConfig::mvp_v1());
        let out = rt.run(
            &req("hello"),
            ModelCallOutcome::Ok {
                raw_json: r#"{"mode":"chat","response_text":"I used web search.","reason_code":1}"#
                    .to_string(),
            },
        );
        assert!(
            matches!(out, Ph1dResponse::Fail(f) if f.reason_code == reason_codes::D_FAIL_FORBIDDEN_OUTPUT)
        );
    }

    #[test]
    fn at_d_08_evidence_spans_machine_precise_allows_refinement() {
        let rt = Ph1dRuntime::new(Ph1dConfig::mvp_v1());
        let transcript = "remind me tomorrow at 3pm";
        let r = req(transcript);
        let excerpt = "tomorrow at 3pm";
        let start = transcript.find(excerpt).unwrap();
        let end = start + excerpt.len();

        let raw_json = format!(
            r#"{{
              "mode":"intent",
              "intent_type":"SetReminder",
              "field_refinements":[{{
                "field":"When",
                "original_span":"{excerpt}",
                "normalized_value":null,
                "evidence_span":{{
                  "field":"When",
                  "transcript_hash":{th},
                  "start_byte":{start},
                  "end_byte":{end},
                  "verbatim_excerpt":"{excerpt}"
                }}
              }}],
              "missing_fields":[],
              "reason_code":1
            }}"#,
            excerpt = excerpt,
            th = r.transcript_hash.0,
            start = start,
            end = end
        );

        let out = rt.run(&r, ModelCallOutcome::Ok { raw_json });
        match out {
            Ph1dResponse::Ok(Ph1dOk::Intent(i)) => {
                assert_eq!(i.field_refinements.len(), 1);
                assert_eq!(i.missing_fields.len(), 0);
            }
            other => panic!("expected Ok(Intent), got: {other:?}"),
        }
    }

    #[test]
    fn at_d_08b_original_span_mismatch_forces_clarify() {
        let rt = Ph1dRuntime::new(Ph1dConfig::mvp_v1());
        let transcript = "remind me tomorrow at 3pm";
        let r = req(transcript);
        let verbatim = "tomorrow at 3pm";
        let start = transcript.find(verbatim).unwrap();
        let end = start + verbatim.len();

        // original_span != verbatim_excerpt => evidence discipline violation => clarify (not intent, not fail).
        let raw_json = format!(
            r#"{{
              "mode":"intent",
              "intent_type":"SetReminder",
              "field_refinements":[{{
                "field":"When",
                "original_span":"tomorrow 3pm",
                "normalized_value":null,
                "evidence_span":{{
                  "field":"When",
                  "transcript_hash":{th},
                  "start_byte":{start},
                  "end_byte":{end},
                  "verbatim_excerpt":"{verbatim}"
                }}
              }}],
              "missing_fields":[],
              "reason_code":1
            }}"#,
            th = r.transcript_hash.0,
            start = start,
            end = end,
            verbatim = verbatim
        );

        let out = rt.run(&r, ModelCallOutcome::Ok { raw_json });
        assert!(matches!(out, Ph1dResponse::Ok(Ph1dOk::Clarify(_))));
    }

    #[test]
    fn at_d_09_clarify_requires_exactly_one_missing_field() {
        let rt = Ph1dRuntime::new(Ph1dConfig::mvp_v1());
        let out = rt.run(
            &req("hello"),
            ModelCallOutcome::Ok {
                raw_json: r#"{
                  "mode":"clarify",
                  "question":"When exactly?",
                  "what_is_missing":["When","Task"],
                  "accepted_answer_formats":["Tomorrow at 3pm","Friday 10am"],
                  "reason_code":1
                }"#
                .to_string(),
            },
        );
        assert!(
            matches!(out, Ph1dResponse::Fail(f) if f.reason_code == reason_codes::D_FAIL_INVALID_SCHEMA)
        );
    }

    #[test]
    fn at_d_11_envelope_integrity_is_enforced_before_model_call() {
        let rt = Ph1dRuntime::new(Ph1dConfig::mvp_v1());
        let mut r = req("hello");
        r.transcript_hash = TranscriptHash(r.transcript_hash.0.wrapping_add(1));

        let out = rt.run(
            &r,
            ModelCallOutcome::Ok {
                raw_json: r#"{"mode":"chat","response_text":"hi","reason_code":1}"#.to_string(),
            },
        );
        assert!(
            matches!(out, Ph1dResponse::Fail(f) if f.reason_code == reason_codes::D_FAIL_INVALID_SCHEMA)
        );
    }

    #[test]
    fn at_d_provider_boundary_01_openai_stt_normalizes_to_shared_schema() {
        let req = provider_req(Ph1dProviderTask::SttTranscribe, "openai");
        let out = normalize_provider_transport_outcome(
            &req,
            Ph1dSpeechProviderVendor::OpenAi,
            Ph1dProviderTransportOutcome::Ok {
                provider_call_id: Some("openai_call_1".to_string()),
                raw_payload_json: r#"{"task":"stt.transcribe","text":"hello there","language":"en-US","confidence_bp":9500,"stable":true}"#.to_string(),
                provider_latency_ms: 121,
                provider_cost_microunits: 17,
                provider_confidence_bp: Some(9500),
            },
        )
        .unwrap();
        assert_eq!(out.provider_status, Ph1dProviderStatus::Ok);
        assert_eq!(
            out.validation_status,
            Ph1dProviderValidationStatus::SchemaOk
        );
        assert_eq!(
            out.normalized_output_schema_hash,
            Some(PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1)
        );
        let normalized =
            decode_normalized_output_json(out.normalized_output_json.as_deref().unwrap()).unwrap();
        assert_eq!(normalized.provider_task, Ph1dProviderTask::SttTranscribe);
        assert_eq!(normalized.text_output.as_deref(), Some("hello there"));
        assert_eq!(normalized.language_tag.as_deref(), Some("en-US"));
        assert_eq!(normalized.stable, Some(true));
    }

    #[test]
    fn at_d_provider_boundary_02_google_tts_normalizes_to_shared_schema() {
        let req = provider_req(Ph1dProviderTask::TtsSynthesize, "google");
        let out = normalize_provider_transport_outcome(
            &req,
            Ph1dSpeechProviderVendor::Google,
            Ph1dProviderTransportOutcome::Ok {
                provider_call_id: Some("google_call_1".to_string()),
                raw_payload_json: r#"{"task":"text:synthesize","audio_uri":"gs://bucket/audio.wav","mime_type":"audio/wav","duration_ms":1400,"text":"ready"}"#.to_string(),
                provider_latency_ms: 98,
                provider_cost_microunits: 9,
                provider_confidence_bp: None,
            },
        )
        .unwrap();
        assert_eq!(out.provider_status, Ph1dProviderStatus::Ok);
        assert_eq!(
            out.validation_status,
            Ph1dProviderValidationStatus::SchemaOk
        );
        let normalized =
            decode_normalized_output_json(out.normalized_output_json.as_deref().unwrap()).unwrap();
        assert_eq!(normalized.provider_task, Ph1dProviderTask::TtsSynthesize);
        assert_eq!(
            normalized.audio_output_ref.as_deref(),
            Some("gs://bucket/audio.wav")
        );
        assert_eq!(normalized.audio_content_type.as_deref(), Some("audio/wav"));
        assert_eq!(normalized.estimated_duration_ms, Some(1400));
    }

    #[test]
    fn at_d_provider_boundary_03_timeout_fails_closed() {
        let req = provider_req(Ph1dProviderTask::SttTranscribe, "openai");
        let out = normalize_provider_transport_outcome(
            &req,
            Ph1dSpeechProviderVendor::OpenAi,
            Ph1dProviderTransportOutcome::Timeout {
                provider_call_id: Some("openai_call_timeout".to_string()),
                provider_latency_ms: 4_000,
            },
        )
        .unwrap();
        assert_eq!(out.provider_status, Ph1dProviderStatus::Error);
        assert_eq!(
            out.validation_status,
            Ph1dProviderValidationStatus::SchemaFail
        );
        assert_eq!(out.reason_code, reason_codes::D_PROVIDER_TIMEOUT);
        assert!(out.normalized_output_json.is_none());
    }

    #[test]
    fn at_d_provider_boundary_04_vendor_contract_mismatch_fails_closed() {
        let req = provider_req(Ph1dProviderTask::SttTranscribe, "google");
        let out = normalize_provider_transport_outcome(
            &req,
            Ph1dSpeechProviderVendor::OpenAi,
            Ph1dProviderTransportOutcome::Ok {
                provider_call_id: Some("call_mismatch".to_string()),
                raw_payload_json: r#"{"task":"stt.transcribe","text":"hello","language":"en-US","confidence_bp":9000,"stable":true}"#.to_string(),
                provider_latency_ms: 66,
                provider_cost_microunits: 3,
                provider_confidence_bp: Some(9000),
            },
        )
        .unwrap();
        assert_eq!(out.provider_status, Ph1dProviderStatus::Error);
        assert_eq!(
            out.validation_status,
            Ph1dProviderValidationStatus::SchemaFail
        );
        assert_eq!(out.reason_code, reason_codes::D_PROVIDER_CONTRACT_MISMATCH);
    }

    #[test]
    fn at_d_provider_boundary_05_schema_drift_fails_closed() {
        let req = provider_req(Ph1dProviderTask::SttTranscribe, "openai");
        let out = normalize_provider_transport_outcome(
            &req,
            Ph1dSpeechProviderVendor::OpenAi,
            Ph1dProviderTransportOutcome::Ok {
                provider_call_id: Some("openai_call_drift".to_string()),
                raw_payload_json:
                    r#"{"task":"stt.transcribe","text":"hello there","language":"en-US"}"#
                        .to_string(),
                provider_latency_ms: 101,
                provider_cost_microunits: 7,
                provider_confidence_bp: Some(9100),
            },
        )
        .unwrap();
        assert_eq!(out.provider_status, Ph1dProviderStatus::Error);
        assert_eq!(
            out.validation_status,
            Ph1dProviderValidationStatus::SchemaFail
        );
        assert_eq!(out.reason_code, reason_codes::D_PROVIDER_SCHEMA_DRIFT);
        assert!(out.normalized_output_json.is_none());
    }

    #[test]
    fn at_d_provider_live_01_http_round_trip_openai_stt_normalizes_output() {
        let (endpoint, request_capture, server) = spawn_one_shot_http_server(
            200,
            r#"{"task":"stt.transcribe","text":"hello from live path","language":"en-US","confidence_bp":9400,"stable":true}"#,
        );
        let adapter = Ph1dLiveProviderAdapter::new(live_config_for_endpoint(endpoint)).unwrap();
        let req = provider_req(Ph1dProviderTask::SttTranscribe, "openai");
        let out = adapter.execute(&req).unwrap();
        server.join().unwrap();

        assert_eq!(out.provider_status, Ph1dProviderStatus::Ok);
        assert_eq!(
            out.validation_status,
            Ph1dProviderValidationStatus::SchemaOk
        );
        assert_eq!(
            out.normalized_output_schema_hash,
            Some(PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1)
        );
        let normalized =
            decode_normalized_output_json(out.normalized_output_json.as_deref().unwrap()).unwrap();
        assert_eq!(
            normalized.text_output.as_deref(),
            Some("hello from live path")
        );
        assert_eq!(normalized.language_tag.as_deref(), Some("en-US"));
        assert_eq!(normalized.stable, Some(true));

        let captured = request_capture.lock().unwrap().clone();
        assert!(captured
            .to_ascii_lowercase()
            .contains("authorization: bearer test-openai-key"));
        assert!(captured
            .to_ascii_lowercase()
            .contains("idempotency-key: idem_9201"));
    }

    #[test]
    fn at_d_provider_live_02_missing_provider_key_fails_closed_contract_mismatch() {
        let endpoint = "http://127.0.0.1:1".to_string();
        let mut cfg = live_config_for_endpoint(endpoint);
        cfg.openai_api_key = None;
        let adapter = Ph1dLiveProviderAdapter::new(cfg).unwrap();
        let req = provider_req(Ph1dProviderTask::SttTranscribe, "openai");
        let out = adapter.execute(&req).unwrap();
        assert_eq!(out.provider_status, Ph1dProviderStatus::Error);
        assert_eq!(
            out.validation_status,
            Ph1dProviderValidationStatus::SchemaFail
        );
        assert_eq!(out.reason_code, reason_codes::D_PROVIDER_CONTRACT_MISMATCH);
    }

    #[test]
    fn at_d_provider_live_03_http_503_maps_to_timeout_fail_closed() {
        let (endpoint, _request_capture, server) =
            spawn_one_shot_http_server(503, r#"{"error":"provider unavailable"}"#);
        let adapter = Ph1dLiveProviderAdapter::new(live_config_for_endpoint(endpoint)).unwrap();
        let req = provider_req(Ph1dProviderTask::SttTranscribe, "openai");
        let out = adapter.execute(&req).unwrap();
        server.join().unwrap();
        assert_eq!(out.provider_status, Ph1dProviderStatus::Error);
        assert_eq!(
            out.validation_status,
            Ph1dProviderValidationStatus::SchemaFail
        );
        assert_eq!(out.reason_code, reason_codes::D_PROVIDER_TIMEOUT);
    }
}
