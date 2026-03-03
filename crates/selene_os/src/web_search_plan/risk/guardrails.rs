#![forbid(unsafe_code)]

pub const RISK_DISCLAIMER_TEXT: &str = "Informational only — not financial/legal advice.";

const FORBIDDEN_RECOMMENDATION_PATTERNS: &[&str] = &[
    "buy",
    "sell",
    "avoid",
    "invest",
    "short",
    "long position",
    "recommend",
    "should purchase",
    "should invest",
    "take action",
    "final decision",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardrailError {
    pub reason_code: &'static str,
    pub message: String,
}

pub fn enforce_non_advice_guardrails(texts: &[String]) -> Result<(), GuardrailError> {
    for text in texts {
        let normalized = text.to_ascii_lowercase();
        for pattern in FORBIDDEN_RECOMMENDATION_PATTERNS {
            if normalized.contains(pattern) {
                return Err(GuardrailError {
                    reason_code: "policy_violation",
                    message: format!("non-advice guardrail blocked pattern '{}'", pattern),
                });
            }
        }
    }
    Ok(())
}

pub fn disclaimer_text() -> &'static str {
    RISK_DISCLAIMER_TEXT
}
