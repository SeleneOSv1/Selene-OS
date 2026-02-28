#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use selene_kernel_contracts::ph1bcast::{
    BcastAckCommitRequest, BcastAckCommitResult, BcastCancelCommitRequest, BcastCancelCommitResult,
    BcastCapabilityId, BcastDeferCommitRequest, BcastDeferCommitResult, BcastDeliverCommitRequest,
    BcastDeliverCommitResult, BcastDeliveryMethod, BcastDraftCreateRequest, BcastDraftCreateResult,
    BcastEscalateCommitRequest, BcastEscalateCommitResult, BcastExpireCommitRequest,
    BcastExpireCommitResult, BcastOutcome, BcastRecipientRegion, BcastRecipientState,
    BcastReminderFiredCommitRequest, BcastReminderFiredCommitResult, BcastRequest,
    BroadcastClassification, BroadcastId, BroadcastRecipientId, Ph1BcastOk, Ph1BcastRefuse,
    Ph1BcastRequest, Ph1BcastResponse, BCAST_NON_URGENT_FOLLOWUP_WINDOW_NS,
};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.BCAST.001 reason-code namespace. Values are placeholders until registry lock.
    pub const BCAST_DRAFT_CREATED: ReasonCodeId = ReasonCodeId(0x4243_0001);
    pub const BCAST_DELIVERED: ReasonCodeId = ReasonCodeId(0x4243_0002);
    pub const BCAST_DEFERRED: ReasonCodeId = ReasonCodeId(0x4243_0003);
    pub const BCAST_REMINDER_SET: ReasonCodeId = ReasonCodeId(0x4243_0008);
    pub const BCAST_REMINDER_FIRED: ReasonCodeId = ReasonCodeId(0x4243_0009);
    pub const BCAST_FOLLOWUP_IMMEDIATE_URGENT: ReasonCodeId = ReasonCodeId(0x4243_000A);
    pub const BCAST_ACK_RECORDED: ReasonCodeId = ReasonCodeId(0x4243_0004);
    pub const BCAST_ESCALATED: ReasonCodeId = ReasonCodeId(0x4243_0005);
    pub const BCAST_EXPIRED: ReasonCodeId = ReasonCodeId(0x4243_0006);
    pub const BCAST_CANCELED: ReasonCodeId = ReasonCodeId(0x4243_0007);

    pub const BCAST_FAIL_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4243_00F1);
    pub const BCAST_FAIL_NOT_FOUND: ReasonCodeId = ReasonCodeId(0x4243_00F2);
    pub const BCAST_FAIL_STATE_TRANSITION_INVALID: ReasonCodeId = ReasonCodeId(0x4243_00F3);
    pub const BCAST_FAIL_SIMULATION_CONTEXT_MISSING: ReasonCodeId = ReasonCodeId(0x4243_00F4);
    pub const BCAST_FAIL_WAITING_WINDOW_NOT_ELAPSED: ReasonCodeId = ReasonCodeId(0x4243_00F5);
    pub const BCAST_FAIL_FALLBACK_POLICY: ReasonCodeId = ReasonCodeId(0x4243_00F6);
    pub const BCAST_FAIL_INTERNAL: ReasonCodeId = ReasonCodeId(0x4243_00FF);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1BcastConfig {
    pub allow_voice_fallback: bool,
}

impl Ph1BcastConfig {
    pub fn mvp_v1() -> Self {
        Self {
            allow_voice_fallback: false,
        }
    }
}

#[derive(Debug, Clone)]
struct BroadcastRow {
    classification: BroadcastClassification,
    state: BcastRecipientState,
}

#[derive(Debug, Clone)]
struct RecipientRow {
    state: BcastRecipientState,
    last_delivery_method: BcastDeliveryMethod,
    recipient_region: BcastRecipientRegion,
    fallback_step: u8,
    wait_until: Option<MonotonicTimeNs>,
}

#[derive(Debug, Default)]
struct Ph1BcastState {
    next_broadcast_seq: u64,
    broadcasts: BTreeMap<BroadcastId, BroadcastRow>,
    recipients: BTreeMap<(BroadcastId, BroadcastRecipientId), RecipientRow>,
    draft_idempotency: BTreeMap<(String, String, String), BroadcastId>,
    action_idempotency: BTreeMap<(String, String, String, String, String), String>,
}

#[derive(Debug, Clone)]
pub struct Ph1BcastRuntime {
    _config: Ph1BcastConfig,
    state: Arc<Mutex<Ph1BcastState>>,
}

impl Default for Ph1BcastRuntime {
    fn default() -> Self {
        Self::new(Ph1BcastConfig::mvp_v1())
    }
}

impl Ph1BcastRuntime {
    pub fn new(config: Ph1BcastConfig) -> Self {
        Self {
            _config: config,
            state: Arc::new(Mutex::new(Ph1BcastState::default())),
        }
    }

    pub fn run(&self, req: &Ph1BcastRequest) -> Ph1BcastResponse {
        if req.validate().is_err() {
            return refuse(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::BCAST_FAIL_SCHEMA_INVALID,
                "bcast request failed contract validation",
            );
        }

        match &req.request {
            BcastRequest::DraftCreate(r) => self.run_draft_create(req, r),
            BcastRequest::DeliverCommit(r) => self.run_deliver(req, r),
            BcastRequest::DeferCommit(r) => self.run_defer(req, r),
            BcastRequest::ReminderFiredCommit(r) => self.run_reminder_fired(req, r),
            BcastRequest::AckCommit(r) => self.run_ack(req, r),
            BcastRequest::EscalateCommit(r) => self.run_escalate(req, r),
            BcastRequest::ExpireCommit(r) => self.run_expire(req, r),
            BcastRequest::CancelCommit(r) => self.run_cancel(req, r),
        }
    }

    fn run_draft_create(
        &self,
        req: &Ph1BcastRequest,
        r: &BcastDraftCreateRequest,
    ) -> Ph1BcastResponse {
        let mut state = match self.lock_state_or_refuse(req) {
            Ok(state) => state,
            Err(out) => return out,
        };
        let draft_idx = (
            r.tenant_id.as_str().to_string(),
            r.sender_user_id.as_str().to_string(),
            r.idempotency_key.clone(),
        );

        if let Some(existing_id) = state.draft_idempotency.get(&draft_idx).cloned() {
            return ok(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::BCAST_DRAFT_CREATED,
                BcastOutcome::DraftCreate(BcastDraftCreateResult {
                    broadcast_id: existing_id,
                    state: BcastRecipientState::DraftCreated,
                    reason_code: reason_codes::BCAST_DRAFT_CREATED,
                }),
            );
        }

        state.next_broadcast_seq = state.next_broadcast_seq.saturating_add(1);
        let broadcast_id =
            match BroadcastId::new(format!("bcast_{:016x}", state.next_broadcast_seq)) {
                Ok(v) => v,
                Err(_) => {
                    return refuse(
                        req.request.capability_id(),
                        req.simulation_id.clone(),
                        reason_codes::BCAST_FAIL_INTERNAL,
                        "failed to generate broadcast id",
                    )
                }
            };

        state.broadcasts.insert(
            broadcast_id.clone(),
            BroadcastRow {
                classification: r.classification,
                state: BcastRecipientState::DraftCreated,
            },
        );
        state
            .draft_idempotency
            .insert(draft_idx, broadcast_id.clone());

        ok(
            req.request.capability_id(),
            req.simulation_id.clone(),
            reason_codes::BCAST_DRAFT_CREATED,
            BcastOutcome::DraftCreate(BcastDraftCreateResult {
                broadcast_id,
                state: BcastRecipientState::DraftCreated,
                reason_code: reason_codes::BCAST_DRAFT_CREATED,
            }),
        )
    }

    fn run_deliver(
        &self,
        req: &Ph1BcastRequest,
        r: &BcastDeliverCommitRequest,
    ) -> Ph1BcastResponse {
        if r.simulation_context.trim().is_empty() {
            return refuse(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::BCAST_FAIL_SIMULATION_CONTEXT_MISSING,
                "simulation_context is required for BCAST deliver commit",
            );
        }

        let mut state = match self.lock_state_or_refuse(req) {
            Ok(state) => state,
            Err(out) => return out,
        };
        let classification = match state.broadcasts.get(&r.broadcast_id) {
            Some(row) => row.classification,
            None => {
                return refuse(
                    req.request.capability_id(),
                    req.simulation_id.clone(),
                    reason_codes::BCAST_FAIL_NOT_FOUND,
                    "broadcast_id not found",
                )
            }
        };
        let followup_immediate = classification == BroadcastClassification::Emergency;
        let deliver_reason_code = if followup_immediate {
            reason_codes::BCAST_FOLLOWUP_IMMEDIATE_URGENT
        } else {
            reason_codes::BCAST_DELIVERED
        };
        let deliver_state = if followup_immediate {
            BcastRecipientState::Followup
        } else {
            BcastRecipientState::Waiting
        };
        let non_urgent_wait_window_ns = parse_non_urgent_wait_window_ns(&r.simulation_context);

        let action_idx = (
            "DELIVER".to_string(),
            r.tenant_id.as_str().to_string(),
            r.broadcast_id.as_str().to_string(),
            r.recipient_id.as_str().to_string(),
            r.idempotency_key.clone(),
        );
        if let Some(existing_ref) = state.action_idempotency.get(&action_idx).cloned() {
            return ok(
                req.request.capability_id(),
                req.simulation_id.clone(),
                deliver_reason_code,
                BcastOutcome::DeliverCommit(BcastDeliverCommitResult {
                    broadcast_id: r.broadcast_id.clone(),
                    recipient_id: r.recipient_id.clone(),
                    delivery_request_ref: existing_ref,
                    recipient_state: deliver_state,
                    followup_immediate,
                    reason_code: deliver_reason_code,
                }),
            );
        }

        let key = (r.broadcast_id.clone(), r.recipient_id.clone());
        let existing_recipient = state.recipients.get(&key).cloned();
        if let Some(rec) = existing_recipient.as_ref() {
            if matches!(
                rec.state,
                BcastRecipientState::Canceled
                    | BcastRecipientState::Expired
                    | BcastRecipientState::Concluded
            ) {
                return refuse(
                    req.request.capability_id(),
                    req.simulation_id.clone(),
                    reason_codes::BCAST_FAIL_STATE_TRANSITION_INVALID,
                    "cannot deliver for terminal recipient state",
                );
            }
        }
        let fallback_step = match validate_fallback_transition(
            existing_recipient.as_ref(),
            r.delivery_method,
            r.recipient_region,
            r.app_unavailable,
        ) {
            Ok(v) => v,
            Err(message) => {
                return refuse(
                    req.request.capability_id(),
                    req.simulation_id.clone(),
                    reason_codes::BCAST_FAIL_FALLBACK_POLICY,
                    message,
                )
            }
        };

        let delivery_request_ref = format!(
            "delivery_{}",
            short_hash_hex(&[
                r.broadcast_id.as_str(),
                r.recipient_id.as_str(),
                r.delivery_plan_ref.as_str(),
                r.idempotency_key.as_str(),
            ])
        );

        let wait_until = if followup_immediate {
            None
        } else if let Some(existing_wait_until) = existing_recipient
            .as_ref()
            .filter(|existing| existing.state == BcastRecipientState::Waiting)
            .and_then(|existing| existing.wait_until)
        {
            Some(existing_wait_until)
        } else {
            Some(MonotonicTimeNs(
                req.now.0.saturating_add(non_urgent_wait_window_ns),
            ))
        };
        state.recipients.insert(
            key,
            RecipientRow {
                state: deliver_state,
                last_delivery_method: r.delivery_method,
                recipient_region: r.recipient_region,
                fallback_step,
                wait_until,
            },
        );
        if let Some(row) = state.broadcasts.get_mut(&r.broadcast_id) {
            row.state = deliver_state;
        }
        state
            .action_idempotency
            .insert(action_idx, delivery_request_ref.clone());

        ok(
            req.request.capability_id(),
            req.simulation_id.clone(),
            deliver_reason_code,
            BcastOutcome::DeliverCommit(BcastDeliverCommitResult {
                broadcast_id: r.broadcast_id.clone(),
                recipient_id: r.recipient_id.clone(),
                delivery_request_ref,
                recipient_state: deliver_state,
                followup_immediate,
                reason_code: deliver_reason_code,
            }),
        )
    }

    fn run_defer(&self, req: &Ph1BcastRequest, r: &BcastDeferCommitRequest) -> Ph1BcastResponse {
        let mut state = match self.lock_state_or_refuse(req) {
            Ok(state) => state,
            Err(out) => return out,
        };
        if !state.broadcasts.contains_key(&r.broadcast_id) {
            return refuse(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::BCAST_FAIL_NOT_FOUND,
                "broadcast_id not found",
            );
        }

        let key = (r.broadcast_id.clone(), r.recipient_id.clone());
        let rec = match state.recipients.get_mut(&key) {
            Some(v) => v,
            None => {
                return refuse(
                    req.request.capability_id(),
                    req.simulation_id.clone(),
                    reason_codes::BCAST_FAIL_NOT_FOUND,
                    "recipient_id not found for broadcast",
                )
            }
        };

        if r.handoff_to_reminder {
            if !matches!(
                rec.state,
                BcastRecipientState::Followup | BcastRecipientState::ReminderFired
            ) {
                return refuse(
                    req.request.capability_id(),
                    req.simulation_id.clone(),
                    reason_codes::BCAST_FAIL_STATE_TRANSITION_INVALID,
                    "reminder handoff requires followup/reminder-fired state",
                );
            }
        } else if !matches!(
            rec.state,
            BcastRecipientState::Waiting
                | BcastRecipientState::Followup
                | BcastRecipientState::ReminderFired
        ) {
            return refuse(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::BCAST_FAIL_STATE_TRANSITION_INVALID,
                "defer requires waiting/followup/reminder-fired state",
            );
        }

        let next_state = if r.handoff_to_reminder {
            BcastRecipientState::ReminderSet
        } else {
            BcastRecipientState::Deferred
        };
        rec.state = next_state;
        rec.wait_until = None;
        if let Some(row) = state.broadcasts.get_mut(&r.broadcast_id) {
            row.state = next_state;
        }
        state.action_idempotency.insert(
            (
                if r.handoff_to_reminder {
                    "DEFER_REMINDER".to_string()
                } else {
                    "DEFER".to_string()
                },
                r.tenant_id.as_str().to_string(),
                r.broadcast_id.as_str().to_string(),
                r.recipient_id.as_str().to_string(),
                r.idempotency_key.clone(),
            ),
            format!("{}", r.defer_until.0),
        );

        let reason_code = if r.handoff_to_reminder {
            reason_codes::BCAST_REMINDER_SET
        } else {
            reason_codes::BCAST_DEFERRED
        };
        ok(
            req.request.capability_id(),
            req.simulation_id.clone(),
            reason_code,
            BcastOutcome::DeferCommit(BcastDeferCommitResult {
                broadcast_id: r.broadcast_id.clone(),
                recipient_id: r.recipient_id.clone(),
                retry_at: r.defer_until,
                recipient_state: next_state,
                handoff_to_reminder: r.handoff_to_reminder,
                reason_code,
            }),
        )
    }

    fn run_reminder_fired(
        &self,
        req: &Ph1BcastRequest,
        r: &BcastReminderFiredCommitRequest,
    ) -> Ph1BcastResponse {
        let mut state = match self.lock_state_or_refuse(req) {
            Ok(state) => state,
            Err(out) => return out,
        };
        if !state.broadcasts.contains_key(&r.broadcast_id) {
            return refuse(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::BCAST_FAIL_NOT_FOUND,
                "broadcast_id not found",
            );
        }

        let action_idx = (
            "REMINDER_FIRED".to_string(),
            r.tenant_id.as_str().to_string(),
            r.broadcast_id.as_str().to_string(),
            r.recipient_id.as_str().to_string(),
            r.idempotency_key.clone(),
        );
        if let Some(existing_ref) = state.action_idempotency.get(&action_idx).cloned() {
            return ok(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::BCAST_REMINDER_FIRED,
                BcastOutcome::ReminderFiredCommit(BcastReminderFiredCommitResult {
                    broadcast_id: r.broadcast_id.clone(),
                    recipient_id: r.recipient_id.clone(),
                    reminder_ref: existing_ref,
                    recipient_state: BcastRecipientState::ReminderFired,
                    reason_code: reason_codes::BCAST_REMINDER_FIRED,
                }),
            );
        }

        let key = (r.broadcast_id.clone(), r.recipient_id.clone());
        let rec = match state.recipients.get_mut(&key) {
            Some(v) => v,
            None => {
                return refuse(
                    req.request.capability_id(),
                    req.simulation_id.clone(),
                    reason_codes::BCAST_FAIL_NOT_FOUND,
                    "recipient_id not found for broadcast",
                )
            }
        };

        if !matches!(
            rec.state,
            BcastRecipientState::ReminderSet | BcastRecipientState::ReminderFired
        ) {
            return refuse(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::BCAST_FAIL_STATE_TRANSITION_INVALID,
                "reminder-fired requires reminder-set/reminder-fired state",
            );
        }

        rec.state = BcastRecipientState::ReminderFired;
        rec.wait_until = None;
        if let Some(row) = state.broadcasts.get_mut(&r.broadcast_id) {
            row.state = BcastRecipientState::ReminderFired;
        }
        state
            .action_idempotency
            .insert(action_idx, r.reminder_ref.clone());

        ok(
            req.request.capability_id(),
            req.simulation_id.clone(),
            reason_codes::BCAST_REMINDER_FIRED,
            BcastOutcome::ReminderFiredCommit(BcastReminderFiredCommitResult {
                broadcast_id: r.broadcast_id.clone(),
                recipient_id: r.recipient_id.clone(),
                reminder_ref: r.reminder_ref.clone(),
                recipient_state: BcastRecipientState::ReminderFired,
                reason_code: reason_codes::BCAST_REMINDER_FIRED,
            }),
        )
    }

    fn run_ack(&self, req: &Ph1BcastRequest, r: &BcastAckCommitRequest) -> Ph1BcastResponse {
        let mut state = match self.lock_state_or_refuse(req) {
            Ok(state) => state,
            Err(out) => return out,
        };
        if !state.broadcasts.contains_key(&r.broadcast_id) {
            return refuse(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::BCAST_FAIL_NOT_FOUND,
                "broadcast_id not found",
            );
        }

        let key = (r.broadcast_id.clone(), r.recipient_id.clone());
        let rec = match state.recipients.get_mut(&key) {
            Some(v) => v,
            None => {
                return refuse(
                    req.request.capability_id(),
                    req.simulation_id.clone(),
                    reason_codes::BCAST_FAIL_NOT_FOUND,
                    "recipient_id not found for broadcast",
                )
            }
        };
        if matches!(
            rec.state,
            BcastRecipientState::Canceled | BcastRecipientState::Expired
        ) {
            return refuse(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::BCAST_FAIL_STATE_TRANSITION_INVALID,
                "ack not allowed from terminal canceled/expired state",
            );
        }
        rec.state = BcastRecipientState::Concluded;
        rec.wait_until = None;
        if let Some(row) = state.broadcasts.get_mut(&r.broadcast_id) {
            row.state = BcastRecipientState::Concluded;
        }

        ok(
            req.request.capability_id(),
            req.simulation_id.clone(),
            reason_codes::BCAST_ACK_RECORDED,
            BcastOutcome::AckCommit(BcastAckCommitResult {
                broadcast_id: r.broadcast_id.clone(),
                recipient_id: r.recipient_id.clone(),
                ack_status: r.ack_status,
                recipient_state: BcastRecipientState::Concluded,
                reason_code: reason_codes::BCAST_ACK_RECORDED,
            }),
        )
    }

    fn run_escalate(
        &self,
        req: &Ph1BcastRequest,
        r: &BcastEscalateCommitRequest,
    ) -> Ph1BcastResponse {
        let mut state = match self.lock_state_or_refuse(req) {
            Ok(state) => state,
            Err(out) => return out,
        };
        if !state.broadcasts.contains_key(&r.broadcast_id) {
            return refuse(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::BCAST_FAIL_NOT_FOUND,
                "broadcast_id not found",
            );
        }
        let key = (r.broadcast_id.clone(), r.recipient_id.clone());
        let rec = match state.recipients.get_mut(&key) {
            Some(v) => v,
            None => {
                return refuse(
                    req.request.capability_id(),
                    req.simulation_id.clone(),
                    reason_codes::BCAST_FAIL_NOT_FOUND,
                    "recipient_id not found for broadcast",
                )
            }
        };
        if !matches!(
            rec.state,
            BcastRecipientState::Waiting | BcastRecipientState::Deferred
        ) {
            return refuse(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::BCAST_FAIL_STATE_TRANSITION_INVALID,
                "escalate requires waiting/deferred state",
            );
        }
        if rec.state == BcastRecipientState::Waiting {
            if let Some(wait_until) = rec.wait_until {
                if req.now.0 < wait_until.0 {
                    return refuse(
                        req.request.capability_id(),
                        req.simulation_id.clone(),
                        reason_codes::BCAST_FAIL_WAITING_WINDOW_NOT_ELAPSED,
                        "non-urgent waiting window has not elapsed",
                    );
                }
            }
        }
        rec.state = BcastRecipientState::Followup;
        rec.wait_until = None;
        if let Some(row) = state.broadcasts.get_mut(&r.broadcast_id) {
            row.state = BcastRecipientState::Followup;
        }

        let sender_notice_ref = format!(
            "sender_notice_{}",
            short_hash_hex(&[
                r.broadcast_id.as_str(),
                r.recipient_id.as_str(),
                r.escalation_reason.as_str(),
                r.idempotency_key.as_str(),
            ])
        );

        ok(
            req.request.capability_id(),
            req.simulation_id.clone(),
            reason_codes::BCAST_ESCALATED,
            BcastOutcome::EscalateCommit(BcastEscalateCommitResult {
                broadcast_id: r.broadcast_id.clone(),
                recipient_id: r.recipient_id.clone(),
                sender_notice_ref,
                recipient_state: BcastRecipientState::Followup,
                reason_code: reason_codes::BCAST_ESCALATED,
            }),
        )
    }

    fn run_expire(&self, req: &Ph1BcastRequest, r: &BcastExpireCommitRequest) -> Ph1BcastResponse {
        let mut state = match self.lock_state_or_refuse(req) {
            Ok(state) => state,
            Err(out) => return out,
        };
        let row = match state.broadcasts.get_mut(&r.broadcast_id) {
            Some(v) => v,
            None => {
                return refuse(
                    req.request.capability_id(),
                    req.simulation_id.clone(),
                    reason_codes::BCAST_FAIL_NOT_FOUND,
                    "broadcast_id not found",
                )
            }
        };
        row.state = BcastRecipientState::Expired;
        for ((bid, _), rec) in &mut state.recipients {
            if bid == &r.broadcast_id {
                rec.state = BcastRecipientState::Expired;
                rec.wait_until = None;
            }
        }

        ok(
            req.request.capability_id(),
            req.simulation_id.clone(),
            reason_codes::BCAST_EXPIRED,
            BcastOutcome::ExpireCommit(BcastExpireCommitResult {
                broadcast_id: r.broadcast_id.clone(),
                state: BcastRecipientState::Expired,
                reason_code: reason_codes::BCAST_EXPIRED,
            }),
        )
    }

    fn run_cancel(&self, req: &Ph1BcastRequest, r: &BcastCancelCommitRequest) -> Ph1BcastResponse {
        let mut state = match self.lock_state_or_refuse(req) {
            Ok(state) => state,
            Err(out) => return out,
        };
        let row = match state.broadcasts.get_mut(&r.broadcast_id) {
            Some(v) => v,
            None => {
                return refuse(
                    req.request.capability_id(),
                    req.simulation_id.clone(),
                    reason_codes::BCAST_FAIL_NOT_FOUND,
                    "broadcast_id not found",
                )
            }
        };
        row.state = BcastRecipientState::Canceled;
        for ((bid, _), rec) in &mut state.recipients {
            if bid == &r.broadcast_id {
                rec.state = BcastRecipientState::Canceled;
                rec.wait_until = None;
            }
        }

        ok(
            req.request.capability_id(),
            req.simulation_id.clone(),
            reason_codes::BCAST_CANCELED,
            BcastOutcome::CancelCommit(BcastCancelCommitResult {
                broadcast_id: r.broadcast_id.clone(),
                state: BcastRecipientState::Canceled,
                reason_code: reason_codes::BCAST_CANCELED,
            }),
        )
    }

    fn lock_state_or_refuse(
        &self,
        req: &Ph1BcastRequest,
    ) -> Result<std::sync::MutexGuard<'_, Ph1BcastState>, Ph1BcastResponse> {
        match self.state.lock() {
            Ok(guard) => Ok(guard),
            Err(poisoned) => {
                let recovered = poisoned.into_inner();
                drop(recovered);
                self.state.clear_poison();
                Err(refuse(
                    req.request.capability_id(),
                    req.simulation_id.clone(),
                    reason_codes::BCAST_FAIL_INTERNAL,
                    "bcast runtime state lock poisoned",
                ))
            }
        }
    }
}

fn validate_fallback_transition(
    existing: Option<&RecipientRow>,
    requested_method: BcastDeliveryMethod,
    requested_region: BcastRecipientRegion,
    app_unavailable: bool,
) -> Result<u8, &'static str> {
    if requested_method == BcastDeliveryMethod::SeleneApp {
        if app_unavailable {
            return Err("app_unavailable must be false for SeleneApp delivery");
        }
        if existing
            .map(|r| {
                r.last_delivery_method != BcastDeliveryMethod::SeleneApp && r.fallback_step > 0
            })
            .unwrap_or(false)
        {
            return Err("cannot return to SeleneApp delivery after fallback path started");
        }
        return Ok(0);
    }

    if !app_unavailable {
        return Err("fallback delivery requires app_unavailable=true");
    }

    let step = match fallback_step_for_method(requested_method, requested_region) {
        Some(v) => v,
        None => {
            return Err("requested fallback delivery method is not allowed for recipient region")
        }
    };

    let previous_step = existing.map(|r| r.fallback_step).unwrap_or(0);
    if let Some(prev) = existing {
        if prev.recipient_region != requested_region {
            return Err("recipient fallback region must remain stable for a broadcast recipient");
        }
    }
    if step < previous_step {
        return Err("fallback delivery method order cannot move backwards");
    }
    if step > previous_step.saturating_add(1) {
        return Err("fallback delivery method order cannot skip required steps");
    }

    Ok(step)
}

fn fallback_step_for_method(
    method: BcastDeliveryMethod,
    region: BcastRecipientRegion,
) -> Option<u8> {
    match region {
        BcastRecipientRegion::Global => match method {
            BcastDeliveryMethod::Sms => Some(1),
            BcastDeliveryMethod::Whatsapp => Some(2),
            BcastDeliveryMethod::Email => Some(3),
            _ => None,
        },
        BcastRecipientRegion::China => match method {
            BcastDeliveryMethod::Sms => Some(1),
            BcastDeliveryMethod::Wechat => Some(2),
            BcastDeliveryMethod::Email => Some(3),
            _ => None,
        },
    }
}

fn ok(
    capability_id: BcastCapabilityId,
    simulation_id: String,
    reason_code: ReasonCodeId,
    outcome: BcastOutcome,
) -> Ph1BcastResponse {
    Ph1BcastResponse::Ok(
        Ph1BcastOk::v1(
            capability_id,
            simulation_id,
            reason_code,
            outcome,
            true,
            true,
        )
        .expect("ph1bcast ok output must validate"),
    )
}

fn refuse(
    capability_id: BcastCapabilityId,
    simulation_id: String,
    reason_code: ReasonCodeId,
    message: impl Into<String>,
) -> Ph1BcastResponse {
    Ph1BcastResponse::Refuse(
        Ph1BcastRefuse::v1(capability_id, simulation_id, reason_code, message.into())
            .expect("ph1bcast refuse output must validate"),
    )
}

fn short_hash_hex(parts: &[&str]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for p in parts {
        for b in p.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x1000_0000_01b3);
        }
        h ^= 0xff;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    format!("{:016x}", h)
}

fn parse_non_urgent_wait_window_ns(simulation_context: &str) -> u64 {
    for token in simulation_context.split(';').map(str::trim) {
        let Some(raw_seconds) = token.strip_prefix("non_urgent_wait_seconds=") else {
            continue;
        };
        let Ok(wait_seconds) = raw_seconds.trim().parse::<u64>() else {
            continue;
        };
        return wait_seconds.saturating_mul(1_000_000_000);
    }
    BCAST_NON_URGENT_FOLLOWUP_WINDOW_NS
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1bcast::{
        BcastAckStatus, BcastCapabilityId, BcastRequest, BcastSimulationType, BroadcastId,
        BroadcastRecipientId, BCAST_ACK_COMMIT, BCAST_CANCEL_COMMIT, BCAST_CREATE_DRAFT,
        BCAST_DEFER_COMMIT, BCAST_DELIVER_COMMIT, BCAST_ESCALATE_COMMIT,
        BCAST_REMINDER_FIRED_COMMIT, PH1BCAST_CONTRACT_VERSION,
    };
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::MonotonicTimeNs;

    fn tenant() -> TenantId {
        TenantId::new("tenant_bcast").unwrap()
    }

    fn sender() -> UserId {
        UserId::new("sender_bcast").unwrap()
    }

    fn recipient_user() -> UserId {
        UserId::new("recipient_user").unwrap()
    }

    fn recipient_id() -> BroadcastRecipientId {
        BroadcastRecipientId::new("recipient_01").unwrap()
    }

    fn req(
        simulation_id: &str,
        simulation_type: BcastSimulationType,
        request: BcastRequest,
    ) -> Ph1BcastRequest {
        Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id: CorrelationId(300),
            turn_id: TurnId(12),
            now: MonotonicTimeNs(100),
            simulation_id: simulation_id.to_string(),
            simulation_type,
            request,
        }
    }

    fn draft_req_with_classification(
        idem: &str,
        classification: BroadcastClassification,
    ) -> Ph1BcastRequest {
        req(
            BCAST_CREATE_DRAFT,
            BcastSimulationType::Draft,
            BcastRequest::DraftCreate(BcastDraftCreateRequest {
                tenant_id: tenant(),
                sender_user_id: sender(),
                audience_spec: "jd".to_string(),
                classification,
                content_payload_ref: "payload_a".to_string(),
                prompt_dedupe_key: Some("pd_1".to_string()),
                idempotency_key: idem.to_string(),
            }),
        )
    }

    fn draft_req(idem: &str) -> Ph1BcastRequest {
        draft_req_with_classification(idem, BroadcastClassification::Priority)
    }

    fn deliver_req_with_options(
        broadcast_id: BroadcastId,
        idem: &str,
        delivery_method: BcastDeliveryMethod,
        recipient_region: BcastRecipientRegion,
        app_unavailable: bool,
    ) -> Ph1BcastRequest {
        req(
            BCAST_DELIVER_COMMIT,
            BcastSimulationType::Commit,
            BcastRequest::DeliverCommit(BcastDeliverCommitRequest {
                tenant_id: tenant(),
                sender_user_id: sender(),
                broadcast_id,
                recipient_id: recipient_id(),
                delivery_method,
                recipient_region,
                app_unavailable,
                app_unavailable_proof_ref: if app_unavailable {
                    Some("app_unavailable_proof:test".to_string())
                } else {
                    None
                },
                delivery_plan_ref: "plan_1".to_string(),
                simulation_context: "sim_ctx".to_string(),
                idempotency_key: idem.to_string(),
            }),
        )
    }

    fn deliver_req(broadcast_id: BroadcastId, idem: &str) -> Ph1BcastRequest {
        deliver_req_with_options(
            broadcast_id,
            idem,
            BcastDeliveryMethod::SeleneApp,
            BcastRecipientRegion::Global,
            false,
        )
    }

    #[test]
    fn at_bcast_01_draft_create_idempotency_returns_same_broadcast_id() {
        let rt = Ph1BcastRuntime::default();
        let out_a = rt.run(&draft_req("idem_draft"));
        let out_b = rt.run(&draft_req("idem_draft"));
        assert!(out_a.validate().is_ok());
        assert!(out_b.validate().is_ok());

        let (id_a, id_b) = match (out_a, out_b) {
            (Ph1BcastResponse::Ok(a), Ph1BcastResponse::Ok(b)) => match (a.outcome, b.outcome) {
                (BcastOutcome::DraftCreate(ra), BcastOutcome::DraftCreate(rb)) => {
                    (ra.broadcast_id, rb.broadcast_id)
                }
                _ => panic!("expected draft outcomes"),
            },
            _ => panic!("expected ok responses"),
        };
        assert_eq!(id_a, id_b);
    }

    #[test]
    fn at_bcast_02_deliver_fails_closed_for_unknown_broadcast() {
        let rt = Ph1BcastRuntime::default();
        let out = rt.run(&deliver_req(
            BroadcastId::new("bcast_missing").unwrap(),
            "idem_2",
        ));
        assert!(out.validate().is_ok());
        match out {
            Ph1BcastResponse::Refuse(r) => {
                assert_eq!(r.reason_code, reason_codes::BCAST_FAIL_NOT_FOUND);
                assert_eq!(r.capability_id, BcastCapabilityId::DeliverCommit);
            }
            _ => panic!("expected refuse response"),
        }
    }

    #[test]
    fn at_bcast_03_deliver_idempotency_returns_same_delivery_ref() {
        let rt = Ph1BcastRuntime::default();
        let draft = rt.run(&draft_req("idem_draft_2"));
        let broadcast_id = match draft {
            Ph1BcastResponse::Ok(v) => match v.outcome {
                BcastOutcome::DraftCreate(r) => r.broadcast_id,
                _ => panic!("expected draft create result"),
            },
            _ => panic!("expected draft ok"),
        };

        let out_a = rt.run(&deliver_req(broadcast_id.clone(), "idem_deliver"));
        let out_b = rt.run(&deliver_req(broadcast_id, "idem_deliver"));
        assert!(out_a.validate().is_ok());
        assert!(out_b.validate().is_ok());

        let (ref_a, ref_b) = match (out_a, out_b) {
            (Ph1BcastResponse::Ok(a), Ph1BcastResponse::Ok(b)) => match (a.outcome, b.outcome) {
                (BcastOutcome::DeliverCommit(ra), BcastOutcome::DeliverCommit(rb)) => {
                    (ra.delivery_request_ref, rb.delivery_request_ref)
                }
                _ => panic!("expected deliver outcomes"),
            },
            _ => panic!("expected ok responses"),
        };
        assert_eq!(ref_a, ref_b);
    }

    #[test]
    fn at_bcast_04_ack_transitions_recipient_to_concluded() {
        let rt = Ph1BcastRuntime::default();
        let draft = rt.run(&draft_req("idem_draft_3"));
        let broadcast_id = match draft {
            Ph1BcastResponse::Ok(v) => match v.outcome {
                BcastOutcome::DraftCreate(r) => r.broadcast_id,
                _ => panic!("expected draft create result"),
            },
            _ => panic!("expected draft ok"),
        };
        let _deliver = rt.run(&deliver_req(broadcast_id.clone(), "idem_deliver_3"));

        let ack_req = req(
            BCAST_ACK_COMMIT,
            BcastSimulationType::Commit,
            BcastRequest::AckCommit(BcastAckCommitRequest {
                tenant_id: tenant(),
                recipient_user_id: recipient_user(),
                broadcast_id,
                recipient_id: recipient_id(),
                ack_status: BcastAckStatus::Received,
                idempotency_key: "idem_ack_1".to_string(),
            }),
        );
        let ack_out = rt.run(&ack_req);
        assert!(ack_out.validate().is_ok());
        match ack_out {
            Ph1BcastResponse::Ok(ok) => match ok.outcome {
                BcastOutcome::AckCommit(v) => {
                    assert_eq!(v.recipient_state, BcastRecipientState::Concluded);
                }
                _ => panic!("expected ack outcome"),
            },
            _ => panic!("expected ok response"),
        }
    }

    #[test]
    fn at_bcast_05_cancel_blocks_future_deliver() {
        let rt = Ph1BcastRuntime::default();
        let draft = rt.run(&draft_req("idem_draft_4"));
        let broadcast_id = match draft {
            Ph1BcastResponse::Ok(v) => match v.outcome {
                BcastOutcome::DraftCreate(r) => r.broadcast_id,
                _ => panic!("expected draft create result"),
            },
            _ => panic!("expected draft ok"),
        };
        let _deliver = rt.run(&deliver_req(broadcast_id.clone(), "idem_deliver_4"));

        let cancel_req = req(
            BCAST_CANCEL_COMMIT,
            BcastSimulationType::Commit,
            BcastRequest::CancelCommit(BcastCancelCommitRequest {
                tenant_id: tenant(),
                sender_user_id: sender(),
                broadcast_id: broadcast_id.clone(),
                cancel_reason: "user canceled".to_string(),
                idempotency_key: "idem_cancel_4".to_string(),
            }),
        );
        let cancel_out = rt.run(&cancel_req);
        assert!(cancel_out.validate().is_ok());

        let deliver_again = rt.run(&deliver_req(broadcast_id, "idem_deliver_5"));
        assert!(deliver_again.validate().is_ok());
        match deliver_again {
            Ph1BcastResponse::Refuse(r) => {
                assert_eq!(
                    r.reason_code,
                    reason_codes::BCAST_FAIL_STATE_TRANSITION_INVALID
                );
            }
            _ => panic!("expected refused delivery after cancel"),
        }
    }

    #[test]
    fn at_bcast_06_defer_and_escalate_paths_are_reason_coded() {
        let rt = Ph1BcastRuntime::default();
        let draft = rt.run(&draft_req("idem_draft_6"));
        let broadcast_id = match draft {
            Ph1BcastResponse::Ok(v) => match v.outcome {
                BcastOutcome::DraftCreate(r) => r.broadcast_id,
                _ => panic!("expected draft create result"),
            },
            _ => panic!("expected draft ok"),
        };
        let _deliver = rt.run(&deliver_req(broadcast_id.clone(), "idem_deliver_6"));

        let defer_req = req(
            BCAST_DEFER_COMMIT,
            BcastSimulationType::Commit,
            BcastRequest::DeferCommit(BcastDeferCommitRequest {
                tenant_id: tenant(),
                sender_user_id: sender(),
                broadcast_id: broadcast_id.clone(),
                recipient_id: recipient_id(),
                defer_until: MonotonicTimeNs(999),
                handoff_to_reminder: false,
                idempotency_key: "idem_defer_6".to_string(),
            }),
        );
        let defer_out = rt.run(&defer_req);
        assert!(defer_out.validate().is_ok());
        match defer_out {
            Ph1BcastResponse::Ok(v) => match v.outcome {
                BcastOutcome::DeferCommit(r) => {
                    assert_eq!(r.reason_code, reason_codes::BCAST_DEFERRED);
                }
                _ => panic!("expected defer result"),
            },
            _ => panic!("expected defer ok"),
        }

        let escalate_req = req(
            BCAST_ESCALATE_COMMIT,
            BcastSimulationType::Commit,
            BcastRequest::EscalateCommit(BcastEscalateCommitRequest {
                tenant_id: tenant(),
                sender_user_id: sender(),
                broadcast_id,
                recipient_id: recipient_id(),
                escalation_reason: "timeout_no_ack".to_string(),
                idempotency_key: "idem_escalate_6".to_string(),
            }),
        );
        let escalate_out = rt.run(&escalate_req);
        assert!(escalate_out.validate().is_ok());
        match escalate_out {
            Ph1BcastResponse::Ok(v) => match v.outcome {
                BcastOutcome::EscalateCommit(r) => {
                    assert_eq!(r.reason_code, reason_codes::BCAST_ESCALATED);
                }
                _ => panic!("expected escalate result"),
            },
            _ => panic!("expected escalate ok"),
        }
    }

    #[test]
    fn at_bcast_07_reminder_handoff_and_fire_follow_mhp_states() {
        let rt = Ph1BcastRuntime::default();
        let draft = rt.run(&draft_req_with_classification(
            "idem_draft_7",
            BroadcastClassification::Emergency,
        ));
        let broadcast_id = match draft {
            Ph1BcastResponse::Ok(v) => match v.outcome {
                BcastOutcome::DraftCreate(r) => r.broadcast_id,
                _ => panic!("expected draft create result"),
            },
            _ => panic!("expected draft ok"),
        };
        let deliver = rt.run(&deliver_req(broadcast_id.clone(), "idem_deliver_7"));
        assert!(deliver.validate().is_ok());
        match deliver {
            Ph1BcastResponse::Ok(v) => match v.outcome {
                BcastOutcome::DeliverCommit(r) => {
                    assert_eq!(r.recipient_state, BcastRecipientState::Followup);
                    assert!(r.followup_immediate);
                    assert_eq!(r.reason_code, reason_codes::BCAST_FOLLOWUP_IMMEDIATE_URGENT);
                }
                _ => panic!("expected deliver result"),
            },
            _ => panic!("expected deliver ok"),
        }

        let defer_req = req(
            BCAST_DEFER_COMMIT,
            BcastSimulationType::Commit,
            BcastRequest::DeferCommit(BcastDeferCommitRequest {
                tenant_id: tenant(),
                sender_user_id: sender(),
                broadcast_id: broadcast_id.clone(),
                recipient_id: recipient_id(),
                defer_until: MonotonicTimeNs(1_000),
                handoff_to_reminder: true,
                idempotency_key: "idem_defer_7".to_string(),
            }),
        );
        let defer_out = rt.run(&defer_req);
        assert!(defer_out.validate().is_ok());
        match defer_out {
            Ph1BcastResponse::Ok(v) => match v.outcome {
                BcastOutcome::DeferCommit(r) => {
                    assert_eq!(r.recipient_state, BcastRecipientState::ReminderSet);
                    assert!(r.handoff_to_reminder);
                    assert_eq!(r.reason_code, reason_codes::BCAST_REMINDER_SET);
                }
                _ => panic!("expected defer result"),
            },
            _ => panic!("expected defer ok"),
        }

        let reminder_fired_req = req(
            BCAST_REMINDER_FIRED_COMMIT,
            BcastSimulationType::Commit,
            BcastRequest::ReminderFiredCommit(BcastReminderFiredCommitRequest {
                tenant_id: tenant(),
                sender_user_id: sender(),
                broadcast_id,
                recipient_id: recipient_id(),
                reminder_ref: "rem_7".to_string(),
                idempotency_key: "idem_reminder_fired_7".to_string(),
            }),
        );
        let reminder_fired_out = rt.run(&reminder_fired_req);
        assert!(reminder_fired_out.validate().is_ok());
        match reminder_fired_out {
            Ph1BcastResponse::Ok(v) => match v.outcome {
                BcastOutcome::ReminderFiredCommit(r) => {
                    assert_eq!(r.recipient_state, BcastRecipientState::ReminderFired);
                    assert_eq!(r.reason_code, reason_codes::BCAST_REMINDER_FIRED);
                }
                _ => panic!("expected reminder fired result"),
            },
            _ => panic!("expected reminder fired ok"),
        }
    }

    #[test]
    fn at_bcast_08_non_urgent_waiting_requires_5m_before_followup_escalate() {
        let rt = Ph1BcastRuntime::default();
        let draft = rt.run(&draft_req("idem_draft_8"));
        let broadcast_id = match draft {
            Ph1BcastResponse::Ok(v) => match v.outcome {
                BcastOutcome::DraftCreate(r) => r.broadcast_id,
                _ => panic!("expected draft create result"),
            },
            _ => panic!("expected draft ok"),
        };

        let deliver = rt.run(&deliver_req(broadcast_id.clone(), "idem_deliver_8"));
        assert!(deliver.validate().is_ok());
        match deliver {
            Ph1BcastResponse::Ok(v) => match v.outcome {
                BcastOutcome::DeliverCommit(r) => {
                    assert_eq!(r.recipient_state, BcastRecipientState::Waiting);
                    assert!(!r.followup_immediate);
                    assert_eq!(r.reason_code, reason_codes::BCAST_DELIVERED);
                }
                _ => panic!("expected deliver result"),
            },
            _ => panic!("expected deliver ok"),
        }

        let escalate_too_early = req(
            BCAST_ESCALATE_COMMIT,
            BcastSimulationType::Commit,
            BcastRequest::EscalateCommit(BcastEscalateCommitRequest {
                tenant_id: tenant(),
                sender_user_id: sender(),
                broadcast_id: broadcast_id.clone(),
                recipient_id: recipient_id(),
                escalation_reason: "waiting_timeout".to_string(),
                idempotency_key: "idem_escalate_early_8".to_string(),
            }),
        );
        let early_out = rt.run(&escalate_too_early);
        assert!(early_out.validate().is_ok());
        match early_out {
            Ph1BcastResponse::Refuse(r) => {
                assert_eq!(
                    r.reason_code,
                    reason_codes::BCAST_FAIL_WAITING_WINDOW_NOT_ELAPSED
                );
            }
            _ => panic!("expected early escalation refusal"),
        }

        let escalate_after_window = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id: CorrelationId(300),
            turn_id: TurnId(12),
            now: MonotonicTimeNs(100 + BCAST_NON_URGENT_FOLLOWUP_WINDOW_NS + 1),
            simulation_id: BCAST_ESCALATE_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Commit,
            request: BcastRequest::EscalateCommit(BcastEscalateCommitRequest {
                tenant_id: tenant(),
                sender_user_id: sender(),
                broadcast_id,
                recipient_id: recipient_id(),
                escalation_reason: "waiting_timeout".to_string(),
                idempotency_key: "idem_escalate_late_8".to_string(),
            }),
        };
        let late_out = rt.run(&escalate_after_window);
        assert!(late_out.validate().is_ok());
        match late_out {
            Ph1BcastResponse::Ok(v) => match v.outcome {
                BcastOutcome::EscalateCommit(r) => {
                    assert_eq!(r.recipient_state, BcastRecipientState::Followup);
                }
                _ => panic!("expected escalate result"),
            },
            _ => panic!("expected late escalation success"),
        }
    }

    #[test]
    fn at_bcast_09_fallback_order_is_locked_and_app_unavailable_gated() {
        let rt = Ph1BcastRuntime::default();
        let draft = rt.run(&draft_req("idem_draft_9"));
        let broadcast_id = match draft {
            Ph1BcastResponse::Ok(v) => match v.outcome {
                BcastOutcome::DraftCreate(r) => r.broadcast_id,
                _ => panic!("expected draft create result"),
            },
            _ => panic!("expected draft ok"),
        };

        let invalid_no_unavailable = rt.run(&deliver_req_with_options(
            broadcast_id.clone(),
            "idem_deliver_9_a",
            BcastDeliveryMethod::Sms,
            BcastRecipientRegion::Global,
            false,
        ));
        assert!(invalid_no_unavailable.validate().is_ok());
        assert!(matches!(
            invalid_no_unavailable,
            Ph1BcastResponse::Refuse(_)
        ));

        let invalid_no_proof = rt.run(&req(
            BCAST_DELIVER_COMMIT,
            BcastSimulationType::Commit,
            BcastRequest::DeliverCommit(BcastDeliverCommitRequest {
                tenant_id: tenant(),
                sender_user_id: sender(),
                broadcast_id: broadcast_id.clone(),
                recipient_id: recipient_id(),
                delivery_method: BcastDeliveryMethod::Sms,
                recipient_region: BcastRecipientRegion::Global,
                app_unavailable: true,
                app_unavailable_proof_ref: None,
                delivery_plan_ref: "plan_1".to_string(),
                simulation_context: "sim_ctx".to_string(),
                idempotency_key: "idem_deliver_9_missing_proof".to_string(),
            }),
        ));
        assert!(invalid_no_proof.validate().is_ok());
        match invalid_no_proof {
            Ph1BcastResponse::Refuse(r) => {
                assert_eq!(r.reason_code, reason_codes::BCAST_FAIL_SCHEMA_INVALID);
            }
            _ => panic!("expected missing app-unavailable proof refusal"),
        }

        let sms_ok = rt.run(&deliver_req_with_options(
            broadcast_id.clone(),
            "idem_deliver_9_b",
            BcastDeliveryMethod::Sms,
            BcastRecipientRegion::Global,
            true,
        ));
        assert!(sms_ok.validate().is_ok());
        assert!(matches!(sms_ok, Ph1BcastResponse::Ok(_)));

        let invalid_skip = rt.run(&deliver_req_with_options(
            broadcast_id.clone(),
            "idem_deliver_9_c",
            BcastDeliveryMethod::Email,
            BcastRecipientRegion::Global,
            true,
        ));
        assert!(invalid_skip.validate().is_ok());
        match invalid_skip {
            Ph1BcastResponse::Refuse(r) => {
                assert_eq!(r.reason_code, reason_codes::BCAST_FAIL_FALLBACK_POLICY);
            }
            _ => panic!("expected fallback ordering refusal"),
        }

        let whatsapp_ok = rt.run(&deliver_req_with_options(
            broadcast_id.clone(),
            "idem_deliver_9_d",
            BcastDeliveryMethod::Whatsapp,
            BcastRecipientRegion::Global,
            true,
        ));
        assert!(whatsapp_ok.validate().is_ok());
        assert!(matches!(whatsapp_ok, Ph1BcastResponse::Ok(_)));

        let email_ok = rt.run(&deliver_req_with_options(
            broadcast_id,
            "idem_deliver_9_e",
            BcastDeliveryMethod::Email,
            BcastRecipientRegion::Global,
            true,
        ));
        assert!(email_ok.validate().is_ok());
        assert!(matches!(email_ok, Ph1BcastResponse::Ok(_)));
    }

    #[test]
    fn at_bcast_10_mutex_poison_fails_closed_without_panic() {
        let rt = Ph1BcastRuntime::default();
        let poisoned_state = rt.state.clone();
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _guard = poisoned_state.lock().unwrap();
            panic!("intentional poison");
        }));

        let out = rt.run(&draft_req("idem_poison_10"));
        assert!(out.validate().is_ok());
        match out {
            Ph1BcastResponse::Refuse(r) => {
                assert_eq!(r.reason_code, reason_codes::BCAST_FAIL_INTERNAL);
            }
            _ => panic!("expected internal refuse on poisoned lock"),
        }

        let recovered = rt.run(&draft_req("idem_poison_10_recovered"));
        assert!(recovered.validate().is_ok());
        assert!(matches!(recovered, Ph1BcastResponse::Ok(_)));
    }
}
