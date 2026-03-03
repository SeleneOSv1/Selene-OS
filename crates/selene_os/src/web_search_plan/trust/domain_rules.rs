#![forbid(unsafe_code)]

pub const TRUST_MODEL_VERSION: &str = "1.0.0";
pub const SPAM_SIGNAL_VERSION: &str = "1.0.0";

pub const OFFICIAL_ALLOWLIST: &[&str] = &[
    "sec.gov",
    "mas.gov.sg",
    "europa.eu",
    "bafin.de",
    "gov.sg",
];

pub const HIGH_ALLOWLIST: &[&str] = &[
    "iso.org",
    "nist.gov",
    "ieee.org",
    "standards.org",
    "oecd.org",
];

pub const MEDIUM_ALLOWLIST: &[&str] = &[
    "reuters.com",
    "bloomberg.com",
    "ft.com",
    "wsj.com",
    "cnbc.com",
];

pub const LOW_ALLOWLIST: &[&str] = &[
    "reddit.com",
    "medium.com",
    "blogspot.com",
    "wordpress.com",
];

pub const CLICKBAIT_KEYWORDS: &[&str] = &[
    "shocking",
    "unbelievable",
    "you won't believe",
    "you wont believe",
    "must see",
    "click here",
    "viral",
    "breaking!!!",
];

pub const TRACKING_QUERY_PARAMS: &[&str] = &[
    "utm_source",
    "utm_medium",
    "utm_campaign",
    "utm_term",
    "utm_content",
    "gclid",
    "fbclid",
    "msclkid",
    "dclid",
    "yclid",
    "mc_cid",
    "mc_eid",
    "ref",
    "ref_src",
    "cmpid",
];

pub fn domain_reputation_adjustment(host: &str) -> f64 {
    match host {
        "sec.gov" | "mas.gov.sg" | "europa.eu" | "bafin.de" => 0.06,
        "reuters.com" | "bloomberg.com" | "wsj.com" | "ft.com" => 0.04,
        "reddit.com" | "medium.com" | "blogspot.com" | "wordpress.com" => -0.05,
        _ => 0.0,
    }
}
