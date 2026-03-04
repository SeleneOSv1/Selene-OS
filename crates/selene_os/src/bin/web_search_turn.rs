#![forbid(unsafe_code)]

use selene_os::web_search_plan::runtime::execute_web_search_turn;
use serde_json::json;
use std::env;
use std::process::ExitCode;

const FIXED_CREATED_AT_MS: i64 = 1_700_000_000_000;

fn main() -> ExitCode {
    let (fixture_mode, query) = match parse_args() {
        Ok(values) => values,
        Err(message) => {
            eprintln!("{}", message);
            eprintln!("usage: web_search_turn [--fixture] <query>");
            return ExitCode::from(64);
        }
    };

    let trace_id = if fixture_mode {
        "trace-web-search-turn-fixture"
    } else {
        "trace-web-search-turn-live"
    };
    let tool_mode = if fixture_mode { "structured" } else { "web" };

    let turn_input_packet = json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.OS",
        "intended_consumers": ["PH1.A", "PH1.B"],
        "created_at_ms": FIXED_CREATED_AT_MS,
        "trace_id": trace_id,
        "transcript": query,
        "identity_scope": "default",
        "language": "en",
        "session_id": "cli-web-search-turn"
    });

    let search_assist_packet = json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.A",
        "intended_consumers": ["PH1.B", "PH1.C"],
        "created_at_ms": FIXED_CREATED_AT_MS,
        "trace_id": trace_id,
        "intent_class": "info_request",
        "search_required": true,
        "confidence": 1.0,
        "missing_fields": [],
        "risk_flags": []
    });

    let tool_request_packet = json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.B",
        "intended_consumers": ["PH1.E"],
        "created_at_ms": FIXED_CREATED_AT_MS,
        "trace_id": trace_id,
        "mode": tool_mode,
        "query": query,
        "importance_tier": "medium",
        "budgets": {
            "max_queries": 1,
            "max_results": 3
        }
    });

    match execute_web_search_turn(
        turn_input_packet,
        search_assist_packet,
        tool_request_packet,
        "policy-v1".to_string(),
    ) {
        Ok((_evidence, _synthesis, write_packet, _audit)) => {
            let formatted_text = write_packet
                .get("formatted_text")
                .and_then(|value| value.as_str())
                .unwrap_or("<missing formatted_text>");
            println!("{}", formatted_text);
            ExitCode::SUCCESS
        }
        Err(reason_code) => {
            eprintln!("FAIL_CLOSED_REASON={}", reason_code);
            ExitCode::from(2)
        }
    }
}

fn parse_args() -> Result<(bool, String), String> {
    let mut fixture_mode = false;
    let mut query_parts: Vec<String> = Vec::new();

    for arg in env::args().skip(1) {
        if arg == "--fixture" {
            fixture_mode = true;
            continue;
        }
        query_parts.push(arg);
    }

    if query_parts.is_empty() {
        return Err("query argument is required".to_string());
    }

    Ok((fixture_mode, query_parts.join(" ")))
}
