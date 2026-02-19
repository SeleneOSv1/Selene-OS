#![forbid(unsafe_code)]

use std::env;
use std::time::Duration;

use selene_kernel_contracts::MonotonicTimeNs;
use selene_storage::ph1f::{
    MobileArtifactSyncKind, MobileArtifactSyncQueueRecord, MobileArtifactSyncState, Ph1fStore,
    StorageError,
};

pub const DEVICE_SYNC_WORKER_MAX_ITEMS: u16 = 16;
pub const DEVICE_SYNC_WORKER_LEASE_MS: u32 = 30_000;
pub const DEVICE_SYNC_RETRY_AFTER_MS_DEFAULT: u32 = 30_000;
pub const DEVICE_SYNC_MAX_ATTEMPTS_DEFAULT: u16 = 5;

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
    pub queue_after: DeviceArtifactSyncQueueMetrics,
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
    run_device_artifact_sync_worker_pass_with_metrics_internal(
        store,
        now,
        worker_id,
        sender,
        device_sync_max_attempts_from_env(),
    )
}

fn run_device_artifact_sync_worker_pass_with_metrics_internal(
    store: &mut Ph1fStore,
    now: MonotonicTimeNs,
    worker_id: String,
    sender: &DeviceArtifactSyncSenderRuntime,
    max_attempts: u16,
) -> Result<DeviceArtifactSyncWorkerPassMetrics, StorageError> {
    let max_attempts = max_attempts.max(1);
    let dequeued = store.device_artifact_sync_dequeue_batch(
        now,
        DEVICE_SYNC_WORKER_MAX_ITEMS,
        DEVICE_SYNC_WORKER_LEASE_MS,
        worker_id.clone(),
    )?;
    let mut metrics = DeviceArtifactSyncWorkerPassMetrics {
        dequeued_count: dequeued.len() as u16,
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
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
    use selene_kernel_contracts::ph1link::{AppPlatform, InviteeType, Ph1LinkRequest};
    use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus};

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

        let open_req = Ph1LinkRequest::invite_open_activate_commit_v1(
            CorrelationId(100),
            TurnId(2),
            MonotonicTimeNs(4),
            token_id.clone(),
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
}
