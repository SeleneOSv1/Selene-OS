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
pub struct SourceMetadata {
    pub schema_version: SchemaVersion,
    /// Optional, coarse hint only. Must not be required for upstream logic.
    pub provider_hint: Option<String>,
    pub retrieved_at_unix_ms: u64,
    pub sources: Vec<SourceRef>,
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
        if self.sources.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "source_metadata.sources",
                reason: "must not be empty",
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
        Ok(())
    }
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
    Time { local_time_iso: String },
    Weather { summary: String },
    WebSearch { items: Vec<ToolTextSnippet> },
    News { items: Vec<ToolTextSnippet> },
    UrlFetchAndCite { citations: Vec<ToolTextSnippet> },
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
        let r = Self {
            schema_version: PH1E_CONTRACT_VERSION,
            request_id,
            query_hash,
            tool_status: ToolStatus::Fail,
            tool_result: None,
            source_metadata: None,
            reason_code: fail_reason_code,
            fail_reason_code: Some(fail_reason_code),
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
                    url: "https://example.com".to_string(),
                }],
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
