#![forbid(unsafe_code)]

use selene_os::web_search_plan::release::{
    generate_release_evidence_pack, GenerateReleaseEvidenceConfig,
};
use std::path::PathBuf;

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse_args(std::env::args().skip(1).collect())?;
    let output_path = generate_release_evidence_pack(&GenerateReleaseEvidenceConfig {
        head_commit: args.head_commit,
        branch: args.branch,
        run30_timestamp_utc: args.timestamp_utc,
        date_tag: args.date_tag,
        release_lock_results_path: args.release_lock_results,
        slo_lock_results_path: args.slo_lock_results,
        output_dir: args.output_dir,
    })?;

    println!("RELEASE_EVIDENCE_PACK={}", output_path.display());
    Ok(())
}

#[derive(Debug, Clone)]
struct CliArgs {
    head_commit: String,
    branch: String,
    timestamp_utc: String,
    date_tag: String,
    release_lock_results: PathBuf,
    slo_lock_results: PathBuf,
    output_dir: PathBuf,
}

fn parse_args(raw: Vec<String>) -> Result<CliArgs, String> {
    let mut head_commit = String::new();
    let mut branch = String::new();
    let mut timestamp_utc = String::new();
    let mut date_tag = String::new();
    let mut release_lock_results = None;
    let mut slo_lock_results = None;
    let mut output_dir = None;

    let mut index = 0usize;
    while index < raw.len() {
        let key = raw[index].as_str();
        let value = raw
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {}", key))?;
        match key {
            "--head-commit" => head_commit = value.clone(),
            "--branch" => branch = value.clone(),
            "--timestamp-utc" => timestamp_utc = value.clone(),
            "--date-tag" => date_tag = value.clone(),
            "--release-lock-results" => {
                release_lock_results = Some(PathBuf::from(value));
            }
            "--slo-lock-results" => {
                slo_lock_results = Some(PathBuf::from(value));
            }
            "--output-dir" => {
                output_dir = Some(PathBuf::from(value));
            }
            unknown => return Err(format!("unsupported argument {}", unknown)),
        }
        index += 2;
    }

    Ok(CliArgs {
        head_commit,
        branch,
        timestamp_utc,
        date_tag,
        release_lock_results: release_lock_results
            .ok_or_else(|| "--release-lock-results is required".to_string())?,
        slo_lock_results: slo_lock_results
            .ok_or_else(|| "--slo-lock-results is required".to_string())?,
        output_dir: output_dir.ok_or_else(|| "--output-dir is required".to_string())?,
    })
}
