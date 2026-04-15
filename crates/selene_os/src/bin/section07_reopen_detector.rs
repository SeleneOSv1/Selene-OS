#![forbid(unsafe_code)]

use selene_os::ph1builder::{
    detect_section07_reopen, render_section07_reopen_report, Section07ReopenDetectorInput,
};
use std::process::ExitCode;

const USAGE: &str =
    "usage: section07_reopen_detector [--changed-file <path>]... [--symbol-hit <token>]...";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct CliArgs {
    changed_files: Vec<String>,
    symbol_hits: Vec<String>,
    help_requested: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliRunResult {
    exit_code: u8,
    stdout: String,
    stderr: String,
}

fn main() -> ExitCode {
    let result = run_cli(std::env::args().skip(1).collect());
    if !result.stdout.is_empty() {
        print!("{}", result.stdout);
    }
    if !result.stderr.is_empty() {
        eprint!("{}", result.stderr);
    }
    ExitCode::from(result.exit_code)
}

fn run_cli(raw: Vec<String>) -> CliRunResult {
    match parse_args(raw) {
        Ok(args) if args.help_requested => CliRunResult {
            exit_code: 0,
            stdout: format!("{USAGE}\n"),
            stderr: String::new(),
        },
        Ok(args) => match detect_section07_reopen(&Section07ReopenDetectorInput {
            changed_files: args.changed_files,
            symbol_hits: args.symbol_hits,
        }) {
            Ok(report) => CliRunResult {
                exit_code: 0,
                stdout: format!("{}\n", render_section07_reopen_report(&report)),
                stderr: String::new(),
            },
            Err(error) => CliRunResult {
                exit_code: 2,
                stdout: String::new(),
                stderr: format!("{error:?}\n"),
            },
        },
        Err(message) => CliRunResult {
            exit_code: 64,
            stdout: String::new(),
            stderr: format!("{message}\n{USAGE}\n"),
        },
    }
}

fn parse_args(raw: Vec<String>) -> Result<CliArgs, String> {
    let mut parsed = CliArgs::default();
    let mut index = 0usize;

    while index < raw.len() {
        match raw[index].as_str() {
            "-h" | "--help" => {
                parsed.help_requested = true;
                index += 1;
            }
            "--changed-file" => {
                let value = flag_value(&raw, index, "--changed-file")?;
                parsed.changed_files.push(value.to_string());
                index += 2;
            }
            "--symbol-hit" => {
                let value = flag_value(&raw, index, "--symbol-hit")?;
                parsed.symbol_hits.push(value.to_string());
                index += 2;
            }
            unknown => return Err(format!("unsupported argument {unknown}")),
        }
    }

    Ok(parsed)
}

fn flag_value<'a>(raw: &'a [String], index: usize, flag: &str) -> Result<&'a str, String> {
    let value = raw
        .get(index + 1)
        .ok_or_else(|| format!("missing value for {flag}"))?;
    if value.starts_with('-') {
        return Err(format!("missing value for {flag}"));
    }
    Ok(value.as_str())
}

#[cfg(test)]
mod section07_reopen_detector {
    use super::*;

    mod tests {
        use super::*;

        #[test]
        fn at_section07_reopen_detector_bin_01_help_exits_cleanly() {
            let result = run_cli(vec!["--help".to_string()]);

            assert_eq!(result.exit_code, 0);
            assert_eq!(result.stdout, format!("{USAGE}\n"));
            assert!(result.stderr.is_empty());
        }

        #[test]
        fn at_section07_reopen_detector_bin_02_empty_input_prints_still_blocked() {
            let result = run_cli(vec![]);

            assert_eq!(result.exit_code, 0);
            assert_eq!(
                result.stdout,
                "status=StillBlocked\nmatched_watchlist_files=none\nmatched_symbols=none\nexplanation=no Section 07 watchlist files changed\n"
            );
            assert!(result.stderr.is_empty());
        }

        #[test]
        fn at_section07_reopen_detector_bin_03_program_d_candidate_prints_stable_report() {
            let result = run_cli(vec![
                "--changed-file".to_string(),
                "crates/selene_os/src/runtime_governance.rs".to_string(),
                "--changed-file".to_string(),
                "crates/selene_storage/src/repo.rs".to_string(),
                "--symbol-hit".to_string(),
                "cluster_consistency".to_string(),
                "--symbol-hit".to_string(),
                "append_builder_proposal_row".to_string(),
            ]);

            assert_eq!(result.exit_code, 0);
            assert_eq!(
                result.stdout,
                "status=ProgramDReopenCandidate\nmatched_watchlist_files=crates/selene_os/src/runtime_governance.rs,crates/selene_storage/src/repo.rs\nmatched_symbols=append_,cluster_consistency\nexplanation=Program D reopen candidate criteria matched with runtime producer and storage bridge evidence\n"
            );
            assert!(result.stderr.is_empty());
        }

        #[test]
        fn at_section07_reopen_detector_bin_04_program_e_candidate_prints_stable_report() {
            let result = run_cli(vec![
                "--changed-file".to_string(),
                "crates/selene_os/src/ph1j.rs".to_string(),
                "--changed-file".to_string(),
                "crates/selene_storage/src/repo.rs".to_string(),
                "--symbol-hit".to_string(),
                "certification_target_ref".to_string(),
            ]);

            assert_eq!(result.exit_code, 0);
            assert_eq!(
                result.stdout,
                "status=ProgramEReopenCandidate\nmatched_watchlist_files=crates/selene_os/src/ph1j.rs,crates/selene_storage/src/repo.rs\nmatched_symbols=certification_target,certification_target_ref\nexplanation=Program E reopen candidate criteria matched with explicit target-bearing certification evidence plus proof/runtime and transport surface movement\n"
            );
            assert!(result.stderr.is_empty());
        }

        #[test]
        fn at_section07_reopen_detector_bin_05_unknown_flag_fails_closed() {
            let result = run_cli(vec!["--weird-flag".to_string()]);

            assert_eq!(result.exit_code, 64);
            assert_eq!(
                result.stderr,
                format!("unsupported argument --weird-flag\n{USAGE}\n")
            );
            assert!(result.stdout.is_empty());
        }

        #[test]
        fn at_section07_reopen_detector_bin_06_missing_flag_value_fails_closed() {
            let result = run_cli(vec!["--changed-file".to_string()]);

            assert_eq!(result.exit_code, 64);
            assert_eq!(
                result.stderr,
                format!("missing value for --changed-file\n{USAGE}\n")
            );
            assert!(result.stdout.is_empty());
        }
    }
}
