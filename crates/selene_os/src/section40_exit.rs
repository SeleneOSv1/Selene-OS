#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use selene_kernel_contracts::ph1j::{AuditEngine, CorrelationId};
use selene_kernel_contracts::ContractViolation;
use selene_storage::ph1f::Ph1fStore;

use crate::ph1_voice_id::VoiceIdContractMigrationStage;
use crate::ph1builder::BuilderPromptRateKpis;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceIdCohortProductionSnapshot {
    pub cohort_key: String,
    pub sample_count: u32,
    pub tar_bp: u16,
    pub frr_bp: u16,
    pub far_bp: u16,
    pub latency_p95_ms: u16,
    pub latency_p99_ms: u16,
}

impl VoiceIdCohortProductionSnapshot {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        cohort_key: String,
        sample_count: u32,
        tar_bp: u16,
        frr_bp: u16,
        far_bp: u16,
        latency_p95_ms: u16,
        latency_p99_ms: u16,
    ) -> Result<Self, ContractViolation> {
        if cohort_key.trim().is_empty() || cohort_key.len() > 64 || !cohort_key.is_ascii() {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id_cohort_production_snapshot.cohort_key",
                reason: "must be non-empty ASCII and <= 64 chars",
            });
        }
        if sample_count == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id_cohort_production_snapshot.sample_count",
                reason: "must be > 0",
            });
        }
        for (field, value) in [
            ("voice_id_cohort_production_snapshot.tar_bp", tar_bp),
            ("voice_id_cohort_production_snapshot.frr_bp", frr_bp),
            ("voice_id_cohort_production_snapshot.far_bp", far_bp),
        ] {
            if value > 10_000 {
                return Err(ContractViolation::InvalidValue {
                    field,
                    reason: "must be <= 10000",
                });
            }
        }
        Ok(Self {
            cohort_key,
            sample_count,
            tar_bp,
            frr_bp,
            far_bp,
            latency_p95_ms,
            latency_p99_ms,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoiceIdProductionTargets {
    pub min_tar_bp: u16,
    pub max_frr_bp: u16,
    pub max_far_bp: u16,
    pub max_latency_p95_ms: u16,
    pub max_latency_p99_ms: u16,
    pub min_samples_per_cohort: u32,
}

impl VoiceIdProductionTargets {
    pub const fn strict_v1() -> Self {
        Self {
            min_tar_bp: 9_900,
            max_frr_bp: 100,
            max_far_bp: 10,
            max_latency_p95_ms: 60,
            max_latency_p99_ms: 120,
            min_samples_per_cohort: 200,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceIdProductionGateReport {
    pub passed: bool,
    pub failing_cohorts: Vec<String>,
    pub findings: Vec<String>,
}

pub fn evaluate_voice_id_production_targets(
    snapshots: &[VoiceIdCohortProductionSnapshot],
    targets: VoiceIdProductionTargets,
) -> Result<VoiceIdProductionGateReport, ContractViolation> {
    if snapshots.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field: "voice_id_production_gate.snapshots",
            reason: "must contain at least one cohort snapshot",
        });
    }

    let mut seen = BTreeSet::new();
    let mut failing = Vec::new();
    let mut findings = Vec::new();

    for cohort in snapshots {
        if !seen.insert(cohort.cohort_key.as_str()) {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id_production_gate.snapshots.cohort_key",
                reason: "cohort keys must be unique",
            });
        }
        let mut failed_here = false;
        if cohort.sample_count < targets.min_samples_per_cohort {
            failed_here = true;
            findings.push(format!(
                "{} sample_count {} < {}",
                cohort.cohort_key, cohort.sample_count, targets.min_samples_per_cohort
            ));
        }
        if cohort.tar_bp < targets.min_tar_bp {
            failed_here = true;
            findings.push(format!(
                "{} tar_bp {} < {}",
                cohort.cohort_key, cohort.tar_bp, targets.min_tar_bp
            ));
        }
        if cohort.frr_bp > targets.max_frr_bp {
            failed_here = true;
            findings.push(format!(
                "{} frr_bp {} > {}",
                cohort.cohort_key, cohort.frr_bp, targets.max_frr_bp
            ));
        }
        if cohort.far_bp > targets.max_far_bp {
            failed_here = true;
            findings.push(format!(
                "{} far_bp {} > {}",
                cohort.cohort_key, cohort.far_bp, targets.max_far_bp
            ));
        }
        if cohort.latency_p95_ms > targets.max_latency_p95_ms {
            failed_here = true;
            findings.push(format!(
                "{} latency_p95_ms {} > {}",
                cohort.cohort_key, cohort.latency_p95_ms, targets.max_latency_p95_ms
            ));
        }
        if cohort.latency_p99_ms > targets.max_latency_p99_ms {
            failed_here = true;
            findings.push(format!(
                "{} latency_p99_ms {} > {}",
                cohort.cohort_key, cohort.latency_p99_ms, targets.max_latency_p99_ms
            ));
        }
        if failed_here {
            failing.push(cohort.cohort_key.clone());
        }
    }

    Ok(VoiceIdProductionGateReport {
        passed: failing.is_empty(),
        failing_cohorts: failing,
        findings,
    })
}

pub const SECTION40_OWNER_ASSIGNMENTS: [(&str, &str); 7] = [
    ("identity_decision", "PH1.VOICE.ID"),
    ("enrollment_gate", "PH1.ONB"),
    ("signal_intake", "PH1.FEEDBACK"),
    ("artifact_build", "PH1.LEARN"),
    ("activation_authority", "PH1.GOV"),
    ("rollout_execution", "PH1.BUILDER"),
    ("step_up_authority", "PH1.ACCESS/CAPREQ"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerAmbiguityGateReport {
    pub passed: bool,
    pub findings: Vec<String>,
}

pub fn evaluate_section40_owner_matrix(assignments: &[(&str, &str)]) -> OwnerAmbiguityGateReport {
    let mut findings = Vec::new();
    let mut seen_capabilities = BTreeSet::new();

    let expected = SECTION40_OWNER_ASSIGNMENTS
        .iter()
        .copied()
        .collect::<Vec<_>>();

    for (capability, owner) in assignments {
        if capability.trim().is_empty() || owner.trim().is_empty() {
            findings.push("owner matrix contains empty capability/owner".to_string());
        }
        if !seen_capabilities.insert(*capability) {
            findings.push(format!(
                "capability {} has multiple owner declarations",
                capability
            ));
        }
    }

    for (capability, expected_owner) in &expected {
        let found = assignments
            .iter()
            .find(|(cap, _)| cap == capability)
            .map(|(_, owner)| *owner);
        match found {
            None => findings.push(format!("capability {} is missing owner", capability)),
            Some(owner) if owner != *expected_owner => findings.push(format!(
                "capability {} owner {} != expected {}",
                capability, owner, expected_owner
            )),
            Some(_) => {}
        }
    }

    for (capability, owner) in assignments {
        if !expected
            .iter()
            .any(|(exp_capability, exp_owner)| exp_capability == capability && exp_owner == owner)
        {
            findings.push(format!(
                "unexpected owner mapping {} -> {}",
                capability, owner
            ));
        }
    }

    OwnerAmbiguityGateReport {
        passed: findings.is_empty(),
        findings,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section40AuditChainGateReport {
    pub passed: bool,
    pub missing_engines: Vec<String>,
    pub duplicate_engine_idempotency: Vec<String>,
    pub event_ids_monotonic: bool,
}

pub fn evaluate_section40_audit_chain(
    store: &Ph1fStore,
    correlation_id: CorrelationId,
) -> Section40AuditChainGateReport {
    let required = [
        "PH1.VOICE.ID",
        "PH1.FEEDBACK",
        "PH1.LEARN",
        "PH1.ACCESS/CAPREQ",
        "PH1.GOV",
        "PH1.BUILDER",
    ];
    let rows = store.audit_events_by_correlation(correlation_id);

    let mut missing = Vec::new();
    for required_engine in required {
        let found = rows
            .iter()
            .any(|row| audit_engine_label(&row.engine) == required_engine);
        if !found {
            missing.push(required_engine.to_string());
        }
    }

    let event_ids_monotonic = rows
        .windows(2)
        .all(|pair| pair[0].event_id.0 < pair[1].event_id.0);

    let mut seen_engine_idem = BTreeSet::new();
    let mut duplicate_engine_idempotency = Vec::new();
    for row in rows {
        if let Some(idempotency) = row.idempotency_key.as_ref() {
            let key = format!("{}|{}", audit_engine_label(&row.engine), idempotency);
            if !seen_engine_idem.insert(key.clone()) {
                duplicate_engine_idempotency.push(key);
            }
        }
    }

    Section40AuditChainGateReport {
        passed: missing.is_empty()
            && event_ids_monotonic
            && duplicate_engine_idempotency.is_empty(),
        missing_engines: missing,
        duplicate_engine_idempotency,
        event_ids_monotonic,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LadderCutoverGateInput {
    pub migration_stage: VoiceIdContractMigrationStage,
    pub migration_shadow_drift_events: u32,
    pub migration_total_events: u32,
    pub prompt_rate_kpis: BuilderPromptRateKpis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LadderCutoverGateReport {
    pub passed: bool,
    pub findings: Vec<String>,
}

pub fn evaluate_ladder_cutover_and_prompt_gate(
    input: LadderCutoverGateInput,
) -> LadderCutoverGateReport {
    let mut findings = Vec::new();

    if input.migration_stage != VoiceIdContractMigrationStage::M3 {
        findings.push(format!(
            "migration stage {} != M3 cutover complete",
            input.migration_stage.as_str()
        ));
    }
    if input.migration_total_events == 0 {
        findings.push("migration_total_events must be > 0".to_string());
    }
    if input.migration_shadow_drift_events > 0 {
        findings.push(format!(
            "migration shadow drift events {} > 0",
            input.migration_shadow_drift_events
        ));
    }
    if !input.prompt_rate_kpis.passes_gate() {
        findings.push("prompt-rate KPI gate failed".to_string());
    }

    LadderCutoverGateReport {
        passed: findings.is_empty(),
        findings,
    }
}

fn audit_engine_label(engine: &AuditEngine) -> &str {
    match engine {
        AuditEngine::Other(name) => name.as_str(),
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use selene_engines::ph1_voice_id::{
        simulation_profile_embedding_from_seed, EnrolledSpeaker as EngineEnrolledSpeaker,
        VoiceIdObservation as EngineVoiceIdObservation,
    };
    use selene_kernel_contracts::ph1_voice_id::{DeviceTrustLevel, Ph1VoiceIdRequest, UserId};
    use selene_kernel_contracts::ph1j::{
        AuditEngine, AuditEventInput, AuditEventType, AuditPayloadMin, AuditSeverity,
        CorrelationId, DeviceId, PayloadKey, PayloadValue, TurnId,
    };
    use selene_kernel_contracts::ph1k::{
        AudioDeviceId, AudioFormat, AudioStreamId, AudioStreamKind, AudioStreamRef, ChannelCount,
        Confidence, FrameDurationMs, SampleFormat, SampleRateHz, SpeechLikeness, VadEvent,
    };
    use selene_kernel_contracts::ph1l::{
        NextAllowedActions, SessionId, SessionSnapshot, PH1L_CONTRACT_VERSION,
    };
    use selene_kernel_contracts::ph1link::AppPlatform;
    use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, SessionState};
    use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore};
    use selene_storage::ph1j::Ph1jRuntime;

    use crate::ph1_voice_id::{
        Ph1VoiceIdLiveConfig, Ph1VoiceIdLiveRuntime, VoiceIdContractMigrationStage,
        VoiceIdentityChannel, VoiceIdentityRuntimeContext, VoiceIdentitySignalScope,
    };
    use crate::ph1builder::BuilderPromptRateKpis;

    use super::{
        evaluate_ladder_cutover_and_prompt_gate, evaluate_section40_audit_chain,
        evaluate_section40_owner_matrix, evaluate_voice_id_production_targets,
        LadderCutoverGateInput, VoiceIdCohortProductionSnapshot, VoiceIdProductionTargets,
        SECTION40_OWNER_ASSIGNMENTS,
    };

    #[test]
    fn at_section40_exit_02_voice_targets_require_per_cohort_pass() {
        let snapshots = vec![
            VoiceIdCohortProductionSnapshot::v1(
                "lang_en_us_device_ios_noise_quiet".to_string(),
                250,
                9_930,
                70,
                8,
                52,
                98,
            )
            .unwrap(),
            VoiceIdCohortProductionSnapshot::v1(
                "lang_en_us_device_android_noise_normal".to_string(),
                260,
                9_910,
                85,
                9,
                59,
                115,
            )
            .unwrap(),
        ];
        let report =
            evaluate_voice_id_production_targets(&snapshots, VoiceIdProductionTargets::strict_v1())
                .expect("snapshot gate evaluation must be valid");
        assert!(report.passed);

        let failing = vec![VoiceIdCohortProductionSnapshot::v1(
            "lang_es_mx_device_android_noise_noisy".to_string(),
            230,
            9_920,
            95,
            25,
            58,
            116,
        )
        .unwrap()];
        let fail_report =
            evaluate_voice_id_production_targets(&failing, VoiceIdProductionTargets::strict_v1())
                .expect("snapshot gate evaluation must be valid");
        assert!(!fail_report.passed);
        assert_eq!(
            fail_report.failing_cohorts,
            vec!["lang_es_mx_device_android_noise_noisy".to_string()]
        );
    }

    #[test]
    fn at_section40_exit_03_owner_matrix_is_single_authoritative() {
        let report = evaluate_section40_owner_matrix(&SECTION40_OWNER_ASSIGNMENTS);
        assert!(report.passed);

        let ambiguous = [
            ("identity_decision", "PH1.VOICE.ID"),
            ("identity_decision", "PH1.ONB"),
            ("signal_intake", "PH1.FEEDBACK"),
        ];
        let fail_report = evaluate_section40_owner_matrix(&ambiguous);
        assert!(!fail_report.passed);
        assert!(fail_report
            .findings
            .iter()
            .any(|f| f.contains("multiple owner declarations")));
    }

    #[test]
    fn at_section40_exit_04_audit_chain_intake_to_rollout_is_deterministic() {
        let mut store = Ph1fStore::new_in_memory();
        let actor = UserId::new("tenant_exit:user_1").unwrap();
        let device = DeviceId::new("device_exit_1").unwrap();
        seed_identity_and_device(&mut store, &actor, &device);

        let runtime = Ph1VoiceIdLiveRuntime::new(
            Ph1VoiceIdLiveConfig::mvp_v1_phone_first()
                .with_contract_migration_stage(VoiceIdContractMigrationStage::M2),
        );
        let correlation_id = CorrelationId(9602);
        let turn_id = TurnId(1);
        let request = sample_live_request(MonotonicTimeNs(3), actor.clone());
        runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &request,
                VoiceIdentityRuntimeContext::from_tenant_app_platform(
                    Some("tenant_exit".to_string()),
                    Some(AppPlatform::Android),
                    VoiceIdentityChannel::Explicit,
                ),
                Vec::new(),
                EngineVoiceIdObservation {
                    primary_fingerprint: None,
                    secondary_fingerprint: None,
                    primary_embedding: None,
                    secondary_embedding: None,
                    spoof_risk: false,
                },
                VoiceIdentitySignalScope::v1(
                    MonotonicTimeNs(3),
                    correlation_id,
                    turn_id,
                    actor.clone(),
                    Some("tenant_exit".to_string()),
                    Some(device.clone()),
                ),
            )
            .expect("voice-id signal emission must succeed");

        store
            .ph1access_capreq_step_up_audit_commit(
                MonotonicTimeNs(4),
                "tenant_exit".to_string(),
                correlation_id,
                turn_id,
                actor.clone(),
                "START".to_string(),
                "CHALLENGE".to_string(),
                "PAYROLL_APPROVE".to_string(),
                "biometric".to_string(),
                ReasonCodeId(0xACCE_0001),
                "exit4_access_start".to_string(),
            )
            .expect("step-up start audit must commit");
        store
            .ph1access_capreq_step_up_audit_commit(
                MonotonicTimeNs(5),
                "tenant_exit".to_string(),
                correlation_id,
                turn_id,
                actor.clone(),
                "FINISH".to_string(),
                "CONTINUE".to_string(),
                "PAYROLL_APPROVE".to_string(),
                "biometric".to_string(),
                ReasonCodeId(0xACCE_0002),
                "exit4_access_finish".to_string(),
            )
            .expect("step-up finish audit must commit");

        emit_other_engine_audit(
            &mut store,
            MonotonicTimeNs(6),
            "tenant_exit",
            correlation_id,
            turn_id,
            &actor,
            &device,
            "PH1.GOV",
            "decision",
            "ALLOW",
            ReasonCodeId(0x474F_0001),
            "exit4_gov_allow",
        );
        emit_other_engine_audit(
            &mut store,
            MonotonicTimeNs(7),
            "tenant_exit",
            correlation_id,
            turn_id,
            &actor,
            &device,
            "PH1.BUILDER",
            "rollout",
            "PROMOTE",
            ReasonCodeId(0xB13D_0001),
            "exit4_builder_promote",
        );

        // Deterministic idempotency: duplicate idempotency key must no-op.
        let before = store.audit_events_by_correlation(correlation_id).len();
        emit_other_engine_audit(
            &mut store,
            MonotonicTimeNs(8),
            "tenant_exit",
            correlation_id,
            turn_id,
            &actor,
            &device,
            "PH1.GOV",
            "decision",
            "ALLOW",
            ReasonCodeId(0x474F_0001),
            "exit4_gov_allow",
        );
        let after = store.audit_events_by_correlation(correlation_id).len();
        assert_eq!(before, after);

        let report = evaluate_section40_audit_chain(&store, correlation_id);
        assert!(report.passed, "{:?}", report);
    }

    #[test]
    fn at_section40_exit_05_ladder_cutover_and_prompt_gate_pass() {
        let mut store = Ph1fStore::new_in_memory();
        let actor = UserId::new("tenant_exit:user_2").unwrap();
        let device = DeviceId::new("device_exit_2").unwrap();
        seed_identity_and_device(&mut store, &actor, &device);

        let runtime = Ph1VoiceIdLiveRuntime::new(
            Ph1VoiceIdLiveConfig::mvp_v1_phone_first()
                .with_contract_migration_stage(VoiceIdContractMigrationStage::M3),
        );
        let correlation_id = CorrelationId(9603);
        let turn_id = TurnId(1);
        let request = sample_live_request(MonotonicTimeNs(3), actor.clone());
        let enrolled = vec![EngineEnrolledSpeaker {
            speaker_id: selene_kernel_contracts::ph1_voice_id::SpeakerId::new("spk_exit_2")
                .unwrap(),
            user_id: Some(actor.clone()),
            fingerprint: 7,
            profile_embedding: Some(simulation_profile_embedding_from_seed(7)),
        }];
        runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &request,
                VoiceIdentityRuntimeContext::from_tenant_app_platform(
                    Some("tenant_exit".to_string()),
                    Some(AppPlatform::Android),
                    VoiceIdentityChannel::Explicit,
                ),
                enrolled,
                EngineVoiceIdObservation {
                    primary_fingerprint: Some(7),
                    secondary_fingerprint: None,
                    primary_embedding: Some(simulation_profile_embedding_from_seed(7)),
                    secondary_embedding: None,
                    spoof_risk: false,
                },
                VoiceIdentitySignalScope::v1(
                    MonotonicTimeNs(3),
                    correlation_id,
                    turn_id,
                    actor,
                    Some("tenant_exit".to_string()),
                    Some(device),
                ),
            )
            .expect("voice-id m3 run must succeed");

        let migration_row = store
            .audit_events_by_correlation(correlation_id)
            .into_iter()
            .find(|row| {
                matches!(&row.engine, AuditEngine::Other(engine) if engine == "PH1.VOICE.ID")
                    && row
                        .payload_min
                        .entries
                        .contains_key(&PayloadKey::new("migration_stage").unwrap())
            })
            .expect("migration audit row must exist");

        let migration_stage = migration_stage_from_payload(&migration_row.payload_min.entries);
        let shadow_drift = migration_row
            .payload_min
            .entries
            .get(&PayloadKey::new("shadow_drift").unwrap())
            .expect("shadow_drift must exist")
            .as_str()
            == "true";

        let prompt_kpis = BuilderPromptRateKpis::v1(100, 0, 8_000, 120).unwrap();
        let report = evaluate_ladder_cutover_and_prompt_gate(LadderCutoverGateInput {
            migration_stage,
            migration_shadow_drift_events: u32::from(shadow_drift),
            migration_total_events: 1,
            prompt_rate_kpis: prompt_kpis,
        });
        assert!(report.passed, "{:?}", report);
    }

    fn migration_stage_from_payload(
        payload: &BTreeMap<PayloadKey, PayloadValue>,
    ) -> VoiceIdContractMigrationStage {
        match payload
            .get(&PayloadKey::new("migration_stage").unwrap())
            .expect("migration_stage must exist")
            .as_str()
        {
            "M0" => VoiceIdContractMigrationStage::M0,
            "M1" => VoiceIdContractMigrationStage::M1,
            "M2" => VoiceIdContractMigrationStage::M2,
            "M3" => VoiceIdContractMigrationStage::M3,
            other => panic!("unexpected migration stage {other}"),
        }
    }

    fn emit_other_engine_audit(
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        tenant_id: &str,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        user_id: &UserId,
        device_id: &DeviceId,
        engine_id: &str,
        payload_key: &str,
        payload_value: &str,
        reason_code: ReasonCodeId,
        idempotency_key: &str,
    ) {
        let mut payload = BTreeMap::new();
        payload.insert(
            PayloadKey::new(payload_key).unwrap(),
            PayloadValue::new(payload_value).unwrap(),
        );
        let event = AuditEventInput::v1(
            now,
            Some(tenant_id.to_string()),
            None,
            None,
            Some(user_id.clone()),
            Some(device_id.clone()),
            AuditEngine::Other(engine_id.to_string()),
            AuditEventType::Other,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            AuditPayloadMin::v1(payload).unwrap(),
            None,
            Some(idempotency_key.to_string()),
        )
        .unwrap();
        Ph1jRuntime::emit(store, event).expect("other-engine audit emit must succeed");
    }

    fn seed_identity_and_device(store: &mut Ph1fStore, user_id: &UserId, device_id: &DeviceId) {
        store
            .insert_identity(IdentityRecord::v1(
                user_id.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_device(
                DeviceRecord::v1(
                    device_id.clone(),
                    user_id.clone(),
                    "phone".to_string(),
                    MonotonicTimeNs(2),
                    None,
                )
                .unwrap(),
            )
            .unwrap();
    }

    fn sample_live_request(now: MonotonicTimeNs, owner_user_id: UserId) -> Ph1VoiceIdRequest {
        let stream_id = AudioStreamId(1);
        let processed_stream_ref = AudioStreamRef::v1(
            stream_id,
            AudioStreamKind::MicProcessed,
            AudioFormat {
                sample_rate_hz: SampleRateHz(16_000),
                channels: ChannelCount(1),
                sample_format: SampleFormat::PcmS16LE,
            },
            FrameDurationMs::Ms20,
        );
        let vad_events = vec![VadEvent::v1(
            stream_id,
            MonotonicTimeNs(now.0.saturating_sub(2_000_000)),
            now,
            Confidence::new(0.95).unwrap(),
            SpeechLikeness::new(0.95).unwrap(),
        )];
        let session_snapshot = SessionSnapshot {
            schema_version: PH1L_CONTRACT_VERSION,
            session_state: SessionState::Active,
            session_id: Some(SessionId(1)),
            next_allowed_actions: NextAllowedActions {
                may_speak: true,
                must_wait: false,
                must_rewake: false,
            },
        };
        Ph1VoiceIdRequest::v1(
            now,
            processed_stream_ref,
            vad_events,
            AudioDeviceId::new("section40_exit_mic_1").unwrap(),
            session_snapshot,
            None,
            false,
            DeviceTrustLevel::Trusted,
            Some(owner_user_id),
        )
        .unwrap()
    }
}
