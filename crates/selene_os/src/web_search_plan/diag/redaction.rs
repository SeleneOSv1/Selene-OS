#![forbid(unsafe_code)]

use crate::web_search_plan::proxy::proxy_redaction::redact_proxy_url;

pub fn redact_url(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("url must not be empty".to_string());
    }

    let parsed = url::Url::parse(trimmed).map_err(|_| "invalid url".to_string())?;
    let scheme = parsed.scheme().to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return Err("unsupported scheme".to_string());
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| "url host missing".to_string())?
        .to_ascii_lowercase();

    let mut out = format!("{}://{}", scheme, host);

    if let Some(port) = parsed.port() {
        let default_port = if scheme == "https" { 443 } else { 80 };
        if port != default_port {
            out.push(':');
            out.push_str(&port.to_string());
        }
    }

    let path = parsed.path();
    if !path.is_empty() && path != "/" {
        out.push_str(path);
    }

    if parsed.query().is_some() {
        out.push_str("?redacted");
    }

    Ok(out)
}

pub fn redact_proxy(input: &str) -> Result<String, String> {
    redact_proxy_url(input).map_err(|_| "invalid proxy url".to_string())
}

pub fn redact_token(_input: &str) -> String {
    "[REDACTED]".to_string()
}

pub fn sanitize_debug_hint(input: &str) -> String {
    let mut out_tokens = Vec::new();

    for token in input.split_whitespace() {
        let lower = token.to_ascii_lowercase();

        if lower.contains("api_key")
            || lower.contains("authorization")
            || lower.contains("cookie")
            || lower.contains("token=")
            || lower.contains("sk-")
            || lower.contains("bearer")
        {
            out_tokens.push(redact_token(token));
            continue;
        }

        if token.starts_with('/') || lower.contains(":\\") {
            out_tokens.push("[REDACTED_PATH]".to_string());
            continue;
        }

        if lower.starts_with("http://") || lower.starts_with("https://") {
            match redact_url(token) {
                Ok(redacted) => out_tokens.push(redacted),
                Err(_) => out_tokens.push("[REDACTED_URL]".to_string()),
            }
            continue;
        }

        out_tokens.push(token.to_string());
    }

    out_tokens.join(" ")
}
