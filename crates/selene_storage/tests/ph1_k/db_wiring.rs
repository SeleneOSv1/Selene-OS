#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1feedback::FeedbackEventType;
use selene_kernel_contracts::ph1j::{AuditEngine, AuditEventType, DeviceId, PayloadKey};
use selene_kernel_contracts::ph1k::{
    AdvancedAudioQualityMetrics, Confidence, DegradationClassBundle, DeviceRoute,
    InterruptCandidateConfidenceBand, InterruptDegradationContext, InterruptGateConfidences,
    InterruptRiskContextClass, InterruptSpeechWindowMetrics,
    InterruptSubjectRelationConfidenceBundle, InterruptTimingMarkers, SpeechLikeness,
    VadDecisionConfidenceBand,
};
use selene_kernel_contracts::ph1learn::LearnSignalType;
use selene_kernel_contracts::ph1pae::PaeMode;
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, Ph1kDeviceHealth,
    Ph1kFeedbackCaptureInput, Ph1kFeedbackIssueKind, Ph1kInterruptCandidateExtendedFields,
    Ph1kRuntimeEventKind, StorageError,
};
use selene_storage::repo::{Ph1fFoundationRepo, Ph1kVoiceRuntimeRepo};

fn user(id: &str) -> UserId {
    UserId::new(id).unwrap()
}

fn device(id: &str) -> DeviceId {
    DeviceId::new(id).unwrap()
}

fn seed_identity_device(store: &mut Ph1fStore, user_id: UserId, device_id: DeviceId) {
    store
        .insert_identity_row(IdentityRecord::v1(
            user_id.clone(),
            None,
            None,
            MonotonicTimeNs(1),
            IdentityStatus::Active,
        ))
        .unwrap();

    store
        .insert_device_row(
            DeviceRecord::v1(
                device_id,
                user_id,
                "desktop".to_string(),
                MonotonicTimeNs(1),
                None,
            )
            .unwrap(),
        )
        .unwrap();
}

fn interrupt_extended_fields() -> Ph1kInterruptCandidateExtendedFields {
    Ph1kInterruptCandidateExtendedFields {
        trigger_phrase_id: 99,
        trigger_locale: "en-US".to_string(),
        candidate_confidence_band: InterruptCandidateConfidenceBand::High,
        vad_decision_confidence_band: VadDecisionConfidenceBand::High,
        risk_context_class: InterruptRiskContextClass::Guarded,
        gate_confidences: InterruptGateConfidences {
            vad_confidence: Confidence::new(0.95).unwrap(),
            speech_likeness: SpeechLikeness::new(0.94).unwrap(),
            echo_safe_confidence: Confidence::new(0.96).unwrap(),
            phrase_confidence: Confidence::new(0.95).unwrap(),
            nearfield_confidence: Some(Confidence::new(0.85).unwrap()),
        },
        degradation_context: InterruptDegradationContext {
            capture_degraded: true,
            aec_unstable: false,
            device_changed: false,
            stream_gap_detected: false,
            class_bundle: DegradationClassBundle::from_flags(true, false, false, false),
        },
        quality_metrics: AdvancedAudioQualityMetrics {
            snr_db: 24.0,
            clipping_ratio: 0.02,
            echo_delay_ms: 10.0,
            packet_loss_pct: 0.10,
            double_talk_score: 0.12,
            erle_db: 22.0,
        },
        timing_markers: InterruptTimingMarkers {
            window_start: MonotonicTimeNs(500),
            window_end: MonotonicTimeNs(580),
        },
        speech_window_metrics: InterruptSpeechWindowMetrics {
            voiced_window_ms: 120,
        },
        subject_relation_confidence_bundle: InterruptSubjectRelationConfidenceBundle {
            lexical_confidence: Confidence::new(0.95).unwrap(),
            vad_confidence: Confidence::new(0.95).unwrap(),
            speech_likeness: SpeechLikeness::new(0.94).unwrap(),
            echo_safe_confidence: Confidence::new(0.96).unwrap(),
            nearfield_confidence: Some(Confidence::new(0.85).unwrap()),
            combined_confidence: Confidence::new(0.94).unwrap(),
        },
        interrupt_policy_profile_id: "k_policy_default".to_string(),
        interrupt_tenant_profile_id: "k_tenant_default".to_string(),
        interrupt_locale_tag: "en-US".to_string(),
        adaptive_device_route: DeviceRoute::BuiltIn,
        adaptive_noise_class: "ELEVATED".to_string(),
        adaptive_capture_to_handoff_latency_ms: 110,
        adaptive_timing_jitter_ms: 4.0,
        adaptive_timing_drift_ppm: 2.0,
        adaptive_device_reliability_score: 0.93,
    }
}

fn feedback_capture_input(issue_kind: Ph1kFeedbackIssueKind) -> Ph1kFeedbackCaptureInput {
    Ph1kFeedbackCaptureInput {
        issue_kind,
        candidate_confidence_band: Some(InterruptCandidateConfidenceBand::Medium),
        risk_context_class: Some(InterruptRiskContextClass::Guarded),
        adaptive_device_route: Some(DeviceRoute::Usb),
        adaptive_noise_class: Some("ELEVATED".to_string()),
        capture_degraded: Some(true),
        aec_unstable: Some(false),
        device_changed: Some(true),
        stream_gap_detected: Some(false),
        failover_from_device: Some("mic_a".to_string()),
        failover_to_device: Some("mic_b".to_string()),
    }
}

fn feedback_capture_input_clean(issue_kind: Ph1kFeedbackIssueKind) -> Ph1kFeedbackCaptureInput {
    let mut input = feedback_capture_input(issue_kind);
    input.capture_degraded = Some(false);
    input.aec_unstable = Some(false);
    input.device_changed = Some(false);
    input.stream_gap_detected = Some(false);
    if !matches!(issue_kind, Ph1kFeedbackIssueKind::BadFailoverSelection) {
        input.failover_from_device = None;
        input.failover_to_device = None;
    }
    input
}

#[test]
fn at_k_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let user_a = user("tenant_a:user_1");
    let user_b = user("tenant_b:user_1");
    let device_a = device("tenant_a_device_1");
    let device_b = device("tenant_b_device_1");
    seed_identity_device(&mut s, user_a, device_a.clone());
    seed_identity_device(&mut s, user_b, device_b.clone());

    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(100),
        "tenant_a".to_string(),
        device_a.clone(),
        None,
        Ph1kRuntimeEventKind::StreamRefs,
        Some(1001),
        None,
        Some(501),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        "k-tenant-a".to_string(),
    )
    .unwrap();

    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(101),
        "tenant_b".to_string(),
        device_b.clone(),
        None,
        Ph1kRuntimeEventKind::StreamRefs,
        Some(2001),
        None,
        Some(601),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        "k-tenant-b".to_string(),
    )
    .unwrap();

    let tenant_mismatch = s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(102),
        "tenant_b".to_string(),
        device_a.clone(),
        None,
        Ph1kRuntimeEventKind::TimingStats,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(1.0),
        Some(2.0),
        Some(3.0),
        Some(0),
        Some(0),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        "k-tenant-mismatch".to_string(),
    );
    assert!(matches!(
        tenant_mismatch,
        Err(StorageError::ContractViolation(_))
    ));

    assert!(s.ph1k_runtime_current_row("tenant_a", &device_a).is_some());
    assert!(s.ph1k_runtime_current_row("tenant_b", &device_b).is_some());
    assert!(s.ph1k_runtime_current_row("tenant_b", &device_a).is_none());
}

#[test]
fn at_k_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u, d.clone());

    let ev = s
        .ph1k_runtime_event_commit_row(
            MonotonicTimeNs(200),
            "tenant_a".to_string(),
            d,
            None,
            Ph1kRuntimeEventKind::InterruptCandidate,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(11),
            Some("stop".to_string()),
            Some(ReasonCodeId(0x4B00_1001)),
            None,
            None,
            None,
            None,
            None,
            "k-append-only".to_string(),
        )
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_ph1k_runtime_event_row(ev.event_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_k_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u, d.clone());

    let first = s
        .ph1k_runtime_event_commit_row(
            MonotonicTimeNs(300),
            "tenant_a".to_string(),
            d.clone(),
            None,
            Ph1kRuntimeEventKind::TimingStats,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(1.5),
            Some(2.5),
            Some(3.5),
            Some(1),
            Some(2),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            "k-idem".to_string(),
        )
        .unwrap();

    let second = s
        .ph1k_runtime_event_commit_row(
            MonotonicTimeNs(301),
            "tenant_a".to_string(),
            d.clone(),
            None,
            Ph1kRuntimeEventKind::TimingStats,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(9.9),
            Some(9.9),
            Some(9.9),
            Some(99),
            Some(99),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            "k-idem".to_string(),
        )
        .unwrap();

    assert_eq!(first.event_id, second.event_id);
    assert_eq!(s.ph1k_runtime_event_rows().len(), 1);
    assert_eq!(
        s.ph1k_runtime_current_row("tenant_a", &d)
            .unwrap()
            .jitter_ms,
        Some(1500)
    );
}

#[test]
fn at_k_db_04_current_table_rebuild_from_ledger() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u, d.clone());

    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(400),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::StreamRefs,
        Some(7001),
        Some(7002),
        Some(333),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        "k-rebuild-stream".to_string(),
    )
    .unwrap();
    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(401),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::DeviceState,
        None,
        None,
        None,
        Some("mic_primary".to_string()),
        Some("spk_primary".to_string()),
        Some(Ph1kDeviceHealth::Healthy),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        "k-rebuild-device".to_string(),
    )
    .unwrap();
    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(402),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::TimingStats,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(2.0),
        Some(3.0),
        Some(4.0),
        Some(5),
        Some(6),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        "k-rebuild-timing".to_string(),
    )
    .unwrap();
    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(403),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::DegradationFlags,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(true),
        Some(false),
        Some(true),
        Some(true),
        "k-rebuild-degrade".to_string(),
    )
    .unwrap();
    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(404),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::InterruptCandidate,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(99),
        Some("hold on".to_string()),
        Some(ReasonCodeId(0x4B00_2001)),
        None,
        None,
        None,
        None,
        None,
        "k-rebuild-interrupt".to_string(),
    )
    .unwrap();
    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(405),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::TtsPlaybackActive,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(true),
        None,
        None,
        None,
        None,
        "k-rebuild-tts".to_string(),
    )
    .unwrap();

    let before = s.ph1k_runtime_current_row("tenant_a", &d).unwrap().clone();
    s.rebuild_ph1k_runtime_current_rows();
    let after = s.ph1k_runtime_current_row("tenant_a", &d).unwrap().clone();

    assert_eq!(before, after);
    assert_eq!(after.tts_playback_active, true);
    assert_eq!(after.capture_degraded, true);
    assert_eq!(after.stream_gap_detected, true);
    assert_eq!(after.last_interrupt_phrase.as_deref(), Some("hold on"));
    assert_eq!(
        after.last_event_id,
        s.ph1k_runtime_event_rows().last().unwrap().event_id
    );
}

#[test]
fn at_k_db_05_interrupt_extended_fields_persist_and_project() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u, d.clone());

    let extended = interrupt_extended_fields();
    let row = s
        .ph1k_runtime_event_commit_row_extended(
            MonotonicTimeNs(500),
            "tenant_a".to_string(),
            d.clone(),
            None,
            Ph1kRuntimeEventKind::InterruptCandidate,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(99),
            Some("hold on".to_string()),
            Some(ReasonCodeId(0x4B00_3001)),
            Some(extended.clone()),
            None,
            None,
            None,
            None,
            None,
            "k-interrupt-extended".to_string(),
        )
        .unwrap();

    let saved = row
        .interrupt_extended
        .expect("extended payload must be persisted");
    assert_eq!(
        saved.candidate_confidence_band,
        InterruptCandidateConfidenceBand::High
    );
    assert_eq!(saved.risk_context_class, InterruptRiskContextClass::Guarded);
    assert_eq!(saved.quality_metrics.snr_db, 24.0);
    assert_eq!(saved.adaptive_noise_class, "ELEVATED");

    let current = s.ph1k_runtime_current_row("tenant_a", &d).unwrap();
    assert_eq!(current.last_interrupt_trigger_phrase_id, Some(99));
    assert_eq!(
        current.last_interrupt_candidate_confidence_band,
        Some(InterruptCandidateConfidenceBand::High)
    );
    assert_eq!(
        current.last_interrupt_vad_decision_confidence_band,
        Some(VadDecisionConfidenceBand::High)
    );
    assert_eq!(
        current.last_interrupt_risk_context_class,
        Some(InterruptRiskContextClass::Guarded)
    );
    assert_eq!(current.last_interrupt_snr_db_milli, Some(24_000));
    assert_eq!(current.last_interrupt_clipping_ratio_milli, Some(20));
    assert_eq!(
        current.last_interrupt_adaptive_noise_class.as_deref(),
        Some("ELEVATED")
    );

    let before = current.clone();
    s.rebuild_ph1k_runtime_current_rows();
    let after = s.ph1k_runtime_current_row("tenant_a", &d).unwrap().clone();
    assert_eq!(before, after);
}

#[test]
fn at_k_db_06_interrupt_extended_fields_fail_closed() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u, d.clone());

    let mut invalid_extended = interrupt_extended_fields();
    invalid_extended.trigger_phrase_id = 88;
    let mismatched_extended = s.ph1k_runtime_event_commit_row_extended(
        MonotonicTimeNs(600),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::InterruptCandidate,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(99),
        Some("hold on".to_string()),
        Some(ReasonCodeId(0x4B00_3002)),
        Some(invalid_extended),
        None,
        None,
        None,
        None,
        None,
        "k-interrupt-extended-mismatch".to_string(),
    );
    assert!(matches!(
        mismatched_extended,
        Err(StorageError::ContractViolation(_))
    ));

    let non_interrupt_with_extended = s.ph1k_runtime_event_commit_row_extended(
        MonotonicTimeNs(601),
        "tenant_a".to_string(),
        d,
        None,
        Ph1kRuntimeEventKind::TimingStats,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(1.0),
        Some(2.0),
        Some(3.0),
        Some(1),
        Some(2),
        None,
        None,
        None,
        Some(interrupt_extended_fields()),
        None,
        None,
        None,
        None,
        None,
        "k-non-interrupt-with-extended".to_string(),
    );
    assert!(matches!(
        non_interrupt_with_extended,
        Err(StorageError::ContractViolation(_))
    ));
}

#[test]
fn at_k_db_07_runtime_commits_emit_reason_coded_ph1j_rows() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u, d.clone());

    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(700),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::StreamRefs,
        Some(7001),
        Some(7002),
        Some(7003),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        "k-audit-stream-1".to_string(),
    )
    .unwrap();
    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(701),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::DeviceState,
        None,
        None,
        None,
        Some("mic_a".to_string()),
        Some("spk_a".to_string()),
        Some(Ph1kDeviceHealth::Healthy),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        "k-audit-device-1".to_string(),
    )
    .unwrap();
    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(702),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::TimingStats,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(1.25),
        Some(0.50),
        Some(2.75),
        Some(1),
        Some(2),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        "k-audit-timing-1".to_string(),
    )
    .unwrap();
    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(703),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::DegradationFlags,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(true),
        Some(false),
        Some(true),
        Some(false),
        "k-audit-degrade-1".to_string(),
    )
    .unwrap();
    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(704),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::TtsPlaybackActive,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(true),
        None,
        None,
        None,
        None,
        "k-audit-tts-1".to_string(),
    )
    .unwrap();
    s.ph1k_runtime_event_commit_row_extended(
        MonotonicTimeNs(705),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::InterruptCandidate,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(99),
        Some("hold on".to_string()),
        Some(ReasonCodeId(0x4B00_0010)),
        Some(interrupt_extended_fields()),
        None,
        None,
        None,
        None,
        None,
        "k-audit-interrupt-1".to_string(),
    )
    .unwrap();

    let rows = s
        .audit_events_by_tenant("tenant_a")
        .into_iter()
        .filter(|row| row.engine == AuditEngine::Ph1K)
        .collect::<Vec<_>>();

    assert_eq!(rows.len(), 6);
    for row in rows {
        assert_eq!(row.event_type, AuditEventType::PerceptionSignalEmitted);
        assert!(row.reason_code.0 > 0);
        assert!(row
            .payload_min
            .entries
            .contains_key(&PayloadKey::new("decision").unwrap()));
        assert!(row
            .payload_min
            .entries
            .contains_key(&PayloadKey::new("event_kind").unwrap()));
        assert!(row
            .payload_min
            .entries
            .contains_key(&PayloadKey::new("event_name").unwrap()));
        assert!(row
            .payload_min
            .entries
            .contains_key(&PayloadKey::new("ph1k_event_id").unwrap()));
    }
}

#[test]
fn at_k_db_08_interrupt_extended_audit_payload_includes_step12_keys() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u, d);

    s.ph1k_runtime_event_commit_row_extended(
        MonotonicTimeNs(800),
        "tenant_a".to_string(),
        device("tenant_a_device_1"),
        None,
        Ph1kRuntimeEventKind::InterruptCandidate,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(99),
        Some("hold on".to_string()),
        Some(ReasonCodeId(0x4B00_0010)),
        Some(interrupt_extended_fields()),
        None,
        None,
        None,
        None,
        None,
        "k-audit-interrupt-extended-2".to_string(),
    )
    .unwrap();

    let row = s
        .audit_events_by_tenant("tenant_a")
        .into_iter()
        .find(|ev| ev.engine == AuditEngine::Ph1K)
        .expect("ph1k audit row must exist");
    let entries = &row.payload_min.entries;
    assert!(entries.contains_key(&PayloadKey::new("candidate_confidence_band").unwrap()));
    assert!(entries.contains_key(&PayloadKey::new("risk_context_class").unwrap()));
    assert!(entries.contains_key(&PayloadKey::new("degradation_context").unwrap()));
    assert!(entries.contains_key(&PayloadKey::new("quality_metrics").unwrap()));
    assert!(entries.contains_key(&PayloadKey::new("timing_markers").unwrap()));
    assert!(entries.contains_key(&PayloadKey::new("subject_relation_confidence_bundle").unwrap()));
    assert!(entries.contains_key(&PayloadKey::new("interrupt_profile_refs").unwrap()));
    assert!(entries.contains_key(&PayloadKey::new("adaptive_profile").unwrap()));
    assert!(entries.len() <= 16);
}

#[test]
fn at_k_db_09_feedback_capture_wires_issue_kinds_and_fingerprints() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let scenarios = [
        Ph1kFeedbackIssueKind::FalseInterrupt,
        Ph1kFeedbackIssueKind::MissedInterrupt,
        Ph1kFeedbackIssueKind::WrongDegradationClassification,
        Ph1kFeedbackIssueKind::BadFailoverSelection,
    ];
    for (idx, issue_kind) in scenarios.iter().enumerate() {
        let row = s
            .ph1k_feedback_capture_commit_row(
                MonotonicTimeNs(900 + idx as u64),
                "tenant_a".to_string(),
                selene_kernel_contracts::ph1j::CorrelationId(9901),
                selene_kernel_contracts::ph1j::TurnId(8801 + idx as u64),
                None,
                u.clone(),
                d.clone(),
                feedback_capture_input(*issue_kind),
                format!("k-feedback-{idx}"),
            )
            .expect("feedback capture must commit");

        assert_eq!(row.issue_kind, *issue_kind);
        assert!(row.primary_fingerprint.starts_with("pkf_"));
        assert!(row.secondary_fingerprint.starts_with("skf_"));
        assert!(!row.primary_fingerprint.ends_with("0000000000000000"));
    }

    let rows = s.ph1k_feedback_capture_rows();
    assert_eq!(rows.len(), 4);
    assert_eq!(
        rows[0].feedback_event_type,
        FeedbackEventType::BargeIn,
        "false interrupt must route to BargeIn"
    );
    assert_eq!(rows[0].signal_bucket, LearnSignalType::BargeIn);
    assert_eq!(rows[0].reason_code, ReasonCodeId(0x4B00_0013));
    assert_eq!(rows[1].feedback_event_type, FeedbackEventType::BargeIn);
    assert_eq!(rows[1].signal_bucket, LearnSignalType::BargeIn);
    assert_eq!(rows[1].reason_code, ReasonCodeId(0x4B00_0014));
    assert_eq!(
        rows[2].feedback_event_type,
        FeedbackEventType::UserCorrection
    );
    assert_eq!(rows[2].signal_bucket, LearnSignalType::UserCorrection);
    assert_eq!(rows[2].reason_code, ReasonCodeId(0x4B00_001B));
    assert_eq!(
        rows[3].feedback_event_type,
        FeedbackEventType::DeliverySwitch
    );
    assert_eq!(rows[3].signal_bucket, LearnSignalType::DeliverySwitch);
    assert_eq!(rows[3].reason_code, ReasonCodeId(0x4B00_001C));

    let feedback_rows =
        s.ph1feedback_audit_rows(selene_kernel_contracts::ph1j::CorrelationId(9901));
    assert_eq!(feedback_rows.len(), 4);
    for row in feedback_rows {
        assert!(row
            .payload_min
            .entries
            .contains_key(&PayloadKey::new("feedback_kind").unwrap()));
        assert!(row
            .payload_min
            .entries
            .contains_key(&PayloadKey::new("cluster_primary_fingerprint").unwrap()));
        assert!(row
            .payload_min
            .entries
            .contains_key(&PayloadKey::new("cluster_secondary_fingerprint").unwrap()));
    }
}

#[test]
fn at_k_db_10_feedback_capture_bad_failover_requires_device_pair() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let mut input = feedback_capture_input(Ph1kFeedbackIssueKind::BadFailoverSelection);
    input.failover_to_device = None;

    let err = s
        .ph1k_feedback_capture_commit_row(
            MonotonicTimeNs(1001),
            "tenant_a".to_string(),
            selene_kernel_contracts::ph1j::CorrelationId(9950),
            selene_kernel_contracts::ph1j::TurnId(8850),
            None,
            u,
            d,
            input,
            "k-feedback-bad-failover-missing-pair".to_string(),
        )
        .expect_err("missing failover target must fail closed");

    match err {
        StorageError::ContractViolation(ContractViolation::InvalidValue { field, .. }) => {
            assert_eq!(field, "ph1k_feedback_capture.failover_pair");
        }
        other => panic!("expected ContractViolation::InvalidValue, got {other:?}"),
    }
}

#[test]
fn at_k_db_11_feedback_capture_routes_to_learn_and_governed_pae_ladder() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let correlation_id = selene_kernel_contracts::ph1j::CorrelationId(9981);

    let row_1 = s
        .ph1k_feedback_capture_commit_row(
            MonotonicTimeNs(1101),
            "tenant_a".to_string(),
            correlation_id,
            selene_kernel_contracts::ph1j::TurnId(8901),
            None,
            u.clone(),
            d.clone(),
            feedback_capture_input_clean(Ph1kFeedbackIssueKind::MissedInterrupt),
            "k-step14-ladder-1".to_string(),
        )
        .expect("step14 capture row 1 must commit");
    assert_eq!(row_1.learn_bundle_id, 1);
    assert_eq!(row_1.pae_mode_from, PaeMode::Shadow);
    assert_eq!(row_1.pae_mode_to, PaeMode::Assist);
    assert_eq!(row_1.pae_decision_action, "PROMOTE");
    assert!(!row_1.pae_rollback_triggered);

    let row_2 = s
        .ph1k_feedback_capture_commit_row(
            MonotonicTimeNs(1102),
            "tenant_a".to_string(),
            correlation_id,
            selene_kernel_contracts::ph1j::TurnId(8902),
            None,
            u.clone(),
            d.clone(),
            feedback_capture_input_clean(Ph1kFeedbackIssueKind::MissedInterrupt),
            "k-step14-ladder-2".to_string(),
        )
        .expect("step14 capture row 2 must commit");
    assert_eq!(row_2.learn_bundle_id, 2);
    assert_eq!(row_2.pae_mode_from, PaeMode::Assist);
    assert_eq!(row_2.pae_mode_to, PaeMode::Lead);
    assert_eq!(row_2.pae_decision_action, "PROMOTE");
    assert!(!row_2.pae_rollback_triggered);

    let row_3 = s
        .ph1k_feedback_capture_commit_row(
            MonotonicTimeNs(1103),
            "tenant_a".to_string(),
            correlation_id,
            selene_kernel_contracts::ph1j::TurnId(8903),
            None,
            u.clone(),
            d.clone(),
            feedback_capture_input_clean(Ph1kFeedbackIssueKind::FalseInterrupt),
            "k-step14-ladder-3".to_string(),
        )
        .expect("step14 capture row 3 must commit");
    assert_eq!(row_3.learn_bundle_id, 3);
    assert_eq!(row_3.pae_mode_from, PaeMode::Lead);
    assert_eq!(row_3.pae_mode_to, PaeMode::Assist);
    assert_eq!(row_3.pae_decision_action, "DEMOTE");
    assert!(row_3.pae_rollback_triggered);
    assert!(row_3.pae_false_interrupt_regression_triggered);
    assert!(!row_3.pae_quality_regression_triggered);
    assert!(row_3.false_interrupt_rate_milli_per_hour > 300);

    let mut degradation_input =
        feedback_capture_input(Ph1kFeedbackIssueKind::WrongDegradationClassification);
    degradation_input.device_changed = Some(false);
    let row_4 = s
        .ph1k_feedback_capture_commit_row(
            MonotonicTimeNs(1104),
            "tenant_a".to_string(),
            correlation_id,
            selene_kernel_contracts::ph1j::TurnId(8904),
            None,
            u,
            d,
            degradation_input,
            "k-step14-ladder-4".to_string(),
        )
        .expect("step14 capture row 4 must commit");
    assert_eq!(row_4.learn_bundle_id, 4);
    assert_eq!(row_4.pae_mode_from, PaeMode::Assist);
    assert_eq!(row_4.pae_mode_to, PaeMode::Shadow);
    assert_eq!(row_4.pae_decision_action, "DEMOTE");
    assert!(row_4.pae_rollback_triggered);
    assert!(row_4.pae_quality_regression_triggered);

    let learn_rows = s.ph1feedback_learn_signal_bundle_rows(correlation_id);
    assert_eq!(learn_rows.len(), 4);
    assert_eq!(learn_rows[0].learn_signal_type, LearnSignalType::BargeIn);
    assert_eq!(
        learn_rows[3].learn_signal_type,
        LearnSignalType::UserCorrection
    );

    let pae_audit_rows: Vec<_> = s
        .audit_events_by_tenant("tenant_a")
        .into_iter()
        .filter(|row| {
            matches!(
                &row.engine,
                AuditEngine::Other(engine_id) if engine_id == "PH1.PAE"
            )
        })
        .collect();
    assert_eq!(pae_audit_rows.len(), 4);
    let route_chain = PayloadKey::new("route_chain").unwrap();
    let decision_action = PayloadKey::new("decision_action").unwrap();
    let mode_from = PayloadKey::new("mode_from").unwrap();
    let mode_to = PayloadKey::new("mode_to").unwrap();
    let rollback = PayloadKey::new("rollback_triggered").unwrap();
    assert_eq!(
        pae_audit_rows[0]
            .payload_min
            .entries
            .get(&route_chain)
            .unwrap()
            .as_str(),
        "PH1.FEEDBACK->PH1.LEARN->PH1.PAE"
    );
    assert_eq!(
        pae_audit_rows[0]
            .payload_min
            .entries
            .get(&decision_action)
            .unwrap()
            .as_str(),
        "PROMOTE"
    );
    assert_eq!(
        pae_audit_rows[2]
            .payload_min
            .entries
            .get(&decision_action)
            .unwrap()
            .as_str(),
        "DEMOTE"
    );
    assert_eq!(
        pae_audit_rows[2]
            .payload_min
            .entries
            .get(&mode_from)
            .unwrap()
            .as_str(),
        "LEAD"
    );
    assert_eq!(
        pae_audit_rows[2]
            .payload_min
            .entries
            .get(&mode_to)
            .unwrap()
            .as_str(),
        "ASSIST"
    );
    assert_eq!(
        pae_audit_rows[2]
            .payload_min
            .entries
            .get(&rollback)
            .unwrap()
            .as_str(),
        "1"
    );
}
