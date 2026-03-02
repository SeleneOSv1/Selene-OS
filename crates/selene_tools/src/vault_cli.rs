#![forbid(unsafe_code)]

use selene_engines::device_vault::DeviceVault;
use selene_kernel_contracts::provider_secrets::ProviderSecretId;

pub fn execute_vault_command(
    vault: &DeviceVault,
    subcommand: &str,
    key_id: Option<&str>,
    value: Option<&str>,
) -> Result<String, String> {
    match subcommand {
        "set" => {
            let key = require_key_id(key_id)?;
            let raw = value.ok_or_else(|| "missing secret input value".to_string())?;
            vault
                .set_secret(key.as_str(), raw)
                .map_err(|e| format!("failed to store key: {e}"))?;
            Ok("OK".to_string())
        }
        "has" => {
            let key = require_key_id(key_id)?;
            let has = vault
                .has_secret(key.as_str())
                .map_err(|e| format!("failed to check key: {e}"))?;
            if has {
                Ok("YES".to_string())
            } else {
                Ok("NO".to_string())
            }
        }
        "del" => {
            let key = require_key_id(key_id)?;
            vault
                .delete_secret(key.as_str())
                .map_err(|e| format!("failed to delete key: {e}"))?;
            Ok("OK".to_string())
        }
        "ls" => {
            let keys = vault
                .list_secret_ids()
                .map_err(|e| format!("failed to list keys: {e}"))?;
            Ok(keys.join("\n"))
        }
        _ => Err(format!(
            "unknown vault subcommand: {subcommand}. expected one of: set, has, del, ls"
        )),
    }
}

pub fn parse_provider_secret_id(raw: &str) -> Result<ProviderSecretId, String> {
    ProviderSecretId::parse(raw).ok_or_else(|| {
        let allowed = ProviderSecretId::allowed_key_names().join(", ");
        format!("unknown key id '{raw}'. allowed: {allowed}")
    })
}

fn require_key_id(raw: Option<&str>) -> Result<ProviderSecretId, String> {
    let raw = raw.ok_or_else(|| {
        let allowed = ProviderSecretId::allowed_key_names().join(", ");
        format!("missing key id. allowed: {allowed}")
    })?;
    parse_provider_secret_id(raw)
}

#[cfg(test)]
mod tests {
    use super::execute_vault_command;
    use selene_engines::device_vault::DeviceVault;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_vault() -> (PathBuf, DeviceVault) {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(1);
        let base = std::env::temp_dir().join(format!("selene-vault-cli-test-{suffix}"));
        fs::create_dir_all(&base).unwrap();
        let vault = DeviceVault::for_paths(base.join("vault.json"), base.join("vault.master.key"));
        (base, vault)
    }

    #[test]
    fn at_vault_cli_01_set_has_del_roundtrip() {
        let (base, vault) = temp_vault();
        assert_eq!(
            execute_vault_command(
                &vault,
                "set",
                Some("brave_search_api_key"),
                Some("brave-secret")
            )
            .unwrap(),
            "OK"
        );
        assert_eq!(
            execute_vault_command(&vault, "has", Some("brave_search_api_key"), None).unwrap(),
            "YES"
        );
        assert_eq!(
            execute_vault_command(&vault, "del", Some("brave_search_api_key"), None).unwrap(),
            "OK"
        );
        assert_eq!(
            execute_vault_command(&vault, "has", Some("brave_search_api_key"), None).unwrap(),
            "NO"
        );
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn at_vault_cli_02_output_never_contains_secret_value() {
        let (base, vault) = temp_vault();
        let sentinel = "DO_NOT_LEAK_SENTINEL";
        let out = execute_vault_command(
            &vault,
            "set",
            Some("openai_api_key"),
            Some(sentinel),
        )
        .unwrap();
        assert!(!out.contains(sentinel));
        fs::remove_dir_all(base).unwrap();
    }
}
