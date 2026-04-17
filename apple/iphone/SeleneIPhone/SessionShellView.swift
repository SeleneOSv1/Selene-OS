import AVFoundation
import Foundation
import Speech
import SwiftUI

final class ExplicitEntryRouter: ObservableObject {
    @Published private(set) var latestContext: ExplicitEntryContext?
    @Published private(set) var latestSessionActiveVisibleContext: SessionActiveVisibleContext?
    @Published private(set) var latestSessionSoftClosedVisibleContext: SessionSoftClosedVisibleContext?
    @Published private(set) var latestSessionSuspendedVisibleContext: SessionSuspendedVisibleContext?
    @Published private(set) var latestSessionOpenVisibleContext: SessionOpenVisibleContext?

    func receive(url: URL) {
        if let sessionActiveContext = SessionActiveVisibleContext(url: url) {
            latestSessionActiveVisibleContext = sessionActiveContext
            latestSessionSoftClosedVisibleContext = nil
            latestSessionSuspendedVisibleContext = nil
            latestSessionOpenVisibleContext = nil
            latestContext = nil
            return
        }

        if let sessionSoftClosedContext = SessionSoftClosedVisibleContext(url: url) {
            latestSessionActiveVisibleContext = nil
            latestSessionSoftClosedVisibleContext = sessionSoftClosedContext
            latestSessionSuspendedVisibleContext = nil
            latestSessionOpenVisibleContext = nil
            latestContext = nil
            return
        }

        if let sessionSuspendedContext = SessionSuspendedVisibleContext(url: url) {
            latestSessionActiveVisibleContext = nil
            latestSessionSoftClosedVisibleContext = nil
            latestSessionSuspendedVisibleContext = sessionSuspendedContext
            latestSessionOpenVisibleContext = nil
            latestContext = nil
            return
        }

        if let sessionOpenContext = SessionOpenVisibleContext(url: url) {
            latestSessionActiveVisibleContext = nil
            latestSessionSoftClosedVisibleContext = nil
            latestSessionSuspendedVisibleContext = nil
            latestSessionOpenVisibleContext = sessionOpenContext
            latestContext = nil
            return
        }

        guard let context = ExplicitEntryContext(url: url) else {
            return
        }

        latestSessionActiveVisibleContext = nil
        latestSessionSoftClosedVisibleContext = nil
        latestSessionSuspendedVisibleContext = nil
        latestSessionOpenVisibleContext = nil
        latestContext = context
    }
}

enum ShellDisplayState: String {
    case explicitEntryReady = "EXPLICIT_ENTRY_READY"
    case onboardingEntryActive = "ONBOARDING_ENTRY_ACTIVE"
    case sessionOpenVisible = "SESSION_OPEN_VISIBLE"
    case sessionActiveVisible = "SESSION_ACTIVE_VISIBLE"
    case sessionSoftClosedVisible = "SESSION_SOFT_CLOSED_VISIBLE"
    case sessionSuspendedVisible = "SESSION_SUSPENDED_VISIBLE"
    case recovering = "RECOVERING"
    case degradedRecovery = "DEGRADED_RECOVERY"
    case quarantinedLocalState = "QUARANTINED_LOCAL_STATE"
    case interruptVisible = "INTERRUPT_VISIBLE"

    var title: String {
        rawValue
    }

    var detail: String {
        switch self {
        case .explicitEntryReady:
            return "The iPhone shell is waiting for lawful explicit entry through canonical app-open / invite-open ingress."
        case .onboardingEntryActive:
            return "A lawful app-open / invite-open route has been parsed and is being rendered as a bounded onboarding-entry takeover surface with read-only onboarding outcome, onboarding_status, prompt-state, artifact/access identifier, and remaining platform-receipt context only."
        case .sessionOpenVisible:
            return "A lawful app-open route has been parsed and is being rendered as a bounded current session banner with session attach outcome continuity labeling only."
        case .sessionActiveVisible:
            return "A lawful app-open route has been parsed and is being rendered as a bounded active-session surface with live dual transcript, current turn envelope, and current governed-output summary only."
        case .sessionSoftClosedVisible:
            return "A lawful app-open route has been parsed and is being rendered as a bounded soft-closed session surface with explicit resume affordance, archived recent slice, and bounded PH1.M resume context only."
        case .sessionSuspendedVisible:
            return "A lawful app-open route has been parsed and is being rendered as a bounded suspended-session hard full takeover with suspended-status explanation and allowed next step only."
        case .recovering:
            return "A lawful app-open route has been parsed and is being rendered as a strong inline recovery restriction while the main session surface remains visible."
        case .degradedRecovery:
            return "A lawful app-open route has been parsed and is being rendered as a degraded recovery inline restriction while the main session surface remains visible."
        case .quarantinedLocalState:
            return "A lawful app-open route has been parsed and is being rendered as a hard quarantine takeover because normal interaction is not lawful."
        case .interruptVisible:
            return "A lawful app-open route has been parsed and is being rendered as a strong inline interruption continuity posture while the main session surface remains visible."
        }
    }
}

enum CanonicalRecoveryMode: String, Equatable {
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

enum CanonicalReconciliationDecision: String, Equatable {
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

private func normalizedRecoveryEnumToken(_ rawValue: String) -> String {
    rawValue
        .trimmingCharacters(in: .whitespacesAndNewlines)
        .lowercased()
        .filter { $0.isLetter || $0.isNumber }
}

enum CanonicalInterruptSubjectRelation: String, Equatable {
    case same = "InterruptSubjectRelation::Same"
    case switchTopic = "InterruptSubjectRelation::Switch"
    case uncertain = "InterruptSubjectRelation::Uncertain"

    static func parse(_ rawValue: String?) -> CanonicalInterruptSubjectRelation? {
        guard let rawValue else {
            return nil
        }

        switch normalizedInterruptEnumToken(rawValue) {
        case "same":
            return .same
        case "switch":
            return .switchTopic
        case "uncertain":
            return .uncertain
        default:
            return nil
        }
    }
}

enum CanonicalInterruptContinuityOutcome: String, Equatable {
    case sameSubjectAppend = "InterruptContinuityOutcome::SameSubjectAppend"
    case switchTopicThenReturnCheck = "InterruptContinuityOutcome::SwitchTopicThenReturnCheck"

    static func parse(_ rawValue: String?) -> CanonicalInterruptContinuityOutcome? {
        guard let rawValue else {
            return nil
        }

        switch normalizedInterruptEnumToken(rawValue) {
        case "samesubjectappend":
            return .sameSubjectAppend
        case "switchtopicthenreturncheck":
            return .switchTopicThenReturnCheck
        default:
            return nil
        }
    }
}

enum CanonicalInterruptResumePolicy: String, Equatable {
    case resumeNow = "InterruptResumePolicy::ResumeNow"
    case resumeLater = "InterruptResumePolicy::ResumeLater"
    case discard = "InterruptResumePolicy::Discard"

    static func parse(_ rawValue: String?) -> CanonicalInterruptResumePolicy? {
        guard let rawValue else {
            return nil
        }

        switch normalizedInterruptEnumToken(rawValue) {
        case "resumenow":
            return .resumeNow
        case "resumelater":
            return .resumeLater
        case "discard":
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

private enum CanonicalInterruptClarifySensitivityLevel: String, CaseIterable, Identifiable {
    case `public` = "public"
    case `private` = "private"
    case confidential = "confidential"

    var id: String {
        rawValue
    }

    static func parse(_ rawValue: String) -> CanonicalInterruptClarifySensitivityLevel? {
        Self(rawValue: rawValue.trimmingCharacters(in: .whitespacesAndNewlines))
    }
}

private enum CanonicalReturnCheckResponse: String, CaseIterable, Identifiable {
    case yes = "Yes"
    case no = "No"

    var id: String {
        rawValue
    }

    var confirmAnswerValue: String {
        switch self {
        case .yes:
            return "ConfirmAnswer::Yes"
        case .no:
            return "ConfirmAnswer::No"
        }
    }
}

private func normalizedInterruptEnumToken(_ rawValue: String) -> String {
    rawValue
        .trimmingCharacters(in: .whitespacesAndNewlines)
        .lowercased()
        .filter { $0.isLetter || $0.isNumber }
}

private func resolvedRecoveryDisplayState(
    recoveryMode: CanonicalRecoveryMode?,
    reconciliationDecision: CanonicalReconciliationDecision?
) -> ShellDisplayState? {
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
    interruptResumePolicy: CanonicalInterruptResumePolicy?,
    returnCheckPending: Bool?,
    continuityFieldsPresent: Bool
) -> ShellDisplayState? {
    if returnCheckPending == true {
        return .interruptVisible
    }

    if interruptSubjectRelation == .uncertain {
        return .interruptVisible
    }

    if interruptResumePolicy == .resumeLater {
        return .interruptVisible
    }

    if interruptContinuityOutcome == .switchTopicThenReturnCheck && continuityFieldsPresent {
        return .interruptVisible
    }

    return nil
}

private func recoveryPostureRowsForVisibleSession(
    sessionState: String,
    sessionID: String,
    recoveryMode: CanonicalRecoveryMode?,
    reconciliationDecision: CanonicalReconciliationDecision?
) -> [EntryMetadataRow] {
    var rows = [
        EntryMetadataRow(label: "session_state", value: sessionState),
        EntryMetadataRow(label: "session_id", value: sessionID),
    ]

    if let recoveryMode {
        rows.append(EntryMetadataRow(label: "recovery_mode", value: recoveryMode.rawValue))
    }

    if let reconciliationDecision {
        rows.append(
            EntryMetadataRow(
                label: "reconciliation_decision",
                value: reconciliationDecision.rawValue
            )
        )
    }

    return rows
}

enum ExplicitEntryRouteKind: String {
    case inviteOpen = "invite-open"
    case appOpen = "app-open"

    var title: String {
        switch self {
        case .inviteOpen:
            return "Invite-open takeover"
        case .appOpen:
            return "App-open takeover"
        }
    }
}

struct SessionOpenVisibleContext: Identifiable, Equatable {
    let id: String
    let sessionID: String
    let sessionState: String
    let sessionAttachOutcome: String
    let recoveryMode: CanonicalRecoveryMode?
    let reconciliationDecision: CanonicalReconciliationDecision?

    init?(url: URL) {
        guard let components = URLComponents(url: url, resolvingAgainstBaseURL: false) else {
            return nil
        }

        let scheme = (components.scheme ?? "").lowercased()
        guard ["selene", "https", "http"].contains(scheme) else {
            return nil
        }

        let host = (components.host ?? "no-host").lowercased()
        let path = components.path.isEmpty ? "/" : components.path
        let lowerPath = path.lowercased()
        let queryItems = components.queryItems ?? []
        let inviteLike = host.contains("invite") || lowerPath.contains("invite") || lowerPath.contains("onboarding")
        let appOpenLike = host.contains("open")
            || lowerPath.contains("open")
            || lowerPath.contains("entry")
            || Self.hasQueryItem(in: queryItems, name: "session_state")

        let sessionState = Self.canonicalSessionState(
            Self.firstQueryValue(in: queryItems, name: "session_state")
        )
        let sessionID = Self.boundedHint(
            Self.firstQueryValue(in: queryItems, name: "session_id")
        )
        let sessionAttachOutcome = Self.canonicalSessionAttachOutcome(
            Self.firstQueryValue(in: queryItems, name: "session_attach_outcome")
        )
        let recoveryMode = CanonicalRecoveryMode.parse(
            Self.firstQueryValue(in: queryItems, name: "recovery_mode")
        )
        let reconciliationDecision = CanonicalReconciliationDecision.parse(
            Self.firstQueryValue(in: queryItems, name: "reconciliation_decision")
        )

        guard !inviteLike,
              appOpenLike,
              let sessionState,
              let sessionID,
              let sessionAttachOutcome else {
            return nil
        }

        self.id = url.absoluteString
        self.sessionID = sessionID
        self.sessionState = sessionState
        self.sessionAttachOutcome = sessionAttachOutcome
        self.recoveryMode = recoveryMode
        self.reconciliationDecision = reconciliationDecision
    }

    var bannerRows: [EntryMetadataRow] {
        [
            EntryMetadataRow(label: "session_state", value: sessionState),
            EntryMetadataRow(label: "session_id", value: sessionID),
        ]
    }

    var attachOutcomeRows: [EntryMetadataRow] {
        [
            EntryMetadataRow(label: "session_attach_outcome", value: sessionAttachOutcome),
            EntryMetadataRow(label: "continuity_label", value: continuityLabel),
        ]
    }

    var continuityLabel: String {
        switch sessionAttachOutcome {
        case "NEW_SESSION_CREATED":
            return "Continuity follows the newly created cloud session for this current session banner."
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

    var recoveryDisplayState: ShellDisplayState? {
        resolvedRecoveryDisplayState(
            recoveryMode: recoveryMode,
            reconciliationDecision: reconciliationDecision
        )
    }

    var recoveryPostureRows: [EntryMetadataRow] {
        recoveryPostureRowsForVisibleSession(
            sessionState: sessionState,
            sessionID: sessionID,
            recoveryMode: recoveryMode,
            reconciliationDecision: reconciliationDecision
        )
    }

    private static func hasQueryItem(in queryItems: [URLQueryItem], name: String) -> Bool {
        queryItems.contains { $0.name.lowercased() == name }
    }

    private static func firstQueryValue(in queryItems: [URLQueryItem], name: String) -> String? {
        queryItems.first(where: { $0.name.lowercased() == name })?.value
    }

    private static func canonicalSessionState(_ rawValue: String?) -> String? {
        guard let rawValue else {
            return nil
        }

        let normalized = rawValue.trimmingCharacters(in: .whitespacesAndNewlines).uppercased()
        guard normalized == "OPEN" else {
            return nil
        }

        return "SessionState::Open"
    }

    private static func canonicalSessionAttachOutcome(_ rawValue: String?) -> String? {
        guard let rawValue else {
            return nil
        }

        switch rawValue.trimmingCharacters(in: .whitespacesAndNewlines).uppercased() {
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

    private static func boundedHint(_ rawValue: String?) -> String? {
        guard let rawValue, !rawValue.isEmpty else {
            return nil
        }

        if rawValue.count <= 18 {
            return rawValue
        }

        return "\(rawValue.prefix(8))...\(rawValue.suffix(4))"
    }
}

struct SessionActiveVisibleContext: Identifiable, Equatable {
    let id: String
    let sessionID: String
    let sessionState: String
    let turnID: String
    let currentUserTurnText: String
    let currentSeleneTurnText: String
    let currentGovernedOutputSummary: String
    let recoveryMode: CanonicalRecoveryMode?
    let reconciliationDecision: CanonicalReconciliationDecision?
    let interruptSubjectRelation: CanonicalInterruptSubjectRelation?
    let interruptContinuityOutcome: CanonicalInterruptContinuityOutcome?
    let interruptResumePolicy: CanonicalInterruptResumePolicy?
    let activeSubjectRef: String?
    let interruptedSubjectRef: String?
    let returnCheckPending: Bool?
    let returnCheckExpiresAt: String?
    let resumeBufferLive: Bool?
    let resumeBufferAnswerID: String?
    let resumeBufferSpokenPrefix: String?
    let resumeBufferUnsaidRemainder: String?
    let resumeBufferTopicHint: String?
    let ttsResumeSnapshotAnswerID: String?
    let ttsResumeSnapshotSpokenCursorByte: String?
    let ttsResumeSnapshotResponseText: String?
    let ttsResumeSnapshotTopicHint: String?
    let interruptClarifyQuestion: String?
    let interruptClarifyWhatIsMissing: String?
    let interruptClarifyAmbiguityFlags: [String]
    let interruptClarifyRoutingHints: [String]
    let interruptClarifyRequiresConfirmation: Bool?
    let interruptClarifySensitivityLevel: String?
    let interruptSubjectRelationConfidence: String?
    let interruptAcceptedAnswerFormats: [String]

    init?(url: URL) {
        guard let components = URLComponents(url: url, resolvingAgainstBaseURL: false) else {
            return nil
        }

        let scheme = (components.scheme ?? "").lowercased()
        guard ["selene", "https", "http"].contains(scheme) else {
            return nil
        }

        let host = (components.host ?? "no-host").lowercased()
        let path = components.path.isEmpty ? "/" : components.path
        let lowerPath = path.lowercased()
        let queryItems = components.queryItems ?? []
        let inviteLike = host.contains("invite") || lowerPath.contains("invite") || lowerPath.contains("onboarding")
        let appOpenLike = host.contains("open")
            || lowerPath.contains("open")
            || lowerPath.contains("entry")
            || Self.hasQueryItem(in: queryItems, name: "session_state")

        let sessionState = Self.canonicalSessionState(
            Self.firstQueryValue(in: queryItems, name: "session_state")
        )
        let sessionID = Self.boundedHint(
            Self.firstQueryValue(in: queryItems, name: "session_id")
        )
        let turnID = Self.boundedHint(
            Self.firstQueryValue(in: queryItems, name: "turn_id")
        )
        let currentUserTurnText = Self.boundedTranscript(
            Self.firstQueryValue(in: queryItems, name: "current_user_turn_text")
        )
        let currentSeleneTurnText = Self.boundedTranscript(
            Self.firstQueryValue(in: queryItems, name: "current_selene_turn_text")
        )
        let currentGovernedOutputSummary = Self.boundedSummary(
            Self.firstQueryValue(in: queryItems, name: "current_governed_output_summary")
        )
        let recoveryMode = CanonicalRecoveryMode.parse(
            Self.firstQueryValue(in: queryItems, name: "recovery_mode")
        )
        let reconciliationDecision = CanonicalReconciliationDecision.parse(
            Self.firstQueryValue(in: queryItems, name: "reconciliation_decision")
        )
        let interruptSubjectRelation = CanonicalInterruptSubjectRelation.parse(
            Self.firstQueryValue(in: queryItems, name: "interrupt_subject_relation")
        )
        let interruptContinuityOutcome = CanonicalInterruptContinuityOutcome.parse(
            Self.firstQueryValue(in: queryItems, name: "interrupt_continuity_outcome")
        )
        let interruptResumePolicy = CanonicalInterruptResumePolicy.parse(
            Self.firstQueryValue(in: queryItems, name: "interrupt_resume_policy")
        )
        let activeSubjectRef = Self.boundedHint(
            Self.firstQueryValue(in: queryItems, name: "active_subject_ref")
        )
        let interruptedSubjectRef = Self.boundedHint(
            Self.firstQueryValue(in: queryItems, name: "interrupted_subject_ref")
        )
        let returnCheckPending = Self.canonicalBoolean(
            Self.firstQueryValue(in: queryItems, name: "return_check_pending")
        )
        let returnCheckExpiresAt = Self.boundedContinuityDetail(
            Self.firstQueryValue(in: queryItems, name: "return_check_expires_at")
        )
        let resumeBufferLive = Self.canonicalBoolean(
            Self.firstQueryValue(in: queryItems, name: "resume_buffer_live")
        )
        let resumeBufferAnswerID = Self.boundedHint(
            Self.firstQueryValue(in: queryItems, name: "resume_buffer_answer_id")
        )
        let resumeBufferSpokenPrefix = Self.boundedTranscript(
            Self.firstQueryValue(in: queryItems, name: "resume_buffer_spoken_prefix")
        )
        let resumeBufferUnsaidRemainder = Self.boundedTranscript(
            Self.firstQueryValue(in: queryItems, name: "resume_buffer_unsaid_remainder")
        )
        let resumeBufferTopicHint = Self.boundedContinuityDetail(
            Self.firstQueryValue(in: queryItems, name: "resume_buffer_topic_hint")
        )
        let ttsResumeSnapshotAnswerID = Self.boundedHint(
            Self.firstQueryValue(in: queryItems, name: "tts_resume_snapshot_answer_id")
        )
        let ttsResumeSnapshotSpokenCursorByte = Self.boundedContinuityDetail(
            Self.firstQueryValue(in: queryItems, name: "tts_resume_snapshot_spoken_cursor_byte")
        )
        let ttsResumeSnapshotResponseText = Self.boundedTranscript(
            Self.firstQueryValue(in: queryItems, name: "tts_resume_snapshot_response_text")
        )
        let ttsResumeSnapshotTopicHint = Self.boundedContinuityDetail(
            Self.firstQueryValue(in: queryItems, name: "tts_resume_snapshot_topic_hint")
        )
        let interruptClarifyQuestion = Self.boundedClarifyQuestion(
            Self.firstQueryValue(in: queryItems, name: "interrupt_clarify_question")
        )
        let interruptClarifyWhatIsMissing = Self.interruptClarifyWhatIsMissing(in: queryItems)
        let interruptClarifyAmbiguityFlags = Self.interruptClarifyAmbiguityFlags(in: queryItems)
        let interruptClarifyRoutingHints = Self.interruptClarifyRoutingHints(in: queryItems)
        let interruptClarifyRequiresConfirmation = Self.interruptClarifyRequiresConfirmation(
            in: queryItems
        )
        let interruptClarifySensitivityLevel = Self.interruptClarifySensitivityLevel(
            in: queryItems
        )
        let interruptSubjectRelationConfidence = Self.interruptSubjectRelationConfidence(
            in: queryItems
        )
        let interruptAcceptedAnswerFormats = Self.interruptAcceptedAnswerFormats(in: queryItems)

        guard !inviteLike,
              appOpenLike,
              let sessionState,
              let sessionID,
              let turnID,
              let currentUserTurnText,
              let currentSeleneTurnText,
              let currentGovernedOutputSummary else {
            return nil
        }

        self.id = url.absoluteString
        self.sessionID = sessionID
        self.sessionState = sessionState
        self.turnID = turnID
        self.currentUserTurnText = currentUserTurnText
        self.currentSeleneTurnText = currentSeleneTurnText
        self.currentGovernedOutputSummary = currentGovernedOutputSummary
        self.recoveryMode = recoveryMode
        self.reconciliationDecision = reconciliationDecision
        self.interruptSubjectRelation = interruptSubjectRelation
        self.interruptContinuityOutcome = interruptContinuityOutcome
        self.interruptResumePolicy = interruptResumePolicy
        self.activeSubjectRef = activeSubjectRef
        self.interruptedSubjectRef = interruptedSubjectRef
        self.returnCheckPending = returnCheckPending
        self.returnCheckExpiresAt = returnCheckExpiresAt
        self.resumeBufferLive = resumeBufferLive
        self.resumeBufferAnswerID = resumeBufferAnswerID
        self.resumeBufferSpokenPrefix = resumeBufferSpokenPrefix
        self.resumeBufferUnsaidRemainder = resumeBufferUnsaidRemainder
        self.resumeBufferTopicHint = resumeBufferTopicHint
        self.ttsResumeSnapshotAnswerID = ttsResumeSnapshotAnswerID
        self.ttsResumeSnapshotSpokenCursorByte = ttsResumeSnapshotSpokenCursorByte
        self.ttsResumeSnapshotResponseText = ttsResumeSnapshotResponseText
        self.ttsResumeSnapshotTopicHint = ttsResumeSnapshotTopicHint
        self.interruptClarifyQuestion = interruptClarifyQuestion
        self.interruptClarifyWhatIsMissing = interruptClarifyWhatIsMissing
        self.interruptClarifyAmbiguityFlags = interruptClarifyAmbiguityFlags
        self.interruptClarifyRoutingHints = interruptClarifyRoutingHints
        self.interruptClarifyRequiresConfirmation = interruptClarifyRequiresConfirmation
        self.interruptClarifySensitivityLevel = interruptClarifySensitivityLevel
        self.interruptSubjectRelationConfidence = interruptSubjectRelationConfidence
        self.interruptAcceptedAnswerFormats = interruptAcceptedAnswerFormats
    }

    var liveTranscriptEntries: [RecentThreadPreviewEntry] {
        [
            RecentThreadPreviewEntry(
                speaker: "You",
                posture: "current_user_turn_text",
                body: currentUserTurnText,
                detail: "Current user turn remains text-visible inside the append-only `conversation_ledger` even when it began as explicit voice."
            ),
            RecentThreadPreviewEntry(
                speaker: "Selene",
                posture: "current_selene_turn_text",
                body: currentSeleneTurnText,
                detail: "Current Selene turn remains text-visible, cloud-authoritative, and tied to the same active session without a local-only transcript fork."
            ),
        ]
    }

    var currentTurnEnvelopeRows: [EntryMetadataRow] {
        [
            EntryMetadataRow(label: "session_state", value: sessionState),
            EntryMetadataRow(label: "session_id", value: sessionID),
            EntryMetadataRow(label: "turn_id", value: turnID),
            EntryMetadataRow(label: "runtime_execution_envelope", value: "current_turn_visible"),
            EntryMetadataRow(label: "governance_state", value: "cloud_authoritative_visible"),
            EntryMetadataRow(label: "proof_state", value: "cloud_authoritative_visible"),
            EntryMetadataRow(label: "computation_state", value: "cloud_authoritative_visible"),
            EntryMetadataRow(label: "identity_state", value: "cloud_authoritative_visible"),
            EntryMetadataRow(label: "memory_state", value: "append_only_conversation_ledger_visible"),
            EntryMetadataRow(label: "authority_state", value: "cloud_authoritative_visible"),
            EntryMetadataRow(label: "artifact_trust_state", value: "cloud_authoritative_visible"),
        ]
    }

    var governedOutputRows: [EntryMetadataRow] {
        [
            EntryMetadataRow(label: "current_governed_output_summary", value: currentGovernedOutputSummary),
            EntryMetadataRow(label: "governed_content_mode", value: "bounded_summary_only"),
            EntryMetadataRow(label: "artifact_loading", value: "explicit_open_required"),
        ]
    }

    var recoveryDisplayState: ShellDisplayState? {
        resolvedRecoveryDisplayState(
            recoveryMode: recoveryMode,
            reconciliationDecision: reconciliationDecision
        )
    }

    var recoveryPostureRows: [EntryMetadataRow] {
        recoveryPostureRowsForVisibleSession(
            sessionState: sessionState,
            sessionID: sessionID,
            recoveryMode: recoveryMode,
            reconciliationDecision: reconciliationDecision
        )
    }

    var interruptDisplayState: ShellDisplayState? {
        resolvedInterruptDisplayState(
            interruptSubjectRelation: interruptSubjectRelation,
            interruptContinuityOutcome: interruptContinuityOutcome,
            interruptResumePolicy: interruptResumePolicy,
            returnCheckPending: returnCheckPending,
            continuityFieldsPresent: hasInterruptContinuityFieldsPresent
        )
    }

    var interruptContinuityRows: [EntryMetadataRow] {
        var rows = [
            EntryMetadataRow(label: "session_state", value: sessionState),
            EntryMetadataRow(label: "session_id", value: sessionID),
        ]

        if let interruptSubjectRelation {
            rows.append(
                EntryMetadataRow(
                    label: "interrupt_subject_relation",
                    value: interruptSubjectRelation.rawValue
                )
            )
        }

        if let interruptContinuityOutcome {
            rows.append(
                EntryMetadataRow(
                    label: "interrupt_continuity_outcome",
                    value: interruptContinuityOutcome.rawValue
                )
            )
        }

        if let interruptResumePolicy {
            rows.append(
                EntryMetadataRow(
                    label: "interrupt_resume_policy",
                    value: interruptResumePolicy.rawValue
                )
            )
        }

        if let activeSubjectRef {
            rows.append(EntryMetadataRow(label: "active_subject_ref", value: activeSubjectRef))
        }

        if let interruptedSubjectRef {
            rows.append(
                EntryMetadataRow(label: "interrupted_subject_ref", value: interruptedSubjectRef)
            )
        }

        if let returnCheckPending {
            rows.append(
                EntryMetadataRow(
                    label: "return_check_pending",
                    value: Self.booleanValue(returnCheckPending)
                )
            )
        }

        if let returnCheckExpiresAt {
            rows.append(
                EntryMetadataRow(label: "return_check_expires_at", value: returnCheckExpiresAt)
            )
        }

        if let resumeBufferLive {
            rows.append(
                EntryMetadataRow(
                    label: "resume_buffer_live",
                    value: Self.booleanValue(resumeBufferLive)
                )
            )
        }

        if let resumeBufferAnswerID {
            rows.append(
                EntryMetadataRow(label: "resume_buffer_answer_id", value: resumeBufferAnswerID)
            )
        }

        if let resumeBufferSpokenPrefix {
            rows.append(
                EntryMetadataRow(
                    label: "resume_buffer_spoken_prefix",
                    value: resumeBufferSpokenPrefix
                )
            )
        }

        if let resumeBufferUnsaidRemainder {
            rows.append(
                EntryMetadataRow(
                    label: "resume_buffer_unsaid_remainder",
                    value: resumeBufferUnsaidRemainder
                )
            )
        }

        if let resumeBufferTopicHint {
            rows.append(
                EntryMetadataRow(label: "resume_buffer_topic_hint", value: resumeBufferTopicHint)
            )
        }

        if let ttsResumeSnapshotAnswerID {
            rows.append(
                EntryMetadataRow(
                    label: "tts_resume_snapshot_answer_id",
                    value: ttsResumeSnapshotAnswerID
                )
            )
        }

        if let ttsResumeSnapshotSpokenCursorByte {
            rows.append(
                EntryMetadataRow(
                    label: "tts_resume_snapshot_spoken_cursor_byte",
                    value: ttsResumeSnapshotSpokenCursorByte
                )
            )
        }

        if let ttsResumeSnapshotResponseText {
            rows.append(
                EntryMetadataRow(
                    label: "tts_resume_snapshot_response_text",
                    value: ttsResumeSnapshotResponseText
                )
            )
        }

        if let ttsResumeSnapshotTopicHint {
            rows.append(
                EntryMetadataRow(
                    label: "tts_resume_snapshot_topic_hint",
                    value: ttsResumeSnapshotTopicHint
                )
            )
        }

        return rows
    }

    var acceptedInterruptPostureSummary: String {
        if returnCheckPending == true {
            return "Clarify before continuing remains the lawful cloud-authored posture while the active session stays visible and the return check remains pending."
        }

        if interruptSubjectRelation == .uncertain {
            return "Clarify before continuing remains the lawful cloud-authored posture until subject relation becomes certain again."
        }

        if interruptContinuityOutcome == .switchTopicThenReturnCheck {
            return "Switch topic remains lawful now while authoritative continuity keeps a later return check for the interrupted topic."
        }

        if interruptResumePolicy == .resumeLater {
            return "Resume later remains the lawful cloud-authored posture for the interrupted topic while the current active session stays visible."
        }

        return "Continue previous topic remains lawful only when cloud-authored interruption continuity keeps the same subject active."
    }

    var shouldPromptReturnCheck: Bool {
        returnCheckPending == true || interruptContinuityOutcome == .switchTopicThenReturnCheck
    }

    var hasLawfulInterruptClarifyDirective: Bool {
        interruptClarifyQuestion != nil && (2...3).contains(interruptAcceptedAnswerFormats.count)
    }

    var hasLawfulInterruptClarifyAmbiguityFlags: Bool {
        (1...2).contains(interruptClarifyAmbiguityFlags.count)
    }

    var hasLawfulInterruptClarifyRoutingHints: Bool {
        (1...2).contains(interruptClarifyRoutingHints.count)
    }

    var hasLawfulInterruptClarifyRequiresConfirmation: Bool {
        interruptClarifyRequiresConfirmation != nil
    }

    var hasLawfulInterruptClarifySensitivityLevel: Bool {
        interruptClarifySensitivityLevel != nil
    }

    var hasLawfulInterruptSubjectReferences: Bool {
        interruptSubjectRelation != nil
            && (activeSubjectRef != nil || interruptedSubjectRef != nil)
    }

    var hasLawfulInterruptReturnCheckExpiry: Bool {
        returnCheckPending == true && returnCheckExpiresAt != nil
    }

    var hasLawfulInterruptResumeBufferLive: Bool {
        resumeBufferLive == true
    }

    var hasLawfulInterruptResumeBufferAnswerID: Bool {
        resumeBufferAnswerID != nil
    }

    var hasLawfulInterruptResumeBufferSpokenPrefix: Bool {
        resumeBufferLive == true && resumeBufferSpokenPrefix != nil
    }

    var hasLawfulInterruptResumeBufferUnsaidRemainder: Bool {
        resumeBufferLive == true && resumeBufferUnsaidRemainder != nil
    }

    var hasLawfulInterruptResumeBufferTopicHint: Bool {
        resumeBufferLive == true && resumeBufferTopicHint != nil
    }

    var hasLawfulInterruptTtsResumeSnapshotAnswerID: Bool {
        ttsResumeSnapshotAnswerID != nil
    }

    var hasLawfulInterruptTtsResumeSnapshotSpokenCursorByte: Bool {
        resumeBufferLive == true && ttsResumeSnapshotSpokenCursorByte != nil
    }

    var hasLawfulInterruptTtsResumeSnapshotResponseText: Bool {
        ttsResumeSnapshotAnswerID != nil
            && ttsResumeSnapshotSpokenCursorByte != nil
            && ttsResumeSnapshotResponseText != nil
    }

    var hasLawfulInterruptTtsResumeSnapshotTopicHint: Bool {
        ttsResumeSnapshotAnswerID != nil
            && ttsResumeSnapshotSpokenCursorByte != nil
            && ttsResumeSnapshotResponseText != nil
            && ttsResumeSnapshotTopicHint != nil
    }

    var hasLawfulInterruptSubjectRelationConfidence: Bool {
        interruptSubjectRelation != nil && interruptSubjectRelationConfidence != nil
    }

    var hasInterruptResponseConflict: Bool {
        hasLawfulInterruptClarifyDirective && returnCheckPending == true
    }

    var hasInterruptResponseProductionSurface: Bool {
        hasLawfulInterruptClarifyDirective || returnCheckPending == true
    }

    private var hasInterruptContinuityFieldsPresent: Bool {
        activeSubjectRef != nil
            || interruptedSubjectRef != nil
            || returnCheckPending != nil
            || returnCheckExpiresAt != nil
            || resumeBufferLive != nil
            || resumeBufferAnswerID != nil
            || resumeBufferSpokenPrefix != nil
            || resumeBufferUnsaidRemainder != nil
            || resumeBufferTopicHint != nil
            || ttsResumeSnapshotAnswerID != nil
            || ttsResumeSnapshotSpokenCursorByte != nil
            || ttsResumeSnapshotResponseText != nil
            || ttsResumeSnapshotTopicHint != nil
    }

    private static func hasQueryItem(in queryItems: [URLQueryItem], name: String) -> Bool {
        queryItems.contains { $0.name.lowercased() == name }
    }

    private static func firstQueryValue(in queryItems: [URLQueryItem], name: String) -> String? {
        queryItems.first(where: { $0.name.lowercased() == name })?.value
    }

    private static func canonicalSessionState(_ rawValue: String?) -> String? {
        guard let rawValue else {
            return nil
        }

        let normalized = rawValue.trimmingCharacters(in: .whitespacesAndNewlines).uppercased()
        guard normalized == "ACTIVE" else {
            return nil
        }

        return "SessionState::Active"
    }

    private static func boundedHint(_ rawValue: String?) -> String? {
        guard let rawValue, !rawValue.isEmpty else {
            return nil
        }

        if rawValue.count <= 18 {
            return rawValue
        }

        return "\(rawValue.prefix(8))...\(rawValue.suffix(4))"
    }

    private static func boundedTranscript(_ rawValue: String?) -> String? {
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

    private static func boundedSummary(_ rawValue: String?) -> String? {
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

    private static func boundedContinuityDetail(_ rawValue: String?) -> String? {
        guard let rawValue else {
            return nil
        }

        let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            return nil
        }

        if trimmed.count <= 96 {
            return trimmed
        }

        return "\(trimmed.prefix(93))..."
    }

    private static func boundedClarifyQuestion(_ rawValue: String?) -> String? {
        guard let rawValue else {
            return nil
        }

        let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty,
              trimmed.count <= 240,
              !trimmed.contains("\n"),
              !trimmed.contains("\r") else {
            return nil
        }

        return trimmed
    }

    private static func boundedClarifyMissingField(_ rawValue: String?) -> String? {
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

    private static func interruptClarifyWhatIsMissing(in queryItems: [URLQueryItem]) -> String? {
        let values = queryItems.filter { $0.name.lowercased() == "interrupt_clarify_what_is_missing" }
        guard values.count == 1 else {
            return nil
        }

        return boundedClarifyMissingField(values[0].value)
    }

    private static func interruptClarifyAmbiguityFlags(in queryItems: [URLQueryItem]) -> [String] {
        var flags: [String] = []

        for queryItem in queryItems where queryItem.name.lowercased() == "interrupt_clarify_ambiguity_flag" {
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

    private static func interruptClarifyRoutingHints(in queryItems: [URLQueryItem]) -> [String] {
        var hints: [String] = []

        for queryItem in queryItems where queryItem.name.lowercased() == "interrupt_clarify_routing_hint" {
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

    private static func interruptClarifyRequiresConfirmation(in queryItems: [URLQueryItem]) -> Bool? {
        let values = queryItems.filter {
            $0.name.lowercased() == "interrupt_clarify_requires_confirmation"
        }
        guard values.count == 1 else {
            return nil
        }

        return canonicalBoolean(values[0].value)
    }

    private static func interruptClarifySensitivityLevel(in queryItems: [URLQueryItem]) -> String? {
        let values = queryItems.filter {
            $0.name.lowercased() == "interrupt_clarify_sensitivity_level"
        }
        guard values.count == 1,
              let value = values[0].value,
              let canonicalValue = CanonicalInterruptClarifySensitivityLevel.parse(value)?.rawValue
        else {
            return nil
        }

        return canonicalValue
    }

    private static func interruptSubjectRelationConfidence(in queryItems: [URLQueryItem]) -> String? {
        let values = queryItems.filter {
            $0.name.lowercased() == "interrupt_subject_relation_confidence"
        }
        guard values.count == 1,
              let value = values[0].value
        else {
            return nil
        }

        let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty,
              !trimmed.contains("\n"),
              !trimmed.contains("\r"),
              let confidence = Double(trimmed),
              confidence.isFinite,
              confidence >= 0.0,
              confidence <= 1.0
        else {
            return nil
        }

        return trimmed
    }

    private static func interruptAcceptedAnswerFormats(in queryItems: [URLQueryItem]) -> [String] {
        var formats: [String] = []

        for queryItem in queryItems where queryItem.name.lowercased() == "interrupt_accepted_answer_format" {
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

    private static func canonicalBoolean(_ rawValue: String?) -> Bool? {
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

    private static func booleanValue(_ value: Bool) -> String {
        value ? "true" : "false"
    }
}

struct SessionSoftClosedVisibleContext: Identifiable, Equatable {
    let id: String
    let sessionID: String
    let sessionState: String
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

        let scheme = (components.scheme ?? "").lowercased()
        guard ["selene", "https", "http"].contains(scheme) else {
            return nil
        }

        let host = (components.host ?? "no-host").lowercased()
        let path = components.path.isEmpty ? "/" : components.path
        let lowerPath = path.lowercased()
        let queryItems = components.queryItems ?? []
        let inviteLike = host.contains("invite") || lowerPath.contains("invite") || lowerPath.contains("onboarding")
        let appOpenLike = host.contains("open")
            || lowerPath.contains("open")
            || lowerPath.contains("entry")
            || Self.hasQueryItem(in: queryItems, name: "session_state")

        let sessionState = Self.canonicalSessionState(
            Self.firstQueryValue(in: queryItems, name: "session_state")
        )
        let sessionID = Self.boundedHint(
            Self.firstQueryValue(in: queryItems, name: "session_id")
        )
        let selectedThreadID = Self.boundedHint(
            Self.firstQueryValue(in: queryItems, name: "selected_thread_id")
        )
        let selectedThreadTitle = Self.boundedTitle(
            Self.firstQueryValue(in: queryItems, name: "selected_thread_title")
        )
        let pendingWorkOrderID = Self.boundedHint(
            Self.firstQueryValue(in: queryItems, name: "pending_work_order_id")
        )
        let resumeTier = Self.canonicalResumeTier(
            Self.firstQueryValue(in: queryItems, name: "resume_tier")
        )
        let resumeSummaryBullets = Self.resumeSummaryBullets(in: queryItems)
        let archivedUserTurnText = Self.boundedTranscript(
            Self.firstQueryValue(in: queryItems, name: "archived_user_turn_text")
        )
        let archivedSeleneTurnText = Self.boundedTranscript(
            Self.firstQueryValue(in: queryItems, name: "archived_selene_turn_text")
        )
        let recoveryMode = CanonicalRecoveryMode.parse(
            Self.firstQueryValue(in: queryItems, name: "recovery_mode")
        )
        let reconciliationDecision = CanonicalReconciliationDecision.parse(
            Self.firstQueryValue(in: queryItems, name: "reconciliation_decision")
        )

        guard !inviteLike,
              appOpenLike,
              let sessionState,
              let sessionID,
              let archivedUserTurnText,
              let archivedSeleneTurnText else {
            return nil
        }

        self.id = url.absoluteString
        self.sessionID = sessionID
        self.sessionState = sessionState
        self.selectedThreadID = selectedThreadID
        self.selectedThreadTitle = selectedThreadTitle
        self.pendingWorkOrderID = pendingWorkOrderID
        self.resumeTier = resumeTier
        self.resumeSummaryBullets = resumeSummaryBullets
        self.archivedUserTurnText = archivedUserTurnText
        self.archivedSeleneTurnText = archivedSeleneTurnText
        self.recoveryMode = recoveryMode
        self.reconciliationDecision = reconciliationDecision
    }

    var sessionRows: [EntryMetadataRow] {
        [
            EntryMetadataRow(label: "session_state", value: sessionState),
            EntryMetadataRow(label: "session_id", value: sessionID),
        ]
    }

    var archivedRecentSliceEntries: [RecentThreadPreviewEntry] {
        [
            RecentThreadPreviewEntry(
                speaker: "You",
                posture: "archived_user_turn_text",
                body: archivedUserTurnText,
                detail: "Archived recent slice remains durable archived conversation truth and stays distinct from bounded PH1.M `resume context` output."
            ),
            RecentThreadPreviewEntry(
                speaker: "Selene",
                posture: "archived_selene_turn_text",
                body: archivedSeleneTurnText,
                detail: "Archived recent slice remains text-visible after visual reset without local auto-reopen, hidden spoken-only output, or local transcript authority."
            ),
        ]
    }

    var resumeContextRows: [EntryMetadataRow] {
        [
            EntryMetadataRow(label: "selected_thread_id", value: selectedThreadID ?? "not_provided"),
            EntryMetadataRow(label: "selected_thread_title", value: selectedThreadTitle ?? "not_provided"),
            EntryMetadataRow(label: "pending_work_order_id", value: pendingWorkOrderID ?? "not_provided"),
            EntryMetadataRow(label: "resume_tier", value: resumeTier ?? "not_provided"),
        ]
    }

    var recoveryDisplayState: ShellDisplayState? {
        resolvedRecoveryDisplayState(
            recoveryMode: recoveryMode,
            reconciliationDecision: reconciliationDecision
        )
    }

    var recoveryPostureRows: [EntryMetadataRow] {
        recoveryPostureRowsForVisibleSession(
            sessionState: sessionState,
            sessionID: sessionID,
            recoveryMode: recoveryMode,
            reconciliationDecision: reconciliationDecision
        )
    }

    private static func hasQueryItem(in queryItems: [URLQueryItem], name: String) -> Bool {
        queryItems.contains { $0.name.lowercased() == name }
    }

    private static func firstQueryValue(in queryItems: [URLQueryItem], name: String) -> String? {
        queryItems.first(where: { $0.name.lowercased() == name })?.value
    }

    private static func resumeSummaryBullets(in queryItems: [URLQueryItem]) -> [String] {
        queryItems.compactMap { queryItem in
            guard queryItem.name.lowercased() == "resume_summary_bullets" else {
                return nil
            }

            return boundedBullet(queryItem.value)
        }
    }

    private static func canonicalSessionState(_ rawValue: String?) -> String? {
        guard let rawValue else {
            return nil
        }

        let normalized = rawValue.trimmingCharacters(in: .whitespacesAndNewlines).uppercased()
        guard normalized == "SOFT_CLOSED" else {
            return nil
        }

        return "SessionState::SoftClosed"
    }

    private static func canonicalResumeTier(_ rawValue: String?) -> String? {
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

    private static func boundedHint(_ rawValue: String?) -> String? {
        guard let rawValue, !rawValue.isEmpty else {
            return nil
        }

        if rawValue.count <= 18 {
            return rawValue
        }

        return "\(rawValue.prefix(8))...\(rawValue.suffix(4))"
    }

    private static func boundedTitle(_ rawValue: String?) -> String? {
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

    private static func boundedTranscript(_ rawValue: String?) -> String? {
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

    private static func boundedBullet(_ rawValue: String?) -> String? {
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
}

struct SessionSuspendedVisibleContext: Identifiable, Equatable {
    let id: String
    let sessionID: String
    let sessionState: String
    let nextAllowedActionsMaySpeak: Bool
    let nextAllowedActionsMustWait: Bool
    let nextAllowedActionsMustRewake: Bool
    let recoveryMode: CanonicalRecoveryMode?
    let reconciliationDecision: CanonicalReconciliationDecision?

    init?(url: URL) {
        guard let components = URLComponents(url: url, resolvingAgainstBaseURL: false) else {
            return nil
        }

        let scheme = (components.scheme ?? "").lowercased()
        guard ["selene", "https", "http"].contains(scheme) else {
            return nil
        }

        let host = (components.host ?? "no-host").lowercased()
        let path = components.path.isEmpty ? "/" : components.path
        let lowerPath = path.lowercased()
        let queryItems = components.queryItems ?? []
        let inviteLike = host.contains("invite") || lowerPath.contains("invite") || lowerPath.contains("onboarding")
        let appOpenLike = host.contains("open")
            || lowerPath.contains("open")
            || lowerPath.contains("entry")
            || Self.hasQueryItem(in: queryItems, name: "session_state")

        let sessionState = Self.canonicalSessionState(
            Self.firstQueryValue(in: queryItems, name: "session_state")
        )
        let sessionID = Self.boundedHint(
            Self.firstQueryValue(in: queryItems, name: "session_id")
        )
        let nextAllowedActionsMaySpeak = Self.canonicalBoolean(
            Self.firstQueryValue(in: queryItems, name: "next_allowed_actions_may_speak")
        )
        let nextAllowedActionsMustWait = Self.canonicalBoolean(
            Self.firstQueryValue(in: queryItems, name: "next_allowed_actions_must_wait")
        )
        let nextAllowedActionsMustRewake = Self.canonicalBoolean(
            Self.firstQueryValue(in: queryItems, name: "next_allowed_actions_must_rewake")
        )
        let recoveryMode = CanonicalRecoveryMode.parse(
            Self.firstQueryValue(in: queryItems, name: "recovery_mode")
        )
        let reconciliationDecision = CanonicalReconciliationDecision.parse(
            Self.firstQueryValue(in: queryItems, name: "reconciliation_decision")
        )

        guard !inviteLike,
              appOpenLike,
              let sessionState,
              let sessionID,
              let nextAllowedActionsMaySpeak,
              let nextAllowedActionsMustWait,
              let nextAllowedActionsMustRewake else {
            return nil
        }

        self.id = url.absoluteString
        self.sessionID = sessionID
        self.sessionState = sessionState
        self.nextAllowedActionsMaySpeak = nextAllowedActionsMaySpeak
        self.nextAllowedActionsMustWait = nextAllowedActionsMustWait
        self.nextAllowedActionsMustRewake = nextAllowedActionsMustRewake
        self.recoveryMode = recoveryMode
        self.reconciliationDecision = reconciliationDecision
    }

    var suspendedStatusRows: [EntryMetadataRow] {
        var rows = [
            EntryMetadataRow(label: "session_state", value: sessionState),
            EntryMetadataRow(label: "session_id", value: sessionID),
        ]

        if let recoveryMode {
            rows.append(EntryMetadataRow(label: "recovery_mode", value: recoveryMode.rawValue))
        }

        if let reconciliationDecision {
            rows.append(
                EntryMetadataRow(
                    label: "reconciliation_decision",
                    value: reconciliationDecision.rawValue
                )
            )
        }

        return rows
    }

    var allowedNextStepRows: [EntryMetadataRow] {
        [
            EntryMetadataRow(
                label: "next_allowed_actions_may_speak",
                value: Self.booleanValue(nextAllowedActionsMaySpeak)
            ),
            EntryMetadataRow(
                label: "next_allowed_actions_must_wait",
                value: Self.booleanValue(nextAllowedActionsMustWait)
            ),
            EntryMetadataRow(
                label: "next_allowed_actions_must_rewake",
                value: Self.booleanValue(nextAllowedActionsMustRewake)
            ),
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

    private static func hasQueryItem(in queryItems: [URLQueryItem], name: String) -> Bool {
        queryItems.contains { $0.name.lowercased() == name }
    }

    private static func firstQueryValue(in queryItems: [URLQueryItem], name: String) -> String? {
        queryItems.first(where: { $0.name.lowercased() == name })?.value
    }

    private static func canonicalSessionState(_ rawValue: String?) -> String? {
        guard let rawValue else {
            return nil
        }

        let normalized = rawValue.trimmingCharacters(in: .whitespacesAndNewlines).uppercased()
        guard normalized == "SUSPENDED" else {
            return nil
        }

        return "SessionState::Suspended"
    }

    private static func canonicalBoolean(_ rawValue: String?) -> Bool? {
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

    private static func boundedHint(_ rawValue: String?) -> String? {
        guard let rawValue, !rawValue.isEmpty else {
            return nil
        }

        if rawValue.count <= 18 {
            return rawValue
        }

        return "\(rawValue.prefix(8))...\(rawValue.suffix(4))"
    }

    private static func booleanValue(_ value: Bool) -> String {
        value ? "true" : "false"
    }
}

private enum RecoveryVisibleSurface {
    case sessionOpen(SessionOpenVisibleContext)
    case sessionActive(SessionActiveVisibleContext)
    case sessionSoftClosed(SessionSoftClosedVisibleContext)

    var sessionState: String {
        switch self {
        case .sessionOpen(let context):
            return context.sessionState
        case .sessionActive(let context):
            return context.sessionState
        case .sessionSoftClosed(let context):
            return context.sessionState
        }
    }

    var sessionID: String {
        switch self {
        case .sessionOpen(let context):
            return context.sessionID
        case .sessionActive(let context):
            return context.sessionID
        case .sessionSoftClosed(let context):
            return context.sessionID
        }
    }

    var recoveryMode: CanonicalRecoveryMode? {
        switch self {
        case .sessionOpen(let context):
            return context.recoveryMode
        case .sessionActive(let context):
            return context.recoveryMode
        case .sessionSoftClosed(let context):
            return context.recoveryMode
        }
    }

    var reconciliationDecision: CanonicalReconciliationDecision? {
        switch self {
        case .sessionOpen(let context):
            return context.reconciliationDecision
        case .sessionActive(let context):
            return context.reconciliationDecision
        case .sessionSoftClosed(let context):
            return context.reconciliationDecision
        }
    }

    var sourceSurfaceTitle: String {
        switch self {
        case .sessionOpen:
            return "SESSION_OPEN_VISIBLE"
        case .sessionActive:
            return "SESSION_ACTIVE_VISIBLE"
        case .sessionSoftClosed:
            return "SESSION_SOFT_CLOSED_VISIBLE"
        }
    }

    var recoveryPostureRows: [EntryMetadataRow] {
        recoveryPostureRowsForVisibleSession(
            sessionState: sessionState,
            sessionID: sessionID,
            recoveryMode: recoveryMode,
            reconciliationDecision: reconciliationDecision
        )
    }
}

struct ExplicitEntryContext: Identifiable, Equatable {
    let id: String
    let routeKind: ExplicitEntryRouteKind
    let scheme: String
    let host: String
    let path: String
    let tenantHint: String?
    let tokenHint: String?
    let deepLinkNonce: String?
    let appInstance: String?
    let deviceFingerprint: String?
    let onboardingSessionID: String?
    let nextStep: String?
    let requiredFields: [String]
    let requiredVerificationGates: [String]
    let blockingField: String?
    let blockingQuestion: String?
    let remainingMissingFields: [String]
    let onboardingStatus: String?
    let remainingPlatformReceiptKinds: [String]
    let voiceArtifactSyncReceiptRef: String?
    let accessEngineInstanceID: String?

    init?(url: URL) {
        guard let components = URLComponents(url: url, resolvingAgainstBaseURL: false) else {
            return nil
        }

        let scheme = (components.scheme ?? "").lowercased()
        guard ["selene", "https", "http"].contains(scheme) else {
            return nil
        }

        let host = (components.host ?? "no-host").lowercased()
        let path = components.path.isEmpty ? "/" : components.path
        let queryItems = components.queryItems ?? []

        let token = Self.firstValue(in: queryItems, names: ["token", "invite", "invite_token"])
        let tenant = Self.firstValue(in: queryItems, names: ["tenant", "tenant_hint"])
        let nonce = Self.firstValue(in: queryItems, names: ["nonce", "deep_link_nonce"])
        let appInstance = Self.firstValue(in: queryItems, names: ["app_instance", "app"])
        let deviceFingerprint = Self.firstValue(in: queryItems, names: ["device_fingerprint", "device"])
        let onboardingSessionID = Self.firstValue(in: queryItems, names: ["onboarding_session_id"])
        let nextStep = Self.firstValue(in: queryItems, names: ["next_step"])
        let requiredFields = Self.values(in: queryItems, names: ["required_field"])
        let requiredVerificationGates = Self.values(in: queryItems, names: ["verification_gate"])
        let blockingField = Self.firstValue(in: queryItems, names: ["blocking_field"])
        let blockingQuestion = Self.firstValue(in: queryItems, names: ["blocking_question"])
        let remainingMissingFields = Self.values(in: queryItems, names: ["remaining_missing_field"])
        let onboardingStatus = Self.firstValue(in: queryItems, names: ["onboarding_status"])
        let remainingPlatformReceiptKinds = Self.values(
            in: queryItems,
            names: ["remaining_platform_receipt_kind"]
        )
        let voiceArtifactSyncReceiptRef = Self.firstValue(
            in: queryItems,
            names: ["voice_artifact_sync_receipt_ref"]
        )
        let accessEngineInstanceID = Self.firstValue(
            in: queryItems,
            names: ["access_engine_instance_id"]
        )

        let lowerPath = path.lowercased()
        let inviteLike = host.contains("invite") || lowerPath.contains("invite") || lowerPath.contains("onboarding") || token != nil
        let openLike = host.contains("open") || lowerPath.contains("open") || lowerPath.contains("entry") || nonce != nil

        let routeKind: ExplicitEntryRouteKind
        if inviteLike {
            routeKind = .inviteOpen
        } else if openLike {
            routeKind = .appOpen
        } else {
            return nil
        }

        self.id = url.absoluteString
        self.routeKind = routeKind
        self.scheme = scheme
        self.host = host
        self.path = path
        self.tenantHint = Self.boundedHint(tenant)
        self.tokenHint = Self.boundedHint(token)
        self.deepLinkNonce = Self.boundedHint(nonce)
        self.appInstance = Self.boundedHint(appInstance)
        self.deviceFingerprint = Self.boundedHint(deviceFingerprint)
        self.onboardingSessionID = Self.boundedHint(onboardingSessionID)
        self.nextStep = Self.boundedHint(nextStep)
        self.requiredFields = Self.boundedValues(requiredFields)
        self.requiredVerificationGates = Self.boundedValues(requiredVerificationGates)
        self.blockingField = Self.boundedHint(blockingField)
        self.blockingQuestion = Self.boundedHint(blockingQuestion)
        self.remainingMissingFields = Self.boundedValues(remainingMissingFields)
        self.onboardingStatus = Self.boundedHint(onboardingStatus)
        self.remainingPlatformReceiptKinds = Self.boundedValues(remainingPlatformReceiptKinds)
        self.voiceArtifactSyncReceiptRef = Self.boundedHint(voiceArtifactSyncReceiptRef)
        self.accessEngineInstanceID = Self.boundedHint(accessEngineInstanceID)
    }

    var rows: [EntryMetadataRow] {
        var rows = [
            EntryMetadataRow(label: "entry_kind", value: routeKind.rawValue),
            EntryMetadataRow(label: "scheme", value: scheme),
            EntryMetadataRow(label: "host", value: host),
            EntryMetadataRow(label: "path", value: path),
        ]

        if let tenantHint {
            rows.append(EntryMetadataRow(label: "tenant_hint", value: tenantHint))
        }
        if let tokenHint {
            rows.append(EntryMetadataRow(label: "token_hint", value: tokenHint))
        }
        if let deepLinkNonce {
            rows.append(EntryMetadataRow(label: "deep_link_nonce", value: deepLinkNonce))
        }
        if let appInstance {
            rows.append(EntryMetadataRow(label: "app_instance", value: appInstance))
        }
        if let deviceFingerprint {
            rows.append(EntryMetadataRow(label: "device_fingerprint", value: deviceFingerprint))
        }

        return rows
    }

    var onboardingOutcomeRows: [EntryMetadataRow] {
        [
            EntryMetadataRow(
                label: "onboarding_session_id",
                value: onboardingSessionID ?? "not_provided"
            ),
            EntryMetadataRow(
                label: "next_step",
                value: nextStep ?? "not_provided"
            ),
            EntryMetadataRow(
                label: "required_fields",
                value: requiredFields.isEmpty ? "none_provided" : "\(requiredFields.count)_provided"
            ),
            EntryMetadataRow(
                label: "required_verification_gates",
                value: requiredVerificationGates.isEmpty ? "none_provided" : "\(requiredVerificationGates.count)_provided"
            ),
        ]
    }

    var onboardingContinueRows: [EntryMetadataRow] {
        [
            EntryMetadataRow(
                label: "onboarding_status",
                value: onboardingStatus ?? "not_provided"
            ),
            EntryMetadataRow(
                label: "remaining_platform_receipt_kinds",
                value: remainingPlatformReceiptKinds.isEmpty
                    ? "none_provided"
                    : "\(remainingPlatformReceiptKinds.count)_provided"
            ),
        ]
    }

    var onboardingContinuePromptRows: [EntryMetadataRow] {
        [
            EntryMetadataRow(
                label: "blocking_field",
                value: blockingField ?? "not_provided"
            ),
            EntryMetadataRow(
                label: "blocking_question",
                value: blockingQuestion ?? "not_provided"
            ),
            EntryMetadataRow(
                label: "remaining_missing_fields",
                value: remainingMissingFields.isEmpty
                    ? "none_provided"
                    : "\(remainingMissingFields.count)_provided"
            ),
        ]
    }

    var onboardingContinueArtifactAccessRows: [EntryMetadataRow] {
        [
            EntryMetadataRow(
                label: "voice_artifact_sync_receipt_ref",
                value: voiceArtifactSyncReceiptRef ?? "not_provided"
            ),
            EntryMetadataRow(
                label: "access_engine_instance_id",
                value: accessEngineInstanceID ?? "not_provided"
            ),
        ]
    }

    private static func firstValue(in queryItems: [URLQueryItem], names: [String]) -> String? {
        for name in names {
            if let value = queryItems.first(where: { $0.name.lowercased() == name })?.value,
               !value.isEmpty {
                return value
            }
        }

        return nil
    }

    private static func values(in queryItems: [URLQueryItem], names: [String]) -> [String] {
        var values: [String] = []

        for queryItem in queryItems {
            guard names.contains(queryItem.name.lowercased()),
                  let value = queryItem.value,
                  !value.isEmpty else {
                continue
            }

            values.append(value)
        }

        return values
    }

    private static func boundedHint(_ rawValue: String?) -> String? {
        guard let rawValue, !rawValue.isEmpty else {
            return nil
        }

        if rawValue.count <= 18 {
            return rawValue
        }

        return "\(rawValue.prefix(8))...\(rawValue.suffix(4))"
    }

    private static func boundedValues(_ rawValues: [String]) -> [String] {
        var bounded: [String] = []

        for rawValue in rawValues {
            guard let value = boundedHint(rawValue), !bounded.contains(value) else {
                continue
            }

            bounded.append(value)
        }

        return bounded
    }
}

struct EntryMetadataRow: Identifiable, Equatable {
    let label: String
    let value: String

    var id: String {
        label
    }
}

private struct SetupReceipt: Identifiable {
    let name: String
    let detail: String

    var id: String {
        name
    }
}

struct RecentThreadPreviewEntry: Identifiable {
    let speaker: String
    let posture: String
    let body: String
    let detail: String

    var id: String {
        "\(speaker)-\(posture)-\(body)"
    }
}

private struct OperationalQueueEntry: Identifiable {
    let name: String
    let posture: String
    let summary: String
    let detail: String

    var id: String {
        name
    }
}

private struct TypedTurnRequestState: Identifiable {
    let id: String
    let text: String
    let byteCount: Int

    var boundedPreview: String {
        if text.count <= 96 {
            return text
        }

        return "\(text.prefix(93))..."
    }
}

private struct ExplicitVoiceTurnRequestState: Identifiable {
    let id: String
    let transcript: String
    let byteCount: Int

    var boundedPreview: String {
        if transcript.count <= 96 {
            return transcript
        }

        return "\(transcript.prefix(93))..."
    }
}

private enum InterruptContinuityResponseKind: String {
    case clarifyDirective = "clarify_directive"
    case returnCheckResponse = "return_check_response"
}

private struct InterruptContinuityResponseRequestState: Identifiable {
    let id: String
    let kind: InterruptContinuityResponseKind
    let responseLabel: String
    let canonicalValue: String
    let sessionID: String
    let turnID: String

    var pendingOperationalEntry: OperationalQueueEntry {
        let responseDetail: String
        switch kind {
        case .clarifyDirective:
            responseDetail = "Clarify directive response: \(responseLabel)."
        case .returnCheckResponse:
            responseDetail = "Return-check response: \(responseLabel) (`\(canonicalValue)`)."
        }

        return OperationalQueueEntry(
            name: id,
            posture: "pending",
            summary: "Awaiting authoritative interruption continuity response.",
            detail: "Bounded continuity response production only. \(responseDetail) Session `\(sessionID)` turn `\(turnID)` remains non-authoritative until canonical follow-up occurs, and this shell does not invent local interrupt law, fake resume authority, or silent discard."
        )
    }
}

private enum VoicePermissionState: String {
    case notRequested = "not_requested"
    case granted = "granted"
    case denied = "denied"
    case restricted = "restricted"
    case unavailable = "unavailable"

    var detail: String {
        switch self {
        case .notRequested:
            return "Permission has not been requested in this foreground session yet."
        case .granted:
            return "Permission is granted for bounded explicit voice capture only."
        case .denied:
            return "Permission was denied. Explicit voice capture stays blocked until the user re-enables access."
        case .restricted:
            return "Permission is restricted by device policy. This shell remains non-authoritative and does not bypass policy."
        case .unavailable:
            return "Permission or recognizer availability is unavailable on this device posture."
        }
    }
}

private final class ExplicitVoiceCaptureController: ObservableObject {
    @Published private(set) var microphonePermission: VoicePermissionState = .notRequested
    @Published private(set) var speechRecognitionPermission: VoicePermissionState = .notRequested
    @Published private(set) var isListening = false
    @Published private(set) var transcriptPreview = ""
    @Published private(set) var pendingRequest: ExplicitVoiceTurnRequestState?
    @Published private(set) var failedRequest: OperationalQueueEntry?

    private let maxVoiceTurnBytes = 16_384
    private let audioEngine = AVAudioEngine()
    private let speechRecognizer: SFSpeechRecognizer?
    private var recognitionRequest: SFSpeechAudioBufferRecognitionRequest?
    private var recognitionTask: SFSpeechRecognitionTask?
    private var hasInputTap = false
    private var requestSequence = 0

    init(locale: Locale? = nil) {
        let resolvedLocale = locale ?? Self.preferredLocale()
        speechRecognizer = SFSpeechRecognizer(locale: resolvedLocale) ?? SFSpeechRecognizer()
        refreshPermissionState()
    }

    func startExplicitVoiceTurn() {
        failedRequest = nil

        guard !isListening else {
            return
        }

        guard pendingRequest == nil else {
            recordFailure(
                name: "failed_explicit_voice_turn_awaiting_authoritative_response",
                summary: "A later explicit voice-turn request could not be produced while the current bounded explicit voice request is already awaiting authoritative response.",
                detail: "This shell keeps only bounded pending / failed posture. It does not queue another local voice request, repair transport, or fabricate local assistant output."
            )
            return
        }

        transcriptPreview = ""
        refreshPermissionState()
        requestMicrophonePermissionIfNeeded { [weak self] granted in
            guard let self else {
                return
            }

            DispatchQueue.main.async {
                self.refreshPermissionState()
                guard granted else {
                    self.recordFailure(
                        name: "failed_explicit_voice_microphone_permission",
                        summary: "Microphone permission is required before a bounded explicit voice-turn request can begin.",
                        detail: "Permission visibility only; this shell does not bypass device policy, start hidden capture, or synthesize a request without foreground user approval."
                    )
                    return
                }

                self.requestSpeechRecognitionPermissionIfNeeded { [weak self] speechGranted in
                    guard let self else {
                        return
                    }

                    DispatchQueue.main.async {
                        self.refreshPermissionState()
                        guard speechGranted else {
                            self.recordFailure(
                                name: "failed_explicit_voice_speech_permission",
                                summary: "Speech-recognition permission is required before a bounded explicit voice-turn request can prepare transcript preview.",
                                detail: "Permission visibility only; this shell does not create hidden spoken-only output, local transcript authority, or silent authoritative acceptance."
                            )
                            return
                        }

                        self.beginCaptureSession()
                    }
                }
            }
        }
    }

    func stopCaptureAndPrepareVoiceTurn() {
        guard isListening else {
            recordFailure(
                name: "failed_explicit_voice_not_listening",
                summary: "The bounded explicit voice surface was not actively listening when request preparation was attempted.",
                detail: "Explicit voice-turn production remains foreground-only and user-visible. Start a new explicit voice turn before preparing another bounded request."
            )
            return
        }

        endCaptureInput()

        let trimmedTranscript = transcriptPreview.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedTranscript.isEmpty else {
            recordFailure(
                name: "failed_explicit_voice_empty_transcript",
                summary: "No bounded transcript preview was available when this explicit voice turn stopped, so no voice request was produced.",
                detail: "Failure visibility only; speak again and retry through the canonical explicit voice path. No local assistant output or authoritative transcript mutation was produced."
            )
            return
        }

        if trimmedTranscript.utf8.count > maxVoiceTurnBytes {
            recordFailure(
                name: "failed_explicit_voice_transcript_validation",
                summary: "The bounded explicit voice transcript exceeded 16384 UTF-8 bytes before any authoritative acceptance occurred.",
                detail: "Failure visibility only; retry a shorter utterance through the canonical explicit voice path. No authoritative transcript turn was appended locally."
            )
            return
        }

        requestSequence += 1
        transcriptPreview = trimmedTranscript
        pendingRequest = ExplicitVoiceTurnRequestState(
            id: String(format: "voice_turn_request_%03d", requestSequence),
            transcript: trimmedTranscript,
            byteCount: trimmedTranscript.utf8.count
        )
    }

    func haltCaptureSession() {
        teardownRecognitionSession()
    }

    private func beginCaptureSession() {
        failedRequest = nil
        teardownRecognitionSession()
        refreshPermissionState()

        guard let speechRecognizer else {
            speechRecognitionPermission = .unavailable
            recordFailure(
                name: "failed_explicit_voice_recognizer_unavailable",
                summary: "No speech recognizer is available for bounded explicit voice-turn request preparation on this device posture.",
                detail: "Unavailable visibility only; the shell remains `EXPLICIT_ONLY`, session-bound, and cloud-authoritative while explicit voice capture stays blocked."
            )
            return
        }

        guard speechRecognizer.isAvailable else {
            speechRecognitionPermission = .unavailable
            recordFailure(
                name: "failed_explicit_voice_recognizer_busy",
                summary: "The speech recognizer is not currently available for a bounded explicit voice turn.",
                detail: "Availability visibility only; retry from the same foreground surface later. No local queue repair authority or hidden retry loop is introduced here."
            )
            return
        }

        do {
            let audioSession = AVAudioSession.sharedInstance()
            try audioSession.setCategory(.record, mode: .measurement, options: [.duckOthers])
            try audioSession.setActive(true, options: .notifyOthersOnDeactivation)

            let request = SFSpeechAudioBufferRecognitionRequest()
            request.shouldReportPartialResults = true
            request.taskHint = .dictation
            recognitionRequest = request

            let inputNode = audioEngine.inputNode
            let format = inputNode.outputFormat(forBus: 0)
            inputNode.installTap(onBus: 0, bufferSize: 1_024, format: format) { [weak self] buffer, _ in
                self?.recognitionRequest?.append(buffer)
            }
            hasInputTap = true

            audioEngine.prepare()
            try audioEngine.start()
            transcriptPreview = ""
            isListening = true

            recognitionTask = speechRecognizer.recognitionTask(with: request) { [weak self] result, error in
                guard let self else {
                    return
                }

                if let result {
                    DispatchQueue.main.async {
                        self.transcriptPreview = result.bestTranscription.formattedString
                    }
                }

                if let error {
                    DispatchQueue.main.async {
                        guard self.isListening else {
                            return
                        }

                        self.teardownRecognitionSession()
                        self.recordFailure(
                            name: "failed_explicit_voice_capture_session",
                            summary: "The bounded explicit voice capture session ended before a request could be prepared.",
                            detail: "Speech capture failed with `\(error.localizedDescription)`. Failure visibility only; no local transcript authority, no hidden retry loop, and no authoritative assistant output were produced."
                        )
                    }
                }
            }
        } catch {
            teardownRecognitionSession()
            recordFailure(
                name: "failed_explicit_voice_capture_start",
                summary: "The bounded explicit voice capture session could not start from this foreground surface.",
                detail: "Capture start failed with `\(error.localizedDescription)`. Failure visibility only; no background capture, no wake behavior, and no autonomous unlock were introduced."
            )
        }
    }

    private func endCaptureInput() {
        if audioEngine.isRunning {
            audioEngine.stop()
        }

        if hasInputTap {
            audioEngine.inputNode.removeTap(onBus: 0)
            hasInputTap = false
        }

        recognitionRequest?.endAudio()
        isListening = false
        deactivateAudioSessionIfNeeded()
    }

    private func teardownRecognitionSession() {
        endCaptureInput()
        recognitionTask?.cancel()
        recognitionTask = nil
        recognitionRequest = nil
    }

    private func deactivateAudioSessionIfNeeded() {
        do {
            try AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
        } catch {
            // Session deactivation remains best-effort cleanup only.
        }
    }

    private func recordFailure(name: String, summary: String, detail: String) {
        failedRequest = OperationalQueueEntry(
            name: name,
            posture: "failed",
            summary: summary,
            detail: detail
        )
    }

    private func refreshPermissionState() {
        microphonePermission = Self.currentMicrophonePermission()
        speechRecognitionPermission = Self.currentSpeechRecognitionPermission(speechRecognizerAvailable: speechRecognizer != nil)
    }

    private func requestMicrophonePermissionIfNeeded(completion: @escaping (Bool) -> Void) {
        switch Self.currentMicrophonePermission() {
        case .granted:
            completion(true)
        case .denied, .restricted, .unavailable:
            completion(false)
        case .notRequested:
            AVAudioApplication.requestRecordPermission { granted in
                completion(granted)
            }
        }
    }

    private func requestSpeechRecognitionPermissionIfNeeded(completion: @escaping (Bool) -> Void) {
        switch Self.currentSpeechRecognitionPermission(speechRecognizerAvailable: speechRecognizer != nil) {
        case .granted:
            completion(true)
        case .denied, .restricted, .unavailable:
            completion(false)
        case .notRequested:
            SFSpeechRecognizer.requestAuthorization { status in
                completion(status == .authorized)
            }
        }
    }

    private static func currentMicrophonePermission() -> VoicePermissionState {
        switch AVAudioApplication.shared.recordPermission {
        case .granted:
            return .granted
        case .denied:
            return .denied
        case .undetermined:
            return .notRequested
        @unknown default:
            return .unavailable
        }
    }

    private static func currentSpeechRecognitionPermission(speechRecognizerAvailable: Bool) -> VoicePermissionState {
        guard speechRecognizerAvailable else {
            return .unavailable
        }

        switch SFSpeechRecognizer.authorizationStatus() {
        case .authorized:
            return .granted
        case .denied:
            return .denied
        case .restricted:
            return .restricted
        case .notDetermined:
            return .notRequested
        @unknown default:
            return .unavailable
        }
    }

    private static func preferredLocale() -> Locale {
        if let identifier = Locale.preferredLanguages.first {
            return Locale(identifier: identifier)
        }

        return Locale(identifier: "en-US")
    }
}

struct SessionShellView: View {
    @ObservedObject var router: ExplicitEntryRouter

    @State private var displayState: ShellDisplayState = .explicitEntryReady
    @State private var activeContext: ExplicitEntryContext?
    @State private var activeSessionActiveContext: SessionActiveVisibleContext?
    @State private var activeSessionSoftClosedContext: SessionSoftClosedVisibleContext?
    @State private var activeSessionSuspendedContext: SessionSuspendedVisibleContext?
    @State private var activeSessionOpenContext: SessionOpenVisibleContext?
    @State private var typedTurnDraft: String = ""
    @State private var typedTurnPendingRequest: TypedTurnRequestState?
    @State private var typedTurnFailedRequest: OperationalQueueEntry?
    @State private var typedTurnRequestSequence: Int = 0
    @State private var interruptResponsePendingRequest: InterruptContinuityResponseRequestState?
    @State private var interruptResponseFailedRequest: OperationalQueueEntry?
    @State private var interruptResponseRequestSequence: Int = 0
    @StateObject private var explicitVoiceController = ExplicitVoiceCaptureController()

    private var activeRecoveryVisibleSurface: RecoveryVisibleSurface? {
        if let activeSessionActiveContext {
            return .sessionActive(activeSessionActiveContext)
        }

        if let activeSessionSoftClosedContext {
            return .sessionSoftClosed(activeSessionSoftClosedContext)
        }

        if let activeSessionOpenContext {
            return .sessionOpen(activeSessionOpenContext)
        }

        return nil
    }

    private let setupReceipts = [
        SetupReceipt(
            name: "install_launch_handshake",
            detail: "Canonical installation / first-launch receipt family rendered as evidence-only shell posture."
        ),
        SetupReceipt(
            name: "push_permission_granted",
            detail: "Canonical push-permission receipt family rendered without mutating device policy locally."
        ),
        SetupReceipt(
            name: "notification_token_bound",
            detail: "Canonical notification-token receipt family rendered as read-only cloud-authoritative setup evidence."
        ),
        SetupReceipt(
            name: "ios_side_button_configured",
            detail: "Canonical side-button setup receipt family rendered without claiming a proven live side-button producer."
        ),
    ]
    private let recentThreadPreviewEntries = [
        RecentThreadPreviewEntry(
            speaker: "You",
            posture: "explicit_recent_user_turn",
            body: "Show the latest lawful session context before any cloud-authoritative request path is opened.",
            detail: "User-side thread preview only; recent-thread visibility stays separate from typed-turn request production and local transcript authority."
        ),
        RecentThreadPreviewEntry(
            speaker: "Selene",
            posture: "bounded_resume_context",
            body: "The shell stays EXPLICIT_ONLY and cloud-authoritative; onboarding fields, setup receipts, and runtime law remain read-only in this surface.",
            detail: "Assistant-side bounded resume context only; append-only conversation storage remains distinct from PH1.M memory."
        ),
        RecentThreadPreviewEntry(
            speaker: "You",
            posture: "next_explicit_step",
            body: "Keep this session surface ready for bounded explicit text-turn production while explicit voice entry remains non-producing.",
            detail: "Recent-thread preview only; no invite activation, no onboarding mutation, and no session resurrection occur locally."
        ),
    ]
    private let systemActivityEntries = [
        OperationalQueueEntry(
            name: "persistence_acknowledgement_state",
            posture: "recovery_posture_visible",
            summary: "Persistence acknowledgement state remains visible as bounded operational posture only while authoritative recovery state is reread from the cloud side.",
            detail: "Recovery posture only; no local persistence repair, no manual resend console, and no hidden auto-heal claim are introduced here."
        ),
        OperationalQueueEntry(
            name: "reconciliation_decision",
            posture: "decision_visibility_only",
            summary: "Reconciliation decision posture remains visible without granting local authority to resolve, rewrite, or complete operational work.",
            detail: "Cloud-authoritative decision visibility only; no local completion of pending work or silent disappearance of failed work occurs here."
        ),
        OperationalQueueEntry(
            name: "broadcast_waiting_followup_reminder_state",
            posture: "operational_wait_state",
            summary: "Broadcast waiting, follow-up, and reminder posture remain bounded operational signals separate from transcript history and archived recall.",
            detail: "Session-bound operational visibility only; no local resend, queue mutation, or transcript blending is introduced by this surface."
        ),
    ]
    private let pendingOperationalEntries = [
        OperationalQueueEntry(
            name: "pending_sync_queue",
            posture: "pending",
            summary: "Sync queue work remains pending and visible while authoritative completion stays cloud-side.",
            detail: "Pending visibility only; this shell does not complete, resend, or repair work locally."
        ),
        OperationalQueueEntry(
            name: "pending_recovery_followup",
            posture: "pending",
            summary: "Recovery follow-up remains pending until canonical authoritative state is reread.",
            detail: "Read-only recovery follow-up only; no local transport authority or autonomous unlock posture is introduced here."
        ),
    ]
    private let failedOperationalEntries = [
        OperationalQueueEntry(
            name: "failed_dead_letter_posture",
            posture: "failed",
            summary: "Dead-letter posture remains visible as failed operational state without claiming local recovery or hidden repair authority.",
            detail: "Failed visibility only; no silent disappearance of failed work and no manual resend console exist in this shell."
        ),
        OperationalQueueEntry(
            name: "failed_reconciliation_review",
            posture: "failed",
            summary: "Reconciliation review remains visibly failed until canonical follow-up occurs in the cloud-authoritative flow.",
            detail: "Operational detail only; no local state rewrite, no queue mutation, and no session resurrection occur here."
        ),
    ]
    private let maxTypedTurnBytes = 16_384
    private let needsAttentionEntries = [
        OperationalQueueEntry(
            name: "unresolved_protected_prompt",
            posture: "human_action_required",
            summary: "Protected prompt posture now remains visible in a separate `Needs Attention` queue when real human action is required.",
            detail: "Read-only required-human-action visibility only; no local acknowledgement, retry, or completion authority is introduced here."
        ),
        OperationalQueueEntry(
            name: "blocked_onboarding",
            posture: "blocked_onboarding",
            summary: "Blocked onboarding remains actionable and separate from transcript history while current onboarding field and receipt posture stay read-only.",
            detail: "The shell shows the requirement boundary only; it does not submit onboarding state, bind receipts, or alter cloud authority locally."
        ),
        OperationalQueueEntry(
            name: "stale_recovery_warning",
            posture: "recovery_warning",
            summary: "Stale or recovery warning posture remains actionable only when human follow-up is required after authoritative recovery review.",
            detail: "Warning visibility only; no hidden auto-heal claim, no local repair authority, and no local completion of pending work exist here."
        ),
        OperationalQueueEntry(
            name: "dead_letter_or_failed_delivery",
            posture: "delivery_attention_required",
            summary: "Dead-letter sync or failed delivery posture now appears here only when canonical follow-up requires real human attention.",
            detail: "The shell does not resend, repair transport, or silently remove failed work; it only shows bounded action-copy."
        ),
        OperationalQueueEntry(
            name: "law_or_governance_failure_posture",
            posture: "governance_attention_required",
            summary: "Law or governance failure posture remains actionable and cloud-authoritative when real human review is required.",
            detail: "Governance visibility only; no local override, no local transcript authority, and no autonomous unlock claim are introduced here."
        ),
        OperationalQueueEntry(
            name: "broadcast_followup_requires_human_action",
            posture: "broadcast_attention_required",
            summary: "Broadcast state remains visible here only when follow-up now requires real human action through the canonical path.",
            detail: "Action-copy only; no local resend console, no queue mutation, and no session resurrection occur in this shell."
        ),
    ]

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                headerCard
                displayStateCard

                if displayState == .onboardingEntryActive, let activeContext {
                    takeoverCard(activeContext)
                } else if displayState == .recovering, let activeRecoveryVisibleSurface {
                    recoveringVisibleCard(activeRecoveryVisibleSurface)
                } else if displayState == .degradedRecovery, let activeRecoveryVisibleSurface {
                    degradedRecoveryVisibleCard(activeRecoveryVisibleSurface)
                } else if displayState == .quarantinedLocalState, let activeRecoveryVisibleSurface {
                    quarantinedLocalStateCard(activeRecoveryVisibleSurface)
                } else if displayState == .interruptVisible, let activeSessionActiveContext {
                    interruptVisibleCard(activeSessionActiveContext)
                } else if displayState == .sessionActiveVisible, let activeSessionActiveContext {
                    sessionActiveVisibleCard(activeSessionActiveContext)
                } else if displayState == .sessionSoftClosedVisible, let activeSessionSoftClosedContext {
                    sessionSoftClosedVisibleCard(activeSessionSoftClosedContext)
                } else if displayState == .sessionSuspendedVisible, let activeSessionSuspendedContext {
                    sessionSuspendedVisibleCard(activeSessionSuspendedContext)
                } else if displayState == .sessionOpenVisible, let activeSessionOpenContext {
                    sessionOpenVisibleCard(activeSessionOpenContext)
                } else {
                    explicitEntryReadyCard
                }

                setupReceiptCard
                boundedSurfaceCard(
                    title: "Session",
                    detail: "One dominant session surface remains primary. Bounded typed-turn request production lives in lawful explicit-ready / open / active posture while bounded soft-closed posture remains limited to explicit resume affordance, archived recent slice, and bounded PH1.M `resume context` only, bounded suspended posture remains limited to hard full takeover, suspended-status explanation, and allowed next step only, and bounded recovery posture remains limited to inline restriction or quarantine takeover plus reread-authoritative-state / canonical-retry / failure-detail visibility only."
                )
            }
            .padding(24)
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .background(Color(.systemGroupedBackground))
        .onChange(of: router.latestContext) { _, newContext in
            guard let newContext else {
                return
            }

            activeSessionActiveContext = nil
            activeSessionSoftClosedContext = nil
            activeSessionSuspendedContext = nil
            activeSessionOpenContext = nil
            clearInterruptResponseState()
            activeContext = newContext
            displayState = .onboardingEntryActive
        }
        .onChange(of: router.latestSessionActiveVisibleContext) { _, newContext in
            guard let newContext else {
                return
            }

            activeContext = nil
            activeSessionSoftClosedContext = nil
            activeSessionSuspendedContext = nil
            activeSessionOpenContext = nil
            clearInterruptResponseState()
            activeSessionActiveContext = newContext
            displayState = newContext.recoveryDisplayState
                ?? newContext.interruptDisplayState
                ?? .sessionActiveVisible
        }
        .onChange(of: router.latestSessionSoftClosedVisibleContext) { _, newContext in
            guard let newContext else {
                return
            }

            activeContext = nil
            activeSessionActiveContext = nil
            activeSessionSuspendedContext = nil
            activeSessionOpenContext = nil
            clearInterruptResponseState()
            activeSessionSoftClosedContext = newContext
            displayState = newContext.recoveryDisplayState ?? .sessionSoftClosedVisible
        }
        .onChange(of: router.latestSessionSuspendedVisibleContext) { _, newContext in
            guard let newContext else {
                return
            }

            activeContext = nil
            activeSessionActiveContext = nil
            activeSessionSoftClosedContext = nil
            activeSessionOpenContext = nil
            clearInterruptResponseState()
            activeSessionSuspendedContext = newContext
            displayState = .sessionSuspendedVisible
        }
        .onChange(of: router.latestSessionOpenVisibleContext) { _, newContext in
            guard let newContext else {
                return
            }

            activeContext = nil
            activeSessionActiveContext = nil
            activeSessionSoftClosedContext = nil
            activeSessionSuspendedContext = nil
            clearInterruptResponseState()
            activeSessionOpenContext = newContext
            displayState = newContext.recoveryDisplayState ?? .sessionOpenVisible
        }
        .onChange(of: displayState) { _, newState in
            if newState == .onboardingEntryActive
                || newState == .sessionSoftClosedVisible
                || newState == .sessionSuspendedVisible
                || newState == .recovering
                || newState == .degradedRecovery
                || newState == .quarantinedLocalState
                || newState == .interruptVisible {
                explicitVoiceController.haltCaptureSession()
            }
        }
        .onDisappear {
            explicitVoiceController.haltCaptureSession()
        }
    }

    private var headerCard: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Selene iPhone")
                .font(.largeTitle.weight(.bold))

            Text("First-class, non-authority")
                .font(.headline)

            VStack(alignment: .leading, spacing: 8) {
                posturePill("EXPLICIT_ONLY")
                posturePill("Cloud authoritative")
                posturePill("No wake parity claimed")
                posturePill("No side-button producer claimed")
                posturePill("No autonomous unlock")
            }

            Text("Bounded explicit-entry shell for governed app-open / invite-open rendering only.")
                .font(.subheadline)
                .foregroundStyle(.secondary)

            Text("H89 preserves the H79 recent thread window, the H83 typed-turn request production posture, the H84 explicit voice-turn request production posture, the H80 history side-drawer recall, the H81 System Activity operational queue with separate Pending and Failed visibility, the H82 Needs Attention actionable queue, the H74-H77 takeover surfaces, the H85 bounded `SESSION_OPEN_VISIBLE` current session banner plus attach-outcome continuity seam, the H86 bounded `SESSION_ACTIVE_VISIBLE` live dual transcript plus current governed-output summary seam, the H87 bounded `SESSION_SOFT_CLOSED_VISIBLE` explicit resume affordance plus archived recent slice plus bounded PH1.M `resume context`, the H88 bounded `SESSION_SUSPENDED_VISIBLE` hard full takeover, suspended-status explanation, and allowed next step only, and now also adds bounded `RECOVERING`, `DEGRADED_RECOVERY`, and `QUARANTINED_LOCAL_STATE` recovery posture.")
                .font(.subheadline)
                .foregroundStyle(.secondary)
        }
    }

    private var displayStateCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 10) {
                Text(displayState.title)
                    .font(.headline.monospaced())

                Text(displayState.detail)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("This shell remains session-bound and cloud-authoritative for onboarding, session truth, identity, governance, runtime law, authoritative transcript state, bounded `NextAllowedActions`, archived recent slice truth, bounded PH1.M `resume context`, and bounded recovery posture while typed and explicit voice surfaces produce bounded explicit turn requests only where those surfaces remain lawful.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Display State")
                .font(.headline)
        }
    }

    private var explicitEntryReadyCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 16) {
                Text("Waiting for lawful app-open / invite-open ingress.")
                    .font(.headline)

                Text("H88 keeps `EXPLICIT_ENTRY_READY` as the bounded explicit-entry surface when no lawful `SESSION_OPEN_VISIBLE`, `SESSION_ACTIVE_VISIBLE`, `SESSION_SOFT_CLOSED_VISIBLE`, or `SESSION_SUSPENDED_VISIBLE` route is active. Recent thread, typed input, explicit voice, history recall, `System Activity`, and `Needs Attention` remain bounded, `EXPLICIT_ONLY`, session-bound, and cloud-authoritative while typed input and explicit voice continue to produce bounded explicit turn requests.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                recentThreadWindowCard
                typedInputAffordanceCard
                explicitVoiceEntryAffordanceCard
                historySideDrawerCard
                systemActivityQueueCard
                needsAttentionQueueCard

                Text("No invite activation, no onboarding mutation, and no local authoritative transcript mutation occur locally. Typed-turn request production stays bounded to the canonical explicit text path and explicit voice-turn request production stays bounded to foreground user-initiated capture only.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("EXPLICIT_ENTRY_READY")
                .font(.headline.monospaced())
        }
    }

    private func recoveringVisibleCard(_ surface: RecoveryVisibleSurface) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 16) {
                Text("RECOVERING")
                    .font(.headline)

                Text("H89 adds bounded inline recovery restriction while the lawful main session surface remains visible and cloud-authored recovery posture is reread from canonical session transport only.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                recoveryVisibleSessionSurface(surface)
                recoveryPostureCard(surface, state: .recovering)
                lawfulRecoveryActionsCard(.recovering)
            }
        } label: {
            Text("RECOVERING")
                .font(.headline.monospaced())
        }
    }

    private func degradedRecoveryVisibleCard(_ surface: RecoveryVisibleSurface) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 16) {
                Text("DEGRADED_RECOVERY")
                    .font(.headline)

                Text("H89 adds bounded degraded recovery restriction while the lawful main session surface remains visible and cloud-authored recovery posture limits normal interaction.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                recoveryVisibleSessionSurface(surface)
                recoveryPostureCard(surface, state: .degradedRecovery)
                lawfulRecoveryActionsCard(.degradedRecovery)
            }
        } label: {
            Text("DEGRADED_RECOVERY")
                .font(.headline.monospaced())
        }
    }

    private func quarantinedLocalStateCard(_ surface: RecoveryVisibleSurface) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 16) {
                Text("QUARANTINED_LOCAL_STATE")
                    .font(.headline)

                Text("H89 adds a bounded hard takeover when quarantine makes normal interaction unlawful and the shell must reread authoritative state before any canonical retry path is reconsidered.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(spacing: 8) {
                    posturePill("EXPLICIT_ONLY")
                    posturePill(surface.sourceSurfaceTitle)
                    posturePill("Cloud authoritative")
                }

                recoveryPostureCard(surface, state: .quarantinedLocalState)
                lawfulRecoveryActionsCard(.quarantinedLocalState)
            }
        } label: {
            Text("QUARANTINED_LOCAL_STATE")
                .font(.headline.monospaced())
        }
    }

    @ViewBuilder
    private func recoveryVisibleSessionSurface(_ surface: RecoveryVisibleSurface) -> some View {
        switch surface {
        case .sessionOpen(let context):
            currentSessionBannerCard(context)
            sessionAttachOutcomeContinuityCard(context)
        case .sessionActive(let context):
            liveDualTranscriptCard(context)
            currentTurnEnvelopeCard(context)
            currentGovernedOutputSummaryCard(context)
        case .sessionSoftClosed(let context):
            archivedRecentSliceCard(context)
            resumeContextCard(context)
        }
    }

    private func recoveryPostureCard(
        _ surface: RecoveryVisibleSurface,
        state: ShellDisplayState
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text(state.title)
                    .font(.headline.monospaced())

                HStack(spacing: 8) {
                    posturePill("EXPLICIT_ONLY")
                    posturePill(surface.sourceSurfaceTitle)
                    posturePill("Cloud authoritative")
                }

                ForEach(surface.recoveryPostureRows) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.label)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 190, alignment: .leading)

                        Text(row.value)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                Text("Overlays change posture, not ownership.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local override, no trust in stale cache, no hidden replay.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Recovery posture")
                .font(.headline)
        }
    }

    private func lawfulRecoveryActionsCard(_ state: ShellDisplayState) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text(lawfulRecoveryActionSummary(for: state))
                    .frame(maxWidth: .infinity, alignment: .leading)

                Button("Reread authoritative state") {}
                    .buttonStyle(.borderedProminent)
                    .disabled(true)

                Button("Retry only through canonical entry path") {}
                    .buttonStyle(.bordered)
                    .disabled(true)

                Button("Inspect failure details") {}
                    .buttonStyle(.bordered)
                    .disabled(true)
            }
        } label: {
            Text("Lawful recovery actions")
                .font(.headline)
        }
    }

    private func lawfulRecoveryActionSummary(for state: ShellDisplayState) -> String {
        switch state {
        case .recovering:
            return "Recovery remains active cloud-side, so bounded reread and canonical retry posture stay visible while normal local interaction is restricted."
        case .degradedRecovery:
            return "Degraded recovery remains active cloud-side, so bounded reread and canonical retry posture stay visible while normal local interaction is further restricted."
        case .quarantinedLocalState:
            return "Quarantine removes lawful normal interaction from the visible surface until authoritative state is reread and the canonical recovery path clears cloud-side."
        default:
            return "Only bounded reread, canonical retry posture, and failure-detail inspection remain visible here."
        }
    }

    private func interruptVisibleCard(_ context: SessionActiveVisibleContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 16) {
                Text("Interrupt continuity")
                    .font(.headline)

                Text("H90 adds bounded `INTERRUPT_VISIBLE` inline interruption continuity posture while the lawful main active-session surface remains visible and cloud-authored continuity truth is rendered without local interrupt authority.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(spacing: 8) {
                    posturePill("EXPLICIT_ONLY")
                    posturePill("SESSION_ACTIVE_VISIBLE")
                    posturePill("Cloud authoritative")
                }

                liveDualTranscriptCard(context)
                currentTurnEnvelopeCard(context)
                currentGovernedOutputSummaryCard(context)
                interruptContinuityCard(context)
                interruptLawfulActionsCard(context)
                if context.hasInterruptResponseProductionSurface {
                    interruptResponseProductionCard(context)
                }
                recentThreadWindowCard
                historySideDrawerCard
                systemActivityQueueCard
                needsAttentionQueueCard
            }
        } label: {
            Text("INTERRUPT_VISIBLE")
                .font(.headline.monospaced())
        }
    }

    private func interruptContinuityCard(_ context: SessionActiveVisibleContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Accepted interrupt posture")
                    .font(.headline)

                ForEach(context.interruptContinuityRows) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.label)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 230, alignment: .leading)

                        Text(row.value)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                Text(context.acceptedInterruptPostureSummary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                if context.hasLawfulInterruptSubjectReferences {
                    interruptSubjectReferencesCard(
                        activeSubjectRef: context.activeSubjectRef,
                        interruptedSubjectRef: context.interruptedSubjectRef
                    )
                }

                if let returnCheckExpiresAt = context.returnCheckExpiresAt,
                   context.hasLawfulInterruptReturnCheckExpiry {
                    interruptReturnCheckExpiryCard(returnCheckExpiresAt)
                }

                if let resumeBufferLive = context.resumeBufferLive,
                   context.hasLawfulInterruptResumeBufferLive {
                    interruptResumeBufferLiveCard(resumeBufferLive)
                }

                if let resumeBufferAnswerID = context.resumeBufferAnswerID,
                   context.hasLawfulInterruptResumeBufferAnswerID {
                    interruptResumeBufferAnswerIDCard(resumeBufferAnswerID)
                }

                if let resumeBufferSpokenPrefix = context.resumeBufferSpokenPrefix,
                   context.hasLawfulInterruptResumeBufferSpokenPrefix {
                    interruptResumeBufferSpokenPrefixCard(resumeBufferSpokenPrefix)
                }

                if let resumeBufferUnsaidRemainder = context.resumeBufferUnsaidRemainder,
                   context.hasLawfulInterruptResumeBufferUnsaidRemainder {
                    interruptResumeBufferUnsaidRemainderCard(resumeBufferUnsaidRemainder)
                }

                if let resumeBufferTopicHint = context.resumeBufferTopicHint,
                   context.hasLawfulInterruptResumeBufferTopicHint {
                    interruptResumeBufferTopicHintCard(resumeBufferTopicHint)
                }

                if let ttsResumeSnapshotAnswerID = context.ttsResumeSnapshotAnswerID,
                   context.hasLawfulInterruptTtsResumeSnapshotAnswerID {
                    interruptTtsResumeSnapshotAnswerIDCard(ttsResumeSnapshotAnswerID)
                }

                if let ttsResumeSnapshotSpokenCursorByte = context.ttsResumeSnapshotSpokenCursorByte,
                   context.hasLawfulInterruptTtsResumeSnapshotSpokenCursorByte {
                    interruptTtsResumeSnapshotSpokenCursorByteCard(
                        ttsResumeSnapshotSpokenCursorByte
                    )
                }

                if let ttsResumeSnapshotResponseText = context.ttsResumeSnapshotResponseText,
                   context.hasLawfulInterruptTtsResumeSnapshotResponseText {
                    interruptTtsResumeSnapshotResponseTextCard(
                        ttsResumeSnapshotResponseText
                    )
                }

                if let ttsResumeSnapshotTopicHint = context.ttsResumeSnapshotTopicHint,
                   context.hasLawfulInterruptTtsResumeSnapshotTopicHint {
                    interruptTtsResumeSnapshotTopicHintCard(ttsResumeSnapshotTopicHint)
                }

                if let interruptSubjectRelationConfidence = context.interruptSubjectRelationConfidence,
                   context.hasLawfulInterruptSubjectRelationConfidence {
                    interruptSubjectRelationConfidenceCard(interruptSubjectRelationConfidence)
                }

                if context.shouldPromptReturnCheck {
                    Text("Do you still want to continue the previous topic?")
                        .font(.subheadline.weight(.semibold))
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            }
        } label: {
            Text("Interrupt continuity")
                .font(.headline)
        }
    }

    private func interruptLawfulActionsCard(_ context: SessionActiveVisibleContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Interrupt posture is rendered, not authored.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local interrupt law, no fake resume authority, no silent discard.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

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
            }
        } label: {
            Text("Lawful interrupt actions")
                .font(.headline)
        }
    }

    private func interruptResponseProductionCard(_ context: SessionActiveVisibleContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Bounded continuity response production only")
                    .frame(maxWidth: .infinity, alignment: .leading)

                if context.hasInterruptResponseConflict {
                    Text("Authoritative interruption truth exposed both clarify-directive detail and a return check, so this shell fails closed and keeps continuity response production read-only until the cloud narrows to one lawful path.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    interruptClarifyDirectiveCard(context, productionEnabled: false)
                    interruptReturnCheckResponseCard(context, productionEnabled: false)
                } else if context.hasLawfulInterruptClarifyDirective {
                    interruptClarifyDirectiveCard(
                        context,
                        productionEnabled: interruptResponsePendingRequest == nil
                    )
                } else if context.returnCheckPending == true {
                    interruptReturnCheckResponseCard(
                        context,
                        productionEnabled: interruptResponsePendingRequest == nil
                    )
                }
            }
        } label: {
            Text("Continuity response production")
                .font(.headline)
        }
    }

    private func interruptClarifyDirectiveCard(
        _ context: SessionActiveVisibleContext,
        productionEnabled: Bool
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored clarify directive")
                    .font(.headline)

                if let interruptClarifyQuestion = context.interruptClarifyQuestion {
                    Text(interruptClarifyQuestion)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("Accepted answer formats")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                ForEach(context.interruptAcceptedAnswerFormats, id: \.self) { answerFormat in
                    Text(answerFormat)
                        .font(.body.monospaced())
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                if let interruptClarifyWhatIsMissing = context.interruptClarifyWhatIsMissing {
                    interruptClarifyBoundaryCard(interruptClarifyWhatIsMissing)
                }

                if context.hasLawfulInterruptClarifyAmbiguityFlags {
                    interruptClarifyAmbiguityCard(context.interruptClarifyAmbiguityFlags)
                }

                if context.hasLawfulInterruptClarifyRoutingHints {
                    interruptClarifyRoutingCard(context.interruptClarifyRoutingHints)
                }

                if let interruptClarifyRequiresConfirmation = context.interruptClarifyRequiresConfirmation,
                   context.hasLawfulInterruptClarifyRequiresConfirmation {
                    interruptClarifyConfirmationCard(interruptClarifyRequiresConfirmation)
                }

                if let interruptClarifySensitivityLevel = context.interruptClarifySensitivityLevel,
                   context.hasLawfulInterruptClarifySensitivityLevel {
                    interruptClarifySensitivityCard(interruptClarifySensitivityLevel)
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
        } label: {
            Text("Cloud-authored clarify directive")
                .font(.headline)
        }
    }

    private func interruptClarifyBoundaryCard(_ missingField: String) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("One question, one missing field")
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Cloud-authored field key only")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .top, spacing: 12) {
                    Text("Missing field")
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                        .frame(width: 140, alignment: .leading)

                    Text(missingField)
                        .font(.body.monospaced())
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("No local field inference, no multi-field bundling.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Clarify boundary")
                .font(.headline)
        }
    }

    private func interruptClarifyAmbiguityCard(_ ambiguityFlags: [String]) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored ambiguity evidence only")
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Ambiguity flags")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                ForEach(ambiguityFlags, id: \.self) { ambiguityFlag in
                    Text(ambiguityFlag)
                        .font(.body.monospaced())
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("Exact cloud-authored flags only")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local ambiguity inference, no local rewrite.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Clarify ambiguity")
                .font(.headline)
        }
    }

    private func interruptClarifyRoutingCard(_ routingHints: [String]) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored routing evidence only")
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Routing hints")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                ForEach(routingHints, id: \.self) { routingHint in
                    Text(routingHint)
                        .font(.body.monospaced())
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("Exact cloud-authored hints only")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local routing guidance, no local gate bypass.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Clarify routing")
                .font(.headline)
        }
    }

    private func interruptClarifyConfirmationCard(_ requiresConfirmation: Bool) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored confirmation evidence only")
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Confirmation posture")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .top, spacing: 12) {
                    Text("Requires confirmation")
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                        .frame(width: 140, alignment: .leading)

                    Text(requiresConfirmation ? "true" : "false")
                        .font(.body.monospaced())
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("Exact cloud-authored confirmation truth only")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local confirmation law, no local execution unlock.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Clarify confirmation")
                .font(.headline)
        }
    }

    private func interruptClarifySensitivityCard(_ sensitivityLevel: String) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored sensitivity evidence only")
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Sensitivity posture")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .top, spacing: 12) {
                    Text("Sensitivity level")
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                        .frame(width: 140, alignment: .leading)

                    Text(sensitivityLevel)
                        .font(.body.monospaced())
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("Exact cloud-authored sensitivity level only")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local sensitivity policy, no local authority upgrade.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Clarify sensitivity")
                .font(.headline)
        }
    }

    private func interruptSubjectReferencesCard(
        activeSubjectRef: String?,
        interruptedSubjectRef: String?
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored continuity subject evidence only")
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Subject posture")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                if let activeSubjectRef {
                    HStack(alignment: .top, spacing: 12) {
                        Text("Active subject ref")
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 140, alignment: .leading)

                        Text(activeSubjectRef)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                if let interruptedSubjectRef {
                    HStack(alignment: .top, spacing: 12) {
                        Text("Interrupted subject ref")
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 140, alignment: .leading)

                        Text(interruptedSubjectRef)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                Text("Exact cloud-authored subject refs only")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local subject binding, no local dispatch unlock.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Subject references")
                .font(.headline)
        }
    }

    private func interruptSubjectRelationConfidenceCard(_ confidence: String) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored continuity confidence evidence only")
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Confidence posture")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .top, spacing: 12) {
                    Text("Relation confidence")
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                        .frame(width: 140, alignment: .leading)

                    Text(confidence)
                        .font(.body.monospaced())
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("Exact cloud-authored confidence only")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local threshold law, no local dispatch unlock.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Subject relation confidence")
                .font(.headline)
        }
    }

    private func interruptReturnCheckExpiryCard(_ returnCheckExpiresAt: String) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored return-check expiry evidence only")
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Expiry posture")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .top, spacing: 12) {
                    Text("Return-check expires at")
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                        .frame(width: 140, alignment: .leading)

                    Text(returnCheckExpiresAt)
                        .font(.body.monospaced())
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("Exact cloud-authored expiry only")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local countdown, no local dispatch unlock.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Return-check expiry")
                .font(.headline)
        }
    }

    private func interruptResumeBufferLiveCard(_ resumeBufferLive: Bool) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored resume-buffer liveness evidence only")
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Resume posture")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .top, spacing: 12) {
                    Text("Resume buffer live")
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                        .frame(width: 140, alignment: .leading)

                    Text(resumeBufferLive ? "true" : "false")
                        .font(.body.monospaced())
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("Exact cloud-authored liveness truth only")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local resume authoring, no local dispatch unlock.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Resume buffer")
                .font(.headline)
        }
    }

    private func interruptResumeBufferAnswerIDCard(_ resumeBufferAnswerID: String) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored resume-buffer answer-ID evidence only")
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Answer posture")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .top, spacing: 12) {
                    Text("Resume buffer answer ID")
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                        .frame(width: 140, alignment: .leading)

                    Text(resumeBufferAnswerID)
                        .font(.body.monospaced())
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("Exact cloud-authored answer ID only")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local resume authoring, no local topic synthesis, and no local dispatch unlock.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Resume answer ID")
                .font(.headline)
        }
    }

    private func interruptResumeBufferSpokenPrefixCard(_ resumeBufferSpokenPrefix: String) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored resume-buffer spoken-prefix evidence only")
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Prefix posture")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .top, spacing: 12) {
                    Text("Resume buffer spoken prefix")
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                        .frame(width: 140, alignment: .leading)

                    Text(resumeBufferSpokenPrefix)
                        .font(.body.monospaced())
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("Exact cloud-authored spoken prefix only")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local resume authoring, no local synthesis of the remaining response, and no local dispatch unlock.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Resume spoken prefix")
                .font(.headline)
        }
    }

    private func interruptResumeBufferUnsaidRemainderCard(_ resumeBufferUnsaidRemainder: String) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored resume-buffer unsaid-remainder evidence only")
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Remainder posture")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .top, spacing: 12) {
                    Text("Resume buffer unsaid remainder")
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                        .frame(width: 140, alignment: .leading)

                    Text(resumeBufferUnsaidRemainder)
                        .font(.body.monospaced())
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("Exact cloud-authored unsaid remainder only")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local resume authoring, no local completion of the remaining response, and no local dispatch unlock.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Resume unsaid remainder")
                .font(.headline)
        }
    }

    private func interruptResumeBufferTopicHintCard(_ resumeBufferTopicHint: String) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored resume-buffer topic-hint evidence only")
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Topic posture")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .top, spacing: 12) {
                    Text("Resume buffer topic hint")
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                        .frame(width: 140, alignment: .leading)

                    Text(resumeBufferTopicHint)
                        .font(.body.monospaced())
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("Exact cloud-authored topic hint only")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local topic synthesis, no local dispatch unlock.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Resume topic hint")
                .font(.headline)
        }
    }

    private func interruptTtsResumeSnapshotSpokenCursorByteCard(
        _ ttsResumeSnapshotSpokenCursorByte: String
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored TTS resume snapshot cursor evidence only")
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Cursor posture")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .top, spacing: 12) {
                    Text("TTS resume snapshot cursor byte")
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                        .frame(width: 140, alignment: .leading)

                    Text(ttsResumeSnapshotSpokenCursorByte)
                        .font(.body.monospaced())
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("Exact cloud-authored snapshot cursor only")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local resume authoring, no local playback math authority, no local response synthesis, and no local dispatch unlock.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("TTS snapshot cursor")
                .font(.headline)
        }
    }

    private func interruptTtsResumeSnapshotAnswerIDCard(
        _ ttsResumeSnapshotAnswerID: String
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored TTS resume snapshot answer-ID evidence only")
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Snapshot answer posture")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .top, spacing: 12) {
                    Text("TTS resume snapshot answer ID")
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                        .frame(width: 140, alignment: .leading)

                    Text(ttsResumeSnapshotAnswerID)
                        .font(.body.monospaced())
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("Exact cloud-authored snapshot answer ID only")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local resume authoring, no local snapshot linkage authority, no local response synthesis, and no local dispatch unlock.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("TTS snapshot answer ID")
                .font(.headline)
        }
    }

    private func interruptTtsResumeSnapshotResponseTextCard(
        _ ttsResumeSnapshotResponseText: String
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored TTS resume snapshot response-text evidence only")
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Snapshot response posture")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .top, spacing: 12) {
                    Text("TTS resume snapshot response text")
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                        .frame(width: 140, alignment: .leading)

                    Text(ttsResumeSnapshotResponseText)
                        .font(.body.monospaced())
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("Exact cloud-authored snapshot response text only")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local resume authoring, no local cursor math authority, no local response synthesis, no local completion authority, and no local dispatch unlock.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("TTS snapshot response text")
                .font(.headline)
        }
    }

    private func interruptTtsResumeSnapshotTopicHintCard(
        _ ttsResumeSnapshotTopicHint: String
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Cloud-authored TTS resume snapshot topic-hint evidence only")
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Snapshot topic posture")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .top, spacing: 12) {
                    Text("tts_resume_snapshot_topic_hint")
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                        .frame(width: 140, alignment: .leading)

                    Text(ttsResumeSnapshotTopicHint)
                        .font(.body.monospaced())
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("Exact cloud-authored snapshot topic hint only")
                    .font(.subheadline.weight(.semibold))
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local topic synthesis, no local resume authoring, no local response synthesis, no local dispatch unlock, no local authority upgrade, no wake parity, no side-button producer, and no autonomous unlock.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("TTS snapshot topic hint")
                .font(.headline)
        }
    }

    private func interruptReturnCheckResponseCard(
        _ context: SessionActiveVisibleContext,
        productionEnabled: Bool
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Return-check response")
                    .font(.headline)

                Text("Do you still want to continue the previous topic?")
                    .font(.subheadline.weight(.semibold))
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
        } label: {
            Text("Return-check response")
                .font(.headline)
        }
    }

    private func sessionOpenVisibleCard(_ context: SessionOpenVisibleContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 16) {
                Text("Ready for next explicit turn")
                    .font(.headline)

                Text("H85 adds a bounded native `SESSION_OPEN_VISIBLE` surface aligned to `SessionState::Open`, `SessionAttachOutcome`, and `session_attach_outcome` carried inside `RuntimeExecutionEnvelope` while preserving the H79-H84 surfaces and keeping the shell `EXPLICIT_ONLY`, session-bound, and cloud-authoritative.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                currentSessionBannerCard(context)
                sessionAttachOutcomeContinuityCard(context)
                recentThreadWindowCard
                typedInputAffordanceCard
                explicitVoiceEntryAffordanceCard
                historySideDrawerCard
                systemActivityQueueCard
                needsAttentionQueueCard

                Text("No local promotion to `Active`, no hidden new session, no local session resurrection, no wake parity claim, no proven live side-button producer claim, and no autonomous unlock are introduced by this surface.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Button("Return to EXPLICIT_ENTRY_READY") {
                    activeSessionOpenContext = nil
                    displayState = .explicitEntryReady
                }
                .buttonStyle(.borderedProminent)
            }
        } label: {
            Text("SESSION_OPEN_VISIBLE")
                .font(.headline.monospaced())
        }
    }

    private func sessionSoftClosedVisibleCard(_ context: SessionSoftClosedVisibleContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 16) {
                Text("Resume the selected thread explicitly")
                    .font(.headline)

                Text("H87 adds a bounded native `SESSION_SOFT_CLOSED_VISIBLE` surface aligned to `SessionState::SoftClosed`, archived conversation truth, and bounded PH1.M `resume context` while preserving the H85 open-session and H86 active-session route-seeding contracts.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(spacing: 8) {
                    posturePill("EXPLICIT_ONLY")
                    posturePill("SessionState::SoftClosed")
                    posturePill("Cloud authoritative")
                }

                ForEach(context.sessionRows) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.label)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 170, alignment: .leading)

                        Text(row.value)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                archivedRecentSliceCard(context)
                resumeContextCard(context)

                Text("Visual reset may clear the screen, but archive truth remains durable.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local auto-reopen from cache alone, no typed-turn request production, no explicit voice-turn request production, no local session resurrection, and no local decision shortcuts are introduced by this surface.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(spacing: 12) {
                    Button("Resume the selected thread explicitly") {}
                        .buttonStyle(.borderedProminent)
                        .disabled(true)

                    Button("Return to EXPLICIT_ENTRY_READY") {
                        activeSessionSoftClosedContext = nil
                        displayState = .explicitEntryReady
                    }
                    .buttonStyle(.bordered)
                }
            }
        } label: {
            Text("SESSION_SOFT_CLOSED_VISIBLE")
                .font(.headline.monospaced())
        }
    }

    private func sessionSuspendedVisibleCard(_ context: SessionSuspendedVisibleContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 16) {
                Text("Session suspended cloud-side")
                    .font(.headline)

                Text("H88 adds a bounded native `SESSION_SUSPENDED_VISIBLE` surface aligned to `SessionState::Suspended`, bounded `NextAllowedActions`, and optional bounded `PersistenceRecoveryMode` / `ReconciliationDecision` explanation while preserving the H85 open-session, H86 active-session, and H87 soft-closed route-seeding contracts.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(spacing: 8) {
                    posturePill("EXPLICIT_ONLY")
                    posturePill("SessionState::Suspended")
                    posturePill("Cloud authoritative")
                }

                suspendedStatusCard(context)
                allowedNextStepCard(context)

                Text("Suspended posture is cloud-authored.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("No local unsuspend and no silent continuation.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("This hard full takeover keeps typed-turn request production, explicit voice-turn request production, local session resurrection, and local decision shortcuts out of the bounded suspended surface itself.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Button("Return to EXPLICIT_ENTRY_READY") {
                    activeSessionSuspendedContext = nil
                    displayState = .explicitEntryReady
                }
                .buttonStyle(.borderedProminent)
            }
        } label: {
            Text("SESSION_SUSPENDED_VISIBLE")
                .font(.headline.monospaced())
        }
    }

    private func archivedRecentSliceCard(_ context: SessionSoftClosedVisibleContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                ForEach(context.archivedRecentSliceEntries) { entry in
                    VStack(alignment: .leading, spacing: 8) {
                        HStack(alignment: .firstTextBaseline, spacing: 12) {
                            Text(entry.speaker)
                                .font(.headline)

                            Spacer()

                            Text(entry.posture)
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)
                        }

                        Text(entry.body)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text(entry.detail)
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .padding(12)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color(.secondarySystemGroupedBackground))
                    .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
                }

                Text("Archived recent slice remains distinct from PH1.M memory and stays bounded to durable archive truth only.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Archived recent slice")
                .font(.headline)
        }
    }

    private func resumeContextCard(_ context: SessionSoftClosedVisibleContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                ForEach(context.resumeContextRows) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.label)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 170, alignment: .leading)

                        Text(row.value)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                if context.resumeSummaryBullets.isEmpty {
                    Text("No bounded `resume_summary_bullets` were provided for this soft-closed preview.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                } else {
                    ForEach(Array(context.resumeSummaryBullets.prefix(3).enumerated()), id: \.offset) { index, bullet in
                        HStack(alignment: .top, spacing: 12) {
                            Text("\(index + 1).")
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)
                                .frame(width: 24, alignment: .leading)

                            Text(bullet)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }

                Text("Resume context remains bounded PH1.M output only.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Resume context")
                .font(.headline)
        }
    }

    private func suspendedStatusCard(_ context: SessionSuspendedVisibleContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                ForEach(context.suspendedStatusRows) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.label)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 190, alignment: .leading)

                        Text(row.value)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                Text("Suspended status remains bounded to session identity, optional recovery explanation, and optional reconciliation explanation only.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Suspended status")
                .font(.headline)
        }
    }

    private func allowedNextStepCard(_ context: SessionSuspendedVisibleContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                ForEach(context.allowedNextStepRows) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.label)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 230, alignment: .leading)

                        Text(row.value)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                Text(context.allowedNextStepSummary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Allowed next step remains authoritative posture only; this surface does not produce turns, unsuspend locally, or continue silently.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Allowed next step")
                .font(.headline)
        }
    }

    private func sessionActiveVisibleCard(_ context: SessionActiveVisibleContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 16) {
                Text("Live authoritative active session")
                    .font(.headline)

                Text("H86 adds a bounded native `SESSION_ACTIVE_VISIBLE` surface aligned to `SessionState::Active`, current `turn_id`, `RuntimeExecutionEnvelope`, the authoritative envelope-state family, and append-only `conversation_ledger` truth while preserving the H85 `session_state`, `session_id`, and `session_attach_outcome` route-seeding contract for `SESSION_OPEN_VISIBLE`.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                liveDualTranscriptCard(context)
                currentTurnEnvelopeCard(context)
                currentGovernedOutputSummaryCard(context)
                recentThreadWindowCard
                typedInputAffordanceCard
                explicitVoiceEntryAffordanceCard
                historySideDrawerCard
                systemActivityQueueCard
                needsAttentionQueueCard

                Text("No explicit interrupt control, no local turn authority, no local decision shortcuts, no heavy governed-content hydration, no wake parity claim, no proven live side-button producer claim, and no autonomous unlock are introduced by this surface.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Button("Return to EXPLICIT_ENTRY_READY") {
                    activeSessionActiveContext = nil
                    displayState = .explicitEntryReady
                }
                .buttonStyle(.borderedProminent)
            }
        } label: {
            Text("SESSION_ACTIVE_VISIBLE")
                .font(.headline.monospaced())
        }
    }

    private func currentSessionBannerCard(_ context: SessionOpenVisibleContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Ready for next explicit turn")
                    .font(.headline)

                HStack(spacing: 8) {
                    posturePill("EXPLICIT_ONLY")
                    posturePill("SessionState::Open")
                    posturePill("Cloud authoritative")
                }

                ForEach(context.bannerRows) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.label)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 160, alignment: .leading)

                        Text(row.value)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                Text("Current session stays open, session-bound, and ready for the next explicit turn while authoritative transcript acceptance, response, and session lifecycle ownership remain cloud-side.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Current session banner")
                .font(.headline)
        }
    }

    private func sessionAttachOutcomeContinuityCard(_ context: SessionOpenVisibleContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                ForEach(context.attachOutcomeRows) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.label)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 170, alignment: .leading)

                        Text(row.value)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                Text("Attach outcome changes inline continuity labeling only.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Session attach outcome")
                .font(.headline)
        }
    }

    private func liveDualTranscriptCard(_ context: SessionActiveVisibleContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                HStack(spacing: 8) {
                    posturePill("EXPLICIT_ONLY")
                    posturePill("Append-only conversation_ledger")
                    posturePill("Cloud authoritative")
                }

                ForEach(context.liveTranscriptEntries) { entry in
                    VStack(alignment: .leading, spacing: 8) {
                        HStack(alignment: .firstTextBaseline, spacing: 12) {
                            Text(entry.speaker)
                                .font(.headline)

                            Spacer()

                            Text(entry.posture)
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)
                        }

                        Text(entry.body)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text(entry.detail)
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .padding(12)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color(.secondarySystemGroupedBackground))
                    .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
                }

                Text("Live transcript remains text-visible even when spoken.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Live dual transcript")
                .font(.headline)
        }
    }

    private func currentTurnEnvelopeCard(_ context: SessionActiveVisibleContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                HStack(spacing: 8) {
                    posturePill("RuntimeExecutionEnvelope")
                    posturePill("SessionState::Active")
                    posturePill("Turn-bound")
                }

                ForEach(context.currentTurnEnvelopeRows) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.label)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 170, alignment: .leading)

                        Text(row.value)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                Text("Current turn envelope remains session-bound, cloud-authoritative, and visible only as bounded runtime state while governance, proof, computation, identity, memory, authority, and artifact-trust ownership remain authoritative.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Current turn envelope")
                .font(.headline)
        }
    }

    private func currentGovernedOutputSummaryCard(_ context: SessionActiveVisibleContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                ForEach(context.governedOutputRows) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.label)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 190, alignment: .leading)

                        Text(row.value)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                Text("Current governed output summary remains a bounded summary card only. No eager artifact, chart, report, or heavy-content hydration is introduced in this run.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Current governed output summary")
                .font(.headline)
        }
    }

    private var recentThreadWindowCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Recent thread preview only. This bounded window reflects append-only conversation posture distinct from PH1.M memory and keeps resume context bounded.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                ForEach(recentThreadPreviewEntries) { entry in
                    VStack(alignment: .leading, spacing: 8) {
                        HStack(alignment: .firstTextBaseline, spacing: 12) {
                            Text(entry.speaker)
                                .font(.headline)

                            Spacer()

                            Text(entry.posture)
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)
                        }

                        Text(entry.body)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text(entry.detail)
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .padding(12)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color(.secondarySystemGroupedBackground))
                    .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
                }

                Text("Older history remains bounded recall only in the explicit side drawer below. No local transcript authority or session resurrection is created here.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Recent Thread Window")
                .font(.headline)
        }
    }

    private var typedInputAffordanceCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Composer-style surface now produces a bounded typed-turn request while transcript authority, authoritative acceptance, and authoritative response remain cloud-side.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .center, spacing: 12) {
                    Image(systemName: "text.cursor")
                        .foregroundStyle(.secondary)

                    TextField(
                        "Type a follow-up for canonical text-turn ingress.",
                        text: $typedTurnDraft
                    )
                    .textFieldStyle(.roundedBorder)
                }

                HStack(spacing: 12) {
                    Button("Send typed turn") {
                        submitTypedTurn()
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(trimmedTypedTurnDraft.isEmpty)

                    Button("Clear draft") {
                        typedTurnDraft = ""
                    }
                    .buttonStyle(.bordered)
                    .disabled(typedTurnDraft.isEmpty)
                }

                HStack(spacing: 8) {
                    posturePill("EXPLICIT_ONLY")
                    posturePill("CanonicalTurnModality::Text")
                    posturePill("RawTurnPayload::Text")
                }

                HStack(spacing: 8) {
                    posturePill("SessionResolveMode::ResumeExisting")
                    posturePill("text/plain")
                    posturePill("Session-bound")
                }

                Text("Bounded request production only: `RuntimeCanonicalIngressRequest::turn(...)` stays aligned to `CanonicalTurnModality::Text`, `RawTurnPayload::Text`, `SessionResolveMode::ResumeExisting`, canonical `text/plain` validation, and explicit requested-trigger posture.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Draft validation: trimmed non-empty text only, `text/plain`, \(trimmedTypedTurnDraft.utf8.count) / \(maxTypedTurnBytes) UTF-8 bytes.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                if let pendingRequest = typedTurnPendingRequest {
                    typedTurnPendingRequestCard(pendingRequest)
                }

                if typedTurnFailedRequest != nil {
                    Text("Latest failed typed-turn posture stays visible below in `Failed` until canonical follow-up occurs.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("No local authority, no local assistant output, no onboarding mutation, and no authoritative transcript mutation are introduced by this affordance.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Typed Input Affordance")
                .font(.headline)
        }
    }

    private var explicitVoiceEntryAffordanceCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Explicit voice entry now produces a bounded session-bound explicit voice-turn request only after foreground user initiation. Capture, transcript preview, pending posture, and failed posture remain explicit, bounded, and cloud-authoritative.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .center, spacing: 12) {
                    Image(systemName: "mic.circle")
                        .font(.system(size: 28))
                        .foregroundStyle(.secondary)

                    VStack(alignment: .leading, spacing: 4) {
                        Text("Explicit voice entry")
                            .font(.headline)

                        Text("Bounded foreground capture aligned to `AppVoiceIngressRequest`, `OsVoiceLiveTurnInput`, `RuntimeExecutionEnvelope`, `OsVoiceTurnContext`, `OsVoiceTrigger::Explicit`, `Ph1VoiceIdRequest`, `VoiceIdentityEmbeddingGateProfiles::mvp_v1_phone_first()`, `ios_explicit`, and `voice_context_ios_explicit()`.")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }

                    Spacer()

                    Text(explicitVoiceController.isListening ? "Live now" : "Bounded only")
                        .font(.caption.weight(.semibold))
                        .padding(.horizontal, 10)
                        .padding(.vertical, 6)
                        .background(
                            explicitVoiceController.isListening
                                ? Color.accentColor.opacity(0.16)
                                : Color.secondary.opacity(0.12)
                        )
                        .clipShape(Capsule())
                }

                HStack(spacing: 8) {
                    posturePill("EXPLICIT_ONLY")
                    posturePill("AppVoiceIngressRequest")
                    posturePill("OsVoiceTrigger::Explicit")
                }

                HStack(spacing: 8) {
                    posturePill("Ph1VoiceIdRequest")
                    posturePill("ios_explicit")
                    posturePill("No wake parity")
                }

                VStack(alignment: .leading, spacing: 10) {
                    permissionStateRow(
                        label: "microphone_permission",
                        state: explicitVoiceController.microphonePermission
                    )
                    permissionStateRow(
                        label: "speech_recognition_permission",
                        state: explicitVoiceController.speechRecognitionPermission
                    )
                }

                Text("Explicit foreground user action is required before microphone capture or speech recognition starts. This shell does not begin capture on wake, side-button, or background triggers.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(spacing: 12) {
                    Button("Start explicit voice turn") {
                        explicitVoiceController.startExplicitVoiceTurn()
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(explicitVoiceController.isListening || explicitVoiceController.pendingRequest != nil)

                    Button("Stop capture and prepare voice request") {
                        explicitVoiceController.stopCaptureAndPrepareVoiceTurn()
                    }
                    .buttonStyle(.bordered)
                    .disabled(!explicitVoiceController.isListening)
                }

                if explicitVoiceController.isListening {
                    Text("Listening for explicit voice capture")
                        .font(.headline)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                if !explicitVoiceController.transcriptPreview.isEmpty {
                    explicitVoiceTranscriptPreviewCard
                }

                if let pendingRequest = explicitVoiceController.pendingRequest {
                    explicitVoicePendingRequestCard(pendingRequest)
                }

                if explicitVoiceController.failedRequest != nil {
                    Text("Latest failed explicit voice posture stays visible below in `Failed` until canonical follow-up occurs.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("No side-button producer claim, no wake claim, and no autonomous unlock claim are introduced by this affordance.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Explicit Voice Entry Affordance")
                .font(.headline)
        }
    }

    private var historySideDrawerCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 16) {
                Text("H80 replaces the placeholder History posture with a bounded read-only side drawer. Older recall remains explicit, session-bound, `EXPLICIT_ONLY`, and cloud-authoritative.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(spacing: 8) {
                    posturePill("Read-only side drawer")
                    posturePill("Windowed recall")
                    posturePill("Archived recall only")
                }

                VStack(alignment: .leading, spacing: 10) {
                    Text("Recent visible transcript stays in the main session surface. Older messages remain behind explicit side-drawer recall only, distinct from PH1.M memory and separate from `System Activity` / `Needs Attention` surfaces.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                .padding(12)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color(.secondarySystemGroupedBackground))
                .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))

                incrementalHistoryExpansionCard
                archivedSessionRecallCard

                Text("No full-history eager load, no silent mutation, no cross-session blending, no raw memory-ledger dump, and no local session resurrection occur in this shell.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("History Side Drawer")
                .font(.headline)
        }
    }

    private var incrementalHistoryExpansionCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Windowed incremental recall only. Older transcript remains behind explicit user action and never eager-loads the full conversation locally.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(spacing: 12) {
                    Button("Load older messages") {}
                        .buttonStyle(.bordered)
                        .disabled(true)

                    Button("Show more history") {}
                        .buttonStyle(.bordered)
                        .disabled(true)
                }

                Text("These controls are read-only affordances only. They do not dispatch requests, synthesize local transcript authority, or blur session boundaries.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Incremental History Expansion")
                .font(.headline)
        }
    }

    private var archivedSessionRecallCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Archived session recall remains explicit after close. This surface previews archived recall without resurrecting a local session, synthesizing an active session, or claiming transcript authority.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                archivedSessionRecallRow(
                    sessionID: "archive_recent_explicit_session",
                    summary: "Most recent explicit-entry session remains recallable only as bounded archived history.",
                    detail: "Closed-session recall only; no synthetic reopen, no local authority, and no PH1.M memory dump are introduced here."
                )

                archivedSessionRecallRow(
                    sessionID: "archive_prior_onboarding_window",
                    summary: "Earlier onboarding-adjacent session remains archived and separate from the current visible thread window.",
                    detail: "Archived recall stays cloud-authoritative and session-scoped only; it does not blend cross-session history into the active window."
                )

                Text("Archived recall stays separate from the recent visible thread window, from `System Activity` queue behavior, and from the separate `Needs Attention` actionable queue.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Archived Session Recall")
                .font(.headline)
        }
    }

    private var systemActivityQueueCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 16) {
                Text("H81 replaces the placeholder `System Activity` posture with a bounded read-only operational queue. Persistence acknowledgement, reconciliation decision, broadcast waiting / follow-up / reminder state, sync queue posture, dead-letter posture, and recovery posture remain visible only, separate from transcript history, the recent thread window, the history side drawer, archived recall, and PH1.M memory, while H82 keeps `Needs Attention` as a separate actionable subset below and H83 allows bounded typed-turn pending / failure posture to surface through the existing `Pending` / `Failed` queues.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(spacing: 8) {
                    posturePill("Read-only operational queue")
                    posturePill("Pending visibility")
                    posturePill("Failed visibility")
                }

                ForEach(systemActivityEntries) { entry in
                    operationalQueueEntryRow(entry)
                }

                pendingOperationalQueueCard
                failedOperationalQueueCard

                Text("No manual resend console, no local transport repair authority, no hidden auto-heal claim, no local completion of pending work, and no silent disappearance of failed work occur in this shell.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("System Activity")
                .font(.headline)
        }
    }

    private var pendingOperationalQueueCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("`Pending` remains a separate operational queue from history. It stays visible only, cloud-authoritative, and session-bound while H83 surfaces bounded typed-turn request posture here and H84 also surfaces bounded explicit voice-turn request posture here when an explicit request is awaiting authoritative response.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                if let interruptResponsePendingRequest {
                    interruptResponsePendingRequestCard(interruptResponsePendingRequest)
                }

                ForEach(displayedPendingOperationalEntries) { entry in
                    operationalQueueEntryRow(entry)
                }

                Text("Pending visibility does not complete work locally, resend work, or blur transcript boundaries.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Pending")
                .font(.headline)
        }
    }

    private var failedOperationalQueueCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("`Failed` remains a separate operational queue from history. It stays visible only, cloud-authoritative, and distinct from the current visible thread window while H83 surfaces bounded typed-turn failure posture here and H84 also surfaces bounded explicit voice failure posture here without implying local transport repair authority.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                ForEach(displayedFailedOperationalEntries) { entry in
                    operationalQueueEntryRow(entry)
                }

                Text("Failed visibility does not repair work locally, erase failure posture, or reanimate a local session.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Failed")
                .font(.headline)
        }
    }

    private var needsAttentionQueueCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 16) {
                Text("H82 adds a bounded read-only `Needs Attention` actionable queue. It remains the human-actionable subset of `System Activity` or protected runtime posture, separate from normal thread, plain history, archived recall, the recent thread window, and PH1.M memory. No mixing unresolved operations into normal scrollback and no non-actionable clutter are introduced here.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(spacing: 8) {
                    posturePill("Read-only actionable queue")
                    posturePill("Required human action")
                    posturePill("Separate from transcript history")
                }

                ForEach(needsAttentionEntries) { entry in
                    needsAttentionEntryRow(entry)
                }

                Text("`Open item`, `Acknowledge`, `Retry through canonical path`, `Inspect reason`, and `Complete the required human action` remain action-copy only. No local acknowledge authority, no local retry authority, no local human-action completion authority, and no local transport repair authority exist in this shell.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Needs Attention")
                .font(.headline)
        }
    }

    private func takeoverCard(_ context: ExplicitEntryContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text(context.routeKind.title)
                    .font(.headline)

                Text("Parsed explicit-entry context only. This takeover surface does not activate invites, complete onboarding, bind tokens, or alter cloud authority.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                VStack(alignment: .leading, spacing: 8) {
                    ForEach(context.rows) { row in
                        HStack(alignment: .top, spacing: 12) {
                            Text(row.label)
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)
                                .frame(width: 140, alignment: .leading)

                            Text(row.value)
                                .font(.body.monospaced())
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }

                Divider()

                onboardingOutcomeSummary(context)

                onboardingContinueStatusSummary(context)

                onboardingContinuePromptSummary(context)

                onboardingContinueArtifactAccessSummary(context)

                outcomeListCard(
                    title: "required_fields",
                    items: context.requiredFields,
                    emptyText: "No required_fields were provided in the bounded route context."
                )

                outcomeListCard(
                    title: "required_verification_gates",
                    items: context.requiredVerificationGates,
                    emptyText: "No required_verification_gates were provided in the bounded route context."
                )

                outcomeListCard(
                    title: "remaining_missing_fields",
                    items: context.remainingMissingFields,
                    emptyText: "No remaining_missing_fields were provided in the bounded route context."
                )

                outcomeListCard(
                    title: "remaining_platform_receipt_kinds",
                    items: context.remainingPlatformReceiptKinds,
                    emptyText: "No remaining_platform_receipt_kinds were provided in the bounded route context."
                )

                Button("Return to EXPLICIT_ENTRY_READY") {
                    activeContext = nil
                    displayState = .explicitEntryReady
                }
                .buttonStyle(.borderedProminent)
            }
        } label: {
            Text("ONBOARDING_ENTRY_ACTIVE")
                .font(.headline.monospaced())
        }
    }

    private func onboardingOutcomeSummary(_ context: ExplicitEntryContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 10) {
                Text("Read-only onboarding outcome preview aligned to `AppInviteLinkOpenOutcome` and bounded takeover posture only.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                ForEach(context.onboardingOutcomeRows) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.label)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 180, alignment: .leading)

                        Text(row.value)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                Text("This takeover surface does not activate invites, complete onboarding, bypass verification, or produce runtime requests locally.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Onboarding Outcome")
                .font(.headline)
        }
    }

    private func onboardingContinueStatusSummary(_ context: ExplicitEntryContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 10) {
                Text("Read-only onboarding continue status preview aligned to bounded `AppOnboardingContinueOutcome` status and receipt-state only.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                ForEach(context.onboardingContinueRows) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.label)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 180, alignment: .leading)

                        Text(row.value)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                Text("This H75 surface still preserves current receipt/task status only, while H77 keeps voice_artifact_sync_receipt_ref and access_engine_instance_id in a separate read-only identifier surface.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Onboarding Continue Status")
                .font(.headline)
        }
    }

    private func onboardingContinuePromptSummary(_ context: ExplicitEntryContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 10) {
                Text("Read-only onboarding continue prompt preview aligned to bounded `AppOnboardingContinueOutcome` missing-field prompt state only.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                ForEach(context.onboardingContinuePromptRows) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.label)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 180, alignment: .leading)

                        Text(row.value)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                Text("This H76 surface preserves current missing-field prompt state only and does not submit required fields, advance onboarding, or produce runtime requests locally.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Onboarding Continue Prompt")
                .font(.headline)
        }
    }

    private func onboardingContinueArtifactAccessSummary(_ context: ExplicitEntryContext) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 10) {
                Text("Read-only onboarding continue artifact/access preview aligned to bounded `AppOnboardingContinueOutcome` identifier state only.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                ForEach(context.onboardingContinueArtifactAccessRows) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.label)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 180, alignment: .leading)

                        Text(row.value)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                Text("This H77 surface preserves identifier visibility only and does not start voice-artifact sync, activate access engines, or produce runtime requests locally.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Onboarding Continue Artifact/Access")
                .font(.headline)
        }
    }

    private func outcomeListCard(title: String, items: [String], emptyText: String) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 8) {
                if items.isEmpty {
                    Text(emptyText)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .foregroundStyle(.secondary)
                } else {
                    ForEach(Array(items.enumerated()), id: \.offset) { index, item in
                        HStack(alignment: .top, spacing: 12) {
                            Text("\(index + 1).")
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)
                                .frame(width: 24, alignment: .leading)

                            Text(item)
                                .font(.body.monospaced())
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }
            }
        } label: {
            Text(title)
                .font(.headline.monospaced())
        }
    }

    private var setupReceiptCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Exact setup-receipt family rendered as read-only evidence surfaces.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                ForEach(setupReceipts) { receipt in
                    VStack(alignment: .leading, spacing: 4) {
                        Text(receipt.name)
                            .font(.caption.monospaced())

                        Text(receipt.detail)
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }
                }
            }
        } label: {
            Text("Setup Receipts")
                .font(.headline)
        }
    }

    private func archivedSessionRecallRow(sessionID: String, summary: String, detail: String) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(sessionID)
                .font(.caption.monospaced())

            Text(summary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text(detail)
                .font(.subheadline)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(.secondarySystemGroupedBackground))
        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
    }

    private func operationalQueueEntryRow(_ entry: OperationalQueueEntry) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .firstTextBaseline, spacing: 12) {
                Text(entry.name)
                    .font(.caption.monospaced())

                Spacer()

                Text(entry.posture)
                    .font(.caption.monospaced())
                    .foregroundStyle(.secondary)
            }

            Text(entry.summary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text(entry.detail)
                .font(.subheadline)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            operationalAffordanceCopy
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(.secondarySystemGroupedBackground))
        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
    }

    private func needsAttentionEntryRow(_ entry: OperationalQueueEntry) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .firstTextBaseline, spacing: 12) {
                Text(entry.name)
                    .font(.caption.monospaced())

                Spacer()

                Text(entry.posture)
                    .font(.caption.monospaced())
                    .foregroundStyle(.secondary)
            }

            Text(entry.summary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text(entry.detail)
                .font(.subheadline)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            needsAttentionAffordanceCopy
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(.secondarySystemGroupedBackground))
        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
    }

    private func typedTurnPendingRequestCard(_ request: TypedTurnRequestState) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 10) {
                Text("Awaiting authoritative response")
                    .font(.headline)

                ForEach(
                    [
                        EntryMetadataRow(label: "request_id", value: request.id),
                        EntryMetadataRow(label: "content_type", value: "text/plain"),
                        EntryMetadataRow(label: "modality", value: "CanonicalTurnModality::Text"),
                        EntryMetadataRow(label: "payload", value: "RawTurnPayload::Text"),
                        EntryMetadataRow(
                            label: "session_resolve_mode",
                            value: "SessionResolveMode::ResumeExisting"
                        ),
                        EntryMetadataRow(
                            label: "requested_trigger",
                            value: "RuntimeEntryTrigger::Explicit"
                        ),
                        EntryMetadataRow(label: "draft_bytes", value: "\(request.byteCount)"),
                    ]
                ) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.label)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 170, alignment: .leading)

                        Text(row.value)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                Text(request.boundedPreview)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("This bounded request preview is session-bound, non-authoritative, and stays separate from transcript history until cloud-visible acceptance or response exists.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Typed Turn Request")
                .font(.headline)
        }
    }

    private var explicitVoiceTranscriptPreviewCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 10) {
                Text("Bounded transcript preview")
                    .font(.headline)

                Text(explicitVoiceController.transcriptPreview)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("Transcript preview remains bounded, session-bound, and non-authoritative until cloud-visible acceptance or response exists.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Transcript Preview")
                .font(.headline)
        }
    }

    private func explicitVoicePendingRequestCard(_ request: ExplicitVoiceTurnRequestState) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 10) {
                Text("Awaiting authoritative response")
                    .font(.headline)

                ForEach(
                    [
                        EntryMetadataRow(label: "request_id", value: request.id),
                        EntryMetadataRow(label: "ingress_request", value: "AppVoiceIngressRequest"),
                        EntryMetadataRow(label: "live_input", value: "OsVoiceLiveTurnInput"),
                        EntryMetadataRow(label: "runtime_envelope", value: "RuntimeExecutionEnvelope"),
                        EntryMetadataRow(label: "voice_context", value: "OsVoiceTurnContext"),
                        EntryMetadataRow(label: "trigger", value: "OsVoiceTrigger::Explicit"),
                        EntryMetadataRow(label: "voice_id_request", value: "Ph1VoiceIdRequest"),
                        EntryMetadataRow(
                            label: "embedding_profile",
                            value: "VoiceIdentityEmbeddingGateProfiles::mvp_v1_phone_first()"
                        ),
                        EntryMetadataRow(label: "platform_channel", value: "ios_explicit"),
                        EntryMetadataRow(label: "voice_context_fn", value: "voice_context_ios_explicit()"),
                        EntryMetadataRow(label: "transcript_bytes", value: "\(request.byteCount)"),
                    ]
                ) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.label)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 170, alignment: .leading)

                        Text(row.value)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                Text(request.boundedPreview)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("This bounded explicit voice request preview remains session-bound, `EXPLICIT_ONLY`, and non-authoritative until cloud-visible acceptance or response exists.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Explicit Voice Turn Request")
                .font(.headline)
        }
    }

    private var operationalAffordanceCopy: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 8) {
                Button("Inspect status") {}
                    .buttonStyle(.bordered)
                    .disabled(true)

                Button("Reread authoritative state") {}
                    .buttonStyle(.bordered)
                    .disabled(true)
            }

            HStack(spacing: 8) {
                Button("Continue canonical flow") {}
                    .buttonStyle(.bordered)
                    .disabled(true)

                Button("Open linked operational detail") {}
                    .buttonStyle(.bordered)
                    .disabled(true)
            }
        }
    }

    private var needsAttentionAffordanceCopy: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 8) {
                Button("Open item") {}
                    .buttonStyle(.bordered)
                    .disabled(true)

                Button("Acknowledge") {}
                    .buttonStyle(.bordered)
                    .disabled(true)

                Button("Retry through canonical path") {}
                    .buttonStyle(.bordered)
                    .disabled(true)
            }

            HStack(spacing: 8) {
                Button("Inspect reason") {}
                    .buttonStyle(.bordered)
                    .disabled(true)

                Button("Complete the required human action") {}
                    .buttonStyle(.bordered)
                    .disabled(true)
            }
        }
    }

    private func boundedSurfaceCard(title: String, detail: String) -> some View {
        GroupBox {
            Text(detail)
                .frame(maxWidth: .infinity, alignment: .leading)
        } label: {
            Text(title)
                .font(.headline)
        }
    }

    private func posturePill(_ text: String) -> some View {
        Text(text)
            .font(.caption.weight(.semibold))
            .padding(.horizontal, 10)
            .padding(.vertical, 6)
            .background(Color.accentColor.opacity(0.12))
            .clipShape(Capsule())
    }

    private func permissionStateRow(label: String, state: VoicePermissionState) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(alignment: .firstTextBaseline, spacing: 12) {
                Text(label)
                    .font(.caption.monospaced())
                    .foregroundStyle(.secondary)

                Spacer()

                Text(state.rawValue)
                    .font(.caption.monospaced())
                    .foregroundStyle(.secondary)
            }

            Text(state.detail)
                .font(.subheadline)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(.secondarySystemGroupedBackground))
        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
    }

    private func interruptResponsePendingRequestCard(
        _ request: InterruptContinuityResponseRequestState
    ) -> some View {
        operationalQueueEntryRow(request.pendingOperationalEntry)
    }

    private var trimmedTypedTurnDraft: String {
        typedTurnDraft.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private var displayedPendingOperationalEntries: [OperationalQueueEntry] {
        var entries = pendingOperationalEntries

        if let request = explicitVoiceController.pendingRequest {
            entries.insert(
                OperationalQueueEntry(
                    name: request.id,
                    posture: "pending",
                    summary: "Bounded explicit voice-turn request mirrors `AppVoiceIngressRequest`, `OsVoiceLiveTurnInput`, `RuntimeExecutionEnvelope`, `OsVoiceTurnContext`, `OsVoiceTrigger::Explicit`, `Ph1VoiceIdRequest`, `VoiceIdentityEmbeddingGateProfiles::mvp_v1_phone_first()`, `ios_explicit`, and `voice_context_ios_explicit()`. Awaiting authoritative response.",
                    detail: "Transcript preview: \(request.boundedPreview). This request stays non-authoritative until cloud-visible acceptance or response exists and does not append an authoritative transcript turn locally."
                ),
                at: 0
            )
        }

        if let request = typedTurnPendingRequest {
            entries.insert(
                OperationalQueueEntry(
                    name: request.id,
                    posture: "pending",
                    summary: "Bounded typed-turn request mirrors `RuntimeCanonicalIngressRequest::turn(...)` with `CanonicalTurnModality::Text`, `RawTurnPayload::Text`, `SessionResolveMode::ResumeExisting`, `text/plain`, and explicit requested-trigger posture. Awaiting authoritative response.",
                    detail: "Preview: \(request.boundedPreview). This request stays non-authoritative until cloud-visible acceptance or response exists and does not append an authoritative transcript turn locally."
                ),
                at: 0
            )
        }

        return entries
    }

    private var displayedFailedOperationalEntries: [OperationalQueueEntry] {
        var entries = failedOperationalEntries

        if let interruptResponseFailedRequest {
            entries.insert(interruptResponseFailedRequest, at: 0)
        }

        if let explicitVoiceFailedRequest = explicitVoiceController.failedRequest {
            entries.insert(explicitVoiceFailedRequest, at: 0)
        }

        if let typedTurnFailedRequest {
            entries.insert(typedTurnFailedRequest, at: 0)
        }

        return entries
    }

    private func clearInterruptResponseState() {
        interruptResponsePendingRequest = nil
        interruptResponseFailedRequest = nil
    }

    private func submitInterruptClarifyResponse(
        _ answerFormat: String,
        context: SessionActiveVisibleContext
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
        context: SessionActiveVisibleContext
    ) {
        submitInterruptResponse(
            kind: .returnCheckResponse,
            responseLabel: response.rawValue,
            canonicalValue: response.confirmAnswerValue,
            context: context
        )
    }

    private func submitInterruptResponse(
        kind: InterruptContinuityResponseKind,
        responseLabel: String,
        canonicalValue: String,
        context: SessionActiveVisibleContext
    ) {
        guard interruptResponsePendingRequest == nil else {
            interruptResponseFailedRequest = OperationalQueueEntry(
                name: "failed_interrupt_continuity_awaiting_authoritative_response",
                posture: "failed",
                summary: "A later interruption continuity response could not be produced while the current bounded interruption continuity response is already awaiting authoritative response.",
                detail: "Latest failed interruption continuity response stays visible below in Failed until canonical follow-up occurs."
            )
            return
        }

        interruptResponseRequestSequence += 1
        interruptResponsePendingRequest = InterruptContinuityResponseRequestState(
            id: String(format: "interrupt_continuity_response_%03d", interruptResponseRequestSequence),
            kind: kind,
            responseLabel: responseLabel,
            canonicalValue: canonicalValue,
            sessionID: context.sessionID,
            turnID: context.turnID
        )
        interruptResponseFailedRequest = nil
    }

    private func submitTypedTurn() {
        let trimmedDraft = trimmedTypedTurnDraft
        guard !trimmedDraft.isEmpty else {
            return
        }

        if trimmedDraft.utf8.count > maxTypedTurnBytes {
            typedTurnFailedRequest = OperationalQueueEntry(
                name: "failed_typed_turn_text_plain_validation",
                posture: "failed",
                summary: "Canonical text-turn validation held this request because the bounded `text/plain` payload exceeded 16384 UTF-8 bytes before any authoritative acceptance occurred.",
                detail: "Failed visibility only; shorten the draft and retry through the canonical typed-turn path. No local assistant output or authoritative transcript mutation was produced."
            )
            return
        }

        guard typedTurnPendingRequest == nil else {
            typedTurnFailedRequest = OperationalQueueEntry(
                name: "failed_typed_turn_awaiting_authoritative_response",
                posture: "failed",
                summary: "A later typed-turn request could not be produced while the current explicit text turn is already awaiting authoritative response.",
                detail: "The shell keeps bounded failed posture only; it does not queue a second request locally, repair transport, fabricate local assistant output, or complete pending work."
            )
            return
        }

        typedTurnRequestSequence += 1
        typedTurnPendingRequest = TypedTurnRequestState(
            id: String(format: "typed_turn_request_%03d", typedTurnRequestSequence),
            text: trimmedDraft,
            byteCount: trimmedDraft.utf8.count
        )
        typedTurnDraft = ""
    }
}

#Preview {
    SessionShellView(router: ExplicitEntryRouter())
}
