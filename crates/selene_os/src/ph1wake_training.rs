#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};
use selene_kernel_contracts::ph1art::ArtifactVersion;
use selene_kernel_contracts::ph1learn::{
    LearnSignalType, WakeFeatureConfigV1, WakePackManifestV1, WakeTrainDatasetPartitionV1,
    WakeTrainDatasetScopeV1, WakeTrainDatasetSliceV1, WakeTrainEvalReportV1,
};
use selene_kernel_contracts::{ContractViolation, Validate};
use selene_storage::ph1f::{Ph1fStore, WakeEnrollmentSampleRecord, WakeSampleResult};

const TRAIN_SPLIT_BP: u16 = 8_000;
const VALIDATION_SPLIT_BP: u16 = 1_000;
const SPLIT_DENOMINATOR_BP: u16 = 10_000;
const DEFAULT_CALIBRATION_THRESHOLD_BP: u16 = 8_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WakeTrainingStep1Config {
    pub offline_pipeline_only: bool,
    pub time_adjacent_window_ms: u64,
}

impl Default for WakeTrainingStep1Config {
    fn default() -> Self {
        Self {
            offline_pipeline_only: true,
            time_adjacent_window_ms: 30_000,
        }
    }
}

impl Validate for WakeTrainingStep1Config {
    fn validate(&self) -> Result<(), ContractViolation> {
        if !self.offline_pipeline_only {
            return Err(ContractViolation::InvalidValue {
                field: "wake_training_step1_config.offline_pipeline_only",
                reason: "must be true",
            });
        }
        if self.time_adjacent_window_ms == 0 || self.time_adjacent_window_ms > 3_600_000 {
            return Err(ContractViolation::InvalidValue {
                field: "wake_training_step1_config.time_adjacent_window_ms",
                reason: "must be in [1, 3600000]",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeTrainingStep1Request {
    pub dataset_snapshot_id: String,
    pub feature_config_id: String,
    pub model_version: String,
    pub model_abi: String,
    pub threshold_profile_id: String,
    pub artifact_version: ArtifactVersion,
    pub package_hash: String,
    pub payload_ref: String,
    pub provenance_ref: String,
    pub rollback_to_artifact_version: Option<ArtifactVersion>,
    pub offline_pipeline_only: bool,
}

impl WakeTrainingStep1Request {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        dataset_snapshot_id: String,
        feature_config_id: String,
        model_version: String,
        model_abi: String,
        threshold_profile_id: String,
        artifact_version: ArtifactVersion,
        package_hash: String,
        payload_ref: String,
        provenance_ref: String,
        rollback_to_artifact_version: Option<ArtifactVersion>,
        offline_pipeline_only: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            dataset_snapshot_id,
            feature_config_id,
            model_version,
            model_abi,
            threshold_profile_id,
            artifact_version,
            package_hash,
            payload_ref,
            provenance_ref,
            rollback_to_artifact_version,
            offline_pipeline_only,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for WakeTrainingStep1Request {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token(
            "wake_training_step1_request.dataset_snapshot_id",
            &self.dataset_snapshot_id,
            128,
        )?;
        validate_token(
            "wake_training_step1_request.feature_config_id",
            &self.feature_config_id,
            96,
        )?;
        validate_token(
            "wake_training_step1_request.model_version",
            &self.model_version,
            64,
        )?;
        validate_token("wake_training_step1_request.model_abi", &self.model_abi, 64)?;
        validate_token(
            "wake_training_step1_request.threshold_profile_id",
            &self.threshold_profile_id,
            96,
        )?;
        self.artifact_version.validate()?;
        validate_lower_hex_sha256(
            "wake_training_step1_request.package_hash",
            &self.package_hash,
        )?;
        validate_token(
            "wake_training_step1_request.payload_ref",
            &self.payload_ref,
            256,
        )?;
        validate_token(
            "wake_training_step1_request.provenance_ref",
            &self.provenance_ref,
            128,
        )?;
        if let Some(rollback) = self.rollback_to_artifact_version {
            rollback.validate()?;
            if rollback >= self.artifact_version {
                return Err(ContractViolation::InvalidValue {
                    field: "wake_training_step1_request.rollback_to_artifact_version",
                    reason: "must be < artifact_version when present",
                });
            }
        }
        if !self.offline_pipeline_only {
            return Err(ContractViolation::InvalidValue {
                field: "wake_training_step1_request.offline_pipeline_only",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeTrainingStep1Output {
    pub feature_config: WakeFeatureConfigV1,
    pub dataset_slices: Vec<WakeTrainDatasetSliceV1>,
    pub eval_report: WakeTrainEvalReportV1,
    pub wake_pack_manifest: WakePackManifestV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WakeTrainingStep2Config {
    pub offline_pipeline_only: bool,
    pub time_adjacent_window_ms: u64,
    pub default_latency_proxy_ms: u16,
}

impl Default for WakeTrainingStep2Config {
    fn default() -> Self {
        Self {
            offline_pipeline_only: true,
            time_adjacent_window_ms: 30_000,
            default_latency_proxy_ms: 120,
        }
    }
}

impl Validate for WakeTrainingStep2Config {
    fn validate(&self) -> Result<(), ContractViolation> {
        if !self.offline_pipeline_only {
            return Err(ContractViolation::InvalidValue {
                field: "wake_training_step2_config.offline_pipeline_only",
                reason: "must be true",
            });
        }
        if self.time_adjacent_window_ms == 0 || self.time_adjacent_window_ms > 3_600_000 {
            return Err(ContractViolation::InvalidValue {
                field: "wake_training_step2_config.time_adjacent_window_ms",
                reason: "must be in [1, 3600000]",
            });
        }
        if self.default_latency_proxy_ms == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "wake_training_step2_config.default_latency_proxy_ms",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeTrainingStep2Request {
    pub dataset_snapshot_id: String,
    pub feature_config_id: String,
    pub model_version: String,
    pub model_abi: String,
    pub artifact_version: ArtifactVersion,
    pub provenance_ref: String,
    pub rollback_to_artifact_version: Option<ArtifactVersion>,
    pub offline_pipeline_only: bool,
}

impl WakeTrainingStep2Request {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        dataset_snapshot_id: String,
        feature_config_id: String,
        model_version: String,
        model_abi: String,
        artifact_version: ArtifactVersion,
        provenance_ref: String,
        rollback_to_artifact_version: Option<ArtifactVersion>,
        offline_pipeline_only: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            dataset_snapshot_id,
            feature_config_id,
            model_version,
            model_abi,
            artifact_version,
            provenance_ref,
            rollback_to_artifact_version,
            offline_pipeline_only,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for WakeTrainingStep2Request {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token(
            "wake_training_step2_request.dataset_snapshot_id",
            &self.dataset_snapshot_id,
            128,
        )?;
        validate_token(
            "wake_training_step2_request.feature_config_id",
            &self.feature_config_id,
            96,
        )?;
        validate_token(
            "wake_training_step2_request.model_version",
            &self.model_version,
            64,
        )?;
        validate_token("wake_training_step2_request.model_abi", &self.model_abi, 64)?;
        self.artifact_version.validate()?;
        validate_token(
            "wake_training_step2_request.provenance_ref",
            &self.provenance_ref,
            128,
        )?;
        if let Some(rollback) = self.rollback_to_artifact_version {
            rollback.validate()?;
            if rollback >= self.artifact_version {
                return Err(ContractViolation::InvalidValue {
                    field: "wake_training_step2_request.rollback_to_artifact_version",
                    reason: "must be < artifact_version when present",
                });
            }
        }
        if !self.offline_pipeline_only {
            return Err(ContractViolation::InvalidValue {
                field: "wake_training_step2_request.offline_pipeline_only",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeDatasetRowV1 {
    pub row_id: String,
    pub source_kind: String,
    pub label: String,
    pub user_id: Option<String>,
    pub device_id: String,
    pub timestamp_ms: u64,
    pub score_bp: Option<u16>,
    pub reason_ref: Option<String>,
    pub extractability: String,
    pub extractable: bool,
    pub latency_proxy_ms: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeLeakageGuardSummaryV1 {
    pub no_user_leakage: bool,
    pub no_device_leakage: bool,
    pub no_time_adjacent_leakage: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeDatasetAssemblySummaryV1 {
    pub total_rows: u32,
    pub positive_rows: u32,
    pub negative_rows: u32,
    pub source_counts: BTreeMap<String, u32>,
    pub label_counts: BTreeMap<String, u32>,
    pub extractability_counts: BTreeMap<String, u32>,
    pub leakage_guards: WakeLeakageGuardSummaryV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeThresholdCalibrationV1 {
    pub threshold_profile_id: String,
    pub calibrated_threshold_bp: u16,
    pub calibration_error_bp: u16,
    pub positive_scored_count: u32,
    pub negative_scored_count: u32,
    pub threshold_calibration_summary_ref: String,
    pub not_measured_metrics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeCandidatePackageV1 {
    pub candidate_package_id: String,
    pub payload_ref: String,
    pub package_hash: String,
    pub payload_len_bytes: u32,
    pub payload_bytes: Vec<u8>,
    pub candidate_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeTrainingStep2Output {
    pub feature_config: WakeFeatureConfigV1,
    pub dataset_rows: Vec<WakeDatasetRowV1>,
    pub dataset_slices: Vec<WakeTrainDatasetSliceV1>,
    pub dataset_summary: WakeDatasetAssemblySummaryV1,
    pub threshold_calibration: WakeThresholdCalibrationV1,
    pub eval_report: WakeTrainEvalReportV1,
    pub wake_pack_manifest: WakePackManifestV1,
    pub wakepack_candidate: WakeCandidatePackageV1,
}

pub fn build_wake_training_step1(
    store: &Ph1fStore,
    request: &WakeTrainingStep1Request,
    config: &WakeTrainingStep1Config,
) -> Result<WakeTrainingStep1Output, ContractViolation> {
    request.validate()?;
    config.validate()?;
    if !request.offline_pipeline_only || !config.offline_pipeline_only {
        return Err(ContractViolation::InvalidValue {
            field: "wake_training_step1.offline_pipeline_only",
            reason: "runtime path must be OFFLINE_PIPELINE_ONLY",
        });
    }

    let feature_config = WakeFeatureConfigV1::locked_default_v1(request.feature_config_id.clone())?;
    let mut examples = collect_dataset_examples(store);
    if examples.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field: "wake_training_step1.dataset_examples",
            reason: "must include at least one wake dataset example",
        });
    }
    examples.sort_by(|a, b| a.example_id.cmp(&b.example_id));

    let (assignments, leakage_report) =
        assign_dataset_partitions(&examples, config.time_adjacent_window_ms)?;
    enforce_leakage_guards(&leakage_report)?;

    let mut dataset_slices = build_global_slices(
        &examples,
        &assignments,
        request.dataset_snapshot_id.as_str(),
        &leakage_report,
    )?;
    dataset_slices.extend(build_per_user_adaptation_slices(
        &examples,
        request.dataset_snapshot_id.as_str(),
    )?);
    if dataset_slices.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field: "wake_training_step1.dataset_slices",
            reason: "must produce at least one dataset slice",
        });
    }

    let eval_report = build_eval_report(request, &examples)?;
    let wake_pack_manifest = WakePackManifestV1::v1(
        request.model_version.clone(),
        request.model_abi.clone(),
        feature_config.feature_config_id.clone(),
        request.threshold_profile_id.clone(),
        request.artifact_version,
        request.package_hash.clone(),
        request.payload_ref.clone(),
        request.provenance_ref.clone(),
        request.dataset_snapshot_id.clone(),
        eval_report.clone(),
        request.rollback_to_artifact_version,
        true,
    )?;

    Ok(WakeTrainingStep1Output {
        feature_config,
        dataset_slices,
        eval_report,
        wake_pack_manifest,
    })
}

pub fn build_wake_training_step2(
    store: &Ph1fStore,
    request: &WakeTrainingStep2Request,
    config: &WakeTrainingStep2Config,
) -> Result<WakeTrainingStep2Output, ContractViolation> {
    request.validate()?;
    config.validate()?;
    if !request.offline_pipeline_only || !config.offline_pipeline_only {
        return Err(ContractViolation::InvalidValue {
            field: "wake_training_step2.offline_pipeline_only",
            reason: "runtime path must be OFFLINE_PIPELINE_ONLY",
        });
    }

    let feature_config = WakeFeatureConfigV1::locked_default_v1(request.feature_config_id.clone())?;
    let mut examples = collect_dataset_examples(store);
    if examples.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field: "wake_training_step2.dataset_examples",
            reason: "must include at least one wake dataset example",
        });
    }
    examples.sort_by(|a, b| a.example_id.cmp(&b.example_id));

    let (assignments, leakage_report) =
        assign_dataset_partitions(&examples, config.time_adjacent_window_ms)?;
    enforce_leakage_guards(&leakage_report)?;

    let mut dataset_slices = build_global_slices(
        &examples,
        &assignments,
        request.dataset_snapshot_id.as_str(),
        &leakage_report,
    )?;
    dataset_slices.extend(build_per_user_adaptation_slices(
        &examples,
        request.dataset_snapshot_id.as_str(),
    )?);
    if dataset_slices.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field: "wake_training_step2.dataset_slices",
            reason: "must produce at least one dataset slice",
        });
    }

    let threshold_calibration = calibrate_threshold_profile(
        request.dataset_snapshot_id.as_str(),
        request.model_version.as_str(),
        feature_config.feature_config_id.as_str(),
        &examples,
    );

    let eval_report = build_eval_report_step2(
        request,
        &examples,
        &dataset_slices,
        &threshold_calibration,
        config.default_latency_proxy_ms,
    )?;

    let dataset_summary = build_dataset_summary(&examples, &leakage_report);
    let wakepack_candidate = build_wakepack_candidate_package(
        request,
        &feature_config,
        &threshold_calibration,
        &eval_report,
        &dataset_summary,
        &dataset_slices,
    )?;

    let wake_pack_manifest = WakePackManifestV1::v1(
        request.model_version.clone(),
        request.model_abi.clone(),
        feature_config.feature_config_id.clone(),
        threshold_calibration.threshold_profile_id.clone(),
        request.artifact_version,
        wakepack_candidate.package_hash.clone(),
        wakepack_candidate.payload_ref.clone(),
        request.provenance_ref.clone(),
        request.dataset_snapshot_id.clone(),
        eval_report.clone(),
        request.rollback_to_artifact_version,
        true,
    )?;

    Ok(WakeTrainingStep2Output {
        feature_config,
        dataset_rows: convert_to_dataset_rows(&examples),
        dataset_slices,
        dataset_summary,
        threshold_calibration,
        eval_report,
        wake_pack_manifest,
        wakepack_candidate,
    })
}

pub fn extract_log_mel_feature_bins(
    feature_config: &WakeFeatureConfigV1,
    pcm_s16: &[i16],
) -> Result<Vec<Vec<u16>>, ContractViolation> {
    feature_config.validate()?;
    if pcm_s16.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field: "extract_log_mel_feature_bins.pcm_s16",
            reason: "must not be empty",
        });
    }
    let frame_samples =
        (feature_config.sample_rate_hz as usize * feature_config.frame_ms as usize) / 1000;
    let hop_samples = (feature_config.sample_rate_hz as usize * feature_config.hop_ms as usize) / 1000;
    if frame_samples == 0 || hop_samples == 0 {
        return Err(ContractViolation::InvalidValue {
            field: "extract_log_mel_feature_bins.frame_or_hop_samples",
            reason: "frame/hop derived sample counts must be > 0",
        });
    }
    if pcm_s16.len() < frame_samples {
        return Err(ContractViolation::InvalidValue {
            field: "extract_log_mel_feature_bins.pcm_s16",
            reason: "input must contain at least one full frame",
        });
    }

    let mut frames = Vec::new();
    let mut start = 0usize;
    while start + frame_samples <= pcm_s16.len() {
        let frame = &pcm_s16[start..start + frame_samples];
        let energy_sum = frame
            .iter()
            .fold(0u64, |acc, sample| acc.saturating_add(sample.unsigned_abs() as u64));
        let energy_bp = (energy_sum
            .saturating_mul(10_000)
            .saturating_div((frame_samples as u64).saturating_mul(i16::MAX as u64)))
        .min(10_000) as u16;
        let mut bins = Vec::with_capacity(feature_config.mel_bins as usize);
        for idx in 0..feature_config.mel_bins {
            let scaled = ((energy_bp as u32)
                .saturating_mul((idx as u32).saturating_add(1))
                .saturating_div(feature_config.mel_bins as u32))
            .min(10_000) as u16;
            bins.push(scaled);
        }
        frames.push(bins);
        start = start.saturating_add(hop_samples);
    }
    Ok(frames)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WakeDatasetLabel {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WakeDatasetSourceKind {
    EnrollmentPass,
    RuntimeAccepted,
    RuntimeRejected,
    LearnSignal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WakeDatasetExtractability {
    RawPcmAvailable,
    MetadataOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WakeDatasetExample {
    example_id: String,
    user_id: Option<String>,
    device_id: String,
    timestamp_ms: u64,
    label: WakeDatasetLabel,
    source_kind: WakeDatasetSourceKind,
    score_bp: Option<u16>,
    reason_ref: Option<String>,
    extractability: WakeDatasetExtractability,
    latency_proxy_ms: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WakeLeakageGuardReport {
    no_user_leakage: bool,
    no_device_leakage: bool,
    no_time_adjacent_leakage: bool,
}

fn collect_dataset_examples(store: &Ph1fStore) -> Vec<WakeDatasetExample> {
    let mut session_user_by_id: BTreeMap<String, String> = BTreeMap::new();
    let mut session_device_by_id: BTreeMap<String, String> = BTreeMap::new();
    for session in store.ph1w_all_enrollment_session_rows() {
        session_user_by_id.insert(
            session.wake_enrollment_session_id.clone(),
            session.user_id.as_str().to_string(),
        );
        session_device_by_id.insert(
            session.wake_enrollment_session_id.clone(),
            session.device_id.as_str().to_string(),
        );
    }

    let mut examples = Vec::new();

    for sample in store.ph1w_all_enrollment_sample_rows() {
        if sample.result != WakeSampleResult::Pass {
            continue;
        }
        let Some(user_id) = session_user_by_id.get(&sample.wake_enrollment_session_id) else {
            continue;
        };
        let Some(device_id) = session_device_by_id.get(&sample.wake_enrollment_session_id) else {
            continue;
        };
        examples.push(WakeDatasetExample {
            example_id: format!(
                "enroll:{}:{}",
                sample.wake_enrollment_session_id, sample.sample_seq
            ),
            user_id: Some(user_id.clone()),
            device_id: device_id.clone(),
            timestamp_ms: ns_to_ms(sample.captured_at.0),
            label: WakeDatasetLabel::Positive,
            source_kind: WakeDatasetSourceKind::EnrollmentPass,
            score_bp: score_from_enrollment_sample(sample.vad_coverage, sample.snr_db, sample.clipping_pct),
            reason_ref: sample
                .reason_code
                .map(|reason| reason_ref_from_code(reason.0)),
            extractability: extractability_from_enrollment_sample(sample),
            latency_proxy_ms: Some(sample.sample_duration_ms),
        });
    }

    for event in store.ph1w_get_runtime_events() {
        examples.push(WakeDatasetExample {
            example_id: format!("runtime:{}", event.wake_event_id),
            user_id: event.user_id.as_ref().map(|user_id| user_id.as_str().to_string()),
            device_id: event.device_id.as_str().to_string(),
            timestamp_ms: ns_to_ms(event.created_at.0),
            label: if event.accepted {
                WakeDatasetLabel::Positive
            } else {
                WakeDatasetLabel::Negative
            },
            source_kind: if event.accepted {
                WakeDatasetSourceKind::RuntimeAccepted
            } else {
                WakeDatasetSourceKind::RuntimeRejected
            },
            score_bp: event.strong_score_bp.or(event.light_score_bp),
            reason_ref: Some(reason_ref_from_code(event.reason_code.0)),
            extractability: WakeDatasetExtractability::MetadataOnly,
            latency_proxy_ms: event
                .window_start_ns
                .zip(event.window_end_ns)
                .map(|(start, end)| ns_to_ms(end.0.saturating_sub(start.0)) as u16),
        });
    }

    for signal in store.wake_learn_signal_rows() {
        let Some(label) = wake_learn_label(signal.event_type) else {
            continue;
        };
        examples.push(WakeDatasetExample {
            example_id: format!("learn:{}", signal.signal_id),
            user_id: None,
            device_id: signal.device_id.as_str().to_string(),
            timestamp_ms: signal.timestamp_ms,
            label,
            source_kind: WakeDatasetSourceKind::LearnSignal,
            score_bp: signal.score_bp,
            reason_ref: signal.reason_code.map(|reason| reason_ref_from_code(reason.0)),
            extractability: WakeDatasetExtractability::MetadataOnly,
            latency_proxy_ms: None,
        });
    }

    examples
}

fn score_from_enrollment_sample(vad_coverage: f32, snr_db: f32, clipping_pct: f32) -> Option<u16> {
    if !vad_coverage.is_finite() || !snr_db.is_finite() || !clipping_pct.is_finite() {
        return None;
    }
    let vad_component = (vad_coverage.clamp(0.0, 1.0) * 6_000.0).round() as i32;
    let snr_component = ((snr_db.clamp(0.0, 30.0) / 30.0) * 4_000.0).round() as i32;
    let clipping_penalty = (clipping_pct.clamp(0.0, 1.0) * 2_000.0).round() as i32;
    let raw = vad_component
        .saturating_add(snr_component)
        .saturating_sub(clipping_penalty)
        .clamp(0, 10_000);
    Some(raw as u16)
}

fn reason_ref_from_code(reason_code_raw: u32) -> String {
    format!("wake_reason_0x{reason_code_raw:08x}")
}

fn extractability_from_enrollment_sample(
    sample: &WakeEnrollmentSampleRecord,
) -> WakeDatasetExtractability {
    if sample.idempotency_key.starts_with("pcm:") {
        WakeDatasetExtractability::RawPcmAvailable
    } else {
        WakeDatasetExtractability::MetadataOnly
    }
}

fn wake_learn_label(signal_type: LearnSignalType) -> Option<WakeDatasetLabel> {
    match signal_type {
        LearnSignalType::WakeAccepted => Some(WakeDatasetLabel::Positive),
        LearnSignalType::WakeRejected
        | LearnSignalType::FalseWake
        | LearnSignalType::MissedWake
        | LearnSignalType::LowConfidenceWake
        | LearnSignalType::NoisyEnvironment => Some(WakeDatasetLabel::Negative),
        _ => None,
    }
}

fn assign_dataset_partitions(
    examples: &[WakeDatasetExample],
    time_adjacent_window_ms: u64,
) -> Result<(BTreeMap<String, WakeTrainDatasetPartitionV1>, WakeLeakageGuardReport), ContractViolation>
{
    let mut base_device_partition: BTreeMap<String, WakeTrainDatasetPartitionV1> = BTreeMap::new();
    for example in examples {
        base_device_partition
            .entry(example.device_id.clone())
            .or_insert_with(|| partition_from_key(example.device_id.as_str()));
    }

    let mut user_partition: BTreeMap<String, WakeTrainDatasetPartitionV1> = BTreeMap::new();
    let mut device_partition_override: BTreeMap<String, WakeTrainDatasetPartitionV1> =
        BTreeMap::new();
    let mut device_conflict = false;
    let mut user_examples = examples
        .iter()
        .filter(|example| example.user_id.is_some())
        .collect::<Vec<_>>();
    user_examples.sort_by(|a, b| a.example_id.cmp(&b.example_id));

    for example in user_examples {
        let user_id = example
            .user_id
            .as_ref()
            .expect("filtered user examples must include user id");
        let default_partition = base_device_partition
            .get(example.device_id.as_str())
            .copied()
            .ok_or(ContractViolation::InvalidValue {
                field: "assign_dataset_partitions.base_device_partition",
                reason: "missing base partition for device",
            })?;
        let selected_partition = if let Some(existing) = user_partition.get(user_id.as_str()) {
            *existing
        } else {
            user_partition.insert(user_id.clone(), default_partition);
            default_partition
        };

        match device_partition_override.get(example.device_id.as_str()) {
            Some(existing) if *existing != selected_partition => {
                device_conflict = true;
            }
            _ => {
                device_partition_override.insert(example.device_id.clone(), selected_partition);
            }
        }
    }

    let mut assignment = BTreeMap::new();
    for example in examples {
        let partition = if let Some(override_partition) =
            device_partition_override.get(example.device_id.as_str())
        {
            *override_partition
        } else if let Some(user_id) = example.user_id.as_ref() {
            user_partition
                .get(user_id.as_str())
                .copied()
                .unwrap_or_else(|| partition_from_key(example.device_id.as_str()))
        } else {
            base_device_partition
                .get(example.device_id.as_str())
                .copied()
                .unwrap_or_else(|| partition_from_key(example.device_id.as_str()))
        };
        assignment.insert(example.example_id.clone(), partition);
    }

    let report = evaluate_leakage_report(
        examples,
        &assignment,
        time_adjacent_window_ms,
        device_conflict,
    );
    Ok((assignment, report))
}

fn evaluate_leakage_report(
    examples: &[WakeDatasetExample],
    assignment: &BTreeMap<String, WakeTrainDatasetPartitionV1>,
    time_adjacent_window_ms: u64,
    device_conflict: bool,
) -> WakeLeakageGuardReport {
    let mut users_by_partition: BTreeMap<WakeTrainDatasetPartitionV1, BTreeSet<String>> =
        BTreeMap::new();
    let mut devices_by_partition: BTreeMap<WakeTrainDatasetPartitionV1, BTreeSet<String>> =
        BTreeMap::new();
    let mut times_by_device: BTreeMap<String, Vec<(u64, WakeTrainDatasetPartitionV1)>> =
        BTreeMap::new();

    for example in examples {
        let Some(partition) = assignment.get(example.example_id.as_str()).copied() else {
            continue;
        };
        devices_by_partition
            .entry(partition)
            .or_default()
            .insert(example.device_id.clone());
        if let Some(user_id) = example.user_id.as_ref() {
            users_by_partition
                .entry(partition)
                .or_default()
                .insert(user_id.clone());
        }
        times_by_device
            .entry(example.device_id.clone())
            .or_default()
            .push((example.timestamp_ms, partition));
    }

    let no_user_leakage = no_set_overlap_across_partitions(&users_by_partition);
    let no_device_leakage = !device_conflict && no_set_overlap_across_partitions(&devices_by_partition);

    let mut no_time_adjacent_leakage = true;
    for rows in times_by_device.values_mut() {
        rows.sort_by_key(|(timestamp_ms, _)| *timestamp_ms);
        for pair in rows.windows(2) {
            let (left_ts, left_partition) = pair[0];
            let (right_ts, right_partition) = pair[1];
            if right_partition != left_partition
                && right_ts.saturating_sub(left_ts) <= time_adjacent_window_ms
            {
                no_time_adjacent_leakage = false;
                break;
            }
        }
        if !no_time_adjacent_leakage {
            break;
        }
    }

    WakeLeakageGuardReport {
        no_user_leakage,
        no_device_leakage,
        no_time_adjacent_leakage,
    }
}

fn no_set_overlap_across_partitions(
    by_partition: &BTreeMap<WakeTrainDatasetPartitionV1, BTreeSet<String>>,
) -> bool {
    let partitions = [
        WakeTrainDatasetPartitionV1::Train,
        WakeTrainDatasetPartitionV1::Validation,
        WakeTrainDatasetPartitionV1::Test,
    ];
    for (idx, left_partition) in partitions.iter().enumerate() {
        let Some(left) = by_partition.get(left_partition) else {
            continue;
        };
        for right_partition in partitions.iter().skip(idx + 1) {
            let Some(right) = by_partition.get(right_partition) else {
                continue;
            };
            if left.iter().any(|value| right.contains(value)) {
                return false;
            }
        }
    }
    true
}

fn enforce_leakage_guards(report: &WakeLeakageGuardReport) -> Result<(), ContractViolation> {
    if !report.no_user_leakage {
        return Err(ContractViolation::InvalidValue {
            field: "wake_training_step1.leakage.no_user_leakage",
            reason: "must be true",
        });
    }
    if !report.no_device_leakage {
        return Err(ContractViolation::InvalidValue {
            field: "wake_training_step1.leakage.no_device_leakage",
            reason: "must be true",
        });
    }
    if !report.no_time_adjacent_leakage {
        return Err(ContractViolation::InvalidValue {
            field: "wake_training_step1.leakage.no_time_adjacent_leakage",
            reason: "must be true",
        });
    }
    Ok(())
}

fn build_global_slices(
    examples: &[WakeDatasetExample],
    assignment: &BTreeMap<String, WakeTrainDatasetPartitionV1>,
    dataset_snapshot_id: &str,
    leakage_report: &WakeLeakageGuardReport,
) -> Result<Vec<WakeTrainDatasetSliceV1>, ContractViolation> {
    let mut out = Vec::new();
    for partition in [
        WakeTrainDatasetPartitionV1::Train,
        WakeTrainDatasetPartitionV1::Validation,
        WakeTrainDatasetPartitionV1::Test,
    ] {
        let partition_examples = examples
            .iter()
            .filter(|example| assignment.get(example.example_id.as_str()) == Some(&partition))
            .collect::<Vec<_>>();
        if partition_examples.is_empty() {
            continue;
        }
        let slice = build_slice_from_examples(
            dataset_snapshot_id,
            format!(
                "wake_global_{}_{}",
                partition_token(partition),
                stable_hex_short(format!("{}:{:?}", dataset_snapshot_id, partition).as_bytes())
            ),
            WakeTrainDatasetScopeV1::GlobalCorpus,
            partition,
            None,
            &partition_examples,
            leakage_report,
        )?;
        out.push(slice);
    }
    Ok(out)
}

fn build_per_user_adaptation_slices(
    examples: &[WakeDatasetExample],
    dataset_snapshot_id: &str,
) -> Result<Vec<WakeTrainDatasetSliceV1>, ContractViolation> {
    let mut by_user: BTreeMap<String, Vec<&WakeDatasetExample>> = BTreeMap::new();
    for example in examples {
        let Some(user_id) = example.user_id.as_ref() else {
            continue;
        };
        by_user.entry(user_id.clone()).or_default().push(example);
    }
    let leakage = WakeLeakageGuardReport {
        no_user_leakage: true,
        no_device_leakage: true,
        no_time_adjacent_leakage: true,
    };

    let mut out = Vec::new();
    for (user_id, user_examples) in by_user {
        let slice = build_slice_from_examples(
            dataset_snapshot_id,
            format!(
                "wake_adapt_{}",
                stable_hex_short(format!("{}:{}", dataset_snapshot_id, user_id).as_bytes())
            ),
            WakeTrainDatasetScopeV1::PerUserAdaptation,
            WakeTrainDatasetPartitionV1::Train,
            None,
            &user_examples,
            &leakage,
        )?;
        out.push(slice);
    }
    Ok(out)
}

fn build_slice_from_examples(
    dataset_snapshot_id: &str,
    slice_id: String,
    scope: WakeTrainDatasetScopeV1,
    partition: WakeTrainDatasetPartitionV1,
    platform_slice: Option<String>,
    examples: &[&WakeDatasetExample],
    leakage_report: &WakeLeakageGuardReport,
) -> Result<WakeTrainDatasetSliceV1, ContractViolation> {
    let positive_count = examples
        .iter()
        .filter(|example| example.label == WakeDatasetLabel::Positive)
        .count() as u32;
    let negative_count = examples
        .iter()
        .filter(|example| example.label == WakeDatasetLabel::Negative)
        .count() as u32;
    let unique_user_count = examples
        .iter()
        .filter_map(|example| example.user_id.as_ref().cloned())
        .collect::<BTreeSet<_>>()
        .len() as u32;
    let unique_device_count = examples
        .iter()
        .map(|example| example.device_id.clone())
        .collect::<BTreeSet<_>>()
        .len() as u32;
    let window_start_ms = examples
        .iter()
        .map(|example| example.timestamp_ms)
        .min()
        .unwrap_or(0);
    let window_end_ms = examples
        .iter()
        .map(|example| example.timestamp_ms)
        .max()
        .unwrap_or(0);

    WakeTrainDatasetSliceV1::v1(
        dataset_snapshot_id.to_string(),
        slice_id,
        scope,
        partition,
        platform_slice,
        positive_count,
        negative_count,
        unique_user_count,
        unique_device_count,
        leakage_report.no_user_leakage,
        leakage_report.no_device_leakage,
        leakage_report.no_time_adjacent_leakage,
        window_start_ms,
        window_end_ms,
    )
}

fn build_eval_report(
    request: &WakeTrainingStep1Request,
    examples: &[WakeDatasetExample],
) -> Result<WakeTrainEvalReportV1, ContractViolation> {
    let total = examples.len() as u32;
    let negatives = examples
        .iter()
        .filter(|example| example.label == WakeDatasetLabel::Negative)
        .count() as u32;
    let far_per_listening_hour_milli = negatives.saturating_mul(1_000).saturating_div(total.max(1));
    let generated_at_ms = examples
        .iter()
        .map(|example| example.timestamp_ms)
        .max()
        .unwrap_or(1);
    let reject_reason_distribution_ref = format!(
        "wake_reject_dist_{}",
        stable_hex_short(format!("{}:{}", request.dataset_snapshot_id, negatives).as_bytes())
    );

    WakeTrainEvalReportV1::v1(
        format!("wake_eval_{}", request.dataset_snapshot_id),
        request.dataset_snapshot_id.clone(),
        request.model_version.clone(),
        request.threshold_profile_id.clone(),
        far_per_listening_hour_milli,
        0,
        0,
        120,
        0,
        reject_reason_distribution_ref,
        generated_at_ms,
    )
}

fn convert_to_dataset_rows(examples: &[WakeDatasetExample]) -> Vec<WakeDatasetRowV1> {
    examples
        .iter()
        .map(|example| WakeDatasetRowV1 {
            row_id: example.example_id.clone(),
            source_kind: source_kind_token(example.source_kind).to_string(),
            label: label_token(example.label).to_string(),
            user_id: example.user_id.clone(),
            device_id: example.device_id.clone(),
            timestamp_ms: example.timestamp_ms,
            score_bp: example.score_bp,
            reason_ref: example.reason_ref.clone(),
            extractability: extractability_token(example.extractability).to_string(),
            extractable: matches!(example.extractability, WakeDatasetExtractability::RawPcmAvailable),
            latency_proxy_ms: example.latency_proxy_ms,
        })
        .collect()
}

fn build_dataset_summary(
    examples: &[WakeDatasetExample],
    leakage_report: &WakeLeakageGuardReport,
) -> WakeDatasetAssemblySummaryV1 {
    let mut source_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut label_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut extractability_counts: BTreeMap<String, u32> = BTreeMap::new();
    for example in examples {
        *source_counts
            .entry(source_kind_token(example.source_kind).to_string())
            .or_default() += 1;
        *label_counts
            .entry(label_token(example.label).to_string())
            .or_default() += 1;
        *extractability_counts
            .entry(extractability_token(example.extractability).to_string())
            .or_default() += 1;
    }
    WakeDatasetAssemblySummaryV1 {
        total_rows: examples.len() as u32,
        positive_rows: examples
            .iter()
            .filter(|example| example.label == WakeDatasetLabel::Positive)
            .count() as u32,
        negative_rows: examples
            .iter()
            .filter(|example| example.label == WakeDatasetLabel::Negative)
            .count() as u32,
        source_counts,
        label_counts,
        extractability_counts,
        leakage_guards: WakeLeakageGuardSummaryV1 {
            no_user_leakage: leakage_report.no_user_leakage,
            no_device_leakage: leakage_report.no_device_leakage,
            no_time_adjacent_leakage: leakage_report.no_time_adjacent_leakage,
        },
    }
}

fn calibrate_threshold_profile(
    dataset_snapshot_id: &str,
    model_version: &str,
    feature_config_id: &str,
    examples: &[WakeDatasetExample],
) -> WakeThresholdCalibrationV1 {
    let mut positive_scores = examples
        .iter()
        .filter(|example| example.label == WakeDatasetLabel::Positive)
        .filter_map(|example| example.score_bp)
        .collect::<Vec<_>>();
    let mut negative_scores = examples
        .iter()
        .filter(|example| example.label == WakeDatasetLabel::Negative)
        .filter_map(|example| example.score_bp)
        .collect::<Vec<_>>();
    positive_scores.sort_unstable();
    negative_scores.sort_unstable();

    let mut not_measured_metrics = Vec::new();
    let (calibrated_threshold_bp, calibration_error_bp) = if !positive_scores.is_empty()
        && !negative_scores.is_empty()
    {
        let positive_low_tail = quantile_bp(&positive_scores, 1, 10);
        let negative_high_tail = quantile_bp(&negative_scores, 9, 10);
        let threshold = ((positive_low_tail as u32 + negative_high_tail as u32) / 2) as u16;
        let overlap_bp = negative_high_tail.saturating_sub(positive_low_tail);
        (threshold, overlap_bp)
    } else {
        not_measured_metrics.push("threshold_calibration_scored_examples_missing".to_string());
        (DEFAULT_CALIBRATION_THRESHOLD_BP, 0)
    };

    let threshold_profile_id = format!(
        "wake_threshold_profile_{}",
        stable_hex_short(
            format!(
                "{}:{}:{}:{}",
                dataset_snapshot_id, model_version, feature_config_id, calibrated_threshold_bp
            )
            .as_bytes()
        )
    );
    let threshold_calibration_summary_ref = format!(
        "wake_threshold_calibration_{}",
        stable_hex_short(
            format!(
                "{}:{}:{}:{}:{}",
                dataset_snapshot_id,
                calibrated_threshold_bp,
                calibration_error_bp,
                positive_scores.len(),
                negative_scores.len()
            )
            .as_bytes()
        )
    );

    WakeThresholdCalibrationV1 {
        threshold_profile_id,
        calibrated_threshold_bp,
        calibration_error_bp,
        positive_scored_count: positive_scores.len() as u32,
        negative_scored_count: negative_scores.len() as u32,
        threshold_calibration_summary_ref,
        not_measured_metrics,
    }
}

fn build_eval_report_step2(
    request: &WakeTrainingStep2Request,
    examples: &[WakeDatasetExample],
    dataset_slices: &[WakeTrainDatasetSliceV1],
    threshold_calibration: &WakeThresholdCalibrationV1,
    default_latency_proxy_ms: u16,
) -> Result<WakeTrainEvalReportV1, ContractViolation> {
    let positive_scored = examples
        .iter()
        .filter(|example| example.label == WakeDatasetLabel::Positive)
        .filter_map(|example| example.score_bp)
        .collect::<Vec<_>>();
    let negative_scored = examples
        .iter()
        .filter(|example| example.label == WakeDatasetLabel::Negative)
        .filter_map(|example| example.score_bp)
        .collect::<Vec<_>>();

    let false_reject_count = positive_scored
        .iter()
        .filter(|score| **score < threshold_calibration.calibrated_threshold_bp)
        .count() as u32;
    let false_accept_count = negative_scored
        .iter()
        .filter(|score| **score >= threshold_calibration.calibrated_threshold_bp)
        .count() as u32;

    let mut not_measured = threshold_calibration.not_measured_metrics.clone();
    if positive_scored.is_empty() {
        not_measured.push("frr_not_measured_no_positive_scores".to_string());
    }
    if negative_scored.is_empty() {
        not_measured.push("far_not_measured_no_negative_scores".to_string());
    }
    not_measured.push("platform_slice_summary_open".to_string());

    let frr_bp = if positive_scored.is_empty() {
        0
    } else {
        false_reject_count
            .saturating_mul(10_000)
            .saturating_div(positive_scored.len() as u32) as u16
    };
    let far_per_listening_hour_milli = if negative_scored.is_empty() {
        0
    } else {
        false_accept_count
            .saturating_mul(1_000)
            .saturating_div(negative_scored.len() as u32)
    };
    let miss_rate_bp = frr_bp;
    let latency_proxy_ms = average_latency_proxy_ms(examples).unwrap_or(default_latency_proxy_ms);
    let (train_example_count, validation_example_count, test_example_count) =
        global_partition_counts(dataset_slices);

    let reject_reason_distribution_ref = build_reject_reason_distribution_ref(examples);
    let generated_at_ms = examples
        .iter()
        .map(|example| example.timestamp_ms)
        .max()
        .unwrap_or(1);
    let not_measured_metrics_ref = if not_measured.is_empty() {
        None
    } else {
        Some(format!(
            "wake_not_measured_{}",
            stable_hex_short(not_measured.join("|").as_bytes())
        ))
    };

    WakeTrainEvalReportV1::v2(
        format!("wake_eval_step2_{}", request.dataset_snapshot_id),
        request.dataset_snapshot_id.clone(),
        request.model_version.clone(),
        threshold_calibration.threshold_profile_id.clone(),
        far_per_listening_hour_milli,
        frr_bp,
        miss_rate_bp,
        latency_proxy_ms,
        Some(threshold_calibration.calibrated_threshold_bp),
        threshold_calibration.calibration_error_bp,
        threshold_calibration.threshold_calibration_summary_ref.clone(),
        reject_reason_distribution_ref,
        train_example_count,
        validation_example_count,
        test_example_count,
        None,
        not_measured_metrics_ref,
        generated_at_ms,
    )
}

fn build_wakepack_candidate_package(
    request: &WakeTrainingStep2Request,
    feature_config: &WakeFeatureConfigV1,
    threshold_calibration: &WakeThresholdCalibrationV1,
    eval_report: &WakeTrainEvalReportV1,
    dataset_summary: &WakeDatasetAssemblySummaryV1,
    dataset_slices: &[WakeTrainDatasetSliceV1],
) -> Result<WakeCandidatePackageV1, ContractViolation> {
    let candidate_package_id = format!(
        "wake_candidate_{}",
        stable_hex_short(
            format!(
                "{}:{}:{}:{}",
                request.dataset_snapshot_id,
                request.model_version,
                threshold_calibration.threshold_profile_id,
                request.artifact_version.0
            )
            .as_bytes()
        )
    );
    let payload_ref = format!(
        "wakepack/candidate/{}/{}.txt",
        request.dataset_snapshot_id, candidate_package_id
    );

    let payload = build_candidate_payload_text(
        request,
        feature_config,
        threshold_calibration,
        eval_report,
        dataset_summary,
        dataset_slices,
    );
    let payload_bytes = payload.into_bytes();
    let payload_len_bytes = u32::try_from(payload_bytes.len()).map_err(|_| {
        ContractViolation::InvalidValue {
            field: "wake_training_step2.candidate_payload_len_bytes",
            reason: "must be <= u32::MAX",
        }
    })?;
    let package_hash = sha256_hex(payload_bytes.as_slice());

    Ok(WakeCandidatePackageV1 {
        candidate_package_id,
        payload_ref,
        package_hash,
        payload_len_bytes,
        payload_bytes,
        candidate_only: true,
    })
}

fn build_candidate_payload_text(
    request: &WakeTrainingStep2Request,
    feature_config: &WakeFeatureConfigV1,
    threshold_calibration: &WakeThresholdCalibrationV1,
    eval_report: &WakeTrainEvalReportV1,
    dataset_summary: &WakeDatasetAssemblySummaryV1,
    dataset_slices: &[WakeTrainDatasetSliceV1],
) -> String {
    let mut lines = Vec::new();
    lines.push("wakepack_candidate_version=1".to_string());
    lines.push(format!("dataset_snapshot_id={}", request.dataset_snapshot_id));
    lines.push(format!("model_version={}", request.model_version));
    lines.push(format!("model_abi={}", request.model_abi));
    lines.push(format!(
        "feature_config_id={}",
        feature_config.feature_config_id
    ));
    lines.push(format!(
        "threshold_profile_id={}",
        threshold_calibration.threshold_profile_id
    ));
    lines.push(format!(
        "threshold_calibrated_bp={}",
        threshold_calibration.calibrated_threshold_bp
    ));
    lines.push(format!(
        "threshold_calibration_error_bp={}",
        threshold_calibration.calibration_error_bp
    ));
    lines.push(format!(
        "eval_far_per_listening_hour_milli={}",
        eval_report.far_per_listening_hour_milli
    ));
    lines.push(format!("eval_frr_bp={}", eval_report.frr_bp));
    lines.push(format!("eval_miss_rate_bp={}", eval_report.miss_rate_bp));
    lines.push(format!(
        "eval_latency_proxy_ms={}",
        eval_report.latency_proxy_ms
    ));
    lines.push(format!(
        "dataset_total_rows={}",
        dataset_summary.total_rows
    ));
    lines.push(format!(
        "dataset_positive_rows={}",
        dataset_summary.positive_rows
    ));
    lines.push(format!(
        "dataset_negative_rows={}",
        dataset_summary.negative_rows
    ));
    lines.push(format!(
        "dataset_slice_count={}",
        dataset_slices.len()
    ));
    for slice in dataset_slices {
        lines.push(format!(
            "slice:{}:{:?}:{:?}:{}:{}:{}",
            slice.slice_id,
            slice.scope,
            slice.partition,
            slice.positive_count,
            slice.negative_count,
            slice.unique_device_count
        ));
    }
    lines.push("offline_pipeline_only=true".to_string());
    lines.join("\n")
}

fn global_partition_counts(dataset_slices: &[WakeTrainDatasetSliceV1]) -> (u32, u32, u32) {
    let mut train = 0u32;
    let mut validation = 0u32;
    let mut test = 0u32;
    for slice in dataset_slices {
        if slice.scope != WakeTrainDatasetScopeV1::GlobalCorpus {
            continue;
        }
        let count = slice.positive_count.saturating_add(slice.negative_count);
        match slice.partition {
            WakeTrainDatasetPartitionV1::Train => train = train.saturating_add(count),
            WakeTrainDatasetPartitionV1::Validation => {
                validation = validation.saturating_add(count)
            }
            WakeTrainDatasetPartitionV1::Test => test = test.saturating_add(count),
        }
    }
    (train, validation, test)
}

fn average_latency_proxy_ms(examples: &[WakeDatasetExample]) -> Option<u16> {
    let mut sum = 0u64;
    let mut count = 0u64;
    for example in examples {
        let Some(latency) = example.latency_proxy_ms else {
            continue;
        };
        sum = sum.saturating_add(latency as u64);
        count = count.saturating_add(1);
    }
    if count == 0 {
        None
    } else {
        Some((sum / count) as u16)
    }
}

fn build_reject_reason_distribution_ref(examples: &[WakeDatasetExample]) -> String {
    let mut counts: BTreeMap<String, u32> = BTreeMap::new();
    for example in examples {
        if example.label != WakeDatasetLabel::Negative {
            continue;
        }
        let key = example
            .reason_ref
            .clone()
            .unwrap_or_else(|| "wake_reason_unknown".to_string());
        *counts.entry(key).or_default() += 1;
    }
    let mut canonical = String::new();
    for (reason_ref, count) in counts {
        canonical.push_str(reason_ref.as_str());
        canonical.push(':');
        canonical.push_str(count.to_string().as_str());
        canonical.push('|');
    }
    format!(
        "wake_reject_dist_{}",
        stable_hex_short(canonical.as_bytes())
    )
}

fn quantile_bp(sorted_values: &[u16], numerator: usize, denominator: usize) -> u16 {
    if sorted_values.is_empty() {
        return 0;
    }
    let idx = ((sorted_values.len().saturating_sub(1)).saturating_mul(numerator)) / denominator;
    sorted_values[idx]
}

fn source_kind_token(source_kind: WakeDatasetSourceKind) -> &'static str {
    match source_kind {
        WakeDatasetSourceKind::EnrollmentPass => "ENROLLMENT_PASS",
        WakeDatasetSourceKind::RuntimeAccepted => "RUNTIME_ACCEPTED",
        WakeDatasetSourceKind::RuntimeRejected => "RUNTIME_REJECTED",
        WakeDatasetSourceKind::LearnSignal => "LEARN_SIGNAL",
    }
}

fn label_token(label: WakeDatasetLabel) -> &'static str {
    match label {
        WakeDatasetLabel::Positive => "POSITIVE",
        WakeDatasetLabel::Negative => "NEGATIVE",
    }
}

fn extractability_token(extractability: WakeDatasetExtractability) -> &'static str {
    match extractability {
        WakeDatasetExtractability::RawPcmAvailable => "RAW_PCM_AVAILABLE",
        WakeDatasetExtractability::MetadataOnly => "METADATA_ONLY",
    }
}

fn partition_from_key(key: &str) -> WakeTrainDatasetPartitionV1 {
    let bucket = stable_bucket_bp(key.as_bytes());
    if bucket < TRAIN_SPLIT_BP {
        WakeTrainDatasetPartitionV1::Train
    } else if bucket < TRAIN_SPLIT_BP.saturating_add(VALIDATION_SPLIT_BP) {
        WakeTrainDatasetPartitionV1::Validation
    } else {
        WakeTrainDatasetPartitionV1::Test
    }
}

fn stable_bucket_bp(bytes: &[u8]) -> u16 {
    let digest = Sha256::digest(bytes);
    let mut head = [0u8; 8];
    head.copy_from_slice(&digest[..8]);
    let value = u64::from_be_bytes(head);
    (value % SPLIT_DENOMINATOR_BP as u64) as u16
}

fn stable_hex_short(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(16);
    for b in digest.iter().take(8) {
        out.push(hex_char((b >> 4) & 0x0F));
        out.push(hex_char(b & 0x0F));
    }
    out
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for b in digest {
        out.push(hex_char((b >> 4) & 0x0F));
        out.push(hex_char(b & 0x0F));
    }
    out
}

fn hex_char(v: u8) -> char {
    match v {
        0..=9 => (b'0' + v) as char,
        10..=15 => (b'a' + (v - 10)) as char,
        _ => '0',
    }
}

fn ns_to_ms(ns: u64) -> u64 {
    ns / 1_000_000
}

fn partition_token(partition: WakeTrainDatasetPartitionV1) -> &'static str {
    match partition {
        WakeTrainDatasetPartitionV1::Train => "train",
        WakeTrainDatasetPartitionV1::Validation => "validation",
        WakeTrainDatasetPartitionV1::Test => "test",
    }
}

fn validate_token(field: &'static str, value: &str, max_len: usize) -> Result<(), ContractViolation> {
    if value.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if value.chars().any(|c| {
        !(c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ':' || c == '.' || c == '/')
    }) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain only ASCII token characters",
        });
    }
    Ok(())
}

fn validate_lower_hex_sha256(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.len() != 64 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be 64-char lowercase SHA-256 hex",
        });
    }
    if value
        .chars()
        .any(|c| !matches!(c, '0'..='9' | 'a'..='f'))
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain lowercase hex chars only",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1j::DeviceId;
    use selene_kernel_contracts::ph1learn::{WakeLearnSignalV1, WakeLearnTrigger};
    use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
    use selene_storage::ph1f::{
        DeviceRecord, IdentityRecord, IdentityStatus, WakeSampleResult, Ph1fStore,
    };

    fn seeded_store() -> Ph1fStore {
        let mut store = Ph1fStore::new_in_memory();
        let t0 = MonotonicTimeNs(1_700_000_000_000_000_000);

        let user_a = UserId::new("user_a".to_string()).unwrap();
        let user_b = UserId::new("user_b".to_string()).unwrap();
        let device_a = DeviceId::new("device_a".to_string()).unwrap();
        let device_b = DeviceId::new("device_b".to_string()).unwrap();

        store
            .insert_identity(IdentityRecord::v1(
                user_a.clone(),
                None,
                None,
                t0,
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                user_b.clone(),
                None,
                None,
                MonotonicTimeNs(t0.0 + 10_000_000),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_device(
                DeviceRecord::v1(
                    device_a.clone(),
                    user_a.clone(),
                    "desktop".to_string(),
                    t0,
                    None,
                )
                .unwrap(),
            )
            .unwrap();
        store
            .insert_device(
                DeviceRecord::v1(
                    device_b.clone(),
                    user_b.clone(),
                    "desktop".to_string(),
                    MonotonicTimeNs(t0.0 + 10_000_000),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let s_a = store
            .ph1w_enroll_start_draft(
                MonotonicTimeNs(t0.0 + 20_000_000),
                user_a.clone(),
                device_a.clone(),
                None,
                3,
                8,
                180_000,
                "idem_start_a".to_string(),
            )
            .unwrap();
        let s_b = store
            .ph1w_enroll_start_draft(
                MonotonicTimeNs(t0.0 + 40_000_000),
                user_b.clone(),
                device_b.clone(),
                None,
                3,
                8,
                180_000,
                "idem_start_b".to_string(),
            )
            .unwrap();

        store
            .ph1w_enroll_sample_commit(
                MonotonicTimeNs(t0.0 + 60_000_000),
                s_a.wake_enrollment_session_id.clone(),
                1_200,
                0.92,
                18.0,
                0.2,
                -18.0,
                -40.0,
                -5.0,
                0.0,
                WakeSampleResult::Pass,
                None,
                "idem_sample_a".to_string(),
            )
            .unwrap();
        store
            .ph1w_enroll_sample_commit(
                MonotonicTimeNs(t0.0 + 80_000_000),
                s_b.wake_enrollment_session_id.clone(),
                1_150,
                0.88,
                14.5,
                0.4,
                -20.0,
                -42.0,
                -6.0,
                0.0,
                WakeSampleResult::Pass,
                None,
                "idem_sample_b".to_string(),
            )
            .unwrap();

        store
            .ph1w_runtime_event_commit(
                MonotonicTimeNs(t0.0 + 100_000_000),
                "wake_evt_accept_a".to_string(),
                None,
                Some(user_a.clone()),
                device_a.clone(),
                true,
                ReasonCodeId(0x1001),
                Some("wake_profile_a".to_string()),
                false,
                false,
                None,
                Some(9_200),
                Some(9_300),
                Some(8_600),
                Some("wake_model_v1".to_string()),
                Some(MonotonicTimeNs(t0.0 + 95_000_000)),
                Some(MonotonicTimeNs(t0.0 + 100_000_000)),
                "idem_evt_a".to_string(),
            )
            .unwrap();
        store
            .ph1w_runtime_event_commit(
                MonotonicTimeNs(t0.0 + 140_000_000),
                "wake_evt_reject_b".to_string(),
                None,
                Some(user_b.clone()),
                device_b.clone(),
                false,
                ReasonCodeId(0x1002),
                Some("wake_profile_b".to_string()),
                false,
                false,
                None,
                Some(4_200),
                Some(4_500),
                Some(8_600),
                Some("wake_model_v1".to_string()),
                Some(MonotonicTimeNs(t0.0 + 135_000_000)),
                Some(MonotonicTimeNs(t0.0 + 140_000_000)),
                "idem_evt_b".to_string(),
            )
            .unwrap();

        let noisy_signal = WakeLearnSignalV1::v1(
            "wake_signal_noise_a".to_string(),
            "idem_signal_noise_a".to_string(),
            "wake_window_noise_a".to_string(),
            LearnSignalType::NoisyEnvironment,
            device_a,
            None,
            WakeLearnTrigger::WakeWord,
            Some("wake_model_v1".to_string()),
            Some(3_800),
            Some(8_600),
            Some(ReasonCodeId(0x1003)),
            Some(3_400),
            Some(4_200),
            1_700_000_001_234,
        )
        .unwrap();
        store
            .wake_learn_signal_commit_and_enqueue(MonotonicTimeNs(t0.0 + 160_000_000), noisy_signal)
            .unwrap();

        store
    }

    fn valid_step1_request() -> WakeTrainingStep1Request {
        WakeTrainingStep1Request::v1(
            "wake_dataset_snapshot_v1".to_string(),
            "wake_feature_cfg_v1".to_string(),
            "wake_model_v1".to_string(),
            "wake_abi_v1".to_string(),
            "wake_threshold_v1".to_string(),
            ArtifactVersion(2),
            "4a6588bde3f9fcd4cea3f238d10ef00f8fbecc6453e28307fc1ff11337f6925f".to_string(),
            "wake_payload_ref_v1".to_string(),
            "wake_provenance_v1".to_string(),
            Some(ArtifactVersion(1)),
            true,
        )
        .unwrap()
    }

    fn valid_step2_request() -> WakeTrainingStep2Request {
        WakeTrainingStep2Request::v1(
            "wake_dataset_snapshot_v2".to_string(),
            "wake_feature_cfg_v1".to_string(),
            "wake_model_v2".to_string(),
            "wake_abi_v1".to_string(),
            ArtifactVersion(3),
            "wake_provenance_v2".to_string(),
            Some(ArtifactVersion(2)),
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_wake_training_step1_a_feature_config_validates() {
        let cfg = WakeFeatureConfigV1::locked_default_v1("wake_feature_cfg_v1".to_string()).unwrap();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn at_wake_training_step1_b_manifest_validates() {
        let eval = WakeTrainEvalReportV1::v1(
            "wake_eval_report_v1".to_string(),
            "wake_dataset_snapshot_v1".to_string(),
            "wake_model_v1".to_string(),
            "wake_threshold_v1".to_string(),
            10,
            110,
            90,
            125,
            40,
            "wake_reject_dist_v1".to_string(),
            1_700_000_001_000,
        )
        .unwrap();
        let manifest = WakePackManifestV1::v1(
            "wake_model_v1".to_string(),
            "wake_abi_v1".to_string(),
            "wake_feature_cfg_v1".to_string(),
            "wake_threshold_v1".to_string(),
            ArtifactVersion(2),
            "4a6588bde3f9fcd4cea3f238d10ef00f8fbecc6453e28307fc1ff11337f6925f".to_string(),
            "wake_payload_ref_v1".to_string(),
            "wake_provenance_v1".to_string(),
            "wake_dataset_snapshot_v1".to_string(),
            eval,
            Some(ArtifactVersion(1)),
            true,
        )
        .unwrap();
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn at_wake_training_step1_c_dataset_splits_are_deterministic() {
        let store = seeded_store();
        let request = valid_step1_request();
        let config = WakeTrainingStep1Config::default();

        let first = build_wake_training_step1(&store, &request, &config).unwrap();
        let second = build_wake_training_step1(&store, &request, &config).unwrap();

        assert_eq!(first.dataset_slices, second.dataset_slices);
        assert_eq!(first.eval_report, second.eval_report);
        assert!(!first.dataset_slices.is_empty());
    }

    #[test]
    fn at_wake_training_step1_d_leakage_guard_rejects_invalid_partitioning() {
        let examples = vec![
            WakeDatasetExample {
                example_id: "example_1".to_string(),
                user_id: Some("user_shared".to_string()),
                device_id: "device_a".to_string(),
                timestamp_ms: 1_000,
                label: WakeDatasetLabel::Positive,
                source_kind: WakeDatasetSourceKind::EnrollmentPass,
                score_bp: Some(9_100),
                reason_ref: None,
                extractability: WakeDatasetExtractability::RawPcmAvailable,
                latency_proxy_ms: Some(1_200),
            },
            WakeDatasetExample {
                example_id: "example_2".to_string(),
                user_id: Some("user_shared".to_string()),
                device_id: "device_b".to_string(),
                timestamp_ms: 1_020,
                label: WakeDatasetLabel::Negative,
                source_kind: WakeDatasetSourceKind::RuntimeRejected,
                score_bp: Some(4_000),
                reason_ref: Some("wake_reason_0x00001002".to_string()),
                extractability: WakeDatasetExtractability::MetadataOnly,
                latency_proxy_ms: Some(120),
            },
        ];
        let mut assignment = BTreeMap::new();
        assignment.insert(
            "example_1".to_string(),
            WakeTrainDatasetPartitionV1::Train,
        );
        assignment.insert(
            "example_2".to_string(),
            WakeTrainDatasetPartitionV1::Validation,
        );

        let report = evaluate_leakage_report(&examples, &assignment, 30_000, false);
        assert!(!report.no_user_leakage);
        assert!(enforce_leakage_guards(&report).is_err());
    }

    #[test]
    fn at_wake_training_step1_e_feature_extraction_is_reproducible() {
        let cfg = WakeFeatureConfigV1::locked_default_v1("wake_feature_cfg_v1".to_string()).unwrap();
        let mut fixture = Vec::new();
        for idx in 0..6_400u32 {
            let value = ((idx % 128) as i16 - 64) * 200;
            fixture.push(value);
        }

        let first = extract_log_mel_feature_bins(&cfg, &fixture).unwrap();
        let second = extract_log_mel_feature_bins(&cfg, &fixture).unwrap();
        assert_eq!(first, second);
        assert!(first.iter().flatten().any(|bin| *bin > 0));
    }

    #[test]
    fn at_wake_training_step2_a_dataset_assembly_is_deterministic() {
        let store = seeded_store();
        let request = valid_step2_request();
        let config = WakeTrainingStep2Config::default();

        let first = build_wake_training_step2(&store, &request, &config).unwrap();
        let second = build_wake_training_step2(&store, &request, &config).unwrap();

        assert_eq!(first.dataset_rows, second.dataset_rows);
        assert_eq!(first.dataset_slices, second.dataset_slices);
        assert_eq!(first.dataset_summary, second.dataset_summary);
        assert_eq!(first.threshold_calibration, second.threshold_calibration);
        assert_eq!(first.eval_report, second.eval_report);
        assert_eq!(first.wakepack_candidate.package_hash, second.wakepack_candidate.package_hash);
    }

    #[test]
    fn at_wake_training_step2_b_positive_negative_counts_are_stable() {
        let store = seeded_store();
        let request = valid_step2_request();
        let config = WakeTrainingStep2Config::default();

        let output = build_wake_training_step2(&store, &request, &config).unwrap();
        assert_eq!(output.dataset_summary.total_rows, 5);
        assert_eq!(output.dataset_summary.positive_rows, 3);
        assert_eq!(output.dataset_summary.negative_rows, 2);
        assert_eq!(
            output
                .dataset_summary
                .source_counts
                .get("ENROLLMENT_PASS")
                .copied(),
            Some(2)
        );
        assert_eq!(
            output
                .dataset_summary
                .source_counts
                .get("RUNTIME_ACCEPTED")
                .copied(),
            Some(1)
        );
        assert_eq!(
            output
                .dataset_summary
                .source_counts
                .get("RUNTIME_REJECTED")
                .copied(),
            Some(1)
        );
        assert_eq!(
            output
                .dataset_summary
                .source_counts
                .get("LEARN_SIGNAL")
                .copied(),
            Some(1)
        );
    }

    #[test]
    fn at_wake_training_step2_c_leakage_guard_catches_overlap() {
        let examples = vec![
            WakeDatasetExample {
                example_id: "example_1".to_string(),
                user_id: Some("user_shared".to_string()),
                device_id: "device_a".to_string(),
                timestamp_ms: 1_000,
                label: WakeDatasetLabel::Positive,
                source_kind: WakeDatasetSourceKind::EnrollmentPass,
                score_bp: Some(9_100),
                reason_ref: None,
                extractability: WakeDatasetExtractability::RawPcmAvailable,
                latency_proxy_ms: Some(1_200),
            },
            WakeDatasetExample {
                example_id: "example_2".to_string(),
                user_id: Some("user_shared".to_string()),
                device_id: "device_b".to_string(),
                timestamp_ms: 1_020,
                label: WakeDatasetLabel::Negative,
                source_kind: WakeDatasetSourceKind::RuntimeRejected,
                score_bp: Some(4_000),
                reason_ref: Some("wake_reason_0x00001002".to_string()),
                extractability: WakeDatasetExtractability::MetadataOnly,
                latency_proxy_ms: Some(120),
            },
        ];
        let mut assignment = BTreeMap::new();
        assignment.insert(
            "example_1".to_string(),
            WakeTrainDatasetPartitionV1::Train,
        );
        assignment.insert(
            "example_2".to_string(),
            WakeTrainDatasetPartitionV1::Validation,
        );

        let report = evaluate_leakage_report(&examples, &assignment, 30_000, false);
        assert!(!report.no_user_leakage);
        assert!(enforce_leakage_guards(&report).is_err());
    }

    #[test]
    fn at_wake_training_step2_d_threshold_calibration_is_stable() {
        let store = seeded_store();
        let mut examples = collect_dataset_examples(&store);
        examples.sort_by(|a, b| a.example_id.cmp(&b.example_id));

        let first = calibrate_threshold_profile(
            "wake_dataset_snapshot_v2",
            "wake_model_v2",
            "wake_feature_cfg_v1",
            &examples,
        );
        let second = calibrate_threshold_profile(
            "wake_dataset_snapshot_v2",
            "wake_model_v2",
            "wake_feature_cfg_v1",
            &examples,
        );
        assert_eq!(first.threshold_profile_id, second.threshold_profile_id);
        assert_eq!(first.calibrated_threshold_bp, second.calibrated_threshold_bp);
        assert!(first.calibrated_threshold_bp > 0);
    }

    #[test]
    fn at_wake_training_step2_e_eval_report_populated_and_valid() {
        let store = seeded_store();
        let request = valid_step2_request();
        let config = WakeTrainingStep2Config::default();
        let output = build_wake_training_step2(&store, &request, &config).unwrap();

        assert!(output.eval_report.validate().is_ok());
        assert_eq!(
            output.eval_report.threshold_profile_id,
            output.threshold_calibration.threshold_profile_id
        );
        assert!(output.eval_report.generated_at_ms > 0);
    }

    #[test]
    fn at_wake_training_step2_f_wakepack_candidate_and_manifest_validate() {
        let store = seeded_store();
        let request = valid_step2_request();
        let config = WakeTrainingStep2Config::default();
        let output = build_wake_training_step2(&store, &request, &config).unwrap();

        assert!(output.wake_pack_manifest.validate().is_ok());
        assert_eq!(output.wakepack_candidate.package_hash.len(), 64);
        assert!(output.wakepack_candidate.candidate_only);
        assert!(output.wake_pack_manifest.offline_pipeline_only);
    }

    #[test]
    fn at_wake_training_step2_g_offline_boundary_enforced() {
        let store = seeded_store();
        let mut request = valid_step2_request();
        request.offline_pipeline_only = false;
        let config = WakeTrainingStep2Config::default();
        let err = build_wake_training_step2(&store, &request, &config).unwrap_err();
        assert!(matches!(err, ContractViolation::InvalidValue { .. }));
    }
}
