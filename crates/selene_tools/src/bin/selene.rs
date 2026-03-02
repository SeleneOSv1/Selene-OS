#![forbid(unsafe_code)]

use std::env;
use std::io::{self, IsTerminal, Read};

use selene_engines::device_vault::DeviceVault;
use selene_tools::vault_cli::{execute_vault_command, parse_provider_secret_id};

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args[0] != "vault" {
        return Err("usage: selene vault <set|has|del|ls> [key_id]".to_string());
    }

    let subcommand = args
        .get(1)
        .ok_or_else(|| "usage: selene vault <set|has|del|ls> [key_id]".to_string())?
        .as_str();
    let key_id = args.get(2).map(String::as_str);
    let value = if subcommand == "set" {
        let key = key_id.ok_or_else(|| "usage: selene vault set <key_id>".to_string())?;
        let parsed = parse_provider_secret_id(key)?;
        Some(read_secret_value(parsed.as_str())?)
    } else {
        None
    };

    let vault = DeviceVault::default_local();
    let output = execute_vault_command(&vault, subcommand, key_id, value.as_deref())?;
    if !output.is_empty() {
        println!("{output}");
    }
    Ok(())
}

fn read_secret_value(key_id: &str) -> Result<String, String> {
    if io::stdin().is_terminal() {
        let prompt = format!("Enter value for {key_id}:");
        let value = rpassword::prompt_password(prompt).map_err(|e| e.to_string())?;
        if value.trim().is_empty() {
            return Err("secret value must not be empty".to_string());
        }
        Ok(value)
    } else {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|e| e.to_string())?;
        let trimmed = input.trim().to_string();
        if trimmed.is_empty() {
            return Err("secret value must not be empty".to_string());
        }
        Ok(trimmed)
    }
}
