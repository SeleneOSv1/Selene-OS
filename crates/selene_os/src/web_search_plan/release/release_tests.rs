#![forbid(unsafe_code)]

use crate::web_search_plan::release::evidence_pack::{
    generate_release_evidence_pack, GenerateReleaseEvidenceConfig,
};
use crate::web_search_plan::{contract_hash::sha256_hex, registry_loader::read_text};
use std::fs;
use std::path::PathBuf;

fn temp_dir(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("selene_run30_release_tests_{}_{}", name, std::process::id()));
    dir
}

fn write_release_lock_results(path: &PathBuf, head: &str) {
    let manifest_hash = {
        let text = read_text("CONTRACT_HASH_MANIFEST.json")
            .expect("contract hash manifest text should load in tests");
        sha256_hex(text.as_bytes())
    };
    let text = format!(
        "HEAD_COMMIT\t{head}\nCONTRACT_HASH_MANIFEST_HASH\t{manifest_hash}\nGATE\tscripts/web_search_plan/check_contracts.sh\tPASS\nGATE\tscripts/web_search_plan/check_replay_harness.sh\tPASS\nGATE\tcargo test -p selene_os web_search_plan::tests --quiet\tPASS\nOVERALL\tPASS\n"
    );
    fs::write(path, text).expect("release lock fixture write should succeed");
}

fn write_slo_lock_results(path: &PathBuf) {
    let text = "SLO\tcitation_coverage\tPASS\trequired=1.0_answer_cases\nSLO\trefusal_correctness\tPASS\trequired=all_refusal_cases_pass\nSLO\tfreshness_compliance\tPASS\trequired=stale_refusal_enforced\nSLO\tdeterminism_replay\tPASS\trequired=replay_snapshot_match\nOVERALL\tPASS\n";
    fs::write(path, text).expect("slo lock fixture write should succeed");
}

#[test]
fn test_release_evidence_pack_generation_is_deterministic() {
    let root = temp_dir("deterministic");
    let output_dir = root.join("release_evidence");
    fs::create_dir_all(&output_dir).expect("temp output dir creation should succeed");
    let release_lock_path = root.join("release_lock.tsv");
    let slo_lock_path = root.join("slo_lock.tsv");
    let head = "1234567890abcdef";

    write_release_lock_results(&release_lock_path, head);
    write_slo_lock_results(&slo_lock_path);

    let config = GenerateReleaseEvidenceConfig {
        head_commit: head.to_string(),
        branch: "main".to_string(),
        run30_timestamp_utc: "2026-03-04T12:00:00Z".to_string(),
        date_tag: "20260304T120000Z".to_string(),
        release_lock_results_path: release_lock_path.clone(),
        slo_lock_results_path: slo_lock_path.clone(),
        output_dir: output_dir.clone(),
    };

    let first_path =
        generate_release_evidence_pack(&config).expect("first evidence pack generation should pass");
    let first_content = fs::read_to_string(&first_path).expect("first evidence file should read");

    let second_path = generate_release_evidence_pack(&config)
        .expect("second evidence pack generation should pass");
    let second_content =
        fs::read_to_string(&second_path).expect("second evidence file should read");

    assert_eq!(first_path, second_path);
    assert_eq!(first_content, second_content);
    assert!(first_content.contains("\"head_commit\": \"1234567890abcdef\""));
    assert!(first_content.contains("\"branch\": \"main\""));
}

#[test]
fn test_release_evidence_pack_fails_on_red_release_lock() {
    let root = temp_dir("red");
    let output_dir = root.join("release_evidence");
    fs::create_dir_all(&output_dir).expect("temp output dir creation should succeed");
    let release_lock_path = root.join("release_lock.tsv");
    let slo_lock_path = root.join("slo_lock.tsv");
    fs::write(
        &release_lock_path,
        "HEAD_COMMIT\tdeadbeef\nCONTRACT_HASH_MANIFEST_HASH\tabc\nGATE\tscripts/web_search_plan/check_contracts.sh\tFAIL\nOVERALL\tFAIL\n",
    )
    .expect("release lock fixture write should succeed");
    write_slo_lock_results(&slo_lock_path);

    let config = GenerateReleaseEvidenceConfig {
        head_commit: "deadbeef".to_string(),
        branch: "main".to_string(),
        run30_timestamp_utc: "2026-03-04T12:00:00Z".to_string(),
        date_tag: "20260304T120000Z".to_string(),
        release_lock_results_path: release_lock_path,
        slo_lock_results_path: slo_lock_path,
        output_dir,
    };

    let err = generate_release_evidence_pack(&config)
        .expect_err("release evidence generation must fail on red release lock");
    assert!(err.contains("release lock results are not PASS"));
}

#[test]
fn test_release_evidence_pack_fails_on_head_mismatch() {
    let root = temp_dir("head_mismatch");
    let output_dir = root.join("release_evidence");
    fs::create_dir_all(&output_dir).expect("temp output dir creation should succeed");
    let release_lock_path = root.join("release_lock.tsv");
    let slo_lock_path = root.join("slo_lock.tsv");
    write_release_lock_results(&release_lock_path, "aaaabbbb");
    write_slo_lock_results(&slo_lock_path);

    let config = GenerateReleaseEvidenceConfig {
        head_commit: "ccccdddd".to_string(),
        branch: "main".to_string(),
        run30_timestamp_utc: "2026-03-04T12:00:00Z".to_string(),
        date_tag: "20260304T120000Z".to_string(),
        release_lock_results_path: release_lock_path,
        slo_lock_results_path: slo_lock_path,
        output_dir,
    };

    let err = generate_release_evidence_pack(&config)
        .expect_err("release evidence generation must fail on head mismatch");
    assert!(err.contains("release lock head mismatch"));
}
