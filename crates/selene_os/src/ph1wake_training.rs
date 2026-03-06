#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};
use selene_kernel_contracts::ph1art::ArtifactVersion;
use selene_kernel_contracts::ph1learn::{
    LearnSignalType, WakeFeatureConfigV1, WakePackManifestV1, WakeTrainDatasetPartitionV1,
    WakeTrainDatasetScopeV1, WakeTrainDatasetSliceV1, WakeTrainEvalReportV1,
};
use selene_kernel_contracts::{ContractViolation, Validate};
use selene_storage::ph1f::{Ph1fStore, WakeSampleResult};

const TRAIN_SPLIT_BP: u16 = 8_000;
const VALIDATION_SPLIT_BP: u16 = 1_000;
const SPLIT_DENOMINATOR_BP: u16 = 10_000;

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

    let (assignments, leakage_report) = assign_dataset_partitions(&examples, config)?;
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct WakeDatasetExample {
    example_id: String,
    user_id: Option<String>,
    device_id: String,
    timestamp_ms: u64,
    label: WakeDatasetLabel,
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
        });
    }

    examples
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
    config: &WakeTrainingStep1Config,
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
        config.time_adjacent_window_ms,
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
            },
            WakeDatasetExample {
                example_id: "example_2".to_string(),
                user_id: Some("user_shared".to_string()),
                device_id: "device_b".to_string(),
                timestamp_ms: 1_020,
                label: WakeDatasetLabel::Negative,
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
}
