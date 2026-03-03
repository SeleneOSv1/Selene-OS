#![forbid(unsafe_code)]

use url::Url;

pub fn redact_url(input: &str) -> String {
    let parsed = Url::parse(input);
    if let Ok(url) = parsed {
        let scheme = url.scheme().to_string();
        let host = url.host_str().unwrap_or_default().to_string();
        if host.is_empty() {
            return "[REDACTED]".to_string();
        }
        let path = url.path().trim().to_string();
        let trimmed_path = if path.is_empty() || path == "/" {
            ""
        } else {
            path.as_str()
        };
        format!("{}://{}{}", scheme, host, trimmed_path)
    } else {
        "[REDACTED]".to_string()
    }
}

pub fn redact_locator(locator: &str) -> String {
    if locator.starts_with("http://") || locator.starts_with("https://") {
        return redact_url(locator);
    }
    if locator.starts_with("file://") {
        return "file://[REDACTED]".to_string();
    }
    if locator.contains('/') || locator.contains('\\') {
        return "[LOCAL_ASSET]".to_string();
    }
    locator.to_string()
}

pub fn redact_secrets(message: &str) -> String {
    let mut sanitized = message.to_string();
    for marker in ["api_key", "token=", "authorization", "bearer ", "sk-"] {
        if sanitized.to_ascii_lowercase().contains(marker) {
            sanitized = "[REDACTED]".to_string();
            break;
        }
    }
    sanitized
}
