#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use selene_kernel_contracts::ph1write::{
    CriticalToken, Ph1WriteOk, Ph1WriteRefuse, Ph1WriteRequest, Ph1WriteResponse, WriteFormatMode,
    WriteRenderStyle,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.WRITE reason-code namespace. Values are placeholders until registry lock.
    pub const WRITE_OK_FORMATTED_TEXT: ReasonCodeId = ReasonCodeId(0x5752_0001);
    pub const WRITE_OK_FALLBACK_ORIGINAL: ReasonCodeId = ReasonCodeId(0x5752_0002);
    pub const WRITE_FAIL_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x5752_00F1);
    pub const WRITE_FALLBACK_CRITICAL_TOKEN_DRIFT: ReasonCodeId = ReasonCodeId(0x5752_00F2);
    pub const WRITE_FALLBACK_REFUSAL_POLICY_LOCK: ReasonCodeId = ReasonCodeId(0x5752_00F3);
    pub const WRITE_FALLBACK_EMPTY_FORMAT: ReasonCodeId = ReasonCodeId(0x5752_00F4);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1WriteConfig {
    pub max_output_chars: usize,
}

impl Ph1WriteConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_output_chars: 32_768,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1WriteRuntime {
    config: Ph1WriteConfig,
}

impl Ph1WriteRuntime {
    pub fn new(config: Ph1WriteConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1WriteRequest) -> Ph1WriteResponse {
        if req.validate().is_err() {
            return Ph1WriteResponse::Refuse(
                Ph1WriteRefuse::v1(
                    reason_codes::WRITE_FAIL_INPUT_SCHEMA_INVALID,
                    "PH1.WRITE request failed contract validation".to_string(),
                )
                .expect("static refuse payload must be valid"),
            );
        }

        if req.is_refusal_or_policy_text || looks_like_refusal_or_policy_text(&req.response_text) {
            return fallback_ok(
                req,
                reason_codes::WRITE_FALLBACK_REFUSAL_POLICY_LOCK,
                &req.response_text,
            );
        }

        let formatted = match req.render_style {
            WriteRenderStyle::Preserve => req.response_text.clone(),
            WriteRenderStyle::Professional => normalize_professional_text(&req.response_text),
        };

        if formatted.trim().is_empty() || formatted.chars().count() > self.config.max_output_chars {
            return fallback_ok(
                req,
                reason_codes::WRITE_FALLBACK_EMPTY_FORMAT,
                &req.response_text,
            );
        }

        let critical_tokens = combined_critical_tokens(req);
        if !critical_tokens_preserved(&critical_tokens, &formatted) {
            return fallback_ok(
                req,
                reason_codes::WRITE_FALLBACK_CRITICAL_TOKEN_DRIFT,
                &req.response_text,
            );
        }

        let ok = Ph1WriteOk::v1(
            req.correlation_id,
            req.turn_id,
            formatted,
            WriteFormatMode::FormattedText,
            reason_codes::WRITE_OK_FORMATTED_TEXT,
            true,
        )
        .expect("formatted output must be valid");
        Ph1WriteResponse::Ok(ok)
    }
}

fn fallback_ok(
    req: &Ph1WriteRequest,
    reason_code: ReasonCodeId,
    original_text: &str,
) -> Ph1WriteResponse {
    let ok = Ph1WriteOk::v1(
        req.correlation_id,
        req.turn_id,
        original_text.to_string(),
        WriteFormatMode::FallbackOriginal,
        reason_code,
        true,
    )
    .expect("fallback output must be valid");
    Ph1WriteResponse::Ok(ok)
}

fn normalize_professional_text(input: &str) -> String {
    let mut compact = String::with_capacity(input.len());
    let mut prev_space = false;
    for ch in input.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                compact.push(' ');
            }
            prev_space = true;
        } else {
            compact.push(ch);
            prev_space = false;
        }
    }

    let mut no_space_before_punct = String::with_capacity(compact.len());
    let mut chars = compact.trim().chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == ' ' && chars.peek().is_some_and(|c| is_sentence_punctuation(*c)) {
            continue;
        }
        no_space_before_punct.push(ch);
    }

    no_space_before_punct
}

fn combined_critical_tokens(req: &Ph1WriteRequest) -> Vec<CriticalToken> {
    let mut set: BTreeSet<String> = req
        .critical_tokens
        .iter()
        .map(|t| t.as_str().to_string())
        .collect();
    for token in extract_critical_tokens_from_source(&req.response_text) {
        set.insert(token);
    }
    set.into_iter()
        .filter_map(|t| CriticalToken::new(t).ok())
        .collect()
}

fn extract_critical_tokens_from_source(text: &str) -> Vec<String> {
    let months = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ];

    let mut out = BTreeSet::new();
    for (idx, raw) in text.split_whitespace().enumerate() {
        let token = trim_edge_punctuation(raw);
        if token.is_empty() || token.len() > 64 {
            continue;
        }
        if token.chars().any(|c| c.is_control() || c.is_whitespace()) {
            continue;
        }
        let lower = token.to_ascii_lowercase();
        let has_digit = token.chars().any(|c| c.is_ascii_digit());
        let has_currency = token.contains('$');
        let looks_time = token.contains(':') && has_digit;
        let month_name = months.contains(&lower.as_str());
        let acronym_like = token.len() >= 2 && token.chars().all(|c| c.is_ascii_uppercase());
        let name_like = idx > 0
            && token
                .chars()
                .next()
                .map(|c| c.is_ascii_uppercase())
                .unwrap_or(false)
            && token.chars().skip(1).any(|c| c.is_ascii_lowercase());

        if has_digit || has_currency || looks_time || month_name || acronym_like || name_like {
            out.insert(token.to_string());
        }
    }
    out.into_iter().collect()
}

fn trim_edge_punctuation(raw: &str) -> &str {
    raw.trim_matches(|c: char| {
        matches!(
            c,
            ',' | '.' | '!' | '?' | ';' | ':' | '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\''
        )
    })
}

fn critical_tokens_preserved(tokens: &[CriticalToken], formatted_text: &str) -> bool {
    tokens.iter().all(|t| formatted_text.contains(t.as_str()))
}

fn looks_like_refusal_or_policy_text(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let guardrails = [
        "i can't",
        "i cannot",
        "i am not able",
        "i'm not able",
        "i won’t",
        "i won't",
        "without your confirmation",
        "permission required",
        "not authorized",
        "policy",
        "safety",
        "cannot proceed",
    ];
    guardrails.iter().any(|p| lower.contains(p))
}

fn is_sentence_punctuation(c: char) -> bool {
    matches!(c, '.' | ',' | '!' | '?' | ';' | ':')
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::MonotonicTimeNs;

    fn runtime() -> Ph1WriteRuntime {
        Ph1WriteRuntime::new(Ph1WriteConfig::mvp_v1())
    }

    fn req(response_text: &str, is_refusal_or_policy_text: bool) -> Ph1WriteRequest {
        let mut tokens = vec![];
        for token in ["John", "$1200", "2026-03-01", "3:00pm"] {
            if response_text.contains(token) {
                tokens.push(CriticalToken::new(token).unwrap());
            }
        }

        Ph1WriteRequest::v1(
            MonotonicTimeNs(50),
            TenantId::new("tenant_a").unwrap(),
            CorrelationId(7001),
            TurnId(3),
            None,
            UserId::new("tenant_a:user_1").unwrap(),
            DeviceId::new("tenant_a_device_1").unwrap(),
            response_text.to_string(),
            WriteRenderStyle::Professional,
            tokens,
            is_refusal_or_policy_text,
            "write-runtime-1".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn at_write_01_preserves_critical_tokens_while_formatting() {
        let input = req("  John   owes $1200 on   2026-03-01 at 3:00pm.  ", false);
        let out = runtime().run(&input);
        match out {
            Ph1WriteResponse::Ok(ok) => {
                assert_eq!(ok.format_mode, WriteFormatMode::FormattedText);
                assert_eq!(
                    ok.formatted_text,
                    "John owes $1200 on 2026-03-01 at 3:00pm."
                );
                assert!(ok.formatted_text.contains("John"));
                assert!(ok.formatted_text.contains("$1200"));
                assert!(ok.formatted_text.contains("2026-03-01"));
                assert!(ok.formatted_text.contains("3:00pm"));
                assert!(ok.validate().is_ok());
            }
            _ => panic!("expected ok"),
        }
    }

    #[test]
    fn at_write_02_refusal_and_policy_text_is_never_weakened() {
        let text = "I cannot proceed without your confirmation.";
        let input = req(text, true);
        let out = runtime().run(&input);
        match out {
            Ph1WriteResponse::Ok(ok) => {
                assert_eq!(ok.format_mode, WriteFormatMode::FallbackOriginal);
                assert_eq!(ok.formatted_text, text);
                assert_eq!(
                    ok.reason_code,
                    reason_codes::WRITE_FALLBACK_REFUSAL_POLICY_LOCK
                );
            }
            _ => panic!("expected ok fallback"),
        }
    }

    #[test]
    fn fallback_triggers_on_implicit_policy_guardrail() {
        let input = req("I cannot proceed without your confirmation.", false);

        let out = runtime().run(&input);
        match out {
            Ph1WriteResponse::Ok(ok) => {
                assert_eq!(ok.format_mode, WriteFormatMode::FallbackOriginal);
                assert_eq!(
                    ok.reason_code,
                    reason_codes::WRITE_FALLBACK_REFUSAL_POLICY_LOCK
                );
                assert_eq!(
                    ok.formatted_text,
                    "I cannot proceed without your confirmation."
                );
            }
            _ => panic!("expected fallback"),
        }
    }
}
