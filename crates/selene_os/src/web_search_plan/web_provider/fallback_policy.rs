#![forbid(unsafe_code)]

use crate::web_search_plan::web_provider::ProviderErrorKind;

pub fn should_trigger_fallback(kind: ProviderErrorKind) -> bool {
    matches!(
        kind,
        ProviderErrorKind::DnsFailed
            | ProviderErrorKind::TlsFailed
            | ProviderErrorKind::ConnectFailed
            | ProviderErrorKind::TimeoutExceeded
            | ProviderErrorKind::HttpNon200
            | ProviderErrorKind::EmptyResults
            | ProviderErrorKind::ParseFailed
    )
}

pub fn fallback_trigger_label(kind: ProviderErrorKind) -> Option<&'static str> {
    if should_trigger_fallback(kind) {
        Some(kind.as_str())
    } else {
        None
    }
}
