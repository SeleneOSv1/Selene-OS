#![forbid(unsafe_code)]

use std::env;
use std::time::{Duration, Instant};

use selene_engines::device_vault;
use selene_engines::ph1write::{Ph1WriteConfig, Ph1WriteRuntime};
use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
use selene_kernel_contracts::ph1l::SessionId;
use selene_kernel_contracts::ph1position::TenantId;
use selene_kernel_contracts::ph1write::{Ph1WriteRequest, Ph1WriteResponse, WriteRenderStyle};
use selene_kernel_contracts::provider_secrets::ProviderSecretId;
use selene_kernel_contracts::{MonotonicTimeNs, Validate};

const SLICE1_PATH: [&str; 8] = [
    "PH1.API",
    "PH1.GATEWAY",
    "PH1.CONV",
    "PH1.X",
    "PH1.PROVIDERS",
    "PH1.OAI",
    "PH1.WRITE",
    "PH1.OBS",
];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Slice1TextConversationRequest {
    pub correlation_id: u64,
    pub turn_id: u64,
    pub device_turn_sequence: Option<u64>,
    pub app_platform: String,
    pub actor_user_id: String,
    pub tenant_id: Option<String>,
    pub device_id: Option<String>,
    pub now_ns: Option<u64>,
    pub thread_key: Option<String>,
    pub user_text_final: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Slice1TextConversationResponse {
    pub status: String,
    pub outcome: String,
    pub session_id: Option<String>,
    pub turn_id: Option<u64>,
    pub session_state: Option<String>,
    pub session_attach_outcome: Option<String>,
    pub failure_class: Option<String>,
    pub reason: Option<String>,
    pub next_move: String,
    pub response_text: String,
    pub reason_code: String,
    pub provenance: Option<Slice1Provenance>,
    #[serde(default)]
    pub tts_text: String,
    #[serde(default)]
    pub source_chips: Vec<serde_json::Value>,
    #[serde(default)]
    pub image_cards: Vec<serde_json::Value>,
    #[serde(default)]
    pub answer_class: Option<String>,
    #[serde(default)]
    pub metadata_safe_for_user: bool,
    pub trace_id: Option<String>,
    pub slice1_trace: Slice1Trace,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Slice1Provenance {
    pub sources: Vec<serde_json::Value>,
    pub retrieved_at: u64,
    pub cache_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Slice1Trace {
    pub trace_id: String,
    pub request_id: String,
    pub correlation_id: u64,
    pub session_id: String,
    pub turn_id: u64,
    pub path: Vec<String>,
    pub desktop_direct_provider_call: bool,
    pub adapter_monolith_provider_execution: bool,
    pub old_ph1os_turn_orchestration: bool,
    pub ph1x_validation_before_provider: bool,
    pub ph1write_final_output: bool,
    pub latency_ms: u64,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub safe_provider_metadata: Vec<Slice1TraceField>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Slice1TraceField {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Slice1ErrorClass {
    InvalidPayload,
    PolicyViolation,
    ProviderFailure,
    WriteFailure,
}

impl Slice1ErrorClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Slice1ErrorClass::InvalidPayload => "INVALID_PAYLOAD",
            Slice1ErrorClass::PolicyViolation => "POLICY_VIOLATION",
            Slice1ErrorClass::ProviderFailure => "PROVIDER_FAILURE",
            Slice1ErrorClass::WriteFailure => "WRITE_FAILURE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slice1Error {
    pub class: Slice1ErrorClass,
    pub reason_code: &'static str,
    pub safe_reason: String,
    pub trace: Option<Slice1Trace>,
}

impl Slice1Error {
    fn invalid(reason_code: &'static str, safe_reason: impl Into<String>) -> Self {
        Self {
            class: Slice1ErrorClass::InvalidPayload,
            reason_code,
            safe_reason: safe_reason.into(),
            trace: None,
        }
    }

    fn policy(
        reason_code: &'static str,
        safe_reason: impl Into<String>,
        trace: Slice1Trace,
    ) -> Self {
        Self {
            class: Slice1ErrorClass::PolicyViolation,
            reason_code,
            safe_reason: safe_reason.into(),
            trace: Some(trace),
        }
    }

    fn provider(
        reason_code: &'static str,
        safe_reason: impl Into<String>,
        trace: Slice1Trace,
    ) -> Self {
        Self {
            class: Slice1ErrorClass::ProviderFailure,
            reason_code,
            safe_reason: safe_reason.into(),
            trace: Some(trace),
        }
    }

    fn write(
        reason_code: &'static str,
        safe_reason: impl Into<String>,
        trace: Slice1Trace,
    ) -> Self {
        Self {
            class: Slice1ErrorClass::WriteFailure,
            reason_code,
            safe_reason: safe_reason.into(),
            trace: Some(trace),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slice1ProviderResult {
    pub provider_id: String,
    pub model_id: String,
    pub response_text: String,
    pub safe_metadata: Vec<(String, String)>,
}

pub trait Slice1Provider {
    fn complete_text(
        &self,
        request: &Slice1ProviderRequest,
    ) -> Result<Slice1ProviderResult, String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slice1ProviderRequest {
    pub request_id: String,
    pub trace_id: String,
    pub user_text: String,
}

#[derive(Debug, Clone)]
pub struct OpenAiSlice1Provider {
    endpoint: String,
    model: String,
    api_key: String,
    api_key_source: Slice1ProviderKeySource,
    timeout: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slice1ProviderKeySource {
    Env,
    DeviceVault,
}

impl Slice1ProviderKeySource {
    fn as_str(self) -> &'static str {
        match self {
            Self::Env => "env",
            Self::DeviceVault => "device_vault",
        }
    }
}

impl OpenAiSlice1Provider {
    pub fn from_env() -> Result<Self, String> {
        let (api_key, api_key_source) = resolve_openai_api_key()?;
        let endpoint = env::var("SELENE_SLICE1_OPENAI_RESPONSES_URL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .or_else(|| {
                env::var("OPENAI_RESPONSES_URL")
                    .ok()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
            })
            .unwrap_or_else(|| "https://api.openai.com/v1/responses".to_string());
        let model = env::var("SELENE_SLICE1_OPENAI_MODEL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .or_else(|| {
                env::var("OPENAI_MODEL")
                    .ok()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
            })
            .unwrap_or_else(|| "gpt-5.5".to_string());
        let timeout_secs = env::var("SELENE_SLICE1_OPENAI_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(45)
            .clamp(1, 120);
        Ok(Self {
            endpoint,
            model,
            api_key,
            api_key_source,
            timeout: Duration::from_secs(timeout_secs),
        })
    }
}

fn resolve_openai_api_key() -> Result<(String, Slice1ProviderKeySource), String> {
    if let Some(api_key) = env::var("OPENAI_API_KEY")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return Ok((api_key, Slice1ProviderKeySource::Env));
    }

    match device_vault::resolve_secret(ProviderSecretId::OpenAIApiKey.as_str()) {
        Ok(Some(api_key)) => Ok((api_key, Slice1ProviderKeySource::DeviceVault)),
        Ok(None) => Err("missing_openai_api_key".to_string()),
        Err(_) => Err("openai_api_key_vault_read_failed".to_string()),
    }
}

impl Slice1Provider for OpenAiSlice1Provider {
    fn complete_text(
        &self,
        request: &Slice1ProviderRequest,
    ) -> Result<Slice1ProviderResult, String> {
        let body = serde_json::json!({
            "model": self.model,
            "input": [
                {
                    "role": "system",
                    "content": "You are Selene Slice 1. Answer normally and concisely. Do not claim to execute protected business actions."
                },
                {
                    "role": "user",
                    "content": request.user_text
                }
            ],
            "metadata": {
                "selene_trace_id": request.trace_id,
                "selene_slice": "probabilistic_core_slice_1"
            }
        });
        let agent = ureq::AgentBuilder::new().timeout(self.timeout).build();
        let response = agent
            .post(&self.endpoint)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .send_json(body)
            .map_err(|_| "openai_responses_request_failed".to_string())?;
        let value: serde_json::Value = response
            .into_json()
            .map_err(|_| "openai_responses_json_decode_failed".to_string())?;
        let response_text = extract_openai_output_text(&value)
            .ok_or_else(|| "openai_responses_missing_output_text".to_string())?;
        Ok(Slice1ProviderResult {
            provider_id: "openai".to_string(),
            model_id: self.model.clone(),
            response_text,
            safe_metadata: vec![
                (
                    "provider_boundary".to_string(),
                    "PH1.PROVIDERS/PH1.OAI".to_string(),
                ),
                ("endpoint_kind".to_string(), "responses".to_string()),
                (
                    "provider_key_source".to_string(),
                    self.api_key_source.as_str().to_string(),
                ),
            ],
        })
    }
}

pub fn run_slice1_text_conversation_from_env(
    request: Slice1TextConversationRequest,
    request_id: String,
) -> Result<Slice1TextConversationResponse, Slice1Error> {
    let provider = OpenAiSlice1Provider::from_env().map_err(|reason| {
        Slice1Error::provider(
            "SLICE1_PROVIDER_CONFIG_FAIL",
            reason.clone(),
            provider_config_error_trace(&request, &request_id, &reason),
        )
    })?;
    run_slice1_text_conversation(request, request_id, &provider)
}

fn provider_config_error_trace(
    request: &Slice1TextConversationRequest,
    request_id: &str,
    reason: &str,
) -> Slice1Trace {
    let mut trace = initial_trace(request, request_id, 0);
    trace.safe_provider_metadata.push(Slice1TraceField {
        key: "provider_key_source".to_string(),
        value: "missing".to_string(),
    });
    trace.safe_provider_metadata.push(Slice1TraceField {
        key: "provider_config_error".to_string(),
        value: reason.to_string(),
    });
    trace
}

pub fn run_slice1_text_conversation<P: Slice1Provider>(
    request: Slice1TextConversationRequest,
    request_id: String,
    provider: &P,
) -> Result<Slice1TextConversationResponse, Slice1Error> {
    let started = Instant::now();
    let now_ns = request.now_ns.unwrap_or(request.correlation_id).max(1);
    validate_request(&request, &request_id)?;
    let mut trace = initial_trace(&request, &request_id, now_ns);

    let conv = ph1conv_turn_packet(&request, &trace.trace_id)?;
    let x = ph1x_validate_slice1(&conv, &mut trace)?;
    if x.protected_like {
        trace.latency_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
        return Err(Slice1Error::policy(
            "SLICE1_PROTECTED_BLOCKED_BY_PH1X",
            "PH1.X blocked protected-looking text in Slice 1; protected execution is out of scope",
            trace,
        ));
    }

    let provider_request = Slice1ProviderRequest {
        request_id: request_id.clone(),
        trace_id: trace.trace_id.clone(),
        user_text: conv.user_text.clone(),
    };
    let provider_result = provider
        .complete_text(&provider_request)
        .map_err(|reason| {
            trace.latency_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            Slice1Error::provider("SLICE1_PROVIDER_FAIL_CLOSED", reason, trace.clone())
        })?;

    trace.provider_id = Some(provider_result.provider_id.clone());
    trace.model_id = Some(provider_result.model_id.clone());
    trace.safe_provider_metadata = provider_result
        .safe_metadata
        .iter()
        .map(|(key, value)| Slice1TraceField {
            key: key.clone(),
            value: value.clone(),
        })
        .collect();

    let write_text =
        ph1write_finalize(&request, now_ns, &provider_result.response_text).map_err(|reason| {
            trace.latency_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            Slice1Error::write("SLICE1_PH1WRITE_FAIL_CLOSED", reason, trace.clone())
        })?;
    trace.ph1write_final_output = true;
    trace.latency_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;

    Ok(Slice1TextConversationResponse {
        status: "ok".to_string(),
        outcome: "SLICE1_TEXT_CONVERSATION_COMPLETED".to_string(),
        session_id: Some(trace.session_id.clone()),
        turn_id: Some(request.turn_id),
        session_state: Some("Active".to_string()),
        session_attach_outcome: Some("ExistingSessionAttached".to_string()),
        failure_class: None,
        reason: None,
        next_move: "render_ph1write_output".to_string(),
        response_text: write_text,
        reason_code: "SLICE1_OK".to_string(),
        provenance: Some(Slice1Provenance {
            sources: Vec::new(),
            retrieved_at: now_ns,
            cache_status: "not_applicable_slice1_no_search".to_string(),
        }),
        tts_text: String::new(),
        source_chips: Vec::new(),
        image_cards: Vec::new(),
        answer_class: Some("probabilistic_public_answer".to_string()),
        metadata_safe_for_user: true,
        trace_id: Some(trace.trace_id.clone()),
        slice1_trace: trace,
    })
}

fn validate_request(
    request: &Slice1TextConversationRequest,
    request_id: &str,
) -> Result<(), Slice1Error> {
    if request_id.trim().is_empty() || request_id.len() > 160 {
        return Err(Slice1Error::invalid(
            "SLICE1_INVALID_REQUEST_ID",
            "request id must be non-empty and bounded",
        ));
    }
    if request.correlation_id == 0 || request.turn_id == 0 {
        return Err(Slice1Error::invalid(
            "SLICE1_INVALID_IDS",
            "correlation_id and turn_id must be non-zero",
        ));
    }
    if request.actor_user_id.trim().is_empty() || request.actor_user_id.len() > 128 {
        return Err(Slice1Error::invalid(
            "SLICE1_INVALID_ACTOR",
            "actor_user_id must be non-empty and bounded",
        ));
    }
    let device_id = request.device_id.as_deref().unwrap_or("desktop");
    if device_id.trim().is_empty() || device_id.len() > 128 {
        return Err(Slice1Error::invalid(
            "SLICE1_INVALID_DEVICE",
            "device_id must be non-empty and bounded",
        ));
    }
    if request.user_text_final.trim().is_empty() {
        return Err(Slice1Error::invalid(
            "SLICE1_EMPTY_TEXT",
            "typed text must not be empty",
        ));
    }
    if request.user_text_final.len() > 16_384 {
        return Err(Slice1Error::invalid(
            "SLICE1_TEXT_TOO_LARGE",
            "typed text exceeds Slice 1 bound",
        ));
    }
    if request.app_platform.trim() != "DESKTOP" {
        return Err(Slice1Error::invalid(
            "SLICE1_DESKTOP_ONLY",
            "Slice 1 only accepts Desktop typed text",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct Slice1ConversationTurn {
    user_text: String,
}

#[derive(Debug, Clone, Copy)]
struct Slice1XDecision {
    protected_like: bool,
}

fn ph1conv_turn_packet(
    request: &Slice1TextConversationRequest,
    trace_id: &str,
) -> Result<Slice1ConversationTurn, Slice1Error> {
    let text = request.user_text_final.trim().to_string();
    if text.is_empty() {
        return Err(Slice1Error::invalid(
            "SLICE1_CONV_EMPTY_TEXT",
            "PH1.CONV cannot build a turn from empty text",
        ));
    }
    let _conversation_turn_packet_id = format!("{trace_id}:PH1.CONV:turn");
    Ok(Slice1ConversationTurn { user_text: text })
}

fn ph1x_validate_slice1(
    conv: &Slice1ConversationTurn,
    trace: &mut Slice1Trace,
) -> Result<Slice1XDecision, Slice1Error> {
    let protected_like = looks_like_protected_action(&conv.user_text);
    trace.ph1x_validation_before_provider = true;
    Ok(Slice1XDecision { protected_like })
}

fn ph1write_finalize(
    request: &Slice1TextConversationRequest,
    now_ns: u64,
    provider_text: &str,
) -> Result<String, String> {
    let tenant = request
        .tenant_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("tenant_slice1");
    let device = request
        .device_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("desktop_slice1");
    let tenant_id = TenantId::new(tenant).map_err(|_| "ph1write_invalid_tenant".to_string())?;
    let user_id =
        UserId::new(&request.actor_user_id).map_err(|_| "ph1write_invalid_user".to_string())?;
    let device_id = DeviceId::new(device).map_err(|_| "ph1write_invalid_device".to_string())?;
    let session_id = Some(SessionId(session_id_u128(request)));
    let write_req = Ph1WriteRequest::v1(
        MonotonicTimeNs(now_ns),
        tenant_id,
        CorrelationId(u128::from(request.correlation_id)),
        TurnId(request.turn_id),
        session_id,
        user_id,
        device_id,
        provider_text.to_string(),
        WriteRenderStyle::Professional,
        Vec::new(),
        false,
        format!("slice1-write-{}", request.turn_id),
    )
    .map_err(|_| "ph1write_request_contract_failed".to_string())?;
    let writer = Ph1WriteRuntime::new(Ph1WriteConfig::mvp_v1());
    match writer.run(&write_req) {
        Ph1WriteResponse::Ok(ok) => {
            ok.validate()
                .map_err(|_| "ph1write_output_contract_failed".to_string())?;
            Ok(ok.formatted_text)
        }
        Ph1WriteResponse::Refuse(refuse) => {
            refuse
                .validate()
                .map_err(|_| "ph1write_refuse_contract_failed".to_string())?;
            Ok(refuse.refusal_text)
        }
    }
}

fn initial_trace(
    request: &Slice1TextConversationRequest,
    request_id: &str,
    now_ns: u64,
) -> Slice1Trace {
    let session_id = format!("slice1-session-{}", session_id_u128(request));
    Slice1Trace {
        trace_id: format!(
            "slice1-trace-{}-{}",
            request.correlation_id, request.turn_id
        ),
        request_id: request_id.to_string(),
        correlation_id: request.correlation_id,
        session_id,
        turn_id: request.turn_id,
        path: SLICE1_PATH.iter().map(|step| (*step).to_string()).collect(),
        desktop_direct_provider_call: false,
        adapter_monolith_provider_execution: false,
        old_ph1os_turn_orchestration: false,
        ph1x_validation_before_provider: false,
        ph1write_final_output: false,
        latency_ms: 0,
        provider_id: None,
        model_id: None,
        safe_provider_metadata: vec![Slice1TraceField {
            key: "created_at_ns".to_string(),
            value: now_ns.to_string(),
        }],
    }
}

fn session_id_u128(request: &Slice1TextConversationRequest) -> u128 {
    let seed = request
        .thread_key
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(&request.actor_user_id);
    let mut hash: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
    for byte in seed.as_bytes() {
        hash ^= u128::from(*byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    if hash == 0 {
        1
    } else {
        hash
    }
}

fn looks_like_protected_action(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let protected_verbs = [
        "pay ",
        "send money",
        "approve payroll",
        "run payroll",
        "post invoice",
        "change bank",
        "update inventory",
        "change salary",
        "approve leave",
        "delete customer",
    ];
    protected_verbs.iter().any(|needle| lower.contains(needle))
}

fn extract_openai_output_text(value: &serde_json::Value) -> Option<String> {
    if let Some(text) = value.get("output_text").and_then(|value| value.as_str()) {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    let mut fragments = Vec::new();
    if let Some(items) = value.get("output").and_then(|value| value.as_array()) {
        for item in items {
            if let Some(content) = item.get("content").and_then(|value| value.as_array()) {
                for part in content {
                    if let Some(text) = part.get("text").and_then(|value| value.as_str()) {
                        if !text.trim().is_empty() {
                            fragments.push(text.trim().to_string());
                        }
                    }
                }
            }
        }
    }
    if fragments.is_empty() {
        None
    } else {
        Some(fragments.join("\n"))
    }
}

pub fn slice1_error_response(
    error: Slice1Error,
    fallback_request_id: String,
) -> Slice1TextConversationResponse {
    let trace = error.trace.unwrap_or_else(|| Slice1Trace {
        trace_id: format!("slice1-trace-error-{fallback_request_id}"),
        request_id: fallback_request_id,
        correlation_id: 0,
        session_id: "slice1-session-unavailable".to_string(),
        turn_id: 0,
        path: SLICE1_PATH.iter().map(|step| (*step).to_string()).collect(),
        desktop_direct_provider_call: false,
        adapter_monolith_provider_execution: false,
        old_ph1os_turn_orchestration: false,
        ph1x_validation_before_provider: false,
        ph1write_final_output: false,
        latency_ms: 0,
        provider_id: None,
        model_id: None,
        safe_provider_metadata: Vec::new(),
    });
    Slice1TextConversationResponse {
        status: "error".to_string(),
        outcome: "SLICE1_TEXT_CONVERSATION_FAILED_CLOSED".to_string(),
        session_id: Some(trace.session_id.clone()),
        turn_id: if trace.turn_id == 0 {
            None
        } else {
            Some(trace.turn_id)
        },
        session_state: Some("Active".to_string()),
        session_attach_outcome: None,
        failure_class: Some(error.class.as_str().to_string()),
        reason: Some(error.safe_reason),
        next_move: "fail_closed".to_string(),
        response_text: String::new(),
        reason_code: error.reason_code.to_string(),
        provenance: None,
        tts_text: String::new(),
        source_chips: Vec::new(),
        image_cards: Vec::new(),
        answer_class: Some("probabilistic_public_answer".to_string()),
        metadata_safe_for_user: true,
        trace_id: Some(trace.trace_id.clone()),
        slice1_trace: trace,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_engines::device_vault::DeviceVault;
    use std::fs;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug)]
    struct MockProvider {
        response_text: String,
    }

    impl Slice1Provider for MockProvider {
        fn complete_text(
            &self,
            request: &Slice1ProviderRequest,
        ) -> Result<Slice1ProviderResult, String> {
            assert_eq!(request.trace_id, "slice1-trace-1001-2002");
            Ok(Slice1ProviderResult {
                provider_id: "mock_openai".to_string(),
                model_id: "mock-model".to_string(),
                response_text: self.response_text.clone(),
                safe_metadata: vec![(
                    "provider_boundary".to_string(),
                    "PH1.PROVIDERS/PH1.OAI".to_string(),
                )],
            })
        }
    }

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct ScopedEnvVar {
        key: &'static str,
        previous: Option<String>,
    }

    impl ScopedEnvVar {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = env::var(key).ok();
            env::set_var(key, value);
            Self { key, previous }
        }

        fn unset(key: &'static str) -> Self {
            let previous = env::var(key).ok();
            env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for ScopedEnvVar {
        fn drop(&mut self) {
            if let Some(value) = self.previous.as_deref() {
                env::set_var(self.key, value);
            } else {
                env::remove_var(self.key);
            }
        }
    }

    fn temp_vault_path(label: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(1);
        let base = env::temp_dir().join(format!("selene-slice1-vault-{label}-{suffix}"));
        fs::create_dir_all(&base).expect("temp vault directory should be created");
        (base.clone(), base.join("device_vault.json"))
    }

    fn request(text: &str) -> Slice1TextConversationRequest {
        Slice1TextConversationRequest {
            correlation_id: 1001,
            turn_id: 2002,
            device_turn_sequence: Some(1),
            app_platform: "DESKTOP".to_string(),
            actor_user_id: "tenant_a:user_1".to_string(),
            tenant_id: Some("tenant_a".to_string()),
            device_id: Some("tenant_a_device_1".to_string()),
            now_ns: Some(3003),
            thread_key: Some("thread_a".to_string()),
            user_text_final: text.to_string(),
        }
    }

    #[test]
    fn slice1_clean_path_produces_trace_and_ph1write_output() {
        let provider = MockProvider {
            response_text: " Hello from the provider. ".to_string(),
        };
        let response =
            run_slice1_text_conversation(request("say hello"), "req_slice1".to_string(), &provider)
                .expect("slice1 should complete");
        assert_eq!(response.status, "ok");
        assert_eq!(response.response_text, "Hello from the provider.");
        assert_eq!(response.slice1_trace.path, SLICE1_PATH);
        assert!(response.slice1_trace.ph1x_validation_before_provider);
        assert!(response.slice1_trace.ph1write_final_output);
        assert!(!response.slice1_trace.desktop_direct_provider_call);
        assert!(!response.slice1_trace.adapter_monolith_provider_execution);
        assert!(!response.slice1_trace.old_ph1os_turn_orchestration);
        assert_eq!(
            response.slice1_trace.provider_id.as_deref(),
            Some("mock_openai")
        );
    }

    #[test]
    fn slice1_blocks_protected_like_text_before_provider() {
        let provider = MockProvider {
            response_text: "should not be used".to_string(),
        };
        let error = run_slice1_text_conversation(
            request("approve payroll for Tim"),
            "req_slice1".to_string(),
            &provider,
        )
        .expect_err("protected-looking text should fail closed");
        assert_eq!(error.class, Slice1ErrorClass::PolicyViolation);
        let trace = error.trace.expect("trace should be present");
        assert!(trace.ph1x_validation_before_provider);
        assert!(trace.provider_id.is_none());
    }

    #[test]
    fn slice1_extracts_openai_output_text() {
        let value = serde_json::json!({
            "output": [
                {
                    "content": [
                        { "type": "output_text", "text": "first" },
                        { "type": "output_text", "text": "second" }
                    ]
                }
            ]
        });
        assert_eq!(
            extract_openai_output_text(&value).as_deref(),
            Some("first\nsecond")
        );
    }

    #[test]
    fn slice1_openai_key_env_wins_over_vault() {
        let _guard = env_lock().lock().expect("env lock should be available");
        let (base, vault_path) = temp_vault_path("env-wins");
        let vault_path_text = vault_path.to_string_lossy().to_string();
        let _vault_scope = ScopedEnvVar::set("SELENE_DEVICE_VAULT_PATH", &vault_path_text);
        let _env_scope = ScopedEnvVar::set("OPENAI_API_KEY", "env-test-key");
        DeviceVault::default_local()
            .set_secret("openai_api_key", "vault-test-key")
            .expect("vault secret should store");

        let provider = OpenAiSlice1Provider::from_env().expect("env key should configure provider");

        assert_eq!(provider.api_key, "env-test-key");
        assert_eq!(provider.api_key_source, Slice1ProviderKeySource::Env);
        fs::remove_dir_all(base).expect("temp vault directory should be removed");
    }

    #[test]
    fn slice1_openai_key_uses_vault_when_env_missing() {
        let _guard = env_lock().lock().expect("env lock should be available");
        let (base, vault_path) = temp_vault_path("vault-fallback");
        let vault_path_text = vault_path.to_string_lossy().to_string();
        let _vault_scope = ScopedEnvVar::set("SELENE_DEVICE_VAULT_PATH", &vault_path_text);
        let _env_scope = ScopedEnvVar::unset("OPENAI_API_KEY");
        DeviceVault::default_local()
            .set_secret("openai_api_key", "vault-test-key")
            .expect("vault secret should store");

        let provider =
            OpenAiSlice1Provider::from_env().expect("vault key should configure provider");

        assert_eq!(provider.api_key, "vault-test-key");
        assert_eq!(
            provider.api_key_source,
            Slice1ProviderKeySource::DeviceVault
        );
        fs::remove_dir_all(base).expect("temp vault directory should be removed");
    }

    #[test]
    fn slice1_openai_key_missing_fails_closed_safely() {
        let _guard = env_lock().lock().expect("env lock should be available");
        let (base, vault_path) = temp_vault_path("missing");
        let vault_path_text = vault_path.to_string_lossy().to_string();
        let _vault_scope = ScopedEnvVar::set("SELENE_DEVICE_VAULT_PATH", &vault_path_text);
        let _env_scope = ScopedEnvVar::unset("OPENAI_API_KEY");

        let error = OpenAiSlice1Provider::from_env().expect_err("missing key should fail closed");

        assert_eq!(error, "missing_openai_api_key");
        assert!(!error.contains("sk-"));
        assert!(!error.contains("vault-test-key"));
        fs::remove_dir_all(base).expect("temp vault directory should be removed");
    }

    #[test]
    fn slice1_missing_provider_key_response_marks_safe_key_source_missing() {
        let _guard = env_lock().lock().expect("env lock should be available");
        let (base, vault_path) = temp_vault_path("missing-trace");
        let vault_path_text = vault_path.to_string_lossy().to_string();
        let _vault_scope = ScopedEnvVar::set("SELENE_DEVICE_VAULT_PATH", &vault_path_text);
        let _env_scope = ScopedEnvVar::unset("OPENAI_API_KEY");

        let error =
            run_slice1_text_conversation_from_env(request("say hello"), "req_slice1".to_string())
                .expect_err("missing key should fail closed with trace");

        assert_eq!(error.class, Slice1ErrorClass::ProviderFailure);
        assert_eq!(error.reason_code, "SLICE1_PROVIDER_CONFIG_FAIL");
        assert_eq!(error.safe_reason, "missing_openai_api_key");
        let trace = error.trace.expect("trace should be present");
        assert!(trace
            .safe_provider_metadata
            .iter()
            .any(|field| { field.key == "provider_key_source" && field.value == "missing" }));
        for field in trace.safe_provider_metadata {
            assert!(!field.value.contains("sk-"));
            assert!(!field.value.contains("vault-test-key"));
            assert!(!field.value.contains("env-test-key"));
        }
        fs::remove_dir_all(base).expect("temp vault directory should be removed");
    }
}
