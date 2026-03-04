#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EnterpriseMode {
    Structured,
    Document,
    Competitive,
    Realtime,
    Regulatory,
    Trust,
    Multihop,
    Temporal,
    Risk,
    Merge,
    Report,
}

impl EnterpriseMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Structured => "structured",
            Self::Document => "document",
            Self::Competitive => "competitive",
            Self::Realtime => "realtime",
            Self::Regulatory => "regulatory",
            Self::Trust => "trust",
            Self::Multihop => "multihop",
            Self::Temporal => "temporal",
            Self::Risk => "risk",
            Self::Merge => "merge",
            Self::Report => "report",
        }
    }
}

pub fn parse_mode(raw: &str) -> Result<EnterpriseMode, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "structured" => Ok(EnterpriseMode::Structured),
        "document" => Ok(EnterpriseMode::Document),
        "competitive" => Ok(EnterpriseMode::Competitive),
        "realtime" | "real_time" => Ok(EnterpriseMode::Realtime),
        "regulatory" => Ok(EnterpriseMode::Regulatory),
        "trust" => Ok(EnterpriseMode::Trust),
        "multihop" => Ok(EnterpriseMode::Multihop),
        "temporal" => Ok(EnterpriseMode::Temporal),
        "risk" => Ok(EnterpriseMode::Risk),
        "merge" => Ok(EnterpriseMode::Merge),
        "report" => Ok(EnterpriseMode::Report),
        other => Err(format!("unsupported enterprise mode {}", other)),
    }
}

pub fn route_mode(query: &str, explicit_mode: Option<&str>) -> Result<EnterpriseMode, String> {
    if let Some(mode) = explicit_mode {
        return parse_mode(mode);
    }

    let normalized = query.to_ascii_lowercase();
    if contains_any(
        normalized.as_str(),
        &["timeline", "as of", "delta", "changed since", "trend"],
    ) {
        return Ok(EnterpriseMode::Temporal);
    }
    if contains_any(
        normalized.as_str(),
        &["risk", "exposure", "stress", "downside"],
    ) {
        return Ok(EnterpriseMode::Risk);
    }
    if contains_any(
        normalized.as_str(),
        &["compare", "competitor", "market position", "feature matrix"],
    ) {
        return Ok(EnterpriseMode::Competitive);
    }
    if contains_any(
        normalized.as_str(),
        &["latest", "today", "live", "current price", "weather", "flight"],
    ) {
        return Ok(EnterpriseMode::Realtime);
    }
    if contains_any(
        normalized.as_str(),
        &["regulatory", "compliance", "law", "statute", "jurisdiction"],
    ) {
        return Ok(EnterpriseMode::Regulatory);
    }
    if contains_any(normalized.as_str(), &["merge", "what changed since last report"]) {
        return Ok(EnterpriseMode::Merge);
    }
    Ok(EnterpriseMode::Report)
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}
