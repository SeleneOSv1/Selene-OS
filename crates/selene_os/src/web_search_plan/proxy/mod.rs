#![forbid(unsafe_code)]

pub mod proxy_config;
pub mod proxy_redaction;
pub mod proxy_retry;
pub mod proxy_self_check;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProxyMode {
    Off,
    Env,
    Explicit,
}

impl ProxyMode {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "off" => Ok(Self::Off),
            "env" => Ok(Self::Env),
            "explicit" => Ok(Self::Explicit),
            other => Err(format!("unsupported proxy mode {}", other)),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Env => "env",
            Self::Explicit => "explicit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProxySelfCheckSeverity {
    Warn,
    Critical,
}

impl ProxySelfCheckSeverity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Warn => "warn",
            Self::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProxyErrorKind {
    ProxyMisconfigured,
    ProxyAuthFailed,
    ProxyConnectFailed,
    ProxyTlsFailed,
    ProxyDnsFailed,
    ProxyTimeout,
}

impl ProxyErrorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProxyMisconfigured => "proxy_misconfigured",
            Self::ProxyAuthFailed => "proxy_auth_failed",
            Self::ProxyConnectFailed => "proxy_connect_failed",
            Self::ProxyTlsFailed => "proxy_tls_failed",
            Self::ProxyDnsFailed => "proxy_dns_failed",
            Self::ProxyTimeout => "proxy_timeout",
        }
    }

    pub const fn reason_code(self) -> &'static str {
        "proxy_misconfigured"
    }
}

#[cfg(test)]
pub mod proxy_tests;
