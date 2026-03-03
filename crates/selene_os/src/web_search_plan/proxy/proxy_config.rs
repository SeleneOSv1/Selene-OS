#![forbid(unsafe_code)]

use crate::web_search_plan::proxy::ProxyMode;
use std::collections::BTreeMap;
use std::sync::OnceLock;

static PROCESS_PROXY_MODE: OnceLock<ProxyMode> = OnceLock::new();

pub trait EnvProvider {
    fn get_var(&self, key: &str) -> Option<String>;
}

#[derive(Debug, Default)]
pub struct SystemEnvProvider;

impl EnvProvider for SystemEnvProvider {
    fn get_var(&self, key: &str) -> Option<String> {
        std::env::var(key)
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    }
}

#[derive(Debug, Clone, Default)]
pub struct MapEnvProvider {
    vars: BTreeMap<String, String>,
}

impl MapEnvProvider {
    pub fn new(entries: &[(&str, &str)]) -> Self {
        let vars = entries
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Self { vars }
    }
}

impl EnvProvider for MapEnvProvider {
    fn get_var(&self, key: &str) -> Option<String> {
        self.vars
            .get(key)
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyConfig {
    pub mode: ProxyMode,
    pub http_proxy_url: Option<String>,
    pub https_proxy_url: Option<String>,
}

impl ProxyConfig {
    pub fn from_env(mode: ProxyMode, env: &impl EnvProvider) -> Self {
        match mode {
            ProxyMode::Off => Self {
                mode,
                http_proxy_url: None,
                https_proxy_url: None,
            },
            ProxyMode::Env => Self {
                mode,
                http_proxy_url: env.get_var("HTTP_PROXY"),
                https_proxy_url: env.get_var("HTTPS_PROXY"),
            },
            ProxyMode::Explicit => Self {
                mode,
                http_proxy_url: env.get_var("SELENE_HTTP_PROXY_URL"),
                https_proxy_url: env.get_var("SELENE_HTTPS_PROXY_URL"),
            },
        }
    }

    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        match self.mode {
            ProxyMode::Off => Vec::new(),
            ProxyMode::Env => {
                let mut missing = Vec::new();
                if self.http_proxy_url.is_none() {
                    missing.push("HTTP_PROXY");
                }
                if self.https_proxy_url.is_none() {
                    missing.push("HTTPS_PROXY");
                }
                missing
            }
            ProxyMode::Explicit => {
                let mut missing = Vec::new();
                if self.http_proxy_url.is_none() {
                    missing.push("SELENE_HTTP_PROXY_URL");
                }
                if self.https_proxy_url.is_none() {
                    missing.push("SELENE_HTTPS_PROXY_URL");
                }
                missing
            }
        }
    }
}

pub fn select_process_proxy_mode(mode: ProxyMode) -> Result<ProxyMode, String> {
    select_proxy_mode_with_lock(&PROCESS_PROXY_MODE, mode)
}

pub fn select_proxy_mode_with_lock(
    lock: &OnceLock<ProxyMode>,
    mode: ProxyMode,
) -> Result<ProxyMode, String> {
    match lock.get() {
        Some(existing) if *existing != mode => Err(format!(
            "proxy mode already locked to {}; refused switch to {}",
            existing.as_str(),
            mode.as_str()
        )),
        Some(existing) => Ok(*existing),
        None => {
            let _ = lock.set(mode);
            Ok(mode)
        }
    }
}
