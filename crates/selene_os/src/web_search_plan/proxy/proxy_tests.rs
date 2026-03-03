#![forbid(unsafe_code)]

use crate::web_search_plan::proxy::proxy_config::{
    select_proxy_mode_with_lock, MapEnvProvider, ProxyConfig,
};
use crate::web_search_plan::proxy::proxy_redaction::redact_proxy_url;
use crate::web_search_plan::proxy::proxy_retry::{
    backoff_for_attempt, fixed_backoff_schedule_ms, DiagnosticRateLimiter, FailureCooldownTracker,
    FailureSignature, FakeClock, MAX_ATTEMPTS,
};
use crate::web_search_plan::proxy::proxy_self_check::run_startup_self_check;
use crate::web_search_plan::proxy::{ProxyErrorKind, ProxyMode, ProxySelfCheckSeverity};
use std::sync::OnceLock;

#[test]
fn test_mode_parsing_and_env_selection() {
    let env = MapEnvProvider::new(&[
        ("HTTP_PROXY", "http://env-http.local:8080"),
        ("HTTPS_PROXY", "https://env-https.local:8443"),
        ("ALL_PROXY", "http://ignored.local:9999"),
        ("SELENE_HTTP_PROXY_URL", "http://explicit-http.local:8080"),
        ("SELENE_HTTPS_PROXY_URL", "https://explicit-https.local:8443"),
    ]);

    let off = ProxyConfig::from_env(ProxyMode::Off, &env);
    assert_eq!(off.http_proxy_url, None);
    assert_eq!(off.https_proxy_url, None);

    let env_mode = ProxyConfig::from_env(ProxyMode::Env, &env);
    assert_eq!(env_mode.http_proxy_url.as_deref(), Some("http://env-http.local:8080"));
    assert_eq!(
        env_mode.https_proxy_url.as_deref(),
        Some("https://env-https.local:8443")
    );
    assert_ne!(
        env_mode.http_proxy_url.as_deref(),
        Some("http://ignored.local:9999")
    );

    let explicit = ProxyConfig::from_env(ProxyMode::Explicit, &env);
    assert_eq!(
        explicit.http_proxy_url.as_deref(),
        Some("http://explicit-http.local:8080")
    );
    assert_eq!(
        explicit.https_proxy_url.as_deref(),
        Some("https://explicit-https.local:8443")
    );

    assert_eq!(ProxyMode::parse("off").unwrap(), ProxyMode::Off);
    assert_eq!(ProxyMode::parse("env").unwrap(), ProxyMode::Env);
    assert_eq!(ProxyMode::parse("explicit").unwrap(), ProxyMode::Explicit);
    assert!(ProxyMode::parse("dynamic").is_err());

    let lock = OnceLock::new();
    assert_eq!(select_proxy_mode_with_lock(&lock, ProxyMode::Env).unwrap(), ProxyMode::Env);
    let err = select_proxy_mode_with_lock(&lock, ProxyMode::Explicit)
        .expect_err("switching mode should be blocked");
    assert!(err.contains("already locked"));
}

#[test]
fn test_explicit_mode_requires_both_vars_and_is_critical() {
    let env = MapEnvProvider::new(&[("SELENE_HTTP_PROXY_URL", "http://proxy.local:8080")]);
    let config = ProxyConfig::from_env(ProxyMode::Explicit, &env);

    let failure = run_startup_self_check(&config).expect_err("expected explicit mode failure");
    assert_eq!(failure.severity, ProxySelfCheckSeverity::Critical);
    assert_eq!(failure.error_kind, ProxyErrorKind::ProxyMisconfigured);
    assert_eq!(failure.reason_code, "proxy_misconfigured");
}

#[test]
fn test_env_mode_missing_vars_is_warn() {
    let env = MapEnvProvider::new(&[]);
    let config = ProxyConfig::from_env(ProxyMode::Env, &env);

    let failure = run_startup_self_check(&config).expect_err("expected env mode warn");
    assert_eq!(failure.severity, ProxySelfCheckSeverity::Warn);
    assert_eq!(failure.error_kind, ProxyErrorKind::ProxyMisconfigured);
    assert_eq!(failure.reason_code, "proxy_misconfigured");
}

#[test]
fn test_redaction_removes_userinfo() {
    let raw = "https://user:super-secret@proxy.example.com:8443";
    let redacted = redact_proxy_url(raw).expect("redaction should pass");
    assert_eq!(redacted, "https://proxy.example.com:8443");
    assert!(!redacted.contains("user"));
    assert!(!redacted.contains("super-secret"));
    assert!(!redacted.contains('@'));
}

#[test]
fn test_diagnostic_rate_limiter_is_deterministic() {
    let clock = FakeClock::new(10_000);
    let mut limiter = DiagnosticRateLimiter::new(clock.clone());

    assert!(limiter.should_emit(ProxyMode::Explicit, ProxyErrorKind::ProxyMisconfigured));
    assert!(!limiter.should_emit(ProxyMode::Explicit, ProxyErrorKind::ProxyMisconfigured));

    clock.advance_ms(4_999);
    assert!(!limiter.should_emit(ProxyMode::Explicit, ProxyErrorKind::ProxyMisconfigured));

    clock.advance_ms(1);
    assert!(limiter.should_emit(ProxyMode::Explicit, ProxyErrorKind::ProxyMisconfigured));
}

#[test]
fn test_retry_backoff_schedule_is_fixed() {
    let first = fixed_backoff_schedule_ms().to_vec();
    let second = fixed_backoff_schedule_ms().to_vec();
    assert_eq!(first, second);
    assert_eq!(first, vec![0, 250, 1000]);
    assert_eq!(MAX_ATTEMPTS, 3);
    assert_eq!(backoff_for_attempt(0), Some(0));
    assert_eq!(backoff_for_attempt(1), Some(250));
    assert_eq!(backoff_for_attempt(2), Some(1000));
    assert_eq!(backoff_for_attempt(3), None);
}

#[test]
fn test_failure_cooldown_after_repeated_identical_failures() {
    let clock = FakeClock::new(0);
    let mut tracker = FailureCooldownTracker::new(clock.clone());
    let signature = FailureSignature {
        mode: ProxyMode::Env,
        error_kind: ProxyErrorKind::ProxyTimeout,
        redacted_proxy_target: "http://proxy.local:8080".to_string(),
    };

    let first = tracker.record_failure(signature.clone());
    assert!(!first.cooldown_engaged);

    let second = tracker.record_failure(signature.clone());
    assert!(!second.cooldown_engaged);

    let third = tracker.record_failure(signature.clone());
    assert!(third.cooldown_engaged);
    assert!(!third.blocked_by_cooldown);

    let blocked = tracker.record_failure(signature.clone());
    assert!(blocked.blocked_by_cooldown);
    assert!(!tracker.can_attempt(&signature));

    clock.advance_ms(30_000);
    assert!(tracker.can_attempt(&signature));
}
