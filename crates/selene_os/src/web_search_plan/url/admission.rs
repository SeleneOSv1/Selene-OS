#![forbid(unsafe_code)]

use crate::web_search_plan::url::UrlFetchErrorKind;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlAdmissionDecision {
    pub admitted: bool,
    pub normalized_url: String,
    pub denied_reason: Option<&'static str>,
    pub safe_public_url: bool,
    pub dns_private_address_validation_proven: bool,
}

pub fn admit_public_fetch_url(input: &str) -> Result<UrlAdmissionDecision, UrlFetchErrorKind> {
    let parsed = Url::parse(input.trim()).map_err(|_| UrlFetchErrorKind::InvalidUrl)?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return denied(
            input,
            UrlFetchErrorKind::UnsupportedScheme,
            "unsupported_scheme",
        );
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return denied(
            input,
            UrlFetchErrorKind::UnsafeUrlBlocked,
            "embedded_credentials_blocked",
        );
    }

    let host = parsed
        .host_str()
        .map(|value| value.trim().trim_end_matches('.').to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .ok_or(UrlFetchErrorKind::InvalidUrl)?;

    if is_metadata_host(host.as_str()) || is_internal_hostname(host.as_str()) {
        return denied(
            parsed.as_str(),
            UrlFetchErrorKind::PrivateUrlBlocked,
            "private_or_internal_host_blocked",
        );
    }

    let dns_private_address_validation_proven = if let Ok(ip) = IpAddr::from_str(host.as_str()) {
        if is_blocked_ip(ip) {
            return denied(
                parsed.as_str(),
                UrlFetchErrorKind::PrivateUrlBlocked,
                "private_or_reserved_ip_blocked",
            );
        }
        true
    } else {
        false
    };

    Ok(UrlAdmissionDecision {
        admitted: true,
        normalized_url: parsed.to_string(),
        denied_reason: None,
        safe_public_url: true,
        dns_private_address_validation_proven,
    })
}

fn denied(
    input: &str,
    kind: UrlFetchErrorKind,
    reason: &'static str,
) -> Result<UrlAdmissionDecision, UrlFetchErrorKind> {
    let _ = input;
    let _ = reason;
    Err(kind)
}

fn is_metadata_host(host: &str) -> bool {
    host == "169.254.169.254"
        || host == "metadata.google.internal"
        || host == "metadata"
        || host == "instance-data"
        || host.ends_with(".metadata.google.internal")
}

fn is_internal_hostname(host: &str) -> bool {
    host == "localhost"
        || host == "localdomain"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host.ends_with(".internal")
        || (!host.contains('.') && IpAddr::from_str(host).is_err())
}

fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_ipv4(v4),
        IpAddr::V6(v6) => {
            if let Some(mapped) = v6.to_ipv4_mapped() {
                return is_blocked_ipv4(mapped);
            }
            is_blocked_ipv6(v6)
        }
    }
}

fn is_blocked_ipv4(ip: Ipv4Addr) -> bool {
    ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_multicast()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.octets()[0] == 0
        || in_ipv4_range(ip, [100, 64, 0, 0], 10)
        || in_ipv4_range(ip, [198, 18, 0, 0], 15)
        || in_ipv4_range(ip, [240, 0, 0, 0], 4)
}

fn is_blocked_ipv6(ip: Ipv6Addr) -> bool {
    ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        || in_ipv6_prefix(ip, 0xfe80, 10)
        || in_ipv6_prefix(ip, 0xfc00, 7)
}

fn in_ipv4_range(ip: Ipv4Addr, base: [u8; 4], prefix_len: u32) -> bool {
    let ip_u32 = u32::from(ip);
    let base_u32 = u32::from(Ipv4Addr::from(base));
    let mask = if prefix_len == 0 {
        0
    } else {
        u32::MAX << (32 - prefix_len)
    };
    (ip_u32 & mask) == (base_u32 & mask)
}

fn in_ipv6_prefix(ip: Ipv6Addr, high_bits: u16, prefix_len: u32) -> bool {
    let first = ip.segments()[0];
    let shift = 16_u32.saturating_sub(prefix_len.min(16));
    (first >> shift) == (high_bits >> shift)
}
