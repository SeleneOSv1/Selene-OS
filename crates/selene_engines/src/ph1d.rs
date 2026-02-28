#![forbid(unsafe_code)]

use serde_json::Value;

use selene_kernel_contracts::ph1d::{
    Ph1dAnalysis, Ph1dChat, Ph1dClarify, Ph1dFail, Ph1dFailureKind, Ph1dFieldRefinement,
    Ph1dIntent, Ph1dOk, Ph1dProviderCallRequest, Ph1dProviderCallResponse, Ph1dProviderTask,
    Ph1dRequest, Ph1dResponse,
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

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.D reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const D_FAIL_INVALID_SCHEMA: ReasonCodeId = ReasonCodeId(0x4400_0001);
    pub const D_FAIL_FORBIDDEN_OUTPUT: ReasonCodeId = ReasonCodeId(0x4400_0002);
    pub const D_FAIL_SAFETY_BLOCK: ReasonCodeId = ReasonCodeId(0x4400_0003);
    pub const D_FAIL_TIMEOUT: ReasonCodeId = ReasonCodeId(0x4400_0004);
    pub const D_FAIL_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4400_0005);

    pub const D_CLARIFY_EVIDENCE_REQUIRED: ReasonCodeId = ReasonCodeId(0x4400_0100);

    // Provider-facing bridge outcomes used by PH1.C live provider adapter path.
    pub const D_PROVIDER_OK: ReasonCodeId = ReasonCodeId(0x4400_1000);
    pub const D_PROVIDER_TIMEOUT: ReasonCodeId = ReasonCodeId(0x4400_1001);
    pub const D_PROVIDER_SCHEMA_DRIFT: ReasonCodeId = ReasonCodeId(0x4400_1002);
    pub const D_PROVIDER_CONTRACT_MISMATCH: ReasonCodeId = ReasonCodeId(0x4400_1003);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedProviderNormalizedOutput {
    pub schema_version: u64,
    pub provider_task: Ph1dProviderTask,
    pub text_output: Option<String>,
    pub language_tag: Option<String>,
    pub confidence_bp: Option<u16>,
    pub stable: Option<bool>,
}

pub fn decode_normalized_output_json(
    json_text: &str,
) -> Result<DecodedProviderNormalizedOutput, ContractViolation> {
    let value: Value =
        serde_json::from_str(json_text).map_err(|_| ContractViolation::InvalidValue {
            field: "ph1d_provider_normalized_output_json",
            reason: "must be valid JSON object",
        })?;
    let obj = value.as_object().ok_or(ContractViolation::InvalidValue {
        field: "ph1d_provider_normalized_output_json",
        reason: "must be a JSON object",
    })?;

    let schema_version = obj.get("schema_version").and_then(|v| v.as_u64()).ok_or(
        ContractViolation::InvalidValue {
            field: "ph1d_provider_normalized_output_json.schema_version",
            reason: "must be a positive integer",
        },
    )?;
    if schema_version == 0 {
        return Err(ContractViolation::InvalidValue {
            field: "ph1d_provider_normalized_output_json.schema_version",
            reason: "must be > 0",
        });
    }

    let provider_task = match obj.get("provider_task").and_then(|v| v.as_str()).ok_or(
        ContractViolation::InvalidValue {
            field: "ph1d_provider_normalized_output_json.provider_task",
            reason: "must be present",
        },
    )? {
        "OCR_TEXT_EXTRACT" => Ph1dProviderTask::OcrTextExtract,
        "STT_TRANSCRIBE" => Ph1dProviderTask::SttTranscribe,
        _ => {
            return Err(ContractViolation::InvalidValue {
                field: "ph1d_provider_normalized_output_json.provider_task",
                reason: "unsupported provider_task",
            })
        }
    };

    let text_output = obj
        .get("text_output")
        .and_then(|v| v.as_str())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());

    let language_tag = obj
        .get("language_tag")
        .and_then(|v| v.as_str())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());

    let confidence_bp = match obj.get("confidence_bp").and_then(|v| v.as_u64()) {
        Some(raw) => {
            if raw > 10_000 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1d_provider_normalized_output_json.confidence_bp",
                    reason: "must be <= 10000",
                });
            }
            Some(raw as u16)
        }
        None => None,
    };

    let stable = obj.get("stable").and_then(|v| v.as_bool());

    Ok(DecodedProviderNormalizedOutput {
        schema_version,
        provider_task,
        text_output,
        language_tag,
        confidence_bp,
        stable,
    })
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
        "ReminderId" | "reminder_id" | "reminderId" => Some(FieldKey::ReminderId),
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
        "UpdateBcastWaitPolicy" | "update_bcast_wait_policy" | "updateBcastWaitPolicy" => {
            Some(IntentType::UpdateBcastWaitPolicy)
        }
        "UpdateBcastUrgentFollowupPolicy"
        | "update_bcast_urgent_followup_policy"
        | "updateBcastUrgentFollowupPolicy" => Some(IntentType::UpdateBcastUrgentFollowupPolicy),
        "UpdateReminder" | "update_reminder" => Some(IntentType::UpdateReminder),
        "CancelReminder" | "cancel_reminder" => Some(IntentType::CancelReminder),
        "ListReminders" | "list_reminders" => Some(IntentType::ListReminders),
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
        FieldKey::ReminderId => (
            "Which reminder ID should I use?".to_string(),
            vec![
                "rem_0000000000000001".to_string(),
                "rem_0000000000000002".to_string(),
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
        FieldKey::ReminderId,
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
    if n > u32::MAX as u64 {
        return None;
    }
    Some(ReasonCodeId(n as u32))
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
