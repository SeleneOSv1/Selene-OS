#![forbid(unsafe_code)]

use selene_os::web_search_plan::enterprise::{
    parse_mode, run_enterprise_pipeline, EnterpriseConstraints, EnterpriseMode, EnterpriseRequest,
};
use selene_os::web_search_plan::merge::{InternalContext, InternalSourceType};
use selene_os::web_search_plan::structured::types::StructuredRow;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

const FIXTURE_NOW_MS: i64 = 1_703_000_000_000;

#[derive(Debug, Clone)]
struct CliArgs {
    fixture_mode: bool,
    mode: EnterpriseMode,
    query: String,
    tier: String,
    jurisdiction: Option<String>,
    as_of_from_ms: Option<i64>,
    as_of_to_ms: Option<i64>,
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{}", message);
            eprintln!("usage: web_search_enterprise_turn [--fixture] --mode <competitive|temporal|risk|merge|report> --query <text> [--tier <low|medium|high>] [--jurisdiction <code>] [--as_of_from <ms>] [--as_of_to <ms>]");
            return ExitCode::from(64);
        }
    };

    let request = if args.fixture_mode {
        match build_fixture_request(&args) {
            Ok(request) => request,
            Err(message) => {
                eprintln!("fixture_build_error={}", message);
                return ExitCode::from(64);
            }
        }
    } else {
        build_live_request(&args)
    };

    match run_enterprise_pipeline(&request, FIXTURE_NOW_MS) {
        Ok(output) => {
            println!("mode={}", output.mode);
            println!("stage_trace={}", output.stage_trace.join(","));
            println!("reason_codes={}", output.reason_codes.join(","));
            println!(
                "output_packets=competitive:{} temporal:{} risk:{} merge:{} report:{} multihop:{}",
                output.competitive_packet.is_some(),
                output.temporal_packet.is_some(),
                output.risk_packet.is_some(),
                output.merge_packet.is_some(),
                output.report_packet.is_some(),
                output.multihop_packet.is_some(),
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("FAIL_CLOSED_REASON={}", error.reason_code);
            eprintln!("FAIL_CLOSED_MESSAGE={}", error.message);
            ExitCode::from(2)
        }
    }
}

fn parse_args() -> Result<CliArgs, String> {
    let mut fixture_mode = false;
    let mut mode: Option<EnterpriseMode> = None;
    let mut query: Option<String> = None;
    let mut tier = "medium".to_string();
    let mut jurisdiction: Option<String> = None;
    let mut as_of_from_ms: Option<i64> = None;
    let mut as_of_to_ms: Option<i64> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--fixture" => fixture_mode = true,
            "--mode" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--mode requires a value".to_string())?;
                mode = Some(parse_mode(value.as_str())?);
            }
            "--query" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--query requires a value".to_string())?;
                if value.trim().is_empty() {
                    return Err("--query cannot be empty".to_string());
                }
                query = Some(value);
            }
            "--tier" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--tier requires a value".to_string())?;
                let normalized = value.trim().to_ascii_lowercase();
                if !matches!(normalized.as_str(), "low" | "medium" | "high") {
                    return Err(format!("unsupported --tier value {}", value));
                }
                tier = normalized;
            }
            "--jurisdiction" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--jurisdiction requires a value".to_string())?;
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    return Err("--jurisdiction cannot be empty".to_string());
                }
                jurisdiction = Some(trimmed.to_string());
            }
            "--as_of_from" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--as_of_from requires a value".to_string())?;
                as_of_from_ms = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| "--as_of_from must be an integer ms timestamp".to_string())?,
                );
            }
            "--as_of_to" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--as_of_to requires a value".to_string())?;
                as_of_to_ms = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| "--as_of_to must be an integer ms timestamp".to_string())?,
                );
            }
            other => return Err(format!("unsupported argument {}", other)),
        }
    }

    let mode = mode.ok_or_else(|| "--mode is required".to_string())?;
    let query = query.ok_or_else(|| "--query is required".to_string())?;

    Ok(CliArgs {
        fixture_mode,
        mode,
        query,
        tier,
        jurisdiction,
        as_of_from_ms,
        as_of_to_ms,
    })
}

fn build_live_request(args: &CliArgs) -> EnterpriseRequest {
    EnterpriseRequest {
        trace_id: "trace-enterprise-cli-live".to_string(),
        query: args.query.clone(),
        mode: args.mode,
        importance_tier: args.tier.clone(),
        created_at_ms: FIXTURE_NOW_MS,
        policy_snapshot_id: "policy-snapshot-default".to_string(),
        jurisdiction: args.jurisdiction.clone(),
        as_of_from_ms: args.as_of_from_ms,
        as_of_to_ms: args.as_of_to_ms,
        constraints: EnterpriseConstraints::default(),
        target_entity: None,
        tool_request_packet: None,
        evidence_packet: None,
        structured_rows: None,
        computation_packet: None,
        internal_context: None,
    }
}

fn build_fixture_request(args: &CliArgs) -> Result<EnterpriseRequest, String> {
    let mut request = EnterpriseRequest {
        trace_id: "trace-enterprise-cli-fixture".to_string(),
        query: args.query.clone(),
        mode: args.mode,
        importance_tier: args.tier.clone(),
        created_at_ms: FIXTURE_NOW_MS,
        policy_snapshot_id: "policy-snapshot-default".to_string(),
        jurisdiction: args.jurisdiction.clone(),
        as_of_from_ms: args.as_of_from_ms,
        as_of_to_ms: args.as_of_to_ms,
        constraints: EnterpriseConstraints::default(),
        target_entity: None,
        tool_request_packet: None,
        evidence_packet: None,
        structured_rows: None,
        computation_packet: None,
        internal_context: None,
    };

    match args.mode {
        EnterpriseMode::Competitive => {
            let fixture = load_json_fixture(&["competitive_fixtures", "pricing_competitors.json"])?;
            request.trace_id = string_field(&fixture, "trace_id")?.to_string();
            request.created_at_ms = int_field(&fixture, "created_at_ms")?;
            request.target_entity = Some(string_field(&fixture, "target_entity")?.to_string());
            request.evidence_packet = Some(value_field(&fixture, "evidence_packet")?.clone());
            request.structured_rows = Some(
                serde_json::from_value(value_field(&fixture, "structured_rows")?.clone()).map_err(
                    |error| format!("failed to parse competitive structured_rows: {}", error),
                )?,
            );
            request.computation_packet = Some(value_field(&fixture, "computation_packet")?.clone());
        }
        EnterpriseMode::Temporal => {
            let temporal = load_json_fixture(&["temporal_fixtures", "missing_timestamps.json"])?;
            let mut rows = parse_rows_fixture("baseline_rows.json")?;
            rows.extend(parse_rows_fixture("compare_rows.json")?);
            request.trace_id = temporal
                .pointer("/evidence_packet/trace_id")
                .and_then(Value::as_str)
                .unwrap_or("trace-temporal-cli-fixture")
                .to_string();
            request.created_at_ms = temporal
                .pointer("/evidence_packet/created_at_ms")
                .and_then(Value::as_i64)
                .unwrap_or(FIXTURE_NOW_MS);
            request.evidence_packet = Some(value_field(&temporal, "evidence_packet")?.clone());
            request.structured_rows = Some(rows);
            request.as_of_from_ms = args.as_of_from_ms;
            request.as_of_to_ms = args.as_of_to_ms;
        }
        EnterpriseMode::Risk => {
            request.evidence_packet = Some(load_json_fixture(&[
                "enterprise_fixtures",
                "enterprise_evidence_packet.json",
            ])?);
            request.computation_packet = Some(load_json_fixture(&[
                "enterprise_fixtures",
                "enterprise_computation_packet.json",
            ])?);
        }
        EnterpriseMode::Merge => {
            request.evidence_packet = Some(load_json_fixture(&[
                "enterprise_fixtures",
                "enterprise_evidence_packet.json",
            ])?);
            request.internal_context = Some(default_internal_context());
        }
        EnterpriseMode::Report => {
            request.evidence_packet = Some(load_json_fixture(&[
                "enterprise_fixtures",
                "enterprise_evidence_packet.json",
            ])?);
            request.computation_packet = Some(load_json_fixture(&[
                "enterprise_fixtures",
                "enterprise_computation_packet.json",
            ])?);
            request.internal_context = Some(default_internal_context());
        }
        _ => {
            return Err(format!(
                "fixture mode supports competitive|temporal|risk|merge|report, got {}",
                args.mode.as_str()
            ));
        }
    }

    Ok(request)
}

fn default_internal_context() -> InternalContext {
    InternalContext {
        prior_summary: "Acme previously reported steady uptime metrics.".to_string(),
        prior_key_points: vec![
            "Status page reports 99.99% uptime SLA.".to_string(),
            "Starter plan costs $10 monthly.".to_string(),
        ],
        prior_timestamp_ms: 1_702_000_000_000,
        internal_source_type: InternalSourceType::PriorReport,
    }
}

fn parse_rows_fixture(name: &str) -> Result<Vec<StructuredRow>, String> {
    let value = load_json_fixture(&["temporal_fixtures", name])?;
    serde_json::from_value(value_field(&value, "rows")?.clone())
        .map_err(|error| format!("failed to parse temporal rows fixture {}: {}", name, error))
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/web_search_plan")
}

fn load_json_fixture(path_parts: &[&str]) -> Result<Value, String> {
    let mut path = fixture_root();
    for part in path_parts {
        path = path.join(part);
    }
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read fixture {}: {}", path.display(), error))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("failed to parse fixture {}: {}", path.display(), error))
}

fn value_field<'a>(value: &'a Value, key: &str) -> Result<&'a Value, String> {
    value
        .get(key)
        .ok_or_else(|| format!("fixture missing field {}", key))
}

fn string_field<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value_field(value, key)?
        .as_str()
        .ok_or_else(|| format!("fixture field {} must be string", key))
}

fn int_field(value: &Value, key: &str) -> Result<i64, String> {
    value_field(value, key)?
        .as_i64()
        .ok_or_else(|| format!("fixture field {} must be integer", key))
}
