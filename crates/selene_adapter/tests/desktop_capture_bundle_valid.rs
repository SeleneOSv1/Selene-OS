#![forbid(unsafe_code)]

use selene_adapter::desktop_mic_producer::synthetic_capture_ref_for_tests;
use selene_adapter::{validate_voice_turn_capture_bundle_for_live_path, VoiceTurnAdapterRequest};

fn base_request_with_capture() -> VoiceTurnAdapterRequest {
    VoiceTurnAdapterRequest {
        correlation_id: 9001,
        turn_id: 9001,
        device_turn_sequence: None,
        app_platform: "DESKTOP".to_string(),
        platform_version: None,
        device_class: None,
        runtime_client_version: None,
        hardware_capability_profile: None,
        network_profile: None,
        claimed_capabilities: None,
        integrity_status: None,
        attestation_ref: None,
        trigger: "EXPLICIT".to_string(),
        actor_user_id: "tenant_1:test_actor".to_string(),
        tenant_id: Some("tenant_1".to_string()),
        device_id: Some("desktop_test_device_1".to_string()),
        now_ns: Some(2_000_000_000),
        thread_key: None,
        project_id: None,
        pinned_context_refs: None,
        thread_policy_flags: None,
        user_text_partial: None,
        user_text_final: None,
        selene_text_partial: None,
        selene_text_final: None,
        audio_capture_ref: Some(synthetic_capture_ref_for_tests(2_000_000_000)),
        visual_input_ref: None,
    }
}

#[test]
fn desktop_capture_bundle_builder_produces_live_valid_ref() {
    let request = base_request_with_capture();
    let result = validate_voice_turn_capture_bundle_for_live_path(&request);
    assert!(
        result.is_ok(),
        "expected synthetic desktop capture ref to validate, got: {:?}",
        result
    );
}

#[test]
fn desktop_capture_bundle_fails_closed_when_required_field_missing() {
    let mut request = base_request_with_capture();
    request
        .audio_capture_ref
        .as_mut()
        .expect("capture ref should be present")
        .selected_mic = None;

    let err = validate_voice_turn_capture_bundle_for_live_path(&request)
        .expect_err("missing selected_mic must fail-closed");
    assert!(
        err.contains("ph1k live capture missing selected_mic"),
        "unexpected error: {err}"
    );
}
