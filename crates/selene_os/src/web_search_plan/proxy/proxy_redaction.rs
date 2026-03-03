#![forbid(unsafe_code)]

use crate::web_search_plan::proxy::ProxyErrorKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactedProxyEndpoint {
    pub scheme: String,
    pub host: String,
    pub port: u16,
}

impl RedactedProxyEndpoint {
    pub fn as_redacted_url(&self) -> String {
        format!("{}://{}:{}", self.scheme, self.host, self.port)
    }
}

pub fn redact_proxy_url(url: &str) -> Result<String, ProxyErrorKind> {
    parse_proxy_endpoint(url).map(|endpoint| endpoint.as_redacted_url())
}

pub fn parse_proxy_endpoint(url: &str) -> Result<RedactedProxyEndpoint, ProxyErrorKind> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(ProxyErrorKind::ProxyMisconfigured);
    }

    let (scheme_raw, rest) = trimmed
        .split_once("://")
        .ok_or(ProxyErrorKind::ProxyMisconfigured)?;
    let scheme = scheme_raw.to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return Err(ProxyErrorKind::ProxyMisconfigured);
    }

    let authority = rest
        .split_once('/')
        .map(|(head, _)| head)
        .unwrap_or(rest)
        .split_once('?')
        .map(|(head, _)| head)
        .unwrap_or(rest)
        .split_once('#')
        .map(|(head, _)| head)
        .unwrap_or(rest);
    if authority.is_empty() {
        return Err(ProxyErrorKind::ProxyMisconfigured);
    }

    // Strip credentials while preserving host:port only.
    let host_port = authority
        .rsplit_once('@')
        .map(|(_, host)| host)
        .unwrap_or(authority);
    let (host, port) = parse_host_port(host_port, &scheme)?;

    Ok(RedactedProxyEndpoint { scheme, host, port })
}

fn parse_host_port(host_port: &str, scheme: &str) -> Result<(String, u16), ProxyErrorKind> {
    if host_port.is_empty() {
        return Err(ProxyErrorKind::ProxyMisconfigured);
    }

    if host_port.starts_with('[') {
        parse_bracketed_ipv6(host_port, scheme)
    } else {
        parse_hostname(host_port, scheme)
    }
}

fn parse_bracketed_ipv6(host_port: &str, scheme: &str) -> Result<(String, u16), ProxyErrorKind> {
    let close = host_port
        .find(']')
        .ok_or(ProxyErrorKind::ProxyMisconfigured)?;
    let host = host_port[..=close].to_string();
    let remainder = &host_port[close + 1..];
    let port = if remainder.is_empty() {
        default_port_for_scheme(scheme)
    } else {
        let raw = remainder
            .strip_prefix(':')
            .ok_or(ProxyErrorKind::ProxyMisconfigured)?;
        parse_port(raw)?
    };
    Ok((host, port))
}

fn parse_hostname(host_port: &str, scheme: &str) -> Result<(String, u16), ProxyErrorKind> {
    if host_port.contains(' ') || host_port.contains('\t') {
        return Err(ProxyErrorKind::ProxyMisconfigured);
    }

    if let Some((host_raw, port_raw)) = host_port.rsplit_once(':') {
        if host_raw.is_empty() {
            return Err(ProxyErrorKind::ProxyMisconfigured);
        }
        if host_raw.contains(':') {
            return Err(ProxyErrorKind::ProxyMisconfigured);
        }
        let port = parse_port(port_raw)?;
        return Ok((host_raw.to_string(), port));
    }

    Ok((host_port.to_string(), default_port_for_scheme(scheme)))
}

fn parse_port(raw: &str) -> Result<u16, ProxyErrorKind> {
    let port = raw
        .parse::<u16>()
        .map_err(|_| ProxyErrorKind::ProxyMisconfigured)?;
    if port == 0 {
        return Err(ProxyErrorKind::ProxyMisconfigured);
    }
    Ok(port)
}

fn default_port_for_scheme(scheme: &str) -> u16 {
    if scheme == "https" { 443 } else { 80 }
}
