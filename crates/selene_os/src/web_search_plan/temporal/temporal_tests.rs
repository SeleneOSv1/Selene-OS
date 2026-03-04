#![forbid(unsafe_code)]

use crate::web_search_plan::packet_validator::{validate_packet, validate_packet_schema_registry};
use crate::web_search_plan::registry_loader::load_packet_schema_registry;
use crate::web_search_plan::structured::types::StructuredRow;
use crate::web_search_plan::temporal::asof::{
    filter_rows_for_window, resolve_asof_windows, AsOfWindow, AsOfWindowInput,
    MissingTimestampPolicy,
};
use crate::web_search_plan::temporal::diff::build_changes;
use crate::web_search_plan::temporal::temporal_packet::{
    build_temporal_comparison_packet, TemporalRequest,
};
use crate::web_search_plan::temporal::timeline::build_timeline_events;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
struct RowsFixture {
    rows: Vec<StructuredRow>,
}

#[derive(Debug, Clone, Deserialize)]
struct MissingTimestampFixture {
    evidence_packet: Value,
    rows: Vec<StructuredRow>,
    windows: TemporalWindowsFixture,
}

#[derive(Debug, Clone, Deserialize)]
struct TemporalWindowsFixture {
    baseline_from_ms: i64,
    baseline_to_ms: i64,
    compare_from_ms: i64,
    compare_to_ms: i64,
}

#[derive(Debug, Clone, Deserialize)]
struct ExpectedChangesFixture {
    expected_change_types: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ExpectedTimelineFixture {
    expected_order: Vec<ExpectedTimelineItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ExpectedTimelineItem {
    entity: String,
    attribute: String,
    as_of_ms: i64,
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/web_search_plan/temporal_fixtures")
}

fn load_fixture<T: for<'de> Deserialize<'de>>(name: &str) -> T {
    let text = fs::read_to_string(fixture_dir().join(name)).expect("fixture should load");
    serde_json::from_str::<T>(&text).expect("fixture should parse")
}

#[test]
fn test_t1_as_of_filtering_deterministic() {
    let baseline: RowsFixture = load_fixture("baseline_rows.json");
    let first = filter_rows_for_window(
        baseline.rows.as_slice(),
        AsOfWindow {
            from_ms: 1_700_000_000_000,
            to_ms: 1_700_300_000_000,
        },
        MissingTimestampPolicy::Exclude,
    );
    let second = filter_rows_for_window(
        baseline.rows.as_slice(),
        AsOfWindow {
            from_ms: 1_700_000_000_000,
            to_ms: 1_700_300_000_000,
        },
        MissingTimestampPolicy::Exclude,
    );
    assert_eq!(first, second, "as-of filtering must be deterministic");

    let keys = first
        .rows
        .iter()
        .map(|row| format!("{}|{}", row.entity, row.attribute))
        .collect::<Vec<String>>();
    assert_eq!(
        keys,
        vec![
            "Acme|Churn".to_string(),
            "Acme|Cost".to_string(),
            "Acme|Revenue".to_string(),
            "Acme|Users".to_string()
        ]
    );
    assert_eq!(first.excluded_missing_timestamp_count, 1);
}

#[test]
fn test_t2_timeline_ordering_deterministic() {
    let baseline: RowsFixture = load_fixture("baseline_rows.json");
    let compare: RowsFixture = load_fixture("compare_rows.json");
    let expected: ExpectedTimelineFixture = load_fixture("expected_timeline.json");
    let missing: MissingTimestampFixture = load_fixture("missing_timestamps.json");

    let rows = baseline
        .rows
        .iter()
        .cloned()
        .chain(compare.rows.iter().cloned())
        .collect::<Vec<StructuredRow>>();

    let first = build_timeline_events(rows.as_slice(), &missing.evidence_packet, false);
    let second = build_timeline_events(rows.as_slice(), &missing.evidence_packet, false);
    assert_eq!(first, second, "timeline events should be deterministic");

    let actual = first
        .iter()
        .map(|entry| ExpectedTimelineItem {
            entity: entry.entity.clone(),
            attribute: entry.attribute.clone(),
            as_of_ms: entry.as_of_ms,
        })
        .collect::<Vec<ExpectedTimelineItem>>();
    assert_eq!(actual, expected.expected_order);
}

#[test]
fn test_t3_diff_detection_is_correct() {
    let baseline: RowsFixture = load_fixture("baseline_rows.json");
    let compare: RowsFixture = load_fixture("compare_rows.json");
    let expected: ExpectedChangesFixture = load_fixture("expected_changes.json");

    let baseline_filtered = baseline
        .rows
        .into_iter()
        .filter(|row| row.as_of_ms == Some(1_700_100_000_000))
        .collect::<Vec<StructuredRow>>();
    let compare_filtered = compare
        .rows
        .into_iter()
        .filter(|row| row.as_of_ms == Some(1_701_100_000_000))
        .collect::<Vec<StructuredRow>>();

    let diff = build_changes(&baseline_filtered, &compare_filtered);
    let actual = diff
        .changes
        .into_iter()
        .map(|change| (change.key, change.change_type.as_str().to_string()))
        .collect::<BTreeMap<String, String>>();
    assert_eq!(actual, expected.expected_change_types);
}

#[test]
fn test_t4_delta_only_computed_when_comparable() {
    let mixed: RowsFixture = load_fixture("mixed_units.json");
    let baseline = mixed.rows[0..1].to_vec();
    let compare = mixed.rows[1..2].to_vec();
    let diff = build_changes(&baseline, &compare);

    let price = diff
        .changes
        .iter()
        .find(|change| change.key.starts_with("acme|price"))
        .expect("price change should exist");
    assert_eq!(price.change_type.as_str(), "modified");
    assert!(price.delta_value.is_none(), "delta should be omitted on mismatch");
}

#[test]
fn test_t5_mixed_units_handled_deterministically() {
    let mixed: RowsFixture = load_fixture("mixed_units.json");
    let baseline = mixed.rows[0..1].to_vec();
    let compare = mixed.rows[1..2].to_vec();
    let first = build_changes(&baseline, &compare);
    let second = build_changes(&baseline, &compare);
    assert_eq!(first, second, "mixed-unit diff must be deterministic");
    assert_eq!(first.unit_mismatch_count, 1);
    assert!(first.reason_codes.iter().any(|code| code == "policy_violation"));
}

#[test]
fn test_t6_missing_timestamps_handled_deterministically() {
    let fixture: MissingTimestampFixture = load_fixture("missing_timestamps.json");
    let request = TemporalRequest {
        trace_id: "trace-temporal-t6".to_string(),
        created_at_ms: 1_702_000_000_000,
        intended_consumers: vec!["PH1.D".to_string()],
        now_ms: 1_702_000_000_000,
        baseline_from_ms: Some(fixture.windows.baseline_from_ms),
        baseline_to_ms: Some(fixture.windows.baseline_to_ms),
        compare_from_ms: Some(fixture.windows.compare_from_ms),
        compare_to_ms: Some(fixture.windows.compare_to_ms),
        allow_default_windows: false,
        policy_snapshot_id: "policy-snapshot-default".to_string(),
    };

    let first = build_temporal_comparison_packet(&request, &fixture.evidence_packet, &fixture.rows)
        .expect("temporal build should pass");
    let second =
        build_temporal_comparison_packet(&request, &fixture.evidence_packet, &fixture.rows)
            .expect("temporal build should pass");
    assert_eq!(first, second, "temporal build must be deterministic");
    assert!(
        first
            .packet
            .uncertainty_flags
            .iter()
            .any(|flag| flag == "missing_timestamps_excluded")
    );
}

#[test]
fn test_t7_output_packet_validates_against_schema() {
    let baseline: RowsFixture = load_fixture("baseline_rows.json");
    let compare: RowsFixture = load_fixture("compare_rows.json");
    let evidence: MissingTimestampFixture = load_fixture("missing_timestamps.json");

    let rows = baseline
        .rows
        .iter()
        .cloned()
        .chain(compare.rows.iter().cloned())
        .collect::<Vec<StructuredRow>>();

    let request = TemporalRequest {
        trace_id: "trace-temporal-t7".to_string(),
        created_at_ms: 1_702_000_000_000,
        intended_consumers: vec!["PH1.D".to_string(), "PH1.WRITE".to_string()],
        now_ms: 1_702_000_000_000,
        baseline_from_ms: Some(1_700_000_000_000),
        baseline_to_ms: Some(1_700_900_000_000),
        compare_from_ms: Some(1_701_000_000_000),
        compare_to_ms: Some(1_701_900_000_000),
        allow_default_windows: false,
        policy_snapshot_id: "policy-snapshot-default".to_string(),
    };
    let output = build_temporal_comparison_packet(&request, &evidence.evidence_packet, &rows)
        .expect("temporal packet should build");

    let registry = load_packet_schema_registry().expect("packet schema registry should load");
    validate_packet_schema_registry(&registry).expect("packet schema should validate");
    let packet_value = serde_json::to_value(&output.packet).expect("packet serialize");
    validate_packet("TemporalComparisonPacket", &packet_value, &registry)
        .expect("comparison packet should validate");
}

#[test]
fn test_resolve_asof_defaults_when_enabled() {
    let first = resolve_asof_windows(&AsOfWindowInput {
        baseline_from_ms: None,
        baseline_to_ms: None,
        compare_from_ms: None,
        compare_to_ms: None,
        now_ms: 1_702_000_000_000,
        allow_default_windows: true,
        default_window_ms: 86_400_000,
    })
    .expect("default windows should resolve");
    let second = resolve_asof_windows(&AsOfWindowInput {
        baseline_from_ms: None,
        baseline_to_ms: None,
        compare_from_ms: None,
        compare_to_ms: None,
        now_ms: 1_702_000_000_000,
        allow_default_windows: true,
        default_window_ms: 86_400_000,
    })
    .expect("default windows should resolve");
    assert_eq!(first, second);
    assert!(first.used_defaults);
}
