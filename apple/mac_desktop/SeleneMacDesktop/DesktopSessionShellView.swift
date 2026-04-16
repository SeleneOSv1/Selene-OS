import Foundation
import SwiftUI

private func firstQueryValue(in queryItems: [URLQueryItem], name: String) -> String? {
    queryItems.first(where: { $0.name == name })?.value
}

private func boundedHint(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty else {
        return nil
    }

    if trimmed.count <= 18 {
        return trimmed
    }

    return "\(trimmed.prefix(8))...\(trimmed.suffix(4))"
}

private func boundedTitle(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty else {
        return nil
    }

    if trimmed.count <= 72 {
        return trimmed
    }

    return "\(trimmed.prefix(69))..."
}

private func boundedTranscript(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty else {
        return nil
    }

    if trimmed.count <= 180 {
        return trimmed
    }

    return "\(trimmed.prefix(177))..."
}

private func boundedSummary(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty else {
        return nil
    }

    if trimmed.count <= 220 {
        return trimmed
    }

    return "\(trimmed.prefix(217))..."
}

private func boundedClarifyQuestion(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty else {
        return nil
    }

    if trimmed.count <= 240 {
        return trimmed
    }

    return "\(trimmed.prefix(237))..."
}

private func boundedClarifyMissingField(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty,
          trimmed.count <= 128,
          !trimmed.contains("\n"),
          !trimmed.contains("\r") else {
        return nil
    }

    return trimmed
}

private func boundedBullet(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty else {
        return nil
    }

    if trimmed.count <= 140 {
        return trimmed
    }

    return "\(trimmed.prefix(137))..."
}

private func boundedResumeSummaryBullets(in queryItems: [URLQueryItem]) -> [String] {
    queryItems.compactMap { queryItem in
        guard queryItem.name == "resume_summary_bullets" else {
            return nil
        }

        return boundedBullet(queryItem.value)
    }
}

private func normalizedRecoveryEnumToken(_ rawValue: String) -> String {
    rawValue
        .trimmingCharacters(in: .whitespacesAndNewlines)
        .lowercased()
        .filter { $0.isLetter || $0.isNumber }
}

private enum CanonicalRecoveryMode: String, Equatable {
    case normal = "PersistenceRecoveryMode::Normal"
    case recovering = "PersistenceRecoveryMode::Recovering"
    case degradedRecovery = "PersistenceRecoveryMode::DegradedRecovery"
    case quarantinedLocalState = "PersistenceRecoveryMode::QuarantinedLocalState"

    static func parse(_ rawValue: String?) -> CanonicalRecoveryMode? {
        guard let rawValue else {
            return nil
        }

        switch normalizedRecoveryEnumToken(rawValue) {
        case "normal":
            return .normal
        case "recovering":
            return .recovering
        case "degradedrecovery":
            return .degradedRecovery
        case "quarantinedlocalstate":
            return .quarantinedLocalState
        default:
            return nil
        }
    }
}

private enum CanonicalReconciliationDecision: String, Equatable {
    case retrySameOperation = "ReconciliationDecision::RetrySameOperation"
    case reusePriorAuthoritativeOutcome = "ReconciliationDecision::ReusePriorAuthoritativeOutcome"
    case rejectStaleOperation = "ReconciliationDecision::RejectStaleOperation"
    case requestFreshSessionState = "ReconciliationDecision::RequestFreshSessionState"
    case quarantineLocalState = "ReconciliationDecision::QuarantineLocalState"

    static func parse(_ rawValue: String?) -> CanonicalReconciliationDecision? {
        guard let rawValue else {
            return nil
        }

        switch normalizedRecoveryEnumToken(rawValue) {
        case "retrysameoperation":
            return .retrySameOperation
        case "reusepriorauthoritativeoutcome":
            return .reusePriorAuthoritativeOutcome
        case "rejectstaleoperation":
            return .rejectStaleOperation
        case "requestfreshsessionstate":
            return .requestFreshSessionState
        case "quarantinelocalstate":
            return .quarantineLocalState
        default:
            return nil
        }
    }
}

private func normalizedInterruptEnumToken(_ rawValue: String) -> String {
    rawValue
        .trimmingCharacters(in: .whitespacesAndNewlines)
        .lowercased()
        .filter { $0.isLetter || $0.isNumber }
}

private enum CanonicalInterruptSubjectRelation: String, Equatable {
    case same = "InterruptSubjectRelation::Same"
    case switchTopic = "InterruptSubjectRelation::Switch"
    case uncertain = "InterruptSubjectRelation::Uncertain"

    static func parse(_ rawValue: String?) -> CanonicalInterruptSubjectRelation? {
        guard let rawValue else {
            return nil
        }

        switch normalizedInterruptEnumToken(rawValue) {
        case "same", "interruptsubjectrelationsame":
            return .same
        case "switch", "interruptsubjectrelationswitch":
            return .switchTopic
        case "uncertain", "interruptsubjectrelationuncertain":
            return .uncertain
        default:
            return nil
        }
    }
}

private enum CanonicalInterruptContinuityOutcome: String, Equatable {
    case sameSubjectAppend = "InterruptContinuityOutcome::SameSubjectAppend"
    case switchTopicThenReturnCheck = "InterruptContinuityOutcome::SwitchTopicThenReturnCheck"

    static func parse(_ rawValue: String?) -> CanonicalInterruptContinuityOutcome? {
        guard let rawValue else {
            return nil
        }

        switch normalizedInterruptEnumToken(rawValue) {
        case "samesubjectappend", "interruptcontinuityoutcomesamesubjectappend":
            return .sameSubjectAppend
        case "switchtopicthenreturncheck", "interruptcontinuityoutcomeswitchtopicthenreturncheck":
            return .switchTopicThenReturnCheck
        default:
            return nil
        }
    }
}

private enum CanonicalInterruptResumePolicy: String, Equatable {
    case resumeNow = "InterruptResumePolicy::ResumeNow"
    case resumeLater = "InterruptResumePolicy::ResumeLater"
    case discard = "InterruptResumePolicy::Discard"

    static func parse(_ rawValue: String?) -> CanonicalInterruptResumePolicy? {
        guard let rawValue else {
            return nil
        }

        switch normalizedInterruptEnumToken(rawValue) {
        case "resumenow", "interruptresumepolicyresumenow":
            return .resumeNow
        case "resumelater", "interruptresumepolicyresumelater":
            return .resumeLater
        case "discard", "interruptresumepolicydiscard":
            return .discard
        default:
            return nil
        }
    }
}

private enum CanonicalInterruptAcceptedAnswerFormat: String, CaseIterable, Identifiable {
    case continuePreviousTopic = "Continue previous topic"
    case switchToNewTopic = "Switch to new topic"
    case notSureYet = "Not sure yet"

    var id: String {
        rawValue
    }

    static func parse(_ rawValue: String) -> CanonicalInterruptAcceptedAnswerFormat? {
        switch rawValue.trimmingCharacters(in: .whitespacesAndNewlines) {
        case Self.continuePreviousTopic.rawValue:
            return .continuePreviousTopic
        case Self.switchToNewTopic.rawValue:
            return .switchToNewTopic
        case Self.notSureYet.rawValue:
            return .notSureYet
        default:
            return nil
        }
    }
}

private enum CanonicalInterruptClarifyAmbiguityFlag: String, CaseIterable, Identifiable {
    case referenceAmbiguous = "reference_ambiguous"
    case recipientAmbiguous = "recipient_ambiguous"
    case dateAmbiguous = "date_ambiguous"
    case amountAmbiguous = "amount_ambiguous"
    case multiIntent = "multi_intent"

    var id: String {
        rawValue
    }

    static func parse(_ rawValue: String) -> CanonicalInterruptClarifyAmbiguityFlag? {
        Self(rawValue: rawValue.trimmingCharacters(in: .whitespacesAndNewlines))
    }
}

private enum CanonicalInterruptClarifyRoutingHint: String, CaseIterable, Identifiable {
    case onboardingStart = "onboarding_start"
    case onboardingConfirmIdentity = "onboarding_confirm_identity"
    case onboardingComplete = "onboarding_complete"
    case onboardingLanguageDetect = "onboarding_language_detect"

    var id: String {
        rawValue
    }

    static func parse(_ rawValue: String) -> CanonicalInterruptClarifyRoutingHint? {
        Self(rawValue: rawValue.trimmingCharacters(in: .whitespacesAndNewlines))
    }
}

private func collectedInterruptAcceptedAnswerFormats(in queryItems: [URLQueryItem]) -> [String] {
    var formats: [String] = []

    for queryItem in queryItems where queryItem.name == "interrupt_accepted_answer_format" {
        guard let value = queryItem.value,
              let canonicalValue = CanonicalInterruptAcceptedAnswerFormat.parse(value)?.rawValue,
              !formats.contains(canonicalValue) else {
            return []
        }

        formats.append(canonicalValue)
    }

    guard (2...3).contains(formats.count) else {
        return []
    }

    return formats
}

private func collectedInterruptClarifyWhatIsMissing(in queryItems: [URLQueryItem]) -> String? {
    let values = queryItems.filter { $0.name == "interrupt_clarify_what_is_missing" }
    guard values.count == 1 else {
        return nil
    }

    return boundedClarifyMissingField(values[0].value)
}

private func collectedInterruptClarifyAmbiguityFlags(in queryItems: [URLQueryItem]) -> [String] {
    var flags: [String] = []

    for queryItem in queryItems where queryItem.name == "interrupt_clarify_ambiguity_flag" {
        guard let value = queryItem.value,
              let canonicalValue = CanonicalInterruptClarifyAmbiguityFlag.parse(value)?.rawValue,
              !flags.contains(canonicalValue),
              flags.count < 2 else {
            return []
        }

        flags.append(canonicalValue)
    }

    if flags.isEmpty {
        return []
    }

    return flags
}

private func collectedInterruptClarifyRoutingHints(in queryItems: [URLQueryItem]) -> [String] {
    var hints: [String] = []

    for queryItem in queryItems where queryItem.name == "interrupt_clarify_routing_hint" {
        guard let value = queryItem.value,
              let canonicalValue = CanonicalInterruptClarifyRoutingHint.parse(value)?.rawValue,
              !hints.contains(canonicalValue),
              hints.count < 2 else {
            return []
        }

        hints.append(canonicalValue)
    }

    if hints.isEmpty {
        return []
    }

    return hints
}

private func canonicalSessionAttachOutcome(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    switch rawValue.trimmingCharacters(in: .whitespacesAndNewlines) {
    case "NEW_SESSION_CREATED":
        return "NEW_SESSION_CREATED"
    case "EXISTING_SESSION_REUSED":
        return "EXISTING_SESSION_REUSED"
    case "EXISTING_SESSION_ATTACHED":
        return "EXISTING_SESSION_ATTACHED"
    case "RETRY_REUSED_RESULT":
        return "RETRY_REUSED_RESULT"
    default:
        return nil
    }
}

private func canonicalActiveSessionState(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    let normalized = rawValue.trimmingCharacters(in: .whitespacesAndNewlines).uppercased()
    guard normalized == "ACTIVE" else {
        return nil
    }

    return "SessionState::Active"
}

private func canonicalSoftClosedSessionState(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    let normalized = rawValue.trimmingCharacters(in: .whitespacesAndNewlines).uppercased()
    guard normalized == "SOFT_CLOSED" else {
        return nil
    }

    return "SessionState::SoftClosed"
}

private func canonicalSuspendedSessionState(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    let normalized = rawValue.trimmingCharacters(in: .whitespacesAndNewlines).uppercased()
    guard normalized == "SUSPENDED" else {
        return nil
    }

    return "SessionState::Suspended"
}

private func canonicalResumeTier(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    switch rawValue.trimmingCharacters(in: .whitespacesAndNewlines).uppercased() {
    case "HOT":
        return "MemoryResumeTier::Hot"
    case "WARM":
        return "MemoryResumeTier::Warm"
    case "COLD":
        return "MemoryResumeTier::Cold"
    default:
        return nil
    }
}

private func canonicalBoolean(_ rawValue: String?) -> Bool? {
    guard let rawValue else {
        return nil
    }

    switch rawValue.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() {
    case "true":
        return true
    case "false":
        return false
    default:
        return nil
    }
}

private func booleanValue(_ value: Bool) -> String {
    value ? "true" : "false"
}

private enum CanonicalReturnCheckResponse: String, CaseIterable, Identifiable {
    case yes = "Yes"
    case no = "No"

    var id: String {
        rawValue
    }

    var canonicalValue: String {
        switch self {
        case .yes:
            return "ConfirmAnswer::Yes"
        case .no:
            return "ConfirmAnswer::No"
        }
    }
}

private enum InterruptContinuityResponseKind: String {
    case clarifyDirective = "clarify_directive"
    case returnCheckResponse = "return_check_response"
}

private struct InterruptContinuityResponseRequestState: Identifiable, Equatable {
    let id: String
    let kind: InterruptContinuityResponseKind
    let responseLabel: String
    let canonicalValue: String
    let sessionID: String
    let turnID: String

    var title: String {
        switch kind {
        case .clarifyDirective:
            return "Pending clarify-directive response"
        case .returnCheckResponse:
            return "Pending return-check response"
        }
    }

    var summary: String {
        "Awaiting authoritative interruption continuity response."
    }

    var detail: String {
        let responseDetail: String

        switch kind {
        case .clarifyDirective:
            responseDetail = "Clarify directive response: \(responseLabel)."
        case .returnCheckResponse:
            responseDetail = "Return-check response: \(responseLabel) (`\(canonicalValue)`)."
        }

        return "Bounded continuity response production only. \(responseDetail) Session `\(sessionID)` turn `\(turnID)` remains non-authoritative until canonical follow-up occurs, and this shell does not invent local interrupt law, fake resume authority, or silent discard."
    }
}

private struct InterruptContinuityResponseFailureState: Identifiable, Equatable {
    let id: String
    let title: String
    let summary: String
    let detail: String
}

private enum DesktopRecoveryDisplayState: String, Equatable {
    case recovering = "RECOVERING"
    case degradedRecovery = "DEGRADED_RECOVERY"
    case quarantinedLocalState = "QUARANTINED_LOCAL_STATE"
}

private enum DesktopInterruptDisplayState: String, Equatable {
    case interruptVisible = "INTERRUPT_VISIBLE"
}

private func resolvedRecoveryDisplayState(
    recoveryMode: CanonicalRecoveryMode?,
    reconciliationDecision: CanonicalReconciliationDecision?
) -> DesktopRecoveryDisplayState? {
    if recoveryMode == .quarantinedLocalState || reconciliationDecision == .quarantineLocalState {
        return .quarantinedLocalState
    }

    switch recoveryMode {
    case .recovering:
        return .recovering
    case .degradedRecovery:
        return .degradedRecovery
    default:
        return nil
    }
}

private func resolvedInterruptDisplayState(
    interruptSubjectRelation: CanonicalInterruptSubjectRelation?,
    interruptContinuityOutcome: CanonicalInterruptContinuityOutcome?,
    interruptResumePolicy: CanonicalInterruptResumePolicy?
) -> DesktopInterruptDisplayState? {
    if interruptSubjectRelation == .uncertain {
        return .interruptVisible
    }

    if interruptContinuityOutcome == .switchTopicThenReturnCheck {
        return .interruptVisible
    }

    if interruptResumePolicy == .resumeLater {
        return .interruptVisible
    }

    return nil
}

private func recoveryPostureRowsForVisibleSession(
    sessionState: String,
    sessionID: String,
    recoveryMode: CanonicalRecoveryMode?,
    reconciliationDecision: CanonicalReconciliationDecision?
) -> [(label: String, value: String)] {
    var rows: [(label: String, value: String)] = [
        ("session_state", sessionState),
        ("session_id", sessionID),
    ]

    if let recoveryMode {
        rows.append(("recovery_mode", recoveryMode.rawValue))
    }

    if let reconciliationDecision {
        rows.append(("reconciliation_decision", reconciliationDecision.rawValue))
    }

    return rows
}

private struct DesktopSessionHeaderContext: Equatable {
    let sessionState: String
    let sessionID: String
    let sessionAttachOutcome: String
    let recoveryMode: CanonicalRecoveryMode?
    let reconciliationDecision: CanonicalReconciliationDecision?

    init(
        sessionState: String,
        sessionID: String,
        sessionAttachOutcome: String,
        recoveryMode: CanonicalRecoveryMode? = nil,
        reconciliationDecision: CanonicalReconciliationDecision? = nil
    ) {
        self.sessionState = sessionState
        self.sessionID = sessionID
        self.sessionAttachOutcome = sessionAttachOutcome
        self.recoveryMode = recoveryMode
        self.reconciliationDecision = reconciliationDecision
    }

    init?(url: URL) {
        guard let components = URLComponents(url: url, resolvingAgainstBaseURL: false) else {
            return nil
        }

        let queryItems = components.queryItems ?? []
        guard
            let sessionState = boundedHint(
                firstQueryValue(in: queryItems, name: "session_state")
            ),
            let sessionID = boundedHint(
                firstQueryValue(in: queryItems, name: "session_id")
            ),
            let sessionAttachOutcome = canonicalSessionAttachOutcome(
                firstQueryValue(in: queryItems, name: "session_attach_outcome")
            )
        else {
            return nil
        }

        self.sessionState = sessionState
        self.sessionID = sessionID
        self.sessionAttachOutcome = sessionAttachOutcome
        self.recoveryMode = CanonicalRecoveryMode.parse(
            firstQueryValue(in: queryItems, name: "recovery_mode")
        )
        self.reconciliationDecision = CanonicalReconciliationDecision.parse(
            firstQueryValue(in: queryItems, name: "reconciliation_decision")
        )
    }
}

private struct DesktopSessionActiveVisibleContext: Equatable {
    let sessionState: String
    let sessionID: String
    let turnID: String
    let currentUserTurnText: String
    let currentSeleneTurnText: String
    let currentGovernedOutputSummary: String
    let sessionAttachOutcome: String?
    let recoveryMode: CanonicalRecoveryMode?
    let reconciliationDecision: CanonicalReconciliationDecision?
    let interruptSubjectRelation: CanonicalInterruptSubjectRelation?
    let interruptContinuityOutcome: CanonicalInterruptContinuityOutcome?
    let interruptResumePolicy: CanonicalInterruptResumePolicy?
    let returnCheckPending: Bool?
    let interruptClarifyQuestion: String?
    let interruptClarifyWhatIsMissing: String?
    let interruptClarifyAmbiguityFlags: [String]
    let interruptClarifyRoutingHints: [String]
    let interruptAcceptedAnswerFormats: [String]

    init?(url: URL) {
        guard let components = URLComponents(url: url, resolvingAgainstBaseURL: false) else {
            return nil
        }

        let queryItems = components.queryItems ?? []
        guard
            let sessionState = canonicalActiveSessionState(
                firstQueryValue(in: queryItems, name: "session_state")
            ),
            let sessionID = boundedHint(
                firstQueryValue(in: queryItems, name: "session_id")
            ),
            let turnID = boundedHint(
                firstQueryValue(in: queryItems, name: "turn_id")
            ),
            let currentUserTurnText = boundedTranscript(
                firstQueryValue(in: queryItems, name: "current_user_turn_text")
            ),
            let currentSeleneTurnText = boundedTranscript(
                firstQueryValue(in: queryItems, name: "current_selene_turn_text")
            ),
            let currentGovernedOutputSummary = boundedSummary(
                firstQueryValue(in: queryItems, name: "current_governed_output_summary")
            )
        else {
            return nil
        }

        self.sessionState = sessionState
        self.sessionID = sessionID
        self.turnID = turnID
        self.currentUserTurnText = currentUserTurnText
        self.currentSeleneTurnText = currentSeleneTurnText
        self.currentGovernedOutputSummary = currentGovernedOutputSummary
        self.sessionAttachOutcome = canonicalSessionAttachOutcome(
            firstQueryValue(in: queryItems, name: "session_attach_outcome")
        )
        self.recoveryMode = CanonicalRecoveryMode.parse(
            firstQueryValue(in: queryItems, name: "recovery_mode")
        )
        self.reconciliationDecision = CanonicalReconciliationDecision.parse(
            firstQueryValue(in: queryItems, name: "reconciliation_decision")
        )
        self.interruptSubjectRelation = CanonicalInterruptSubjectRelation.parse(
            firstQueryValue(in: queryItems, name: "interrupt_subject_relation")
        )
        self.interruptContinuityOutcome = CanonicalInterruptContinuityOutcome.parse(
            firstQueryValue(in: queryItems, name: "interrupt_continuity_outcome")
        )
        self.interruptResumePolicy = CanonicalInterruptResumePolicy.parse(
            firstQueryValue(in: queryItems, name: "interrupt_resume_policy")
        )
        self.returnCheckPending = canonicalBoolean(
            firstQueryValue(in: queryItems, name: "return_check_pending")
        )
        self.interruptClarifyQuestion = boundedClarifyQuestion(
            firstQueryValue(in: queryItems, name: "interrupt_clarify_question")
        )
        self.interruptClarifyWhatIsMissing = collectedInterruptClarifyWhatIsMissing(in: queryItems)
        self.interruptClarifyAmbiguityFlags = collectedInterruptClarifyAmbiguityFlags(in: queryItems)
        self.interruptClarifyRoutingHints = collectedInterruptClarifyRoutingHints(in: queryItems)
        self.interruptAcceptedAnswerFormats = collectedInterruptAcceptedAnswerFormats(in: queryItems)
    }

    var interruptContinuityRows: [(label: String, value: String)] {
        var rows: [(label: String, value: String)] = []

        if let interruptSubjectRelation {
            rows.append(("interrupt_subject_relation", interruptSubjectRelation.rawValue))
        }

        if let interruptContinuityOutcome {
            rows.append(("interrupt_continuity_outcome", interruptContinuityOutcome.rawValue))
        }

        if let interruptResumePolicy {
            rows.append(("interrupt_resume_policy", interruptResumePolicy.rawValue))
        }

        return rows
    }

    var acceptedInterruptPostureSummary: String {
        if interruptSubjectRelation == .uncertain {
            return "Clarify before continuing remains the lawful cloud-authored posture until subject relation becomes certain again."
        }

        if interruptContinuityOutcome == .switchTopicThenReturnCheck {
            return "Switch topic remains lawful now while authoritative continuity keeps a later return check cloud-side."
        }

        if interruptResumePolicy == .resumeLater {
            return "Resume later remains the lawful cloud-authored posture for the interrupted topic while the current active session stays visible."
        }

        return "Continue previous topic remains lawful only while cloud-authored continuity keeps the same active subject visible."
    }

    var hasLawfulInterruptClarifyDirective: Bool {
        interruptClarifyQuestion != nil && (2...3).contains(interruptAcceptedAnswerFormats.count)
    }

    var hasLawfulInterruptClarifyMissingField: Bool {
        hasLawfulInterruptClarifyDirective && interruptClarifyWhatIsMissing != nil
    }

    var hasLawfulInterruptClarifyAmbiguityFlags: Bool {
        (1...2).contains(interruptClarifyAmbiguityFlags.count)
    }

    var hasLawfulInterruptClarifyRoutingHints: Bool {
        (1...2).contains(interruptClarifyRoutingHints.count)
    }

    var hasInterruptResponseConflict: Bool {
        hasLawfulInterruptClarifyDirective && returnCheckPending == true
    }

    var hasInterruptResponseProductionSurface: Bool {
        hasLawfulInterruptClarifyDirective || returnCheckPending == true
    }
}

private struct DesktopSessionSoftClosedVisibleContext: Equatable {
    let sessionState: String
    let sessionID: String
    let selectedThreadID: String?
    let selectedThreadTitle: String?
    let pendingWorkOrderID: String?
    let resumeTier: String?
    let resumeSummaryBullets: [String]
    let archivedUserTurnText: String
    let archivedSeleneTurnText: String
    let recoveryMode: CanonicalRecoveryMode?
    let reconciliationDecision: CanonicalReconciliationDecision?

    init?(url: URL) {
        guard let components = URLComponents(url: url, resolvingAgainstBaseURL: false) else {
            return nil
        }

        let queryItems = components.queryItems ?? []
        guard
            let sessionState = canonicalSoftClosedSessionState(
                firstQueryValue(in: queryItems, name: "session_state")
            ),
            let sessionID = boundedHint(
                firstQueryValue(in: queryItems, name: "session_id")
            ),
            let archivedUserTurnText = boundedTranscript(
                firstQueryValue(in: queryItems, name: "archived_user_turn_text")
            ),
            let archivedSeleneTurnText = boundedTranscript(
                firstQueryValue(in: queryItems, name: "archived_selene_turn_text")
            )
        else {
            return nil
        }

        self.sessionState = sessionState
        self.sessionID = sessionID
        self.selectedThreadID = boundedHint(
            firstQueryValue(in: queryItems, name: "selected_thread_id")
        )
        self.selectedThreadTitle = boundedTitle(
            firstQueryValue(in: queryItems, name: "selected_thread_title")
        )
        self.pendingWorkOrderID = boundedHint(
            firstQueryValue(in: queryItems, name: "pending_work_order_id")
        )
        self.resumeTier = canonicalResumeTier(
            firstQueryValue(in: queryItems, name: "resume_tier")
        )
        self.resumeSummaryBullets = boundedResumeSummaryBullets(in: queryItems)
        self.archivedUserTurnText = archivedUserTurnText
        self.archivedSeleneTurnText = archivedSeleneTurnText
        self.recoveryMode = CanonicalRecoveryMode.parse(
            firstQueryValue(in: queryItems, name: "recovery_mode")
        )
        self.reconciliationDecision = CanonicalReconciliationDecision.parse(
            firstQueryValue(in: queryItems, name: "reconciliation_decision")
        )
    }
}

private struct DesktopSessionSuspendedVisibleContext: Equatable {
    let sessionState: String
    let sessionID: String
    let nextAllowedActionsMaySpeak: Bool
    let nextAllowedActionsMustWait: Bool
    let nextAllowedActionsMustRewake: Bool
    let recoveryMode: CanonicalRecoveryMode?
    let reconciliationDecision: CanonicalReconciliationDecision?

    init?(url: URL) {
        guard let components = URLComponents(url: url, resolvingAgainstBaseURL: false) else {
            return nil
        }

        let queryItems = components.queryItems ?? []
        guard
            let sessionState = canonicalSuspendedSessionState(
                firstQueryValue(in: queryItems, name: "session_state")
            ),
            let sessionID = boundedHint(
                firstQueryValue(in: queryItems, name: "session_id")
            ),
            let nextAllowedActionsMaySpeak = canonicalBoolean(
                firstQueryValue(in: queryItems, name: "next_allowed_actions_may_speak")
            ),
            let nextAllowedActionsMustWait = canonicalBoolean(
                firstQueryValue(in: queryItems, name: "next_allowed_actions_must_wait")
            ),
            let nextAllowedActionsMustRewake = canonicalBoolean(
                firstQueryValue(in: queryItems, name: "next_allowed_actions_must_rewake")
            )
        else {
            return nil
        }

        self.sessionState = sessionState
        self.sessionID = sessionID
        self.nextAllowedActionsMaySpeak = nextAllowedActionsMaySpeak
        self.nextAllowedActionsMustWait = nextAllowedActionsMustWait
        self.nextAllowedActionsMustRewake = nextAllowedActionsMustRewake
        self.recoveryMode = CanonicalRecoveryMode.parse(
            firstQueryValue(in: queryItems, name: "recovery_mode")
        )
        self.reconciliationDecision = CanonicalReconciliationDecision.parse(
            firstQueryValue(in: queryItems, name: "reconciliation_decision")
        )
    }

    var suspendedStatusRows: [(label: String, value: String)] {
        var rows: [(label: String, value: String)] = [
            ("session_state", sessionState),
            ("session_id", sessionID),
        ]

        if let recoveryMode {
            rows.append(("recovery_mode", recoveryMode.rawValue))
        }

        if let reconciliationDecision {
            rows.append(("reconciliation_decision", reconciliationDecision.rawValue))
        }

        return rows
    }

    var allowedNextStepRows: [(label: String, value: String)] {
        [
            ("next_allowed_actions_may_speak", booleanValue(nextAllowedActionsMaySpeak)),
            ("next_allowed_actions_must_wait", booleanValue(nextAllowedActionsMustWait)),
            ("next_allowed_actions_must_rewake", booleanValue(nextAllowedActionsMustRewake)),
        ]
    }

    var allowedNextStepSummary: String {
        if nextAllowedActionsMustRewake {
            return "Must re-wake through the lawful explicit-entry path before any next turn can be requested."
        }

        if nextAllowedActionsMustWait {
            return "Must wait for authoritative reread or cloud-side recovery review before any next turn can be requested."
        }

        if nextAllowedActionsMaySpeak {
            return "A later explicit next step may become lawful only after the authoritative suspended posture clears cloud-side."
        }

        return "No next turn is currently lawful from this bounded suspended surface."
    }
}

private enum DesktopRecoveryVisibleSurface {
    case sessionHeader(DesktopSessionHeaderContext)
    case sessionActive(DesktopSessionActiveVisibleContext)
    case sessionSoftClosed(DesktopSessionSoftClosedVisibleContext)

    var sessionState: String {
        switch self {
        case .sessionHeader(let context):
            return context.sessionState
        case .sessionActive(let context):
            return context.sessionState
        case .sessionSoftClosed(let context):
            return context.sessionState
        }
    }

    var sessionID: String {
        switch self {
        case .sessionHeader(let context):
            return context.sessionID
        case .sessionActive(let context):
            return context.sessionID
        case .sessionSoftClosed(let context):
            return context.sessionID
        }
    }

    var recoveryMode: CanonicalRecoveryMode? {
        switch self {
        case .sessionHeader(let context):
            return context.recoveryMode
        case .sessionActive(let context):
            return context.recoveryMode
        case .sessionSoftClosed(let context):
            return context.recoveryMode
        }
    }

    var reconciliationDecision: CanonicalReconciliationDecision? {
        switch self {
        case .sessionHeader(let context):
            return context.reconciliationDecision
        case .sessionActive(let context):
            return context.reconciliationDecision
        case .sessionSoftClosed(let context):
            return context.reconciliationDecision
        }
    }

    var sourceSurfaceTitle: String {
        switch self {
        case .sessionHeader:
            return "SESSION_OPEN_VISIBLE"
        case .sessionActive:
            return "SESSION_ACTIVE_VISIBLE"
        case .sessionSoftClosed:
            return "SESSION_SOFT_CLOSED_VISIBLE"
        }
    }

    var recoveryPostureRows: [(label: String, value: String)] {
        recoveryPostureRowsForVisibleSession(
            sessionState: sessionState,
            sessionID: sessionID,
            recoveryMode: recoveryMode,
            reconciliationDecision: reconciliationDecision
        )
    }
}

struct DesktopSessionShellView: View {
    @State private var latestSessionHeaderContext: DesktopSessionHeaderContext?
    @State private var latestSessionActiveVisibleContext: DesktopSessionActiveVisibleContext?
    @State private var latestSessionSoftClosedVisibleContext: DesktopSessionSoftClosedVisibleContext?
    @State private var latestSessionSuspendedVisibleContext: DesktopSessionSuspendedVisibleContext?
    @State private var interruptResponsePendingRequest: InterruptContinuityResponseRequestState?
    @State private var interruptResponseFailedRequest: InterruptContinuityResponseFailureState?
    @State private var interruptResponseRequestSequence: Int = 0

    var body: some View {
        HStack(alignment: .top, spacing: 20) {
            VStack(alignment: .leading, spacing: 16) {
                posturePanel

                historyCard
            }
            .frame(width: 270, alignment: .topLeading)

            VStack(alignment: .leading, spacing: 16) {
                sessionCard
                .frame(maxWidth: .infinity, minHeight: 360, alignment: .topLeading)
            }
            .frame(maxWidth: .infinity, alignment: .topLeading)

            VStack(alignment: .leading, spacing: 16) {
                systemActivityCard

                needsAttentionCard
            }
            .frame(width: 300, alignment: .topLeading)
        }
        .padding(24)
        .frame(minWidth: 1180, minHeight: 720, alignment: .topLeading)
        .background(Color(nsColor: .windowBackgroundColor))
        .onOpenURL { url in
            if let context = DesktopSessionActiveVisibleContext(url: url) {
                clearInterruptResponseState()
                latestSessionActiveVisibleContext = context
                latestSessionSoftClosedVisibleContext = nil
                latestSessionSuspendedVisibleContext = nil

                if let sessionAttachOutcome = context.sessionAttachOutcome {
                    latestSessionHeaderContext = DesktopSessionHeaderContext(
                        sessionState: context.sessionState,
                        sessionID: context.sessionID,
                        sessionAttachOutcome: sessionAttachOutcome,
                        recoveryMode: context.recoveryMode,
                        reconciliationDecision: context.reconciliationDecision
                    )
                } else if latestSessionHeaderContext?.sessionID != context.sessionID {
                    latestSessionHeaderContext = nil
                }

                return
            }

            if let context = DesktopSessionSoftClosedVisibleContext(url: url) {
                clearInterruptResponseState()
                latestSessionSoftClosedVisibleContext = context
                latestSessionActiveVisibleContext = nil
                latestSessionSuspendedVisibleContext = nil

                if latestSessionHeaderContext?.sessionID != context.sessionID {
                    latestSessionHeaderContext = nil
                }

                return
            }

            if let context = DesktopSessionSuspendedVisibleContext(url: url) {
                clearInterruptResponseState()
                latestSessionSuspendedVisibleContext = context
                latestSessionActiveVisibleContext = nil
                latestSessionSoftClosedVisibleContext = nil

                if latestSessionHeaderContext?.sessionID != context.sessionID {
                    latestSessionHeaderContext = nil
                }

                return
            }

            if let context = DesktopSessionHeaderContext(url: url) {
                clearInterruptResponseState()
                latestSessionHeaderContext = context
                latestSessionActiveVisibleContext = nil
                latestSessionSoftClosedVisibleContext = nil
                latestSessionSuspendedVisibleContext = nil
            }
        }
    }

    private var activeRecoveryVisibleSurface: DesktopRecoveryVisibleSurface? {
        guard latestSessionSuspendedVisibleContext == nil else {
            return nil
        }

        if let latestSessionActiveVisibleContext {
            return .sessionActive(latestSessionActiveVisibleContext)
        }

        if let latestSessionSoftClosedVisibleContext {
            return .sessionSoftClosed(latestSessionSoftClosedVisibleContext)
        }

        if let latestSessionHeaderContext {
            return .sessionHeader(latestSessionHeaderContext)
        }

        return nil
    }

    private var activeRecoveryDisplayState: DesktopRecoveryDisplayState? {
        guard let activeRecoveryVisibleSurface else {
            return nil
        }

        let normalizedSessionState = normalizedRecoveryEnumToken(activeRecoveryVisibleSurface.sessionState)
        guard normalizedSessionState != "suspended", normalizedSessionState != "sessionstatesuspended" else {
            return nil
        }

        return resolvedRecoveryDisplayState(
            recoveryMode: activeRecoveryVisibleSurface.recoveryMode,
            reconciliationDecision: activeRecoveryVisibleSurface.reconciliationDecision
        )
    }

    private var activeInterruptDisplayState: DesktopInterruptDisplayState? {
        guard activeRecoveryDisplayState == nil,
              latestSessionSuspendedVisibleContext == nil,
              let latestSessionActiveVisibleContext else {
            return nil
        }

        return resolvedInterruptDisplayState(
            interruptSubjectRelation: latestSessionActiveVisibleContext.interruptSubjectRelation,
            interruptContinuityOutcome: latestSessionActiveVisibleContext.interruptContinuityOutcome,
            interruptResumePolicy: latestSessionActiveVisibleContext.interruptResumePolicy
        )
    }

    private var posturePanel: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Selene Mac Desktop")
                    .font(.largeTitle.weight(.bold))

                Text("First-class, non-authority")
                    .font(.headline)

                VStack(alignment: .leading, spacing: 8) {
                    posturePill("Wake word or explicit entry")
                    posturePill("Cloud authoritative")
                    posturePill("Session-bound placeholder")
                }

                Text("Bounded desktop placeholder surface only. No local authority, proof, governance, or law behavior.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
    }

    private var sessionCard: some View {
        Group {
            if let latestSessionSuspendedVisibleContext {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored desktop suspended-session evidence only.")
                            .font(.subheadline.weight(.semibold))

                        Text("Bounded read-only suspended posture for the cloud-authoritative desktop session surface.")
                            .foregroundStyle(.secondary)

                        ForEach(latestSessionSuspendedVisibleContext.suspendedStatusRows, id: \.label) { row in
                            metadataRow(label: row.label, value: row.value)
                        }

                        Text("This suspended posture remains a hard full takeover. No local unsuspend, local reread, local retry, or local re-wake production is available here.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)

                        Text("No local authority, no local resume authoring, no local wake authority, no local governance or law execution, no local dispatch unlock, and no local attach or reopen authority are introduced by this bounded suspended surface.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                } label: {
                    Text("Session")
                        .font(.headline)
                }
            } else if activeRecoveryDisplayState == .quarantinedLocalState,
                      let activeRecoveryVisibleSurface
            {
                quarantinedLocalStateSessionCard(activeRecoveryVisibleSurface)
            } else if let latestSessionSoftClosedVisibleContext {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored desktop soft-closed evidence only.")
                            .font(.subheadline.weight(.semibold))

                        Text("Bounded read-only soft-closed session posture for the cloud-authoritative desktop session surface.")
                            .foregroundStyle(.secondary)

                        metadataRow(label: "session_state", value: latestSessionSoftClosedVisibleContext.sessionState)
                        metadataRow(label: "session_id", value: latestSessionSoftClosedVisibleContext.sessionID)

                        Text("Visual reset may clear the screen, but archive truth remains durable and the explicit resume affordance remains non-producing here.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)

                        Button("Resume the selected thread explicitly") {}
                            .buttonStyle(.borderedProminent)
                            .disabled(true)

                        Text("No local authority, no local resume authoring, no local wake authority, no local governance or law execution, no local archive fabrication, no local PH1.M synthesis, and no local attach, reopen, or thread-selection authority.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                } label: {
                    Text("Session")
                        .font(.headline)
                }
            } else if let latestSessionActiveVisibleContext {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored desktop active-session evidence only.")
                            .font(.subheadline.weight(.semibold))

                        Text("Bounded read-only current session and current turn posture for the cloud-authoritative desktop session surface.")
                            .foregroundStyle(.secondary)

                        metadataRow(label: "session_state", value: latestSessionActiveVisibleContext.sessionState)
                        metadataRow(label: "session_id", value: latestSessionActiveVisibleContext.sessionID)
                        metadataRow(label: "turn_id", value: latestSessionActiveVisibleContext.turnID)

                        if let sessionAttachOutcome = latestSessionActiveVisibleContext.sessionAttachOutcome {
                            metadataRow(label: "session_attach_outcome", value: sessionAttachOutcome)

                            Text(continuityLabel(for: sessionAttachOutcome))
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                        }

                        Text("No local authority, no local resume authoring, no local wake authority, no local governance or law execution, no local transcript or governed-output synthesis, and no local attach, reopen, or turn-production authority.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                } label: {
                    Text("Session")
                        .font(.headline)
                }
            } else if let latestSessionHeaderContext {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored desktop session-header evidence only.")
                            .font(.subheadline.weight(.semibold))

                        Text("Bounded read-only session posture for the current cloud-authoritative desktop session surface.")
                            .foregroundStyle(.secondary)

                        metadataRow(label: "session_state", value: latestSessionHeaderContext.sessionState)
                        metadataRow(label: "session_id", value: latestSessionHeaderContext.sessionID)
                        metadataRow(
                            label: "session_attach_outcome",
                            value: latestSessionHeaderContext.sessionAttachOutcome
                        )

                        Text(continuityLabel(for: latestSessionHeaderContext.sessionAttachOutcome))
                            .font(.footnote)
                            .foregroundStyle(.secondary)

                        Text("No local authority, no local resume authoring, no local wake authority, no local governance or law execution, no transcript or interruption ownership, and no local attach or reopen authority.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                } label: {
                    Text("Session")
                        .font(.headline)
                }
            } else {
                sectionCard(
                    title: "Session",
                    detail: "One dominant session surface placeholder for the cloud-authoritative Selene runtime."
                )
            }
        }
    }

    private var historyCard: some View {
        Group {
            if latestSessionSuspendedVisibleContext != nil {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored suspended-status explanation only.")
                            .font(.subheadline.weight(.semibold))

                        Text("This bounded desktop surface remains in a dominant suspended posture selected by the authoritative runtime, so live dual transcript and archived recent-slice visibility stay withheld here.")
                            .foregroundStyle(.secondary)

                        Text("Suspended posture remains explanation-only on macOS in this run: no local transcript authority, no local archive fabrication, no hidden continuation, and no local unsuspend path are introduced.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                } label: {
                    Text("History")
                        .font(.headline)
                }
            } else if activeRecoveryDisplayState == .quarantinedLocalState {
                quarantinedLocalStateHistoryCard
            } else if let latestSessionActiveVisibleContext {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored live dual-transcript evidence only.")
                            .font(.subheadline.weight(.semibold))

                        transcriptEntry(
                            speaker: "You",
                            posture: "current_user_turn_text",
                            body: latestSessionActiveVisibleContext.currentUserTurnText,
                            detail: "Current user turn remains text-visible, session-bound, and cloud-authoritative for this active desktop session."
                        )

                        transcriptEntry(
                            speaker: "Selene",
                            posture: "current_selene_turn_text",
                            body: latestSessionActiveVisibleContext.currentSeleneTurnText,
                            detail: "Current Selene turn remains text-visible and tied to the same active cloud session without a local-only transcript fork."
                        )

                        Text("No local transcript authority, no local turn synthesis, and no local dispatch unlock are introduced by this bounded desktop surface.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                } label: {
                    Text("History")
                        .font(.headline)
                }
            } else if let latestSessionSoftClosedVisibleContext {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored archived recent-slice evidence only.")
                            .font(.subheadline.weight(.semibold))

                        transcriptEntry(
                            speaker: "You",
                            posture: "archived_user_turn_text",
                            body: latestSessionSoftClosedVisibleContext.archivedUserTurnText,
                            detail: "Archived recent slice remains durable archived conversation truth and stays distinct from bounded PH1.M resume-context output."
                        )

                        transcriptEntry(
                            speaker: "Selene",
                            posture: "archived_selene_turn_text",
                            body: latestSessionSoftClosedVisibleContext.archivedSeleneTurnText,
                            detail: "Archived recent slice remains text-visible after visual reset without local auto-reopen, hidden spoken-only output, or local transcript authority."
                        )

                        Text("Archived recent slice remains distinct from PH1.M memory and stays bounded to durable archive truth only.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                } label: {
                    Text("History")
                        .font(.headline)
                }
            } else {
                sectionCard(
                    title: "History",
                    detail: "Bounded history placeholder aligned to the governed desktop session surface."
                )
            }
        }
    }

    private var systemActivityCard: some View {
        Group {
            if let latestSessionSuspendedVisibleContext {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored allowed-next-step evidence only.")
                            .font(.subheadline.weight(.semibold))

                        ForEach(latestSessionSuspendedVisibleContext.allowedNextStepRows, id: \.label) { row in
                            metadataRow(label: row.label, value: row.value)
                        }

                        Text(latestSessionSuspendedVisibleContext.allowedNextStepSummary)
                            .font(.footnote)
                            .foregroundStyle(.secondary)

                        Text("Allowed-next-step visibility remains read-only and non-producing here. No local retry, local reread, local unsuspend, or local re-wake production authority is introduced by this desktop surface.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                } label: {
                    Text("System Activity")
                        .font(.headline)
                }
            } else if activeRecoveryDisplayState == .quarantinedLocalState {
                quarantinedLocalStateSystemActivityCard
            } else if let latestSessionActiveVisibleContext {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored governed-output summary evidence only.")
                            .font(.subheadline.weight(.semibold))

                        metadataRow(
                            label: "current_governed_output_summary",
                            value: latestSessionActiveVisibleContext.currentGovernedOutputSummary
                        )

                        Text("Bounded summary only. No local governed-output synthesis, no local artifact expansion, and no local dispatch unlock authority are introduced by this desktop surface.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                } label: {
                    Text("System Activity")
                        .font(.headline)
                }
            } else if let latestSessionSoftClosedVisibleContext {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored PH1.M resume-context evidence only.")
                            .font(.subheadline.weight(.semibold))

                        ForEach([
                            ("selected_thread_id", latestSessionSoftClosedVisibleContext.selectedThreadID ?? "not_provided"),
                            ("selected_thread_title", latestSessionSoftClosedVisibleContext.selectedThreadTitle ?? "not_provided"),
                            ("pending_work_order_id", latestSessionSoftClosedVisibleContext.pendingWorkOrderID ?? "not_provided"),
                            ("resume_tier", latestSessionSoftClosedVisibleContext.resumeTier ?? "not_provided"),
                        ], id: \.0) { row in
                            metadataRow(label: row.0, value: row.1)
                        }

                        if latestSessionSoftClosedVisibleContext.resumeSummaryBullets.isEmpty {
                            Text("No bounded `resume_summary_bullets` were provided for this soft-closed preview.")
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                        } else {
                            ForEach(Array(latestSessionSoftClosedVisibleContext.resumeSummaryBullets.prefix(3).enumerated()), id: \.offset) { index, bullet in
                                HStack(alignment: .firstTextBaseline, spacing: 10) {
                                    Text("\(index + 1).")
                                        .font(.caption.weight(.semibold))
                                        .foregroundStyle(.secondary)

                                    Text(bullet)
                                        .frame(maxWidth: .infinity, alignment: .leading)
                                }
                            }
                        }

                        Text("Resume context remains bounded PH1.M output only. No local thread-selection authority, no local resume synthesis, and no local dispatch unlock authority are introduced by this desktop surface.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                } label: {
                    Text("System Activity")
                        .font(.headline)
                }
            } else {
                sectionCard(
                    title: "System Activity",
                    detail: "Read-only operational placeholder for governed sync, recovery, and alert posture."
                )
            }
        }
    }

    private var needsAttentionCard: some View {
        Group {
            if activeRecoveryDisplayState == .recovering, let activeRecoveryVisibleSurface {
                recoveryRestrictionCard(activeRecoveryVisibleSurface, state: .recovering)
            } else if activeRecoveryDisplayState == .degradedRecovery, let activeRecoveryVisibleSurface {
                recoveryRestrictionCard(activeRecoveryVisibleSurface, state: .degradedRecovery)
            } else if activeInterruptDisplayState == .interruptVisible,
                      let latestSessionActiveVisibleContext
            {
                interruptVisibleCard(latestSessionActiveVisibleContext)
            } else {
                sectionCard(
                    title: "Needs Attention",
                    detail: "Bounded actionable placeholder kept separate from transcript history."
                )
            }
        }
    }

    private func sectionCard(title: String, detail: String) -> some View {
        GroupBox {
            Text(detail)
                .frame(maxWidth: .infinity, alignment: .leading)
        } label: {
            Text(title)
                .font(.headline)
        }
    }

    private func metadataRow(label: String, value: String) -> some View {
        HStack(alignment: .firstTextBaseline, spacing: 10) {
            Text(label)
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)
            Text(value)
                .textSelection(.enabled)
        }
    }

    private func transcriptEntry(
        speaker: String,
        posture: String,
        body: String,
        detail: String
    ) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(speaker)
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)

            Text(body)
                .textSelection(.enabled)

            Text("\(posture): \(detail)")
                .font(.footnote)
                .foregroundStyle(.secondary)
        }
    }

    private func continuityLabel(for sessionAttachOutcome: String) -> String {
        switch sessionAttachOutcome {
        case "NEW_SESSION_CREATED":
            return "Continuity follows the newly created cloud session selected by the authoritative runtime."
        case "EXISTING_SESSION_REUSED":
            return "Continuity stays on the existing cloud session without creating a new local session."
        case "EXISTING_SESSION_ATTACHED":
            return "Continuity attaches to the existing cloud session already selected by the authoritative runtime."
        case "RETRY_REUSED_RESULT":
            return "Continuity stays on the existing cloud session while authoritative retry reuse remains visible."
        default:
            return "Continuity remains cloud-authoritative and session-bound."
        }
    }

    private func recoveryRestrictionCard(
        _ surface: DesktopRecoveryVisibleSurface,
        state: DesktopRecoveryDisplayState
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored recovery restriction evidence only.")
                    .font(.subheadline.weight(.semibold))

                Text("\(state.rawValue) remains bounded, read-only, and reread from canonical session transport only.")
                    .foregroundStyle(.secondary)

                metadataRow(label: "source_surface", value: surface.sourceSurfaceTitle)

                ForEach(surface.recoveryPostureRows, id: \.label) { row in
                    metadataRow(label: row.label, value: row.value)
                }

                Text("Reread authoritative state before any normal interaction is reconsidered from this bounded desktop surface.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)

                Text(recoveryRestrictionSummary(for: state))
                    .font(.footnote)
                    .foregroundStyle(.secondary)

                Text("No local authority, no local unsuspend authority, no local reread authority, no local retry authority, no local queue repair authority, no local transcript authority, no local archive fabrication, no local governed-output synthesis, no local PH1.M synthesis, no local resume-buffer synthesis, no local dispatch unlock, and no local attach or reopen authority.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        } label: {
            Text("Needs Attention")
                .font(.headline)
        }
    }

    private func quarantinedLocalStateSessionCard(_ surface: DesktopRecoveryVisibleSurface) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored quarantine recovery evidence only.")
                    .font(.subheadline.weight(.semibold))

                Text("QUARANTINED_LOCAL_STATE remains a bounded hard full takeover, so the desktop shell must reread authoritative state before any lawful interaction is reconsidered.")
                    .foregroundStyle(.secondary)

                metadataRow(label: "source_surface", value: surface.sourceSurfaceTitle)

                ForEach(surface.recoveryPostureRows, id: \.label) { row in
                    metadataRow(label: row.label, value: row.value)
                }

                Text("This quarantine posture changes visibility, not ownership. No local override, no trust in stale cache, no hidden replay, and no local attach or reopen authority are introduced here.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        } label: {
            Text("Session")
                .font(.headline)
        }
    }

    private var quarantinedLocalStateHistoryCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored quarantine explanation only.")
                    .font(.subheadline.weight(.semibold))

                Text("QUARANTINED_LOCAL_STATE withholds live dual transcript and archived recent-slice visibility from this bounded desktop surface until authoritative state is reread cloud-side.")
                    .foregroundStyle(.secondary)

                Text("No local transcript authority, no local archive fabrication, and no hidden continuation path are introduced while quarantine remains active.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        } label: {
            Text("History")
                .font(.headline)
        }
    }

    private var quarantinedLocalStateSystemActivityCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored reread-authoritative-state guidance only.")
                    .font(.subheadline.weight(.semibold))

                Text("Quarantine withholds governed-output-summary and PH1.M resume-context visibility here until authoritative state is reread and the canonical recovery posture clears cloud-side.")
                    .foregroundStyle(.secondary)

                Text("No local reread authority, no local retry authority, no local queue repair authority, no local governed-output synthesis, no local PH1.M synthesis, and no local dispatch unlock are introduced by this bounded recovery surface.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        } label: {
            Text("System Activity")
                .font(.headline)
        }
    }

    private func recoveryRestrictionSummary(for state: DesktopRecoveryDisplayState) -> String {
        switch state {
        case .recovering:
            return "Recovery remains active cloud-side, so normal interaction stays restricted while the lawful main session surface remains visible in bounded read-only posture."
        case .degradedRecovery:
            return "Degraded recovery remains active cloud-side, so normal interaction stays further restricted while the lawful main session surface remains visible in bounded read-only posture."
        case .quarantinedLocalState:
            return "Quarantine removes lawful normal interaction from this desktop surface until authoritative state is reread and the canonical recovery posture clears cloud-side."
        }
    }

    private func interruptVisibleCard(_ context: DesktopSessionActiveVisibleContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored interrupt continuity evidence only.")
                    .font(.subheadline.weight(.semibold))

                Text("INTERRUPT_VISIBLE remains bounded, read-only, active-session-bound, and cloud-authoritative while the lawful main active-session surface stays visible.")
                    .foregroundStyle(.secondary)

                Text("INTERRUPT_VISIBLE")
                    .font(.headline.monospaced())

                ForEach(context.interruptContinuityRows, id: \.label) { row in
                    metadataRow(label: row.label, value: row.value)
                }

                if let returnCheckPending = context.returnCheckPending {
                    metadataRow(
                        label: "return_check_pending",
                        value: booleanValue(returnCheckPending)
                    )
                }

                Text(context.acceptedInterruptPostureSummary)
                    .font(.footnote)
                    .foregroundStyle(.secondary)

                Text("Lawful interrupt actions remain rendered, not authored, from this bounded desktop surface.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)

                Button("Clarify before continuing") {}
                    .buttonStyle(.borderedProminent)
                    .disabled(true)

                Button("Continue previous topic") {}
                    .buttonStyle(.bordered)
                    .disabled(true)

                Button("Switch topic") {}
                    .buttonStyle(.bordered)
                    .disabled(true)

                Button("Resume later") {}
                    .buttonStyle(.bordered)
                    .disabled(true)

                if context.hasInterruptResponseProductionSurface {
                    Divider()
                    interruptResponseProductionSection(context)
                }

                Text("No local interrupt authority, no local clarify authority, no local continue / switch-topic / resume-later authority, no local resume authoring, no local wake authority, no local governance or law execution, no local transcript authority, and no local dispatch unlock are introduced by this bounded interrupt surface.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        } label: {
            Text("Needs Attention")
                .font(.headline)
        }
    }

    @ViewBuilder
    private func interruptResponseProductionSection(
        _ context: DesktopSessionActiveVisibleContext
    ) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Bounded continuity response production only")
                .font(.subheadline.weight(.semibold))

            if context.hasInterruptResponseConflict {
                Text("Authoritative interruption truth exposed both clarify-directive detail and a return check, so this shell fails closed and keeps continuity response production read-only until the cloud narrows to one lawful path.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)

                interruptClarifyDirectiveSection(context, productionEnabled: false)
                interruptReturnCheckResponseSection(context, productionEnabled: false)
            } else if context.hasLawfulInterruptClarifyDirective {
                interruptClarifyDirectiveSection(
                    context,
                    productionEnabled: interruptResponsePendingRequest == nil
                )
            } else if context.returnCheckPending == true {
                interruptReturnCheckResponseSection(
                    context,
                    productionEnabled: interruptResponsePendingRequest == nil
                )
            }

            if let interruptResponsePendingRequest {
                interruptResponsePendingRequestCard(interruptResponsePendingRequest)
            }

            if let interruptResponseFailedRequest {
                interruptResponseFailedRequestCard(interruptResponseFailedRequest)
            }
        }
    }

    private func interruptClarifyDirectiveSection(
        _ context: DesktopSessionActiveVisibleContext,
        productionEnabled: Bool
    ) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Cloud-authored clarify directive")
                .font(.subheadline.weight(.semibold))

            if let interruptClarifyQuestion = context.interruptClarifyQuestion {
                metadataRow(
                    label: "interrupt_clarify_question",
                    value: interruptClarifyQuestion
                )
            }

            Text("Accepted answer formats")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)

            ForEach(context.interruptAcceptedAnswerFormats, id: \.self) { answerFormat in
                Text(answerFormat)
                    .font(.body.monospaced())
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .textSelection(.enabled)
            }

            if let interruptClarifyWhatIsMissing = context.interruptClarifyWhatIsMissing,
               context.hasLawfulInterruptClarifyMissingField {
                interruptClarifyBoundaryCard(interruptClarifyWhatIsMissing)
            }

            if context.hasLawfulInterruptClarifyAmbiguityFlags {
                interruptClarifyAmbiguityCard(context.interruptClarifyAmbiguityFlags)
            }

            if context.hasLawfulInterruptClarifyRoutingHints {
                interruptClarifyRoutingCard(context.interruptClarifyRoutingHints)
            }

            ForEach(context.interruptAcceptedAnswerFormats, id: \.self) { answerFormat in
                if answerFormat == CanonicalInterruptAcceptedAnswerFormat.continuePreviousTopic.rawValue {
                    Button(answerFormat) {
                        submitInterruptClarifyResponse(answerFormat, context: context)
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(!productionEnabled)
                } else {
                    Button(answerFormat) {
                        submitInterruptClarifyResponse(answerFormat, context: context)
                    }
                    .buttonStyle(.bordered)
                    .disabled(!productionEnabled)
                }
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    private func interruptClarifyBoundaryCard(_ missingField: String) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Clarify boundary")
                .font(.subheadline.weight(.semibold))

            Text("One question, one missing field")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("Cloud-authored field key only")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            metadataRow(
                label: "interrupt_clarify_what_is_missing",
                value: missingField
            )

            Text("No local field inference, no multi-field bundling.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func interruptClarifyAmbiguityCard(_ ambiguityFlags: [String]) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Clarify ambiguity")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored ambiguity evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            ForEach(ambiguityFlags, id: \.self) { ambiguityFlag in
                Text(ambiguityFlag)
                    .font(.body.monospaced())
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .textSelection(.enabled)
            }

            Text("Exact cloud-authored flags only")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("No local ambiguity inference, no local rewrite.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func interruptClarifyRoutingCard(_ routingHints: [String]) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Clarify routing")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored routing evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("Routing hints")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            ForEach(routingHints, id: \.self) { routingHint in
                Text(routingHint)
                    .font(.body.monospaced())
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .textSelection(.enabled)
            }

            Text("Exact cloud-authored hints only")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("No local routing guidance, no local gate bypass.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func interruptReturnCheckResponseSection(
        _ context: DesktopSessionActiveVisibleContext,
        productionEnabled: Bool
    ) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Return-check response")
                .font(.subheadline.weight(.semibold))

            if let returnCheckPending = context.returnCheckPending {
                metadataRow(
                    label: "return_check_pending",
                    value: booleanValue(returnCheckPending)
                )
            }

            Text("Do you still want to continue the previous topic?")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            HStack(spacing: 12) {
                Button("Yes") {
                    submitInterruptReturnCheckResponse(.yes, context: context)
                }
                .buttonStyle(.borderedProminent)
                .disabled(!productionEnabled)

                Button("No") {
                    submitInterruptReturnCheckResponse(.no, context: context)
                }
                .buttonStyle(.bordered)
                .disabled(!productionEnabled)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    private func interruptResponsePendingRequestCard(
        _ request: InterruptContinuityResponseRequestState
    ) -> some View {
        interruptResponseStatusCard(
            title: request.title,
            summary: request.summary,
            detail: request.detail
        )
    }

    private func interruptResponseFailedRequestCard(
        _ request: InterruptContinuityResponseFailureState
    ) -> some View {
        interruptResponseStatusCard(
            title: request.title,
            summary: request.summary,
            detail: request.detail
        )
    }

    private func interruptResponseStatusCard(
        title: String,
        summary: String,
        detail: String
    ) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(.subheadline.weight(.semibold))

            Text(summary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text(detail)
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func clearInterruptResponseState() {
        interruptResponsePendingRequest = nil
        interruptResponseFailedRequest = nil
    }

    private func submitInterruptClarifyResponse(
        _ answerFormat: String,
        context: DesktopSessionActiveVisibleContext
    ) {
        submitInterruptResponse(
            kind: .clarifyDirective,
            responseLabel: answerFormat,
            canonicalValue: answerFormat,
            context: context
        )
    }

    private func submitInterruptReturnCheckResponse(
        _ response: CanonicalReturnCheckResponse,
        context: DesktopSessionActiveVisibleContext
    ) {
        submitInterruptResponse(
            kind: .returnCheckResponse,
            responseLabel: response.rawValue,
            canonicalValue: response.canonicalValue,
            context: context
        )
    }

    private func submitInterruptResponse(
        kind: InterruptContinuityResponseKind,
        responseLabel: String,
        canonicalValue: String,
        context: DesktopSessionActiveVisibleContext
    ) {
        guard interruptResponsePendingRequest == nil else {
            interruptResponseFailedRequest = InterruptContinuityResponseFailureState(
                id: "failed_interrupt_continuity_awaiting_authoritative_response",
                title: "Failed interruption continuity response",
                summary: "A later interruption continuity response could not be produced while the current bounded interruption continuity response is already awaiting authoritative follow-up.",
                detail: "Latest failed interruption continuity response stays visible here until canonical follow-up occurs."
            )
            return
        }

        interruptResponseRequestSequence += 1
        interruptResponsePendingRequest = InterruptContinuityResponseRequestState(
            id: String(
                format: "interrupt_continuity_response_%03d",
                interruptResponseRequestSequence
            ),
            kind: kind,
            responseLabel: responseLabel,
            canonicalValue: canonicalValue,
            sessionID: context.sessionID,
            turnID: context.turnID
        )
        interruptResponseFailedRequest = nil
    }

    private func posturePill(_ text: String) -> some View {
        Text(text)
            .font(.caption.weight(.semibold))
            .padding(.horizontal, 10)
            .padding(.vertical, 6)
            .background(Color.accentColor.opacity(0.12))
            .clipShape(Capsule())
    }
}

#Preview {
    DesktopSessionShellView()
}
