#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use selene_kernel_contracts::ph1_voice_id::Ph1VoiceIdResponse;
use selene_kernel_contracts::ph1l::SessionId;
use selene_kernel_contracts::ph1m::{
    FreshMemoryHandoff, FreshMemoryHandoffReason, MemoryAgeLabel, MemoryArchiveExcerpt,
    MemoryBundleItem, MemoryCandidate, MemoryCommitDecision, MemoryCommitStatus, MemoryConfidence,
    MemoryConflictStatus, MemoryConsent, MemoryContinuationDecision,
    MemoryContinuationDecisionKind, MemoryEmotionalThreadState, MemoryEvidencePacket,
    MemoryEvidenceType, MemoryExposureLevel, MemoryGraphEdgeInput, MemoryGraphNodeInput,
    MemoryHintEntry, MemoryItemTag, MemoryKey, MemoryLayer, MemoryLedgerEvent,
    MemoryLedgerEventKind, MemoryMetricPayload, MemoryPrivacyStatus, MemoryProposedItem,
    MemoryProvenance, MemoryProvenanceTier, MemoryRecallStyle, MemoryRecentArchiveMatch,
    MemoryResumeAction, MemoryResumeDeliveryMode, MemoryResumeTier, MemoryRetentionMode,
    MemorySafeSummaryItem, MemorySensitivityFlag, MemorySuppressionRule, MemorySuppressionRuleKind,
    MemorySuppressionTargetType, MemoryTrustLevel, MemoryUsePolicy, PendingWorkItem,
    Ph1mContextBundleBuildRequest, Ph1mContextBundleBuildResponse,
    Ph1mEmotionalThreadUpdateRequest, Ph1mEmotionalThreadUpdateResponse, Ph1mForgetRequest,
    Ph1mForgetResponse, Ph1mGraphUpdateRequest, Ph1mGraphUpdateResponse,
    Ph1mHintBundleBuildRequest, Ph1mHintBundleBuildResponse, Ph1mMetricsEmitRequest,
    Ph1mMetricsEmitResponse, Ph1mProposeRequest, Ph1mProposeResponse, Ph1mRecallRequest,
    Ph1mRecallResponse, Ph1mRecentArchiveRecallRequest, Ph1mRecentArchiveRecallResponse,
    Ph1mResumeSelectRequest, Ph1mResumeSelectResponse, Ph1mRetentionModeSetRequest,
    Ph1mRetentionModeSetResponse, Ph1mSafeSummaryRequest, Ph1mSafeSummaryResponse,
    Ph1mSuppressionSetRequest, Ph1mSuppressionSetResponse, Ph1mThreadDigestUpsertRequest,
    Ph1mThreadDigestUpsertResponse, MEMORY_CONTEXT_BUNDLE_MAX_BYTES, MEMORY_RESUME_HOT_WINDOW_MS,
    MEMORY_RESUME_WARM_WINDOW_MS, MEMORY_UNRESOLVED_DECAY_WINDOW_MS,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, ReasonCodeId, Validate};

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
    pub const M_RECENT_ARCHIVE_RECALL_READY: ReasonCodeId = ReasonCodeId(0x4D00_0017);
    pub const M_RECENT_ARCHIVE_RECALL_EMPTY: ReasonCodeId = ReasonCodeId(0x4D00_0018);
    pub const M_FRESH_CONTINUATION_READY: ReasonCodeId = ReasonCodeId(0x4D00_0019);
    pub const M_FRESH_CONTINUATION_CLARIFY: ReasonCodeId = ReasonCodeId(0x4D00_001A);
    pub const M_FRESH_CONTINUATION_NO_MATCH: ReasonCodeId = ReasonCodeId(0x4D00_001B);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1mConfig {
    pub micro_ttl_ms: u64,
    pub micro_promote_after_seen: u32,
    pub resume_hot_window_ms: u64,
    pub resume_warm_window_ms: u64,
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

const NS_PER_MS: u64 = 1_000_000;
const DAY_MS: u64 = 24 * 60 * 60 * 1000;
const FRESH_MEMORY_CONTINUATION_WINDOW_NS: u64 = 10 * 60 * 1_000_000_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshMemoryPriorTurnEvidence {
    pub created_at: MonotonicTimeNs,
    pub source_session_id: Option<SessionId>,
    pub source_thread_key: Option<String>,
    pub source_turn_ref: String,
    pub user_text: Option<String>,
    pub response_text: Option<String>,
    pub tool_family: Option<String>,
    pub entity_focus: Vec<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshMemoryContinuationRequest {
    pub now: MonotonicTimeNs,
    pub current_text: String,
    pub current_thread_key: Option<String>,
    pub current_turn_ref: String,
    pub evidence_ref_prefix: String,
    pub prior_turns: Vec<FreshMemoryPriorTurnEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshMemoryContinuationResolution {
    pub rewritten_query: Option<String>,
    pub tool_family: Option<String>,
    pub current_entity_focus: Vec<String>,
    pub previous_entity_focus: Vec<String>,
    pub memory_recall_request_ref: Option<String>,
    pub fresh_memory_handoff_ref: Option<String>,
    pub memory_evidence_packet_ref: Option<String>,
    pub memory_continuation_decision_ref: Option<String>,
    pub handoff: Option<FreshMemoryHandoff>,
    pub memory_evidence_packet: Option<MemoryEvidencePacket>,
    pub memory_continuation_decision: MemoryContinuationDecision,
}

fn recent_archive_query_terms(text: &str) -> BTreeSet<String> {
    let raw_tokens = recent_archive_raw_tokens(text);
    let mut terms = BTreeSet::new();
    let mut i = 0;
    while i < raw_tokens.len() {
        let token = raw_tokens[i].as_str();
        if token == "ph1" {
            if let Some(next) = raw_tokens.get(i + 1) {
                if matches!(next.as_str(), "m" | "x" | "e" | "l") {
                    push_recent_archive_term(&mut terms, &format!("ph1{next}"));
                    i += 2;
                    continue;
                }
            }
        }
        if token == "seventy" && raw_tokens.get(i + 1).is_some_and(|next| next == "two") {
            push_recent_archive_term(&mut terms, "72h");
            i += 2;
            continue;
        }
        if token == "celine" && recent_archive_token_window_has_wake_address(&raw_tokens, i) {
            push_recent_archive_term(&mut terms, "selene");
            i += 1;
            continue;
        }
        push_recent_archive_term(&mut terms, token);
        i += 1;
    }
    recent_archive_expand_query_terms(&mut terms);
    terms
}

fn recent_archive_raw_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            current.push(ch.to_ascii_lowercase());
        } else if !current.is_empty() {
            tokens.push(current.clone());
            current.clear();
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn recent_archive_token_window_has_wake_address(tokens: &[String], index: usize) -> bool {
    let start = index.saturating_sub(2);
    let end = (index + 3).min(tokens.len());
    tokens[start..end].iter().any(|token| {
        matches!(
            token.as_str(),
            "wake" | "waking" | "woke" | "listen" | "listening" | "selene" | "assistant"
        )
    })
}

fn recent_archive_expand_query_terms(terms: &mut BTreeSet<String>) {
    if terms.contains("desktop")
        && (terms.contains("thin")
            || terms.contains("stay")
            || terms.contains("staying")
            || terms.contains("bridge"))
    {
        for term in [
            "capture",
            "play",
            "playback",
            "audio",
            "transport",
            "render",
        ] {
            terms.insert(term.to_string());
        }
    }
    if terms.contains("72h") {
        terms.insert("recall".to_string());
        terms.insert("archive".to_string());
    }
}

fn push_recent_archive_term(terms: &mut BTreeSet<String>, term: &str) {
    let normalized = match term {
        "72" | "72h" => "72h",
        "activesessioncontext" => "active_session_context",
        "waking" | "woke" | "wakeword" => "wake",
        "recalled" | "recalling" | "remembered" | "remembering" => "recall",
        "becoming" => "become",
        "staying" | "stays" => "stay",
        "rendering" | "renders" => "render",
        "transporting" | "transports" => "transport",
        "capturing" | "captures" => "capture",
        "played" | "plays" | "playing" => "play",
        "archival" | "archived" => "archive",
        other => other,
    };
    if normalized == "active_session_context" {
        for term in ["active", "session", "context"] {
            push_recent_archive_term(terms, term);
        }
        return;
    }
    if normalized.len() < 3 && !normalized.starts_with("ph1") {
        return;
    }
    if matches!(
        normalized,
        "what"
            | "did"
            | "discuss"
            | "yesterday"
            | "about"
            | "find"
            | "where"
            | "talked"
            | "show"
            | "from"
            | "earlier"
            | "continue"
            | "with"
            | "the"
            | "was"
            | "were"
            | "and"
            | "for"
            | "our"
            | "this"
            | "that"
            | "recent"
            | "smoke"
            | "test"
    ) {
        return;
    }
    terms.insert(normalized.to_string());
}

fn recent_archive_time_window(req: &Ph1mRecentArchiveRecallRequest) -> (u64, u64) {
    let requested_window_ns = req.window_ms.saturating_mul(NS_PER_MS);
    if req.query_text.to_ascii_lowercase().contains("yesterday") {
        let end = req.now.0.saturating_sub(DAY_MS.saturating_mul(NS_PER_MS));
        let start = req
            .now
            .0
            .saturating_sub(DAY_MS.saturating_mul(2).saturating_mul(NS_PER_MS))
            .max(req.now.0.saturating_sub(requested_window_ns));
        return (start, end);
    }
    (req.now.0.saturating_sub(requested_window_ns), req.now.0)
}

fn recent_archive_excerpt(thread: &ThreadEntry, query_terms: &BTreeSet<String>) -> String {
    let selected = thread
        .summary_bullets
        .iter()
        .max_by(|left, right| {
            let left_terms = recent_archive_query_terms(left);
            let right_terms = recent_archive_query_terms(right);
            recent_archive_overlap_score(query_terms, &left_terms)
                .cmp(&recent_archive_overlap_score(query_terms, &right_terms))
        })
        .filter(|bullet| {
            let bullet_terms = recent_archive_query_terms(bullet);
            query_terms.iter().any(|term| bullet_terms.contains(term))
        })
        .or_else(|| thread.summary_bullets.first())
        .map(String::as_str)
        .unwrap_or(thread.thread_title.as_str());
    truncate_chars(selected, 512)
}

fn recent_archive_match_reason(overlap: &[String]) -> String {
    let joined = overlap
        .iter()
        .take(8)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    truncate_chars(&format!("matched terms: {joined}"), 192)
}

fn recent_archive_overlap_score(
    query_terms: &BTreeSet<String>,
    corpus_terms: &BTreeSet<String>,
) -> u16 {
    query_terms
        .iter()
        .filter(|term| corpus_terms.contains(*term))
        .map(|term| recent_archive_term_weight(term))
        .sum::<usize>()
        .min(u16::MAX as usize) as u16
}

fn recent_archive_term_weight(term: &str) -> usize {
    match term {
        "ph1m" | "ph1x" | "desktop" | "selene" => 24,
        "archive" | "storage" | "intent" | "context" | "capture" | "transport" | "render" => 18,
        "recall" | "active" | "wake" | "listen" | "memory" | "72h" => 14,
        _ => 10,
    }
}

fn recent_archive_match_density_bonus(base_score: usize, corpus_term_count: usize) -> usize {
    if corpus_term_count == 0 {
        0
    } else {
        base_score.saturating_mul(32) / corpus_term_count
    }
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect()
}

fn fresh_memory_normalized_text(text: &str) -> String {
    let mut out = String::new();
    let mut last_space = false;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_space = false;
        } else if ch.is_whitespace() || matches!(ch, '?' | '？' | '.' | ',' | '!' | ';' | ':') {
            if !last_space && !out.is_empty() {
                out.push(' ');
                last_space = true;
            }
        }
    }
    out.trim().to_string()
}

fn fresh_memory_clean_place_candidate(raw: &str) -> Option<String> {
    let mut text = raw
        .trim()
        .trim_matches(|ch: char| matches!(ch, '?' | '？' | '.' | ',' | '!' | ';' | ':'))
        .trim()
        .to_string();
    for prefix in ["in ", "for ", "at "] {
        if text.to_ascii_lowercase().starts_with(prefix) {
            text = text[prefix.len()..].trim().to_string();
        }
    }
    while text.to_ascii_lowercase().ends_with(" please") {
        let keep = text.len().saturating_sub(" please".len());
        text = text[..keep].trim().to_string();
    }
    if text.is_empty() || text.len() > 96 {
        return None;
    }
    let normalized = fresh_memory_normalized_text(&text);
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();
    if tokens.is_empty() || tokens.len() > 5 {
        return None;
    }
    if tokens.iter().any(|token| {
        matches!(
            *token,
            "what"
                | "who"
                | "why"
                | "how"
                | "name"
                | "story"
                | "joke"
                | "write"
                | "tell"
                | "explain"
                | "approve"
                | "payroll"
                | "session"
                | "archive"
                | "search"
                | "memory"
                | "evidence"
                | "result"
        )
    }) {
        return None;
    }
    Some(fresh_memory_title_case_place(&text))
}

fn fresh_memory_title_case_place(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            let rest = chars.collect::<String>();
            format!("{}{}", first.to_uppercase(), rest.to_ascii_lowercase())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn fresh_memory_followup_location(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.len() > 160 {
        return None;
    }
    let normalized = fresh_memory_normalized_text(trimmed);
    for prefix in [
        "and what about in ",
        "also what about in ",
        "what about in ",
        "and what about ",
        "also what about ",
        "what about ",
        "same question for ",
        "same for ",
        "do the same for ",
    ] {
        if let Some(rest) = normalized.strip_prefix(prefix) {
            return fresh_memory_clean_place_candidate(rest);
        }
    }
    None
}

fn fresh_memory_bare_place_fragment(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.contains('?') || trimmed.contains('？') || trimmed.len() > 96 {
        return None;
    }
    let normalized = fresh_memory_normalized_text(trimmed);
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();
    if tokens.is_empty() || tokens.len() > 4 {
        return None;
    }
    if tokens.iter().any(|token| {
        matches!(
            *token,
            "and"
                | "also"
                | "same"
                | "what"
                | "about"
                | "time"
                | "weather"
                | "name"
                | "story"
                | "joke"
                | "approve"
                | "payroll"
                | "yes"
                | "no"
                | "do"
                | "it"
                | "you"
        )
    }) {
        return None;
    }
    fresh_memory_clean_place_candidate(trimmed)
}

fn fresh_memory_extract_time_place(text: &str) -> Option<String> {
    let trimmed = text.trim();
    let normalized = fresh_memory_normalized_text(trimmed);
    if !(normalized.contains("time")
        || normalized.contains("clock")
        || normalized.contains("what s it in")
        || normalized.contains("whats it in"))
    {
        return None;
    }
    for marker in [" in ", " for ", " at "] {
        if let Some(idx) = normalized.rfind(marker) {
            let raw_start = trimmed
                .to_ascii_lowercase()
                .rfind(marker.trim())
                .unwrap_or(idx)
                .saturating_add(marker.trim().len());
            return fresh_memory_clean_place_candidate(&trimmed[raw_start..]);
        }
    }
    None
}

fn fresh_memory_prior_tool_family(prior: &FreshMemoryPriorTurnEvidence) -> Option<&str> {
    prior.tool_family.as_deref().or_else(|| {
        prior
            .user_text
            .as_deref()
            .and_then(|text| fresh_memory_extract_time_place(text).map(|_| "time"))
    })
}

fn fresh_memory_latest_prior_tool_turn<'a>(
    req: &'a FreshMemoryContinuationRequest,
    tool_family: &str,
) -> Option<(&'a FreshMemoryPriorTurnEvidence, String)> {
    req.prior_turns.iter().rev().find_map(|prior| {
        let within_window =
            req.now.0.saturating_sub(prior.created_at.0) <= FRESH_MEMORY_CONTINUATION_WINDOW_NS;
        if !within_window || fresh_memory_prior_tool_family(prior) != Some(tool_family) {
            return None;
        }
        let entity = prior.entity_focus.first().cloned().or_else(|| {
            prior
                .user_text
                .as_deref()
                .and_then(fresh_memory_extract_time_place)
        })?;
        Some((prior, entity))
    })
}

fn fresh_memory_ref(prefix: &str, suffix: &str) -> String {
    let compact = suffix
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, ':' | '_' | '-') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    truncate_chars(&format!("{prefix}:{compact}"), 128)
}

fn fresh_memory_compact_refs(
    prior: &FreshMemoryPriorTurnEvidence,
    current_turn_ref: &str,
) -> Vec<String> {
    let mut refs = Vec::new();
    refs.push(truncate_chars(&prior.source_turn_ref, 128));
    refs.push(fresh_memory_ref("current_turn", current_turn_ref));
    for value in &prior.evidence_refs {
        let value = truncate_chars(value, 128);
        if !refs.contains(&value) {
            refs.push(value);
        }
        if refs.len() >= 8 {
            break;
        }
    }
    refs
}

fn fresh_memory_no_match_resolution(
    decision: MemoryContinuationDecisionKind,
    confidence: MemoryConfidence,
    reason_code: ReasonCodeId,
    clarification_prompt: Option<String>,
) -> Result<FreshMemoryContinuationResolution, ContractViolation> {
    Ok(FreshMemoryContinuationResolution {
        rewritten_query: None,
        tool_family: None,
        current_entity_focus: Vec::new(),
        previous_entity_focus: Vec::new(),
        memory_recall_request_ref: None,
        fresh_memory_handoff_ref: None,
        memory_evidence_packet_ref: None,
        memory_continuation_decision_ref: None,
        handoff: None,
        memory_evidence_packet: None,
        memory_continuation_decision: MemoryContinuationDecision::v1(
            decision,
            confidence,
            reason_code,
            None,
            None,
            clarification_prompt,
        )?,
    })
}

pub fn resolve_fresh_memory_continuation(
    req: &FreshMemoryContinuationRequest,
) -> Result<FreshMemoryContinuationResolution, ContractViolation> {
    let current = req.current_text.trim();
    if current.is_empty() {
        return fresh_memory_no_match_resolution(
            MemoryContinuationDecisionKind::NoMemoryMatch,
            MemoryConfidence::Low,
            reason_codes::M_FRESH_CONTINUATION_NO_MATCH,
            None,
        );
    }

    let followup_location = fresh_memory_followup_location(current);
    let bare_fragment = fresh_memory_bare_place_fragment(current);
    if followup_location.is_none() && bare_fragment.is_none() {
        return fresh_memory_no_match_resolution(
            MemoryContinuationDecisionKind::AnswerNormally,
            MemoryConfidence::Low,
            reason_codes::M_FRESH_CONTINUATION_NO_MATCH,
            None,
        );
    }

    let Some((prior, previous_place)) = fresh_memory_latest_prior_tool_turn(req, "time") else {
        return fresh_memory_no_match_resolution(
            MemoryContinuationDecisionKind::NoMemoryMatch,
            MemoryConfidence::Low,
            reason_codes::M_FRESH_CONTINUATION_NO_MATCH,
            None,
        );
    };

    if let Some(fragment) = bare_fragment.filter(|_| followup_location.is_none()) {
        let decision_ref = fresh_memory_ref(
            "ph1m_continuation_decision",
            &format!("clarify:{}", req.evidence_ref_prefix),
        );
        let packet_ref = fresh_memory_ref(
            "ph1m_memory_evidence",
            &format!("fresh:{}", req.evidence_ref_prefix),
        );
        let evidence_refs = fresh_memory_compact_refs(prior, &req.current_turn_ref);
        let packet = MemoryEvidencePacket::v1(
            MemoryEvidenceType::Fresh,
            Some("time lookup".to_string()),
            MemoryAgeLabel::BeforeSleep,
            MemoryConfidence::Med,
            evidence_refs.clone(),
            false,
            true,
            Some(format!("We were checking the time for {previous_place}.")),
            true,
            MemoryRecallStyle::IRemember,
            MemoryTrustLevel::InferredSummary,
            MemoryPrivacyStatus::Allowed,
            MemoryConflictStatus::Current,
        )?;
        let prompt = format!("Do you mean the time question, or something else about {fragment}?");
        return Ok(FreshMemoryContinuationResolution {
            rewritten_query: None,
            tool_family: Some("time".to_string()),
            current_entity_focus: vec![fragment],
            previous_entity_focus: vec![previous_place],
            memory_recall_request_ref: Some(fresh_memory_ref(
                "ph1m_recall_request",
                &format!("fresh:{}", req.evidence_ref_prefix),
            )),
            fresh_memory_handoff_ref: None,
            memory_evidence_packet_ref: Some(packet_ref.clone()),
            memory_continuation_decision_ref: Some(decision_ref.clone()),
            handoff: None,
            memory_evidence_packet: Some(packet),
            memory_continuation_decision: MemoryContinuationDecision::v1(
                MemoryContinuationDecisionKind::AskClarification,
                MemoryConfidence::Med,
                reason_codes::M_FRESH_CONTINUATION_CLARIFY,
                Some(packet_ref),
                Some("fresh time topic was available but the fragment was too vague".to_string()),
                Some(prompt),
            )?,
        });
    }

    let current_place = followup_location.expect("checked above");
    let evidence_refs = fresh_memory_compact_refs(prior, &req.current_turn_ref);
    let handoff_ref = fresh_memory_ref(
        "ph1m_fresh_handoff",
        &format!("sleep:{}", req.evidence_ref_prefix),
    );
    let packet_ref = fresh_memory_ref(
        "ph1m_memory_evidence",
        &format!("fresh:{}", req.evidence_ref_prefix),
    );
    let decision_ref = fresh_memory_ref(
        "ph1m_continuation_decision",
        &format!("continue:{}", req.evidence_ref_prefix),
    );
    let recall_ref = fresh_memory_ref(
        "ph1m_recall_request",
        &format!("fresh:{}", req.evidence_ref_prefix),
    );
    let handoff = FreshMemoryHandoff::v1(
        handoff_ref.clone(),
        prior.source_session_id,
        prior
            .source_thread_key
            .clone()
            .or_else(|| req.current_thread_key.clone()),
        Some(prior.source_turn_ref.clone()),
        Some("time lookup".to_string()),
        Some("answer local time".to_string()),
        Some("time".to_string()),
        vec![previous_place.clone()],
        Some("time_answer".to_string()),
        MemoryAgeLabel::BeforeSleep,
        MemoryConfidence::High,
        evidence_refs.clone(),
        true,
        FreshMemoryHandoffReason::SessionSleep,
        None,
    )?;
    let packet = MemoryEvidencePacket::v1(
        MemoryEvidenceType::Fresh,
        Some("time lookup".to_string()),
        MemoryAgeLabel::BeforeSleep,
        MemoryConfidence::High,
        evidence_refs,
        true,
        false,
        Some(format!("We were checking the time for {previous_place}.")),
        true,
        MemoryRecallStyle::IRemember,
        MemoryTrustLevel::InferredSummary,
        MemoryPrivacyStatus::Allowed,
        MemoryConflictStatus::Current,
    )?;
    Ok(FreshMemoryContinuationResolution {
        rewritten_query: Some(format!("what is the time in {current_place}")),
        tool_family: Some("time".to_string()),
        current_entity_focus: vec![current_place],
        previous_entity_focus: vec![previous_place],
        memory_recall_request_ref: Some(recall_ref),
        fresh_memory_handoff_ref: Some(handoff_ref),
        memory_evidence_packet_ref: Some(packet_ref.clone()),
        memory_continuation_decision_ref: Some(decision_ref),
        handoff: Some(handoff),
        memory_evidence_packet: Some(packet),
        memory_continuation_decision: MemoryContinuationDecision::v1(
            MemoryContinuationDecisionKind::ContinueAutomatically,
            MemoryConfidence::High,
            reason_codes::M_FRESH_CONTINUATION_READY,
            Some(packet_ref),
            Some("same time question with a new place".to_string()),
            None,
        )?,
    })
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

    pub fn fresh_memory_continuation(
        &self,
        req: &FreshMemoryContinuationRequest,
    ) -> Result<FreshMemoryContinuationResolution, ContractViolation> {
        let _ = self.config;
        resolve_fresh_memory_continuation(req)
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

    pub fn recent_archive_recall(
        &self,
        req: &Ph1mRecentArchiveRecallRequest,
    ) -> Result<Ph1mRecentArchiveRecallResponse, ContractViolation> {
        req.validate()?;

        if !self.identity_ok(&req.speaker_assertion) {
            return Ph1mRecentArchiveRecallResponse::v1(
                vec![],
                reason_codes::M_REJECT_UNKNOWN_SPEAKER,
            );
        }

        if req.policy_context_ref.privacy_mode {
            return Ph1mRecentArchiveRecallResponse::v1(vec![], reason_codes::M_POLICY_BLOCKED);
        }

        let query_terms = recent_archive_query_terms(&req.query_text);
        if query_terms.is_empty() {
            return Ph1mRecentArchiveRecallResponse::v1(
                vec![],
                reason_codes::M_RECENT_ARCHIVE_RECALL_EMPTY,
            );
        }
        let (window_start, window_end) = recent_archive_time_window(req);
        let mut matches = Vec::new();

        for thread in self.threads.values() {
            if thread.last_updated_at.0 < window_start || thread.last_updated_at.0 > window_end {
                continue;
            }
            if self.is_suppressed(
                MemorySuppressionTargetType::ThreadId,
                &thread.thread_id,
                MemorySuppressionRuleKind::DoNotMention,
            ) {
                continue;
            }

            let mut corpus_terms = recent_archive_query_terms(&thread.thread_title);
            for bullet in &thread.summary_bullets {
                corpus_terms.extend(recent_archive_query_terms(bullet));
            }
            let overlap = query_terms
                .iter()
                .filter(|term| corpus_terms.contains(*term))
                .cloned()
                .collect::<Vec<_>>();
            if overlap.is_empty() {
                continue;
            }

            let base_score = recent_archive_overlap_score(&query_terms, &corpus_terms) as usize;
            let score = base_score
                .saturating_add(recent_archive_match_density_bonus(
                    base_score,
                    corpus_terms.len(),
                ))
                .saturating_add((thread.pinned as usize) * 2)
                .saturating_add((thread.unresolved as usize).min(1))
                .min(u16::MAX as usize) as u16;
            matches.push(MemoryRecentArchiveMatch::v1(
                format!("thread:{}", thread.thread_id),
                Some(thread.thread_id.clone()),
                thread.last_updated_at,
                recent_archive_excerpt(thread, &query_terms),
                recent_archive_match_reason(&overlap),
                score.max(1),
            )?);
        }

        matches.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| b.matched_at.0.cmp(&a.matched_at.0))
                .then_with(|| a.archive_ref_id.cmp(&b.archive_ref_id))
        });
        matches.truncate(req.max_matches as usize);

        let reason_code = if matches.is_empty() {
            reason_codes::M_RECENT_ARCHIVE_RECALL_EMPTY
        } else {
            reason_codes::M_RECENT_ARCHIVE_RECALL_READY
        };
        Ph1mRecentArchiveRecallResponse::v1(matches, reason_code)
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
                    let summary_match = if effective_mode == MemoryRetentionMode::RememberEverything
                    {
                        entry
                            .summary_bullets
                            .iter()
                            .any(|v| v.to_ascii_lowercase().contains(topic_hint))
                    } else {
                        false
                    };
                    if !title.contains(topic_hint)
                        && !thread_id.contains(topic_hint)
                        && !summary_match
                    {
                        return None;
                    }
                }
                let age_ns = req.now.0.saturating_sub(entry.last_updated_at.0);
                let tier = resume_tier_for(age_ns, entry.unresolved, effective_mode, &self.config);
                if tier == MemoryResumeTier::Cold && req.topic_hint.is_none() {
                    return None;
                }
                let unresolved_boost =
                    entry.unresolved && age_ns <= ms_to_ns(self.config.unresolved_decay_window_ms);
                Some((entry, tier, unresolved_boost))
            })
            .collect();

        candidates.sort_by(|a, b| {
            let (a_entry, a_tier, a_unresolved_boost) = a;
            let (b_entry, b_tier, b_unresolved_boost) = b;
            tier_rank(*b_tier)
                .cmp(&tier_rank(*a_tier))
                .then_with(|| {
                    if effective_mode == MemoryRetentionMode::RememberEverything {
                        b_entry.use_count.cmp(&a_entry.use_count)
                    } else {
                        core::cmp::Ordering::Equal
                    }
                })
                .then_with(|| {
                    b_entry
                        .pinned
                        .cmp(&a_entry.pinned)
                        .then_with(|| b_unresolved_boost.cmp(a_unresolved_boost))
                        .then_with(|| b_entry.last_updated_at.0.cmp(&a_entry.last_updated_at.0))
                        .then_with(|| b_entry.use_count.cmp(&a_entry.use_count))
                        .then_with(|| a_entry.thread_id.cmp(&b_entry.thread_id))
                })
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
    let _ = retention_mode;
    let warm_window_ms = cfg.resume_warm_window_ms;
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
        Ph1mRecentArchiveRecallRequest, Ph1mResumeSelectRequest, Ph1mRetentionModeSetRequest,
        Ph1mSafeSummaryRequest, Ph1mSuppressionSetRequest, Ph1mThreadDigestUpsertRequest,
        MEMORY_RESUME_HOT_WINDOW_MS, MEMORY_RESUME_WARM_WINDOW_MS,
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
        propose_item_with_confidence(key, value, MemoryConfidence::High)
    }

    fn propose_item_with_confidence(
        key: &str,
        value: &str,
        confidence: MemoryConfidence,
    ) -> MemoryProposedItem {
        MemoryProposedItem::v1(
            MemoryKey::new(key).unwrap(),
            MemoryValue::v1(value.to_string(), None).unwrap(),
            MemoryLayer::LongTerm,
            MemorySensitivityFlag::Low,
            confidence,
            MemoryConsent::NotRequested,
            format!("Evidence: {value}"),
            MemoryProvenance::v1(None, None).unwrap(),
        )
        .unwrap()
    }

    fn propose_micro_item_with_confidence(
        key: &str,
        value: &str,
        confidence: MemoryConfidence,
    ) -> MemoryProposedItem {
        MemoryProposedItem::v1(
            MemoryKey::new(key).unwrap(),
            MemoryValue::v1(value.to_string(), None).unwrap(),
            MemoryLayer::Micro,
            MemorySensitivityFlag::Low,
            confidence,
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
    fn recent_archive_recall_by_topic_uses_72h_window() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let now = MonotonicTimeNs(ms_to_ns(96 * 60 * 60 * 1000));
        let in_window = MemoryThreadDigest::v1(
            "thread_active_session_context".to_string(),
            "Active session context repair".to_string(),
            vec![
                "We decided active session context belongs to PH1.X and adapter, not PH1.M."
                    .to_string(),
                "Sydney and London are same-session follow-ups.".to_string(),
            ],
            false,
            false,
            MonotonicTimeNs(now.0.saturating_sub(ms_to_ns(30 * 60 * 60 * 1000))),
            3,
        )
        .unwrap();
        let out_of_window = MemoryThreadDigest::v1(
            "thread_old_context".to_string(),
            "Old active session context note".to_string(),
            vec!["This older note should not be returned by recent recall.".to_string()],
            false,
            false,
            MonotonicTimeNs(now.0.saturating_sub(ms_to_ns(90 * 60 * 60 * 1000))),
            1,
        )
        .unwrap();
        for (digest, key) in [
            (in_window, "idem_recent_in"),
            (out_of_window, "idem_recent_out"),
        ] {
            rt.thread_digest_upsert(
                &Ph1mThreadDigestUpsertRequest::v1(
                    now,
                    speaker_ok(),
                    policy_ok(),
                    MemoryRetentionMode::Default,
                    digest,
                    key.to_string(),
                )
                .unwrap(),
            )
            .unwrap();
        }

        let out = rt
            .recent_archive_recall(
                &Ph1mRecentArchiveRecallRequest::v1(
                    now,
                    speaker_ok(),
                    policy_ok(),
                    "What did we decide about active session context?".to_string(),
                    MEMORY_RESUME_HOT_WINDOW_MS,
                    4,
                )
                .unwrap(),
            )
            .unwrap();

        assert_eq!(out.matches.len(), 1);
        assert_eq!(
            out.matches[0].thread_id.as_deref(),
            Some("thread_active_session_context")
        );
        assert!(out.matches[0].excerpt_text.contains("PH1.X"));
        assert_eq!(out.reason_code, reason_codes::M_RECENT_ARCHIVE_RECALL_READY);
    }

    #[test]
    fn recent_archive_yesterday_uses_fixed_clock_range() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let now = MonotonicTimeNs(ms_to_ns(120 * 60 * 60 * 1000));
        let yesterday = MemoryThreadDigest::v1(
            "thread_yesterday_continuity".to_string(),
            "Continuous conversation yesterday".to_string(),
            vec!["We discussed continuous conversation and session continuity.".to_string()],
            false,
            false,
            MonotonicTimeNs(now.0.saturating_sub(ms_to_ns(30 * 60 * 60 * 1000))),
            2,
        )
        .unwrap();
        let today = MemoryThreadDigest::v1(
            "thread_today_continuity".to_string(),
            "Continuous conversation today".to_string(),
            vec!["This current-day note must not satisfy a yesterday query.".to_string()],
            false,
            false,
            MonotonicTimeNs(now.0.saturating_sub(ms_to_ns(3 * 60 * 60 * 1000))),
            2,
        )
        .unwrap();
        for (digest, key) in [(yesterday, "idem_yesterday"), (today, "idem_today")] {
            rt.thread_digest_upsert(
                &Ph1mThreadDigestUpsertRequest::v1(
                    now,
                    speaker_ok(),
                    policy_ok(),
                    MemoryRetentionMode::Default,
                    digest,
                    key.to_string(),
                )
                .unwrap(),
            )
            .unwrap();
        }

        let out = rt
            .recent_archive_recall(
                &Ph1mRecentArchiveRecallRequest::v1(
                    now,
                    speaker_ok(),
                    policy_ok(),
                    "What did we discuss yesterday about continuous conversation?".to_string(),
                    MEMORY_RESUME_HOT_WINDOW_MS,
                    4,
                )
                .unwrap(),
            )
            .unwrap();

        assert_eq!(out.matches.len(), 1);
        assert_eq!(
            out.matches[0].thread_id.as_deref(),
            Some("thread_yesterday_continuity")
        );
    }

    #[test]
    fn recent_archive_recall_distinguishes_ph1m_ph1x_and_desktop_thin_queries() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let now = MonotonicTimeNs(ms_to_ns(96 * 60 * 60 * 1000));
        for (thread_id, title, bullet, idem) in [
            (
                "thread_seed_active_context",
                "Active session context seed",
                "For this smoke test, activeSessionContext belongs to PH1.X and adapter, not PH1.M.",
                "idem_seed_active_context",
            ),
            (
                "thread_seed_ph1m_recall",
                "PH1.M recent recall seed",
                "For this smoke test, seventy-two hour recall belongs to PH1.M and the storage archive.",
                "idem_seed_ph1m_recall",
            ),
            (
                "thread_seed_desktop_thin",
                "Desktop thin bridge seed",
                "For this smoke test, Desktop must only capture, play audio, transport, and render.",
                "idem_seed_desktop_thin",
            ),
        ] {
            rt.thread_digest_upsert(
                &Ph1mThreadDigestUpsertRequest::v1(
                    now,
                    speaker_ok(),
                    policy_ok(),
                    MemoryRetentionMode::Default,
                    MemoryThreadDigest::v1(
                        thread_id.to_string(),
                        title.to_string(),
                        vec![bullet.to_string()],
                        false,
                        false,
                        now,
                        1,
                    )
                    .unwrap(),
                    idem.to_string(),
                )
                .unwrap(),
            )
            .unwrap();
        }

        let ph1m = rt
            .recent_archive_recall(
                &Ph1mRecentArchiveRecallRequest::v1(
                    now,
                    speaker_ok(),
                    policy_ok(),
                    "What did we decide about PH1.M and seventy-two hour recall?".to_string(),
                    MEMORY_RESUME_HOT_WINDOW_MS,
                    2,
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(
            ph1m.matches[0].thread_id.as_deref(),
            Some("thread_seed_ph1m_recall")
        );

        let ph1x = rt
            .recent_archive_recall(
                &Ph1mRecentArchiveRecallRequest::v1(
                    now,
                    speaker_ok(),
                    policy_ok(),
                    "What did we discuss about active session context?".to_string(),
                    MEMORY_RESUME_HOT_WINDOW_MS,
                    2,
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(
            ph1x.matches[0].thread_id.as_deref(),
            Some("thread_seed_active_context")
        );

        let desktop = rt
            .recent_archive_recall(
                &Ph1mRecentArchiveRecallRequest::v1(
                    now,
                    speaker_ok(),
                    policy_ok(),
                    "Find where we talked about Desktop staying thin.".to_string(),
                    MEMORY_RESUME_HOT_WINDOW_MS,
                    2,
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(
            desktop.matches[0].thread_id.as_deref(),
            Some("thread_seed_desktop_thin")
        );
    }

    #[test]
    fn recent_archive_recall_normalizes_celine_only_in_wake_context() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let now = MonotonicTimeNs(ms_to_ns(96 * 60 * 60 * 1000));
        for (thread_id, title, bullet, idem) in [
            (
                "thread_seed_wake_celine",
                "Wake Celine transcript seed",
                "For this smoke test, Wake Celine should acknowledge readiness and then listen for the real prompt.",
                "idem_seed_wake_celine",
            ),
            (
                "thread_seed_celine_entity",
                "Celine entity note",
                "The project note mentioned Celine as an unrelated person name.",
                "idem_seed_celine_entity",
            ),
        ] {
            rt.thread_digest_upsert(
                &Ph1mThreadDigestUpsertRequest::v1(
                    now,
                    speaker_ok(),
                    policy_ok(),
                    MemoryRetentionMode::Default,
                    MemoryThreadDigest::v1(
                        thread_id.to_string(),
                        title.to_string(),
                        vec![bullet.to_string()],
                        false,
                        false,
                        now,
                        1,
                    )
                    .unwrap(),
                    idem.to_string(),
                )
                .unwrap(),
            )
            .unwrap();
        }

        let wake = rt
            .recent_archive_recall(
                &Ph1mRecentArchiveRecallRequest::v1(
                    now,
                    speaker_ok(),
                    policy_ok(),
                    "What did we say about waking Selene?".to_string(),
                    MEMORY_RESUME_HOT_WINDOW_MS,
                    2,
                )
                .unwrap(),
            )
            .unwrap();

        assert_eq!(
            wake.matches[0].thread_id.as_deref(),
            Some("thread_seed_wake_celine")
        );
        assert_ne!(
            wake.matches[0].thread_id.as_deref(),
            Some("thread_seed_celine_entity")
        );
    }

    #[test]
    fn recent_archive_recall_blocks_unknown_speaker() {
        let rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let out = rt
            .recent_archive_recall(
                &Ph1mRecentArchiveRecallRequest::v1(
                    MonotonicTimeNs(ms_to_ns(10 * 60 * 60 * 1000)),
                    speaker_unknown(),
                    policy_ok(),
                    "What did we decide about active session context?".to_string(),
                    MEMORY_RESUME_HOT_WINDOW_MS,
                    4,
                )
                .unwrap(),
            )
            .unwrap();
        assert!(out.matches.is_empty());
        assert_eq!(out.reason_code, reason_codes::M_REJECT_UNKNOWN_SPEAKER);
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
        assert_eq!(out.resume_tier, None);
        assert_eq!(out.resume_action, MemoryResumeAction::None);
        assert!(out.resume_summary_bullets.is_empty());
    }

    #[test]
    fn remember_everything_keeps_warm_boundary_at_30d() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let older_than_warm_ns = ms_to_ns(MEMORY_RESUME_WARM_WINDOW_MS.saturating_add(1));
        let now = MonotonicTimeNs(older_than_warm_ns.saturating_add(6_000_000_000));
        rt.retention_mode_set(
            &Ph1mRetentionModeSetRequest::v1(
                now,
                speaker_ok(),
                policy_ok(),
                MemoryRetentionMode::RememberEverything,
                "idem_ret_keep_30d".to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        rt.thread_digest_upsert(
            &Ph1mThreadDigestUpsertRequest::v1(
                now,
                speaker_ok(),
                policy_ok(),
                MemoryRetentionMode::RememberEverything,
                MemoryThreadDigest::v1(
                    "thread_old_re".to_string(),
                    "Older than warm".to_string(),
                    vec!["old summary".to_string()],
                    false,
                    false,
                    MonotonicTimeNs(now.0.saturating_sub(older_than_warm_ns)),
                    2,
                )
                .unwrap(),
                "idem_thread_keep_30d".to_string(),
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
                    MemoryRetentionMode::RememberEverything,
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
        assert_eq!(out.resume_tier, None);
        assert_eq!(out.resume_action, MemoryResumeAction::None);
    }

    #[test]
    fn remember_everything_topic_hint_matches_summary_bullets() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let warm_delta_ns = ms_to_ns(MEMORY_RESUME_WARM_WINDOW_MS.saturating_sub(1));
        let hot_ns = ms_to_ns(MEMORY_RESUME_HOT_WINDOW_MS);
        let now = MonotonicTimeNs(warm_delta_ns.saturating_add(7_000_000_000));
        rt.retention_mode_set(
            &Ph1mRetentionModeSetRequest::v1(
                now,
                speaker_ok(),
                policy_ok(),
                MemoryRetentionMode::RememberEverything,
                "idem_ret_summary_hint".to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        rt.thread_digest_upsert(
            &Ph1mThreadDigestUpsertRequest::v1(
                now,
                speaker_ok(),
                policy_ok(),
                MemoryRetentionMode::RememberEverything,
                MemoryThreadDigest::v1(
                    "thread_ops".to_string(),
                    "Operations updates".to_string(),
                    vec!["Pizza order blocked by missing integration".to_string()],
                    false,
                    false,
                    MonotonicTimeNs(
                        now.0
                            .saturating_sub(warm_delta_ns.max(hot_ns.saturating_add(1))),
                    ),
                    4,
                )
                .unwrap(),
                "idem_thread_summary_hint".to_string(),
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
                    MemoryRetentionMode::RememberEverything,
                    true,
                    true,
                    true,
                    false,
                    3,
                    Some("pizza".to_string()),
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(out.selected_thread_id.as_deref(), Some("thread_ops"));
        assert_eq!(out.resume_tier, Some(MemoryResumeTier::Warm));
        assert_eq!(out.resume_action, MemoryResumeAction::Suggest);
    }

    #[test]
    fn remember_everything_prefers_higher_use_warm_thread_for_resume() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let cfg = Ph1mConfig::mvp_v1();
        let older_warm_age_ms = cfg.resume_hot_window_ms.saturating_add(10_000);
        let newer_warm_age_ms = cfg.resume_hot_window_ms.saturating_add(1_000);
        assert!(older_warm_age_ms <= cfg.resume_warm_window_ms);
        assert!(newer_warm_age_ms <= cfg.resume_warm_window_ms);
        let now = MonotonicTimeNs(ms_to_ns(older_warm_age_ms).saturating_add(7_500_000_000));

        rt.retention_mode_set(
            &Ph1mRetentionModeSetRequest::v1(
                now,
                speaker_ok(),
                policy_ok(),
                MemoryRetentionMode::RememberEverything,
                "idem_ret_rank_remember_everything".to_string(),
            )
            .unwrap(),
        )
        .unwrap();

        let shared_title = "Shared warm thread".to_string();
        let shared_summary = vec!["Shared summary".to_string()];

        rt.thread_digest_upsert(
            &Ph1mThreadDigestUpsertRequest::v1(
                now,
                speaker_ok(),
                policy_ok(),
                MemoryRetentionMode::RememberEverything,
                MemoryThreadDigest::v1(
                    "thread_high_use_older".to_string(),
                    shared_title.clone(),
                    shared_summary.clone(),
                    false,
                    false,
                    MonotonicTimeNs(now.0.saturating_sub(ms_to_ns(older_warm_age_ms))),
                    9,
                )
                .unwrap(),
                "idem_thread_high_use_older".to_string(),
            )
            .unwrap(),
        )
        .unwrap();

        rt.thread_digest_upsert(
            &Ph1mThreadDigestUpsertRequest::v1(
                now,
                speaker_ok(),
                policy_ok(),
                MemoryRetentionMode::RememberEverything,
                MemoryThreadDigest::v1(
                    "thread_low_use_newer".to_string(),
                    shared_title,
                    shared_summary,
                    false,
                    false,
                    MonotonicTimeNs(now.0.saturating_sub(ms_to_ns(newer_warm_age_ms))),
                    1,
                )
                .unwrap(),
                "idem_thread_low_use_newer".to_string(),
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
                    MemoryRetentionMode::RememberEverything,
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

        assert_eq!(
            out.selected_thread_id.as_deref(),
            Some("thread_high_use_older")
        );
        assert_eq!(out.resume_tier, Some(MemoryResumeTier::Warm));
        assert_eq!(out.resume_action, MemoryResumeAction::Suggest);
    }

    #[test]
    fn default_prefers_more_recent_warm_thread_over_use_count_for_resume() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let cfg = Ph1mConfig::mvp_v1();
        let older_warm_age_ms = cfg.resume_hot_window_ms.saturating_add(10_000);
        let newer_warm_age_ms = cfg.resume_hot_window_ms.saturating_add(1_000);
        assert!(older_warm_age_ms <= cfg.resume_warm_window_ms);
        assert!(newer_warm_age_ms <= cfg.resume_warm_window_ms);
        let now = MonotonicTimeNs(ms_to_ns(older_warm_age_ms).saturating_add(7_500_000_000));

        let shared_title = "Shared warm thread".to_string();
        let shared_summary = vec!["Shared summary".to_string()];

        rt.thread_digest_upsert(
            &Ph1mThreadDigestUpsertRequest::v1(
                now,
                speaker_ok(),
                policy_ok(),
                MemoryRetentionMode::Default,
                MemoryThreadDigest::v1(
                    "thread_high_use_older".to_string(),
                    shared_title.clone(),
                    shared_summary.clone(),
                    false,
                    false,
                    MonotonicTimeNs(now.0.saturating_sub(ms_to_ns(older_warm_age_ms))),
                    9,
                )
                .unwrap(),
                "idem_default_thread_high_use_older".to_string(),
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
                    "thread_low_use_newer".to_string(),
                    shared_title,
                    shared_summary,
                    false,
                    false,
                    MonotonicTimeNs(now.0.saturating_sub(ms_to_ns(newer_warm_age_ms))),
                    1,
                )
                .unwrap(),
                "idem_default_thread_low_use_newer".to_string(),
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

        assert_eq!(
            out.selected_thread_id.as_deref(),
            Some("thread_low_use_newer")
        );
        assert_eq!(out.resume_tier, Some(MemoryResumeTier::Warm));
        assert_eq!(out.resume_action, MemoryResumeAction::Suggest);
    }

    #[test]
    fn resume_select_prefers_actionable_warm_over_cold_without_topic() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let warm_delta_ns = ms_to_ns(MEMORY_RESUME_WARM_WINDOW_MS.saturating_sub(1));
        let cold_delta_ns = ms_to_ns(MEMORY_RESUME_WARM_WINDOW_MS.saturating_add(1));
        let now = MonotonicTimeNs(cold_delta_ns.saturating_add(8_000_000_000));

        rt.thread_digest_upsert(
            &Ph1mThreadDigestUpsertRequest::v1(
                now,
                speaker_ok(),
                policy_ok(),
                MemoryRetentionMode::Default,
                MemoryThreadDigest::v1(
                    "thread_cold_pinned".to_string(),
                    "Pinned but old".to_string(),
                    vec!["Old".to_string()],
                    true,
                    false,
                    MonotonicTimeNs(now.0.saturating_sub(cold_delta_ns)),
                    100,
                )
                .unwrap(),
                "idem_thread_cold".to_string(),
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
                    "thread_warm".to_string(),
                    "Warm and actionable".to_string(),
                    vec!["Need follow-up".to_string()],
                    false,
                    false,
                    MonotonicTimeNs(now.0.saturating_sub(warm_delta_ns)),
                    1,
                )
                .unwrap(),
                "idem_thread_warm".to_string(),
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

        assert_eq!(out.selected_thread_id.as_deref(), Some("thread_warm"));
        assert_eq!(out.resume_tier, Some(MemoryResumeTier::Warm));
        assert_eq!(out.resume_action, MemoryResumeAction::Suggest);
    }

    #[test]
    fn resume_select_unresolved_within_decay_window_breaks_warm_tie() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let cfg = Ph1mConfig::mvp_v1();
        assert!(cfg.resume_hot_window_ms < cfg.resume_warm_window_ms);
        assert!(cfg.resume_warm_window_ms < cfg.unresolved_decay_window_ms);

        let warm_age_ms = cfg.resume_hot_window_ms.saturating_add(1);
        assert!(warm_age_ms <= cfg.resume_warm_window_ms);
        assert!(warm_age_ms <= cfg.unresolved_decay_window_ms);

        let warm_age_ns = ms_to_ns(warm_age_ms);
        let now = MonotonicTimeNs(warm_age_ns.saturating_add(9_000_000_000));
        let last_updated = MonotonicTimeNs(now.0.saturating_sub(warm_age_ns));
        let shared_title = "Same warm thread".to_string();
        let shared_summary = vec!["Same summary".to_string()];

        rt.thread_digest_upsert(
            &Ph1mThreadDigestUpsertRequest::v1(
                now,
                speaker_ok(),
                policy_ok(),
                MemoryRetentionMode::Default,
                MemoryThreadDigest::v1(
                    "thread_alpha_resolved".to_string(),
                    shared_title.clone(),
                    shared_summary.clone(),
                    false,
                    false,
                    last_updated,
                    7,
                )
                .unwrap(),
                "idem_thread_alpha_resolved".to_string(),
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
                    "thread_unresolved_warm".to_string(),
                    shared_title,
                    shared_summary,
                    false,
                    true,
                    last_updated,
                    7,
                )
                .unwrap(),
                "idem_thread_unresolved_warm".to_string(),
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

        assert_eq!(
            out.selected_thread_id,
            Some("thread_unresolved_warm".to_string())
        );
        assert_eq!(out.resume_tier, Some(MemoryResumeTier::Warm));
        assert_eq!(out.resume_action, MemoryResumeAction::Suggest);
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
    fn context_bundle_high_confidence_emits_confirmed_tag() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let key = MemoryKey::new("project:active:jp_trip").unwrap();
        rt.propose(
            &Ph1mProposeRequest::v1(
                MonotonicTimeNs(10),
                speaker_ok(),
                policy_ok(),
                vec![propose_item_with_confidence(
                    key.as_str(),
                    "Japan Trip",
                    MemoryConfidence::High,
                )],
            )
            .unwrap(),
        )
        .unwrap();

        let out = rt
            .context_bundle_build(
                &Ph1mContextBundleBuildRequest::v1(
                    MonotonicTimeNs(11),
                    speaker_ok(),
                    policy_ok(),
                    vec![key.clone()],
                    vec![],
                    None,
                    None,
                    None,
                    true,
                    1024,
                    8,
                    0,
                )
                .unwrap(),
            )
            .unwrap();

        assert!(out.push_items.is_empty());
        assert_eq!(out.pull_items.len(), 1);
        let item = &out.pull_items[0];
        assert_eq!(item.memory_key, key);
        assert_eq!(item.tag, MemoryItemTag::Confirmed);
        assert_eq!(item.confidence, MemoryConfidence::High);
        assert_eq!(item.provenance_tier, MemoryProvenanceTier::UserStated);
    }

    #[test]
    fn context_bundle_confidence_ranking_prefers_high_over_low() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let high_key = MemoryKey::new("project:active:travel_priority").unwrap();
        let low_key = MemoryKey::new("project:active:travel_note").unwrap();
        rt.propose(
            &Ph1mProposeRequest::v1(
                MonotonicTimeNs(10),
                speaker_ok(),
                policy_ok(),
                vec![
                    propose_item_with_confidence(
                        high_key.as_str(),
                        "Book the flight first",
                        MemoryConfidence::High,
                    ),
                    propose_item_with_confidence(
                        low_key.as_str(),
                        "Maybe visit a museum",
                        MemoryConfidence::Low,
                    ),
                ],
            )
            .unwrap(),
        )
        .unwrap();

        let out = rt
            .context_bundle_build(
                &Ph1mContextBundleBuildRequest::v1(
                    MonotonicTimeNs(11),
                    speaker_ok(),
                    policy_ok(),
                    vec![high_key.clone(), low_key.clone()],
                    vec![],
                    None,
                    None,
                    None,
                    true,
                    1024,
                    8,
                    0,
                )
                .unwrap(),
            )
            .unwrap();

        assert!(out.push_items.is_empty());
        assert_eq!(out.pull_items.len(), 2);

        let high_index = out
            .pull_items
            .iter()
            .position(|item| item.memory_key == high_key)
            .expect("expected high-confidence pull item");
        let low_index = out
            .pull_items
            .iter()
            .position(|item| item.memory_key == low_key)
            .expect("expected low-confidence pull item");
        assert!(high_index < low_index);

        let high_item = &out.pull_items[high_index];
        let low_item = &out.pull_items[low_index];
        assert_eq!(high_item.tag, MemoryItemTag::Confirmed);
        assert_eq!(high_item.provenance_tier, MemoryProvenanceTier::UserStated);
        assert_eq!(low_item.tag, MemoryItemTag::Tentative);
        assert_eq!(low_item.provenance_tier, MemoryProvenanceTier::UserStated);
    }

    #[test]
    fn context_bundle_current_state_conflict_emits_conflict_tag() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let key = MemoryKey::new("project:active:trip_owner").unwrap();
        rt.propose(
            &Ph1mProposeRequest::v1(
                MonotonicTimeNs(10),
                speaker_ok(),
                policy_ok(),
                vec![propose_item_with_confidence(
                    key.as_str(),
                    "Selene",
                    MemoryConfidence::High,
                )],
            )
            .unwrap(),
        )
        .unwrap();

        let out = rt
            .context_bundle_build(
                &Ph1mContextBundleBuildRequest::v1(
                    MonotonicTimeNs(11),
                    speaker_ok(),
                    policy_ok(),
                    vec![key.clone()],
                    vec![MemoryContextFact::v1(
                        key.clone(),
                        MemoryValue::v1("Jordan".to_string(), None).unwrap(),
                    )
                    .unwrap()],
                    None,
                    None,
                    None,
                    true,
                    1024,
                    8,
                    0,
                )
                .unwrap(),
            )
            .unwrap();

        assert!(out.push_items.is_empty());
        assert_eq!(out.pull_items.len(), 1);
        let item = &out.pull_items[0];
        assert_eq!(item.memory_key, key);
        assert_eq!(item.tag, MemoryItemTag::Conflict);
        assert_eq!(item.confidence, MemoryConfidence::High);
        assert_eq!(item.provenance_tier, MemoryProvenanceTier::UserStated);
        assert_eq!(out.metric_payload.conflict_count, 1);
        assert_eq!(out.metric_payload.stale_count, 0);
    }

    #[test]
    fn context_bundle_conflict_metric_counts_only_conflicting_entries() {
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let conflict_key = MemoryKey::new("project:active:flight_step").unwrap();
        let aligned_key = MemoryKey::new("project:active:insurance_step").unwrap();
        rt.propose(
            &Ph1mProposeRequest::v1(
                MonotonicTimeNs(10),
                speaker_ok(),
                policy_ok(),
                vec![
                    propose_item_with_confidence(
                        conflict_key.as_str(),
                        "Book the flight",
                        MemoryConfidence::High,
                    ),
                    propose_item_with_confidence(
                        aligned_key.as_str(),
                        "Buy travel insurance",
                        MemoryConfidence::High,
                    ),
                ],
            )
            .unwrap(),
        )
        .unwrap();

        let out = rt
            .context_bundle_build(
                &Ph1mContextBundleBuildRequest::v1(
                    MonotonicTimeNs(11),
                    speaker_ok(),
                    policy_ok(),
                    vec![conflict_key.clone(), aligned_key.clone()],
                    vec![
                        MemoryContextFact::v1(
                            conflict_key.clone(),
                            MemoryValue::v1("Change the flight".to_string(), None).unwrap(),
                        )
                        .unwrap(),
                        MemoryContextFact::v1(
                            aligned_key.clone(),
                            MemoryValue::v1("Buy travel insurance".to_string(), None).unwrap(),
                        )
                        .unwrap(),
                    ],
                    None,
                    None,
                    None,
                    true,
                    1024,
                    8,
                    0,
                )
                .unwrap(),
            )
            .unwrap();

        assert!(out.push_items.is_empty());
        assert_eq!(out.pull_items.len(), 2);

        let conflict_item = out
            .pull_items
            .iter()
            .find(|item| item.memory_key == conflict_key)
            .expect("expected conflicting pull item");
        let aligned_item = out
            .pull_items
            .iter()
            .find(|item| item.memory_key == aligned_key)
            .expect("expected aligned pull item");
        assert_eq!(conflict_item.tag, MemoryItemTag::Conflict);
        assert_eq!(aligned_item.tag, MemoryItemTag::Confirmed);
        assert_eq!(out.metric_payload.conflict_count, 1);
        assert_eq!(out.metric_payload.stale_count, 0);
        assert_eq!(
            conflict_item.provenance_tier,
            MemoryProvenanceTier::UserStated
        );
        assert_eq!(
            aligned_item.provenance_tier,
            MemoryProvenanceTier::UserStated
        );
    }

    #[test]
    fn context_bundle_expired_micro_entry_emits_stale_tag() {
        let cfg = Ph1mConfig::mvp_v1();
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let key = MemoryKey::new("project:active:boarding_gate").unwrap();
        let propose_time = MonotonicTimeNs(10);
        let now = MonotonicTimeNs(
            propose_time
                .0
                .saturating_add(ms_to_ns(cfg.micro_ttl_ms))
                .saturating_add(1),
        );
        rt.propose(
            &Ph1mProposeRequest::v1(
                propose_time,
                speaker_ok(),
                policy_ok(),
                vec![propose_micro_item_with_confidence(
                    key.as_str(),
                    "Gate A12",
                    MemoryConfidence::High,
                )],
            )
            .unwrap(),
        )
        .unwrap();

        let out = rt
            .context_bundle_build(
                &Ph1mContextBundleBuildRequest::v1(
                    now,
                    speaker_ok(),
                    policy_ok(),
                    vec![key.clone()],
                    vec![],
                    None,
                    None,
                    None,
                    true,
                    1024,
                    8,
                    0,
                )
                .unwrap(),
            )
            .unwrap();

        assert!(out.push_items.is_empty());
        assert_eq!(out.pull_items.len(), 1);
        let item = &out.pull_items[0];
        assert_eq!(item.memory_key, key);
        assert_eq!(item.tag, MemoryItemTag::Stale);
        assert_eq!(item.confidence, MemoryConfidence::High);
        assert_eq!(item.provenance_tier, MemoryProvenanceTier::UserStated);
        assert_eq!(out.metric_payload.stale_count, 1);
        assert_eq!(out.metric_payload.conflict_count, 0);
    }

    #[test]
    fn context_bundle_stale_metric_counts_only_expired_entries() {
        let cfg = Ph1mConfig::mvp_v1();
        let mut rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let stale_key = MemoryKey::new("project:active:boarding_gate").unwrap();
        let confirmed_key = MemoryKey::new("project:active:hotel_booking").unwrap();
        let propose_time = MonotonicTimeNs(10);
        let now = MonotonicTimeNs(
            propose_time
                .0
                .saturating_add(ms_to_ns(cfg.micro_ttl_ms))
                .saturating_add(1),
        );
        rt.propose(
            &Ph1mProposeRequest::v1(
                propose_time,
                speaker_ok(),
                policy_ok(),
                vec![
                    propose_micro_item_with_confidence(
                        stale_key.as_str(),
                        "Gate A12",
                        MemoryConfidence::High,
                    ),
                    propose_item_with_confidence(
                        confirmed_key.as_str(),
                        "Hotel booked",
                        MemoryConfidence::High,
                    ),
                ],
            )
            .unwrap(),
        )
        .unwrap();

        let out = rt
            .context_bundle_build(
                &Ph1mContextBundleBuildRequest::v1(
                    now,
                    speaker_ok(),
                    policy_ok(),
                    vec![stale_key.clone(), confirmed_key.clone()],
                    vec![],
                    None,
                    None,
                    None,
                    true,
                    1024,
                    8,
                    0,
                )
                .unwrap(),
            )
            .unwrap();

        assert!(out.push_items.is_empty());
        assert_eq!(out.pull_items.len(), 2);

        let stale_item = out
            .pull_items
            .iter()
            .find(|item| item.memory_key == stale_key)
            .expect("expected expired pull item");
        let confirmed_item = out
            .pull_items
            .iter()
            .find(|item| item.memory_key == confirmed_key)
            .expect("expected non-expired pull item");
        assert_eq!(stale_item.tag, MemoryItemTag::Stale);
        assert_eq!(confirmed_item.tag, MemoryItemTag::Confirmed);
        assert_eq!(out.metric_payload.stale_count, 1);
        assert_eq!(out.metric_payload.conflict_count, 0);
        assert_eq!(stale_item.provenance_tier, MemoryProvenanceTier::UserStated);
        assert_eq!(
            confirmed_item.provenance_tier,
            MemoryProvenanceTier::UserStated
        );
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

    fn stage8_prior_new_york_time_turn() -> FreshMemoryPriorTurnEvidence {
        FreshMemoryPriorTurnEvidence {
            created_at: MonotonicTimeNs(8_000_000_000),
            source_session_id: Some(SessionId(8_001)),
            source_thread_key: Some("stage8-fresh-memory".to_string()),
            source_turn_ref: "conversation_turn:8001".to_string(),
            user_text: Some("What time is it in New York?".to_string()),
            response_text: Some("It's 5:00 AM in New York.".to_string()),
            tool_family: Some("time".to_string()),
            entity_focus: vec!["New York".to_string()],
            evidence_refs: vec![
                "internal_history_event:8001".to_string(),
                "ph1e_tool:time".to_string(),
            ],
        }
    }

    #[test]
    fn fresh_memory_continues_time_after_sleep_for_explicit_followup() {
        let rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let out = rt
            .fresh_memory_continuation(&FreshMemoryContinuationRequest {
                now: MonotonicTimeNs(40_000_000_000),
                current_text: "What about Sydney?".to_string(),
                current_thread_key: Some("stage8-fresh-memory".to_string()),
                current_turn_ref: "turn:8002".to_string(),
                evidence_ref_prefix: "8002:8002".to_string(),
                prior_turns: vec![stage8_prior_new_york_time_turn()],
            })
            .unwrap();

        assert_eq!(
            out.memory_continuation_decision.decision,
            MemoryContinuationDecisionKind::ContinueAutomatically
        );
        assert_eq!(
            out.rewritten_query.as_deref(),
            Some("what is the time in Sydney")
        );
        assert_eq!(out.tool_family.as_deref(), Some("time"));
        assert_eq!(out.current_entity_focus, vec!["Sydney".to_string()]);
        assert!(out.fresh_memory_handoff_ref.is_some());
        assert!(out.memory_evidence_packet_ref.is_some());
        assert!(out.memory_continuation_decision_ref.is_some());
        assert!(out.handoff.is_some());
        assert!(out.memory_evidence_packet.is_some());
    }

    #[test]
    fn fresh_memory_does_not_steal_unrelated_name_question() {
        let rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let out = rt
            .fresh_memory_continuation(&FreshMemoryContinuationRequest {
                now: MonotonicTimeNs(40_000_000_000),
                current_text: "What is your name?".to_string(),
                current_thread_key: Some("stage8-fresh-memory".to_string()),
                current_turn_ref: "turn:8003".to_string(),
                evidence_ref_prefix: "8003:8003".to_string(),
                prior_turns: vec![stage8_prior_new_york_time_turn()],
            })
            .unwrap();

        assert_eq!(
            out.memory_continuation_decision.decision,
            MemoryContinuationDecisionKind::AnswerNormally
        );
        assert!(out.rewritten_query.is_none());
        assert!(out.memory_evidence_packet_ref.is_none());
    }

    #[test]
    fn fresh_memory_bare_place_fragment_asks_clarification_not_auto_continue() {
        let rt = Ph1mRuntime::new(Ph1mConfig::mvp_v1());
        let out = rt
            .fresh_memory_continuation(&FreshMemoryContinuationRequest {
                now: MonotonicTimeNs(40_000_000_000),
                current_text: "Sydney".to_string(),
                current_thread_key: Some("stage8-fresh-memory".to_string()),
                current_turn_ref: "turn:8004".to_string(),
                evidence_ref_prefix: "8004:8004".to_string(),
                prior_turns: vec![stage8_prior_new_york_time_turn()],
            })
            .unwrap();

        assert_eq!(
            out.memory_continuation_decision.decision,
            MemoryContinuationDecisionKind::AskClarification
        );
        assert!(out.rewritten_query.is_none());
        assert!(out
            .memory_continuation_decision
            .clarification_prompt
            .as_deref()
            .unwrap_or_default()
            .contains("time question"));
    }
}
