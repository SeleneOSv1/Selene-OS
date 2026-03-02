#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use rand::rngs::OsRng;
use rand::RngCore;
use selene_kernel_contracts::provider_secrets::ProviderSecretId;
use serde::{Deserialize, Serialize};

const VAULT_SCHEMA_VERSION: u8 = 1;
const MASTER_KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

#[derive(Debug)]
pub enum VaultError {
    UnknownKeyId(String),
    InvalidSecretValue,
    Io(std::io::Error),
    Json(serde_json::Error),
    Decode(base64::DecodeError),
    Crypto,
}

impl std::fmt::Display for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownKeyId(key) => write!(f, "unknown key id: {key}"),
            Self::InvalidSecretValue => write!(f, "invalid secret value"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Json(err) => write!(f, "json error: {err}"),
            Self::Decode(err) => write!(f, "decode error: {err}"),
            Self::Crypto => write!(f, "vault cryptographic operation failed"),
        }
    }
}

impl std::error::Error for VaultError {}

impl From<std::io::Error> for VaultError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for VaultError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<base64::DecodeError> for VaultError {
    fn from(value: base64::DecodeError) -> Self {
        Self::Decode(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct VaultDocument {
    schema_version: u8,
    entries: BTreeMap<String, VaultEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VaultEntry {
    nonce_b64: String,
    ciphertext_b64: String,
    updated_at_unix_ms: u64,
}

#[derive(Debug, Clone)]
pub struct DeviceVault {
    vault_path: PathBuf,
    key_path: PathBuf,
}

impl DeviceVault {
    pub fn default_local() -> Self {
        let vault_path = env::var("SELENE_DEVICE_VAULT_PATH")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(default_vault_path);
        let mut key_path = vault_path.clone();
        key_path.set_extension("master.key");
        Self::for_paths(vault_path, key_path)
    }

    pub fn for_paths(vault_path: PathBuf, key_path: PathBuf) -> Self {
        Self {
            vault_path,
            key_path,
        }
    }

    pub fn set_secret(&self, key_id: &str, value: &str) -> Result<(), VaultError> {
        validate_key_id(key_id)?;
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(VaultError::InvalidSecretValue);
        }

        self.ensure_parent_dirs()?;
        let key = self.load_or_create_master_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| VaultError::Crypto)?;
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, trimmed.as_bytes())
            .map_err(|_| VaultError::Crypto)?;

        let mut doc = self.read_document()?.unwrap_or_default();
        doc.schema_version = VAULT_SCHEMA_VERSION;
        doc.entries.insert(
            key_id.to_string(),
            VaultEntry {
                nonce_b64: BASE64.encode(nonce_bytes),
                ciphertext_b64: BASE64.encode(ciphertext),
                updated_at_unix_ms: now_unix_ms(),
            },
        );
        self.write_document(&doc)?;
        Ok(())
    }

    pub fn resolve_secret(&self, key_id: &str) -> Result<Option<String>, VaultError> {
        validate_key_id(key_id)?;
        let Some(doc) = self.read_document()? else {
            return Ok(None);
        };
        let Some(entry) = doc.entries.get(key_id) else {
            return Ok(None);
        };
        let key = self.load_or_create_master_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| VaultError::Crypto)?;
        let nonce_raw = BASE64.decode(entry.nonce_b64.as_bytes())?;
        if nonce_raw.len() != NONCE_LEN {
            return Err(VaultError::Crypto);
        }
        let nonce = Nonce::from_slice(&nonce_raw);
        let ciphertext = BASE64.decode(entry.ciphertext_b64.as_bytes())?;
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| VaultError::Crypto)?;
        let secret = String::from_utf8(plaintext).map_err(|_| VaultError::Crypto)?;
        if secret.trim().is_empty() {
            return Ok(None);
        }
        Ok(Some(secret))
    }

    pub fn has_secret(&self, key_id: &str) -> Result<bool, VaultError> {
        Ok(self.resolve_secret(key_id)?.is_some())
    }

    pub fn delete_secret(&self, key_id: &str) -> Result<bool, VaultError> {
        validate_key_id(key_id)?;
        let Some(mut doc) = self.read_document()? else {
            return Ok(false);
        };
        let removed = doc.entries.remove(key_id).is_some();
        if removed {
            self.write_document(&doc)?;
        }
        Ok(removed)
    }

    pub fn list_secret_ids(&self) -> Result<Vec<String>, VaultError> {
        let Some(doc) = self.read_document()? else {
            return Ok(Vec::new());
        };
        let mut keys: Vec<String> = doc
            .entries
            .keys()
            .filter(|key| ProviderSecretId::parse(key).is_some())
            .cloned()
            .collect();
        keys.sort();
        Ok(keys)
    }

    fn ensure_parent_dirs(&self) -> Result<(), VaultError> {
        if let Some(parent) = self.vault_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = self.key_path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    fn read_document(&self) -> Result<Option<VaultDocument>, VaultError> {
        if !self.vault_path.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(&self.vault_path)?;
        if raw.trim().is_empty() {
            return Ok(Some(VaultDocument::default()));
        }
        let doc = serde_json::from_str::<VaultDocument>(&raw)?;
        if doc.schema_version != VAULT_SCHEMA_VERSION {
            return Err(VaultError::Crypto);
        }
        Ok(Some(doc))
    }

    fn write_document(&self, doc: &VaultDocument) -> Result<(), VaultError> {
        self.ensure_parent_dirs()?;
        let serialized = serde_json::to_vec_pretty(doc)?;
        atomic_write(&self.vault_path, &serialized)?;
        Ok(())
    }

    fn load_or_create_master_key(&self) -> Result<[u8; MASTER_KEY_LEN], VaultError> {
        if self.key_path.exists() {
            let encoded = fs::read_to_string(&self.key_path)?;
            let decoded = BASE64.decode(encoded.trim().as_bytes())?;
            if decoded.len() != MASTER_KEY_LEN {
                return Err(VaultError::Crypto);
            }
            let mut key = [0u8; MASTER_KEY_LEN];
            key.copy_from_slice(&decoded);
            return Ok(key);
        }

        self.ensure_parent_dirs()?;
        let mut key = [0u8; MASTER_KEY_LEN];
        OsRng.fill_bytes(&mut key);
        let encoded = BASE64.encode(key);
        write_new_file_restricted(&self.key_path, encoded.as_bytes())?;
        Ok(key)
    }
}

pub fn resolve_secret(key_id: &str) -> Result<Option<String>, VaultError> {
    DeviceVault::default_local().resolve_secret(key_id)
}

fn validate_key_id(raw: &str) -> Result<ProviderSecretId, VaultError> {
    ProviderSecretId::parse(raw).ok_or_else(|| VaultError::UnknownKeyId(raw.to_string()))
}

fn default_vault_path() -> PathBuf {
    if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg_config_home).join("selene").join("device_vault.json");
    }
    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("selene")
            .join("device_vault.json");
    }
    PathBuf::from(".selene").join("device_vault.json")
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(1)
        .max(1)
}

fn atomic_write(path: &Path, data: &[u8]) -> Result<(), VaultError> {
    let mut tmp = path.to_path_buf();
    tmp.set_extension("tmp");
    fs::write(&tmp, data)?;
    fs::rename(tmp, path)?;
    Ok(())
}

fn write_new_file_restricted(path: &Path, data: &[u8]) -> Result<(), VaultError> {
    let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;
    file.write_all(data)?;
    file.flush()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        fs::set_permissions(path, perms)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{DeviceVault, VaultError};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_paths(name: &str) -> (PathBuf, PathBuf, PathBuf) {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(1);
        let base = std::env::temp_dir().join(format!("selene-vault-test-{name}-{suffix}"));
        let vault_path = base.join("device_vault.json");
        let key_path = base.join("device_vault.master.key");
        (base, vault_path, key_path)
    }

    #[test]
    fn at_vault_01_set_get_roundtrip_keeps_plaintext_out_of_file() {
        let (base, vault_path, key_path) = temp_paths("roundtrip");
        fs::create_dir_all(&base).unwrap();
        let vault = DeviceVault::for_paths(vault_path.clone(), key_path);
        let sentinel = "TOP_SECRET_SENTINEL_123";

        vault
            .set_secret("brave_search_api_key", sentinel)
            .expect("set should succeed");
        let got = vault
            .resolve_secret("brave_search_api_key")
            .expect("resolve should succeed")
            .expect("secret should exist");
        assert_eq!(got, sentinel);

        let raw = fs::read_to_string(vault_path).expect("vault file should exist");
        assert!(!raw.contains(sentinel));
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn at_vault_02_has_and_delete_behave_deterministically() {
        let (base, vault_path, key_path) = temp_paths("has-del");
        fs::create_dir_all(&base).unwrap();
        let vault = DeviceVault::for_paths(vault_path, key_path);

        assert!(!vault.has_secret("openai_api_key").unwrap());
        vault
            .set_secret("openai_api_key", "sk-demo")
            .expect("set should succeed");
        assert!(vault.has_secret("openai_api_key").unwrap());
        assert!(vault.delete_secret("openai_api_key").unwrap());
        assert!(!vault.has_secret("openai_api_key").unwrap());
        assert!(!vault.delete_secret("openai_api_key").unwrap());
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn at_vault_03_unknown_key_id_fails_closed() {
        let (base, vault_path, key_path) = temp_paths("unknown");
        fs::create_dir_all(&base).unwrap();
        let vault = DeviceVault::for_paths(vault_path, key_path);
        let err = vault
            .set_secret("not_real_secret", "value")
            .expect_err("unknown key must fail");
        assert!(matches!(err, VaultError::UnknownKeyId(_)));
        fs::remove_dir_all(base).unwrap();
    }
}
