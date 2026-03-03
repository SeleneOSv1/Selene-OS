#![forbid(unsafe_code)]

use crate::web_search_plan::proxy::proxy_config::ProxyConfig;
use crate::web_search_plan::proxy::proxy_redaction::redact_proxy_url;
use crate::web_search_plan::proxy::{ProxyErrorKind, ProxyMode, ProxySelfCheckSeverity};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxySelfCheckFailure {
    pub severity: ProxySelfCheckSeverity,
    pub error_kind: ProxyErrorKind,
    pub reason_code: &'static str,
    pub redacted_details: Option<String>,
    pub details: String,
}

pub fn run_startup_self_check(config: &ProxyConfig) -> Result<(), ProxySelfCheckFailure> {
    match config.mode {
        ProxyMode::Off => Ok(()),
        ProxyMode::Explicit => check_explicit_mode(config),
        ProxyMode::Env => check_env_mode(config),
    }
}

fn check_explicit_mode(config: &ProxyConfig) -> Result<(), ProxySelfCheckFailure> {
    let missing = config.missing_required_fields();
    if !missing.is_empty() {
        return Err(failure(
            ProxySelfCheckSeverity::Critical,
            ProxyErrorKind::ProxyMisconfigured,
            None,
            format!("missing explicit proxy vars: {}", missing.join(",")),
        ));
    }

    validate_proxy_urls(config, ProxySelfCheckSeverity::Critical)
}

fn check_env_mode(config: &ProxyConfig) -> Result<(), ProxySelfCheckFailure> {
    let missing = config.missing_required_fields();
    if !missing.is_empty() {
        return Err(failure(
            ProxySelfCheckSeverity::Warn,
            ProxyErrorKind::ProxyMisconfigured,
            None,
            format!("missing env proxy vars: {}", missing.join(",")),
        ));
    }

    validate_proxy_urls(config, ProxySelfCheckSeverity::Warn)
}

fn validate_proxy_urls(
    config: &ProxyConfig,
    severity: ProxySelfCheckSeverity,
) -> Result<(), ProxySelfCheckFailure> {
    for (label, url_opt) in [
        ("http_proxy_url", config.http_proxy_url.as_deref()),
        ("https_proxy_url", config.https_proxy_url.as_deref()),
    ] {
        if let Some(url) = url_opt {
            match redact_proxy_url(url) {
                Ok(redacted) => {
                    if redacted.contains('@') {
                        return Err(failure(
                            severity,
                            ProxyErrorKind::ProxyMisconfigured,
                            Some(redacted),
                            format!("{} redaction failed to strip userinfo", label),
                        ));
                    }
                }
                Err(kind) => {
                    return Err(failure(
                        severity,
                        kind,
                        None,
                        format!("{} failed validation", label),
                    ))
                }
            }
        }
    }
    Ok(())
}

fn failure(
    severity: ProxySelfCheckSeverity,
    error_kind: ProxyErrorKind,
    redacted_details: Option<String>,
    details: String,
) -> ProxySelfCheckFailure {
    ProxySelfCheckFailure {
        severity,
        error_kind,
        reason_code: error_kind.reason_code(),
        redacted_details,
        details,
    }
}
