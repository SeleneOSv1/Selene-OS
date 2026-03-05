#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use selene_kernel_contracts::ph1art::ArtifactVersion;
use selene_kernel_contracts::ph1j::DeviceId;
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, Validate};
use sha2::{Digest, Sha256};
use selene_storage::ph1f::{
    MobileArtifactSyncKind, MobileArtifactSyncQueueRecord, MobileArtifactSyncState, Ph1fStore,
    StorageError,
};

pub const DEVICE_SYNC_WORKER_MAX_ITEMS: u16 = 16;
pub const DEVICE_SYNC_WORKER_LEASE_MS: u32 = 30_000;
pub const DEVICE_SYNC_RETRY_AFTER_MS_DEFAULT: u32 = 30_000;
pub const DEVICE_SYNC_MAX_ATTEMPTS_DEFAULT: u16 = 5;
pub const DEVICE_SYNC_PULL_RETRY_AFTER_MS_DEFAULT: u32 = 30_000;
pub const DEVICE_SYNC_PULL_CACHE_DIR_DEFAULT: &str = ".runtime/device_artifact_cache";
pub const WAKE_ARTIFACT_REASON_HASH_MISMATCH: ReasonCodeId = ReasonCodeId(0x57A0_5101);
pub const WAKE_ARTIFACT_REASON_ACTIVATION_FAILED: ReasonCodeId = ReasonCodeId(0x57A0_5102);
pub const WAKE_ARTIFACT_REASON_DOWNLOAD_FAILED: ReasonCodeId = ReasonCodeId(0x57A0_5103);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DeviceArtifactSyncQueueMetrics {
    pub queued_count: u32,
    pub in_flight_count: u32,
    pub acked_count: u32,
    pub dead_letter_count: u32,
    pub replay_due_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DeviceArtifactSyncWorkerPassMetrics {
    pub dequeued_count: u16,
    pub acked_count: u16,
    pub retry_scheduled_count: u16,
    pub dead_lettered_count: u16,
    pub pulled_device_count: u16,
    pub pulled_update_count: u16,
    pub apply_activated_count: u16,
    pub apply_rollback_count: u16,
    pub apply_noop_count: u16,
    pub pull_error_count: u16,
    pub queue_after: DeviceArtifactSyncQueueMetrics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DeviceArtifactPullApplyMetrics {
    pub pulled_device_count: u16,
    pub pulled_update_count: u16,
    pub activated_count: u16,
    pub rollback_count: u16,
    pub noop_count: u16,
    pub pull_error_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DeviceArtifactPullRequest {
    pub schema_version: u8,
    pub request_id: String,
    pub device_id: String,
    pub platform: String,
    pub current_active_versions: Vec<DeviceArtifactVersionState>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DeviceArtifactVersionState {
    pub artifact_type: String,
    pub artifact_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct DeviceArtifactPullResponse {
    pub schema_version: u8,
    pub updates: Vec<DeviceArtifactPullUpdate>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DeviceArtifactPullUpdate {
    pub artifact_type: String,
    pub artifact_version: u32,
    pub payload_ref: String,
    pub package_hash: String,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DeviceArtifactSyncEnvelope {
    pub schema_version: u8,
    pub sync_job_id: String,
    pub sync_kind: String,
    pub receipt_ref: String,
    pub artifact_profile_id: String,
    pub onboarding_session_id: Option<String>,
    pub user_id: Option<String>,
    pub device_id: String,
    pub enqueued_at_ns: u64,
    pub attempt_count: u16,
    pub idempotency_key: String,
}

impl DeviceArtifactSyncEnvelope {
    pub fn from_row(row: &MobileArtifactSyncQueueRecord) -> Self {
        Self {
            schema_version: 1,
            sync_job_id: row.sync_job_id.clone(),
            sync_kind: sync_kind_label(row.sync_kind).to_string(),
            receipt_ref: row.receipt_ref.clone(),
            artifact_profile_id: row.artifact_profile_id.clone(),
            onboarding_session_id: row
                .onboarding_session_id
                .as_ref()
                .map(|v| v.as_str().to_string()),
            user_id: row.user_id.as_ref().map(|v| v.as_str().to_string()),
            device_id: row.device_id.as_str().to_string(),
            enqueued_at_ns: row.enqueued_at.0,
            attempt_count: row.attempt_count,
            idempotency_key: row.idempotency_key.clone(),
        }
    }
}

fn sync_kind_label(kind: MobileArtifactSyncKind) -> &'static str {
    match kind {
        MobileArtifactSyncKind::WakeProfile => "WakeProfile",
        MobileArtifactSyncKind::VoiceProfile => "VoiceProfile",
        MobileArtifactSyncKind::VoiceArtifactManifest => "VoiceArtifactManifest",
        MobileArtifactSyncKind::WakeArtifactManifest => "WakeArtifactManifest",
        MobileArtifactSyncKind::EmoArtifactManifest => "EmoArtifactManifest",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceArtifactSyncSendReceipt {
    pub remote_ack_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceArtifactSyncSendError {
    pub message: String,
    pub retry_after_ms: u32,
}

impl DeviceArtifactSyncSendError {
    pub fn retryable(message: impl Into<String>, retry_after_ms: u32) -> Self {
        let msg = message.into();
        let bounded_retry_after = retry_after_ms.clamp(1_000, 300_000);
        let bounded_msg = if msg.len() > 256 {
            msg.chars().take(256).collect::<String>()
        } else {
            msg
        };
        Self {
            message: bounded_msg,
            retry_after_ms: bounded_retry_after,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceArtifactPullError {
    pub message: String,
    pub retry_after_ms: u32,
}

impl DeviceArtifactPullError {
    pub fn retryable(message: impl Into<String>, retry_after_ms: u32) -> Self {
        let msg = message.into();
        let bounded_retry_after = retry_after_ms.clamp(1_000, 300_000);
        let bounded_msg = if msg.len() > 256 {
            msg.chars().take(256).collect::<String>()
        } else {
            msg
        };
        Self {
            message: bounded_msg,
            retry_after_ms: bounded_retry_after,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceArtifactSyncHttpSenderConfig {
    pub endpoint: String,
    pub bearer_token: Option<String>,
    pub connect_timeout_ms: u64,
    pub request_timeout_ms: u64,
}

impl DeviceArtifactSyncHttpSenderConfig {
    pub fn from_env() -> Option<Self> {
        let endpoint = env::var("SELENE_ENGINE_B_SYNC_ENDPOINT").ok()?;
        let endpoint = endpoint.trim().to_string();
        if endpoint.is_empty() {
            return None;
        }
        let bearer_token = env::var("SELENE_ENGINE_B_SYNC_BEARER").ok().and_then(|v| {
            let s = v.trim().to_string();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        });
        let connect_timeout_ms = env::var("SELENE_ENGINE_B_SYNC_CONNECT_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|v| (100..=60_000).contains(v))
            .unwrap_or(3_000);
        let request_timeout_ms = env::var("SELENE_ENGINE_B_SYNC_REQUEST_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|v| (100..=120_000).contains(v))
            .unwrap_or(10_000);

        Some(Self {
            endpoint,
            bearer_token,
            connect_timeout_ms,
            request_timeout_ms,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceArtifactPullHttpConfig {
    pub endpoint: String,
    pub bearer_token: Option<String>,
    pub connect_timeout_ms: u64,
    pub request_timeout_ms: u64,
    pub cache_dir: String,
}

impl DeviceArtifactPullHttpConfig {
    pub fn from_env() -> Option<Self> {
        let endpoint = env::var("SELENE_ENGINE_B_SYNC_PULL_ENDPOINT").ok()?;
        let endpoint = endpoint.trim().to_string();
        if endpoint.is_empty() {
            return None;
        }
        let bearer_token = env::var("SELENE_ENGINE_B_SYNC_PULL_BEARER")
            .ok()
            .and_then(|v| {
                let s = v.trim().to_string();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            })
            .or_else(|| {
                env::var("SELENE_ENGINE_B_SYNC_BEARER").ok().and_then(|v| {
                    let s = v.trim().to_string();
                    if s.is_empty() {
                        None
                    } else {
                        Some(s)
                    }
                })
            });
        let connect_timeout_ms = env::var("SELENE_ENGINE_B_SYNC_PULL_CONNECT_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|v| (100..=60_000).contains(v))
            .unwrap_or(3_000);
        let request_timeout_ms = env::var("SELENE_ENGINE_B_SYNC_PULL_REQUEST_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|v| (100..=120_000).contains(v))
            .unwrap_or(10_000);
        let cache_dir = env::var("SELENE_ENGINE_B_SYNC_CACHE_DIR")
            .ok()
            .and_then(|v| {
                let trimmed = v.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            })
            .unwrap_or_else(|| DEVICE_SYNC_PULL_CACHE_DIR_DEFAULT.to_string());
        Some(Self {
            endpoint,
            bearer_token,
            connect_timeout_ms,
            request_timeout_ms,
            cache_dir,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceArtifactPullRuntime {
    Disabled,
    Http(DeviceArtifactPullHttpConfig),
    #[cfg(test)]
    StaticForTests(DeviceArtifactPullResponse),
}

impl Default for DeviceArtifactPullRuntime {
    fn default() -> Self {
        Self::from_env_or_disabled()
    }
}

impl DeviceArtifactPullRuntime {
    pub fn from_env_or_disabled() -> Self {
        if let Some(config) = DeviceArtifactPullHttpConfig::from_env() {
            return Self::Http(config);
        }
        Self::Disabled
    }

    fn cache_dir(&self) -> PathBuf {
        match self {
            Self::Disabled => PathBuf::from(DEVICE_SYNC_PULL_CACHE_DIR_DEFAULT),
            Self::Http(config) => PathBuf::from(config.cache_dir.as_str()),
            #[cfg(test)]
            Self::StaticForTests(_) => PathBuf::from(DEVICE_SYNC_PULL_CACHE_DIR_DEFAULT),
        }
    }

    fn pull(
        &self,
        request: &DeviceArtifactPullRequest,
    ) -> Result<DeviceArtifactPullResponse, DeviceArtifactPullError> {
        match self {
            Self::Disabled => Ok(DeviceArtifactPullResponse {
                schema_version: 1,
                updates: Vec::new(),
            }),
            Self::Http(config) => pull_http_sync_updates(config, request),
            #[cfg(test)]
            Self::StaticForTests(response) => Ok(response.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceArtifactSyncSenderRuntime {
    LoopbackAck,
    Http(DeviceArtifactSyncHttpSenderConfig),
    AlwaysFail {
        message: String,
        retry_after_ms: u32,
    },
}

impl Default for DeviceArtifactSyncSenderRuntime {
    fn default() -> Self {
        Self::from_env_or_loopback()
    }
}

impl DeviceArtifactSyncSenderRuntime {
    pub fn from_env_or_loopback() -> Self {
        if let Some(config) = DeviceArtifactSyncHttpSenderConfig::from_env() {
            return Self::Http(config);
        }
        Self::LoopbackAck
    }

    #[cfg(test)]
    pub fn always_fail_for_tests(message: &str, retry_after_ms: u32) -> Self {
        Self::AlwaysFail {
            message: message.to_string(),
            retry_after_ms,
        }
    }

    pub fn send(
        &self,
        envelope: &DeviceArtifactSyncEnvelope,
    ) -> Result<DeviceArtifactSyncSendReceipt, DeviceArtifactSyncSendError> {
        match self {
            Self::LoopbackAck => Ok(DeviceArtifactSyncSendReceipt {
                remote_ack_ref: Some(format!("loopback_ack:{}", envelope.sync_job_id)),
            }),
            Self::AlwaysFail {
                message,
                retry_after_ms,
            } => Err(DeviceArtifactSyncSendError::retryable(
                message.clone(),
                *retry_after_ms,
            )),
            Self::Http(config) => send_http_sync_envelope(config, envelope),
        }
    }
}

pub fn run_device_artifact_sync_worker_pass(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    worker_id: String,
    sender: &DeviceArtifactSyncSenderRuntime,
) -> Result<(), StorageError> {
    let _ = run_device_artifact_sync_worker_pass_with_metrics(store, now, worker_id, sender)?;
    Ok(())
}

pub fn run_device_artifact_sync_worker_pass_with_metrics(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    worker_id: String,
    sender: &DeviceArtifactSyncSenderRuntime,
) -> Result<DeviceArtifactSyncWorkerPassMetrics, StorageError> {
    let pull_runtime = DeviceArtifactPullRuntime::from_env_or_disabled();
    run_device_artifact_sync_worker_pass_with_metrics_internal(
        store,
        now,
        worker_id,
        sender,
        &pull_runtime,
        device_sync_max_attempts_from_env(),
    )
}

fn run_device_artifact_sync_worker_pass_with_metrics_internal(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    worker_id: String,
    sender: &DeviceArtifactSyncSenderRuntime,
    pull_runtime: &DeviceArtifactPullRuntime,
    max_attempts: u16,
) -> Result<DeviceArtifactSyncWorkerPassMetrics, StorageError> {
    let pull_metrics = run_device_artifact_pull_apply_pass_internal(
        store,
        now,
        worker_id.as_str(),
        pull_runtime,
        None,
    )?;
    let max_attempts = max_attempts.max(1);
    let dequeued = store.device_artifact_sync_dequeue_batch(
        now,
        DEVICE_SYNC_WORKER_MAX_ITEMS,
        DEVICE_SYNC_WORKER_LEASE_MS,
        worker_id.clone(),
    )?;
    let mut metrics = DeviceArtifactSyncWorkerPassMetrics {
        dequeued_count: dequeued.len() as u16,
        pulled_device_count: pull_metrics.pulled_device_count,
        pulled_update_count: pull_metrics.pulled_update_count,
        apply_activated_count: pull_metrics.activated_count,
        apply_rollback_count: pull_metrics.rollback_count,
        apply_noop_count: pull_metrics.noop_count,
        pull_error_count: pull_metrics.pull_error_count,
        ..DeviceArtifactSyncWorkerPassMetrics::default()
    };
    if dequeued.is_empty() {
        metrics.queue_after = snapshot_queue_metrics(store, now);
        return Ok(metrics);
    }
    for row in dequeued {
        let envelope = DeviceArtifactSyncEnvelope::from_row(&row);
        match sender.send(&envelope) {
            Ok(_receipt) => {
                store.device_artifact_sync_ack_commit(
                    now,
                    &row.sync_job_id,
                    Some(worker_id.as_str()),
                )?;
                metrics.acked_count = metrics.acked_count.saturating_add(1);
            }
            Err(err) => {
                if row.attempt_count >= max_attempts {
                    store.device_artifact_sync_dead_letter_commit(
                        now,
                        &row.sync_job_id,
                        Some(worker_id.as_str()),
                        err.message,
                    )?;
                    metrics.dead_lettered_count = metrics.dead_lettered_count.saturating_add(1);
                } else {
                    store.device_artifact_sync_fail_commit(
                        now,
                        &row.sync_job_id,
                        Some(worker_id.as_str()),
                        err.message,
                        err.retry_after_ms,
                    )?;
                    metrics.retry_scheduled_count = metrics.retry_scheduled_count.saturating_add(1);
                }
            }
        }
    }
    metrics.queue_after = snapshot_queue_metrics(store, now);
    Ok(metrics)
}

fn snapshot_queue_metrics(
    store: &Ph1fStore,
    now: MonotonicTimeNs,
) -> DeviceArtifactSyncQueueMetrics {
    let mut out = DeviceArtifactSyncQueueMetrics::default();
    for row in store.device_artifact_sync_queue_rows() {
        match row.state {
            MobileArtifactSyncState::Queued => {
                out.queued_count = out.queued_count.saturating_add(1)
            }
            MobileArtifactSyncState::InFlight => {
                out.in_flight_count = out.in_flight_count.saturating_add(1)
            }
            MobileArtifactSyncState::Acked => out.acked_count = out.acked_count.saturating_add(1),
            MobileArtifactSyncState::DeadLetter => {
                out.dead_letter_count = out.dead_letter_count.saturating_add(1)
            }
        }
    }
    out.replay_due_count = store.device_artifact_sync_replay_due_rows(now).len() as u32;
    out
}

fn device_sync_max_attempts_from_env() -> u16 {
    env::var("SELENE_ENGINE_B_SYNC_MAX_ATTEMPTS")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .filter(|v| (1..=100).contains(v))
        .unwrap_or(DEVICE_SYNC_MAX_ATTEMPTS_DEFAULT)
}

type ActivationHook = dyn Fn(&DeviceId, ArtifactVersion, &str) -> Result<(), String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WakeApplyOutcome {
    Activated,
    RolledBack,
    NoopAlreadyActive,
}

fn run_device_artifact_pull_apply_pass_internal(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    worker_id: &str,
    pull_runtime: &DeviceArtifactPullRuntime,
    activation_hook: Option<&ActivationHook>,
) -> Result<DeviceArtifactPullApplyMetrics, StorageError> {
    let mut metrics = DeviceArtifactPullApplyMetrics::default();
    let cache_dir = pull_runtime.cache_dir();
    let known_devices = store.device_artifact_sync_known_device_ids();
    for device_id in known_devices {
        let platform = store
            .get_device(&device_id)
            .map(|row| row.device_type.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let current_active_versions = store
            .wake_artifact_apply_current_row(&device_id)
            .and_then(|row| row.active_artifact_version)
            .map(|version| {
                vec![DeviceArtifactVersionState {
                    artifact_type: "WakePack".to_string(),
                    artifact_version: version.0,
                }]
            })
            .unwrap_or_default();
        let request_id = stable_sync_key(
            "wake_pull",
            format!("{}:{}:{}:{}", worker_id, device_id.as_str(), now.0, platform).as_bytes(),
            24,
        );
        let request = DeviceArtifactPullRequest {
            schema_version: 1,
            request_id,
            device_id: device_id.as_str().to_string(),
            platform,
            current_active_versions,
        };
        let response = match pull_runtime.pull(&request) {
            Ok(resp) => resp,
            Err(_) => {
                metrics.pull_error_count = metrics.pull_error_count.saturating_add(1);
                continue;
            }
        };
        metrics.pulled_device_count = metrics.pulled_device_count.saturating_add(1);
        metrics.pulled_update_count = metrics
            .pulled_update_count
            .saturating_add(response.updates.len().min(u16::MAX as usize) as u16);
        for update in response.updates {
            if !update.artifact_type.eq_ignore_ascii_case("WakePack") {
                continue;
            }
            match apply_wake_artifact_update(
                store,
                now,
                &device_id,
                &cache_dir,
                pull_runtime,
                &update,
                activation_hook,
            ) {
                Ok(WakeApplyOutcome::Activated) => {
                    metrics.activated_count = metrics.activated_count.saturating_add(1);
                }
                Ok(WakeApplyOutcome::RolledBack) => {
                    metrics.rollback_count = metrics.rollback_count.saturating_add(1);
                }
                Ok(WakeApplyOutcome::NoopAlreadyActive) => {
                    metrics.noop_count = metrics.noop_count.saturating_add(1);
                }
                Err(_) => {
                    metrics.pull_error_count = metrics.pull_error_count.saturating_add(1);
                }
            }
        }
    }
    Ok(metrics)
}

fn apply_wake_artifact_update(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    device_id: &DeviceId,
    cache_dir: &Path,
    pull_runtime: &DeviceArtifactPullRuntime,
    update: &DeviceArtifactPullUpdate,
    activation_hook: Option<&ActivationHook>,
) -> Result<WakeApplyOutcome, StorageError> {
    let artifact_version = ArtifactVersion(update.artifact_version);
    artifact_version.validate().map_err(StorageError::ContractViolation)?;
    if let Some(current) = store.wake_artifact_apply_current_row(device_id) {
        if current.active_artifact_version == Some(artifact_version) {
            return Ok(WakeApplyOutcome::NoopAlreadyActive);
        }
    }
    if update.payload_ref.trim().is_empty() || update.payload_ref.len() > 256 {
        return Err(StorageError::ContractViolation(
            selene_kernel_contracts::ContractViolation::InvalidValue {
                field: "device_artifact_pull_update.payload_ref",
                reason: "must be non-empty and <= 256 chars",
            },
        ));
    }
    if update.package_hash.trim().is_empty() || update.package_hash.len() > 64 {
        return Err(StorageError::ContractViolation(
            selene_kernel_contracts::ContractViolation::InvalidValue {
                field: "device_artifact_pull_update.package_hash",
                reason: "must be non-empty and <= 64 chars",
            },
        ));
    }

    let base_idem = update.idempotency_key.as_deref().unwrap_or("pull_update");
    let payload_bytes = match load_payload_bytes_from_ref(pull_runtime, update.payload_ref.as_str()) {
        Ok(bytes) => bytes,
        Err(_) => {
            let stage_key = stable_sync_key(
                "wake_stage",
                format!(
                    "{}:{}:{}:{}:download_fail",
                    device_id.as_str(),
                    artifact_version.0,
                    update.package_hash,
                    base_idem
                )
                .as_bytes(),
                24,
            );
            let _ = store.wake_artifact_stage_commit(
                now,
                device_id.clone(),
                artifact_version,
                update.package_hash.clone(),
                update.payload_ref.clone(),
                None,
                stage_key,
            );
            let rollback_key = stable_sync_key(
                "wake_rollback",
                format!(
                    "{}:{}:{}:{}",
                    device_id.as_str(),
                    artifact_version.0,
                    update.package_hash,
                    base_idem
                )
                .as_bytes(),
                24,
            );
            let _ = store.wake_artifact_rollback_commit(
                now,
                device_id.clone(),
                artifact_version,
                WAKE_ARTIFACT_REASON_DOWNLOAD_FAILED,
                rollback_key,
            );
            return Ok(WakeApplyOutcome::RolledBack);
        }
    };
    let payload_hash = sha256_hex(payload_bytes.as_slice());
    if payload_hash != update.package_hash {
        let stage_key = stable_sync_key(
            "wake_stage",
            format!(
                "{}:{}:{}:{}:hash_mismatch",
                device_id.as_str(),
                artifact_version.0,
                update.package_hash,
                base_idem
            )
            .as_bytes(),
            24,
        );
        let _ = store.wake_artifact_stage_commit(
            now,
            device_id.clone(),
            artifact_version,
            update.package_hash.clone(),
            update.payload_ref.clone(),
            None,
            stage_key,
        );
        let rollback_key = stable_sync_key(
            "wake_rollback",
            format!(
                "{}:{}:{}:{}",
                device_id.as_str(),
                artifact_version.0,
                update.package_hash,
                base_idem
            )
            .as_bytes(),
            24,
        );
        let _ = store.wake_artifact_rollback_commit(
            now,
            device_id.clone(),
            artifact_version,
            WAKE_ARTIFACT_REASON_HASH_MISMATCH,
            rollback_key,
        );
        return Ok(WakeApplyOutcome::RolledBack);
    }

    let cache_ref = persist_payload_to_cache(
        cache_dir,
        device_id,
        artifact_version,
        payload_hash.as_str(),
        payload_bytes.as_slice(),
    )?;
    let stage_key = stable_sync_key(
        "wake_stage",
        format!(
            "{}:{}:{}:{}",
            device_id.as_str(),
            artifact_version.0,
            payload_hash,
            base_idem
        )
        .as_bytes(),
        24,
    );
    let _ = store.wake_artifact_stage_commit(
        now,
        device_id.clone(),
        artifact_version,
        payload_hash.clone(),
        update.payload_ref.clone(),
        Some(cache_ref.clone()),
        stage_key,
    )?;

    if let Some(hook) = activation_hook {
        if hook(device_id, artifact_version, cache_ref.as_str()).is_err() {
            let rollback_key = stable_sync_key(
                "wake_rollback",
                format!(
                    "{}:{}:{}:{}:hook",
                    device_id.as_str(),
                    artifact_version.0,
                    payload_hash,
                    base_idem
                )
                .as_bytes(),
                24,
            );
            let _ = store.wake_artifact_rollback_commit(
                now,
                device_id.clone(),
                artifact_version,
                WAKE_ARTIFACT_REASON_ACTIVATION_FAILED,
                rollback_key,
            );
            return Ok(WakeApplyOutcome::RolledBack);
        }
    }
    let activate_key = stable_sync_key(
        "wake_activate",
        format!(
            "{}:{}:{}:{}",
            device_id.as_str(),
            artifact_version.0,
            payload_hash,
            base_idem
        )
        .as_bytes(),
        24,
    );
    let _ = store.wake_artifact_activate_commit(
        now,
        device_id.clone(),
        artifact_version,
        activate_key,
    )?;
    Ok(WakeApplyOutcome::Activated)
}

fn load_payload_bytes_from_ref(
    pull_runtime: &DeviceArtifactPullRuntime,
    payload_ref: &str,
) -> Result<Vec<u8>, DeviceArtifactPullError> {
    if let Some(rest) = payload_ref.strip_prefix("inline:") {
        return Ok(rest.as_bytes().to_vec());
    }
    if let Some(path) = payload_ref.strip_prefix("file://") {
        return fs::read(path).map_err(|err| {
            DeviceArtifactPullError::retryable(
                format!("payload file read failed: {}", err),
                DEVICE_SYNC_PULL_RETRY_AFTER_MS_DEFAULT,
            )
        });
    }
    if payload_ref.starts_with("http://") || payload_ref.starts_with("https://") {
        let (connect_timeout_ms, request_timeout_ms, bearer_token) = match pull_runtime {
            DeviceArtifactPullRuntime::Http(config) => (
                config.connect_timeout_ms,
                config.request_timeout_ms,
                config.bearer_token.as_ref().cloned(),
            ),
            _ => (3_000, 10_000, None),
        };
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_millis(connect_timeout_ms))
            .timeout_read(Duration::from_millis(request_timeout_ms))
            .timeout_write(Duration::from_millis(request_timeout_ms))
            .build();
        let mut req = agent.get(payload_ref);
        if let Some(token) = bearer_token {
            req = req.set("authorization", format!("Bearer {}", token).as_str());
        }
        let response = match req.call() {
            Ok(resp) => resp,
            Err(ureq::Error::Status(code, resp)) => {
                let retry_after = parse_retry_after_ms(resp.header("retry-after"));
                return Err(DeviceArtifactPullError::retryable(
                    format!("payload download failed with http status {}", code),
                    retry_after,
                ));
            }
            Err(ureq::Error::Transport(err)) => {
                return Err(DeviceArtifactPullError::retryable(
                    format!("payload transport error: {}", err),
                    DEVICE_SYNC_PULL_RETRY_AFTER_MS_DEFAULT,
                ));
            }
        };
        let mut reader = response.into_reader();
        let mut bytes = Vec::new();
        use std::io::Read as _;
        reader.read_to_end(&mut bytes).map_err(|err| {
            DeviceArtifactPullError::retryable(
                format!("payload read failed: {}", err),
                DEVICE_SYNC_PULL_RETRY_AFTER_MS_DEFAULT,
            )
        })?;
        return Ok(bytes);
    }
    Ok(payload_ref.as_bytes().to_vec())
}

fn persist_payload_to_cache(
    cache_dir: &Path,
    device_id: &DeviceId,
    artifact_version: ArtifactVersion,
    package_hash: &str,
    payload_bytes: &[u8],
) -> Result<String, StorageError> {
    fs::create_dir_all(cache_dir).map_err(|_| {
        StorageError::ContractViolation(selene_kernel_contracts::ContractViolation::InvalidValue {
            field: "wake_artifact_apply.cache_dir",
            reason: "must be creatable",
        })
    })?;
    let device_slug = sanitize_key_fragment(device_id.as_str());
    let filename = format!(
        "wakepack_{}_v{}_{}.bin",
        device_slug,
        artifact_version.0,
        &package_hash[..16]
    );
    let path = cache_dir.join(filename);
    fs::write(&path, payload_bytes).map_err(|_| {
        StorageError::ContractViolation(selene_kernel_contracts::ContractViolation::InvalidValue {
            field: "wake_artifact_apply.local_cache_ref",
            reason: "must be writable",
        })
    })?;
    Ok(path.to_string_lossy().to_string())
}

fn sanitize_key_fragment(raw: &str) -> String {
    raw.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn stable_sync_key(prefix: &str, bytes: &[u8], take_chars: usize) -> String {
    let hash = sha256_hex(bytes);
    let suffix_len = take_chars.clamp(8, 56);
    format!("{}_{}", prefix, &hash[..suffix_len])
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    format!("{:x}", digest)
}

fn pull_http_sync_updates(
    config: &DeviceArtifactPullHttpConfig,
    request: &DeviceArtifactPullRequest,
) -> Result<DeviceArtifactPullResponse, DeviceArtifactPullError> {
    let payload = serde_json::to_string(request).map_err(|err| {
        DeviceArtifactPullError::retryable(
            format!("pull payload encode failed: {}", err),
            DEVICE_SYNC_PULL_RETRY_AFTER_MS_DEFAULT,
        )
    })?;
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_millis(config.connect_timeout_ms))
        .timeout_read(Duration::from_millis(config.request_timeout_ms))
        .timeout_write(Duration::from_millis(config.request_timeout_ms))
        .build();
    let mut req = agent
        .post(&config.endpoint)
        .set("content-type", "application/json")
        .set("idempotency-key", request.request_id.as_str())
        .set("x-selene-device-id", request.device_id.as_str());
    if let Some(token) = config.bearer_token.as_ref() {
        req = req.set("authorization", &format!("Bearer {}", token));
    }
    match req.send_string(&payload) {
        Ok(resp) => {
            if (200..=299).contains(&resp.status()) {
                let text = resp.into_string().map_err(|err| {
                    DeviceArtifactPullError::retryable(
                        format!("pull response decode failed: {}", err),
                        DEVICE_SYNC_PULL_RETRY_AFTER_MS_DEFAULT,
                    )
                })?;
                let parsed: DeviceArtifactPullResponse = serde_json::from_str(text.as_str()).map_err(
                    |err| {
                        DeviceArtifactPullError::retryable(
                            format!("pull response json parse failed: {}", err),
                            DEVICE_SYNC_PULL_RETRY_AFTER_MS_DEFAULT,
                        )
                    },
                )?;
                Ok(parsed)
            } else {
                let retry_after = parse_retry_after_ms(resp.header("retry-after"));
                Err(DeviceArtifactPullError::retryable(
                    format!("pull failed with http status {}", resp.status()),
                    retry_after,
                ))
            }
        }
        Err(ureq::Error::Status(code, resp)) => {
            let retry_after = parse_retry_after_ms(resp.header("retry-after"));
            Err(DeviceArtifactPullError::retryable(
                format!("pull failed with http status {}", code),
                retry_after,
            ))
        }
        Err(ureq::Error::Transport(err)) => Err(DeviceArtifactPullError::retryable(
            format!("pull transport error: {}", err),
            DEVICE_SYNC_PULL_RETRY_AFTER_MS_DEFAULT,
        )),
    }
}

fn send_http_sync_envelope(
    config: &DeviceArtifactSyncHttpSenderConfig,
    envelope: &DeviceArtifactSyncEnvelope,
) -> Result<DeviceArtifactSyncSendReceipt, DeviceArtifactSyncSendError> {
    let payload = serde_json::to_string(envelope).map_err(|err| {
        DeviceArtifactSyncSendError::retryable(
            format!("sync payload encode failed: {}", err),
            DEVICE_SYNC_RETRY_AFTER_MS_DEFAULT,
        )
    })?;
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_millis(config.connect_timeout_ms))
        .timeout_read(Duration::from_millis(config.request_timeout_ms))
        .timeout_write(Duration::from_millis(config.request_timeout_ms))
        .build();
    let mut req = agent
        .post(&config.endpoint)
        .set("content-type", "application/json")
        .set("idempotency-key", &envelope.idempotency_key)
        .set("x-selene-sync-job-id", &envelope.sync_job_id);
    if let Some(token) = config.bearer_token.as_ref() {
        req = req.set("authorization", &format!("Bearer {}", token));
    }
    match req.send_string(&payload) {
        Ok(resp) => {
            if (200..=299).contains(&resp.status()) {
                Ok(DeviceArtifactSyncSendReceipt {
                    remote_ack_ref: Some(format!(
                        "http:{}:{}",
                        resp.status(),
                        envelope.sync_job_id
                    )),
                })
            } else {
                let retry_after = parse_retry_after_ms(resp.header("retry-after"));
                Err(DeviceArtifactSyncSendError::retryable(
                    format!("sync failed with http status {}", resp.status()),
                    retry_after,
                ))
            }
        }
        Err(ureq::Error::Status(code, resp)) => {
            let retry_after = parse_retry_after_ms(resp.header("retry-after"));
            Err(DeviceArtifactSyncSendError::retryable(
                format!("sync failed with http status {}", code),
                retry_after,
            ))
        }
        Err(ureq::Error::Transport(err)) => Err(DeviceArtifactSyncSendError::retryable(
            format!("sync transport error: {}", err),
            DEVICE_SYNC_RETRY_AFTER_MS_DEFAULT,
        )),
    }
}

fn parse_retry_after_ms(retry_after_header: Option<&str>) -> u32 {
    let Some(header) = retry_after_header else {
        return DEVICE_SYNC_RETRY_AFTER_MS_DEFAULT;
    };
    let seconds = header.trim().parse::<u32>().ok();
    seconds
        .map(|s| s.saturating_mul(1_000))
        .filter(|ms| (1_000..=300_000).contains(ms))
        .unwrap_or(DEVICE_SYNC_RETRY_AFTER_MS_DEFAULT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1art::ArtifactVersion;
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
    use selene_kernel_contracts::ph1link::{AppPlatform, InviteeType, Ph1LinkRequest};
    use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus, WakeArtifactApplyState};

    fn user(id: &str) -> UserId {
        UserId::new(id).unwrap()
    }

    fn device(id: &str) -> DeviceId {
        DeviceId::new(id).unwrap()
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
                    Some("audio_profile_sync".to_string()),
                )
                .unwrap(),
            )
            .unwrap();
    }

    fn seed_onboarding_session(store: &mut Ph1fStore, user_id: &UserId, fp: &str) -> String {
        let link_rt = crate::ph1link::Ph1LinkRuntime::new(crate::ph1link::Ph1LinkConfig::mvp_v1());
        let draft_req = Ph1LinkRequest::invite_generate_draft_v1(
            CorrelationId(100),
            TurnId(1),
            MonotonicTimeNs(3),
            user_id.clone(),
            InviteeType::Employee,
            Some("tenant_1".to_string()),
            None,
            None,
            None,
        )
        .unwrap();
        let token_id = match link_rt.run(store, &draft_req).unwrap() {
            selene_kernel_contracts::ph1link::Ph1LinkResponse::Ok(ok) => {
                ok.link_generate_result
                    .expect("link generate result must exist")
                    .token_id
            }
            _ => panic!("expected link generate ok"),
        };
        let token_signature = store
            .ph1link_get_link(&token_id)
            .expect("link must exist after generate")
            .token_signature
            .clone();

        let open_req = Ph1LinkRequest::invite_open_activate_commit_v1(
            CorrelationId(100),
            TurnId(2),
            MonotonicTimeNs(4),
            token_id.clone(),
            token_signature,
            fp.to_string(),
            AppPlatform::Ios,
            "ios_instance_onb".to_string(),
            "nonce_onb".to_string(),
            MonotonicTimeNs(4),
            "idem_sync_open".to_string(),
        )
        .unwrap();
        let _ = link_rt.run(store, &open_req).unwrap();

        let onb = store
            .ph1onb_session_start_draft(
                MonotonicTimeNs(5),
                token_id,
                None,
                Some("tenant_1".to_string()),
                fp.to_string(),
                AppPlatform::Ios,
                "ios_instance_onb".to_string(),
                "nonce_onb".to_string(),
                MonotonicTimeNs(4),
            )
            .unwrap();
        onb.onboarding_session_id.as_str().to_string()
    }

    fn seed_voice_sync_receipt(
        store: &mut Ph1fStore,
        onb_session_id: &str,
        device_id: &DeviceId,
        idempotency_suffix: &str,
    ) -> String {
        let started = store
            .ph1vid_enroll_start_draft(
                MonotonicTimeNs(10),
                selene_kernel_contracts::ph1onb::OnboardingSessionId::new(onb_session_id).unwrap(),
                device_id.clone(),
                true,
                8,
                120_000,
                2,
            )
            .unwrap();
        store
            .ph1vid_enroll_sample_commit(
                MonotonicTimeNs(11),
                started.voice_enrollment_session_id.clone(),
                "audio:sync:1".to_string(),
                1,
                1_350,
                0.93,
                18.0,
                0.2,
                0.0,
                None,
                format!("sync-sample-1-{idempotency_suffix}"),
            )
            .unwrap();
        store
            .ph1vid_enroll_sample_commit(
                MonotonicTimeNs(12),
                started.voice_enrollment_session_id.clone(),
                "audio:sync:2".to_string(),
                2,
                1_360,
                0.94,
                18.1,
                0.2,
                0.0,
                None,
                format!("sync-sample-2-{idempotency_suffix}"),
            )
            .unwrap();
        let completed = store
            .ph1vid_enroll_complete_commit(
                MonotonicTimeNs(13),
                started.voice_enrollment_session_id,
                format!("sync-complete-{idempotency_suffix}"),
            )
            .unwrap();
        completed
            .voice_artifact_sync_receipt_ref
            .expect("voice sync receipt must exist")
    }

    fn pull_update(version: u32, payload: &str, package_hash: Option<&str>, idem: &str) -> DeviceArtifactPullUpdate {
        DeviceArtifactPullUpdate {
            artifact_type: "WakePack".to_string(),
            artifact_version: version,
            payload_ref: format!("inline:{}", payload),
            package_hash: package_hash
                .map(|v| v.to_string())
                .unwrap_or_else(|| sha256_hex(payload.as_bytes())),
            idempotency_key: Some(idem.to_string()),
        }
    }

    fn seed_active_wake_artifact(
        store: &mut Ph1fStore,
        device_id: &DeviceId,
        version: u32,
        payload: &str,
        idem_suffix: &str,
    ) {
        let artifact_version = ArtifactVersion(version);
        let payload_hash = sha256_hex(payload.as_bytes());
        store
            .wake_artifact_stage_commit(
                MonotonicTimeNs(5),
                device_id.clone(),
                artifact_version,
                payload_hash.clone(),
                format!("inline:{}", payload),
                Some(format!("/tmp/wake_cache_seed_{}", idem_suffix)),
                format!("seed-stage-{}", idem_suffix),
            )
            .unwrap();
        store
            .wake_artifact_activate_commit(
                MonotonicTimeNs(6),
                device_id.clone(),
                artifact_version,
                format!("seed-activate-{}", idem_suffix),
            )
            .unwrap();
    }

    #[test]
    fn at_device_sync_worker_01_success_ack_updates_metrics() {
        let mut store = Ph1fStore::new_in_memory();
        let u = user("tenant_1:user_sync_ok");
        let d = device("device_sync_ok");
        seed_identity_and_device(&mut store, &u, &d);
        let onb = seed_onboarding_session(&mut store, &u, "fp_sync_ok");
        let receipt = seed_voice_sync_receipt(&mut store, &onb, &d, "ok");

        let metrics = run_device_artifact_sync_worker_pass_with_metrics_internal(
            &mut store,
            MonotonicTimeNs(100),
            "worker_sync_ok".to_string(),
            &DeviceArtifactSyncSenderRuntime::LoopbackAck,
            &DeviceArtifactPullRuntime::Disabled,
            5,
        )
        .unwrap();

        let row = store
            .mobile_artifact_sync_queue_row_for_receipt(&receipt)
            .expect("queue row must exist");
        assert_eq!(row.state, MobileArtifactSyncState::Acked);
        assert_eq!(metrics.dequeued_count, 1);
        assert_eq!(metrics.acked_count, 1);
        assert_eq!(metrics.retry_scheduled_count, 0);
        assert_eq!(metrics.dead_lettered_count, 0);
    }

    #[test]
    fn at_device_sync_worker_02_failure_schedules_retry_before_max_attempts() {
        let mut store = Ph1fStore::new_in_memory();
        let u = user("tenant_1:user_sync_retry");
        let d = device("device_sync_retry");
        seed_identity_and_device(&mut store, &u, &d);
        let onb = seed_onboarding_session(&mut store, &u, "fp_sync_retry");
        let receipt = seed_voice_sync_receipt(&mut store, &onb, &d, "retry");

        let metrics = run_device_artifact_sync_worker_pass_with_metrics_internal(
            &mut store,
            MonotonicTimeNs(200),
            "worker_sync_retry".to_string(),
            &DeviceArtifactSyncSenderRuntime::always_fail_for_tests("engine_b_timeout", 5_000),
            &DeviceArtifactPullRuntime::Disabled,
            3,
        )
        .unwrap();

        let row = store
            .mobile_artifact_sync_queue_row_for_receipt(&receipt)
            .expect("queue row must exist");
        assert_eq!(row.state, MobileArtifactSyncState::InFlight);
        assert_eq!(row.acked_at, None);
        assert_eq!(row.last_error.as_deref(), Some("engine_b_timeout"));
        assert_eq!(metrics.dequeued_count, 1);
        assert_eq!(metrics.acked_count, 0);
        assert_eq!(metrics.retry_scheduled_count, 1);
        assert_eq!(metrics.dead_lettered_count, 0);
    }

    #[test]
    fn at_device_sync_worker_03_failure_dead_letters_at_max_attempts() {
        let mut store = Ph1fStore::new_in_memory();
        let u = user("tenant_1:user_sync_dead");
        let d = device("device_sync_dead");
        seed_identity_and_device(&mut store, &u, &d);
        let onb = seed_onboarding_session(&mut store, &u, "fp_sync_dead");
        let receipt = seed_voice_sync_receipt(&mut store, &onb, &d, "dead");

        let metrics = run_device_artifact_sync_worker_pass_with_metrics_internal(
            &mut store,
            MonotonicTimeNs(300),
            "worker_sync_dead".to_string(),
            &DeviceArtifactSyncSenderRuntime::always_fail_for_tests("engine_b_down", 5_000),
            &DeviceArtifactPullRuntime::Disabled,
            1,
        )
        .unwrap();

        let row = store
            .mobile_artifact_sync_queue_row_for_receipt(&receipt)
            .expect("queue row must exist");
        assert_eq!(row.state, MobileArtifactSyncState::DeadLetter);
        assert_eq!(row.lease_expires_at, None);
        assert_eq!(row.acked_at, None);
        assert_eq!(row.last_error.as_deref(), Some("engine_b_down"));
        assert_eq!(metrics.dequeued_count, 1);
        assert_eq!(metrics.acked_count, 0);
        assert_eq!(metrics.retry_scheduled_count, 0);
        assert_eq!(metrics.dead_lettered_count, 1);
        assert_eq!(metrics.queue_after.dead_letter_count, 1);
    }

    #[test]
    fn at_device_sync_pull_apply_01_hash_mismatch_rolls_back_and_preserves_last_known_good() {
        let mut store = Ph1fStore::new_in_memory();
        let u = user("tenant_1:user_pull_hash_mismatch");
        let d = device("device_pull_hash_mismatch");
        seed_identity_and_device(&mut store, &u, &d);
        seed_active_wake_artifact(&mut store, &d, 1, "wake_v1_seed", "hash_mismatch");

        let response = DeviceArtifactPullResponse {
            schema_version: 1,
            updates: vec![pull_update(
                2,
                "wake_v2_bad_hash",
                Some("0000000000000000000000000000000000000000000000000000000000000000"),
                "hash-mismatch",
            )],
        };
        let metrics = run_device_artifact_pull_apply_pass_internal(
            &mut store,
            MonotonicTimeNs(50),
            "worker_pull_hash_mismatch",
            &DeviceArtifactPullRuntime::StaticForTests(response),
            None,
        )
        .unwrap();
        assert_eq!(metrics.rollback_count, 1);
        assert_eq!(metrics.activated_count, 0);
        let current = store
            .wake_artifact_apply_current_row(&d)
            .expect("apply current must exist");
        assert_eq!(current.active_artifact_version, Some(ArtifactVersion(1)));
        assert_eq!(current.last_known_good_artifact_version, Some(ArtifactVersion(1)));
        assert_eq!(
            store.wake_artifact_blocked_reason(&d, ArtifactVersion(2)),
            Some(WAKE_ARTIFACT_REASON_HASH_MISMATCH)
        );
    }

    #[test]
    fn at_device_sync_pull_apply_02_successfully_stages_and_activates() {
        let mut store = Ph1fStore::new_in_memory();
        let u = user("tenant_1:user_pull_success");
        let d = device("device_pull_success");
        seed_identity_and_device(&mut store, &u, &d);
        seed_active_wake_artifact(&mut store, &d, 1, "wake_v1_seed", "success");

        let response = DeviceArtifactPullResponse {
            schema_version: 1,
            updates: vec![pull_update(2, "wake_v2_good", None, "pull-success")],
        };
        let metrics = run_device_artifact_pull_apply_pass_internal(
            &mut store,
            MonotonicTimeNs(60),
            "worker_pull_success",
            &DeviceArtifactPullRuntime::StaticForTests(response),
            None,
        )
        .unwrap();
        assert_eq!(metrics.activated_count, 1);
        assert_eq!(metrics.rollback_count, 0);
        let current = store
            .wake_artifact_apply_current_row(&d)
            .expect("apply current must exist");
        assert_eq!(current.active_artifact_version, Some(ArtifactVersion(2)));
        assert_eq!(current.last_known_good_artifact_version, Some(ArtifactVersion(1)));
        assert!(store.wake_artifact_apply_rows().iter().any(|row| {
            row.device_id == d
                && row.artifact_version == ArtifactVersion(2)
                && row.state == WakeArtifactApplyState::Active
        }));
    }

    #[test]
    fn at_device_sync_pull_apply_03_activation_error_rolls_back_to_last_known_good() {
        let mut store = Ph1fStore::new_in_memory();
        let u = user("tenant_1:user_pull_activation_fail");
        let d = device("device_pull_activation_fail");
        seed_identity_and_device(&mut store, &u, &d);
        seed_active_wake_artifact(&mut store, &d, 1, "wake_v1_seed", "activation_fail");

        let response = DeviceArtifactPullResponse {
            schema_version: 1,
            updates: vec![pull_update(2, "wake_v2_hook_fail", None, "pull-hook-fail")],
        };
        let hook = |_device: &DeviceId, _version: ArtifactVersion, _cache_ref: &str| -> Result<(), String> {
            Err("activation_failed".to_string())
        };
        let metrics = run_device_artifact_pull_apply_pass_internal(
            &mut store,
            MonotonicTimeNs(70),
            "worker_pull_activation_fail",
            &DeviceArtifactPullRuntime::StaticForTests(response),
            Some(&hook),
        )
        .unwrap();
        assert_eq!(metrics.rollback_count, 1);
        assert_eq!(metrics.activated_count, 0);
        let current = store
            .wake_artifact_apply_current_row(&d)
            .expect("apply current must exist");
        assert_eq!(current.active_artifact_version, Some(ArtifactVersion(1)));
        assert_eq!(current.last_known_good_artifact_version, Some(ArtifactVersion(1)));
        assert_eq!(
            store.wake_artifact_blocked_reason(&d, ArtifactVersion(2)),
            Some(WAKE_ARTIFACT_REASON_ACTIVATION_FAILED)
        );
    }

    #[test]
    fn at_device_sync_pull_apply_04_applying_same_artifact_twice_is_noop() {
        let mut store = Ph1fStore::new_in_memory();
        let u = user("tenant_1:user_pull_idempotent");
        let d = device("device_pull_idempotent");
        seed_identity_and_device(&mut store, &u, &d);

        let response = DeviceArtifactPullResponse {
            schema_version: 1,
            updates: vec![pull_update(1, "wake_v1_once", None, "pull-once")],
        };
        let first_metrics = run_device_artifact_pull_apply_pass_internal(
            &mut store,
            MonotonicTimeNs(80),
            "worker_pull_idempotent_1",
            &DeviceArtifactPullRuntime::StaticForTests(response.clone()),
            None,
        )
        .unwrap();
        let second_metrics = run_device_artifact_pull_apply_pass_internal(
            &mut store,
            MonotonicTimeNs(81),
            "worker_pull_idempotent_2",
            &DeviceArtifactPullRuntime::StaticForTests(response),
            None,
        )
        .unwrap();

        assert_eq!(first_metrics.activated_count, 1);
        assert_eq!(second_metrics.noop_count, 1);
        assert_eq!(store.wake_artifact_apply_rows().len(), 2);
        let current = store
            .wake_artifact_apply_current_row(&d)
            .expect("apply current must exist");
        assert_eq!(current.active_artifact_version, Some(ArtifactVersion(1)));
    }
}
