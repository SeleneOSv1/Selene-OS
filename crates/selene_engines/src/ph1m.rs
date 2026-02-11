#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1_voice_id::Ph1VoiceIdResponse;
use selene_kernel_contracts::ph1m::{
    MemoryCandidate, MemoryCommitDecision, MemoryCommitStatus, MemoryConfidence, MemoryConsent,
    MemoryKey, MemoryLayer, MemoryLedgerEvent, MemoryLedgerEventKind, MemoryProposedItem,
    MemoryProvenance, MemorySensitivityFlag, MemoryUsePolicy, Ph1mForgetRequest,
    Ph1mForgetResponse, Ph1mProposeRequest, Ph1mProposeResponse, Ph1mRecallRequest,
    Ph1mRecallResponse,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1mConfig {
    pub micro_ttl_ms: u64,
    pub micro_promote_after_seen: u32,
}

impl Ph1mConfig {
    pub fn mvp_v1() -> Self {
        Self {
            // Default 30 days; spec suggests 30-90 days. Stored in milliseconds for easy policy tuning.
            micro_ttl_ms: 30_u64 * 24 * 60 * 60 * 1000,
            micro_promote_after_seen: 2,
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
pub struct Ph1mRuntime {
    config: Ph1mConfig,
    current: BTreeMap<MemoryKey, MemoryEntry>,
}

impl Ph1mRuntime {
    pub fn new(config: Ph1mConfig) -> Self {
        Self {
            config,
            current: BTreeMap::new(),
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
}
