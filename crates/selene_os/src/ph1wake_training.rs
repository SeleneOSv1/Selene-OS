#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};
use selene_kernel_contracts::ph1art::ArtifactVersion;
use selene_kernel_contracts::ph1j::DeviceId;
use selene_kernel_contracts::ph1learn::{
    LearnSignalType, WakeFeatureConfigV1, WakePackManifestV1, WakeTrainDatasetPartitionV1,
    WakeTrainDatasetScopeV1, WakeTrainDatasetSliceV1, WakeTrainEvalReportV1,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, ReasonCodeId, Validate};
use selene_storage::ph1f::{
    Ph1fStore, StorageError, WakeEnrollmentSampleRecord, WakePromotionCurrentRecord,
    WakePromotionState, WakeSampleResult,
};

const TRAIN_SPLIT_BP: u16 = 8_000;
const VALIDATION_SPLIT_BP: u16 = 1_000;
const SPLIT_DENOMINATOR_BP: u16 = 10_000;
const DEFAULT_CALIBRATION_THRESHOLD_BP: u16 = 8_000;
const DSCNN_DEPTHWISE_KERNEL_SIZE: usize = 3;
pub const WAKE_PROMOTION_REASON_GATE_PASS: ReasonCodeId = ReasonCodeId(0x57A1_4000);
pub const WAKE_PROMOTION_REASON_METRIC_NOT_MEASURED: ReasonCodeId = ReasonCodeId(0x57A1_4001);
pub const WAKE_PROMOTION_REASON_GATE_FAILED: ReasonCodeId = ReasonCodeId(0x57A1_4002);
pub const WAKE_PROMOTION_REASON_GATE_REQUIRES_ROLLBACK: ReasonCodeId = ReasonCodeId(0x57A1_4003);
pub const WAKE_PROMOTION_REASON_ROLLBACK_TARGET_MISSING: ReasonCodeId = ReasonCodeId(0x57A1_4004);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WakeTrainingStep3Config {
    pub offline_pipeline_only: bool,
    pub time_adjacent_window_ms: u64,
    pub default_latency_proxy_ms: u16,
    pub random_seed: u64,
    pub epoch_count: u16,
    pub learning_rate_milli: u16,
    pub hidden_channels: u16,
    pub target_frame_count: u16,
}

impl Default for WakeTrainingStep3Config {
    fn default() -> Self {
        Self {
            offline_pipeline_only: true,
            time_adjacent_window_ms: 30_000,
            default_latency_proxy_ms: 120,
            random_seed: 0x5E1E_C0A3_u64,
            epoch_count: 32,
            learning_rate_milli: 25,
            hidden_channels: 12,
            target_frame_count: 96,
        }
    }
}

impl Validate for WakeTrainingStep3Config {
    fn validate(&self) -> Result<(), ContractViolation> {
        if !self.offline_pipeline_only {
            return Err(ContractViolation::InvalidValue {
                field: "wake_training_step3_config.offline_pipeline_only",
                reason: "must be true",
            });
        }
        if self.time_adjacent_window_ms == 0 || self.time_adjacent_window_ms > 3_600_000 {
            return Err(ContractViolation::InvalidValue {
                field: "wake_training_step3_config.time_adjacent_window_ms",
                reason: "must be in [1, 3600000]",
            });
        }
        if self.default_latency_proxy_ms == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "wake_training_step3_config.default_latency_proxy_ms",
                reason: "must be > 0",
            });
        }
        if self.epoch_count == 0 || self.epoch_count > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "wake_training_step3_config.epoch_count",
                reason: "must be in [1, 512]",
            });
        }
        if self.learning_rate_milli == 0 || self.learning_rate_milli > 5_000 {
            return Err(ContractViolation::InvalidValue {
                field: "wake_training_step3_config.learning_rate_milli",
                reason: "must be in [1, 5000]",
            });
        }
        if self.hidden_channels == 0 || self.hidden_channels > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "wake_training_step3_config.hidden_channels",
                reason: "must be in [1, 128]",
            });
        }
        if self.target_frame_count < 8 || self.target_frame_count > 1_024 {
            return Err(ContractViolation::InvalidValue {
                field: "wake_training_step3_config.target_frame_count",
                reason: "must be in [8, 1024]",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeTrainingStep3Request {
    pub dataset_snapshot_id: String,
    pub feature_config_id: String,
    pub model_version: String,
    pub model_abi: String,
    pub artifact_version: ArtifactVersion,
    pub provenance_ref: String,
    pub rollback_to_artifact_version: Option<ArtifactVersion>,
    pub offline_pipeline_only: bool,
    pub pcm_by_ref: BTreeMap<String, Vec<i16>>,
}

impl WakeTrainingStep3Request {
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
        pcm_by_ref: BTreeMap<String, Vec<i16>>,
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
            pcm_by_ref,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for WakeTrainingStep3Request {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token(
            "wake_training_step3_request.dataset_snapshot_id",
            &self.dataset_snapshot_id,
            128,
        )?;
        validate_token(
            "wake_training_step3_request.feature_config_id",
            &self.feature_config_id,
            96,
        )?;
        validate_token(
            "wake_training_step3_request.model_version",
            &self.model_version,
            64,
        )?;
        validate_token("wake_training_step3_request.model_abi", &self.model_abi, 64)?;
        self.artifact_version.validate()?;
        validate_token(
            "wake_training_step3_request.provenance_ref",
            &self.provenance_ref,
            128,
        )?;
        if let Some(rollback) = self.rollback_to_artifact_version {
            rollback.validate()?;
            if rollback >= self.artifact_version {
                return Err(ContractViolation::InvalidValue {
                    field: "wake_training_step3_request.rollback_to_artifact_version",
                    reason: "must be < artifact_version when present",
                });
            }
        }
        if !self.offline_pipeline_only {
            return Err(ContractViolation::InvalidValue {
                field: "wake_training_step3_request.offline_pipeline_only",
                reason: "must be true",
            });
        }
        for (pcm_ref, pcm) in &self.pcm_by_ref {
            validate_token("wake_training_step3_request.pcm_by_ref.key", pcm_ref, 256)?;
            if pcm.is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "wake_training_step3_request.pcm_by_ref.value",
                    reason: "pcm vectors must be non-empty",
                });
            }
            if pcm.len() > 320_000 {
                return Err(ContractViolation::InvalidValue {
                    field: "wake_training_step3_request.pcm_by_ref.value",
                    reason: "pcm vectors must be <= 320000 samples",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeFeatureTensorRowV1 {
    pub row_id: String,
    pub example_id: String,
    pub source_kind: String,
    pub label: String,
    pub user_id: Option<String>,
    pub device_id: String,
    pub platform: Option<String>,
    pub wake_window_id: Option<String>,
    pub partition: WakeTrainDatasetPartitionV1,
    pub frame_count: u16,
    pub mel_bins: u16,
    pub tensor_bins: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeTensorBuildSummaryV1 {
    pub total_dataset_rows: u32,
    pub extractable_rows: u32,
    pub tensor_row_count: u32,
    pub excluded_non_extractable_rows: u32,
    pub excluded_missing_pcm_rows: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeDsCnnTrainingSummaryV1 {
    pub random_seed: u64,
    pub epoch_count: u16,
    pub learning_rate_milli: u16,
    pub hidden_channels: u16,
    pub train_example_count: u32,
    pub validation_example_count: u32,
    pub test_example_count: u32,
    pub final_train_loss_milli: u32,
    pub validation_loss_milli: u32,
    pub test_accuracy_bp: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeTrainingStep3Output {
    pub feature_config: WakeFeatureConfigV1,
    pub dataset_rows: Vec<WakeDatasetRowV1>,
    pub dataset_slices: Vec<WakeTrainDatasetSliceV1>,
    pub dataset_summary: WakeDatasetAssemblySummaryV1,
    pub tensor_rows: Vec<WakeFeatureTensorRowV1>,
    pub tensor_summary: WakeTensorBuildSummaryV1,
    pub training_summary: WakeDsCnnTrainingSummaryV1,
    pub threshold_calibration: WakeThresholdCalibrationV1,
    pub eval_report: WakeTrainEvalReportV1,
    pub wake_pack_manifest: WakePackManifestV1,
    pub wakepack_candidate: WakeCandidatePackageV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakePromotionGateOutcomeV1 {
    PassToShadow,
    PassToCanary,
    PassToActive,
    Block,
    RequireRollback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakePromotionGateDecisionV1 {
    pub outcome: WakePromotionGateOutcomeV1,
    pub reason_code: ReasonCodeId,
    pub target_state: WakePromotionState,
    pub not_measured_metrics_ref: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WakePromotionGatePolicyV1 {
    pub require_measured_metrics: bool,
    pub max_far_per_listening_hour_milli: u32,
    pub max_frr_bp: u16,
    pub max_miss_rate_bp: u16,
    pub max_latency_proxy_ms: u16,
    pub max_threshold_calibration_error_bp: u16,
    pub rollback_far_per_listening_hour_milli: u32,
    pub rollback_frr_bp: u16,
    pub rollback_miss_rate_bp: u16,
    pub rollback_latency_proxy_ms: u16,
    pub rollback_threshold_calibration_error_bp: u16,
}

impl Default for WakePromotionGatePolicyV1 {
    fn default() -> Self {
        Self {
            require_measured_metrics: true,
            max_far_per_listening_hour_milli: 250,
            max_frr_bp: 1_500,
            max_miss_rate_bp: 1_500,
            max_latency_proxy_ms: 350,
            max_threshold_calibration_error_bp: 1_000,
            rollback_far_per_listening_hour_milli: 500,
            rollback_frr_bp: 3_000,
            rollback_miss_rate_bp: 3_000,
            rollback_latency_proxy_ms: 600,
            rollback_threshold_calibration_error_bp: 2_000,
        }
    }
}

impl Validate for WakePromotionGatePolicyV1 {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.max_far_per_listening_hour_milli == 0
            || self.max_frr_bp == 0
            || self.max_miss_rate_bp == 0
            || self.max_latency_proxy_ms == 0
            || self.max_threshold_calibration_error_bp == 0
        {
            return Err(ContractViolation::InvalidValue {
                field: "wake_promotion_gate_policy_v1",
                reason: "all max thresholds must be > 0",
            });
        }
        if self.max_frr_bp > 10_000
            || self.max_miss_rate_bp > 10_000
            || self.max_threshold_calibration_error_bp > 10_000
            || self.rollback_frr_bp > 10_000
            || self.rollback_miss_rate_bp > 10_000
            || self.rollback_threshold_calibration_error_bp > 10_000
        {
            return Err(ContractViolation::InvalidValue {
                field: "wake_promotion_gate_policy_v1",
                reason: "basis-point thresholds must be <= 10000",
            });
        }
        if self.rollback_far_per_listening_hour_milli < self.max_far_per_listening_hour_milli
            || self.rollback_frr_bp < self.max_frr_bp
            || self.rollback_miss_rate_bp < self.max_miss_rate_bp
            || self.rollback_latency_proxy_ms < self.max_latency_proxy_ms
            || self.rollback_threshold_calibration_error_bp < self.max_threshold_calibration_error_bp
        {
            return Err(ContractViolation::InvalidValue {
                field: "wake_promotion_gate_policy_v1.rollback_thresholds",
                reason: "rollback thresholds must be >= max thresholds",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeRollbackDrillResultV1 {
    pub rollback_ready: bool,
    pub candidate_artifact_version: ArtifactVersion,
    pub rollback_target_artifact_version: Option<ArtifactVersion>,
    pub active_artifact_version: Option<ArtifactVersion>,
    pub last_known_good_artifact_version: Option<ArtifactVersion>,
    pub reason_code: Option<ReasonCodeId>,
}

pub fn evaluate_wake_promotion_gate(
    eval_report: &WakeTrainEvalReportV1,
    target_state: WakePromotionState,
    policy: &WakePromotionGatePolicyV1,
) -> Result<WakePromotionGateDecisionV1, ContractViolation> {
    eval_report.validate()?;
    policy.validate()?;
    if !matches!(
        target_state,
        WakePromotionState::Shadow | WakePromotionState::Canary | WakePromotionState::Active
    ) {
        return Err(ContractViolation::InvalidValue {
            field: "evaluate_wake_promotion_gate.target_state",
            reason: "must be SHADOW/CANARY/ACTIVE",
        });
    }

    if policy.require_measured_metrics && eval_report.not_measured_metrics_ref.is_some() {
        return Ok(WakePromotionGateDecisionV1 {
            outcome: WakePromotionGateOutcomeV1::Block,
            reason_code: WAKE_PROMOTION_REASON_METRIC_NOT_MEASURED,
            target_state,
            not_measured_metrics_ref: eval_report.not_measured_metrics_ref.clone(),
        });
    }

    let rollback_threshold_breached = eval_report.far_per_listening_hour_milli
        > policy.rollback_far_per_listening_hour_milli
        || eval_report.frr_bp > policy.rollback_frr_bp
        || eval_report.miss_rate_bp > policy.rollback_miss_rate_bp
        || eval_report.latency_proxy_ms > policy.rollback_latency_proxy_ms
        || eval_report.threshold_calibration_error_bp
            > policy.rollback_threshold_calibration_error_bp;
    if target_state == WakePromotionState::Active && rollback_threshold_breached {
        return Ok(WakePromotionGateDecisionV1 {
            outcome: WakePromotionGateOutcomeV1::RequireRollback,
            reason_code: WAKE_PROMOTION_REASON_GATE_REQUIRES_ROLLBACK,
            target_state,
            not_measured_metrics_ref: eval_report.not_measured_metrics_ref.clone(),
        });
    }

    let max_threshold_breached = eval_report.far_per_listening_hour_milli
        > policy.max_far_per_listening_hour_milli
        || eval_report.frr_bp > policy.max_frr_bp
        || eval_report.miss_rate_bp > policy.max_miss_rate_bp
        || eval_report.latency_proxy_ms > policy.max_latency_proxy_ms
        || eval_report.threshold_calibration_error_bp > policy.max_threshold_calibration_error_bp;
    if max_threshold_breached {
        return Ok(WakePromotionGateDecisionV1 {
            outcome: WakePromotionGateOutcomeV1::Block,
            reason_code: WAKE_PROMOTION_REASON_GATE_FAILED,
            target_state,
            not_measured_metrics_ref: eval_report.not_measured_metrics_ref.clone(),
        });
    }

    let outcome = match target_state {
        WakePromotionState::Shadow => WakePromotionGateOutcomeV1::PassToShadow,
        WakePromotionState::Canary => WakePromotionGateOutcomeV1::PassToCanary,
        WakePromotionState::Active => WakePromotionGateOutcomeV1::PassToActive,
        _ => unreachable!("validated target_state"),
    };
    Ok(WakePromotionGateDecisionV1 {
        outcome,
        reason_code: WAKE_PROMOTION_REASON_GATE_PASS,
        target_state,
        not_measured_metrics_ref: eval_report.not_measured_metrics_ref.clone(),
    })
}

pub fn wake_promote_candidate_to_shadow(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    manifest: &WakePackManifestV1,
    cohort_assignment_ref: Option<String>,
    policy: &WakePromotionGatePolicyV1,
    decision_ref: String,
    idempotency_key: String,
) -> Result<WakePromotionCurrentRecord, StorageError> {
    manifest.validate().map_err(StorageError::ContractViolation)?;
    let gate = evaluate_wake_promotion_gate(
        &manifest.eval_metrics_summary,
        WakePromotionState::Shadow,
        policy,
    )
    .map_err(StorageError::ContractViolation)?;
    if gate.outcome != WakePromotionGateOutcomeV1::PassToShadow {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "wake_promote_candidate_to_shadow.gate",
                reason: "gate must PASS_TO_SHADOW",
            },
        ));
    }
    store.wake_promotion_transition_commit(
        now,
        manifest.artifact_version,
        WakePromotionState::Shadow,
        Some(now),
        cohort_assignment_ref,
        None,
        None,
        decision_ref,
        idempotency_key,
        false,
    )
}

pub fn wake_promote_shadow_to_canary(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    manifest: &WakePackManifestV1,
    cohort_assignment_ref: Option<String>,
    policy: &WakePromotionGatePolicyV1,
    decision_ref: String,
    idempotency_key: String,
) -> Result<WakePromotionCurrentRecord, StorageError> {
    manifest.validate().map_err(StorageError::ContractViolation)?;
    let gate = evaluate_wake_promotion_gate(
        &manifest.eval_metrics_summary,
        WakePromotionState::Canary,
        policy,
    )
    .map_err(StorageError::ContractViolation)?;
    if gate.outcome != WakePromotionGateOutcomeV1::PassToCanary {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "wake_promote_shadow_to_canary.gate",
                reason: "gate must PASS_TO_CANARY",
            },
        ));
    }
    store.wake_promotion_transition_commit(
        now,
        manifest.artifact_version,
        WakePromotionState::Canary,
        Some(now),
        cohort_assignment_ref,
        None,
        None,
        decision_ref,
        idempotency_key,
        false,
    )
}

pub fn wake_promote_canary_to_active(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    manifest: &WakePackManifestV1,
    policy: &WakePromotionGatePolicyV1,
    rollout_device_ids: &[DeviceId],
    decision_ref: String,
    idempotency_key: String,
) -> Result<WakePromotionCurrentRecord, StorageError> {
    manifest.validate().map_err(StorageError::ContractViolation)?;
    let gate = evaluate_wake_promotion_gate(
        &manifest.eval_metrics_summary,
        WakePromotionState::Active,
        policy,
    )
    .map_err(StorageError::ContractViolation)?;
    if gate.outcome != WakePromotionGateOutcomeV1::PassToActive {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "wake_promote_canary_to_active.gate",
                reason: "gate must PASS_TO_ACTIVE",
            },
        ));
    }
    for device_id in rollout_device_ids {
        let drill = run_wake_rollback_drill(store, device_id, manifest.artifact_version)?;
        if !drill.rollback_ready {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "wake_promote_canary_to_active.rollback_drill",
                    reason: "last_known_good rollback drill must pass for all rollout devices",
                },
            ));
        }
    }
    store.wake_promotion_transition_commit(
        now,
        manifest.artifact_version,
        WakePromotionState::Active,
        Some(now),
        None,
        None,
        None,
        decision_ref,
        idempotency_key,
        true,
    )
}

pub fn wake_block_candidate_version(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    artifact_version: ArtifactVersion,
    blocked_reason_code: ReasonCodeId,
    decision_ref: String,
    idempotency_key: String,
) -> Result<WakePromotionCurrentRecord, StorageError> {
    store.wake_promotion_transition_commit(
        now,
        artifact_version,
        WakePromotionState::Blocked,
        None,
        None,
        Some(blocked_reason_code),
        None,
        decision_ref,
        idempotency_key,
        false,
    )
}

pub fn wake_rollback_active_version(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    artifact_version: ArtifactVersion,
    rollback_reason_code: ReasonCodeId,
    decision_ref: String,
    idempotency_key: String,
) -> Result<WakePromotionCurrentRecord, StorageError> {
    store.wake_promotion_transition_commit(
        now,
        artifact_version,
        WakePromotionState::RolledBack,
        None,
        None,
        None,
        Some(rollback_reason_code),
        decision_ref,
        idempotency_key,
        false,
    )
}

pub fn run_wake_rollback_drill(
    store: &Ph1fStore,
    device_id: &DeviceId,
    candidate_artifact_version: ArtifactVersion,
) -> Result<WakeRollbackDrillResultV1, StorageError> {
    candidate_artifact_version
        .validate()
        .map_err(StorageError::ContractViolation)?;
    let Some(current) = store.wake_artifact_apply_current_row(device_id) else {
        return Ok(WakeRollbackDrillResultV1 {
            rollback_ready: false,
            candidate_artifact_version,
            rollback_target_artifact_version: None,
            active_artifact_version: None,
            last_known_good_artifact_version: None,
            reason_code: Some(WAKE_PROMOTION_REASON_ROLLBACK_TARGET_MISSING),
        });
    };
    let rollback_target = current
        .last_known_good_artifact_version
        .or(current.active_artifact_version);
    let rollback_ready = rollback_target.is_some()
        && rollback_target != Some(candidate_artifact_version)
        && current.last_known_good_artifact_version.is_some();
    Ok(WakeRollbackDrillResultV1 {
        rollback_ready,
        candidate_artifact_version,
        rollback_target_artifact_version: rollback_target,
        active_artifact_version: current.active_artifact_version,
        last_known_good_artifact_version: current.last_known_good_artifact_version,
        reason_code: if rollback_ready {
            None
        } else {
            Some(WAKE_PROMOTION_REASON_ROLLBACK_TARGET_MISSING)
        },
    })
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

pub fn build_wake_training_step3(
    store: &Ph1fStore,
    request: &WakeTrainingStep3Request,
    config: &WakeTrainingStep3Config,
) -> Result<WakeTrainingStep3Output, ContractViolation> {
    request.validate()?;
    config.validate()?;
    if !request.offline_pipeline_only || !config.offline_pipeline_only {
        return Err(ContractViolation::InvalidValue {
            field: "wake_training_step3.offline_pipeline_only",
            reason: "runtime path must be OFFLINE_PIPELINE_ONLY",
        });
    }

    let feature_config = WakeFeatureConfigV1::locked_default_v1(request.feature_config_id.clone())?;
    let mut examples = collect_dataset_examples(store);
    if examples.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field: "wake_training_step3.dataset_examples",
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
            field: "wake_training_step3.dataset_slices",
            reason: "must produce at least one dataset slice",
        });
    }

    let dataset_rows = convert_to_dataset_rows(&examples);
    let dataset_summary = build_dataset_summary(&examples, &leakage_report);
    let (tensor_rows, tensor_summary) = build_feature_tensors_for_step3(
        &examples,
        &assignments,
        &feature_config,
        config.target_frame_count,
        &request.pcm_by_ref,
    )?;
    if tensor_rows.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field: "wake_training_step3.tensor_rows",
            reason: "must include at least one extractable tensor row",
        });
    }

    let (model, training_summary) = train_dscnn_baseline(&tensor_rows, config)?;
    let example_meta_by_id: BTreeMap<String, (Option<String>, Option<u16>)> = examples
        .iter()
        .map(|example| {
            (
                example.example_id.clone(),
                (example.reason_ref.clone(), example.latency_proxy_ms),
            )
        })
        .collect();
    let scored_examples = score_dscnn_model(&model, &tensor_rows, &example_meta_by_id)?;
    let threshold_calibration = calibrate_threshold_profile_from_model_scores(
        request.dataset_snapshot_id.as_str(),
        request.model_version.as_str(),
        feature_config.feature_config_id.as_str(),
        &scored_examples,
    );

    let eval_report = build_eval_report_step3(
        request,
        &tensor_rows,
        &scored_examples,
        &threshold_calibration,
        &dataset_slices,
        config.default_latency_proxy_ms,
    )?;

    let wakepack_candidate = build_wakepack_real_package(
        request,
        &feature_config,
        &threshold_calibration,
        &eval_report,
        &training_summary,
        &dataset_summary,
        &tensor_summary,
        &model,
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

    Ok(WakeTrainingStep3Output {
        feature_config,
        dataset_rows,
        dataset_slices,
        dataset_summary,
        tensor_rows,
        tensor_summary,
        training_summary,
        threshold_calibration,
        eval_report,
        wake_pack_manifest,
        wakepack_candidate,
    })
}

#[derive(Debug, Clone)]
struct DsCnnModel {
    frame_count: usize,
    mel_bins: usize,
    hidden_channels: usize,
    depthwise_kernel: Vec<[f32; DSCNN_DEPTHWISE_KERNEL_SIZE]>,
    depthwise_bias: Vec<f32>,
    pointwise_kernel: Vec<Vec<f32>>,
    pointwise_bias: Vec<f32>,
    output_kernel: Vec<f32>,
    output_bias: f32,
}

#[derive(Debug, Clone)]
struct DsCnnForwardCache {
    depth_linear: Vec<f32>,
    depth_act: Vec<f32>,
    point_linear: Vec<f32>,
    pooled: Vec<f32>,
    score: f32,
}

#[derive(Debug, Clone)]
struct DsCnnScoredExample {
    example_id: String,
    partition: WakeTrainDatasetPartitionV1,
    label_positive: bool,
    score_bp: u16,
    reason_ref: Option<String>,
    latency_proxy_ms: Option<u16>,
}

fn build_feature_tensors_for_step3(
    examples: &[WakeDatasetExample],
    assignments: &BTreeMap<String, WakeTrainDatasetPartitionV1>,
    feature_config: &WakeFeatureConfigV1,
    target_frame_count: u16,
    pcm_by_ref: &BTreeMap<String, Vec<i16>>,
) -> Result<(Vec<WakeFeatureTensorRowV1>, WakeTensorBuildSummaryV1), ContractViolation> {
    feature_config.validate()?;
    let mut tensor_rows = Vec::new();
    let mut extractable_rows = 0u32;
    let mut excluded_non_extractable_rows = 0u32;
    let mut excluded_missing_pcm_rows = 0u32;
    let target_frame_count_usize = target_frame_count as usize;
    let mel_bins = feature_config.mel_bins as usize;

    for example in examples {
        let Some(partition) = assignments.get(example.example_id.as_str()).copied() else {
            continue;
        };
        if !matches!(example.extractability, WakeDatasetExtractability::RawPcmAvailable) {
            excluded_non_extractable_rows = excluded_non_extractable_rows.saturating_add(1);
            continue;
        }
        extractable_rows = extractable_rows.saturating_add(1);
        let Some(pcm_ref) = example.extractable_pcm_ref.as_ref() else {
            excluded_missing_pcm_rows = excluded_missing_pcm_rows.saturating_add(1);
            continue;
        };
        let Some(pcm) = pcm_by_ref.get(pcm_ref.as_str()) else {
            excluded_missing_pcm_rows = excluded_missing_pcm_rows.saturating_add(1);
            continue;
        };
        let bins = extract_log_mel_feature_bins(feature_config, pcm)?;
        let normalized_bins =
            normalize_feature_bins_frame_count(&bins, target_frame_count_usize, mel_bins);
        tensor_rows.push(WakeFeatureTensorRowV1 {
            row_id: format!(
                "tensor_{}",
                stable_hex_short(
                    format!(
                        "{}:{}:{}",
                        example.example_id, target_frame_count, feature_config.feature_config_id
                    )
                    .as_bytes()
                )
            ),
            example_id: example.example_id.clone(),
            source_kind: source_kind_token(example.source_kind).to_string(),
            label: label_token(example.label).to_string(),
            user_id: example.user_id.clone(),
            device_id: example.device_id.clone(),
            platform: example.platform.clone(),
            wake_window_id: example.wake_window_id.clone(),
            partition,
            frame_count: target_frame_count,
            mel_bins: feature_config.mel_bins,
            tensor_bins: normalized_bins,
        });
    }

    tensor_rows.sort_by(|a, b| a.row_id.cmp(&b.row_id));
    let summary = WakeTensorBuildSummaryV1 {
        total_dataset_rows: examples.len() as u32,
        extractable_rows,
        tensor_row_count: tensor_rows.len() as u32,
        excluded_non_extractable_rows,
        excluded_missing_pcm_rows,
    };
    Ok((tensor_rows, summary))
}

fn normalize_feature_bins_frame_count(
    bins: &[Vec<u16>],
    target_frame_count: usize,
    mel_bins: usize,
) -> Vec<u16> {
    let mut out = vec![0u16; target_frame_count.saturating_mul(mel_bins)];
    for (frame_idx, frame) in bins.iter().take(target_frame_count).enumerate() {
        for (mel_idx, bin) in frame.iter().take(mel_bins).enumerate() {
            out[frame_idx * mel_bins + mel_idx] = *bin;
        }
    }
    out
}

fn train_dscnn_baseline(
    tensor_rows: &[WakeFeatureTensorRowV1],
    config: &WakeTrainingStep3Config,
) -> Result<(DsCnnModel, WakeDsCnnTrainingSummaryV1), ContractViolation> {
    if tensor_rows.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field: "train_dscnn_baseline.tensor_rows",
            reason: "must be non-empty",
        });
    }
    let frame_count = tensor_rows[0].frame_count as usize;
    let mel_bins = tensor_rows[0].mel_bins as usize;
    let hidden_channels = config.hidden_channels as usize;
    if frame_count == 0 || mel_bins == 0 {
        return Err(ContractViolation::InvalidValue {
            field: "train_dscnn_baseline.shape",
            reason: "frame_count and mel_bins must be > 0",
        });
    }
    for row in tensor_rows {
        if row.frame_count as usize != frame_count
            || row.mel_bins as usize != mel_bins
            || row.tensor_bins.len() != frame_count.saturating_mul(mel_bins)
        {
            return Err(ContractViolation::InvalidValue {
                field: "train_dscnn_baseline.tensor_rows",
                reason: "all rows must share the same tensor shape",
            });
        }
    }

    let mut train_rows = Vec::new();
    let mut validation_rows = Vec::new();
    let mut test_rows = Vec::new();
    for row in tensor_rows {
        match row.partition {
            WakeTrainDatasetPartitionV1::Train => train_rows.push(row),
            WakeTrainDatasetPartitionV1::Validation => validation_rows.push(row),
            WakeTrainDatasetPartitionV1::Test => test_rows.push(row),
        }
    }
    if train_rows.is_empty() || !contains_both_labels(&train_rows) {
        for fallback in validation_rows.iter().chain(test_rows.iter()) {
            if !train_rows.iter().any(|row| row.row_id == fallback.row_id) {
                train_rows.push(*fallback);
            }
            if contains_both_labels(&train_rows) {
                break;
            }
        }
    }
    if train_rows.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field: "train_dscnn_baseline.train_rows",
            reason: "must include at least one TRAIN tensor row",
        });
    }
    if !contains_both_labels(&train_rows) {
        return Err(ContractViolation::InvalidValue {
            field: "train_dscnn_baseline.train_rows",
            reason: "TRAIN rows (after deterministic fallback) must include both POSITIVE and NEGATIVE labels",
        });
    }

    let mut seed = config.random_seed;
    let mut model = DsCnnModel {
        frame_count,
        mel_bins,
        hidden_channels,
        depthwise_kernel: (0..mel_bins)
            .map(|_| {
                [
                    seeded_weight(&mut seed, 0.08),
                    seeded_weight(&mut seed, 0.08),
                    seeded_weight(&mut seed, 0.08),
                ]
            })
            .collect(),
        depthwise_bias: (0..mel_bins).map(|_| seeded_weight(&mut seed, 0.02)).collect(),
        pointwise_kernel: (0..hidden_channels)
            .map(|_| (0..mel_bins).map(|_| seeded_weight(&mut seed, 0.05)).collect())
            .collect(),
        pointwise_bias: (0..hidden_channels).map(|_| seeded_weight(&mut seed, 0.02)).collect(),
        output_kernel: (0..hidden_channels)
            .map(|_| seeded_weight(&mut seed, 0.05))
            .collect(),
        output_bias: seeded_weight(&mut seed, 0.02),
    };

    let mut final_train_loss = 0.0f32;
    let lr = config.learning_rate_milli as f32 / 1000.0;
    for _epoch in 0..config.epoch_count {
        let mut grad_depthwise_kernel = vec![[0.0f32; DSCNN_DEPTHWISE_KERNEL_SIZE]; mel_bins];
        let mut grad_depthwise_bias = vec![0.0f32; mel_bins];
        let mut grad_pointwise_kernel = vec![vec![0.0f32; mel_bins]; hidden_channels];
        let mut grad_pointwise_bias = vec![0.0f32; hidden_channels];
        let mut grad_output_kernel = vec![0.0f32; hidden_channels];
        let mut grad_output_bias = 0.0f32;
        let mut total_loss = 0.0f32;

        for row in &train_rows {
            let input = tensor_bins_as_f32(row.tensor_bins.as_slice());
            let y = label_token_is_positive(row.label.as_str()) as u8 as f32;
            let forward = dscnn_forward(&model, input.as_slice());
            let score = forward.score.clamp(1e-6, 1.0 - 1e-6);
            total_loss += -(y * score.ln() + (1.0 - y) * (1.0 - score).ln());
            let dlogit = score - y;

            for h in 0..hidden_channels {
                grad_output_kernel[h] += dlogit * forward.pooled[h];
            }
            grad_output_bias += dlogit;

            let mut ddepth_act = vec![0.0f32; frame_count * mel_bins];
            for h in 0..hidden_channels {
                let dpooled = dlogit * model.output_kernel[h];
                for t in 0..frame_count {
                    let pidx = point_idx(h, t, frame_count);
                    let mut dpoint_linear = dpooled / frame_count as f32;
                    if forward.point_linear[pidx] <= 0.0 {
                        dpoint_linear = 0.0;
                    }
                    grad_pointwise_bias[h] += dpoint_linear;
                    for c in 0..mel_bins {
                        let didx = depth_idx(c, t, frame_count);
                        grad_pointwise_kernel[h][c] += dpoint_linear * forward.depth_act[didx];
                        ddepth_act[didx] += dpoint_linear * model.pointwise_kernel[h][c];
                    }
                }
            }

            for c in 0..mel_bins {
                for t in 0..frame_count {
                    let didx = depth_idx(c, t, frame_count);
                    let mut ddepth_linear = ddepth_act[didx];
                    if forward.depth_linear[didx] <= 0.0 {
                        ddepth_linear = 0.0;
                    }
                    grad_depthwise_bias[c] += ddepth_linear;
                    for k in 0..DSCNN_DEPTHWISE_KERNEL_SIZE {
                        let src_t = t as isize + k as isize - 1;
                        if src_t < 0 || src_t >= frame_count as isize {
                            continue;
                        }
                        let src_idx = src_t as usize * mel_bins + c;
                        grad_depthwise_kernel[c][k] += ddepth_linear * input[src_idx];
                    }
                }
            }
        }

        final_train_loss = total_loss / train_rows.len() as f32;
        let inv_n = 1.0f32 / train_rows.len() as f32;
        for c in 0..mel_bins {
            for k in 0..DSCNN_DEPTHWISE_KERNEL_SIZE {
                model.depthwise_kernel[c][k] -= lr * grad_depthwise_kernel[c][k] * inv_n;
            }
            model.depthwise_bias[c] -= lr * grad_depthwise_bias[c] * inv_n;
        }
        for h in 0..hidden_channels {
            for c in 0..mel_bins {
                model.pointwise_kernel[h][c] -= lr * grad_pointwise_kernel[h][c] * inv_n;
            }
            model.pointwise_bias[h] -= lr * grad_pointwise_bias[h] * inv_n;
            model.output_kernel[h] -= lr * grad_output_kernel[h] * inv_n;
        }
        model.output_bias -= lr * grad_output_bias * inv_n;
    }

    let validation_loss = average_partition_loss(&model, &validation_rows);
    let (test_loss, test_accuracy_bp) = partition_loss_and_accuracy(&model, &test_rows);
    let summary = WakeDsCnnTrainingSummaryV1 {
        random_seed: config.random_seed,
        epoch_count: config.epoch_count,
        learning_rate_milli: config.learning_rate_milli,
        hidden_channels: config.hidden_channels,
        train_example_count: train_rows.len() as u32,
        validation_example_count: validation_rows.len() as u32,
        test_example_count: test_rows.len() as u32,
        final_train_loss_milli: (final_train_loss.max(0.0) * 1000.0).round() as u32,
        validation_loss_milli: (validation_loss.max(0.0) * 1000.0).round() as u32,
        test_accuracy_bp,
    };
    let _ = test_loss;
    Ok((model, summary))
}

fn score_dscnn_model(
    model: &DsCnnModel,
    tensor_rows: &[WakeFeatureTensorRowV1],
    example_meta_by_id: &BTreeMap<String, (Option<String>, Option<u16>)>,
) -> Result<Vec<DsCnnScoredExample>, ContractViolation> {
    let mut out = Vec::new();
    for row in tensor_rows {
        if row.tensor_bins.len() != model.frame_count.saturating_mul(model.mel_bins) {
            return Err(ContractViolation::InvalidValue {
                field: "score_dscnn_model.tensor_bins",
                reason: "tensor shape mismatch",
            });
        }
        let forward = dscnn_forward(model, tensor_bins_as_f32(row.tensor_bins.as_slice()).as_slice());
        let (reason_ref, latency_proxy_ms) = example_meta_by_id
            .get(row.example_id.as_str())
            .cloned()
            .unwrap_or((None, None));
        out.push(DsCnnScoredExample {
            example_id: row.example_id.clone(),
            partition: row.partition,
            label_positive: label_token_is_positive(row.label.as_str()),
            score_bp: (forward.score * 10_000.0).round().clamp(0.0, 10_000.0) as u16,
            reason_ref,
            latency_proxy_ms,
        });
    }
    out.sort_by(|a, b| a.example_id.cmp(&b.example_id));
    Ok(out)
}

fn calibrate_threshold_profile_from_model_scores(
    dataset_snapshot_id: &str,
    model_version: &str,
    feature_config_id: &str,
    scored_examples: &[DsCnnScoredExample],
) -> WakeThresholdCalibrationV1 {
    let mut calibration_examples = scored_examples
        .iter()
        .filter(|row| row.partition == WakeTrainDatasetPartitionV1::Validation)
        .collect::<Vec<_>>();
    if !contains_both_labels_scored(&calibration_examples) {
        calibration_examples = scored_examples.iter().collect::<Vec<_>>();
    }

    let mut positive_scores = calibration_examples
        .iter()
        .filter(|row| row.label_positive)
        .map(|row| row.score_bp)
        .collect::<Vec<_>>();
    let mut negative_scores = calibration_examples
        .iter()
        .filter(|row| !row.label_positive)
        .map(|row| row.score_bp)
        .collect::<Vec<_>>();
    positive_scores.sort_unstable();
    negative_scores.sort_unstable();

    let mut not_measured_metrics = Vec::new();
    let (calibrated_threshold_bp, calibration_error_bp) = if !positive_scores.is_empty()
        && !negative_scores.is_empty()
    {
        let mut candidates = BTreeSet::new();
        candidates.insert(0u16);
        candidates.insert(10_000u16);
        for score in positive_scores.iter().chain(negative_scores.iter()) {
            candidates.insert(*score);
        }
        let mut best_threshold = DEFAULT_CALIBRATION_THRESHOLD_BP;
        let mut best_errors = u32::MAX;
        for candidate in candidates {
            let false_reject = positive_scores
                .iter()
                .filter(|score| **score < candidate)
                .count() as u32;
            let false_accept = negative_scores
                .iter()
                .filter(|score| **score >= candidate)
                .count() as u32;
            let errors = false_reject.saturating_add(false_accept);
            if errors < best_errors
                || (errors == best_errors
                    && abs_u16_diff(candidate, DEFAULT_CALIBRATION_THRESHOLD_BP)
                        < abs_u16_diff(best_threshold, DEFAULT_CALIBRATION_THRESHOLD_BP))
            {
                best_errors = errors;
                best_threshold = candidate;
            }
        }
        let total = (positive_scores.len() + negative_scores.len()) as u32;
        let error_bp = best_errors.saturating_mul(10_000).saturating_div(total.max(1)) as u16;
        (best_threshold, error_bp)
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

fn build_eval_report_step3(
    request: &WakeTrainingStep3Request,
    tensor_rows: &[WakeFeatureTensorRowV1],
    scored_examples: &[DsCnnScoredExample],
    threshold_calibration: &WakeThresholdCalibrationV1,
    dataset_slices: &[WakeTrainDatasetSliceV1],
    default_latency_proxy_ms: u16,
) -> Result<WakeTrainEvalReportV1, ContractViolation> {
    let mut eval_rows = scored_examples
        .iter()
        .filter(|row| row.partition == WakeTrainDatasetPartitionV1::Test)
        .collect::<Vec<_>>();
    if !contains_both_labels_scored(&eval_rows) {
        eval_rows = scored_examples.iter().collect::<Vec<_>>();
    }
    let positives = eval_rows.iter().filter(|row| row.label_positive).count() as u32;
    let negatives = eval_rows.iter().filter(|row| !row.label_positive).count() as u32;
    let false_reject = eval_rows
        .iter()
        .filter(|row| row.label_positive && row.score_bp < threshold_calibration.calibrated_threshold_bp)
        .count() as u32;
    let false_accept = eval_rows
        .iter()
        .filter(|row| !row.label_positive && row.score_bp >= threshold_calibration.calibrated_threshold_bp)
        .count() as u32;

    let mut not_measured = threshold_calibration.not_measured_metrics.clone();
    if positives == 0 {
        not_measured.push("frr_not_measured_no_positive_scores".to_string());
    }
    if negatives == 0 {
        not_measured.push("far_not_measured_no_negative_scores".to_string());
    }
    not_measured.push("platform_slice_summary_open".to_string());

    let frr_bp = false_reject
        .saturating_mul(10_000)
        .saturating_div(positives.max(1)) as u16;
    let far_per_listening_hour_milli = false_accept
        .saturating_mul(1_000)
        .saturating_div(negatives.max(1));
    let miss_rate_bp = frr_bp;
    let latency_proxy_ms = {
        let mut sum = 0u64;
        let mut count = 0u64;
        let mut latency_by_example = BTreeMap::new();
        for row in scored_examples {
            if let Some(latency) = row.latency_proxy_ms {
                latency_by_example.insert(row.example_id.clone(), latency);
            }
        }
        for tensor in tensor_rows {
            if let Some(latency) = latency_by_example.get(tensor.example_id.as_str()) {
                sum = sum.saturating_add(*latency as u64);
                count = count.saturating_add(1);
            }
        }
        if count == 0 {
            default_latency_proxy_ms
        } else {
            (sum / count) as u16
        }
    };
    let (train_example_count, validation_example_count, test_example_count) =
        global_partition_counts(dataset_slices);

    let reject_reason_distribution_ref = {
        let mut counts = BTreeMap::new();
        for row in scored_examples {
            if row.label_positive {
                continue;
            }
            let key = row
                .reason_ref
                .clone()
                .unwrap_or_else(|| "wake_reason_unknown".to_string());
            *counts.entry(key).or_insert(0u32) += 1;
        }
        let mut canonical = String::new();
        for (reason, count) in counts {
            canonical.push_str(reason.as_str());
            canonical.push(':');
            canonical.push_str(count.to_string().as_str());
            canonical.push('|');
        }
        format!("wake_reject_dist_{}", stable_hex_short(canonical.as_bytes()))
    };
    let generated_at_ms = 1_700_000_000_000u64
        .saturating_add(scored_examples.len() as u64)
        .max(1);
    let not_measured_metrics_ref = if not_measured.is_empty() {
        None
    } else {
        Some(format!(
            "wake_not_measured_{}",
            stable_hex_short(not_measured.join("|").as_bytes())
        ))
    };

    WakeTrainEvalReportV1::v2(
        format!("wake_eval_step3_{}", request.dataset_snapshot_id),
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

#[allow(clippy::too_many_arguments)]
fn build_wakepack_real_package(
    request: &WakeTrainingStep3Request,
    feature_config: &WakeFeatureConfigV1,
    threshold_calibration: &WakeThresholdCalibrationV1,
    eval_report: &WakeTrainEvalReportV1,
    training_summary: &WakeDsCnnTrainingSummaryV1,
    dataset_summary: &WakeDatasetAssemblySummaryV1,
    tensor_summary: &WakeTensorBuildSummaryV1,
    model: &DsCnnModel,
) -> Result<WakeCandidatePackageV1, ContractViolation> {
    let candidate_package_id = format!(
        "wake_dscnn_candidate_{}",
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
        "wakepack/candidate/{}/{}.wpk",
        request.dataset_snapshot_id, candidate_package_id
    );

    let payload_bytes = serialize_wakepack_payload(
        request,
        feature_config,
        threshold_calibration,
        eval_report,
        training_summary,
        dataset_summary,
        tensor_summary,
        model,
    );
    let payload_len_bytes = u32::try_from(payload_bytes.len()).map_err(|_| {
        ContractViolation::InvalidValue {
            field: "wake_training_step3.candidate_payload_len_bytes",
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

#[allow(clippy::too_many_arguments)]
fn serialize_wakepack_payload(
    request: &WakeTrainingStep3Request,
    feature_config: &WakeFeatureConfigV1,
    threshold_calibration: &WakeThresholdCalibrationV1,
    eval_report: &WakeTrainEvalReportV1,
    training_summary: &WakeDsCnnTrainingSummaryV1,
    dataset_summary: &WakeDatasetAssemblySummaryV1,
    tensor_summary: &WakeTensorBuildSummaryV1,
    model: &DsCnnModel,
) -> Vec<u8> {
    let mut lines = Vec::new();
    lines.push("wakepack_format=WAKEPACK_V1".to_string());
    lines.push("model_arch=DS_CNN_BASELINE".to_string());
    lines.push(format!("dataset_snapshot_id={}", request.dataset_snapshot_id));
    lines.push(format!("model_version={}", request.model_version));
    lines.push(format!("model_abi={}", request.model_abi));
    lines.push(format!("feature_config_id={}", feature_config.feature_config_id));
    lines.push(format!(
        "threshold_profile_id={}",
        threshold_calibration.threshold_profile_id
    ));
    lines.push(format!(
        "threshold_calibrated_bp={}",
        threshold_calibration.calibrated_threshold_bp
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
    lines.push(format!("train_rows={}", training_summary.train_example_count));
    lines.push(format!("val_rows={}", training_summary.validation_example_count));
    lines.push(format!("test_rows={}", training_summary.test_example_count));
    lines.push(format!(
        "tensor_rows={}",
        tensor_summary.tensor_row_count
    ));
    lines.push(format!("dataset_rows={}", dataset_summary.total_rows));
    lines.push(format!("frame_count={}", model.frame_count));
    lines.push(format!("mel_bins={}", model.mel_bins));
    lines.push(format!("hidden_channels={}", model.hidden_channels));
    lines.push(format!(
        "training_seed={}",
        training_summary.random_seed
    ));
    lines.push(format!(
        "training_epochs={}",
        training_summary.epoch_count
    ));
    lines.push(format!(
        "training_lr_milli={}",
        training_summary.learning_rate_milli
    ));
    lines.push("offline_pipeline_only=true".to_string());
    lines.push("depthwise_kernel=".to_string() + encode_dscnn_depthwise(model).as_str());
    lines.push("depthwise_bias=".to_string() + encode_f32_vec(model.depthwise_bias.as_slice()).as_str());
    lines.push("pointwise_kernel=".to_string() + encode_dscnn_pointwise(model).as_str());
    lines.push("pointwise_bias=".to_string() + encode_f32_vec(model.pointwise_bias.as_slice()).as_str());
    lines.push("output_kernel=".to_string() + encode_f32_vec(model.output_kernel.as_slice()).as_str());
    lines.push(format!("output_bias={}", format_f32(model.output_bias)));
    lines.join("\n").into_bytes()
}

fn encode_dscnn_depthwise(model: &DsCnnModel) -> String {
    let mut out = String::new();
    for kernel in &model.depthwise_kernel {
        out.push('[');
        for (idx, value) in kernel.iter().enumerate() {
            if idx > 0 {
                out.push(',');
            }
            out.push_str(format_f32(*value).as_str());
        }
        out.push(']');
    }
    out
}

fn encode_dscnn_pointwise(model: &DsCnnModel) -> String {
    let mut out = String::new();
    for kernel_row in &model.pointwise_kernel {
        out.push('[');
        out.push_str(encode_f32_vec(kernel_row.as_slice()).as_str());
        out.push(']');
    }
    out
}

fn encode_f32_vec(values: &[f32]) -> String {
    let mut out = String::new();
    for (idx, value) in values.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(format_f32(*value).as_str());
    }
    out
}

fn format_f32(value: f32) -> String {
    format!("{:.8}", value)
}

fn contains_both_labels(rows: &[&WakeFeatureTensorRowV1]) -> bool {
    let has_positive = rows
        .iter()
        .any(|row| label_token_is_positive(row.label.as_str()));
    let has_negative = rows
        .iter()
        .any(|row| !label_token_is_positive(row.label.as_str()));
    has_positive && has_negative
}

fn contains_both_labels_scored(rows: &[&DsCnnScoredExample]) -> bool {
    let has_positive = rows.iter().any(|row| row.label_positive);
    let has_negative = rows.iter().any(|row| !row.label_positive);
    has_positive && has_negative
}

fn average_partition_loss(model: &DsCnnModel, rows: &[&WakeFeatureTensorRowV1]) -> f32 {
    if rows.is_empty() {
        return 0.0;
    }
    let mut loss = 0.0f32;
    for row in rows {
        let input = tensor_bins_as_f32(row.tensor_bins.as_slice());
        let y = label_token_is_positive(row.label.as_str()) as u8 as f32;
        let score = dscnn_forward(model, input.as_slice()).score.clamp(1e-6, 1.0 - 1e-6);
        loss += -(y * score.ln() + (1.0 - y) * (1.0 - score).ln());
    }
    loss / rows.len() as f32
}

fn partition_loss_and_accuracy(model: &DsCnnModel, rows: &[&WakeFeatureTensorRowV1]) -> (f32, u16) {
    if rows.is_empty() {
        return (0.0, 0);
    }
    let mut loss = 0.0f32;
    let mut correct = 0u32;
    for row in rows {
        let input = tensor_bins_as_f32(row.tensor_bins.as_slice());
        let y_pos = label_token_is_positive(row.label.as_str());
        let score = dscnn_forward(model, input.as_slice()).score.clamp(1e-6, 1.0 - 1e-6);
        let y = y_pos as u8 as f32;
        loss += -(y * score.ln() + (1.0 - y) * (1.0 - score).ln());
        if (score >= 0.5) == y_pos {
            correct = correct.saturating_add(1);
        }
    }
    let loss_avg = loss / rows.len() as f32;
    let accuracy_bp = correct.saturating_mul(10_000).saturating_div(rows.len() as u32) as u16;
    (loss_avg, accuracy_bp)
}

fn seeded_weight(seed: &mut u64, scale: f32) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    let unit = (*seed as f64 / u64::MAX as f64) as f32;
    (unit * 2.0 - 1.0) * scale
}

fn dscnn_forward(model: &DsCnnModel, input: &[f32]) -> DsCnnForwardCache {
    let frames = model.frame_count;
    let mel = model.mel_bins;
    let hidden = model.hidden_channels;

    let mut depth_linear = vec![0.0f32; frames * mel];
    let mut depth_act = vec![0.0f32; frames * mel];
    for c in 0..mel {
        for t in 0..frames {
            let mut z = model.depthwise_bias[c];
            for k in 0..DSCNN_DEPTHWISE_KERNEL_SIZE {
                let src_t = t as isize + k as isize - 1;
                if src_t < 0 || src_t >= frames as isize {
                    continue;
                }
                let src_idx = src_t as usize * mel + c;
                z += input[src_idx] * model.depthwise_kernel[c][k];
            }
            let idx = depth_idx(c, t, frames);
            depth_linear[idx] = z;
            depth_act[idx] = z.max(0.0);
        }
    }

    let mut point_linear = vec![0.0f32; hidden * frames];
    let mut point_act = vec![0.0f32; hidden * frames];
    let mut pooled = vec![0.0f32; hidden];
    for h in 0..hidden {
        let mut pooled_sum = 0.0f32;
        for t in 0..frames {
            let mut z = model.pointwise_bias[h];
            for c in 0..mel {
                z += depth_act[depth_idx(c, t, frames)] * model.pointwise_kernel[h][c];
            }
            let pidx = point_idx(h, t, frames);
            point_linear[pidx] = z;
            point_act[pidx] = z.max(0.0);
            pooled_sum += point_act[pidx];
        }
        pooled[h] = pooled_sum / frames as f32;
    }
    let mut logit = model.output_bias;
    for h in 0..hidden {
        logit += pooled[h] * model.output_kernel[h];
    }
    let score = sigmoid(logit);
    DsCnnForwardCache {
        depth_linear,
        depth_act,
        point_linear,
        pooled,
        score,
    }
}

fn sigmoid(x: f32) -> f32 {
    if x >= 0.0 {
        let z = (-x).exp();
        1.0 / (1.0 + z)
    } else {
        let z = x.exp();
        z / (1.0 + z)
    }
}

fn tensor_bins_as_f32(bins: &[u16]) -> Vec<f32> {
    bins.iter().map(|v| *v as f32 / 10_000.0).collect()
}

fn label_token_is_positive(label: &str) -> bool {
    label == "POSITIVE"
}

fn depth_idx(channel: usize, frame: usize, frame_count: usize) -> usize {
    channel * frame_count + frame
}

fn point_idx(hidden: usize, frame: usize, frame_count: usize) -> usize {
    hidden * frame_count + frame
}

fn abs_u16_diff(left: u16, right: u16) -> u16 {
    if left >= right {
        left - right
    } else {
        right - left
    }
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
    platform: Option<String>,
    timestamp_ms: u64,
    label: WakeDatasetLabel,
    source_kind: WakeDatasetSourceKind,
    wake_window_id: Option<String>,
    score_bp: Option<u16>,
    reason_ref: Option<String>,
    extractable_pcm_ref: Option<String>,
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
    let mut device_platform_by_id: BTreeMap<String, String> = BTreeMap::new();
    for session in store.ph1w_all_enrollment_session_rows() {
        session_user_by_id.insert(
            session.wake_enrollment_session_id.clone(),
            session.user_id.as_str().to_string(),
        );
        session_device_by_id.insert(
            session.wake_enrollment_session_id.clone(),
            session.device_id.as_str().to_string(),
        );
        if let Some(device_record) = store.get_device(&session.device_id) {
            device_platform_by_id.insert(
                session.device_id.as_str().to_string(),
                device_record.device_type.clone(),
            );
        }
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
            platform: device_platform_by_id.get(device_id.as_str()).cloned(),
            timestamp_ms: ns_to_ms(sample.captured_at.0),
            label: WakeDatasetLabel::Positive,
            source_kind: WakeDatasetSourceKind::EnrollmentPass,
            wake_window_id: None,
            score_bp: score_from_enrollment_sample(sample.vad_coverage, sample.snr_db, sample.clipping_pct),
            reason_ref: sample
                .reason_code
                .map(|reason| reason_ref_from_code(reason.0)),
            extractable_pcm_ref: extractable_pcm_ref_from_enrollment_sample(sample),
            extractability: extractability_from_enrollment_sample(sample),
            latency_proxy_ms: Some(sample.sample_duration_ms),
        });
    }

    for event in store.ph1w_get_runtime_events() {
        let device_id = event.device_id.as_str().to_string();
        let runtime_pcm_ref =
            extractable_pcm_ref_from_runtime_event_idempotency(event.idempotency_key.as_str());
        examples.push(WakeDatasetExample {
            example_id: format!("runtime:{}", event.wake_event_id),
            user_id: event.user_id.as_ref().map(|user_id| user_id.as_str().to_string()),
            device_id: device_id.clone(),
            platform: device_platform_by_id.get(device_id.as_str()).cloned(),
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
            wake_window_id: Some(event.wake_event_id.clone()),
            score_bp: event.strong_score_bp.or(event.light_score_bp),
            reason_ref: Some(reason_ref_from_code(event.reason_code.0)),
            extractable_pcm_ref: runtime_pcm_ref.clone(),
            extractability: if runtime_pcm_ref.is_some() {
                WakeDatasetExtractability::RawPcmAvailable
            } else {
                WakeDatasetExtractability::MetadataOnly
            },
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
        let device_id = signal.device_id.as_str().to_string();
        examples.push(WakeDatasetExample {
            example_id: format!("learn:{}", signal.signal_id),
            user_id: None,
            device_id: device_id.clone(),
            platform: device_platform_by_id.get(device_id.as_str()).cloned(),
            timestamp_ms: signal.timestamp_ms,
            label,
            source_kind: WakeDatasetSourceKind::LearnSignal,
            wake_window_id: Some(signal.wake_window_id.clone()),
            score_bp: signal.score_bp,
            reason_ref: signal.reason_code.map(|reason| reason_ref_from_code(reason.0)),
            extractable_pcm_ref: None,
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
    if extractable_pcm_ref_from_enrollment_sample(sample).is_some() {
        WakeDatasetExtractability::RawPcmAvailable
    } else {
        WakeDatasetExtractability::MetadataOnly
    }
}

fn extractable_pcm_ref_from_enrollment_sample(sample: &WakeEnrollmentSampleRecord) -> Option<String> {
    let trimmed = sample.idempotency_key.trim();
    let pcm_ref = trimmed.strip_prefix("pcm:")?;
    if pcm_ref.is_empty() {
        None
    } else {
        Some(pcm_ref.to_string())
    }
}

fn extractable_pcm_ref_from_runtime_event_idempotency(idempotency_key: &str) -> Option<String> {
    let trimmed = idempotency_key.trim();
    let pcm_ref = trimmed.strip_prefix("pcm:")?;
    if pcm_ref.is_empty() {
        None
    } else {
        Some(pcm_ref.to_string())
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
                "pcm:wake_pcm_enroll_a".to_string(),
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
                "pcm:wake_pcm_rt_accept".to_string(),
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
                "pcm:wake_pcm_rt_reject".to_string(),
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

    fn step3_pcm_fixtures() -> BTreeMap<String, Vec<i16>> {
        let mut map = BTreeMap::new();
        map.insert("wake_pcm_enroll_a".to_string(), pcm_fixture(4_800, 1_100, 0));
        map.insert("wake_pcm_rt_accept".to_string(), pcm_fixture(5_500, 1_100, 1_300));
        map.insert("wake_pcm_rt_reject".to_string(), pcm_fixture(1_000, 1_100, 3_700));
        map
    }

    fn pcm_fixture(amplitude: i16, sample_count: usize, phase_stride: u32) -> Vec<i16> {
        let mut out = Vec::with_capacity(sample_count);
        for idx in 0..sample_count {
            let phase = ((idx as u32).wrapping_mul(17).wrapping_add(phase_stride)) % 128;
            let carrier = phase as i16 - 64;
            out.push(carrier.saturating_mul(amplitude).saturating_div(64));
        }
        out
    }

    fn valid_step3_request() -> WakeTrainingStep3Request {
        WakeTrainingStep3Request::v1(
            "wake_dataset_snapshot_v3".to_string(),
            "wake_feature_cfg_v1".to_string(),
            "wake_model_v3".to_string(),
            "wake_abi_v1".to_string(),
            ArtifactVersion(4),
            "wake_provenance_v3".to_string(),
            Some(ArtifactVersion(3)),
            true,
            step3_pcm_fixtures(),
        )
        .unwrap()
    }

    fn promotion_manifest_with_metrics(
        artifact_version: ArtifactVersion,
        far_per_listening_hour_milli: u32,
        frr_bp: u16,
        miss_rate_bp: u16,
        latency_proxy_ms: u16,
        threshold_calibration_error_bp: u16,
        not_measured_metrics_ref: Option<String>,
    ) -> WakePackManifestV1 {
        let eval = WakeTrainEvalReportV1::v2(
            format!("wake_eval_promote_v{}", artifact_version.0),
            format!("wake_dataset_snapshot_promote_v{}", artifact_version.0),
            format!("wake_model_promote_v{}", artifact_version.0),
            format!("wake_threshold_promote_v{}", artifact_version.0),
            far_per_listening_hour_milli,
            frr_bp,
            miss_rate_bp,
            latency_proxy_ms,
            Some(8_100),
            threshold_calibration_error_bp,
            format!("wake_calibration_ref_v{}", artifact_version.0),
            format!("wake_reject_dist_ref_v{}", artifact_version.0),
            120,
            20,
            25,
            None,
            not_measured_metrics_ref,
            1_700_000_002_000 + artifact_version.0 as u64,
        )
        .unwrap();
        WakePackManifestV1::v1(
            format!("wake_model_promote_v{}", artifact_version.0),
            "wake_abi_v1".to_string(),
            "wake_feature_cfg_v1".to_string(),
            eval.threshold_profile_id.clone(),
            artifact_version,
            "4a6588bde3f9fcd4cea3f238d10ef00f8fbecc6453e28307fc1ff11337f6925f".to_string(),
            format!("wake_payload_ref_promote_v{}", artifact_version.0),
            format!("wake_provenance_ref_promote_v{}", artifact_version.0),
            eval.dataset_snapshot_id.clone(),
            eval,
            Some(ArtifactVersion(artifact_version.0.saturating_sub(1).max(1))),
            true,
        )
        .unwrap()
    }

    fn seed_device_apply_chain_for_rollback_drill(store: &mut Ph1fStore, device_id: &DeviceId) {
        let hash_v1 =
            "1111111111111111111111111111111111111111111111111111111111111111".to_string();
        let hash_v2 =
            "2222222222222222222222222222222222222222222222222222222222222222".to_string();
        store
            .wake_artifact_stage_commit(
                MonotonicTimeNs(2_000),
                device_id.clone(),
                ArtifactVersion(1),
                hash_v1,
                "wake_payload_ref_v1".to_string(),
                None,
                "wake_stage_v1".to_string(),
            )
            .unwrap();
        store
            .wake_artifact_activate_commit(
                MonotonicTimeNs(2_001),
                device_id.clone(),
                ArtifactVersion(1),
                "wake_activate_v1".to_string(),
            )
            .unwrap();
        store
            .wake_artifact_stage_commit(
                MonotonicTimeNs(2_002),
                device_id.clone(),
                ArtifactVersion(2),
                hash_v2,
                "wake_payload_ref_v2".to_string(),
                None,
                "wake_stage_v2".to_string(),
            )
            .unwrap();
        store
            .wake_artifact_activate_commit(
                MonotonicTimeNs(2_003),
                device_id.clone(),
                ArtifactVersion(2),
                "wake_activate_v2".to_string(),
            )
            .unwrap();
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
                platform: Some("desktop".to_string()),
                timestamp_ms: 1_000,
                label: WakeDatasetLabel::Positive,
                source_kind: WakeDatasetSourceKind::EnrollmentPass,
                wake_window_id: None,
                score_bp: Some(9_100),
                reason_ref: None,
                extractable_pcm_ref: Some("wake_pcm_example_1".to_string()),
                extractability: WakeDatasetExtractability::RawPcmAvailable,
                latency_proxy_ms: Some(1_200),
            },
            WakeDatasetExample {
                example_id: "example_2".to_string(),
                user_id: Some("user_shared".to_string()),
                device_id: "device_b".to_string(),
                platform: Some("desktop".to_string()),
                timestamp_ms: 1_020,
                label: WakeDatasetLabel::Negative,
                source_kind: WakeDatasetSourceKind::RuntimeRejected,
                wake_window_id: Some("wake_evt_example_2".to_string()),
                score_bp: Some(4_000),
                reason_ref: Some("wake_reason_0x00001002".to_string()),
                extractable_pcm_ref: None,
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
                platform: Some("desktop".to_string()),
                timestamp_ms: 1_000,
                label: WakeDatasetLabel::Positive,
                source_kind: WakeDatasetSourceKind::EnrollmentPass,
                wake_window_id: None,
                score_bp: Some(9_100),
                reason_ref: None,
                extractable_pcm_ref: Some("wake_pcm_example_1".to_string()),
                extractability: WakeDatasetExtractability::RawPcmAvailable,
                latency_proxy_ms: Some(1_200),
            },
            WakeDatasetExample {
                example_id: "example_2".to_string(),
                user_id: Some("user_shared".to_string()),
                device_id: "device_b".to_string(),
                platform: Some("desktop".to_string()),
                timestamp_ms: 1_020,
                label: WakeDatasetLabel::Negative,
                source_kind: WakeDatasetSourceKind::RuntimeRejected,
                wake_window_id: Some("wake_evt_example_2".to_string()),
                score_bp: Some(4_000),
                reason_ref: Some("wake_reason_0x00001002".to_string()),
                extractable_pcm_ref: None,
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

    #[test]
    fn at_wake_training_step3_a_tensor_generation_is_deterministic() {
        let store = seeded_store();
        let request = valid_step3_request();
        let config = WakeTrainingStep3Config::default();
        let first = build_wake_training_step3(&store, &request, &config).unwrap();
        let second = build_wake_training_step3(&store, &request, &config).unwrap();
        assert_eq!(first.tensor_rows, second.tensor_rows);
        assert_eq!(first.tensor_summary, second.tensor_summary);
    }

    #[test]
    fn at_wake_training_step3_b_non_extractable_rows_excluded_but_counted() {
        let store = seeded_store();
        let request = valid_step3_request();
        let config = WakeTrainingStep3Config::default();
        let output = build_wake_training_step3(&store, &request, &config).unwrap();
        assert!(output.tensor_summary.total_dataset_rows > output.tensor_summary.tensor_row_count);
        assert!(output.tensor_summary.excluded_non_extractable_rows > 0);
        assert_eq!(output.dataset_summary.total_rows, 5);
    }

    #[test]
    fn at_wake_training_step3_c_dscnn_training_emits_non_empty_payload() {
        let store = seeded_store();
        let request = valid_step3_request();
        let config = WakeTrainingStep3Config::default();
        let output = build_wake_training_step3(&store, &request, &config).unwrap();
        assert!(!output.wakepack_candidate.payload_bytes.is_empty());
        assert!(output.training_summary.final_train_loss_milli > 0);
        assert!(output.wakepack_candidate.payload_len_bytes > 0);
    }

    #[test]
    fn at_wake_training_step3_d_threshold_uses_trained_scores() {
        let store = seeded_store();
        let request = valid_step3_request();
        let config = WakeTrainingStep3Config::default();
        let output = build_wake_training_step3(&store, &request, &config).unwrap();
        assert_eq!(
            output.eval_report.threshold_profile_id,
            output.threshold_calibration.threshold_profile_id
        );
        assert_eq!(
            output.eval_report.threshold_profile_calibrated_bp,
            Some(output.threshold_calibration.calibrated_threshold_bp)
        );
        assert!(output.threshold_calibration.positive_scored_count > 0);
        assert!(output.threshold_calibration.negative_scored_count > 0);
    }

    #[test]
    fn at_wake_training_step3_e_wakepack_hash_is_deterministic() {
        let store = seeded_store();
        let request = valid_step3_request();
        let config = WakeTrainingStep3Config::default();
        let first = build_wake_training_step3(&store, &request, &config).unwrap();
        let second = build_wake_training_step3(&store, &request, &config).unwrap();
        assert_eq!(first.wakepack_candidate.package_hash, second.wakepack_candidate.package_hash);
        assert_eq!(first.wakepack_candidate.payload_bytes, second.wakepack_candidate.payload_bytes);
        assert!(first.wake_pack_manifest.validate().is_ok());
    }

    #[test]
    fn at_wake_training_step3_f_offline_boundary_enforced() {
        let store = seeded_store();
        let mut request = valid_step3_request();
        request.offline_pipeline_only = false;
        let config = WakeTrainingStep3Config::default();
        let err = build_wake_training_step3(&store, &request, &config).unwrap_err();
        assert!(matches!(err, ContractViolation::InvalidValue { .. }));
    }

    #[test]
    fn at_wake_training_step3_g_reproducible_with_same_seed() {
        let store = seeded_store();
        let request = valid_step3_request();
        let config = WakeTrainingStep3Config {
            random_seed: 0xCAFE_BABE_2026_0306,
            ..WakeTrainingStep3Config::default()
        };
        let first = build_wake_training_step3(&store, &request, &config).unwrap();
        let second = build_wake_training_step3(&store, &request, &config).unwrap();
        assert_eq!(
            first.threshold_calibration.calibrated_threshold_bp,
            second.threshold_calibration.calibrated_threshold_bp
        );
        assert_eq!(
            first.wakepack_candidate.package_hash,
            second.wakepack_candidate.package_hash
        );
        assert_eq!(first.training_summary, second.training_summary);
    }

    #[test]
    fn at_wake_training_step4_a_candidate_to_shadow_and_canary_to_active_passes() {
        let mut store = seeded_store();
        let device_a = DeviceId::new("device_a".to_string()).unwrap();
        let artifact_version = ArtifactVersion(20);
        let policy = WakePromotionGatePolicyV1::default();
        let manifest = promotion_manifest_with_metrics(artifact_version, 90, 900, 900, 140, 320, None);

        store
            .wake_promotion_transition_commit(
                MonotonicTimeNs(3_000),
                artifact_version,
                WakePromotionState::Candidate,
                None,
                None,
                None,
                None,
                "wake_gate_candidate_ready".to_string(),
                "wake_promo_candidate_step4".to_string(),
                false,
            )
            .unwrap();
        let shadow = wake_promote_candidate_to_shadow(
            &mut store,
            MonotonicTimeNs(3_001),
            &manifest,
            Some("cohort_shadow_5pct".to_string()),
            &policy,
            "wake_gate_shadow_step4".to_string(),
            "wake_promo_shadow_step4".to_string(),
        )
        .unwrap();
        assert_eq!(shadow.state, WakePromotionState::Shadow);

        let canary = wake_promote_shadow_to_canary(
            &mut store,
            MonotonicTimeNs(3_002),
            &manifest,
            Some("cohort_canary_10pct".to_string()),
            &policy,
            "wake_gate_canary_step4".to_string(),
            "wake_promo_canary_step4".to_string(),
        )
        .unwrap();
        assert_eq!(canary.state, WakePromotionState::Canary);

        seed_device_apply_chain_for_rollback_drill(&mut store, &device_a);
        let active = wake_promote_canary_to_active(
            &mut store,
            MonotonicTimeNs(3_003),
            &manifest,
            &policy,
            &[device_a],
            "wake_gate_active_step4".to_string(),
            "wake_promo_active_step4".to_string(),
        )
        .unwrap();
        assert_eq!(active.state, WakePromotionState::Active);
        assert_eq!(store.wake_promotion_active_artifact_version(), Some(artifact_version));
    }

    #[test]
    fn at_wake_training_step4_b_missing_required_metrics_blocks_gate() {
        let policy = WakePromotionGatePolicyV1::default();
        let manifest = promotion_manifest_with_metrics(
            ArtifactVersion(21),
            90,
            900,
            900,
            140,
            320,
            Some("wake_not_measured_metrics_ref".to_string()),
        );
        let gate = evaluate_wake_promotion_gate(
            &manifest.eval_metrics_summary,
            WakePromotionState::Shadow,
            &policy,
        )
        .unwrap();
        assert_eq!(gate.outcome, WakePromotionGateOutcomeV1::Block);
        assert_eq!(gate.reason_code, WAKE_PROMOTION_REASON_METRIC_NOT_MEASURED);
    }

    #[test]
    fn at_wake_training_step4_c_severe_regression_requires_rollback() {
        let policy = WakePromotionGatePolicyV1::default();
        let manifest =
            promotion_manifest_with_metrics(ArtifactVersion(22), 750, 3_500, 3_500, 650, 2_100, None);
        let gate = evaluate_wake_promotion_gate(
            &manifest.eval_metrics_summary,
            WakePromotionState::Active,
            &policy,
        )
        .unwrap();
        assert_eq!(gate.outcome, WakePromotionGateOutcomeV1::RequireRollback);
        assert_eq!(gate.reason_code, WAKE_PROMOTION_REASON_GATE_REQUIRES_ROLLBACK);
    }

    #[test]
    fn at_wake_training_step4_d_blocked_versions_require_explicit_revalidation() {
        let mut store = seeded_store();
        let policy = WakePromotionGatePolicyV1::default();
        let artifact_version = ArtifactVersion(23);
        let manifest = promotion_manifest_with_metrics(artifact_version, 120, 900, 900, 160, 300, None);
        store
            .wake_promotion_transition_commit(
                MonotonicTimeNs(3_100),
                artifact_version,
                WakePromotionState::Candidate,
                None,
                None,
                None,
                None,
                "wake_gate_candidate_step4d".to_string(),
                "wake_promo_candidate_step4d".to_string(),
                false,
            )
            .unwrap();
        wake_block_candidate_version(
            &mut store,
            MonotonicTimeNs(3_101),
            artifact_version,
            ReasonCodeId(0x57A1_5001),
            "wake_gate_block_step4d".to_string(),
            "wake_promo_block_step4d".to_string(),
        )
        .unwrap();

        let blocked_err = wake_promote_candidate_to_shadow(
            &mut store,
            MonotonicTimeNs(3_102),
            &manifest,
            None,
            &policy,
            "wake_gate_shadow_step4d".to_string(),
            "wake_promo_shadow_step4d".to_string(),
        )
        .expect_err("blocked version must fail until explicit revalidation");
        assert!(matches!(blocked_err, StorageError::ContractViolation(_)));

        store
            .wake_promotion_revalidate_blocked_version(
                artifact_version,
                "wake_revalidation_step4d".to_string(),
                "wake_revalidation_idem_step4d".to_string(),
            )
            .unwrap();
        store
            .wake_promotion_transition_commit(
                MonotonicTimeNs(3_102),
                artifact_version,
                WakePromotionState::Candidate,
                None,
                None,
                None,
                None,
                "wake_gate_candidate_step4d_revalidated".to_string(),
                "wake_promo_candidate_step4d_revalidated".to_string(),
                false,
            )
            .unwrap();
        let shadow = wake_promote_candidate_to_shadow(
            &mut store,
            MonotonicTimeNs(3_103),
            &manifest,
            None,
            &policy,
            "wake_gate_shadow_step4d_retry".to_string(),
            "wake_promo_shadow_step4d_retry".to_string(),
        )
        .unwrap();
        assert_eq!(shadow.state, WakePromotionState::Shadow);
    }

    #[test]
    fn at_wake_training_step4_e_rollback_drill_fails_without_last_known_good() {
        let store = seeded_store();
        let device_a = DeviceId::new("device_a".to_string()).unwrap();
        let drill = run_wake_rollback_drill(&store, &device_a, ArtifactVersion(24)).unwrap();
        assert!(!drill.rollback_ready);
        assert_eq!(
            drill.reason_code,
            Some(WAKE_PROMOTION_REASON_ROLLBACK_TARGET_MISSING)
        );
    }

    #[test]
    fn at_wake_training_step4_f_rollback_drill_passes_with_valid_pointer() {
        let mut store = seeded_store();
        let device_a = DeviceId::new("device_a".to_string()).unwrap();
        seed_device_apply_chain_for_rollback_drill(&mut store, &device_a);
        let drill = run_wake_rollback_drill(&store, &device_a, ArtifactVersion(25)).unwrap();
        assert!(drill.rollback_ready);
        assert!(drill.last_known_good_artifact_version.is_some());
        assert!(drill.rollback_target_artifact_version.is_some());
    }

    #[test]
    fn at_wake_training_step4_g_invalid_transition_path_fails_closed() {
        let mut store = seeded_store();
        let policy = WakePromotionGatePolicyV1::default();
        let manifest = promotion_manifest_with_metrics(ArtifactVersion(26), 100, 800, 800, 150, 200, None);
        let err = wake_promote_shadow_to_canary(
            &mut store,
            MonotonicTimeNs(3_200),
            &manifest,
            None,
            &policy,
            "wake_gate_canary_without_shadow".to_string(),
            "wake_promo_canary_without_shadow".to_string(),
        )
        .expect_err("shadow->canary without prior shadow state must fail");
        assert!(matches!(err, StorageError::ContractViolation(_)));
    }
}
