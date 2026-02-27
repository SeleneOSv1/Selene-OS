#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1n::{
    AmbiguityFlag, Chat, Clarify, EvidenceSpan, FieldKey, FieldValue, IntentDraft, IntentField,
    IntentType, OverallConfidence, Ph1nRequest, Ph1nResponse, SensitivityLevel, TimeExpression,
    TimeExpressionKind, TranscriptHash, UncertainSpanKind,
};
use selene_kernel_contracts::{ContractViolation, SchemaVersion, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.NLP reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const N_INTENT_OK: ReasonCodeId = ReasonCodeId(0x4E00_0001);
    pub const N_CHAT_GREETING: ReasonCodeId = ReasonCodeId(0x4E00_0002);
    pub const N_CHAT_NO_INTENT: ReasonCodeId = ReasonCodeId(0x4E00_0003);
    pub const N_CLARIFY_TOO_LONG: ReasonCodeId = ReasonCodeId(0x4E00_0010);
    pub const N_CLARIFY_MISSING_FIELD: ReasonCodeId = ReasonCodeId(0x4E00_0011);
    pub const N_CLARIFY_AMBIGUOUS: ReasonCodeId = ReasonCodeId(0x4E00_0012);
    pub const N_CLARIFY_MULTI_INTENT: ReasonCodeId = ReasonCodeId(0x4E00_0013);
    pub const N_CLARIFY_UNCERTAIN_SPAN: ReasonCodeId = ReasonCodeId(0x4E00_0014);
}

const INTENT_SCHEMA_VERSION_V1: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1nConfig {
    pub max_transcript_len: usize,
}

impl Ph1nConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_transcript_len: 32_768,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1nRuntime {
    config: Ph1nConfig,
}

impl Ph1nRuntime {
    pub fn new(config: Ph1nConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1nRequest) -> Result<Ph1nResponse, ContractViolation> {
        req.validate()?;
        let transcript = &req.transcript_ok.transcript_text;
        if transcript.len() > self.config.max_transcript_len {
            return Ok(Ph1nResponse::Clarify(Clarify::v1(
                "That was a lot. Can you repeat that more briefly?".to_string(),
                vec![FieldKey::Task],
                vec![
                    "One short sentence".to_string(),
                    "A few keywords".to_string(),
                ],
                reason_codes::N_CLARIFY_TOO_LONG,
                SensitivityLevel::Public,
                false,
                vec![],
                vec![],
            )?));
        }

        // If PH1.C provided uncertain spans, fail closed into a targeted one-question clarify.
        if let Some(c) = clarify_for_uncertain_span(transcript, &req.uncertain_spans)? {
            return Ok(Ph1nResponse::Clarify(c));
        }

        let lower = transcript.to_ascii_lowercase();
        let stripped = strip_wake_prefix(&lower);

        if is_greeting(stripped) {
            return Ok(Ph1nResponse::Chat(Chat::v1(
                "Hello.".to_string(),
                reason_codes::N_CHAT_GREETING,
            )?));
        }

        // Deterministic intent classification (placeholder taxonomy).
        let intents = detect_intents(stripped);
        match intents.len() {
            0 => Ok(Ph1nResponse::Chat(Chat::v1(
                "Okay. What would you like to do?".to_string(),
                reason_codes::N_CHAT_NO_INTENT,
            )?)),
            1 => normalize_intent(req, intents[0]),
            _ => Ok(Ph1nResponse::Clarify(clarify_for_multi_intent(&intents)?)),
        }
    }
}

fn meta_for_intent(intent_type: IntentType) -> (SensitivityLevel, bool) {
    match intent_type {
        IntentType::SendMoney => (SensitivityLevel::Confidential, true),
        IntentType::MemoryForgetRequest => (SensitivityLevel::Private, true),
        IntentType::CreateInviteLink
        | IntentType::CapreqManage
        | IntentType::AccessSchemaManage
        | IntentType::AccessEscalationVote
        | IntentType::AccessInstanceCompileRefresh => (SensitivityLevel::Private, true),
        IntentType::MemoryRememberRequest | IntentType::MemoryQuery => {
            (SensitivityLevel::Private, false)
        }
        IntentType::SetReminder | IntentType::CreateCalendarEvent | IntentType::BookTable => {
            (SensitivityLevel::Private, true)
        }
        IntentType::TimeQuery
        | IntentType::WeatherQuery
        | IntentType::WebSearchQuery
        | IntentType::NewsQuery
        | IntentType::UrlFetchAndCiteQuery
        | IntentType::DocumentUnderstandQuery
        | IntentType::PhotoUnderstandQuery
        | IntentType::DataAnalysisQuery
        | IntentType::DeepResearchQuery
        | IntentType::RecordModeQuery => {
            (SensitivityLevel::Public, false)
        }
        IntentType::Continue | IntentType::MoreDetail => (SensitivityLevel::Public, false),
    }
}

fn max_sensitivity(a: SensitivityLevel, b: SensitivityLevel) -> SensitivityLevel {
    use SensitivityLevel::*;
    match (a, b) {
        (Confidential, _) | (_, Confidential) => Confidential,
        (Private, _) | (_, Private) => Private,
        _ => Public,
    }
}

fn meta_for_intents(intents: &[IntentType]) -> (SensitivityLevel, bool) {
    let mut sens = SensitivityLevel::Public;
    let mut confirm = false;
    for &t in intents {
        let (s, c) = meta_for_intent(t);
        sens = max_sensitivity(sens, s);
        confirm |= c;
    }
    (sens, confirm)
}

fn strip_wake_prefix(s: &str) -> &str {
    let s = s.trim_start();
    for prefix in ["selene", "hey selene", "yo selene"] {
        if let Some(rest) = s.strip_prefix(prefix) {
            return rest.trim_start_matches(|c: char| c == ',' || c == ':' || c.is_whitespace());
        }
    }
    s
}

fn is_greeting(s: &str) -> bool {
    let s = s.trim();
    matches!(
        s,
        "hi" | "hello" | "hey" | "yo" | "sup" | "what's up" | "whats up"
    )
}

fn looks_like_send_or_share_link(lower: &str) -> bool {
    let has_link = contains_word(lower, "link");
    let has_send_or_share = contains_word(lower, "send") || contains_word(lower, "share");
    has_link && has_send_or_share
}

fn looks_like_generate_link(lower: &str) -> bool {
    let has_link = contains_word(lower, "link");
    let has_generate = contains_word(lower, "generate")
        || contains_word(lower, "create")
        || contains_word(lower, "make");
    has_link && has_generate
}

fn looks_like_web_search(lower: &str) -> bool {
    (contains_word(lower, "search")
        && (contains_word(lower, "web")
            || contains_word(lower, "internet")
            || contains_word(lower, "online")))
        || lower.starts_with("search for ")
        || lower.starts_with("look up ")
        || lower.contains(" look up ")
        || lower.starts_with("find ")
        || lower.contains(" find ")
        || lower.starts_with("google ")
}

fn looks_like_news_query(lower: &str) -> bool {
    (contains_word(lower, "news")
        && (contains_word(lower, "latest")
            || contains_word(lower, "today")
            || contains_word(lower, "headlines")
            || contains_word(lower, "about")))
        || lower.starts_with("news about ")
        || lower.starts_with("latest news")
        || lower.starts_with("headlines")
}

fn looks_like_url_fetch_and_cite(lower: &str) -> bool {
    (contains_word(lower, "url")
        && (contains_word(lower, "open")
            || contains_word(lower, "fetch")
            || contains_word(lower, "cite")
            || contains_word(lower, "citation")))
        || (lower.contains("http://")
            || lower.contains("https://")
            || lower.contains("www."))
            && (contains_word(lower, "cite") || contains_word(lower, "citation"))
}

fn looks_like_document_understand(lower: &str) -> bool {
    (contains_word(lower, "pdf")
        || contains_word(lower, "document")
        || contains_word(lower, "doc")
        || contains_word(lower, "file"))
        && (contains_word(lower, "read")
            || contains_word(lower, "summarize")
            || contains_word(lower, "summary")
            || contains_word(lower, "extract")
            || contains_word(lower, "what does this"))
}

fn looks_like_photo_understand(lower: &str) -> bool {
    (contains_word(lower, "photo")
        || contains_word(lower, "image")
        || contains_word(lower, "screenshot")
        || contains_word(lower, "picture")
        || contains_word(lower, "chart"))
        && (contains_word(lower, "read")
            || contains_word(lower, "summarize")
            || contains_word(lower, "summary")
            || contains_word(lower, "extract")
            || contains_word(lower, "analyze")
            || contains_word(lower, "say")
            || contains_word(lower, "what does this"))
}

fn looks_like_data_analysis(lower: &str) -> bool {
    ((contains_word(lower, "analyze")
        || contains_word(lower, "analysis")
        || contains_word(lower, "calculate")
        || contains_word(lower, "stats")
        || contains_word(lower, "statistics")
        || contains_word(lower, "transform")
        || contains_word(lower, "chart")
        || contains_word(lower, "table"))
        && (contains_word(lower, "csv")
            || contains_word(lower, "spreadsheet")
            || contains_word(lower, "sheet")
            || contains_word(lower, "file")
            || contains_word(lower, "data")
            || contains_word(lower, "dataset")))
        || lower.starts_with("analyze this file")
        || lower.starts_with("analyze this csv")
        || lower.starts_with("analyze this spreadsheet")
}

fn looks_like_deep_research(lower: &str) -> bool {
    (contains_word(lower, "research")
        && (contains_word(lower, "deep")
            || contains_word(lower, "report")
            || contains_word(lower, "sources")
            || contains_word(lower, "synthesize")
            || contains_word(lower, "multi-source")
            || contains_word(lower, "multi source")))
        || lower.starts_with("deep research ")
        || lower.starts_with("research this topic")
        || lower.starts_with("research and summarize ")
}

fn looks_like_record_mode(lower: &str) -> bool {
    ((contains_word(lower, "recording")
        || contains_word(lower, "meeting")
        || contains_word(lower, "transcript")
        || contains_word(lower, "audio")
        || lower.starts_with("record mode "))
        && (contains_word(lower, "summarize")
            || contains_word(lower, "summary")
            || lower.contains("action item")
            || lower.contains("action items")
            || contains_word(lower, "minutes")
            || lower.contains("follow up")
            || lower.contains("follow-up")))
        || lower.starts_with("summarize this recording")
        || lower.starts_with("summarize this meeting")
}

fn detect_intents(lower: &str) -> Vec<IntentType> {
    let s = lower
        .trim()
        .trim_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace());
    let data_analysis = looks_like_data_analysis(s);
    let deep_research = looks_like_deep_research(s);
    let record_mode = looks_like_record_mode(s);
    let mut out: Vec<IntentType> = Vec::new();

    let mut push = |t: IntentType| {
        if !out.contains(&t) {
            out.push(t);
        }
    };

    // Conversation-control intents should win when the utterance is short and direct.
    if matches!(
        s,
        "continue" | "go on" | "keep going" | "carry on" | "resume" | "go ahead"
    ) {
        push(IntentType::Continue);
        return out;
    }
    if matches!(
        s,
        "more detail"
            | "more details"
            | "tell me more"
            | "elaborate"
            | "go deeper"
            | "expand on that"
            | "expand"
            | "explain more"
    ) {
        push(IntentType::MoreDetail);
        return out;
    }

    if s.contains("weather") {
        push(IntentType::WeatherQuery);
    }
    if s.contains("forget")
        || s.contains("delete memory")
        || s.contains("erase memory")
        || s.contains("remove memory")
    {
        push(IntentType::MemoryForgetRequest);
    }
    if s.contains("what do you remember")
        || s.contains("what did we say about")
        || s.contains("what did i say about")
        || s.contains("remember about")
        || s.contains("recall")
    {
        push(IntentType::MemoryQuery);
    }
    if (s.contains("remember ") || s == "remember")
        && !s.contains("what do you remember")
        && !s.contains("remember about")
    {
        push(IntentType::MemoryRememberRequest);
    }
    if s.contains("what time")
        || s == "time"
        || s.starts_with("time ")
        || s.contains("current time")
    {
        push(IntentType::TimeQuery);
    }
    if looks_like_web_search(s) {
        push(IntentType::WebSearchQuery);
    }
    if looks_like_news_query(s) {
        push(IntentType::NewsQuery);
    }
    if looks_like_url_fetch_and_cite(s) {
        push(IntentType::UrlFetchAndCiteQuery);
    }
    if looks_like_document_understand(s) {
        push(IntentType::DocumentUnderstandQuery);
    }
    if data_analysis {
        push(IntentType::DataAnalysisQuery);
    }
    if deep_research {
        push(IntentType::DeepResearchQuery);
    }
    if record_mode {
        push(IntentType::RecordModeQuery);
    }
    if looks_like_photo_understand(s) && !data_analysis {
        push(IntentType::PhotoUnderstandQuery);
    }
    if s.contains("remind me") || s.contains("reminder") {
        push(IntentType::SetReminder);
    }
    // Generic scheduling / defer-to-later phrasing is treated as "reminder-like" in the skeleton.
    if (s.contains("set ") && s.contains(" for ")) || s.contains(" later") {
        push(IntentType::SetReminder);
    }
    if (s.contains("meeting") || s.contains("schedule")) && !record_mode {
        push(IntentType::CreateCalendarEvent);
    }
    if s.contains("book a table") || s.contains("book table") {
        push(IntentType::BookTable);
    }
    if s.contains("send money") {
        push(IntentType::SendMoney);
    }
    if s.contains("invite link")
        || s.contains("onboarding link")
        || (s.contains("invite") && s.contains("link"))
        || (s.contains("onboard") && (s.contains("link") || s.contains("invite")))
        || s.contains("send an invite")
        || looks_like_send_or_share_link(s)
        || looks_like_generate_link(s)
    {
        push(IntentType::CreateInviteLink);
    }
    if s.contains("capreq")
        || s.contains("capability request")
        || s.contains("request capability")
        || s.contains("request access")
        || s.contains("access request")
    {
        push(IntentType::CapreqManage);
    }
    if s.contains("access schema")
        || s.contains("access profile")
        || s.contains("permission template")
        || s.contains("master access")
        || s.contains(" ap_")
    {
        push(IntentType::AccessSchemaManage);
    }
    if s.contains("escalation vote")
        || s.contains("access vote")
        || s.contains("vote on access")
        || s.contains("board vote")
    {
        push(IntentType::AccessEscalationVote);
    }
    if s.contains("compile access")
        || s.contains("refresh access instance")
        || s.contains("rebuild access instance")
        || s.contains("access compile refresh")
    {
        push(IntentType::AccessInstanceCompileRefresh);
    }
    out
}

fn normalize_intent(
    req: &Ph1nRequest,
    intent_type: IntentType,
) -> Result<Ph1nResponse, ContractViolation> {
    let t = &req.transcript_ok.transcript_text;

    match intent_type {
        IntentType::TimeQuery
        | IntentType::WeatherQuery
        | IntentType::WebSearchQuery
        | IntentType::NewsQuery
        | IntentType::UrlFetchAndCiteQuery
        | IntentType::DocumentUnderstandQuery
        | IntentType::PhotoUnderstandQuery
        | IntentType::DataAnalysisQuery
        | IntentType::DeepResearchQuery
        | IntentType::RecordModeQuery => {
            let (sens, confirm) = meta_for_intent(intent_type);
            Ok(Ph1nResponse::IntentDraft(IntentDraft::v1(
                intent_type,
                INTENT_SCHEMA_VERSION_V1,
                vec![],
                vec![],
                OverallConfidence::High,
                vec![evidence_span(FieldKey::Task, t, t)?],
                reason_codes::N_INTENT_OK,
                sens,
                confirm,
                vec![],
                vec![],
            )?))
        }

        IntentType::Continue | IntentType::MoreDetail => {
            let (sens, confirm) = meta_for_intent(intent_type);
            let mut missing = Vec::new();
            let mut flags = Vec::new();
            if req.confirmed_context.is_none() {
                missing.push(FieldKey::ReferenceTarget);
                flags.push(AmbiguityFlag::ReferenceAmbiguous);
            }
            Ok(Ph1nResponse::IntentDraft(IntentDraft::v1(
                intent_type,
                INTENT_SCHEMA_VERSION_V1,
                vec![],
                missing,
                OverallConfidence::High,
                vec![evidence_span(FieldKey::Task, t, t)?],
                reason_codes::N_INTENT_OK,
                sens,
                confirm,
                flags,
                vec![],
            )?))
        }

        IntentType::SetReminder => normalize_reminder(req),
        IntentType::CreateCalendarEvent => normalize_calendar_event(req),
        IntentType::BookTable => normalize_book_table(req),
        IntentType::SendMoney => normalize_send_money(req),
        IntentType::MemoryRememberRequest => normalize_memory_remember(req),
        IntentType::MemoryForgetRequest => normalize_memory_forget(req),
        IntentType::MemoryQuery => normalize_memory_query(req),
        IntentType::CreateInviteLink => normalize_create_invite_link(req),
        IntentType::CapreqManage => normalize_capreq_manage(req),
        IntentType::AccessSchemaManage
        | IntentType::AccessEscalationVote
        | IntentType::AccessInstanceCompileRefresh => normalize_access_control(req, intent_type),
    }
}

fn normalize_memory_remember(req: &Ph1nRequest) -> Result<Ph1nResponse, ContractViolation> {
    let t = &req.transcript_ok.transcript_text;
    let lower = t.to_ascii_lowercase();

    let subject = extract_memory_subject(
        &lower,
        t,
        &["remember this ", "remember that ", "remember "],
    );
    let mut fields = Vec::new();
    let mut evidence = Vec::new();
    let mut missing = Vec::new();
    let mut flags = Vec::new();

    if let Some(span) = subject {
        if req.confirmed_context.is_none() && is_reference_like(&span) {
            missing.push(FieldKey::ReferenceTarget);
            flags.push(AmbiguityFlag::ReferenceAmbiguous);
        } else {
            fields.push(IntentField {
                key: FieldKey::Task,
                value: FieldValue::verbatim(span.clone())?,
                confidence: OverallConfidence::High,
            });
            evidence.push(evidence_span(FieldKey::Task, t, &span)?);
        }
    } else if req.confirmed_context.is_none() && contains_reference_word(&lower) {
        missing.push(FieldKey::ReferenceTarget);
        flags.push(AmbiguityFlag::ReferenceAmbiguous);
    } else {
        missing.push(FieldKey::Task);
    }

    let (sens, confirm) = meta_for_intent(IntentType::MemoryRememberRequest);
    Ok(Ph1nResponse::IntentDraft(IntentDraft::v1(
        IntentType::MemoryRememberRequest,
        INTENT_SCHEMA_VERSION_V1,
        fields,
        missing,
        OverallConfidence::High,
        evidence,
        reason_codes::N_INTENT_OK,
        sens,
        confirm,
        flags,
        vec![],
    )?))
}

fn normalize_memory_forget(req: &Ph1nRequest) -> Result<Ph1nResponse, ContractViolation> {
    let t = &req.transcript_ok.transcript_text;
    let lower = t.to_ascii_lowercase();

    let subject = extract_memory_subject(
        &lower,
        t,
        &[
            "forget this ",
            "forget that ",
            "forget ",
            "delete memory ",
            "remove memory ",
        ],
    );
    let mut fields = Vec::new();
    let mut evidence = Vec::new();
    let mut missing = Vec::new();
    let mut flags = Vec::new();

    if let Some(span) = subject {
        if req.confirmed_context.is_none() && is_reference_like(&span) {
            missing.push(FieldKey::ReferenceTarget);
            flags.push(AmbiguityFlag::ReferenceAmbiguous);
        } else {
            fields.push(IntentField {
                key: FieldKey::Task,
                value: FieldValue::verbatim(span.clone())?,
                confidence: OverallConfidence::High,
            });
            evidence.push(evidence_span(FieldKey::Task, t, &span)?);
        }
    } else if req.confirmed_context.is_none() && contains_reference_word(&lower) {
        missing.push(FieldKey::ReferenceTarget);
        flags.push(AmbiguityFlag::ReferenceAmbiguous);
    } else {
        missing.push(FieldKey::Task);
    }

    let (sens, confirm) = meta_for_intent(IntentType::MemoryForgetRequest);
    Ok(Ph1nResponse::IntentDraft(IntentDraft::v1(
        IntentType::MemoryForgetRequest,
        INTENT_SCHEMA_VERSION_V1,
        fields,
        missing,
        OverallConfidence::High,
        evidence,
        reason_codes::N_INTENT_OK,
        sens,
        confirm,
        flags,
        vec![],
    )?))
}

fn normalize_memory_query(req: &Ph1nRequest) -> Result<Ph1nResponse, ContractViolation> {
    let t = &req.transcript_ok.transcript_text;
    let lower = t.to_ascii_lowercase();

    let subject = extract_memory_subject(
        &lower,
        t,
        &[
            "what do you remember about ",
            "what did we say about ",
            "what did i say about ",
            "remember about ",
            "recall ",
        ],
    );
    let mut fields = Vec::new();
    let mut evidence = Vec::new();
    let mut missing = Vec::new();
    let mut flags = Vec::new();

    if let Some(span) = subject {
        if req.confirmed_context.is_none() && is_reference_like(&span) {
            missing.push(FieldKey::ReferenceTarget);
            flags.push(AmbiguityFlag::ReferenceAmbiguous);
        } else {
            fields.push(IntentField {
                key: FieldKey::Task,
                value: FieldValue::verbatim(span.clone())?,
                confidence: OverallConfidence::High,
            });
            evidence.push(evidence_span(FieldKey::Task, t, &span)?);
        }
    } else if req.confirmed_context.is_none() && contains_reference_word(&lower) {
        missing.push(FieldKey::ReferenceTarget);
        flags.push(AmbiguityFlag::ReferenceAmbiguous);
    } else {
        evidence.push(evidence_span(FieldKey::Task, t, t)?);
    }

    let (sens, confirm) = meta_for_intent(IntentType::MemoryQuery);
    Ok(Ph1nResponse::IntentDraft(IntentDraft::v1(
        IntentType::MemoryQuery,
        INTENT_SCHEMA_VERSION_V1,
        fields,
        missing,
        OverallConfidence::High,
        evidence,
        reason_codes::N_INTENT_OK,
        sens,
        confirm,
        flags,
        vec![],
    )?))
}

fn normalize_reminder(req: &Ph1nRequest) -> Result<Ph1nResponse, ContractViolation> {
    let t = &req.transcript_ok.transcript_text;
    let lower = t.to_ascii_lowercase();

    let (task_span, task_norm) = extract_task_after(&lower, t, "remind me to ")
        .or_else(|| extract_task_after(&lower, t, "remind me "))
        .unwrap_or((None, None));

    let when = extract_when_span(t);

    let mut fields = Vec::new();
    let mut evidence = Vec::new();
    let mut missing = Vec::new();

    if let Some(span) = task_span {
        if req.confirmed_context.is_none() && is_reference_like(&span) {
            missing.push(FieldKey::ReferenceTarget);
        } else {
            fields.push(IntentField {
                key: FieldKey::Task,
                value: FieldValue::verbatim(span.clone())?,
                confidence: OverallConfidence::High,
            });
            evidence.push(evidence_span(FieldKey::Task, t, &span)?);
        }
    } else {
        if req.confirmed_context.is_none() && contains_reference_word(&lower) {
            missing.push(FieldKey::ReferenceTarget);
        } else {
            missing.push(FieldKey::Task);
        }
    }

    if let Some((orig, norm)) = when {
        let value = if let Some(n) = norm {
            FieldValue::time(orig.clone(), n)?
        } else {
            FieldValue::verbatim(orig.clone())?
        };
        fields.push(IntentField {
            key: FieldKey::When,
            value,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::When, t, &orig)?);
    } else {
        missing.push(FieldKey::When);
    }

    if !missing.is_empty() {
        return Ok(Ph1nResponse::Clarify(clarify_for_missing(
            intent_type_for_missing(IntentType::SetReminder),
            &missing,
        )?));
    }

    let _ = task_norm; // reserved for future canonicalization.
    let (sens, confirm) = meta_for_intent(IntentType::SetReminder);
    Ok(Ph1nResponse::IntentDraft(IntentDraft::v1(
        IntentType::SetReminder,
        INTENT_SCHEMA_VERSION_V1,
        fields,
        vec![],
        OverallConfidence::High,
        evidence,
        reason_codes::N_INTENT_OK,
        sens,
        confirm,
        vec![],
        vec![],
    )?))
}

fn normalize_calendar_event(req: &Ph1nRequest) -> Result<Ph1nResponse, ContractViolation> {
    let t = &req.transcript_ok.transcript_text;
    let lower = t.to_ascii_lowercase();

    let when = extract_when_span(t);
    let person = extract_simple_person(&lower, t);

    let mut fields = Vec::new();
    let mut evidence = Vec::new();
    let mut missing = Vec::new();

    if let Some((orig, norm)) = when {
        let value = if let Some(n) = norm {
            FieldValue::time(orig.clone(), n)?
        } else {
            FieldValue::verbatim(orig.clone())?
        };
        fields.push(IntentField {
            key: FieldKey::When,
            value,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::When, t, &orig)?);
    } else {
        missing.push(FieldKey::When);
    }

    if let Some(p) = person {
        fields.push(IntentField {
            key: FieldKey::Person,
            value: FieldValue::verbatim(p.clone())?,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::Person, t, &p)?);
    }

    if !missing.is_empty() {
        return Ok(Ph1nResponse::Clarify(clarify_for_missing(
            intent_type_for_missing(IntentType::CreateCalendarEvent),
            &missing,
        )?));
    }

    let (sens, confirm) = meta_for_intent(IntentType::CreateCalendarEvent);
    Ok(Ph1nResponse::IntentDraft(IntentDraft::v1(
        IntentType::CreateCalendarEvent,
        INTENT_SCHEMA_VERSION_V1,
        fields,
        vec![],
        OverallConfidence::High,
        evidence,
        reason_codes::N_INTENT_OK,
        sens,
        confirm,
        vec![],
        vec![],
    )?))
}

fn normalize_book_table(req: &Ph1nRequest) -> Result<Ph1nResponse, ContractViolation> {
    let t = &req.transcript_ok.transcript_text;
    let lower = t.to_ascii_lowercase();

    let when = extract_when_span(t);
    let place = extract_place_after_at(&lower, t);
    let party_size = extract_party_size(&lower, t);

    let mut fields = Vec::new();
    let mut evidence = Vec::new();
    let mut missing = Vec::new();

    if let Some((orig, norm)) = when {
        let value = if let Some(n) = norm {
            FieldValue::time(orig.clone(), n)?
        } else {
            FieldValue::verbatim(orig.clone())?
        };
        fields.push(IntentField {
            key: FieldKey::When,
            value,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::When, t, &orig)?);
    } else {
        missing.push(FieldKey::When);
    }

    if let Some(p) = place {
        if req.confirmed_context.is_none() && is_reference_like(&p) {
            missing.push(FieldKey::ReferenceTarget);
        } else {
            fields.push(IntentField {
                key: FieldKey::Place,
                value: FieldValue::verbatim(p.clone())?,
                confidence: OverallConfidence::High,
            });
            evidence.push(evidence_span(FieldKey::Place, t, &p)?);
        }
    } else {
        if req.confirmed_context.is_none() && contains_reference_word(&lower) {
            missing.push(FieldKey::ReferenceTarget);
        } else {
            missing.push(FieldKey::Place);
        }
    }

    if let Some((orig, norm)) = party_size {
        let value = if let Some(n) = norm {
            FieldValue::normalized(orig.clone(), n)?
        } else {
            FieldValue::verbatim(orig.clone())?
        };
        fields.push(IntentField {
            key: FieldKey::PartySize,
            value,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::PartySize, t, &orig)?);
    } else {
        missing.push(FieldKey::PartySize);
    }

    if !missing.is_empty() {
        return Ok(Ph1nResponse::Clarify(clarify_for_missing(
            intent_type_for_missing(IntentType::BookTable),
            &missing,
        )?));
    }

    let (sens, confirm) = meta_for_intent(IntentType::BookTable);
    Ok(Ph1nResponse::IntentDraft(IntentDraft::v1(
        IntentType::BookTable,
        INTENT_SCHEMA_VERSION_V1,
        fields,
        vec![],
        OverallConfidence::High,
        evidence,
        reason_codes::N_INTENT_OK,
        sens,
        confirm,
        vec![],
        vec![],
    )?))
}

fn normalize_send_money(req: &Ph1nRequest) -> Result<Ph1nResponse, ContractViolation> {
    let t = &req.transcript_ok.transcript_text;
    let lower = t.to_ascii_lowercase();

    let recipient = extract_recipient_after_to(&lower, t);
    let amount = extract_amount(&lower, t);

    let mut fields = Vec::new();
    let mut evidence = Vec::new();
    let mut missing = Vec::new();

    if let Some(r) = recipient {
        fields.push(IntentField {
            key: FieldKey::Recipient,
            value: FieldValue::verbatim(r.clone())?,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::Recipient, t, &r)?);
    } else {
        missing.push(FieldKey::Recipient);
    }

    if let Some((orig, norm)) = amount {
        let value = if let Some(n) = norm {
            FieldValue::normalized(orig.clone(), n)?
        } else {
            FieldValue::verbatim(orig.clone())?
        };
        fields.push(IntentField {
            key: FieldKey::Amount,
            value,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::Amount, t, &orig)?);
    } else {
        missing.push(FieldKey::Amount);
    }

    if !missing.is_empty() {
        return Ok(Ph1nResponse::Clarify(clarify_for_missing(
            intent_type_for_missing(IntentType::SendMoney),
            &missing,
        )?));
    }

    let (sens, confirm) = meta_for_intent(IntentType::SendMoney);
    Ok(Ph1nResponse::IntentDraft(IntentDraft::v1(
        IntentType::SendMoney,
        INTENT_SCHEMA_VERSION_V1,
        fields,
        vec![],
        OverallConfidence::High,
        evidence,
        reason_codes::N_INTENT_OK,
        sens,
        confirm,
        vec![],
        vec![],
    )?))
}

fn normalize_create_invite_link(req: &Ph1nRequest) -> Result<Ph1nResponse, ContractViolation> {
    let t = &req.transcript_ok.transcript_text;
    let lower = t.to_ascii_lowercase();
    let delivery_requested = looks_like_send_or_share_link(&lower);

    let invitee_type = extract_invitee_type(&lower, t).or_else(|| {
        let fallback_span = excerpt_from_lower_match(&lower, t, "link")
            .or_else(|| excerpt_from_lower_match(&lower, t, "invite"))
            .unwrap_or_else(|| t.to_string());
        Some((fallback_span, "associate".to_string()))
    });
    let delivery_method = extract_delivery_method(&lower, t).or_else(|| {
        if delivery_requested {
            let fallback_span = excerpt_from_lower_match(&lower, t, "send")
                .or_else(|| excerpt_from_lower_match(&lower, t, "share"))
                .unwrap_or_else(|| "send".to_string());
            Some((fallback_span, "selene_app".to_string()))
        } else {
            None
        }
    });
    let recipient_contact = extract_recipient_contact(t);
    let recipient_name = extract_recipient_after_to(&lower, t);
    let runtime_tenant_id = req
        .runtime_tenant_id
        .as_ref()
        .map(|tenant| tenant.trim().to_string())
        .filter(|tenant| !tenant.is_empty());
    let transcript_tenant_hint = extract_tenant_id(&lower, t);
    let tenant_id = runtime_tenant_id.clone();

    let mut fields = Vec::new();
    let mut evidence = Vec::new();
    let mut missing = Vec::new();

    if let Some((orig, norm)) = invitee_type {
        fields.push(IntentField {
            key: FieldKey::InviteeType,
            value: FieldValue::normalized(orig.clone(), norm)?,
            confidence: OverallConfidence::High,
        });
        if excerpt_from_lower_match(&lower, t, &orig.to_ascii_lowercase()).is_some() {
            evidence.push(evidence_span(FieldKey::InviteeType, t, &orig)?);
        }
    }

    if let Some((orig, norm)) = delivery_method {
        fields.push(IntentField {
            key: FieldKey::DeliveryMethod,
            value: FieldValue::normalized(orig.clone(), norm)?,
            confidence: OverallConfidence::High,
        });
        if excerpt_from_lower_match(&lower, t, &orig.to_ascii_lowercase()).is_some() {
            evidence.push(evidence_span(FieldKey::DeliveryMethod, t, &orig)?);
        }
    }

    if let Some(c) = recipient_contact {
        fields.push(IntentField {
            key: FieldKey::RecipientContact,
            value: FieldValue::verbatim(c.clone())?,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::RecipientContact, t, &c)?);
    } else {
        if let Some(name) = recipient_name {
            fields.push(IntentField {
                key: FieldKey::Recipient,
                value: FieldValue::verbatim(name.clone())?,
                confidence: OverallConfidence::High,
            });
            evidence.push(evidence_span(FieldKey::Recipient, t, &name)?);
            if delivery_requested {
                missing.push(FieldKey::RecipientContact);
            }
        } else if delivery_requested {
            missing.push(FieldKey::Recipient);
        }
    }

    if let Some(tenant) = tenant_id {
        fields.push(IntentField {
            key: FieldKey::TenantId,
            value: FieldValue::verbatim(tenant.clone())?,
            confidence: OverallConfidence::High,
        });
        if transcript_tenant_hint.as_deref() == Some(tenant.as_str()) {
            evidence.push(evidence_span(FieldKey::TenantId, t, &tenant)?);
        }
    }

    let (sens, confirm) = meta_for_intent(IntentType::CreateInviteLink);
    Ok(Ph1nResponse::IntentDraft(IntentDraft::v1(
        IntentType::CreateInviteLink,
        INTENT_SCHEMA_VERSION_V1,
        fields,
        missing,
        OverallConfidence::High,
        evidence,
        reason_codes::N_INTENT_OK,
        sens,
        confirm,
        vec![],
        vec![],
    )?))
}

fn normalize_capreq_manage(req: &Ph1nRequest) -> Result<Ph1nResponse, ContractViolation> {
    let t = &req.transcript_ok.transcript_text;
    let lower = t.to_ascii_lowercase();

    let action = detect_capreq_action(&lower);
    let requested_capability = extract_requested_capability_id(&lower, t);
    let target_scope = extract_target_scope_ref(&lower, t);
    let justification = extract_justification(&lower, t);
    let tenant_id = extract_tenant_id(&lower, t);
    let capreq_id = extract_capreq_id(&lower, t);

    let mut fields = Vec::new();
    let mut evidence = Vec::new();
    let mut missing = Vec::new();

    fields.push(IntentField {
        key: FieldKey::CapreqAction,
        value: FieldValue::normalized(action.to_string(), action.to_string())?,
        confidence: OverallConfidence::High,
    });
    evidence.push(evidence_span(FieldKey::CapreqAction, t, t)?);

    if let Some(tenant) = tenant_id {
        fields.push(IntentField {
            key: FieldKey::TenantId,
            value: FieldValue::verbatim(tenant.clone())?,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::TenantId, t, &tenant)?);
    } else {
        missing.push(FieldKey::TenantId);
    }

    let snapshot_complete =
        requested_capability.is_some() && target_scope.is_some() && justification.is_some();

    match action {
        "create_draft" => {
            if let Some(cap) = requested_capability {
                fields.push(IntentField {
                    key: FieldKey::RequestedCapabilityId,
                    value: FieldValue::verbatim(cap.clone())?,
                    confidence: OverallConfidence::High,
                });
                evidence.push(evidence_span(FieldKey::RequestedCapabilityId, t, &cap)?);
            } else {
                missing.push(FieldKey::RequestedCapabilityId);
            }

            if let Some(scope) = target_scope {
                fields.push(IntentField {
                    key: FieldKey::TargetScopeRef,
                    value: FieldValue::verbatim(scope.clone())?,
                    confidence: OverallConfidence::High,
                });
                evidence.push(evidence_span(FieldKey::TargetScopeRef, t, &scope)?);
            } else {
                missing.push(FieldKey::TargetScopeRef);
            }

            if let Some(just) = justification {
                fields.push(IntentField {
                    key: FieldKey::Justification,
                    value: FieldValue::verbatim(just.clone())?,
                    confidence: OverallConfidence::High,
                });
                evidence.push(evidence_span(FieldKey::Justification, t, &just)?);
            } else {
                missing.push(FieldKey::Justification);
            }

            if let Some(existing_id) = capreq_id {
                fields.push(IntentField {
                    key: FieldKey::CapreqId,
                    value: FieldValue::verbatim(existing_id.clone())?,
                    confidence: OverallConfidence::High,
                });
                evidence.push(evidence_span(FieldKey::CapreqId, t, &existing_id)?);
            }
        }
        "submit_for_approval" => {
            if let Some(existing_id) = capreq_id {
                fields.push(IntentField {
                    key: FieldKey::CapreqId,
                    value: FieldValue::verbatim(existing_id.clone())?,
                    confidence: OverallConfidence::High,
                });
                evidence.push(evidence_span(FieldKey::CapreqId, t, &existing_id)?);
            } else if !snapshot_complete {
                missing.push(FieldKey::CapreqId);
            }

            if let Some(cap) = requested_capability {
                fields.push(IntentField {
                    key: FieldKey::RequestedCapabilityId,
                    value: FieldValue::verbatim(cap.clone())?,
                    confidence: OverallConfidence::High,
                });
                evidence.push(evidence_span(FieldKey::RequestedCapabilityId, t, &cap)?);
            }
            if let Some(scope) = target_scope {
                fields.push(IntentField {
                    key: FieldKey::TargetScopeRef,
                    value: FieldValue::verbatim(scope.clone())?,
                    confidence: OverallConfidence::High,
                });
                evidence.push(evidence_span(FieldKey::TargetScopeRef, t, &scope)?);
            }
            if let Some(just) = justification {
                fields.push(IntentField {
                    key: FieldKey::Justification,
                    value: FieldValue::verbatim(just.clone())?,
                    confidence: OverallConfidence::High,
                });
                evidence.push(evidence_span(FieldKey::Justification, t, &just)?);
            }
        }
        _ => {
            if let Some(existing_id) = capreq_id {
                fields.push(IntentField {
                    key: FieldKey::CapreqId,
                    value: FieldValue::verbatim(existing_id.clone())?,
                    confidence: OverallConfidence::High,
                });
                evidence.push(evidence_span(FieldKey::CapreqId, t, &existing_id)?);
            } else {
                missing.push(FieldKey::CapreqId);
            }
        }
    }

    if !missing.is_empty() {
        return Ok(Ph1nResponse::Clarify(clarify_for_missing(
            intent_type_for_missing(IntentType::CapreqManage),
            &missing,
        )?));
    }

    let (sens, confirm) = meta_for_intent(IntentType::CapreqManage);
    Ok(Ph1nResponse::IntentDraft(IntentDraft::v1(
        IntentType::CapreqManage,
        INTENT_SCHEMA_VERSION_V1,
        fields,
        vec![],
        OverallConfidence::High,
        evidence,
        reason_codes::N_INTENT_OK,
        sens,
        confirm,
        vec![],
        vec![],
    )?))
}

fn normalize_access_control(
    req: &Ph1nRequest,
    intent_type: IntentType,
) -> Result<Ph1nResponse, ContractViolation> {
    match intent_type {
        IntentType::AccessSchemaManage => normalize_access_schema_manage(req),
        IntentType::AccessEscalationVote => normalize_access_escalation_vote(req),
        IntentType::AccessInstanceCompileRefresh => normalize_access_instance_compile_refresh(req),
        _ => Err(ContractViolation::InvalidValue {
            field: "intent_type",
            reason: "unsupported access intent",
        }),
    }
}

fn normalize_access_schema_manage(req: &Ph1nRequest) -> Result<Ph1nResponse, ContractViolation> {
    let t = &req.transcript_ok.transcript_text;
    let lower = t.to_ascii_lowercase();

    let mut fields = Vec::new();
    let mut evidence = Vec::new();
    let mut missing = Vec::new();

    let action = detect_access_schema_action(&lower);
    fields.push(IntentField {
        key: FieldKey::ApAction,
        value: FieldValue::normalized(action.to_string(), action.to_string())?,
        confidence: OverallConfidence::High,
    });
    evidence.push(evidence_span(FieldKey::ApAction, t, t)?);

    let scope = detect_access_scope(&lower);
    fields.push(IntentField {
        key: FieldKey::ApScope,
        value: FieldValue::normalized(scope.to_string(), scope.to_string())?,
        confidence: OverallConfidence::High,
    });
    evidence.push(evidence_span(FieldKey::ApScope, t, t)?);

    if let Some(profile_id) = extract_access_profile_id(&lower, t) {
        fields.push(IntentField {
            key: FieldKey::AccessProfileId,
            value: FieldValue::verbatim(profile_id.clone())?,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::AccessProfileId, t, &profile_id)?);
    } else {
        missing.push(FieldKey::AccessProfileId);
    }

    if let Some(version_id) = extract_schema_version_id(&lower, t) {
        fields.push(IntentField {
            key: FieldKey::SchemaVersionId,
            value: FieldValue::verbatim(version_id.clone())?,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::SchemaVersionId, t, &version_id)?);
    } else {
        missing.push(FieldKey::SchemaVersionId);
    }

    if scope == "TENANT" {
        if let Some(tenant_id) = extract_tenant_id(&lower, t) {
            fields.push(IntentField {
                key: FieldKey::TenantId,
                value: FieldValue::verbatim(tenant_id.clone())?,
                confidence: OverallConfidence::High,
            });
            evidence.push(evidence_span(FieldKey::TenantId, t, &tenant_id)?);
        } else {
            missing.push(FieldKey::TenantId);
        }
    }

    if let Some((orig, norm)) = detect_access_review_channel(&lower, t) {
        fields.push(IntentField {
            key: FieldKey::AccessReviewChannel,
            value: FieldValue::normalized(orig.clone(), norm.clone())?,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::AccessReviewChannel, t, &orig)?);
    } else {
        missing.push(FieldKey::AccessReviewChannel);
    }

    if let Some((orig, norm)) = detect_access_rule_action(&lower, t) {
        fields.push(IntentField {
            key: FieldKey::AccessRuleAction,
            value: FieldValue::normalized(orig.clone(), norm.clone())?,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::AccessRuleAction, t, &orig)?);
    } else if action == "ACTIVATE" {
        // Activation requires explicit rule-review confirmation state.
        missing.push(FieldKey::AccessRuleAction);
    }

    if action == "CREATE_DRAFT" || action == "UPDATE" {
        if let Some(payload) = extract_profile_payload_json(t) {
            fields.push(IntentField {
                key: FieldKey::ProfilePayloadJson,
                value: FieldValue::verbatim(payload.clone())?,
                confidence: OverallConfidence::High,
            });
            evidence.push(evidence_span(FieldKey::ProfilePayloadJson, t, &payload)?);
        }
    }

    if !missing.is_empty() {
        return Ok(Ph1nResponse::Clarify(clarify_for_missing(
            intent_type_for_missing(IntentType::AccessSchemaManage),
            &missing,
        )?));
    }

    let (sens, confirm) = meta_for_intent(IntentType::AccessSchemaManage);
    Ok(Ph1nResponse::IntentDraft(IntentDraft::v1(
        IntentType::AccessSchemaManage,
        INTENT_SCHEMA_VERSION_V1,
        fields,
        vec![],
        OverallConfidence::High,
        evidence,
        reason_codes::N_INTENT_OK,
        sens,
        confirm,
        vec![],
        vec![],
    )?))
}

fn normalize_access_escalation_vote(req: &Ph1nRequest) -> Result<Ph1nResponse, ContractViolation> {
    let t = &req.transcript_ok.transcript_text;
    let lower = t.to_ascii_lowercase();

    let mut fields = Vec::new();
    let mut evidence = Vec::new();
    let mut missing = Vec::new();

    if let Some(tenant_id) = extract_tenant_id(&lower, t) {
        fields.push(IntentField {
            key: FieldKey::TenantId,
            value: FieldValue::verbatim(tenant_id.clone())?,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::TenantId, t, &tenant_id)?);
    } else {
        missing.push(FieldKey::TenantId);
    }

    let vote_action = detect_vote_action(&lower);
    fields.push(IntentField {
        key: FieldKey::VoteAction,
        value: FieldValue::normalized(vote_action.to_string(), vote_action.to_string())?,
        confidence: OverallConfidence::High,
    });
    evidence.push(evidence_span(FieldKey::VoteAction, t, t)?);

    let vote_value = detect_vote_value(&lower);
    fields.push(IntentField {
        key: FieldKey::VoteValue,
        value: FieldValue::normalized(vote_value.to_string(), vote_value.to_string())?,
        confidence: OverallConfidence::High,
    });
    evidence.push(evidence_span(FieldKey::VoteValue, t, t)?);

    if let Some(case_id) = extract_escalation_case_id(&lower, t) {
        fields.push(IntentField {
            key: FieldKey::EscalationCaseId,
            value: FieldValue::verbatim(case_id.clone())?,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::EscalationCaseId, t, &case_id)?);
    } else {
        missing.push(FieldKey::EscalationCaseId);
    }

    if let Some(board_policy_id) = extract_board_policy_id(&lower, t) {
        fields.push(IntentField {
            key: FieldKey::BoardPolicyId,
            value: FieldValue::verbatim(board_policy_id.clone())?,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::BoardPolicyId, t, &board_policy_id)?);
    }

    if !missing.is_empty() {
        return Ok(Ph1nResponse::Clarify(clarify_for_missing(
            intent_type_for_missing(IntentType::AccessEscalationVote),
            &missing,
        )?));
    }

    let (sens, confirm) = meta_for_intent(IntentType::AccessEscalationVote);
    Ok(Ph1nResponse::IntentDraft(IntentDraft::v1(
        IntentType::AccessEscalationVote,
        INTENT_SCHEMA_VERSION_V1,
        fields,
        vec![],
        OverallConfidence::High,
        evidence,
        reason_codes::N_INTENT_OK,
        sens,
        confirm,
        vec![],
        vec![],
    )?))
}

fn normalize_access_instance_compile_refresh(
    req: &Ph1nRequest,
) -> Result<Ph1nResponse, ContractViolation> {
    let t = &req.transcript_ok.transcript_text;
    let lower = t.to_ascii_lowercase();

    let mut fields = Vec::new();
    let mut evidence = Vec::new();
    let mut missing = Vec::new();

    if let Some(tenant_id) = extract_tenant_id(&lower, t) {
        fields.push(IntentField {
            key: FieldKey::TenantId,
            value: FieldValue::verbatim(tenant_id.clone())?,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::TenantId, t, &tenant_id)?);
    } else {
        missing.push(FieldKey::TenantId);
    }

    if let Some(target_user_id) = extract_target_user_id(&lower, t) {
        fields.push(IntentField {
            key: FieldKey::TargetUserId,
            value: FieldValue::verbatim(target_user_id.clone())?,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::TargetUserId, t, &target_user_id)?);
    } else {
        missing.push(FieldKey::TargetUserId);
    }

    if let Some(profile_id) = extract_access_profile_id(&lower, t) {
        fields.push(IntentField {
            key: FieldKey::AccessProfileId,
            value: FieldValue::verbatim(profile_id.clone())?,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::AccessProfileId, t, &profile_id)?);
    }

    if let Some(position_id) = extract_position_id(&lower, t) {
        fields.push(IntentField {
            key: FieldKey::PositionId,
            value: FieldValue::verbatim(position_id.clone())?,
            confidence: OverallConfidence::High,
        });
        evidence.push(evidence_span(FieldKey::PositionId, t, &position_id)?);
    }

    let compile_reason = detect_compile_reason(&lower);
    fields.push(IntentField {
        key: FieldKey::CompileReason,
        value: FieldValue::normalized(compile_reason.to_string(), compile_reason.to_string())?,
        confidence: OverallConfidence::High,
    });
    evidence.push(evidence_span(FieldKey::CompileReason, t, t)?);

    if !missing.is_empty() {
        return Ok(Ph1nResponse::Clarify(clarify_for_missing(
            intent_type_for_missing(IntentType::AccessInstanceCompileRefresh),
            &missing,
        )?));
    }

    let (sens, confirm) = meta_for_intent(IntentType::AccessInstanceCompileRefresh);
    Ok(Ph1nResponse::IntentDraft(IntentDraft::v1(
        IntentType::AccessInstanceCompileRefresh,
        INTENT_SCHEMA_VERSION_V1,
        fields,
        vec![],
        OverallConfidence::High,
        evidence,
        reason_codes::N_INTENT_OK,
        sens,
        confirm,
        vec![],
        vec![],
    )?))
}

fn intent_type_for_missing(intent: IntentType) -> IntentType {
    intent
}

fn clarify_for_missing(
    intent_type: IntentType,
    missing: &[FieldKey],
) -> Result<Clarify, ContractViolation> {
    // Deterministic "one question" policy: ask for the highest-priority missing field only.
    let primary = select_primary_missing(missing);
    let (question, formats) = match (intent_type, primary) {
        (_, FieldKey::IntentChoice) => (
            "Which one should I do first?".to_string(),
            vec!["The first one".to_string(), "The second one".to_string()],
        ),
        (_, FieldKey::ReferenceTarget) => (
            "What does that refer to?".to_string(),
            vec![
                "The meeting".to_string(),
                "The reminder".to_string(),
                "The last thing we talked about".to_string(),
            ],
        ),
        (IntentType::SetReminder, FieldKey::Task) => (
            "What should I remind you about?".to_string(),
            vec![
                "Remind me to call mom".to_string(),
                "Remind me to pay rent".to_string(),
            ],
        ),
        (IntentType::SetReminder, FieldKey::When) => (
            "When should I remind you?".to_string(),
            vec![
                "Tomorrow at 3pm".to_string(),
                "In 30 minutes".to_string(),
                "Tonight at 8".to_string(),
            ],
        ),
        (IntentType::CreateCalendarEvent, FieldKey::When) => (
            "When is the meeting?".to_string(),
            vec![
                "Tomorrow at 3pm".to_string(),
                "Friday 10am".to_string(),
                "2026-02-10 15:00".to_string(),
            ],
        ),
        (IntentType::BookTable, FieldKey::When) => (
            "What day and time should I book it for?".to_string(),
            vec![
                "Tomorrow 7pm".to_string(),
                "Friday at 8".to_string(),
                "2026-02-10 19:00".to_string(),
            ],
        ),
        (IntentType::BookTable, FieldKey::Place) => (
            "Where should I book the table?".to_string(),
            vec!["At Marina Bay".to_string(), "At Sushi Den".to_string()],
        ),
        (IntentType::BookTable, FieldKey::PartySize) => (
            "For how many people?".to_string(),
            vec!["For 2".to_string(), "For four".to_string()],
        ),
        (IntentType::SendMoney, FieldKey::Amount) => (
            "How much should I send?".to_string(),
            vec![
                "$20".to_string(),
                "100 dollars".to_string(),
                "15".to_string(),
            ],
        ),
        (IntentType::SendMoney, FieldKey::Recipient) => (
            "Who should I send it to?".to_string(),
            vec!["To Alex".to_string(), "To John".to_string()],
        ),
        (IntentType::MemoryRememberRequest, FieldKey::Task) => (
            "What exactly should I remember?".to_string(),
            vec![
                "Remember that Benji is my preferred name".to_string(),
                "Remember my parking spot is B12".to_string(),
                "Remember I prefer short replies".to_string(),
            ],
        ),
        (IntentType::MemoryForgetRequest, FieldKey::Task) => (
            "What should I forget?".to_string(),
            vec![
                "Forget my old nickname".to_string(),
                "Forget parking spot B12".to_string(),
                "Forget that preference".to_string(),
            ],
        ),
        (IntentType::MemoryQuery, FieldKey::Task) => (
            "What topic should I recall?".to_string(),
            vec![
                "What do you remember about payroll".to_string(),
                "Recall my language preference".to_string(),
                "What did we say about Japan".to_string(),
            ],
        ),
        (IntentType::CapreqManage, FieldKey::RequestedCapabilityId) => (
            "Which capability should this request cover?".to_string(),
            vec![
                "position.activate".to_string(),
                "access.override.create".to_string(),
                "payroll.approve".to_string(),
            ],
        ),
        (IntentType::CapreqManage, FieldKey::CapreqAction) => (
            "Which capability-request action should I run?".to_string(),
            vec![
                "create_draft".to_string(),
                "submit_for_approval".to_string(),
                "approve".to_string(),
            ],
        ),
        (IntentType::CapreqManage, FieldKey::CapreqId) => (
            "Which capability request ID is this for?".to_string(),
            vec![
                "capreq_abc123".to_string(),
                "capreq_tenant_1_payroll".to_string(),
                "capreq_store_17_mgr".to_string(),
            ],
        ),
        (IntentType::CapreqManage, FieldKey::TargetScopeRef) => (
            "What target scope should this apply to?".to_string(),
            vec![
                "store_17".to_string(),
                "team.finance".to_string(),
                "tenant_default".to_string(),
            ],
        ),
        (IntentType::CapreqManage, FieldKey::Justification) => (
            "What is the justification?".to_string(),
            vec![
                "Monthly payroll processing".to_string(),
                "Need temporary manager coverage".to_string(),
                "Required for onboarding completion".to_string(),
            ],
        ),
        (IntentType::CapreqManage, FieldKey::TenantId) => (
            "Which tenant/company is this for?".to_string(),
            vec![
                "tenant_1".to_string(),
                "acme".to_string(),
                "selene_inc".to_string(),
            ],
        ),
        (IntentType::AccessSchemaManage, FieldKey::AccessProfileId) => (
            "Which access profile should I manage?".to_string(),
            vec![
                "AP_CLERK".to_string(),
                "AP_DRIVER".to_string(),
                "AP_CEO".to_string(),
            ],
        ),
        (IntentType::AccessSchemaManage, FieldKey::SchemaVersionId) => (
            "Which schema version should I use?".to_string(),
            vec!["v1".to_string(), "v2".to_string(), "v3".to_string()],
        ),
        (IntentType::AccessSchemaManage, FieldKey::ApScope) => (
            "Is this global or tenant scope?".to_string(),
            vec!["GLOBAL".to_string(), "TENANT".to_string()],
        ),
        (IntentType::AccessSchemaManage, FieldKey::ApAction) => (
            "What access profile action should I run?".to_string(),
            vec![
                "CREATE_DRAFT".to_string(),
                "UPDATE".to_string(),
                "ACTIVATE".to_string(),
                "RETIRE".to_string(),
            ],
        ),
        (IntentType::AccessSchemaManage, FieldKey::AccessReviewChannel) => (
            "Should I send this to your phone or desktop for review, or read it out loud?"
                .to_string(),
            vec!["PHONE_DESKTOP".to_string(), "READ_OUT_LOUD".to_string()],
        ),
        (IntentType::AccessSchemaManage, FieldKey::AccessRuleAction) => (
            "Which rule action should I record?".to_string(),
            vec![
                "AGREE".to_string(),
                "DISAGREE".to_string(),
                "EDIT".to_string(),
                "DELETE".to_string(),
                "DISABLE".to_string(),
                "ADD_CUSTOM_RULE".to_string(),
            ],
        ),
        (IntentType::AccessSchemaManage, FieldKey::ProfilePayloadJson) => (
            "Please provide the profile rule payload.".to_string(),
            vec![
                "{\"allow\":[\"LINK_INVITE\"]}".to_string(),
                "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"],\"limits\":{\"amount\":5000}}".to_string(),
            ],
        ),
        (IntentType::AccessEscalationVote, FieldKey::EscalationCaseId) => (
            "Which escalation case is this for?".to_string(),
            vec![
                "esc_case_001".to_string(),
                "esc_case_store_17".to_string(),
                "esc_case_budget_2026".to_string(),
            ],
        ),
        (IntentType::AccessEscalationVote, FieldKey::VoteAction) => (
            "What vote action should I run?".to_string(),
            vec!["CAST_VOTE".to_string(), "RESOLVE".to_string()],
        ),
        (IntentType::AccessEscalationVote, FieldKey::VoteValue) => (
            "What vote value should I record?".to_string(),
            vec!["APPROVE".to_string(), "REJECT".to_string()],
        ),
        (IntentType::AccessInstanceCompileRefresh, FieldKey::TargetUserId) => (
            "Which user should I compile access for?".to_string(),
            vec![
                "user_123".to_string(),
                "employee_warehouse_mgr".to_string(),
                "driver_27".to_string(),
            ],
        ),
        (IntentType::AccessInstanceCompileRefresh, FieldKey::CompileReason) => (
            "Why are we compiling this access instance?".to_string(),
            vec![
                "POSITION_CHANGED".to_string(),
                "AP_VERSION_ACTIVATED".to_string(),
                "OVERRIDE_UPDATED".to_string(),
            ],
        ),
        // Default deterministic fallback.
        (_, _) => (
            "Can you clarify that?".to_string(),
            vec![
                "One short sentence".to_string(),
                "A few keywords".to_string(),
            ],
        ),
    };

    let rc = if primary == FieldKey::ReferenceTarget {
        reason_codes::N_CLARIFY_AMBIGUOUS
    } else {
        reason_codes::N_CLARIFY_MISSING_FIELD
    };

    let (sens, confirm) = meta_for_intent(intent_type);
    let ambiguity_flags = match primary {
        FieldKey::ReferenceTarget => vec![AmbiguityFlag::ReferenceAmbiguous],
        FieldKey::IntentChoice => vec![AmbiguityFlag::MultiIntent],
        _ => vec![],
    };

    Clarify::v1(
        question,
        vec![primary],
        formats,
        rc,
        sens,
        confirm,
        ambiguity_flags,
        vec![],
    )
}

fn select_primary_missing(missing: &[FieldKey]) -> FieldKey {
    // Priority: Amount > Recipient > Task > When > Place > PartySize > Person
    for k in [
        FieldKey::IntentChoice,
        FieldKey::ReferenceTarget,
        FieldKey::AccessReviewChannel,
        FieldKey::AccessRuleAction,
        FieldKey::ApAction,
        FieldKey::AccessProfileId,
        FieldKey::SchemaVersionId,
        FieldKey::ApScope,
        FieldKey::ProfilePayloadJson,
        FieldKey::EscalationCaseId,
        FieldKey::VoteAction,
        FieldKey::VoteValue,
        FieldKey::TargetUserId,
        FieldKey::CompileReason,
        FieldKey::CapreqAction,
        FieldKey::CapreqId,
        FieldKey::RequestedCapabilityId,
        FieldKey::TargetScopeRef,
        FieldKey::Justification,
        FieldKey::TenantId,
        FieldKey::Amount,
        FieldKey::Recipient,
        FieldKey::Task,
        FieldKey::When,
        FieldKey::Place,
        FieldKey::PartySize,
        FieldKey::Person,
    ] {
        if missing.contains(&k) {
            return k;
        }
    }
    missing[0]
}

fn contains_reference_word(lower: &str) -> bool {
    // Deterministic boundary-aware scan for simple reference words.
    // We keep this conservative: only a small set of pronouns commonly used as "referents".
    contains_word(lower, "that")
        || contains_word(lower, "it")
        || contains_word(lower, "this")
        || contains_word(lower, "there")
}

fn is_reference_like(span: &str) -> bool {
    let s = span.trim().to_ascii_lowercase();
    matches!(
        s.as_str(),
        "that"
            | "it"
            | "this"
            | "there"
            | "that thing"
            | "this thing"
            | "that one"
            | "this one"
            | "that stuff"
            | "this stuff"
    )
}

fn contains_word(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let bytes = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.len() > bytes.len() {
        return false;
    }
    let mut i = 0usize;
    while i + n.len() <= bytes.len() {
        if &bytes[i..i + n.len()] == n {
            let left_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
            let right_ok =
                i + n.len() == bytes.len() || !bytes[i + n.len()].is_ascii_alphanumeric();
            if left_ok && right_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn clarify_for_multi_intent(intents: &[IntentType]) -> Result<Clarify, ContractViolation> {
    let mut labels: Vec<String> = intents.iter().map(intent_label).collect();
    labels.sort();
    labels.dedup();
    let mut formats = labels.into_iter().take(3).collect::<Vec<_>>();
    if formats.len() < 2 {
        formats.push("The first one".to_string());
        formats.push("The second one".to_string());
        formats.truncate(3);
    }
    let (sens, confirm) = meta_for_intents(intents);
    Clarify::v1(
        "I heard more than one request. Which one should I do first?".to_string(),
        vec![FieldKey::IntentChoice],
        formats,
        reason_codes::N_CLARIFY_MULTI_INTENT,
        sens,
        confirm,
        vec![AmbiguityFlag::MultiIntent],
        vec![],
    )
}

fn intent_label(t: &IntentType) -> String {
    match t {
        IntentType::WeatherQuery => "Weather".to_string(),
        IntentType::TimeQuery => "Time".to_string(),
        IntentType::WebSearchQuery => "Search the web".to_string(),
        IntentType::NewsQuery => "Get news".to_string(),
        IntentType::UrlFetchAndCiteQuery => "Open URL and cite".to_string(),
        IntentType::DocumentUnderstandQuery => "Read and summarize document".to_string(),
        IntentType::PhotoUnderstandQuery => "Understand photo or screenshot".to_string(),
        IntentType::DataAnalysisQuery => "Analyze uploaded data".to_string(),
        IntentType::DeepResearchQuery => "Deep research report".to_string(),
        IntentType::RecordModeQuery => "Summarize recording and action items".to_string(),
        IntentType::SetReminder => "Set a reminder".to_string(),
        IntentType::CreateCalendarEvent => "Schedule a meeting".to_string(),
        IntentType::BookTable => "Book a table".to_string(),
        IntentType::SendMoney => "Send money".to_string(),
        IntentType::MemoryRememberRequest => "Remember this".to_string(),
        IntentType::MemoryForgetRequest => "Forget memory".to_string(),
        IntentType::MemoryQuery => "Recall memory".to_string(),
        IntentType::CreateInviteLink => "Create an invite link".to_string(),
        IntentType::CapreqManage => "Manage a capability request".to_string(),
        IntentType::AccessSchemaManage => "Manage access schemas".to_string(),
        IntentType::AccessEscalationVote => "Manage access escalation votes".to_string(),
        IntentType::AccessInstanceCompileRefresh => {
            "Compile or refresh access instance".to_string()
        }
        IntentType::Continue => "Continue".to_string(),
        IntentType::MoreDetail => "More detail".to_string(),
    }
}

fn transcript_hash_v1a64(transcript: &str) -> TranscriptHash {
    // FNV-1a 64-bit (stable across platforms, deterministic).
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut h = OFFSET;
    for &b in transcript.as_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(PRIME);
    }
    TranscriptHash(h)
}

fn evidence_span(
    field: FieldKey,
    transcript: &str,
    excerpt: &str,
) -> Result<EvidenceSpan, ContractViolation> {
    let th = transcript_hash_v1a64(transcript);
    let start = transcript
        .find(excerpt)
        .ok_or(ContractViolation::InvalidValue {
            field: "evidence_span.verbatim_excerpt",
            reason: "excerpt must be a substring of transcript_text",
        })?;
    let end = start.saturating_add(excerpt.len());
    let ev = EvidenceSpan {
        field,
        transcript_hash: th,
        start_byte: start as u32,
        end_byte: end as u32,
        verbatim_excerpt: excerpt.to_string(),
    };
    ev.validate()?;
    Ok(ev)
}

fn clarify_for_uncertain_span(
    transcript: &str,
    spans: &[selene_kernel_contracts::ph1n::UncertainSpan],
) -> Result<Option<Clarify>, ContractViolation> {
    let Some(s) = spans.first() else {
        return Ok(None);
    };

    // Extract the uncertain excerpt safely (byte offsets).
    let start = s.start_byte as usize;
    let end = s.end_byte as usize;
    let excerpt = transcript.get(start..end).unwrap_or("").trim().to_string();

    let field = s.field_hint.unwrap_or_else(|| match s.kind {
        UncertainSpanKind::AmountLike => FieldKey::Amount,
        UncertainSpanKind::DateTimeLike => FieldKey::When,
        UncertainSpanKind::NameLike => FieldKey::Person,
        UncertainSpanKind::NumberLike => FieldKey::Task,
        UncertainSpanKind::Unknown => FieldKey::ReferenceTarget,
    });

    let (question, formats) = match field {
        FieldKey::Amount => (
            format!("I didn’t catch the amount. I heard \"{excerpt}\". What amount?"),
            vec![
                "$20".to_string(),
                "100 dollars".to_string(),
                "15".to_string(),
            ],
        ),
        FieldKey::When => (
            format!("I didn’t catch the time. I heard \"{excerpt}\". What time?"),
            vec![
                "Tomorrow at 3pm".to_string(),
                "Friday 10am".to_string(),
                "2026-02-10 15:00".to_string(),
            ],
        ),
        _ => (
            format!("I didn’t catch that part. I heard \"{excerpt}\". Can you repeat it?"),
            vec!["One short phrase".to_string(), "Just that part".to_string()],
        ),
    };

    // Best-effort meta from detected intents (still deterministic; keyword-only).
    let lower = transcript.to_ascii_lowercase();
    let intents = detect_intents(strip_wake_prefix(&lower));
    let (sens, confirm) = if intents.is_empty() {
        (SensitivityLevel::Private, false)
    } else {
        meta_for_intents(&intents)
    };
    let ambiguity_flags = match field {
        FieldKey::Amount => vec![AmbiguityFlag::AmountAmbiguous],
        FieldKey::When => vec![AmbiguityFlag::DateAmbiguous],
        FieldKey::Recipient => vec![AmbiguityFlag::RecipientAmbiguous],
        FieldKey::ReferenceTarget => vec![AmbiguityFlag::ReferenceAmbiguous],
        _ => vec![],
    };

    Ok(Some(Clarify::v1(
        question,
        vec![field],
        formats.into_iter().take(3).collect(),
        reason_codes::N_CLARIFY_UNCERTAIN_SPAN,
        sens,
        confirm,
        ambiguity_flags,
        vec![],
    )?))
}

fn extract_task_after(
    lower: &str,
    original: &str,
    marker: &str,
) -> Option<(Option<String>, Option<String>)> {
    let idx = lower.find(marker)?;
    let start = idx + marker.len();
    let orig_tail = &original[start..];
    let orig_tail = orig_tail.trim();
    if orig_tail.is_empty() {
        return Some((None, None));
    }

    // Stop at a deterministic time-marker boundary if present.
    let stop = earliest_stop_index(
        orig_tail,
        &[
            " tomorrow",
            " tmr",
            " today",
            " tonight",
            " 明天",
            " next week",
            " later",
        ],
    );
    let task = match stop {
        Some(i) => orig_tail[..i].trim(),
        None => orig_tail,
    };
    if task.trim().is_empty() {
        return Some((None, None));
    }
    Some((Some(task.to_string()), None))
}

fn extract_memory_subject(lower: &str, original: &str, markers: &[&str]) -> Option<String> {
    for marker in markers {
        if let Some((subject, _norm)) = extract_task_after(lower, original, marker) {
            if let Some(subject) = subject {
                return Some(subject);
            }
        }
    }
    None
}

fn earliest_stop_index(haystack: &str, needles: &[&str]) -> Option<usize> {
    needles
        .iter()
        .filter_map(|n| haystack.to_ascii_lowercase().find(n).map(|i| i))
        .min()
}

fn extract_when_span(original: &str) -> Option<(String, Option<TimeExpression>)> {
    // Deterministic, bounded patterns. This does NOT guess.
    // - ASCII slang mapping: tmr -> tomorrow
    // - Explicit tokens: today/tomorrow/tonight + optional time
    // - Minimal Chinese mapping: 明天 + "<digit>点"
    let lower = original.to_ascii_lowercase();

    if let Some(span) = extract_chinese_tomorrow_time(original) {
        return Some(span);
    }

    // Find the earliest explicit date token.
    let mut best: Option<(usize, &'static str, Option<String>)> = None;
    for (token, norm) in [
        ("tmr", Some("tomorrow".to_string())),
        ("tomorrow", None),
        ("today", None),
        ("tonight", None),
    ] {
        if let Some(idx) = lower.find(token) {
            match &best {
                None => best = Some((idx, token, norm.clone())),
                Some((best_idx, _, _)) if idx < *best_idx => {
                    best = Some((idx, token, norm.clone()))
                }
                _ => {}
            }
        }
    }
    let (idx, token, norm) = best?;
    let tail = &original[idx..];

    // Include an immediate time token if present (e.g., "tomorrow 3pm").
    let mut end = token.len();
    if let Some(t) = extract_time_token(&tail[end..]) {
        // `t` already includes leading whitespace; extend end accordingly.
        end += t.len();
    }

    let orig = tail[..end].trim().to_string();

    // Always emit a structured time expression for explicit time tokens (relative or absolute),
    // but never resolve relative phrases to absolute timestamps without time_context.
    let normalized = if let Some(n) = norm {
        if orig.to_ascii_lowercase().starts_with("tmr") {
            orig.replacen("tmr", &n, 1)
        } else {
            n
        }
    } else {
        orig.to_ascii_lowercase()
    };

    Some((
        orig,
        Some(TimeExpression {
            kind: TimeExpressionKind::DateKeyword,
            normalized,
        }),
    ))
}

fn extract_time_token(s: &str) -> Option<String> {
    // Expect: whitespace then time like "3pm", "3 pm", "7:30", "10am"
    let s = s;
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i == 0 {
        return None;
    }
    let start = 0;

    // Read digits (1-2).
    let mut j = i;
    let mut digits = 0usize;
    while j < bytes.len() && bytes[j].is_ascii_digit() && digits < 2 {
        j += 1;
        digits += 1;
    }
    if digits == 0 {
        return None;
    }

    // Optional ":mm"
    if j + 2 < bytes.len()
        && bytes[j] == b':'
        && bytes[j + 1].is_ascii_digit()
        && bytes[j + 2].is_ascii_digit()
    {
        j += 3;
    }

    // Optional whitespace.
    let mut k = j;
    while k < bytes.len() && bytes[k].is_ascii_whitespace() {
        k += 1;
    }

    // Optional am/pm.
    let end = if k + 1 < bytes.len() {
        let a = bytes[k].to_ascii_lowercase();
        let b = bytes[k + 1].to_ascii_lowercase();
        if (a == b'a' || a == b'p') && b == b'm' {
            k + 2
        } else {
            j
        }
    } else {
        j
    };

    Some(s[start..end].to_string())
}

fn extract_chinese_tomorrow_time(original: &str) -> Option<(String, Option<TimeExpression>)> {
    let idx = original.find("明天")?;
    let tail = &original[idx..];

    // Look for a pattern like "明天 7点" within the tail.
    let mut hour: Option<u32> = None;
    let mut hour_start: Option<usize> = None;
    let bytes = tail.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i].is_ascii_digit() {
            hour_start = Some(i);
            let mut j = i;
            let mut val: u32 = 0;
            let mut digits = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() && digits < 2 {
                val = val * 10 + (bytes[j] - b'0') as u32;
                j += 1;
                digits += 1;
            }
            if j < bytes.len() && tail[j..].starts_with("点") {
                hour = Some(val);
                break;
            }
        }
    }

    let orig = if let Some(hs) = hour_start {
        // Include "明天" up through "<digits>点".
        let mut end = hs;
        while end < bytes.len() && !bytes[end].is_ascii_digit() {
            end += 1;
        }
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end < bytes.len() && tail[end..].starts_with("点") {
            end += "点".len();
        }
        tail[..end].trim().to_string()
    } else {
        "明天".to_string()
    };

    let normalized = hour
        .map(|h| format!("tomorrow {:02}:00", h))
        .unwrap_or_else(|| "tomorrow".to_string());
    Some((
        orig,
        Some(TimeExpression {
            kind: TimeExpressionKind::DateKeyword,
            normalized,
        }),
    ))
}

fn extract_simple_person(lower: &str, original: &str) -> Option<String> {
    // Deterministic: "meeting <Name>"
    let idx = lower.find("meeting ")?;
    let start = idx + "meeting ".len();
    let tail = &original[start..];
    let tail = tail.trim();
    if tail.is_empty() {
        return None;
    }
    let name = tail.split_whitespace().next().unwrap_or("");
    if name.trim().is_empty() {
        return None;
    }
    Some(name.to_string())
}

fn extract_place_after_at(lower: &str, original: &str) -> Option<String> {
    // Deterministic: "at <Place ...>" until " for " if present.
    let idx = lower.find(" at ")?;
    let start = idx + " at ".len();
    let tail = &original[start..];
    let tail_lower = &lower[start..];
    let stop = tail_lower.find(" for ").unwrap_or(tail.len());
    let place = tail[..stop].trim();
    if place.is_empty() {
        return None;
    }
    Some(place.to_string())
}

fn extract_party_size(lower: &str, original: &str) -> Option<(String, Option<String>)> {
    let idx = lower.find(" for ")?;
    let start = idx + " for ".len();
    let tail = &original[start..];
    let word = tail.split_whitespace().next()?;
    let norm = number_word_to_digit(word).or_else(|| digits_only(word).map(|d| d.to_string()));
    Some((word.to_string(), norm))
}

fn digits_only(s: &str) -> Option<u32> {
    if s.is_empty() {
        return None;
    }
    let mut val: u32 = 0;
    for b in s.as_bytes() {
        if !b.is_ascii_digit() {
            return None;
        }
        val = val.saturating_mul(10).saturating_add((b - b'0') as u32);
    }
    Some(val)
}

fn number_word_to_digit(word: &str) -> Option<String> {
    match word.to_ascii_lowercase().as_str() {
        "one" => Some("1".to_string()),
        "two" => Some("2".to_string()),
        "three" => Some("3".to_string()),
        "four" => Some("4".to_string()),
        "five" => Some("5".to_string()),
        _ => None,
    }
}

fn extract_recipient_after_to(lower: &str, original: &str) -> Option<String> {
    let idx = lower.find(" to ")?;
    let start = idx + " to ".len();
    let tail = &original[start..];
    let name = tail.split_whitespace().next()?;
    if name.trim().is_empty() {
        return None;
    }
    Some(name.to_string())
}

fn extract_amount(lower: &str, original: &str) -> Option<(String, Option<String>)> {
    // Minimal deterministic amount parsing: "$20" or "20" or "100 dollars".
    // This intentionally ignores currency semantics for now.
    let tokens: Vec<&str> = lower.split_whitespace().collect();
    for (i, tok) in tokens.iter().enumerate() {
        let cleaned = tok.trim_matches(|c: char| c == '$' || c == ',' || c == '.');
        if let Some(d) = digits_only(cleaned) {
            // Try to capture original token span.
            let orig_tokens: Vec<&str> = original.split_whitespace().collect();
            let orig_tok = orig_tokens.get(i).copied().unwrap_or(*tok);
            return Some((orig_tok.to_string(), Some(d.to_string())));
        }
    }
    None
}

fn excerpt_from_lower_match(lower: &str, original: &str, needle: &str) -> Option<String> {
    let idx = lower.find(needle)?;
    let end = idx.saturating_add(needle.len());
    if end > original.len() {
        return None;
    }
    Some(original[idx..end].to_string())
}

fn extract_invitee_type(lower: &str, original: &str) -> Option<(String, String)> {
    if let Some(orig) = excerpt_from_lower_match(lower, original, "company") {
        return Some((orig, "company".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "customer") {
        return Some((orig, "customer".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "employee") {
        return Some((orig, "employee".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "associate") {
        return Some((orig, "associate".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "friend") {
        return Some((orig, "friend".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "family_member") {
        return Some((orig, "family_member".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "family member") {
        return Some((orig, "family_member".to_string()));
    }
    for w in ["husband", "wife", "son", "daughter", "family"] {
        if let Some(orig) = excerpt_from_lower_match(lower, original, w) {
            return Some((orig, "family_member".to_string()));
        }
    }
    None
}

fn extract_delivery_method(lower: &str, original: &str) -> Option<(String, String)> {
    if let Some(orig) = excerpt_from_lower_match(lower, original, "sms") {
        return Some((orig, "sms".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "text") {
        return Some((orig, "sms".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "email") {
        return Some((orig, "email".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "whatsapp") {
        return Some((orig, "whatsapp".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "wechat") {
        return Some((orig, "wechat".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "qr") {
        return Some((orig, "qr".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "copy link") {
        return Some((orig, "copy_link".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "copy-link") {
        return Some((orig, "copy_link".to_string()));
    }
    None
}

fn extract_recipient_contact(original: &str) -> Option<String> {
    // Minimal deterministic extraction: first email-like token, otherwise phone-like token.
    for tok in original.split_whitespace() {
        // Strip common surrounding punctuation without removing email/phone characters.
        let t = tok.trim_matches(|c: char| {
            matches!(
                c,
                ',' | ';' | ':' | ')' | ']' | '}' | '!' | '?' | '"' | '\'' | '(' | '[' | '{'
            )
        });
        if t.contains('@') {
            return Some(t.to_string());
        }
        let digit_count = t.chars().filter(|c| c.is_ascii_digit()).count();
        if digit_count >= 7 {
            return Some(t.to_string());
        }
    }
    None
}

fn extract_token_after_phrase(lower: &str, original: &str, phrase: &str) -> Option<String> {
    let idx = lower.find(phrase)?;
    let start = idx + phrase.len();
    let tail = &original[start..];
    let token = tail.trim_start().split_whitespace().next()?;
    let cleaned = token.trim_matches(|c: char| {
        matches!(
            c,
            ',' | ';' | ':' | ')' | ']' | '}' | '!' | '?' | '"' | '\'' | '(' | '[' | '{'
        )
    });
    if cleaned.is_empty() {
        return None;
    }
    Some(cleaned.to_string())
}

fn extract_tail_after_phrase(lower: &str, original: &str, phrase: &str) -> Option<String> {
    let idx = lower.find(phrase)?;
    let start = idx + phrase.len();
    let tail = original[start..].trim();
    if tail.is_empty() {
        return None;
    }
    Some(tail.to_string())
}

fn detect_access_schema_action(lower: &str) -> &'static str {
    if lower.contains("activate ap")
        || lower.contains("activate profile")
        || lower.contains("activate access profile")
        || lower.contains("set active")
    {
        return "ACTIVATE";
    }
    if lower.contains("retire ap")
        || lower.contains("retire profile")
        || lower.contains("retire access profile")
        || lower.contains("deactivate ap")
    {
        return "RETIRE";
    }
    if lower.contains("update ap")
        || lower.contains("edit ap")
        || lower.contains("modify ap")
        || lower.contains("change ap")
    {
        return "UPDATE";
    }
    "CREATE_DRAFT"
}

fn detect_access_scope(lower: &str) -> &'static str {
    if lower.contains("global") || lower.contains("selene global") {
        "GLOBAL"
    } else {
        "TENANT"
    }
}

fn detect_access_review_channel(lower: &str, original: &str) -> Option<(String, String)> {
    if let Some(orig) = excerpt_from_lower_match(lower, original, "read out loud") {
        return Some((orig, "READ_OUT_LOUD".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "read it out loud") {
        return Some((orig, "READ_OUT_LOUD".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "read aloud") {
        return Some((orig, "READ_OUT_LOUD".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "phone")
        .or_else(|| excerpt_from_lower_match(lower, original, "desktop"))
    {
        return Some((orig, "PHONE_DESKTOP".to_string()));
    }
    None
}

fn detect_access_rule_action(lower: &str, original: &str) -> Option<(String, String)> {
    if let Some(orig) = excerpt_from_lower_match(lower, original, "add custom")
        .or_else(|| excerpt_from_lower_match(lower, original, "add rule"))
    {
        return Some((orig, "ADD_CUSTOM_RULE".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "disable") {
        return Some((orig, "DISABLE".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "delete") {
        return Some((orig, "DELETE".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "edit") {
        return Some((orig, "EDIT".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "disagree")
        .or_else(|| excerpt_from_lower_match(lower, original, "reject"))
    {
        return Some((orig, "DISAGREE".to_string()));
    }
    if let Some(orig) = excerpt_from_lower_match(lower, original, "agree")
        .or_else(|| excerpt_from_lower_match(lower, original, "approve"))
    {
        return Some((orig, "AGREE".to_string()));
    }
    None
}

fn extract_access_profile_id(lower: &str, original: &str) -> Option<String> {
    if let Some(found) = original.split_whitespace().find_map(|tok| {
        let cleaned = tok.trim_matches(|c: char| {
            matches!(
                c,
                ',' | ';' | ':' | ')' | ']' | '}' | '!' | '?' | '"' | '\'' | '(' | '[' | '{'
            )
        });
        let up = cleaned.to_ascii_uppercase();
        if up.starts_with("AP_") && up.len() <= 64 {
            Some(up)
        } else {
            None
        }
    }) {
        return Some(found);
    }

    extract_token_after_phrase(lower, original, "profile ")
        .or_else(|| extract_token_after_phrase(lower, original, "ap "))
}

fn extract_schema_version_id(lower: &str, original: &str) -> Option<String> {
    if let Some(tok) = original.split_whitespace().find_map(|tok| {
        let cleaned = tok.trim_matches(|c: char| {
            matches!(
                c,
                ',' | ';' | ':' | ')' | ']' | '}' | '!' | '?' | '"' | '\'' | '(' | '[' | '{'
            )
        });
        let lower = cleaned.to_ascii_lowercase();
        if lower.starts_with('v')
            && lower.len() <= 16
            && lower[1..].chars().all(|c| c.is_ascii_digit())
        {
            Some(lower)
        } else {
            None
        }
    }) {
        return Some(tok);
    }
    extract_token_after_phrase(lower, original, "version ").map(|v| v.to_ascii_lowercase())
}

fn extract_profile_payload_json(original: &str) -> Option<String> {
    let start = original.find('{')?;
    let end = original.rfind('}')?;
    if end <= start {
        return None;
    }
    let payload = original[start..=end].trim();
    if payload.is_empty() {
        return None;
    }
    Some(payload.to_string())
}

fn detect_vote_action(lower: &str) -> &'static str {
    if lower.contains("resolve") || lower.contains("finalize vote") {
        "RESOLVE"
    } else {
        "CAST_VOTE"
    }
}

fn detect_vote_value(lower: &str) -> &'static str {
    if lower.contains("reject") || lower.contains("deny") {
        "REJECT"
    } else {
        "APPROVE"
    }
}

fn extract_escalation_case_id(lower: &str, original: &str) -> Option<String> {
    extract_token_after_phrase(lower, original, "case ")
        .or_else(|| extract_token_after_phrase(lower, original, "case id "))
        .or_else(|| extract_token_after_phrase(lower, original, "escalation "))
}

fn extract_board_policy_id(lower: &str, original: &str) -> Option<String> {
    extract_token_after_phrase(lower, original, "board policy ")
        .or_else(|| extract_token_after_phrase(lower, original, "policy "))
}

fn extract_target_user_id(lower: &str, original: &str) -> Option<String> {
    extract_token_after_phrase(lower, original, "user ")
        .or_else(|| extract_token_after_phrase(lower, original, "employee "))
}

fn extract_position_id(lower: &str, original: &str) -> Option<String> {
    extract_token_after_phrase(lower, original, "position ")
}

fn detect_compile_reason(lower: &str) -> &'static str {
    if lower.contains("position changed") || lower.contains("position update") {
        "POSITION_CHANGED"
    } else if lower.contains("ap version") || lower.contains("profile version") {
        "AP_VERSION_ACTIVATED"
    } else if lower.contains("override") {
        "OVERRIDE_UPDATED"
    } else {
        "MANUAL_REFRESH"
    }
}

fn extract_requested_capability_id(lower: &str, original: &str) -> Option<String> {
    extract_token_after_phrase(lower, original, "capability ")
        .or_else(|| extract_token_after_phrase(lower, original, "request capability "))
        .or_else(|| extract_token_after_phrase(lower, original, "access to "))
        .or_else(|| extract_token_after_phrase(lower, original, "permission for "))
}

fn extract_target_scope_ref(lower: &str, original: &str) -> Option<String> {
    extract_token_after_phrase(lower, original, "scope ")
        .or_else(|| extract_token_after_phrase(lower, original, "for scope "))
        .or_else(|| extract_token_after_phrase(lower, original, "team "))
        .or_else(|| extract_token_after_phrase(lower, original, "store "))
}

fn extract_justification(lower: &str, original: &str) -> Option<String> {
    extract_tail_after_phrase(lower, original, "because ")
        .or_else(|| extract_tail_after_phrase(lower, original, "so that "))
}

fn extract_tenant_id(lower: &str, original: &str) -> Option<String> {
    extract_token_after_phrase(lower, original, "tenant ")
        .or_else(|| extract_token_after_phrase(lower, original, "company "))
}

fn detect_capreq_action(lower: &str) -> &'static str {
    let is_capreq = lower.contains("capreq")
        || lower.contains("capability request")
        || lower.contains("access request")
        || lower.contains("request capability");
    if !is_capreq {
        return "create_draft";
    }
    if lower.contains("submit capreq")
        || lower.contains("submit capability request")
        || lower.contains("submit access request")
    {
        return "submit_for_approval";
    }
    if lower.contains("approve capreq")
        || lower.contains("approve capability request")
        || lower.contains("approve access request")
        || lower.contains("approved capreq")
        || lower.contains("approved capability request")
    {
        return "approve";
    }
    if lower.contains("reject capreq")
        || lower.contains("reject capability request")
        || lower.contains("reject access request")
        || lower.contains("deny capreq")
        || lower.contains("deny capability request")
        || lower.contains("denied capreq")
    {
        return "reject";
    }
    if lower.contains("fulfill capreq")
        || lower.contains("fulfill capability request")
        || lower.contains("mark capreq fulfilled")
        || lower.contains("mark capability request fulfilled")
    {
        return "fulfill";
    }
    if lower.contains("cancel capreq")
        || lower.contains("cancel capability request")
        || lower.contains("revoke capreq")
        || lower.contains("withdraw capreq")
    {
        return "cancel";
    }
    "create_draft"
}

fn extract_capreq_id(lower: &str, original: &str) -> Option<String> {
    extract_token_after_phrase(lower, original, "capreq id ")
        .or_else(|| extract_token_after_phrase(lower, original, "capreq_id "))
        .or_else(|| extract_token_after_phrase(lower, original, "request id "))
        .or_else(|| extract_token_after_phrase(lower, original, "request_id "))
        .or_else(|| {
            original.split_whitespace().find_map(|tok| {
                let cleaned = tok.trim_matches(|c: char| {
                    matches!(
                        c,
                        ',' | ';'
                            | ':'
                            | ')'
                            | ']'
                            | '}'
                            | '!'
                            | '?'
                            | '"'
                            | '\''
                            | '('
                            | '['
                            | '{'
                    )
                });
                if cleaned.to_ascii_lowercase().starts_with("capreq_") {
                    Some(cleaned.to_string())
                } else {
                    None
                }
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1c::{
        ConfidenceBucket, LanguageTag, SessionStateRef, TranscriptOk,
    };
    use selene_kernel_contracts::ph1w::SessionState;

    fn req(transcript: &str, lang: &str) -> Ph1nRequest {
        let ok = TranscriptOk::v1(
            transcript.to_string(),
            LanguageTag::new(lang).unwrap(),
            ConfidenceBucket::High,
        )
        .unwrap();
        Ph1nRequest::v1(ok, SessionStateRef::v1(SessionState::Active, false)).unwrap()
    }

    #[test]
    fn at_n_01_broken_english_structured() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt
            .run(&req("Selene tomorrow 3pm meeting John confirm", "en"))
            .unwrap();

        match out {
            Ph1nResponse::IntentDraft(d) => {
                assert_eq!(d.intent_type, IntentType::CreateCalendarEvent);
                assert_eq!(d.overall_confidence, OverallConfidence::High);
                assert!(d.fields.iter().any(|f| f.key == FieldKey::When));
                assert!(d.fields.iter().any(|f| f.key == FieldKey::Person));
            }
            _ => panic!("expected intent_draft"),
        }
    }

    #[test]
    fn at_n_02_code_switch_preserved() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt
            .run(&req(
                "Selene book a table 明天 7点 at Marina Bay for two",
                "en",
            ))
            .unwrap();

        match out {
            Ph1nResponse::IntentDraft(d) => {
                assert_eq!(d.intent_type, IntentType::BookTable);
                assert!(d
                    .evidence_spans
                    .iter()
                    .any(|e| e.verbatim_excerpt.contains("明天")));
                assert!(d.fields.iter().any(|f| f.key == FieldKey::PartySize));
            }
            _ => panic!("expected intent_draft"),
        }
    }

    #[test]
    fn at_n_03_slang_does_not_break_intent_but_ambiguous_task_clarifies() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt
            .run(&req(
                "yo Selene can you set that thing for tmr morning",
                "en",
            ))
            .unwrap();
        assert!(matches!(out, Ph1nResponse::Clarify(_)));
    }

    #[test]
    fn at_n_04_ambiguous_slang_triggers_clarify() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt.run(&req("Selene handle that later", "en")).unwrap();
        match out {
            Ph1nResponse::Clarify(c) => {
                assert!(c.question.to_ascii_lowercase().contains("what"));
                assert_eq!(c.what_is_missing, vec![FieldKey::ReferenceTarget]);
                assert_eq!(c.reason_code, reason_codes::N_CLARIFY_AMBIGUOUS);
            }
            _ => panic!("expected clarify"),
        }
    }

    #[test]
    fn at_n_05_mixed_scripts_preserved_verbatim() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt
            .run(&req("Selene remind me to call 妈妈 tomorrow", "en"))
            .unwrap();
        match out {
            Ph1nResponse::IntentDraft(d) => {
                assert_eq!(d.intent_type, IntentType::SetReminder);
                assert!(d
                    .evidence_spans
                    .iter()
                    .any(|e| e.verbatim_excerpt.contains("妈妈")));
            }
            _ => panic!("expected intent_draft"),
        }
    }

    #[test]
    fn at_n_06_no_guessing_on_dates_times() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt.run(&req("Selene schedule it next week", "en")).unwrap();
        assert!(matches!(out, Ph1nResponse::Clarify(_)));
    }

    #[test]
    fn at_n_07_numbers_never_invented_send_money_clarifies_amount() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt.run(&req("Selene send money to Alex", "en")).unwrap();
        match out {
            Ph1nResponse::Clarify(c) => {
                assert!(c.what_is_missing.contains(&FieldKey::Amount));
            }
            _ => panic!("expected clarify"),
        }
    }

    #[test]
    fn run2_send_link_tenant_normalizes_create_invite() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let req = req("Selene send a link to Tom", "en")
            .with_runtime_tenant_id(Some("tenant_1".to_string()))
            .unwrap();
        let out = rt.run(&req).unwrap();
        match out {
            Ph1nResponse::IntentDraft(d) => {
                assert_eq!(d.intent_type, IntentType::CreateInviteLink);
                assert_eq!(d.required_fields_missing, vec![FieldKey::RecipientContact]);
                let tenant = d
                    .fields
                    .iter()
                    .find(|f| f.key == FieldKey::TenantId)
                    .expect("tenant field must be present");
                assert_eq!(tenant.value.original_span, "tenant_1");

                let recipient = d
                    .fields
                    .iter()
                    .find(|f| f.key == FieldKey::Recipient)
                    .expect("recipient name must be present");
                assert_eq!(recipient.value.original_span, "Tom");

                let method = d
                    .fields
                    .iter()
                    .find(|f| f.key == FieldKey::DeliveryMethod)
                    .expect("delivery method must be present");
                assert_eq!(method.value.normalized_value.as_deref(), Some("selene_app"));
            }
            _ => panic!("expected intent_draft"),
        }
    }

    #[test]
    fn run2_send_link_transcript_tenant_hint_mismatch_is_ignored() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let req = req("Selene send a link to Tom for tenant tenant_999", "en")
            .with_runtime_tenant_id(Some("tenant_1".to_string()))
            .unwrap();
        let out = rt.run(&req).unwrap();
        match out {
            Ph1nResponse::IntentDraft(d) => {
                assert_eq!(d.intent_type, IntentType::CreateInviteLink);
                let tenant = d
                    .fields
                    .iter()
                    .find(|f| f.key == FieldKey::TenantId)
                    .expect("tenant field must be present from runtime scope");
                assert_eq!(tenant.value.original_span, "tenant_1");
            }
            _ => panic!("expected intent_draft"),
        }
    }

    #[test]
    fn at_n_13_capreq_manage_structured() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt
            .run(&req(
                "Selene create a capability request for capability payroll.approve scope store_17 tenant tenant_1 because monthly payroll processing",
                "en",
            ))
            .unwrap();

        match out {
            Ph1nResponse::IntentDraft(d) => {
                assert_eq!(d.intent_type, IntentType::CapreqManage);
                assert!(d
                    .fields
                    .iter()
                    .any(|f| f.key == FieldKey::RequestedCapabilityId));
                assert!(d.fields.iter().any(|f| f.key == FieldKey::TargetScopeRef));
                assert!(d.fields.iter().any(|f| f.key == FieldKey::TenantId));
                assert!(d.fields.iter().any(|f| f.key == FieldKey::Justification));
                let action = d
                    .fields
                    .iter()
                    .find(|f| f.key == FieldKey::CapreqAction)
                    .expect("capreq action field");
                assert_eq!(
                    action.value.normalized_value.as_deref(),
                    Some("create_draft")
                );
            }
            _ => panic!("expected intent_draft"),
        }
    }

    #[test]
    fn at_n_14_capreq_submit_with_capreq_id_structured() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt
            .run(&req(
                "Selene submit capreq capreq_abc123 tenant tenant_1",
                "en",
            ))
            .unwrap();

        match out {
            Ph1nResponse::IntentDraft(d) => {
                assert_eq!(d.intent_type, IntentType::CapreqManage);
                assert!(d.fields.iter().any(|f| f.key == FieldKey::CapreqId));
                let action = d
                    .fields
                    .iter()
                    .find(|f| f.key == FieldKey::CapreqAction)
                    .expect("capreq action field");
                assert_eq!(
                    action.value.normalized_value.as_deref(),
                    Some("submit_for_approval")
                );
            }
            _ => panic!("expected intent_draft"),
        }
    }

    #[test]
    fn at_n_15_capreq_approve_missing_capreq_id_clarifies() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt
            .run(&req(
                "Selene approve capability request tenant tenant_1",
                "en",
            ))
            .unwrap();

        match out {
            Ph1nResponse::Clarify(c) => {
                assert_eq!(c.what_is_missing, vec![FieldKey::CapreqId]);
            }
            _ => panic!("expected clarify"),
        }
    }

    #[test]
    fn at_n_16_access_schema_manage_requires_review_channel() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt
            .run(&req(
                "Selene create AP_CLERK version v1 for tenant tenant_1",
                "en",
            ))
            .unwrap();
        match out {
            Ph1nResponse::Clarify(c) => {
                assert_eq!(c.what_is_missing, vec![FieldKey::AccessReviewChannel]);
            }
            _ => panic!("expected clarify"),
        }
    }

    #[test]
    fn at_n_17_access_schema_manage_structured_with_channel_and_action() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt
            .run(&req(
                "Selene activate AP_CLERK version v2 tenant tenant_1 read out loud agree",
                "en",
            ))
            .unwrap();
        match out {
            Ph1nResponse::IntentDraft(d) => {
                assert_eq!(d.intent_type, IntentType::AccessSchemaManage);
                assert!(d.fields.iter().any(|f| f.key == FieldKey::ApAction));
                assert!(d.fields.iter().any(|f| f.key == FieldKey::AccessProfileId));
                assert!(d.fields.iter().any(|f| f.key == FieldKey::SchemaVersionId));
                assert!(d
                    .fields
                    .iter()
                    .any(|f| f.key == FieldKey::AccessReviewChannel));
                assert!(d.fields.iter().any(|f| f.key == FieldKey::AccessRuleAction));
            }
            _ => panic!("expected intent_draft"),
        }
    }

    #[test]
    fn at_n_18_web_search_normalizes_from_common_phrase() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt
            .run(&req(
                "Selene search the web for PH1 tool routing best practices",
                "en",
            ))
            .unwrap();
        match out {
            Ph1nResponse::IntentDraft(d) => {
                assert_eq!(d.intent_type, IntentType::WebSearchQuery);
                assert_eq!(d.required_fields_missing, Vec::<FieldKey>::new());
            }
            _ => panic!("expected intent_draft"),
        }
    }

    #[test]
    fn at_n_19_news_normalizes_from_common_phrase() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt
            .run(&req(
                "Selene what's the latest news about AI chip policy",
                "en",
            ))
            .unwrap();
        match out {
            Ph1nResponse::IntentDraft(d) => {
                assert_eq!(d.intent_type, IntentType::NewsQuery);
                assert_eq!(d.required_fields_missing, Vec::<FieldKey>::new());
            }
            _ => panic!("expected intent_draft"),
        }
    }

    #[test]
    fn at_n_20_url_fetch_and_cite_normalizes_from_common_phrase() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt
            .run(&req(
                "Selene open this URL and cite it: https://example.com/spec",
                "en",
            ))
            .unwrap();
        match out {
            Ph1nResponse::IntentDraft(d) => {
                assert_eq!(d.intent_type, IntentType::UrlFetchAndCiteQuery);
                assert_eq!(d.required_fields_missing, Vec::<FieldKey>::new());
            }
            _ => panic!("expected intent_draft"),
        }
    }

    #[test]
    fn at_n_21_document_understand_normalizes_from_common_phrase() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt
            .run(&req("Selene read this PDF and summarize it", "en"))
            .unwrap();
        match out {
            Ph1nResponse::IntentDraft(d) => {
                assert_eq!(d.intent_type, IntentType::DocumentUnderstandQuery);
                assert_eq!(d.required_fields_missing, Vec::<FieldKey>::new());
            }
            _ => panic!("expected intent_draft"),
        }
    }

    #[test]
    fn at_n_22_photo_understand_normalizes_from_common_phrase() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt
            .run(&req("Selene what does this screenshot say?", "en"))
            .unwrap();
        match out {
            Ph1nResponse::IntentDraft(d) => {
                assert_eq!(d.intent_type, IntentType::PhotoUnderstandQuery);
                assert_eq!(d.required_fields_missing, Vec::<FieldKey>::new());
            }
            _ => panic!("expected intent_draft"),
        }
    }

    #[test]
    fn at_n_23_data_analysis_normalizes_from_common_phrase() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt
            .run(&req(
                "Selene analyze this CSV and show summary stats and a chart",
                "en",
            ))
            .unwrap();
        match out {
            Ph1nResponse::IntentDraft(d) => {
                assert_eq!(d.intent_type, IntentType::DataAnalysisQuery);
                assert_eq!(d.required_fields_missing, Vec::<FieldKey>::new());
            }
            _ => panic!("expected intent_draft"),
        }
    }

    #[test]
    fn at_n_24_deep_research_normalizes_from_common_phrase() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt
            .run(&req(
                "Selene do deep research on AI chip export controls with sources",
                "en",
            ))
            .unwrap();
        match out {
            Ph1nResponse::IntentDraft(d) => {
                assert_eq!(d.intent_type, IntentType::DeepResearchQuery);
                assert_eq!(d.required_fields_missing, Vec::<FieldKey>::new());
            }
            _ => panic!("expected intent_draft"),
        }
    }

    #[test]
    fn at_n_25_record_mode_normalizes_from_common_phrase() {
        let rt = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
        let out = rt
            .run(&req(
                "Selene summarize this meeting recording and list action items",
                "en",
            ))
            .unwrap();
        match out {
            Ph1nResponse::IntentDraft(d) => {
                assert_eq!(d.intent_type, IntentType::RecordModeQuery);
                assert_eq!(d.required_fields_missing, Vec::<FieldKey>::new());
            }
            _ => panic!("expected intent_draft"),
        }
    }
}
