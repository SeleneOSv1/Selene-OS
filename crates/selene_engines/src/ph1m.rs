#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1_voice_id::Ph1VoiceIdResponse;
use selene_kernel_contracts::ph1m::{
    MemoryArchiveExcerpt, MemoryBundleItem, MemoryCandidate, MemoryCommitDecision,
    MemoryCommitStatus, MemoryConfidence, MemoryConsent, MemoryEmotionalThreadState,
    MemoryExposureLevel, MemoryGraphEdgeInput, MemoryGraphNodeInput, MemoryHintEntry,
    MemoryItemTag, MemoryKey, MemoryLayer, MemoryLedgerEvent, MemoryLedgerEventKind,
    MemoryMetricPayload, MemoryProposedItem, MemoryProvenance, MemoryProvenanceTier,
    MemoryResumeAction, MemoryResumeDeliveryMode, MemoryResumeTier, MemoryRetentionMode,
    MemorySafeSummaryItem, MemorySensitivityFlag, MemorySuppressionRule, MemorySuppressionRuleKind,
    MemorySuppressionTargetType, MemoryUsePolicy, PendingWorkItem, Ph1mContextBundleBuildRequest,
    Ph1mContextBundleBuildResponse, Ph1mEmotionalThreadUpdateRequest,
    Ph1mEmotionalThreadUpdateResponse, Ph1mForgetRequest, Ph1mForgetResponse,
    Ph1mGraphUpdateRequest, Ph1mGraphUpdateResponse, Ph1mHintBundleBuildRequest,
    Ph1mHintBundleBuildResponse, Ph1mMetricsEmitRequest, Ph1mMetricsEmitResponse,
    Ph1mProposeRequest, Ph1mProposeResponse, Ph1mRecallRequest, Ph1mRecallResponse,
    Ph1mResumeSelectRequest, Ph1mResumeSelectResponse, Ph1mRetentionModeSetRequest,
    Ph1mRetentionModeSetResponse, Ph1mSafeSummaryRequest, Ph1mSafeSummaryResponse,
    Ph1mSuppressionSetRequest, Ph1mSuppressionSetResponse, Ph1mThreadDigestUpsertRequest,
    Ph1mThreadDigestUpsertResponse, MEMORY_CONTEXT_BUNDLE_MAX_BYTES, MEMORY_RESUME_HOT_WINDOW_MS,
    MEMORY_RESUME_WARM_WINDOW_MS, MEMORY_UNRESOLVED_DECAY_WINDOW_MS,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.M reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const M_STORED: ReasonCodeId = ReasonCodeId(0x4D00_0001);
    pub const M_UPDATED: ReasonCodeId = ReasonCodeId(0x4D00_0002);
    pub const M_NEEDS_CONSENT: ReasonCodeId = ReasonCodeId(0x4D00_0003);
    pub const M_REJECT_UNKNOWN_SPEAKER: ReasonCodeId = ReasonCodeId(0x4D00_0004);
    pub const M_REJECT_SENSITIVE_NO_CONSENT: ReasonCodeId = ReasonCodeId(0x4D00_0005);
    pub const M_FORGOTTEN: ReasonCodeId = ReasonCodeId(0x4D00_0006);
    pub const M_NOT_FOUND: ReasonCodeId = ReasonCodeId(0x4D00_0007);
    pub const M_POLICY_BLOCKED: ReasonCodeId = ReasonCodeId(0x4D00_0008);
    pub const M_THREAD_DIGEST_UPSERTED: ReasonCodeId = ReasonCodeId(0x4D00_0009);
    pub const M_RESUME_AUTO_LOAD: ReasonCodeId = ReasonCodeId(0x4D00_000A);
    pub const M_RESUME_SUGGEST: ReasonCodeId = ReasonCodeId(0x4D00_000B);
    pub const M_RESUME_NONE: ReasonCodeId = ReasonCodeId(0x4D00_000C);
    pub const M_HINT_BUNDLE_READY: ReasonCodeId = ReasonCodeId(0x4D00_000D);
    pub const M_CONTEXT_BUNDLE_READY: ReasonCodeId = ReasonCodeId(0x4D00_000E);
    pub const M_CONTEXT_BUNDLE_EMPTY: ReasonCodeId = ReasonCodeId(0x4D00_000F);
    pub const M_SUPPRESSION_APPLIED: ReasonCodeId = ReasonCodeId(0x4D00_0010);
    pub const M_SAFE_SUMMARY_READY: ReasonCodeId = ReasonCodeId(0x4D00_0011);
    pub const M_EMO_THREAD_UPDATED: ReasonCodeId = ReasonCodeId(0x4D00_0012);
    pub const M_METRICS_EMITTED: ReasonCodeId = ReasonCodeId(0x4D00_0013);
    pub const M_GRAPH_UPDATED: ReasonCodeId = ReasonCodeId(0x4D00_0014);
    pub const M_RETENTION_MODE_UPDATED: ReasonCodeId = ReasonCodeId(0x4D00_0015);
    pub const M_CLARIFY_REQUIRED: ReasonCodeId = ReasonCodeId(0x4D00_0016);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1mConfig {
    pub micro_ttl_ms: u64,
    pub micro_promote_after_seen: u32,
    pub resume_hot_window_ms: u64,
    pub resume_warm_window_ms: u64,
    pub resume_warm_window_remember_everything_ms: u64,
    pub unresolved_decay_window_ms: u64,
}

impl Ph1mConfig {
    pub fn mvp_v1() -> Self {
        Self {
            // Default 30 days; spec suggests 30-90 days. Stored in milliseconds for easy policy tuning.
            micro_ttl_ms: 30_u64 * 24 * 60 * 60 * 1000,
            micro_promote_after_seen: 2,
            resume_hot_window_ms: MEMORY_RESUME_HOT_WINDOW_MS,
            resume_warm_window_ms: MEMORY_RESUME_WARM_WINDOW_MS,
            resume_warm_window_remember_everything_ms: 60_u64 * 24 * 60 * 60 * 1000,
            unresolved_decay_window_ms: MEMORY_UNRESOLVED_DECAY_WINDOW_MS,
        }
    }
}

#[derive(Debug, Clone)]
struct MemoryEntry {
    key: MemoryKey,
    value: selene_kernel_contracts::ph1m::MemoryValue,
    layer: MemoryLayer,
    sensitivity: MemorySensitivityFlag,
    use_policy: MemoryUsePolicy,
    confidence: MemoryConfidence,
    consent: MemoryConsent,
    last_seen_at: MonotonicTimeNs,
    expires_at: Option<MonotonicTimeNs>,
    evidence_quote: String,
    provenance: MemoryProvenance,
    seen_count: u32,
}

#[derive(Debug, Clone)]
struct ThreadEntry {
    thread_id: String,
    thread_title: String,
    summary_bullets: Vec<String>,
    pinned: bool,
    unresolved: bool,
    last_updated_at: MonotonicTimeNs,
    use_count: u32,
}

#[derive(Debug, Clone)]
struct SuppressionEntry {
    rule: MemorySuppressionRule,
}

#[derive(Debug, Clone)]
struct EmotionalThreadEntry {
    _state: MemoryEmotionalThreadState,
}

#[derive(Debug, Clone)]
struct GraphNodeEntry {
    _node: MemoryGraphNodeInput,
}

#[derive(Debug, Clone)]
struct GraphEdgeEntry {
    _edge: MemoryGraphEdgeInput,
}

#[derive(Debug, Clone)]
pub struct Ph1mRuntime {
    config: Ph1mConfig,
    current: BTreeMap<MemoryKey, MemoryEntry>,
    threads: BTreeMap<String, ThreadEntry>,
    suppression_rules: BTreeMap<
        (
            MemorySuppressionTargetType,
            String,
            MemorySuppressionRuleKind,
        ),
        SuppressionEntry,
    >,
    emotional_threads: BTreeMap<String, EmotionalThreadEntry>,
    graph_nodes: BTreeMap<String, GraphNodeEntry>,
    graph_edges: BTreeMap<String, GraphEdgeEntry>,
    metrics_ledger: Vec<MemoryMetricPayload>,
    retention_mode: MemoryRetentionMode,
}

impl Ph1mRuntime {
    pub fn new(config: Ph1mConfig) -> Self {
        Self {
            config,
            current: BTreeMap::new(),
            threads: BTreeMap::new(),
            suppression_rules: BTreeMap::new(),
            emotional_threads: BTreeMap::new(),
            graph_nodes: BTreeMap::new(),
            graph_edges: BTreeMap::new(),
            metrics_ledger: vec![],
            retention_mode: MemoryRetentionMode::Default,
        }
    }

    pub fn propose(
        &mut self,
        req: &Ph1mProposeRequest,
    ) -> Result<Ph1mProposeResponse, ContractViolation> {
        req.validate()?;

        let speaker_ok = match &req.speaker_assertion {
            Ph1VoiceIdResponse::SpeakerAssertionOk(ok) => ok,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(_) => {
                let mut decisions = Vec::with_capacity(req.proposals.len());
                for p in &req.proposals {
                    decisions.push(MemoryCommitDecision::v1(
                        p.memory_key.clone(),
                        MemoryCommitStatus::Rejected,
                        reason_codes::M_REJECT_UNKNOWN_SPEAKER,
                        None,
                    )?);
                }
                return Ph1mProposeResponse::v1(decisions, vec![]);
            }
        };

        // Minimal policy: privacy mode blocks any new storage in the skeleton.
        if req.policy_context_ref.privacy_mode {
            let mut decisions = Vec::with_capacity(req.proposals.len());
            for p in &req.proposals {
                decisions.push(MemoryCommitDecision::v1(
                    p.memory_key.clone(),
                    MemoryCommitStatus::Rejected,
                    reason_codes::M_POLICY_BLOCKED,
                    None,
                )?);
            }
            return Ph1mProposeResponse::v1(decisions, vec![]);
        }

        let mut decisions: Vec<MemoryCommitDecision> = Vec::with_capacity(req.proposals.len());
        let mut events: Vec<MemoryLedgerEvent> = Vec::new();

        for p in &req.proposals {
            p.validate()?;

            if self.is_suppressed(
                MemorySuppressionTargetType::TopicKey,
                p.memory_key.as_str(),
                MemorySuppressionRuleKind::DoNotStore,
            ) {
                decisions.push(MemoryCommitDecision::v1(
                    p.memory_key.clone(),
                    MemoryCommitStatus::Rejected,
                    reason_codes::M_POLICY_BLOCKED,
                    None,
                )?);
                continue;
            }

            if p.consent == MemoryConsent::Denied {
                decisions.push(MemoryCommitDecision::v1(
                    p.memory_key.clone(),
                    MemoryCommitStatus::Rejected,
                    reason_codes::M_POLICY_BLOCKED,
                    None,
                )?);
                continue;
            }

            if p.sensitivity_flag == MemorySensitivityFlag::Sensitive
                && p.consent != MemoryConsent::Confirmed
            {
                decisions.push(MemoryCommitDecision::v1(
                    p.memory_key.clone(),
                    MemoryCommitStatus::NeedsConsent,
                    reason_codes::M_NEEDS_CONSENT,
                    Some("Do you want me to remember that for next time?".to_string()),
                )?);
                continue;
            }

            let now = req.now;
            let existed = self.current.get(&p.memory_key).cloned();

            let (mut layer, mut use_policy, mut expires_at, seen_count) =
                initial_policy_for(p, existed.as_ref(), now, &self.config);

            // Promoting micro-memory when repeated or explicitly remembered.
            if layer == MemoryLayer::Micro
                && (seen_count >= self.config.micro_promote_after_seen
                    || matches!(
                        p.consent,
                        MemoryConsent::Confirmed | MemoryConsent::ExplicitRemember
                    ))
            {
                layer = MemoryLayer::LongTerm;
                use_policy = MemoryUsePolicy::AlwaysUsable;
                expires_at = None;
            }

            let kind = if existed.is_some() {
                MemoryLedgerEventKind::Updated
            } else {
                MemoryLedgerEventKind::Stored
            };
            let rc = if existed.is_some() {
                reason_codes::M_UPDATED
            } else {
                reason_codes::M_STORED
            };

            let entry = MemoryEntry {
                key: p.memory_key.clone(),
                value: p.memory_value.clone(),
                layer,
                sensitivity: p.sensitivity_flag,
                use_policy,
                confidence: p.confidence,
                consent: p.consent,
                last_seen_at: now,
                expires_at,
                evidence_quote: p.evidence_quote.clone(),
                provenance: p.provenance.clone(),
                seen_count,
            };
            self.current.insert(p.memory_key.clone(), entry);

            let ev = MemoryLedgerEvent::v1(
                kind,
                now,
                p.memory_key.clone(),
                Some(p.memory_value.clone()),
                Some(p.evidence_quote.clone()),
                p.provenance.clone(),
                layer,
                p.sensitivity_flag,
                p.confidence,
                p.consent,
                rc,
            )?;
            events.push(ev);

            decisions.push(MemoryCommitDecision::v1(
                p.memory_key.clone(),
                if existed.is_some() {
                    MemoryCommitStatus::Updated
                } else {
                    MemoryCommitStatus::Stored
                },
                rc,
                None,
            )?);

            let _ = speaker_ok; // speaker identity is enforced by upstream; persistence will attribute later.
        }

        Ph1mProposeResponse::v1(decisions, events)
    }

    pub fn recall(
        &mut self,
        req: &Ph1mRecallRequest,
    ) -> Result<Ph1mRecallResponse, ContractViolation> {
        req.validate()?;

        match &req.speaker_assertion {
            Ph1VoiceIdResponse::SpeakerAssertionOk(_) => {}
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(_) => {
                return Ph1mRecallResponse::v1(
                    vec![],
                    Some(reason_codes::M_REJECT_UNKNOWN_SPEAKER),
                );
            }
        }

        // Purge expired micro-memory deterministically on recall ticks.
        let now = req.now;
        let expired: Vec<MemoryKey> = self
            .current
            .iter()
            .filter_map(|(k, v)| match v.expires_at {
                Some(t) if t.0 <= now.0 => Some(k.clone()),
                _ => None,
            })
            .collect();
        for k in expired {
            self.current.remove(&k);
        }

        let mut out: Vec<MemoryCandidate> = Vec::new();

        for k in &req.requested_keys {
            if out.len() >= req.max_candidates as usize {
                break;
            }
            if self.is_suppressed(
                MemorySuppressionTargetType::TopicKey,
                k.as_str(),
                MemorySuppressionRuleKind::DoNotMention,
            ) {
                continue;
            }
            let Some(e) = self.current.get(k) else {
                continue;
            };

            if e.sensitivity == MemorySensitivityFlag::Sensitive && !req.allow_sensitive {
                continue;
            }

            out.push(MemoryCandidate::v1(
                e.key.clone(),
                e.value.clone(),
                e.confidence,
                e.last_seen_at,
                e.evidence_quote.clone(),
                e.provenance.clone(),
                e.sensitivity,
                e.use_policy,
                e.expires_at,
            )?);
        }

        Ph1mRecallResponse::v1(out, None)
    }

    pub fn forget(
        &mut self,
        req: &Ph1mForgetRequest,
    ) -> Result<Ph1mForgetResponse, ContractViolation> {
        req.validate()?;

        match &req.speaker_assertion {
            Ph1VoiceIdResponse::SpeakerAssertionOk(_) => {}
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(_) => {
                return Ph1mForgetResponse::v1(
                    false,
                    None,
                    Some(reason_codes::M_REJECT_UNKNOWN_SPEAKER),
                );
            }
        }

        let Some(entry) = self.current.remove(&req.target_key) else {
            return Ph1mForgetResponse::v1(false, None, Some(reason_codes::M_NOT_FOUND));
        };

        let ev = MemoryLedgerEvent::v1(
            MemoryLedgerEventKind::Forgotten,
            req.now,
            entry.key.clone(),
            None,
            None,
            entry.provenance.clone(),
            entry.layer,
            entry.sensitivity,
            entry.confidence,
            entry.consent,
            reason_codes::M_FORGOTTEN,
        )?;

        Ph1mForgetResponse::v1(true, Some(ev), None)
    }

    pub fn thread_digest_upsert(
        &mut self,
        req: &Ph1mThreadDigestUpsertRequest,
    ) -> Result<Ph1mThreadDigestUpsertResponse, ContractViolation> {
        req.validate()?;

        match &req.speaker_assertion {
            Ph1VoiceIdResponse::SpeakerAssertionOk(_) => {}
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(_) => {
                return Ph1mThreadDigestUpsertResponse::v1(
                    false,
                    req.thread_digest.thread_id.clone(),
                    reason_codes::M_REJECT_UNKNOWN_SPEAKER,
                );
            }
        }

        if req.policy_context_ref.privacy_mode {
            return Ph1mThreadDigestUpsertResponse::v1(
                false,
                req.thread_digest.thread_id.clone(),
                reason_codes::M_POLICY_BLOCKED,
            );
        }

        let existed = self
            .threads
            .contains_key(req.thread_digest.thread_id.as_str());
        let entry = ThreadEntry {
            thread_id: req.thread_digest.thread_id.clone(),
            thread_title: req.thread_digest.thread_title.clone(),
            summary_bullets: req.thread_digest.summary_bullets.clone(),
            pinned: req.thread_digest.pinned,
            unresolved: req.thread_digest.unresolved,
            last_updated_at: req.thread_digest.last_updated_at,
            use_count: req.thread_digest.use_count,
        };
        self.threads.insert(entry.thread_id.clone(), entry);

        Ph1mThreadDigestUpsertResponse::v1(
            !existed,
            req.thread_digest.thread_id.clone(),
            reason_codes::M_THREAD_DIGEST_UPSERTED,
        )
    }

    pub fn resume_select(
        &self,
        req: &Ph1mResumeSelectRequest,
    ) -> Result<Ph1mResumeSelectResponse, ContractViolation> {
        req.validate()?;

        match &req.speaker_assertion {
            Ph1VoiceIdResponse::SpeakerAssertionOk(_) => {}
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(_) => {
                return Ph1mResumeSelectResponse::v1(
                    None,
                    None,
                    None,
                    MemoryResumeAction::None,
                    vec![],
                    reason_codes::M_REJECT_UNKNOWN_SPEAKER,
                );
            }
        }

        let lower_topic_hint = req.topic_hint.as_ref().map(|v| v.to_ascii_lowercase());
        let effective_mode = self.retention_mode;

        let mut pending_candidates: Vec<(&PendingWorkItem, MemoryResumeTier)> =
            if req.allow_pending_work_resume {
                req.pending_work_orders
                    .iter()
                    .filter(|work| pending_status_is_resume_eligible(work.status))
                    .filter(|work| {
                        !req.suppressed_work_order_ids
                            .iter()
                            .any(|v| v == &work.work_order_id)
                            && !self.is_suppressed(
                                MemorySuppressionTargetType::WorkOrderId,
                                &work.work_order_id,
                                MemorySuppressionRuleKind::DoNotMention,
                            )
                    })
                    .filter(|work| {
                        if let Some(thread_id) = &work.thread_id {
                            !req.suppressed_thread_ids.iter().any(|v| v == thread_id)
                                && !self.is_suppressed(
                                    MemorySuppressionTargetType::ThreadId,
                                    thread_id,
                                    MemorySuppressionRuleKind::DoNotMention,
                                )
                        } else {
                            true
                        }
                    })
                    .filter_map(|work| {
                        let age_ns = req.now.0.saturating_sub(work.last_updated_at.0);
                        let tier = resume_tier_for(age_ns, false, effective_mode, &self.config);
                        if tier == MemoryResumeTier::Cold && req.topic_hint.is_none() {
                            return None;
                        }
                        Some((work, tier))
                    })
                    .collect()
            } else {
                vec![]
            };
        pending_candidates.sort_by(|a, b| {
            let (a_work, a_tier) = a;
            let (b_work, b_tier) = b;
            tier_rank(*b_tier)
                .cmp(&tier_rank(*a_tier))
                .then_with(|| b_work.last_updated_at.0.cmp(&a_work.last_updated_at.0))
                .then_with(|| b_work.use_count.cmp(&a_work.use_count))
                .then_with(|| a_work.work_order_id.cmp(&b_work.work_order_id))
        });

        if let Some((pending, tier)) = pending_candidates.into_iter().next() {
            let (action, delivery_mode, reason_code) =
                select_resume_action_and_delivery(tier, req, req.topic_hint.is_some());
            let summary = if matches!(
                action,
                MemoryResumeAction::AutoLoad | MemoryResumeAction::Suggest
            ) {
                pending
                    .summary_bullets
                    .iter()
                    .take(req.max_summary_bullets as usize)
                    .cloned()
                    .collect::<Vec<_>>()
            } else {
                vec![]
            };
            return Ph1mResumeSelectResponse::v1_with_delivery(
                pending.thread_id.clone(),
                pending
                    .thread_id
                    .clone()
                    .map(|_| format!("Pending WorkOrder {}", pending.work_order_id)),
                Some(pending.work_order_id.clone()),
                Some(tier),
                action,
                delivery_mode,
                summary,
                reason_code,
            );
        }

        let mut candidates: Vec<(&ThreadEntry, MemoryResumeTier, bool)> = self
            .threads
            .values()
            .filter(|entry| {
                !req.suppressed_thread_ids
                    .iter()
                    .any(|v| v == &entry.thread_id)
                    && !self.is_suppressed(
                        MemorySuppressionTargetType::ThreadId,
                        &entry.thread_id,
                        MemorySuppressionRuleKind::DoNotMention,
                    )
            })
            .filter_map(|entry| {
                if let Some(topic_hint) = &lower_topic_hint {
                    let title = entry.thread_title.to_ascii_lowercase();
                    let thread_id = entry.thread_id.to_ascii_lowercase();
                    if !title.contains(topic_hint) && !thread_id.contains(topic_hint) {
                        return None;
                    }
                }
                let age_ns = req.now.0.saturating_sub(entry.last_updated_at.0);
                let tier = resume_tier_for(age_ns, entry.unresolved, effective_mode, &self.config);
                let unresolved_boost =
                    entry.unresolved && age_ns <= ms_to_ns(self.config.unresolved_decay_window_ms);
                Some((entry, tier, unresolved_boost))
            })
            .collect();

        candidates.sort_by(|a, b| {
            let (a_entry, _, a_unresolved_boost) = a;
            let (b_entry, _, b_unresolved_boost) = b;
            b_entry
                .pinned
                .cmp(&a_entry.pinned)
                .then_with(|| b_unresolved_boost.cmp(a_unresolved_boost))
                .then_with(|| b_entry.last_updated_at.0.cmp(&a_entry.last_updated_at.0))
                .then_with(|| b_entry.use_count.cmp(&a_entry.use_count))
                .then_with(|| a_entry.thread_id.cmp(&b_entry.thread_id))
        });

        let Some((selected, tier, _)) = candidates.into_iter().next() else {
            return Ph1mResumeSelectResponse::v1_with_delivery(
                None,
                None,
                None,
                None,
                MemoryResumeAction::None,
                MemoryResumeDeliveryMode::None,
                vec![],
                reason_codes::M_RESUME_NONE,
            );
        };

        let (action, delivery_mode, reason_code) =
            select_resume_action_and_delivery(tier, req, req.topic_hint.is_some());

        let summary = if matches!(
            action,
            MemoryResumeAction::AutoLoad | MemoryResumeAction::Suggest
        ) {
            selected
                .summary_bullets
                .iter()
                .take(req.max_summary_bullets as usize)
                .cloned()
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        Ph1mResumeSelectResponse::v1_with_delivery(
            Some(selected.thread_id.clone()),
            Some(selected.thread_title.clone()),
            None,
            Some(tier),
            action,
            delivery_mode,
            summary,
            reason_code,
        )
    }

    pub fn hint_bundle_build(
        &self,
        req: &Ph1mHintBundleBuildRequest,
    ) -> Result<Ph1mHintBundleBuildResponse, ContractViolation> {
        req.validate()?;
        if !self.identity_ok(&req.speaker_assertion) {
            return Ph1mHintBundleBuildResponse::v1(vec![], reason_codes::M_REJECT_UNKNOWN_SPEAKER);
        }
        if req.policy_context_ref.privacy_mode {
            return Ph1mHintBundleBuildResponse::v1(vec![], reason_codes::M_POLICY_BLOCKED);
        }

        let mut hints = vec![];
        let mut push_keys = vec![
            "preferred_name".to_string(),
            "profile_language".to_string(),
            "contact_preference".to_string(),
        ];
        for (target_type, target_id, rule_kind) in self.suppression_rules.keys() {
            if *target_type == MemorySuppressionTargetType::TopicKey
                && *rule_kind == MemorySuppressionRuleKind::DoNotRepeat
            {
                push_keys.push(format!("do_not_repeat:{target_id}"));
            }
        }
        push_keys.sort();
        push_keys.dedup();

        for key in push_keys {
            if hints.len() >= req.max_hints as usize {
                break;
            }
            if let Some(stripped_key) = key.strip_prefix("do_not_repeat:") {
                hints.push(MemoryHintEntry::v1(
                    "do_not_repeat".to_string(),
                    stripped_key.to_string(),
                )?);
                continue;
            }
            let Some(entry) = self.current.get(&MemoryKey::new(key.clone())?) else {
                continue;
            };
            if entry.sensitivity == MemorySensitivityFlag::Sensitive {
                continue;
            }
            if self.is_suppressed(
                MemorySuppressionTargetType::TopicKey,
                key.as_str(),
                MemorySuppressionRuleKind::DoNotMention,
            ) {
                continue;
            }
            hints.push(MemoryHintEntry::v1(key, entry.value.verbatim.clone())?);
        }

        Ph1mHintBundleBuildResponse::v1(hints, reason_codes::M_HINT_BUNDLE_READY)
    }

    pub fn context_bundle_build(
        &self,
        req: &Ph1mContextBundleBuildRequest,
    ) -> Result<Ph1mContextBundleBuildResponse, ContractViolation> {
        req.validate()?;
        if !self.identity_ok(&req.speaker_assertion) {
            return Ph1mContextBundleBuildResponse::v1(
                vec![],
                vec![],
                vec![],
                MemoryMetricPayload::v1(0, 0, 0, 0, 0, 0, 0, 0, 0, 0)?,
                reason_codes::M_REJECT_UNKNOWN_SPEAKER,
            );
        }
        if req.policy_context_ref.privacy_mode {
            return Ph1mContextBundleBuildResponse::v1(
                vec![],
                vec![],
                vec![],
                MemoryMetricPayload::v1(0, 0, 0, 0, 0, 0, 0, 0, 0, 0)?,
                reason_codes::M_POLICY_BLOCKED,
            );
        }

        let mut candidates: Vec<MemoryBundleItem> = vec![];
        let lower_topic_hint = req.topic_hint.as_ref().map(|v| v.to_ascii_lowercase());
        let mut do_not_mention_hits_count: u16 = 0;
        for (key, entry) in &self.current {
            if candidates.len() >= req.max_atoms as usize {
                break;
            }
            if !req.requested_keys.is_empty() && !req.requested_keys.iter().any(|v| v == key) {
                let include = if let Some(topic_hint) = &lower_topic_hint {
                    key.as_str().to_ascii_lowercase().contains(topic_hint)
                        || entry
                            .value
                            .verbatim
                            .to_ascii_lowercase()
                            .contains(topic_hint)
                } else {
                    false
                };
                if !include {
                    continue;
                }
            }
            if self.is_suppressed(
                MemorySuppressionTargetType::TopicKey,
                key.as_str(),
                MemorySuppressionRuleKind::DoNotMention,
            ) {
                do_not_mention_hits_count = do_not_mention_hits_count.saturating_add(1);
                continue;
            }
            if let Some(thread_id) = &req.thread_id {
                if self.is_suppressed(
                    MemorySuppressionTargetType::ThreadId,
                    thread_id,
                    MemorySuppressionRuleKind::DoNotMention,
                ) {
                    do_not_mention_hits_count = do_not_mention_hits_count.saturating_add(1);
                    continue;
                }
            }
            if let Some(work_order_id) = &req.work_order_id {
                if self.is_suppressed(
                    MemorySuppressionTargetType::WorkOrderId,
                    work_order_id,
                    MemorySuppressionRuleKind::DoNotMention,
                ) {
                    do_not_mention_hits_count = do_not_mention_hits_count.saturating_add(1);
                    continue;
                }
            }
            if entry.sensitivity == MemorySensitivityFlag::Sensitive && !req.allow_sensitive {
                continue;
            }
            let stale = match entry.expires_at {
                Some(expires_at) => expires_at.0 <= req.now.0,
                None => false,
            };
            let conflict = req.current_state_facts.iter().any(|fact| {
                fact.memory_key == *key && fact.memory_value.verbatim != entry.value.verbatim
            });
            let tag = if conflict {
                MemoryItemTag::Conflict
            } else if stale {
                MemoryItemTag::Stale
            } else if entry.confidence == MemoryConfidence::High {
                MemoryItemTag::Confirmed
            } else {
                MemoryItemTag::Tentative
            };
            candidates.push(MemoryBundleItem::v1(
                key.as_str().to_string(),
                key.clone(),
                entry.value.clone(),
                tag,
                exposure_for(entry.sensitivity),
                entry.confidence,
                MemoryProvenanceTier::UserStated,
                false,
                entry.last_seen_at,
                entry.seen_count,
            )?);
        }

        candidates.sort_by(|a, b| {
            b.pinned
                .cmp(&a.pinned)
                .then_with(|| b.provenance_tier.cmp(&a.provenance_tier))
                .then_with(|| confidence_rank(b.confidence).cmp(&confidence_rank(a.confidence)))
                .then_with(|| b.last_used_at.0.cmp(&a.last_used_at.0))
                .then_with(|| b.use_count.cmp(&a.use_count))
                .then_with(|| a.item_id.cmp(&b.item_id))
        });

        let mut push_items: Vec<MemoryBundleItem> = vec![];
        let mut pull_items: Vec<MemoryBundleItem> = vec![];
        for item in candidates {
            if item.exposure_level == MemoryExposureLevel::InternalOnly {
                continue;
            }
            if is_push_memory_key(item.memory_key.as_str()) {
                push_items.push(item);
            } else {
                pull_items.push(item);
            }
        }
        if push_items.len() > req.max_atoms as usize {
            push_items.truncate(req.max_atoms as usize);
        }
        let remaining = req.max_atoms.saturating_sub(push_items.len() as u8) as usize;
        if pull_items.len() > remaining {
            pull_items.truncate(remaining);
        }

        let mut archive_excerpts: Vec<MemoryArchiveExcerpt> = vec![];
        if req.max_excerpts > 0 {
            for thread in self.threads.values() {
                if archive_excerpts.len() >= req.max_excerpts as usize {
                    break;
                }
                if let Some(topic_hint) = &lower_topic_hint {
                    if !thread
                        .thread_title
                        .to_ascii_lowercase()
                        .contains(topic_hint)
                        && !thread.thread_id.to_ascii_lowercase().contains(topic_hint)
                    {
                        continue;
                    }
                }
                if self.is_suppressed(
                    MemorySuppressionTargetType::ThreadId,
                    &thread.thread_id,
                    MemorySuppressionRuleKind::DoNotMention,
                ) {
                    continue;
                }
                if let Some(first) = thread.summary_bullets.first() {
                    archive_excerpts.push(MemoryArchiveExcerpt::v1(
                        format!("thread:{}", thread.thread_id),
                        first.clone(),
                    )?);
                }
            }
        }

        let mut context_bundle_bytes: u32 = 0;
        let mut confirmed_count: u8 = 0;
        let mut tentative_count: u8 = 0;
        let mut stale_count: u8 = 0;
        let mut conflict_count: u8 = 0;
        let mut clarification_due_to_memory_count: u16 = 0;

        for item in push_items.iter().chain(pull_items.iter()) {
            context_bundle_bytes = context_bundle_bytes
                .saturating_add(item.memory_key.as_str().len() as u32)
                .saturating_add(item.memory_value.verbatim.len() as u32);
            match item.tag {
                MemoryItemTag::Confirmed => confirmed_count = confirmed_count.saturating_add(1),
                MemoryItemTag::Tentative => {
                    tentative_count = tentative_count.saturating_add(1);
                    clarification_due_to_memory_count =
                        clarification_due_to_memory_count.saturating_add(1);
                }
                MemoryItemTag::Stale => {
                    stale_count = stale_count.saturating_add(1);
                    clarification_due_to_memory_count =
                        clarification_due_to_memory_count.saturating_add(1);
                }
                MemoryItemTag::Conflict => {
                    conflict_count = conflict_count.saturating_add(1);
                    clarification_due_to_memory_count =
                        clarification_due_to_memory_count.saturating_add(1);
                }
            }
        }
        for excerpt in &archive_excerpts {
            context_bundle_bytes = context_bundle_bytes
                .saturating_add(excerpt.archive_ref_id.len() as u32)
                .saturating_add(excerpt.excerpt_text.len() as u32);
        }

        if context_bundle_bytes > req.max_bundle_bytes {
            while context_bundle_bytes > req.max_bundle_bytes && !pull_items.is_empty() {
                let popped = pull_items.pop().expect("pop is non-empty");
                context_bundle_bytes = context_bundle_bytes
                    .saturating_sub(popped.memory_key.as_str().len() as u32)
                    .saturating_sub(popped.memory_value.verbatim.len() as u32);
            }
            while context_bundle_bytes > req.max_bundle_bytes && !archive_excerpts.is_empty() {
                let popped = archive_excerpts.pop().expect("pop is non-empty");
                context_bundle_bytes = context_bundle_bytes
                    .saturating_sub(popped.archive_ref_id.len() as u32)
                    .saturating_sub(popped.excerpt_text.len() as u32);
            }
        }

        if context_bundle_bytes > MEMORY_CONTEXT_BUNDLE_MAX_BYTES {
            context_bundle_bytes = MEMORY_CONTEXT_BUNDLE_MAX_BYTES;
        }

        let metric_payload = MemoryMetricPayload::v1(
            context_bundle_bytes,
            (push_items.len() + pull_items.len()) as u8,
            archive_excerpts.len() as u8,
            confirmed_count,
            tentative_count,
            stale_count,
            conflict_count,
            conflict_count as u16,
            clarification_due_to_memory_count,
            do_not_mention_hits_count,
        )?;
        let reason_code = if push_items.is_empty() && pull_items.is_empty() {
            reason_codes::M_CONTEXT_BUNDLE_EMPTY
        } else {
            reason_codes::M_CONTEXT_BUNDLE_READY
        };
        Ph1mContextBundleBuildResponse::v1(
            push_items,
            pull_items,
            archive_excerpts,
            metric_payload,
            reason_code,
        )
    }

    pub fn suppression_set(
        &mut self,
        req: &Ph1mSuppressionSetRequest,
    ) -> Result<Ph1mSuppressionSetResponse, ContractViolation> {
        req.validate()?;
        if !self.identity_ok(&req.speaker_assertion) {
            return Ph1mSuppressionSetResponse::v1(
                false,
                req.rule.clone(),
                reason_codes::M_REJECT_UNKNOWN_SPEAKER,
            );
        }
        if req.policy_context_ref.privacy_mode {
            return Ph1mSuppressionSetResponse::v1(
                false,
                req.rule.clone(),
                reason_codes::M_POLICY_BLOCKED,
            );
        }

        self.suppression_rules.insert(
            (
                req.rule.target_type,
                req.rule.target_id.clone(),
                req.rule.rule_kind,
            ),
            SuppressionEntry {
                rule: req.rule.clone(),
            },
        );
        Ph1mSuppressionSetResponse::v1(true, req.rule.clone(), reason_codes::M_SUPPRESSION_APPLIED)
    }

    pub fn safe_summary(
        &self,
        req: &Ph1mSafeSummaryRequest,
    ) -> Result<Ph1mSafeSummaryResponse, ContractViolation> {
        req.validate()?;
        if !self.identity_ok(&req.speaker_assertion) {
            return Ph1mSafeSummaryResponse::v1(vec![], 0, reason_codes::M_REJECT_UNKNOWN_SPEAKER);
        }

        let mut summary_items = vec![];
        let mut output_bytes: u16 = 0;
        for (key, entry) in &self.current {
            if summary_items.len() >= req.max_items as usize {
                break;
            }
            let exposure = exposure_for(entry.sensitivity);
            if exposure == MemoryExposureLevel::InternalOnly {
                continue;
            }
            if self.is_suppressed(
                MemorySuppressionTargetType::TopicKey,
                key.as_str(),
                MemorySuppressionRuleKind::DoNotMention,
            ) {
                continue;
            }
            let snippet = format!("{}: {}", key.as_str(), entry.value.verbatim);
            let candidate_bytes = output_bytes.saturating_add(snippet.len() as u16);
            if candidate_bytes > req.max_bytes {
                break;
            }
            output_bytes = candidate_bytes;
            summary_items.push(MemorySafeSummaryItem::v1(key.clone(), snippet, exposure)?);
        }
        Ph1mSafeSummaryResponse::v1(
            summary_items,
            output_bytes,
            reason_codes::M_SAFE_SUMMARY_READY,
        )
    }

    pub fn emotional_thread_update(
        &mut self,
        req: &Ph1mEmotionalThreadUpdateRequest,
    ) -> Result<Ph1mEmotionalThreadUpdateResponse, ContractViolation> {
        req.validate()?;
        if !self.identity_ok(&req.speaker_assertion) {
            return Ph1mEmotionalThreadUpdateResponse::v1(
                req.thread_state.clone(),
                reason_codes::M_REJECT_UNKNOWN_SPEAKER,
            );
        }
        if req.policy_context_ref.privacy_mode {
            return Ph1mEmotionalThreadUpdateResponse::v1(
                req.thread_state.clone(),
                reason_codes::M_POLICY_BLOCKED,
            );
        }
        self.emotional_threads.insert(
            req.thread_state.thread_key.clone(),
            EmotionalThreadEntry {
                _state: req.thread_state.clone(),
            },
        );
        Ph1mEmotionalThreadUpdateResponse::v1(
            req.thread_state.clone(),
            reason_codes::M_EMO_THREAD_UPDATED,
        )
    }

    pub fn metrics_emit(
        &mut self,
        req: &Ph1mMetricsEmitRequest,
    ) -> Result<Ph1mMetricsEmitResponse, ContractViolation> {
        req.validate()?;
        if !self.identity_ok(&req.speaker_assertion) {
            return Ph1mMetricsEmitResponse::v1(false, reason_codes::M_REJECT_UNKNOWN_SPEAKER);
        }
        if req.policy_context_ref.privacy_mode {
            return Ph1mMetricsEmitResponse::v1(false, reason_codes::M_POLICY_BLOCKED);
        }
        self.metrics_ledger.push(req.payload.clone());
        Ph1mMetricsEmitResponse::v1(true, reason_codes::M_METRICS_EMITTED)
    }

    pub fn graph_update(
        &mut self,
        req: &Ph1mGraphUpdateRequest,
    ) -> Result<Ph1mGraphUpdateResponse, ContractViolation> {
        req.validate()?;
        if !self.identity_ok(&req.speaker_assertion) {
            return Ph1mGraphUpdateResponse::v1(0, reason_codes::M_REJECT_UNKNOWN_SPEAKER);
        }
        if req.policy_context_ref.privacy_mode {
            return Ph1mGraphUpdateResponse::v1(0, reason_codes::M_POLICY_BLOCKED);
        }
        let mut count: u16 = 0;
        for node in &req.nodes {
            self.graph_nodes.insert(
                node.node_id.clone(),
                GraphNodeEntry {
                    _node: node.clone(),
                },
            );
            count = count.saturating_add(1);
        }
        for edge in &req.edges {
            self.graph_edges.insert(
                edge.edge_id.clone(),
                GraphEdgeEntry {
                    _edge: edge.clone(),
                },
            );
            count = count.saturating_add(1);
        }
        Ph1mGraphUpdateResponse::v1(count, reason_codes::M_GRAPH_UPDATED)
    }

    pub fn retention_mode_set(
        &mut self,
        req: &Ph1mRetentionModeSetRequest,
    ) -> Result<Ph1mRetentionModeSetResponse, ContractViolation> {
        req.validate()?;
        if !self.identity_ok(&req.speaker_assertion) {
            return Ph1mRetentionModeSetResponse::v1(
                self.retention_mode,
                req.now,
                reason_codes::M_REJECT_UNKNOWN_SPEAKER,
            );
        }
        self.retention_mode = req.memory_retention_mode;
        Ph1mRetentionModeSetResponse::v1(
            self.retention_mode,
            req.now,
            reason_codes::M_RETENTION_MODE_UPDATED,
        )
    }
}

impl Ph1mRuntime {
    fn identity_ok(&self, speaker_assertion: &Ph1VoiceIdResponse) -> bool {
        matches!(speaker_assertion, Ph1VoiceIdResponse::SpeakerAssertionOk(_))
    }

    fn is_suppressed(
        &self,
        target_type: MemorySuppressionTargetType,
        target_id: &str,
        rule_kind: MemorySuppressionRuleKind,
    ) -> bool {
        self.suppression_rules
            .get(&(target_type, target_id.to_string(), rule_kind))
            .map(|entry| entry.rule.active)
            .unwrap_or(false)
    }
}

fn resume_tier_for(
    age_ns: u64,
    unresolved: bool,
    retention_mode: MemoryRetentionMode,
    cfg: &Ph1mConfig,
) -> MemoryResumeTier {
    let warm_window_ms = match retention_mode {
        MemoryRetentionMode::Default => cfg.resume_warm_window_ms,
        MemoryRetentionMode::RememberEverything => cfg.resume_warm_window_remember_everything_ms,
    };
    let hot_ns = ms_to_ns(cfg.resume_hot_window_ms);
    let warm_ns = ms_to_ns(warm_window_ms);
    let unresolved_decay_ns = ms_to_ns(cfg.unresolved_decay_window_ms);

    if unresolved && age_ns > unresolved_decay_ns {
        return MemoryResumeTier::Cold;
    }
    if age_ns <= hot_ns {
        return MemoryResumeTier::Hot;
    }
    if age_ns <= warm_ns {
        return MemoryResumeTier::Warm;
    }
    MemoryResumeTier::Cold
}

fn select_resume_action_and_delivery(
    tier: MemoryResumeTier,
    req: &Ph1mResumeSelectRequest,
    explicit_topic_request: bool,
) -> (
    MemoryResumeAction,
    MemoryResumeDeliveryMode,
    selene_kernel_contracts::ReasonCodeId,
) {
    let action = match tier {
        MemoryResumeTier::Hot => {
            if req.allow_auto_resume && !req.auto_resume_disabled_by_user {
                if req.voice_delivery_allowed || req.allow_text_delivery {
                    MemoryResumeAction::AutoLoad
                } else if req.allow_suggest {
                    MemoryResumeAction::Suggest
                } else {
                    MemoryResumeAction::None
                }
            } else if req.allow_suggest {
                MemoryResumeAction::Suggest
            } else {
                MemoryResumeAction::None
            }
        }
        MemoryResumeTier::Warm => {
            if req.allow_suggest {
                MemoryResumeAction::Suggest
            } else {
                MemoryResumeAction::None
            }
        }
        MemoryResumeTier::Cold => {
            if explicit_topic_request && req.allow_suggest {
                MemoryResumeAction::Suggest
            } else {
                MemoryResumeAction::None
            }
        }
    };

    let delivery_mode = match action {
        MemoryResumeAction::None => MemoryResumeDeliveryMode::None,
        _ => {
            if req.voice_delivery_allowed {
                MemoryResumeDeliveryMode::Voice
            } else if req.allow_text_delivery {
                MemoryResumeDeliveryMode::Text
            } else {
                MemoryResumeDeliveryMode::None
            }
        }
    };

    let reason_code = match action {
        MemoryResumeAction::AutoLoad => reason_codes::M_RESUME_AUTO_LOAD,
        MemoryResumeAction::Suggest => reason_codes::M_RESUME_SUGGEST,
        MemoryResumeAction::None => reason_codes::M_RESUME_NONE,
    };
    (action, delivery_mode, reason_code)
}

fn pending_status_is_resume_eligible(
    status: selene_kernel_contracts::ph1m::PendingWorkStatus,
) -> bool {
    matches!(
        status,
        selene_kernel_contracts::ph1m::PendingWorkStatus::Draft
            | selene_kernel_contracts::ph1m::PendingWorkStatus::Clarify
            | selene_kernel_contracts::ph1m::PendingWorkStatus::Confirm
    )
}

fn tier_rank(tier: MemoryResumeTier) -> u8 {
    match tier {
        MemoryResumeTier::Hot => 3,
        MemoryResumeTier::Warm => 2,
        MemoryResumeTier::Cold => 1,
    }
}

fn confidence_rank(confidence: MemoryConfidence) -> u8 {
    match confidence {
        MemoryConfidence::High => 3,
        MemoryConfidence::Med => 2,
        MemoryConfidence::Low => 1,
    }
}

fn exposure_for(sensitivity: MemorySensitivityFlag) -> MemoryExposureLevel {
    match sensitivity {
        MemorySensitivityFlag::Low => MemoryExposureLevel::SafeToSpeak,
        MemorySensitivityFlag::Sensitive => MemoryExposureLevel::SafeToText,
    }
}

fn is_push_memory_key(key: &str) -> bool {
    matches!(
        key,
        "preferred_name" | "profile_language" | "contact_preference"
    )
}

fn initial_policy_for(
    p: &MemoryProposedItem,
    existed: Option<&MemoryEntry>,
    now: MonotonicTimeNs,
    cfg: &Ph1mConfig,
) -> (MemoryLayer, MemoryUsePolicy, Option<MonotonicTimeNs>, u32) {
    let mut seen_count = existed.map(|e| e.seen_count).unwrap_or(0).saturating_add(1);

    let mut layer = p.layer;
    let mut use_policy = match p.layer {
        MemoryLayer::LongTerm => MemoryUsePolicy::AlwaysUsable,
        MemoryLayer::Working => MemoryUsePolicy::ContextRelevantOnly,
        MemoryLayer::Micro => MemoryUsePolicy::RepeatedOrConfirmed,
    };

    if p.sensitivity_flag == MemorySensitivityFlag::Sensitive {
        // Sensitive items must be user-requested/confirmed.
        use_policy = MemoryUsePolicy::UserRequestedOnly;
    }

    let mut expires_at = match layer {
        MemoryLayer::Micro => Some(MonotonicTimeNs(
            now.0.saturating_add(ms_to_ns(cfg.micro_ttl_ms)),
        )),
        _ => None,
    };

    // If we're updating an existing entry, keep the higher-seen_count to be deterministic.
    if let Some(e) = existed {
        seen_count = seen_count.max(e.seen_count.saturating_add(1));
        // Preserve expiry if already set later than recomputed value.
        if let (Some(old), Some(new)) = (e.expires_at, expires_at) {
            if old.0 > new.0 {
                expires_at = Some(old);
            }
        }
        // Preserve the most restrictive use policy across updates.
        use_policy = more_restrictive(use_policy, e.use_policy);
        // Preserve the "stronger" layer unless explicitly promoted later.
        layer = more_durable_layer(layer, e.layer);
    }

    (layer, use_policy, expires_at, seen_count)
}

fn more_restrictive(a: MemoryUsePolicy, b: MemoryUsePolicy) -> MemoryUsePolicy {
    use MemoryUsePolicy::*;
    // Deterministic ordering from strictest to loosest.
    let rank = |p| match p {
        UserRequestedOnly => 0,
        RepeatedOrConfirmed => 1,
        ContextRelevantOnly => 2,
        AlwaysUsable => 3,
    };
    if rank(a) <= rank(b) {
        a
    } else {
        b
    }
}

fn more_durable_layer(a: MemoryLayer, b: MemoryLayer) -> MemoryLayer {
    use MemoryLayer::*;
    let rank = |l| match l {
        LongTerm => 3,
        Working => 2,
        Micro => 1,
    };
    if rank(a) >= rank(b) {
        a
    } else {
        b
    }
}

fn ms_to_ns(ms: u64) -> u64 {
    ms.saturating_mul(1_000_000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::{
        DiarizationSegment, IdentityConfidence, SpeakerAssertionOk, SpeakerAssertionUnknown,
        SpeakerId, SpeakerLabel, UserId,
    };
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1m::{
        MemoryContextFact, MemoryEmotionalThreadState, MemoryMetricPayload, MemoryResumeAction,
        MemoryResumeDeliveryMode, MemoryResumeTier, MemoryRetentionMode, MemorySuppressionRule,
        MemorySuppressionRuleKind, MemorySuppressionTargetType, MemoryThreadDigest,
        PendingWorkItem, PendingWorkStatus, Ph1mContextBundleBuildRequest,
        Ph1mEmotionalThreadUpdateRequest, Ph1mGraphUpdateRequest, Ph1mMetricsEmitRequest,
        Ph1mResumeSelectRequest, Ph1mRetentionModeSetRequest, Ph1mSafeSummaryRequest,
        Ph1mSuppressionSetRequest, Ph1mThreadDigestUpsertRequest, MEMORY_RESUME_HOT_WINDOW_MS,
        MEMORY_RESUME_WARM_WINDOW_MS,
    };
    use selene_kernel_contracts::ph1m::{MemoryProvenance, MemoryValue};
    use selene_kernel_contracts::ReasonCodeId;

    fn policy_ok() -> PolicyContextRef {
        PolicyContextRef::v1(false, false, SafetyTier::Standard)
    }

    fn speaker_ok() -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1(
                SpeakerId::new("spk").unwrap(),
                Some(UserId::new("user").unwrap()),
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(0),
                    MonotonicTimeNs(1),
                    Some(SpeakerLabel::speaker_a()),
                )
                .unwrap()],
                SpeakerLabel::speaker_a(),
            )
            .unwrap(),
        )
    }

    fn speaker_unknown() -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1(IdentityConfidence::Medium, ReasonCodeId(1), vec![])
                .unwrap(),
        )
    }

    fn propose_item(key: &str, value: &str) -> MemoryProposedItem {
        MemoryProposedItem::v1(
            MemoryKey::new(key).unwrap(),
            MemoryValue::v1(value.to_string(), None).unwrap(),
            MemoryLayer::LongTerm,
            MemorySensitivityFlag::Low,
            MemoryConfidence::High,
            MemoryConsent::NotRequested,
            format!("Evidence: {value}"),
            MemoryProvenance::v1(None, None).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn at_m_01_no_fake_familiarity_candidates_are_evidence_backed() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());

        let req = Ph1mProposeRequest::v1(
            MonotonicTimeNs(10),
            speaker_ok(),
            policy_ok(),
            vec![propose_item("preferred_name", "John")],
        )
        .unwrap();
        rt.propose(&req).unwrap();

        let recall = Ph1mRecallRequest::v1(
            MonotonicTimeNs(11),
            speaker_ok(),
            policy_ok(),
            vec![MemoryKey::new("preferred_name").unwrap()],
            true,
            10,
        )
        .unwrap();
        let out = rt.recall(&recall).unwrap();
        assert_eq!(out.candidates.len(), 1);
        assert!(out.candidates[0].evidence_quote.contains("Evidence"));
    }

    #[test]
    fn at_m_02_micro_memory_stores_with_ttl_and_is_cautious() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let item = MemoryProposedItem::v1(
            MemoryKey::new("micro:name:benji").unwrap(),
            MemoryValue::v1("Benji".to_string(), None).unwrap(),
            MemoryLayer::Micro,
            MemorySensitivityFlag::Low,
            MemoryConfidence::Med,
            MemoryConsent::NotRequested,
            "He said: Benji".to_string(),
            MemoryProvenance::v1(None, None).unwrap(),
        )
        .unwrap();

        let req = Ph1mProposeRequest::v1(MonotonicTimeNs(0), speaker_ok(), policy_ok(), vec![item])
            .unwrap();
        let out = rt.propose(&req).unwrap();
        assert_eq!(out.decisions[0].status, MemoryCommitStatus::Stored);

        let recall = Ph1mRecallRequest::v1(
            MonotonicTimeNs(1),
            speaker_ok(),
            policy_ok(),
            vec![MemoryKey::new("micro:name:benji").unwrap()],
            true,
            10,
        )
        .unwrap();
        let out = rt.recall(&recall).unwrap();
        assert_eq!(out.candidates.len(), 1);
        assert_eq!(
            out.candidates[0].use_policy,
            MemoryUsePolicy::RepeatedOrConfirmed
        );
        assert!(out.candidates[0].expires_at.is_some());
    }

    #[test]
    fn at_m_03_user_override_is_immediate() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        rt.propose(
            &Ph1mProposeRequest::v1(
                MonotonicTimeNs(0),
                speaker_ok(),
                policy_ok(),
                vec![propose_item("nickname:him", "Ben")],
            )
            .unwrap(),
        )
        .unwrap();

        let update = MemoryProposedItem::v1(
            MemoryKey::new("nickname:him").unwrap(),
            MemoryValue::v1("Benji".to_string(), None).unwrap(),
            MemoryLayer::LongTerm,
            MemorySensitivityFlag::Low,
            MemoryConfidence::High,
            MemoryConsent::ExplicitRemember,
            "Call him Benji".to_string(),
            MemoryProvenance::v1(None, None).unwrap(),
        )
        .unwrap();
        let out = rt
            .propose(
                &Ph1mProposeRequest::v1(
                    MonotonicTimeNs(1),
                    speaker_ok(),
                    policy_ok(),
                    vec![update],
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(out.decisions[0].status, MemoryCommitStatus::Updated);

        let recall = Ph1mRecallRequest::v1(
            MonotonicTimeNs(2),
            speaker_ok(),
            policy_ok(),
            vec![MemoryKey::new("nickname:him").unwrap()],
            true,
            10,
        )
        .unwrap();
        let out = rt.recall(&recall).unwrap();
        assert_eq!(out.candidates[0].memory_value.verbatim, "Benji");
    }

    #[test]
    fn at_m_04_mixed_language_memory_preserved_verbatim() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let item = MemoryProposedItem::v1(
            MemoryKey::new("micro:call_target").unwrap(),
            MemoryValue::v1("妈妈".to_string(), None).unwrap(),
            MemoryLayer::Micro,
            MemorySensitivityFlag::Low,
            MemoryConfidence::High,
            MemoryConsent::NotRequested,
            "remind me to call 妈妈".to_string(),
            MemoryProvenance::v1(None, None).unwrap(),
        )
        .unwrap();
        rt.propose(
            &Ph1mProposeRequest::v1(MonotonicTimeNs(0), speaker_ok(), policy_ok(), vec![item])
                .unwrap(),
        )
        .unwrap();

        let out = rt
            .recall(
                &Ph1mRecallRequest::v1(
                    MonotonicTimeNs(1),
                    speaker_ok(),
                    policy_ok(),
                    vec![MemoryKey::new("micro:call_target").unwrap()],
                    true,
                    10,
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(out.candidates[0].memory_value.verbatim, "妈妈");
    }

    #[test]
    fn at_m_05_sensitive_requires_confirmation() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let item = MemoryProposedItem::v1(
            MemoryKey::new("ssn").unwrap(),
            MemoryValue::v1("123-45-6789".to_string(), None).unwrap(),
            MemoryLayer::LongTerm,
            MemorySensitivityFlag::Sensitive,
            MemoryConfidence::High,
            MemoryConsent::NotRequested,
            "my ssn is ...".to_string(),
            MemoryProvenance::v1(None, None).unwrap(),
        )
        .unwrap();
        let out = rt
            .propose(
                &Ph1mProposeRequest::v1(MonotonicTimeNs(0), speaker_ok(), policy_ok(), vec![item])
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(out.decisions[0].status, MemoryCommitStatus::NeedsConsent);

        let recall = rt
            .recall(
                &Ph1mRecallRequest::v1(
                    MonotonicTimeNs(1),
                    speaker_ok(),
                    policy_ok(),
                    vec![MemoryKey::new("ssn").unwrap()],
                    true,
                    10,
                )
                .unwrap(),
            )
            .unwrap();
        assert!(recall.candidates.is_empty());

        let confirmed = MemoryProposedItem::v1(
            MemoryKey::new("ssn").unwrap(),
            MemoryValue::v1("123-45-6789".to_string(), None).unwrap(),
            MemoryLayer::LongTerm,
            MemorySensitivityFlag::Sensitive,
            MemoryConfidence::High,
            MemoryConsent::Confirmed,
            "my ssn is ...".to_string(),
            MemoryProvenance::v1(None, None).unwrap(),
        )
        .unwrap();
        rt.propose(
            &Ph1mProposeRequest::v1(
                MonotonicTimeNs(2),
                speaker_ok(),
                policy_ok(),
                vec![confirmed],
            )
            .unwrap(),
        )
        .unwrap();

        let recall = rt
            .recall(
                &Ph1mRecallRequest::v1(
                    MonotonicTimeNs(3),
                    speaker_ok(),
                    policy_ok(),
                    vec![MemoryKey::new("ssn").unwrap()],
                    true,
                    10,
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(recall.candidates.len(), 1);
        assert_eq!(
            recall.candidates[0].sensitivity_flag,
            MemorySensitivityFlag::Sensitive
        );
        assert_eq!(
            recall.candidates[0].use_policy,
            MemoryUsePolicy::UserRequestedOnly
        );
    }

    #[test]
    fn at_m_06_forget_is_real() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        rt.propose(
            &Ph1mProposeRequest::v1(
                MonotonicTimeNs(0),
                speaker_ok(),
                policy_ok(),
                vec![propose_item("preferred_name", "John")],
            )
            .unwrap(),
        )
        .unwrap();

        let out = rt
            .forget(
                &Ph1mForgetRequest::v1(
                    MonotonicTimeNs(1),
                    speaker_ok(),
                    policy_ok(),
                    MemoryKey::new("preferred_name").unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        assert!(out.forgotten);
        assert!(out.ledger_event.is_some());

        let out = rt
            .recall(
                &Ph1mRecallRequest::v1(
                    MonotonicTimeNs(2),
                    speaker_ok(),
                    policy_ok(),
                    vec![MemoryKey::new("preferred_name").unwrap()],
                    true,
                    10,
                )
                .unwrap(),
            )
            .unwrap();
        assert!(out.candidates.is_empty());
    }

    #[test]
    fn memory_is_not_used_for_unknown_speaker() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let out = rt
            .recall(
                &Ph1mRecallRequest::v1(
                    MonotonicTimeNs(0),
                    speaker_unknown(),
                    policy_ok(),
                    vec![MemoryKey::new("preferred_name").unwrap()],
                    true,
                    10,
                )
                .unwrap(),
            )
            .unwrap();
        assert!(out.candidates.is_empty());
        assert_eq!(
            out.fail_reason_code,
            Some(reason_codes::M_REJECT_UNKNOWN_SPEAKER)
        );
    }

    #[test]
    fn resume_hot_window_auto_loads_with_72h_policy() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let hot_delta_ns = ms_to_ns(MEMORY_RESUME_HOT_WINDOW_MS.saturating_sub(1));
        let now = MonotonicTimeNs(hot_delta_ns.saturating_add(5_000_000_000));
        let digest = MemoryThreadDigest::v1(
            "thread_japan_trip".to_string(),
            "Japan ski trip".to_string(),
            vec![
                "Flights shortlisted".to_string(),
                "Need hotel confirmation".to_string(),
            ],
            false,
            true,
            MonotonicTimeNs(now.0.saturating_sub(hot_delta_ns)),
            5,
        )
        .unwrap();
        rt.thread_digest_upsert(
            &Ph1mThreadDigestUpsertRequest::v1(
                now,
                speaker_ok(),
                policy_ok(),
                MemoryRetentionMode::Default,
                digest,
                "idem_hot".to_string(),
            )
            .unwrap(),
        )
        .unwrap();

        let out = rt
            .resume_select(
                &Ph1mResumeSelectRequest::v1(
                    now,
                    speaker_ok(),
                    policy_ok(),
                    MemoryRetentionMode::Default,
                    true,
                    true,
                    true,
                    false,
                    3,
                    None,
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(out.resume_tier, Some(MemoryResumeTier::Hot));
        assert_eq!(out.resume_action, MemoryResumeAction::AutoLoad);
        assert_eq!(out.resume_summary_bullets.len(), 2);
    }

    #[test]
    fn resume_warm_window_suggests_with_30d_policy() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let warm_delta_ns = ms_to_ns(MEMORY_RESUME_WARM_WINDOW_MS.saturating_sub(1));
        let hot_ns = ms_to_ns(MEMORY_RESUME_HOT_WINDOW_MS);
        let now = MonotonicTimeNs(warm_delta_ns.saturating_add(5_000_000_000));
        let digest = MemoryThreadDigest::v1(
            "thread_payroll".to_string(),
            "Payroll follow-up".to_string(),
            vec!["Pending approval".to_string()],
            false,
            false,
            MonotonicTimeNs(
                now.0
                    .saturating_sub(warm_delta_ns.max(hot_ns.saturating_add(1))),
            ),
            2,
        )
        .unwrap();
        rt.thread_digest_upsert(
            &Ph1mThreadDigestUpsertRequest::v1(
                now,
                speaker_ok(),
                policy_ok(),
                MemoryRetentionMode::Default,
                digest,
                "idem_warm".to_string(),
            )
            .unwrap(),
        )
        .unwrap();

        let out = rt
            .resume_select(
                &Ph1mResumeSelectRequest::v1(
                    now,
                    speaker_ok(),
                    policy_ok(),
                    MemoryRetentionMode::Default,
                    true,
                    true,
                    true,
                    false,
                    3,
                    None,
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(out.resume_tier, Some(MemoryResumeTier::Warm));
        assert_eq!(out.resume_action, MemoryResumeAction::Suggest);
    }

    #[test]
    fn resume_cold_window_returns_none() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let older_than_warm_ns = ms_to_ns(MEMORY_RESUME_WARM_WINDOW_MS.saturating_add(1));
        let now = MonotonicTimeNs(older_than_warm_ns.saturating_add(5_000_000_000));
        let digest = MemoryThreadDigest::v1(
            "thread_old".to_string(),
            "Old topic".to_string(),
            vec!["Old summary".to_string()],
            false,
            false,
            MonotonicTimeNs(now.0.saturating_sub(older_than_warm_ns)),
            1,
        )
        .unwrap();
        rt.thread_digest_upsert(
            &Ph1mThreadDigestUpsertRequest::v1(
                now,
                speaker_ok(),
                policy_ok(),
                MemoryRetentionMode::Default,
                digest,
                "idem_cold".to_string(),
            )
            .unwrap(),
        )
        .unwrap();

        let out = rt
            .resume_select(
                &Ph1mResumeSelectRequest::v1(
                    now,
                    speaker_ok(),
                    policy_ok(),
                    MemoryRetentionMode::Default,
                    true,
                    true,
                    true,
                    false,
                    3,
                    None,
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(out.resume_tier, Some(MemoryResumeTier::Cold));
        assert_eq!(out.resume_action, MemoryResumeAction::None);
        assert!(out.resume_summary_bullets.is_empty());
    }

    #[test]
    fn resume_select_blocks_unknown_speaker() {
        let rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let out = rt
            .resume_select(
                &Ph1mResumeSelectRequest::v1(
                    MonotonicTimeNs(1),
                    speaker_unknown(),
                    policy_ok(),
                    MemoryRetentionMode::Default,
                    true,
                    true,
                    true,
                    false,
                    3,
                    None,
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(out.resume_action, MemoryResumeAction::None);
        assert_eq!(out.reason_code, reason_codes::M_REJECT_UNKNOWN_SPEAKER);
    }

    #[test]
    fn suppression_do_not_store_blocks_memory_propose() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let sup_req = Ph1mSuppressionSetRequest::v1(
            MonotonicTimeNs(1),
            speaker_ok(),
            policy_ok(),
            MemorySuppressionRule::v1(
                MemorySuppressionTargetType::TopicKey,
                "preferred_name".to_string(),
                MemorySuppressionRuleKind::DoNotStore,
                true,
                ReasonCodeId(101),
                MonotonicTimeNs(1),
            )
            .unwrap(),
            "idem_sup_store".to_string(),
        )
        .unwrap();
        rt.suppression_set(&sup_req).unwrap();

        let out = rt
            .propose(
                &Ph1mProposeRequest::v1(
                    MonotonicTimeNs(2),
                    speaker_ok(),
                    policy_ok(),
                    vec![propose_item("preferred_name", "John")],
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(out.decisions[0].status, MemoryCommitStatus::Rejected);
        assert_eq!(out.decisions[0].reason_code, reason_codes::M_POLICY_BLOCKED);
    }

    #[test]
    fn context_bundle_is_bounded_and_tagged() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        rt.propose(
            &Ph1mProposeRequest::v1(
                MonotonicTimeNs(10),
                speaker_ok(),
                policy_ok(),
                vec![
                    propose_item("preferred_name", "John"),
                    propose_item("project:active:jp_trip", "Japan Trip"),
                ],
            )
            .unwrap(),
        )
        .unwrap();

        let req = Ph1mContextBundleBuildRequest::v1(
            MonotonicTimeNs(11),
            speaker_ok(),
            policy_ok(),
            vec![],
            vec![MemoryContextFact::v1(
                MemoryKey::new("preferred_name").unwrap(),
                MemoryValue::v1("John".to_string(), None).unwrap(),
            )
            .unwrap()],
            Some("japan".to_string()),
            None,
            None,
            true,
            512,
            20,
            2,
        )
        .unwrap();
        let out = rt.context_bundle_build(&req).unwrap();
        assert!(out.metric_payload.context_bundle_bytes <= 512);
        assert!(out.push_items.len() + out.pull_items.len() <= 20);
        assert!(out.archive_excerpts.len() <= 2);
        let has_confirmed = out
            .push_items
            .iter()
            .chain(out.pull_items.iter())
            .any(|v| v.tag == MemoryItemTag::Confirmed);
        assert!(has_confirmed);
    }

    #[test]
    fn safe_summary_is_bounded() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        rt.propose(
            &Ph1mProposeRequest::v1(
                MonotonicTimeNs(10),
                speaker_ok(),
                policy_ok(),
                vec![
                    propose_item("preferred_name", "John"),
                    propose_item("profile_language", "English"),
                ],
            )
            .unwrap(),
        )
        .unwrap();
        let out = rt
            .safe_summary(
                &Ph1mSafeSummaryRequest::v1(
                    MonotonicTimeNs(11),
                    speaker_ok(),
                    policy_ok(),
                    10,
                    128,
                )
                .unwrap(),
            )
            .unwrap();
        assert!(out.output_bytes <= 128);
        assert!(out.summary_items.len() <= 10);
    }

    #[test]
    fn retention_mode_updates_and_affects_resume_delivery_text_fallback() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let now = MonotonicTimeNs(ms_to_ns(MEMORY_RESUME_HOT_WINDOW_MS.saturating_sub(1)) + 200);
        rt.retention_mode_set(
            &Ph1mRetentionModeSetRequest::v1(
                now,
                speaker_ok(),
                policy_ok(),
                MemoryRetentionMode::RememberEverything,
                "idem_ret".to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        rt.thread_digest_upsert(
            &Ph1mThreadDigestUpsertRequest::v1(
                now,
                speaker_ok(),
                policy_ok(),
                MemoryRetentionMode::Default,
                MemoryThreadDigest::v1(
                    "thread_japan_trip".to_string(),
                    "Japan ski trip".to_string(),
                    vec!["Flights shortlisted".to_string()],
                    false,
                    false,
                    MonotonicTimeNs(now.0.saturating_sub(ms_to_ns(1000))),
                    1,
                )
                .unwrap(),
                "idem_thread".to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        let req = Ph1mResumeSelectRequest::v1(
            now,
            speaker_ok(),
            policy_ok(),
            MemoryRetentionMode::Default,
            true,
            true,
            false,
            false,
            3,
            None,
        )
        .unwrap()
        .with_text_delivery(true)
        .unwrap();
        let out = rt.resume_select(&req).unwrap();
        assert_eq!(out.resume_action, MemoryResumeAction::AutoLoad);
        assert_eq!(out.resume_delivery_mode, MemoryResumeDeliveryMode::Text);
    }

    #[test]
    fn pending_work_is_prioritized_for_resume() {
        let rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let now = MonotonicTimeNs(ms_to_ns(MEMORY_RESUME_HOT_WINDOW_MS.saturating_sub(1)) + 200);
        let pending = PendingWorkItem::v1(
            "wo_123".to_string(),
            PendingWorkStatus::Confirm,
            Some("thread_pending_payroll".to_string()),
            vec!["Awaiting confirmation".to_string()],
            MonotonicTimeNs(now.0.saturating_sub(ms_to_ns(1000))),
            3,
        )
        .unwrap();
        let req = Ph1mResumeSelectRequest::v1(
            now,
            speaker_ok(),
            policy_ok(),
            MemoryRetentionMode::Default,
            true,
            true,
            true,
            false,
            3,
            None,
        )
        .unwrap()
        .with_pending_work_context(vec![pending], vec![], vec![])
        .unwrap();
        let out = rt.resume_select(&req).unwrap();
        assert_eq!(out.pending_work_order_id.as_deref(), Some("wo_123"));
        assert_eq!(out.resume_action, MemoryResumeAction::AutoLoad);
    }

    #[test]
    fn emotional_metrics_and_graph_capabilities_are_non_authoritative() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let emo = rt
            .emotional_thread_update(
                &Ph1mEmotionalThreadUpdateRequest::v1(
                    MonotonicTimeNs(1),
                    speaker_ok(),
                    policy_ok(),
                    MemoryEmotionalThreadState::v1(
                        "tone".to_string(),
                        vec!["calm".to_string()],
                        Some("tone only".to_string()),
                        MonotonicTimeNs(1),
                    )
                    .unwrap(),
                    "idem_emo".to_string(),
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(emo.reason_code, reason_codes::M_EMO_THREAD_UPDATED);

        let metrics = rt
            .metrics_emit(
                &Ph1mMetricsEmitRequest::v1(
                    MonotonicTimeNs(2),
                    speaker_ok(),
                    policy_ok(),
                    MemoryMetricPayload::v1(0, 0, 0, 0, 0, 0, 0, 0, 0, 0).unwrap(),
                    "idem_metrics".to_string(),
                )
                .unwrap(),
            )
            .unwrap();
        assert!(metrics.emitted);

        let graph = rt
            .graph_update(
                &Ph1mGraphUpdateRequest::v1(
                    MonotonicTimeNs(3),
                    speaker_ok(),
                    policy_ok(),
                    vec![],
                    vec![],
                    "idem_graph".to_string(),
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(graph.reason_code, reason_codes::M_GRAPH_UPDATED);
    }
}
