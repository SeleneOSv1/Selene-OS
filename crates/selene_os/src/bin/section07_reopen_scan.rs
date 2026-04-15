#![forbid(unsafe_code)]

use selene_os::ph1builder::{
    detect_section07_reopen, render_section07_reopen_report, Section07ReopenDetectorInput,
    Section07ReopenReport,
};
use std::collections::BTreeSet;
use std::process::{Command, ExitCode};

const USAGE: &str =
    "usage: section07_reopen_scan --repo-root <path> --base <rev> --head <rev>";

const SECTION07_WATCHLIST: [&str; 7] = [
    "crates/selene_os/src/runtime_governance.rs",
    "crates/selene_os/src/runtime_law.rs",
    "crates/selene_kernel_contracts/src/runtime_execution.rs",
    "crates/selene_storage/src/ph1f.rs",
    "crates/selene_storage/src/repo.rs",
    "crates/selene_os/src/ph1j.rs",
    "crates/selene_kernel_contracts/src/ph1j.rs",
];

const SECTION07_PROGRAM_D_RUNTIME_TOKENS: [&str; 6] = [
    "observe_node_policy_version",
    "GovernanceDecisionLogEntry",
    "cluster_consistency",
    "drift_signals",
    "quarantined_subsystems",
    "subsystem_certifications",
];

const SECTION07_PROGRAM_D_STORAGE_TOKENS: [&str; 8] = [
    "append_",
    "commit_row",
    "insert",
    "upsert",
    "persist",
    "store",
    "snapshot",
    "audit",
];

const SECTION07_PROGRAM_E_TARGET_TOKENS: [&str; 5] = [
    "IdentityCertification",
    "identity_certification",
    "certification_target",
    "identity_target",
    "certification_target_ref",
];

const SECTION07_PROGRAM_E_INSUFFICIENT_TOKENS: [&str; 8] = [
    "CanonicalProofRecordInput",
    "artifact_trust_state_from_receipt",
    "artifact_trust_entries",
    "artifact_trust_root_registry",
    "proof_record_ref",
    "proof_entry_ref",
    "decision_log_ref",
    "target_id",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliArgs {
    repo_root: String,
    base: String,
    head: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedArgs {
    Help,
    Run(CliArgs),
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
    run_cli_with(raw, run_git)
}

fn run_cli_with<F>(raw: Vec<String>, mut git_runner: F) -> CliRunResult
where
    F: FnMut(&str, &[&str]) -> Result<String, String>,
{
    match parse_args(raw) {
        Ok(ParsedArgs::Help) => CliRunResult {
            exit_code: 0,
            stdout: format!("{USAGE}\n"),
            stderr: String::new(),
        },
        Ok(ParsedArgs::Run(args)) => match scan_section07_reopen(&args, &mut git_runner) {
            Ok(report) => CliRunResult {
                exit_code: 0,
                stdout: format!("{}\n", render_section07_reopen_report(&report)),
                stderr: String::new(),
            },
            Err(message) => CliRunResult {
                exit_code: 2,
                stdout: String::new(),
                stderr: format!("{message}\n"),
            },
        },
        Err(message) => CliRunResult {
            exit_code: 64,
            stdout: String::new(),
            stderr: format!("{message}\n{USAGE}\n"),
        },
    }
}

fn parse_args(raw: Vec<String>) -> Result<ParsedArgs, String> {
    let mut repo_root = None;
    let mut base = None;
    let mut head = None;
    let mut help_requested = false;
    let mut index = 0usize;

    while index < raw.len() {
        match raw[index].as_str() {
            "-h" | "--help" => {
                help_requested = true;
                index += 1;
            }
            "--repo-root" => {
                let value = flag_value(&raw, index, "--repo-root")?;
                assign_unique_flag(&mut repo_root, value, "--repo-root")?;
                index += 2;
            }
            "--base" => {
                let value = flag_value(&raw, index, "--base")?;
                assign_unique_flag(&mut base, value, "--base")?;
                index += 2;
            }
            "--head" => {
                let value = flag_value(&raw, index, "--head")?;
                assign_unique_flag(&mut head, value, "--head")?;
                index += 2;
            }
            unknown => return Err(format!("unsupported argument {unknown}")),
        }
    }

    if help_requested {
        return Ok(ParsedArgs::Help);
    }

    Ok(ParsedArgs::Run(CliArgs {
        repo_root: repo_root.ok_or_else(|| "missing required argument --repo-root".to_string())?,
        base: base.ok_or_else(|| "missing required argument --base".to_string())?,
        head: head.ok_or_else(|| "missing required argument --head".to_string())?,
    }))
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

fn assign_unique_flag(slot: &mut Option<String>, value: &str, flag: &str) -> Result<(), String> {
    if slot.is_some() {
        return Err(format!("duplicate argument {flag}"));
    }
    *slot = Some(value.to_string());
    Ok(())
}

fn scan_section07_reopen<F>(
    args: &CliArgs,
    git_runner: &mut F,
) -> Result<Section07ReopenReport, String>
where
    F: FnMut(&str, &[&str]) -> Result<String, String>,
{
    let range = format!("{}..{}", args.base, args.head);
    let name_only = run_diff_name_only(args, &range, git_runner)?;
    let patch = run_diff_patch(args, &range, git_runner)?;

    detect_section07_reopen(&Section07ReopenDetectorInput {
        changed_files: collect_changed_files_from_name_only(&name_only),
        symbol_hits: collect_symbol_hits_from_patch(&patch),
    })
    .map_err(|error| format!("{error:?}"))
}

fn run_diff_name_only<F>(args: &CliArgs, range: &str, git_runner: &mut F) -> Result<String, String>
where
    F: FnMut(&str, &[&str]) -> Result<String, String>,
{
    let mut git_args = vec![
        "diff".to_string(),
        "--name-only".to_string(),
        range.to_string(),
        "--".to_string(),
    ];
    git_args.extend(SECTION07_WATCHLIST.iter().map(|path| (*path).to_string()));
    invoke_git_runner(git_runner, &args.repo_root, &git_args)
}

fn run_diff_patch<F>(args: &CliArgs, range: &str, git_runner: &mut F) -> Result<String, String>
where
    F: FnMut(&str, &[&str]) -> Result<String, String>,
{
    let mut git_args = vec![
        "diff".to_string(),
        "--unified=0".to_string(),
        range.to_string(),
        "--".to_string(),
    ];
    git_args.extend(SECTION07_WATCHLIST.iter().map(|path| (*path).to_string()));
    invoke_git_runner(git_runner, &args.repo_root, &git_args)
}

fn invoke_git_runner<F>(
    git_runner: &mut F,
    repo_root: &str,
    git_args: &[String],
) -> Result<String, String>
where
    F: FnMut(&str, &[&str]) -> Result<String, String>,
{
    let refs = git_args.iter().map(String::as_str).collect::<Vec<_>>();
    git_runner(repo_root, &refs)
}

fn collect_changed_files_from_name_only(name_only: &str) -> Vec<String> {
    let mut matched = BTreeSet::new();

    for line in name_only.lines() {
        let candidate = line.trim().replace('\\', "/");
        if candidate.is_empty() {
            continue;
        }
        for watch in SECTION07_WATCHLIST {
            if section07_path_matches(&candidate, watch) {
                matched.insert(watch.to_string());
            }
        }
    }

    matched.into_iter().collect()
}

fn collect_symbol_hits_from_patch(patch: &str) -> Vec<String> {
    let mut matched = BTreeSet::new();

    for line in patch.lines() {
        if !is_changed_diff_line(line) {
            continue;
        }

        for token in SECTION07_PROGRAM_D_RUNTIME_TOKENS {
            if line.contains(token) {
                matched.insert(token.to_string());
            }
        }
        for token in SECTION07_PROGRAM_D_STORAGE_TOKENS {
            if line.contains(token) {
                matched.insert(token.to_string());
            }
        }
        for token in SECTION07_PROGRAM_E_TARGET_TOKENS {
            if line.contains(token) {
                matched.insert(token.to_string());
            }
        }
        for token in SECTION07_PROGRAM_E_INSUFFICIENT_TOKENS {
            if line.contains(token) {
                matched.insert(token.to_string());
            }
        }
    }

    matched.into_iter().collect()
}

fn is_changed_diff_line(line: &str) -> bool {
    (line.starts_with('+') && !line.starts_with("+++"))
        || (line.starts_with('-') && !line.starts_with("---"))
}

fn section07_path_matches(candidate: &str, watch: &str) -> bool {
    candidate == watch || candidate.ends_with(&format!("/{watch}"))
}

fn run_git(repo_root: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .map_err(|error| format!("git command failed: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            return Err(format!("git command failed with status {}", output.status));
        }
        return Err(format!("git command failed: {stderr}"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod section07_reopen_scan {
    use super::*;

    mod tests {
        use super::*;

        #[test]
        fn at_section07_reopen_scan_bin_01_help_exits_cleanly() {
            let result = run_cli_with(vec!["--help".to_string()], |_, _| {
                panic!("git should not run for --help")
            });

            assert_eq!(result.exit_code, 0);
            assert_eq!(result.stdout, format!("{USAGE}\n"));
            assert!(result.stderr.is_empty());
        }

        #[test]
        fn at_section07_reopen_scan_bin_02_name_only_output_collects_and_sorts_watchlist_files() {
            let collected = collect_changed_files_from_name_only(
                "crates/selene_storage/src/repo.rs\n\
                 crates/selene_os/src/runtime_governance.rs\n\
                 crates/selene_storage/src/repo.rs\n\
                 crates/selene_os/src/not_section07.rs\n",
            );

            assert_eq!(
                collected,
                vec![
                    "crates/selene_os/src/runtime_governance.rs".to_string(),
                    "crates/selene_storage/src/repo.rs".to_string(),
                ]
            );
        }

        #[test]
        fn at_section07_reopen_scan_bin_03_patch_output_collects_program_d_tokens() {
            let collected = collect_symbol_hits_from_patch(
                "diff --git a/x b/x\n\
                 @@ -1 +1 @@\n\
                 +cluster_consistency\n\
                 -append_builder_proposal_row\n\
                 +persist_runtime_drift_snapshot\n",
            );

            assert_eq!(
                collected,
                vec![
                    "append_".to_string(),
                    "cluster_consistency".to_string(),
                    "persist".to_string(),
                    "snapshot".to_string(),
                ]
            );
        }

        #[test]
        fn at_section07_reopen_scan_bin_04_patch_output_collects_program_e_and_insufficient_tokens(
        ) {
            let collected = collect_symbol_hits_from_patch(
                "diff --git a/x b/x\n\
                 @@ -1 +1 @@\n\
                 +certification_target_ref\n\
                 +artifact_trust_entries\n\
                 -decision_log_ref\n\
                 +target_id\n",
            );

            assert_eq!(
                collected,
                vec![
                    "artifact_trust_entries".to_string(),
                    "certification_target".to_string(),
                    "certification_target_ref".to_string(),
                    "decision_log_ref".to_string(),
                    "target_id".to_string(),
                ]
            );
        }

        #[test]
        fn at_section07_reopen_scan_bin_05_program_d_candidate_diff_prints_stable_report() {
            let result = run_cli_with(
                vec![
                    "--repo-root".to_string(),
                    "/repo".to_string(),
                    "--base".to_string(),
                    "abc".to_string(),
                    "--head".to_string(),
                    "def".to_string(),
                ],
                |_, args| {
                    if args.contains(&"--name-only") {
                        Ok("crates/selene_storage/src/repo.rs\ncrates/selene_os/src/runtime_governance.rs\n"
                            .to_string())
                    } else if args.contains(&"--unified=0") {
                        Ok("diff --git a/x b/x\n@@ -1 +1 @@\n+cluster_consistency\n+append_builder_proposal_row\n".to_string())
                    } else {
                        Err("unexpected git args".to_string())
                    }
                },
            );

            assert_eq!(result.exit_code, 0);
            assert_eq!(
                result.stdout,
                "status=ProgramDReopenCandidate\nmatched_watchlist_files=crates/selene_os/src/runtime_governance.rs,crates/selene_storage/src/repo.rs\nmatched_symbols=append_,cluster_consistency\nexplanation=Program D reopen candidate criteria matched with runtime producer and storage bridge evidence\n"
            );
            assert!(result.stderr.is_empty());
        }

        #[test]
        fn at_section07_reopen_scan_bin_06_program_e_candidate_diff_prints_stable_report() {
            let result = run_cli_with(
                vec![
                    "--repo-root".to_string(),
                    "/repo".to_string(),
                    "--base".to_string(),
                    "abc".to_string(),
                    "--head".to_string(),
                    "def".to_string(),
                ],
                |_, args| {
                    if args.contains(&"--name-only") {
                        Ok("crates/selene_os/src/ph1j.rs\ncrates/selene_storage/src/repo.rs\n"
                            .to_string())
                    } else if args.contains(&"--unified=0") {
                        Ok("diff --git a/x b/x\n@@ -1 +1 @@\n+certification_target_ref\n".to_string())
                    } else {
                        Err("unexpected git args".to_string())
                    }
                },
            );

            assert_eq!(result.exit_code, 0);
            assert_eq!(
                result.stdout,
                "status=ProgramEReopenCandidate\nmatched_watchlist_files=crates/selene_os/src/ph1j.rs,crates/selene_storage/src/repo.rs\nmatched_symbols=certification_target,certification_target_ref\nexplanation=Program E reopen candidate criteria matched with explicit target-bearing certification evidence plus proof/runtime and transport surface movement\n"
            );
            assert!(result.stderr.is_empty());
        }

        #[test]
        fn at_section07_reopen_scan_bin_07_artifact_trust_and_generic_target_id_only_diff_stays_blocked(
        ) {
            let result = run_cli_with(
                vec![
                    "--repo-root".to_string(),
                    "/repo".to_string(),
                    "--base".to_string(),
                    "abc".to_string(),
                    "--head".to_string(),
                    "def".to_string(),
                ],
                |_, args| {
                    if args.contains(&"--name-only") {
                        Ok("crates/selene_os/src/ph1j.rs\ncrates/selene_storage/src/repo.rs\n"
                            .to_string())
                    } else if args.contains(&"--unified=0") {
                        Ok("diff --git a/x b/x\n@@ -1 +1 @@\n+artifact_trust_entries\n+target_id\n".to_string())
                    } else {
                        Err("unexpected git args".to_string())
                    }
                },
            );

            assert_eq!(result.exit_code, 0);
            assert_eq!(
                result.stdout,
                "status=StillBlocked\nmatched_watchlist_files=crates/selene_os/src/ph1j.rs,crates/selene_storage/src/repo.rs\nmatched_symbols=artifact_trust_entries,target_id\nexplanation=watchlist files changed, but only adjacent artifact-trust, proof-ref, or generic target carriers matched\n"
            );
            assert!(result.stderr.is_empty());
        }

        #[test]
        fn at_section07_reopen_scan_bin_08_unknown_flag_fails_closed() {
            let result = run_cli_with(vec!["--weird".to_string()], |_, _| {
                panic!("git should not run for malformed args")
            });

            assert_eq!(result.exit_code, 64);
            assert_eq!(
                result.stderr,
                format!("unsupported argument --weird\n{USAGE}\n")
            );
            assert!(result.stdout.is_empty());
        }

        #[test]
        fn at_section07_reopen_scan_bin_09_missing_flag_value_fails_closed() {
            let result = run_cli_with(
                vec![
                    "--repo-root".to_string(),
                    "/repo".to_string(),
                    "--base".to_string(),
                    "abc".to_string(),
                    "--head".to_string(),
                ],
                |_, _| panic!("git should not run for malformed args"),
            );

            assert_eq!(result.exit_code, 64);
            assert_eq!(
                result.stderr,
                format!("missing value for --head\n{USAGE}\n")
            );
            assert!(result.stdout.is_empty());
        }

        #[test]
        fn at_section07_reopen_scan_bin_10_git_failure_fails_closed() {
            let result = run_cli_with(
                vec![
                    "--repo-root".to_string(),
                    "/repo".to_string(),
                    "--base".to_string(),
                    "abc".to_string(),
                    "--head".to_string(),
                    "def".to_string(),
                ],
                |_, args| {
                    if args.contains(&"--name-only") {
                        Err("git command failed: synthetic failure".to_string())
                    } else {
                        panic!("patch diff should not run after name-only failure");
                    }
                },
            );

            assert_eq!(result.exit_code, 2);
            assert_eq!(result.stderr, "git command failed: synthetic failure\n");
            assert!(result.stdout.is_empty());
        }
    }
}
