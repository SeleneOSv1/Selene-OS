#![forbid(unsafe_code)]

use crate::web_search_plan::url::UrlFetchErrorKind;
use url::{form_urlencoded, Url};

pub const CANON_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalUrl {
    pub canonical_url: String,
    pub canon_version: &'static str,
}

pub fn canonicalize_url(input: &str) -> Result<CanonicalUrl, UrlFetchErrorKind> {
    let mut parsed = Url::parse(input.trim()).map_err(|_| UrlFetchErrorKind::InvalidUrl)?;
    let lower_scheme = parsed.scheme().to_ascii_lowercase();
    if lower_scheme != "http" && lower_scheme != "https" {
        return Err(UrlFetchErrorKind::UnsupportedScheme);
    }
    parsed
        .set_scheme(&lower_scheme)
        .map_err(|_| UrlFetchErrorKind::InvalidUrl)?;

    if let Some(host) = parsed.host_str() {
        let lower_host = host.to_ascii_lowercase();
        parsed
            .set_host(Some(&lower_host))
            .map_err(|_| UrlFetchErrorKind::InvalidUrl)?;
    }

    if parsed.username() != "" || parsed.password().is_some() {
        return Err(UrlFetchErrorKind::UnsafeUrlBlocked);
    }

    strip_default_port(&mut parsed)?;
    normalize_path(&mut parsed);
    normalize_query(&mut parsed);
    parsed.set_fragment(None);

    Ok(CanonicalUrl {
        canonical_url: parsed.to_string(),
        canon_version: CANON_VERSION,
    })
}

fn strip_default_port(parsed: &mut Url) -> Result<(), UrlFetchErrorKind> {
    let port = parsed.port();
    let should_strip = matches!(
        (parsed.scheme(), port),
        ("http", Some(80)) | ("https", Some(443))
    );
    if should_strip {
        parsed
            .set_port(None)
            .map_err(|_| UrlFetchErrorKind::InvalidUrl)?;
    }
    Ok(())
}

fn normalize_path(parsed: &mut Url) {
    let path = parsed.path().to_string();
    if path.is_empty() {
        parsed.set_path("/");
        return;
    }
    if path.len() > 1 && path.ends_with('/') {
        let trimmed = path.trim_end_matches('/');
        if trimmed.is_empty() {
            parsed.set_path("/");
        } else {
            parsed.set_path(trimmed);
        }
    }
}

fn normalize_query(parsed: &mut Url) {
    let Some(_) = parsed.query() else {
        return;
    };

    let mut kept: Vec<(String, String)> = Vec::new();
    for (key, value) in parsed.query_pairs() {
        if is_tracking_param(key.as_ref()) {
            continue;
        }
        kept.push((key.to_string(), value.to_string()));
    }

    if kept.is_empty() {
        parsed.set_query(None);
        return;
    }

    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (key, value) in kept {
        serializer.append_pair(&key, &value);
    }
    let query = serializer.finish();
    parsed.set_query(Some(&query));
}

fn is_tracking_param(param: &str) -> bool {
    let lower = param.to_ascii_lowercase();
    if lower.starts_with("utm_") {
        return true;
    }
    matches!(
        lower.as_str(),
        "gclid"
            | "fbclid"
            | "msclkid"
            | "dclid"
            | "yclid"
            | "igshid"
            | "mc_cid"
            | "mc_eid"
            | "_hsenc"
            | "_hsmi"
            | "ref_src"
    )
}
