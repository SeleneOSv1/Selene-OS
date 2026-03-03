#![forbid(unsafe_code)]

use crate::web_search_plan::regulatory::trust_tier::TrustTier;

pub const COMPLIANCE_CONFIDENCE_VERSION: &str = "1.0.0";
pub const MODERATE_CORROBORATION_THRESHOLD: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComplianceConfidence {
    Low,
    Moderate,
    High,
}

impl ComplianceConfidence {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::High => "HIGH",
            Self::Moderate => "MODERATE",
            Self::Low => "LOW",
        }
    }

    pub const fn rank(self) -> u8 {
        match self {
            Self::Low => 0,
            Self::Moderate => 1,
            Self::High => 2,
        }
    }

    pub fn parse_threshold(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "high" => Some(Self::High),
            "moderate" | "medium" => Some(Self::Moderate),
            "low" => Some(Self::Low),
            _ => None,
        }
    }
}

pub fn assess_confidence(
    trust_tiers: &[TrustTier],
    all_fresh: bool,
    corroboration_count: usize,
) -> ComplianceConfidence {
    if !trust_tiers.is_empty()
        && all_fresh
        && trust_tiers.iter().all(|tier| *tier == TrustTier::Official)
    {
        return ComplianceConfidence::High;
    }

    if trust_tiers.iter().any(|tier| *tier == TrustTier::Official || *tier == TrustTier::High)
        && corroboration_count >= MODERATE_CORROBORATION_THRESHOLD
    {
        return ComplianceConfidence::Moderate;
    }

    ComplianceConfidence::Low
}

pub fn meets_required_threshold(
    actual: ComplianceConfidence,
    required: ComplianceConfidence,
) -> bool {
    actual.rank() >= required.rank()
}
