#![forbid(unsafe_code)]

use selene_os::web_search_plan::eval::{generate_eval_report, ContinuousEvalConfig};
use std::path::PathBuf;

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse_args(std::env::args().skip(1).collect())?;
    let outcome = generate_eval_report(&ContinuousEvalConfig {
        head_commit: args.head_commit,
        run_timestamp_utc: args.timestamp_utc,
        date_tag: args.date_tag,
        output_dir: args.output_dir,
    })?;

    println!("EVAL_REPORT_FILE={}", outcome.report_path.display());
    println!(
        "EVAL_OVERALL={}",
        if outcome.overall_pass { "PASS" } else { "FAIL" }
    );
    if outcome.failing_case_ids.is_empty() {
        println!("EVAL_FAILING_CASE_IDS=none");
    } else {
        println!(
            "EVAL_FAILING_CASE_IDS={}",
            outcome.failing_case_ids.join(",")
        );
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct CliArgs {
    head_commit: String,
    timestamp_utc: String,
    date_tag: String,
    output_dir: PathBuf,
}

fn parse_args(raw: Vec<String>) -> Result<CliArgs, String> {
    let mut head_commit = String::new();
    let mut timestamp_utc = String::new();
    let mut date_tag = String::new();
    let mut output_dir = None;

    let mut index = 0usize;
    while index < raw.len() {
        let key = raw[index].as_str();
        let value = raw
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {}", key))?;
        match key {
            "--head-commit" => head_commit = value.clone(),
            "--timestamp-utc" => timestamp_utc = value.clone(),
            "--date-tag" => date_tag = value.clone(),
            "--output-dir" => output_dir = Some(PathBuf::from(value)),
            unknown => return Err(format!("unsupported argument {}", unknown)),
        }
        index += 2;
    }

    if head_commit.trim().is_empty() {
        return Err("--head-commit is required".to_string());
    }
    if timestamp_utc.trim().is_empty() {
        return Err("--timestamp-utc is required".to_string());
    }
    if date_tag.trim().is_empty() {
        return Err("--date-tag is required".to_string());
    }

    Ok(CliArgs {
        head_commit,
        timestamp_utc,
        date_tag,
        output_dir: output_dir.ok_or_else(|| "--output-dir is required".to_string())?,
    })
}
