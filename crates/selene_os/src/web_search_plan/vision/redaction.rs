#![forbid(unsafe_code)]

use crate::web_search_plan::contract_hash::sha256_hex;

pub fn redact_locator(locator: &str) -> String {
    let trimmed = locator.trim();
    if trimmed.is_empty() {
        return "asset://redacted/empty".to_string();
    }

    if is_http_locator(trimmed) {
        return redact_http_locator(trimmed);
    }

    let locator_hash = sha256_hex(trimmed.as_bytes());
    format!("asset://local/{}", &locator_hash[..16])
}

pub fn redact_error_message(message: &str) -> String {
    let mut out = message.replace("api_key", "[redacted_key]");
    out = out.replace("authorization", "[redacted_auth]");

    if let Some(index) = out.to_ascii_lowercase().find("sk-") {
        out.replace_range(index.., "[redacted_secret]");
    }

    out
}

pub fn is_http_locator(locator: &str) -> bool {
    let lower = locator.trim().to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

fn redact_http_locator(locator: &str) -> String {
    match url::Url::parse(locator) {
        Ok(mut parsed) => {
            let _ = parsed.set_username("");
            let _ = parsed.set_password(None);
            parsed.set_query(None);
            parsed.set_fragment(None);

            let mut rebuilt = format!(
                "{}://{}",
                parsed.scheme(),
                parsed
                    .host_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| "unknown-host".to_string())
            );

            if let Some(port) = parsed.port() {
                rebuilt.push(':');
                rebuilt.push_str(&port.to_string());
            }

            let path = parsed.path();
            if path != "/" && !path.is_empty() {
                rebuilt.push_str(path);
            }

            rebuilt
        }
        Err(_) => "asset://redacted/invalid-url".to_string(),
    }
}

pub fn debug_string_without_secrets(parts: &[&str]) -> String {
    let joined = parts.join(" | ");
    redact_error_message(&joined)
}
