import AppKit
import Foundation
import AVFoundation
import Speech
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

private func boundedOnboardingContinueFieldInput(_ rawValue: String) -> String? {
    let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty,
          trimmed.count <= 256,
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

private func collectedInterruptClarifyRequiresConfirmation(in queryItems: [URLQueryItem]) -> Bool? {
    let values = queryItems.filter { $0.name == "interrupt_clarify_requires_confirmation" }
    guard values.count == 1 else {
        return nil
    }

    return canonicalBoolean(values[0].value)
}

private func collectedInterruptClarifySensitivityLevel(in queryItems: [URLQueryItem]) -> String? {
    let values = queryItems.filter { $0.name == "interrupt_clarify_sensitivity_level" }
    guard values.count == 1,
          let value = values[0].value,
          let canonicalValue = CanonicalInterruptClarifySensitivityLevel.parse(value)?.rawValue else {
        return nil
    }

    return canonicalValue
}

private func collectedInterruptSubjectRelationConfidence(in queryItems: [URLQueryItem]) -> String? {
    let values = queryItems.filter { $0.name == "interrupt_subject_relation_confidence" }
    guard values.count == 1,
          let value = values[0].value else {
        return nil
    }

    let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty,
          !trimmed.contains("\n"),
          !trimmed.contains("\r"),
          let confidence = Double(trimmed),
          confidence.isFinite,
          confidence >= 0.0,
          confidence <= 1.0 else {
        return nil
    }

    return trimmed
}

private func collectedInterruptSubjectRefValue(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty,
          !trimmed.contains("\n"),
          !trimmed.contains("\r"),
          trimmed.count <= 256 else {
        return nil
    }

    return trimmed
}

private func collectedActiveSubjectRef(in queryItems: [URLQueryItem]) -> String? {
    let values = queryItems.filter { $0.name == "active_subject_ref" }
    guard values.count == 1 else {
        return nil
    }

    return collectedInterruptSubjectRefValue(values[0].value)
}

private func collectedInterruptedSubjectRef(in queryItems: [URLQueryItem]) -> String? {
    let values = queryItems.filter { $0.name == "interrupted_subject_ref" }
    guard values.count == 1 else {
        return nil
    }

    return collectedInterruptSubjectRefValue(values[0].value)
}

private func collectedReturnCheckExpiresAt(in queryItems: [URLQueryItem]) -> String? {
    let values = queryItems.filter { $0.name == "return_check_expires_at" }
    guard values.count == 1,
          let value = values[0].value else {
        return nil
    }

    let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty,
          !trimmed.contains("\n"),
          !trimmed.contains("\r") else {
        return nil
    }

    return trimmed
}

private func collectedResumeBufferExpiresAt(in queryItems: [URLQueryItem]) -> String? {
    let values = queryItems.filter { $0.name == "resume_buffer_expires_at" }
    guard values.count == 1,
          let value = values[0].value else {
        return nil
    }

    let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty,
          !trimmed.contains("\n"),
          !trimmed.contains("\r") else {
        return nil
    }

    return trimmed
}

private func collectedResumeBufferLive(in queryItems: [URLQueryItem]) -> Bool? {
    let values = queryItems.filter { $0.name == "resume_buffer_live" }
    guard values.count == 1 else {
        return nil
    }

    return canonicalBoolean(values[0].value)
}

private func collectedResumeBufferTopicHint(in queryItems: [URLQueryItem]) -> String? {
    let values = queryItems.filter { $0.name == "resume_buffer_topic_hint" }
    guard values.count == 1,
          let value = values[0].value else {
        return nil
    }

    let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty,
          !trimmed.contains("\n"),
          !trimmed.contains("\r") else {
        return nil
    }

    return trimmed
}

private func canonicalRemainingPlatformReceiptKind(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty else {
        return nil
    }

    switch trimmed {
    case "install_launch_handshake",
         "mic_permission_granted",
         "desktop_wakeword_configured",
         "desktop_pairing_bound":
        return trimmed
    default:
        return nil
    }
}

private func collectedRemainingPlatformReceiptKinds(in queryItems: [URLQueryItem]) -> [String] {
    var receiptKinds: [String] = []

    for queryItem in queryItems where queryItem.name == "remaining_platform_receipt_kind" {
        guard let canonicalValue = canonicalRemainingPlatformReceiptKind(queryItem.value),
              !receiptKinds.contains(canonicalValue),
              receiptKinds.count < 4 else {
            return []
        }

        receiptKinds.append(canonicalValue)
    }

    return receiptKinds
}

private func collectedWakeRuntimeEventCanonicalBoolean(
    in queryItems: [URLQueryItem],
    name: String
) -> Bool? {
    let values = queryItems.filter { $0.name == name }
    guard values.count == 1 else {
        return nil
    }

    return canonicalBoolean(values[0].value)
}

private func collectedWakeRuntimeEventSingleLineValue(
    in queryItems: [URLQueryItem],
    name: String
) -> String? {
    let values = queryItems.filter { $0.name == name }
    guard values.count == 1,
          let value = values[0].value else {
        return nil
    }

    let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty,
          !trimmed.contains("\n"),
          !trimmed.contains("\r") else {
        return nil
    }

    return trimmed
}

private func collectedAuthorityStateSingleLineValue(
    in queryItems: [URLQueryItem],
    name: String
) -> String? {
    let values = queryItems.filter { $0.name == name }
    guard values.count == 1,
          let value = values[0].value else {
        return nil
    }

    let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty,
          !trimmed.contains("\n"),
          !trimmed.contains("\r") else {
        return nil
    }

    return trimmed
}

private func collectedSingleLineInterruptValue(
    in queryItems: [URLQueryItem],
    name: String
) -> String? {
    let values = queryItems.filter { $0.name == name }
    guard values.count == 1,
          let value = values[0].value else {
        return nil
    }

    let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty,
          !trimmed.contains("\n"),
          !trimmed.contains("\r") else {
        return nil
    }

    return trimmed
}

private func collectedMultilineInterruptValue(
    in queryItems: [URLQueryItem],
    name: String
) -> String? {
    let values = queryItems.filter { $0.name == name }
    guard values.count == 1,
          let value = values[0].value else {
        return nil
    }

    let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty else {
        return nil
    }

    return trimmed
}

private func collectedResumeBufferAnswerID(in queryItems: [URLQueryItem]) -> String? {
    collectedSingleLineInterruptValue(in: queryItems, name: "resume_buffer_answer_id")
}

private func collectedResumeBufferSpokenPrefix(in queryItems: [URLQueryItem]) -> String? {
    collectedMultilineInterruptValue(in: queryItems, name: "resume_buffer_spoken_prefix")
}

private func collectedResumeBufferUnsaidRemainder(in queryItems: [URLQueryItem]) -> String? {
    collectedMultilineInterruptValue(in: queryItems, name: "resume_buffer_unsaid_remainder")
}

private func collectedTtsResumeSnapshotAnswerID(in queryItems: [URLQueryItem]) -> String? {
    collectedSingleLineInterruptValue(in: queryItems, name: "tts_resume_snapshot_answer_id")
}

private func collectedTtsResumeSnapshotSpokenCursorByte(in queryItems: [URLQueryItem]) -> String? {
    collectedSingleLineInterruptValue(
        in: queryItems,
        name: "tts_resume_snapshot_spoken_cursor_byte"
    )
}

private func collectedTtsResumeSnapshotResponseText(in queryItems: [URLQueryItem]) -> String? {
    collectedMultilineInterruptValue(in: queryItems, name: "tts_resume_snapshot_response_text")
}

private func collectedTtsResumeSnapshotTopicHint(in queryItems: [URLQueryItem]) -> String? {
    let values = queryItems.filter { $0.name == "tts_resume_snapshot_topic_hint" }
    guard values.count == 1,
          let value = values[0].value else {
        return nil
    }

    let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty,
          !trimmed.contains("\n"),
          !trimmed.contains("\r"),
          trimmed.count <= 64 else {
        return nil
    }

    return trimmed
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

struct ExplicitVoiceTurnRequestState: Identifiable {
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

private struct DesktopAuthoritativeReplyRenderState: Equatable {
    let title: String
    let summary: String
    let authoritativeResponseText: String?
}

private struct DesktopAuthoritativeReplyProvenanceRenderState: Equatable {
    struct Source: Identifiable, Equatable {
        let title: String
        let url: String

        var id: String {
            "\(title)|\(url)"
        }
    }

    let title: String
    let summary: String
    let authoritativeResponseProvenance: AuthoritativeResponseProvenance?
    let sources: [Source]
    let retrievedAtLabel: String?
    let cacheStatusLabel: String?
}

private struct DesktopAuthoritativeReplyPlaybackState: Equatable {
    enum Phase: String, Equatable {
        case idle = "idle"
        case speaking = "speaking"
        case failed = "failed"
    }

    let phase: Phase
    let title: String
    let summary: String
    let detail: String

    static let idle = DesktopAuthoritativeReplyPlaybackState(
        phase: .idle,
        title: "Authoritative reply playback idle",
        summary: "Playback remains available only for cloud-authored reply text that is already visible in the bounded reply surface.",
        detail: "Bounded native macOS speech playback only. This shell remains explicitly non-authoritative, does not mutate transcript preview, and does not claim wake parity, native wake-listener integration, or autonomous-unlock capability."
    )

    static func speaking(authoritativeResponseText: String) -> DesktopAuthoritativeReplyPlaybackState {
        let boundedPreview = boundedSummary(authoritativeResponseText) ?? "Cloud-authored reply playback is active."
        return DesktopAuthoritativeReplyPlaybackState(
            phase: .speaking,
            title: "Authoritative reply playback active",
            summary: boundedPreview,
            detail: "This shell is speaking only the already-rendered cloud-authored authoritative reply text. No local reply synthesis, transcript mutation, wake behavior, or autonomous-unlock capability is introduced here."
        )
    }

    static func stopped() -> DesktopAuthoritativeReplyPlaybackState {
        DesktopAuthoritativeReplyPlaybackState(
            phase: .idle,
            title: "Authoritative reply playback stopped",
            summary: "Playback has been stopped and the bounded reply surface remains read-only.",
            detail: "Stopping playback does not alter authoritative reply text, transcript preview, or conversation history."
        )
    }

    static func completed() -> DesktopAuthoritativeReplyPlaybackState {
        DesktopAuthoritativeReplyPlaybackState(
            phase: .idle,
            title: "Authoritative reply playback finished",
            summary: "Playback finished for the current cloud-authored authoritative reply.",
            detail: "Completion visibility only. This shell remains non-authoritative and does not mutate transcript preview, reply-surface posture, or wake behavior."
        )
    }

    static func failed(summary: String, detail: String) -> DesktopAuthoritativeReplyPlaybackState {
        DesktopAuthoritativeReplyPlaybackState(
            phase: .failed,
            title: "Authoritative reply playback failed",
            summary: summary,
            detail: detail
        )
    }
}

@MainActor
private final class DesktopAuthoritativeReplyPlaybackController: NSObject, ObservableObject, NSSpeechSynthesizerDelegate {
    @Published private(set) var playbackState: DesktopAuthoritativeReplyPlaybackState = .idle

    private let speechSynthesizer = NSSpeechSynthesizer()
    private var stopWasRequested = false

    override init() {
        super.init()
        speechSynthesizer.delegate = self
    }

    @discardableResult
    func play(authoritativeResponseText: String?) -> DesktopAuthoritativeReplyPlaybackState {
        guard let authoritativeResponseText else {
            let failedState = DesktopAuthoritativeReplyPlaybackState.failed(
                summary: "Playback is unavailable because no cloud-authored reply text is present.",
                detail: "This shell fails closed when canonical runtime reply text is missing and does not fabricate local speech content."
            )
            playbackState = failedState
            return failedState
        }

        let trimmed = authoritativeResponseText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            let failedState = DesktopAuthoritativeReplyPlaybackState.failed(
                summary: "Playback is unavailable because the reply text is empty.",
                detail: "This shell fails closed when canonical runtime reply text is empty and does not synthesize substitute content."
            )
            playbackState = failedState
            return failedState
        }

        if speechSynthesizer.isSpeaking {
            speechSynthesizer.stopSpeaking()
        }

        stopWasRequested = false

        guard speechSynthesizer.startSpeaking(trimmed) else {
            let failedState = DesktopAuthoritativeReplyPlaybackState.failed(
                summary: "Native macOS reply playback could not start.",
                detail: "The shell remains read-only and non-authoritative while bounded native speech playback initialization is unavailable."
            )
            playbackState = failedState
            return failedState
        }

        let speakingState = DesktopAuthoritativeReplyPlaybackState.speaking(authoritativeResponseText: trimmed)
        playbackState = speakingState
        return speakingState
    }

    @discardableResult
    func stop() -> DesktopAuthoritativeReplyPlaybackState {
        stopWasRequested = true
        if speechSynthesizer.isSpeaking {
            speechSynthesizer.stopSpeaking()
        }
        let stoppedState = DesktopAuthoritativeReplyPlaybackState.stopped()
        playbackState = stoppedState
        return stoppedState
    }

    func reset() {
        stopWasRequested = false
        if speechSynthesizer.isSpeaking {
            speechSynthesizer.stopSpeaking()
        }
        playbackState = .idle
    }

    nonisolated func speechSynthesizer(_ sender: NSSpeechSynthesizer, didFinishSpeaking finishedSpeaking: Bool) {
        Task { @MainActor in
            if stopWasRequested {
                stopWasRequested = false
                playbackState = .stopped()
            } else if finishedSpeaking {
                playbackState = .completed()
            } else {
                playbackState = .failed(
                    summary: "Native macOS reply playback ended before completion.",
                    detail: "The shell remains explicitly non-authoritative and does not retry or fabricate substitute playback output."
                )
            }
        }
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
    @Published private(set) var failedRequest: InterruptContinuityResponseFailureState?

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
                id: "failed_explicit_voice_turn_awaiting_authoritative_response",
                title: "Failed explicit voice request",
                summary: "A later explicit voice-turn request could not be produced while the current bounded explicit voice request is already awaiting authoritative follow-up.",
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
                        id: "failed_explicit_voice_microphone_permission",
                        title: "Failed explicit voice request",
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
                                id: "failed_explicit_voice_speech_permission",
                                title: "Failed explicit voice request",
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
                id: "failed_explicit_voice_not_listening",
                title: "Failed explicit voice request",
                summary: "The bounded explicit voice surface was not actively listening when request preparation was attempted.",
                detail: "Explicit voice-turn production remains foreground-only and user-visible. Start a new explicit voice turn before preparing another bounded request."
            )
            return
        }

        endCaptureInput()

        let trimmedTranscript = transcriptPreview.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedTranscript.isEmpty else {
            recordFailure(
                id: "failed_explicit_voice_empty_transcript",
                title: "Failed explicit voice request",
                summary: "No bounded transcript preview was available when this explicit voice turn stopped, so no voice request was produced.",
                detail: "Failure visibility only; speak again and retry through the canonical explicit voice path. No local assistant output or authoritative transcript mutation was produced."
            )
            return
        }

        if trimmedTranscript.utf8.count > maxVoiceTurnBytes {
            recordFailure(
                id: "failed_explicit_voice_transcript_validation",
                title: "Failed explicit voice request",
                summary: "The bounded explicit voice transcript exceeded 16384 UTF-8 bytes before any authoritative acceptance occurred.",
                detail: "Failure visibility only; retry a shorter utterance through the canonical explicit voice path. No authoritative transcript turn was appended locally."
            )
            return
        }

        requestSequence += 1
        transcriptPreview = trimmedTranscript
        pendingRequest = ExplicitVoiceTurnRequestState(
            id: String(format: "desktop_voice_turn_request_%03d", requestSequence),
            transcript: trimmedTranscript,
            byteCount: trimmedTranscript.utf8.count
        )
    }

    func haltCaptureSession() {
        teardownRecognitionSession()
    }

    func clearPendingPreparedVoiceTurn() {
        pendingRequest = nil
    }

    private func beginCaptureSession() {
        failedRequest = nil
        teardownRecognitionSession()
        refreshPermissionState()

        guard let speechRecognizer else {
            speechRecognitionPermission = .unavailable
            recordFailure(
                id: "failed_explicit_voice_recognizer_unavailable",
                title: "Failed explicit voice request",
                summary: "No speech recognizer is available for bounded explicit voice-turn request preparation on this device posture.",
                detail: "Unavailable visibility only; the shell remains `EXPLICIT_ONLY`, session-bound, and cloud-authoritative while explicit voice capture stays blocked."
            )
            return
        }

        guard speechRecognizer.isAvailable else {
            speechRecognitionPermission = .unavailable
            recordFailure(
                id: "failed_explicit_voice_recognizer_busy",
                title: "Failed explicit voice request",
                summary: "The speech recognizer is not currently available for a bounded explicit voice turn.",
                detail: "Availability visibility only; retry from the same foreground surface later. No local queue repair authority or hidden retry loop is introduced here."
            )
            return
        }

        do {
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
                            id: "failed_explicit_voice_capture_session",
                            title: "Failed explicit voice request",
                            summary: "The bounded explicit voice capture session ended before a request could be prepared.",
                            detail: "Speech capture failed with `\(error.localizedDescription)`. Failure visibility only; no local transcript authority, no hidden retry loop, and no authoritative assistant output were produced."
                        )
                    }
                }
            }
        } catch {
            teardownRecognitionSession()
            recordFailure(
                id: "failed_explicit_voice_capture_start",
                title: "Failed explicit voice request",
                summary: "The bounded explicit voice capture session could not start from this foreground surface.",
                detail: "Capture start failed with `\(error.localizedDescription)`. Failure visibility only; no background capture, no wake behavior, and no autonomous-unlock capability were introduced."
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
    }

    private func teardownRecognitionSession() {
        endCaptureInput()
        recognitionTask?.cancel()
        recognitionTask = nil
        recognitionRequest = nil
    }

    private func recordFailure(id: String, title: String, summary: String, detail: String) {
        failedRequest = InterruptContinuityResponseFailureState(
            id: id,
            title: title,
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
            AVCaptureDevice.requestAccess(for: .audio) { granted in
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
        switch AVCaptureDevice.authorizationStatus(for: .audio) {
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
    let interruptClarifyRequiresConfirmation: Bool?
    let interruptClarifySensitivityLevel: String?
    let interruptSubjectRelationConfidence: String?
    let activeSubjectRef: String?
    let interruptedSubjectRef: String?
    let returnCheckExpiresAt: String?
    let resumeBufferLive: Bool?
    let resumeBufferExpiresAt: String?
    let resumeBufferAnswerID: String?
    let resumeBufferSpokenPrefix: String?
    let resumeBufferUnsaidRemainder: String?
    let resumeBufferTopicHint: String?
    let ttsResumeSnapshotAnswerID: String?
    let ttsResumeSnapshotSpokenCursorByte: String?
    let ttsResumeSnapshotResponseText: String?
    let ttsResumeSnapshotTopicHint: String?
    let authorityStatePolicyContextRef: String?
    let authorityStateSimulationCertificationState: String?
    let authorityStateOnboardingReadinessState: String?
    let authorityStatePolicyDecision: String?
    let authorityStateIdentityScopeRequired: Bool?
    let authorityStateIdentityScopeSatisfied: Bool?
    let authorityStateMemoryScopeAllowed: Bool?
    let authorityStateReasonCode: String?
    let wakeRuntimeEventCreatedAt: String?
    let wakeRuntimeEventAccepted: Bool?
    let wakeRuntimeEventReasonCode: String?
    let wakeRuntimeEventWakeProfileID: String?
    let wakeRuntimeEventTtsActiveAtTrigger: Bool?
    let wakeRuntimeEventMediaPlaybackActiveAtTrigger: Bool?
    let wakeRuntimeEventSuppressionReasonCode: String?
    let wakeRuntimeEventLightScoreBP: String?
    let wakeRuntimeEventStrongScoreBP: String?
    let wakeRuntimeEventThresholdUsedBP: String?
    let wakeRuntimeEventModelVersion: String?
    let wakeRuntimeEventWindowStartNS: String?
    let wakeRuntimeEventWindowEndNS: String?
    let interruptAcceptedAnswerFormats: [String]
    let remainingPlatformReceiptKinds: [String]

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

        let authorityStateIdentityScopeRequiredValues = queryItems.filter {
            $0.name == "authority_state_identity_scope_required"
        }
        let authorityStateIdentityScopeRequired = authorityStateIdentityScopeRequiredValues.count == 1
            ? canonicalBoolean(authorityStateIdentityScopeRequiredValues[0].value)
            : nil
        let authorityStateIdentityScopeSatisfiedValues = queryItems.filter {
            $0.name == "authority_state_identity_scope_satisfied"
        }
        let authorityStateIdentityScopeSatisfied =
            authorityStateIdentityScopeSatisfiedValues.count == 1
            ? canonicalBoolean(authorityStateIdentityScopeSatisfiedValues[0].value)
            : nil
        let authorityStateMemoryScopeAllowedValues = queryItems.filter {
            $0.name == "authority_state_memory_scope_allowed"
        }
        let authorityStateMemoryScopeAllowed = authorityStateMemoryScopeAllowedValues.count == 1
            ? canonicalBoolean(authorityStateMemoryScopeAllowedValues[0].value)
            : nil

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
        self.interruptClarifyRequiresConfirmation = collectedInterruptClarifyRequiresConfirmation(
            in: queryItems
        )
        self.interruptClarifySensitivityLevel = collectedInterruptClarifySensitivityLevel(
            in: queryItems
        )
        self.interruptSubjectRelationConfidence = collectedInterruptSubjectRelationConfidence(
            in: queryItems
        )
        self.activeSubjectRef = collectedActiveSubjectRef(in: queryItems)
        self.interruptedSubjectRef = collectedInterruptedSubjectRef(in: queryItems)
        self.returnCheckExpiresAt = collectedReturnCheckExpiresAt(in: queryItems)
        self.resumeBufferLive = collectedResumeBufferLive(in: queryItems)
        self.resumeBufferExpiresAt = collectedResumeBufferExpiresAt(in: queryItems)
        self.resumeBufferAnswerID = collectedResumeBufferAnswerID(in: queryItems)
        self.resumeBufferSpokenPrefix = collectedResumeBufferSpokenPrefix(in: queryItems)
        self.resumeBufferUnsaidRemainder = collectedResumeBufferUnsaidRemainder(in: queryItems)
        self.resumeBufferTopicHint = collectedResumeBufferTopicHint(in: queryItems)
        self.ttsResumeSnapshotAnswerID = collectedTtsResumeSnapshotAnswerID(in: queryItems)
        self.ttsResumeSnapshotSpokenCursorByte = collectedTtsResumeSnapshotSpokenCursorByte(
            in: queryItems
        )
        self.ttsResumeSnapshotResponseText = collectedTtsResumeSnapshotResponseText(
            in: queryItems
        )
        self.ttsResumeSnapshotTopicHint = collectedTtsResumeSnapshotTopicHint(in: queryItems)
        self.authorityStatePolicyContextRef = collectedAuthorityStateSingleLineValue(
            in: queryItems,
            name: "authority_state_policy_context_ref"
        )
        self.authorityStateSimulationCertificationState = collectedAuthorityStateSingleLineValue(
            in: queryItems,
            name: "authority_state_simulation_certification_state"
        )
        self.authorityStateOnboardingReadinessState = collectedAuthorityStateSingleLineValue(
            in: queryItems,
            name: "authority_state_onboarding_readiness_state"
        )
        self.authorityStatePolicyDecision = collectedAuthorityStateSingleLineValue(
            in: queryItems,
            name: "authority_state_policy_decision"
        )
        self.authorityStateIdentityScopeRequired = authorityStateIdentityScopeRequired
        self.authorityStateIdentityScopeSatisfied = authorityStateIdentityScopeSatisfied
        self.authorityStateMemoryScopeAllowed = authorityStateMemoryScopeAllowed
        self.authorityStateReasonCode = collectedAuthorityStateSingleLineValue(
            in: queryItems,
            name: "authority_state_reason_code"
        )
        self.wakeRuntimeEventCreatedAt = collectedWakeRuntimeEventSingleLineValue(
            in: queryItems,
            name: "wake_runtime_event_created_at"
        )
        self.wakeRuntimeEventAccepted = collectedWakeRuntimeEventCanonicalBoolean(
            in: queryItems,
            name: "wake_runtime_event_accepted"
        )
        self.wakeRuntimeEventReasonCode = collectedWakeRuntimeEventSingleLineValue(
            in: queryItems,
            name: "wake_runtime_event_reason_code"
        )
        self.wakeRuntimeEventWakeProfileID = collectedWakeRuntimeEventSingleLineValue(
            in: queryItems,
            name: "wake_runtime_event_wake_profile_id"
        )
        self.wakeRuntimeEventTtsActiveAtTrigger = collectedWakeRuntimeEventCanonicalBoolean(
            in: queryItems,
            name: "wake_runtime_event_tts_active_at_trigger"
        )
        self.wakeRuntimeEventMediaPlaybackActiveAtTrigger =
            collectedWakeRuntimeEventCanonicalBoolean(
                in: queryItems,
                name: "wake_runtime_event_media_playback_active_at_trigger"
            )
        self.wakeRuntimeEventSuppressionReasonCode =
            collectedWakeRuntimeEventSingleLineValue(
                in: queryItems,
                name: "wake_runtime_event_suppression_reason_code"
            )
        self.wakeRuntimeEventLightScoreBP = collectedWakeRuntimeEventSingleLineValue(
            in: queryItems,
            name: "wake_runtime_event_light_score_bp"
        )
        self.wakeRuntimeEventStrongScoreBP = collectedWakeRuntimeEventSingleLineValue(
            in: queryItems,
            name: "wake_runtime_event_strong_score_bp"
        )
        self.wakeRuntimeEventThresholdUsedBP = collectedWakeRuntimeEventSingleLineValue(
            in: queryItems,
            name: "wake_runtime_event_threshold_used_bp"
        )
        self.wakeRuntimeEventModelVersion = collectedWakeRuntimeEventSingleLineValue(
            in: queryItems,
            name: "wake_runtime_event_model_version"
        )
        self.wakeRuntimeEventWindowStartNS = collectedWakeRuntimeEventSingleLineValue(
            in: queryItems,
            name: "wake_runtime_event_window_start_ns"
        )
        self.wakeRuntimeEventWindowEndNS = collectedWakeRuntimeEventSingleLineValue(
            in: queryItems,
            name: "wake_runtime_event_window_end_ns"
        )
        self.interruptAcceptedAnswerFormats = collectedInterruptAcceptedAnswerFormats(in: queryItems)
        self.remainingPlatformReceiptKinds = collectedRemainingPlatformReceiptKinds(in: queryItems)
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

    var hasLawfulInterruptClarifyRequiresConfirmation: Bool {
        interruptClarifyRequiresConfirmation != nil
    }

    var hasLawfulInterruptClarifySensitivityLevel: Bool {
        interruptClarifySensitivityLevel != nil
    }

    var hasLawfulInterruptSubjectRelationConfidence: Bool {
        interruptSubjectRelation != nil && interruptSubjectRelationConfidence != nil
    }

    var hasLawfulInterruptSubjectReferences: Bool {
        interruptSubjectRelation != nil && (activeSubjectRef != nil || interruptedSubjectRef != nil)
    }

    var hasLawfulInterruptReturnCheckExpiry: Bool {
        returnCheckPending == true && returnCheckExpiresAt != nil
    }

    var hasLawfulInterruptResumeBufferLive: Bool {
        resumeBufferLive == true
    }

    var hasLawfulInterruptResumeBufferExpiresAt: Bool {
        resumeBufferLive == true && resumeBufferExpiresAt != nil
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
        ttsResumeSnapshotAnswerID != nil && ttsResumeSnapshotTopicHint != nil
    }

    var hasLawfulAuthorityStateCarrierFamily: Bool {
        authorityStateSimulationCertificationState != nil
            && authorityStateOnboardingReadinessState != nil
            && authorityStatePolicyDecision != nil
            && authorityStateIdentityScopeRequired != nil
            && authorityStateIdentityScopeSatisfied != nil
            && authorityStateMemoryScopeAllowed != nil
    }

    var hasLawfulOnboardingPlatformSetupReceiptCarrierFamily: Bool {
        !remainingPlatformReceiptKinds.isEmpty
    }

    var hasLawfulWakeRuntimeEventEvidenceCarrierFamily: Bool {
        wakeRuntimeEventCreatedAt != nil
            && wakeRuntimeEventAccepted != nil
            && wakeRuntimeEventReasonCode != nil
            && wakeRuntimeEventTtsActiveAtTrigger != nil
            && wakeRuntimeEventMediaPlaybackActiveAtTrigger != nil
    }

    var wakeRuntimeEventEvidenceRows: [(label: String, value: String)] {
        guard hasLawfulWakeRuntimeEventEvidenceCarrierFamily,
              let wakeRuntimeEventCreatedAt,
              let wakeRuntimeEventAccepted,
              let wakeRuntimeEventReasonCode,
              let wakeRuntimeEventTtsActiveAtTrigger,
              let wakeRuntimeEventMediaPlaybackActiveAtTrigger else {
            return []
        }

        return [
            ("wake_runtime_event_created_at", wakeRuntimeEventCreatedAt),
            ("wake_runtime_event_accepted", booleanValue(wakeRuntimeEventAccepted)),
            ("wake_runtime_event_reason_code", wakeRuntimeEventReasonCode),
            ("wake_runtime_event_wake_profile_id", wakeRuntimeEventWakeProfileID ?? "not_provided"),
            (
                "wake_runtime_event_tts_active_at_trigger",
                booleanValue(wakeRuntimeEventTtsActiveAtTrigger)
            ),
            (
                "wake_runtime_event_media_playback_active_at_trigger",
                booleanValue(wakeRuntimeEventMediaPlaybackActiveAtTrigger)
            ),
            (
                "wake_runtime_event_suppression_reason_code",
                wakeRuntimeEventSuppressionReasonCode ?? "not_provided"
            ),
            ("wake_runtime_event_light_score_bp", wakeRuntimeEventLightScoreBP ?? "not_provided"),
            (
                "wake_runtime_event_strong_score_bp",
                wakeRuntimeEventStrongScoreBP ?? "not_provided"
            ),
            (
                "wake_runtime_event_threshold_used_bp",
                wakeRuntimeEventThresholdUsedBP ?? "not_provided"
            ),
            (
                "wake_runtime_event_model_version",
                wakeRuntimeEventModelVersion ?? "not_provided"
            ),
            (
                "wake_runtime_event_window_start_ns",
                wakeRuntimeEventWindowStartNS ?? "not_provided"
            ),
            ("wake_runtime_event_window_end_ns", wakeRuntimeEventWindowEndNS ?? "not_provided"),
        ]
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

enum DesktopOnboardingEntryRouteKind: String, Equatable {
    case inviteOpen = "INVITE_OPEN"
    case appOpen = "APP_OPEN"

    var title: String {
        switch self {
        case .inviteOpen:
            return "Invite-open onboarding entry"
        case .appOpen:
            return "App-open onboarding entry"
        }
    }
}

struct DesktopOnboardingEntryContext: Identifiable, Equatable {
    let id: String
    let routeKind: DesktopOnboardingEntryRouteKind
    let scheme: String
    let host: String
    let path: String
    let tokenID: String
    let tokenSignature: String
    let tenantID: String?
    let tenantHint: String?
    let deepLinkNonce: String
    let appInstanceID: String
    let deviceFingerprint: String

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

        guard
            let tokenID = Self.routeField(firstQueryValue(in: queryItems, name: "token_id")),
            let tokenSignature = Self.routeField(firstQueryValue(in: queryItems, name: "token_signature")),
            let deepLinkNonce = Self.routeField(firstQueryValue(in: queryItems, name: "deep_link_nonce")),
            let appInstanceID = Self.routeField(firstQueryValue(in: queryItems, name: "app_instance_id")),
            let deviceFingerprint = Self.routeField(firstQueryValue(in: queryItems, name: "device_fingerprint"))
        else {
            return nil
        }

        let tenantID = Self.routeField(firstQueryValue(in: queryItems, name: "tenant_id"))
        let tenantHint = Self.routeField(firstQueryValue(in: queryItems, name: "tenant_hint"))

        let lowerPath = path.lowercased()
        let inviteLike = host.contains("invite") || lowerPath.contains("invite") || lowerPath.contains("onboarding")
        let appOpenLike = host.contains("open") || host.contains("entry") || lowerPath.contains("open") || lowerPath.contains("entry") || scheme == "selene"
        guard inviteLike || appOpenLike else {
            return nil
        }

        self.id = url.absoluteString
        self.routeKind = inviteLike ? .inviteOpen : .appOpen
        self.scheme = scheme
        self.host = host
        self.path = path
        self.tokenID = tokenID
        self.tokenSignature = tokenSignature
        self.tenantID = tenantID
        self.tenantHint = tenantHint
        self.deepLinkNonce = deepLinkNonce
        self.appInstanceID = appInstanceID
        self.deviceFingerprint = deviceFingerprint
    }

    var routeRows: [(label: String, value: String)] {
        var rows: [(label: String, value: String)] = [
            ("entry_kind", routeKind.rawValue),
            ("scheme", scheme),
            ("host", host),
            ("path", path),
            ("token_id", boundedHint(tokenID) ?? tokenID),
            ("deep_link_nonce", boundedHint(deepLinkNonce) ?? deepLinkNonce),
            ("app_instance_id", boundedHint(appInstanceID) ?? appInstanceID),
            ("device_fingerprint", boundedHint(deviceFingerprint) ?? deviceFingerprint),
        ]

        if let tenantID {
            rows.append(("tenant_id", boundedHint(tenantID) ?? tenantID))
        } else if let tenantHint {
            rows.append(("tenant_hint", boundedHint(tenantHint) ?? tenantHint))
        }

        return rows
    }

    private static func routeField(_ rawValue: String?) -> String? {
        guard let rawValue else {
            return nil
        }

        let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty, trimmed.count <= 256, !trimmed.contains("\n"), !trimmed.contains("\r") else {
            return nil
        }

        return trimmed
    }
}

struct DesktopOnboardingContinuePromptState: Identifiable, Equatable {
    let onboardingSessionID: String
    let nextStep: String
    let blockingField: String
    let blockingQuestion: String?
    let remainingMissingFields: [String]

    var id: String {
        [
            onboardingSessionID,
            nextStep,
            blockingField,
            blockingQuestion ?? "question_not_provided",
            remainingMissingFields.joined(separator: "|"),
        ].joined(separator: "::")
    }
}

struct DesktopTermsAcceptPromptState: Identifiable, Equatable {
    let onboardingSessionID: String
    let nextStep: String
    let termsVersionID: String

    var id: String {
        [
            onboardingSessionID,
            nextStep,
            termsVersionID,
        ].joined(separator: "::")
    }
}

struct DesktopPrimaryDeviceConfirmPromptState: Identifiable, Equatable {
    let onboardingSessionID: String
    let nextStep: String
    let deviceID: String
    let proofOK: Bool

    var id: String {
        [
            onboardingSessionID,
            nextStep,
            deviceID,
            proofOK ? "proof_ok_true" : "proof_ok_false",
        ].joined(separator: "::")
    }
}

struct DesktopVoiceEnrollPromptState: Identifiable, Equatable {
    let onboardingSessionID: String
    let nextStep: String
    let deviceID: String
    let transcriptPreview: String

    var id: String {
        [
            onboardingSessionID,
            nextStep,
            deviceID,
            String(transcriptPreview.utf8.count),
            String(transcriptPreview.prefix(48)),
        ].joined(separator: "::")
    }
}

struct DesktopWakeEnrollStartDraftPromptState: Identifiable, Equatable {
    let onboardingSessionID: String
    let nextStep: String
    let deviceID: String
    let voiceArtifactSyncReceiptRef: String?

    var id: String {
        [
            onboardingSessionID,
            nextStep,
            deviceID,
            voiceArtifactSyncReceiptRef ?? "voice_receipt_not_provided",
        ].joined(separator: "::")
    }
}

struct DesktopWakeEnrollSampleCommitPromptState: Identifiable, Equatable {
    let onboardingSessionID: String
    let nextStep: String
    let deviceID: String
    let proofOK: Bool
    let voiceArtifactSyncReceiptRef: String?

    var id: String {
        [
            onboardingSessionID,
            nextStep,
            deviceID,
            proofOK ? "proof_ok_true" : "proof_ok_false",
            voiceArtifactSyncReceiptRef ?? "voice_receipt_not_provided",
        ].joined(separator: "::")
    }
}

struct DesktopWakeEnrollCompleteCommitPromptState: Identifiable, Equatable {
    let onboardingSessionID: String
    let nextStep: String
    let deviceID: String
    let voiceArtifactSyncReceiptRef: String?

    var id: String {
        [
            onboardingSessionID,
            nextStep,
            deviceID,
            voiceArtifactSyncReceiptRef ?? "voice_receipt_not_provided",
        ].joined(separator: "::")
    }
}

struct DesktopPlatformSetupReceiptDraft: Identifiable, Equatable {
    let onboardingSessionID: String
    let receiptKind: String
    let proofMaterial: String
    let proofSummary: String

    var id: String {
        "\(onboardingSessionID)::\(receiptKind)"
    }

    var buttonTitle: String {
        switch receiptKind {
        case "install_launch_handshake":
            return "Submit install launch handshake"
        case "mic_permission_granted":
            return "Submit microphone permission receipt"
        default:
            return "Submit desktop platform receipt"
        }
    }
}

struct DesktopSessionShellView: View {
    @State private var latestSessionHeaderContext: DesktopSessionHeaderContext?
    @State private var latestSessionActiveVisibleContext: DesktopSessionActiveVisibleContext?
    @State private var latestSessionSoftClosedVisibleContext: DesktopSessionSoftClosedVisibleContext?
    @State private var latestSessionSuspendedVisibleContext: DesktopSessionSuspendedVisibleContext?
    @State private var desktopOnboardingEntryContext: DesktopOnboardingEntryContext?
    @State private var interruptResponsePendingRequest: InterruptContinuityResponseRequestState?
    @State private var interruptResponseFailedRequest: InterruptContinuityResponseFailureState?
    @State private var interruptResponseRequestSequence: Int = 0
    @StateObject private var explicitVoiceController = ExplicitVoiceCaptureController()
    @StateObject private var desktopCanonicalRuntimeBridge = DesktopCanonicalRuntimeBridge()
    @StateObject private var desktopAuthoritativeReplyPlaybackController = DesktopAuthoritativeReplyPlaybackController()
    @State private var desktopCanonicalRuntimeOutcomeState: DesktopCanonicalRuntimeOutcomeState?
    @State private var desktopInviteOpenRuntimeOutcomeState: DesktopInviteOpenRuntimeOutcomeState?
    @State private var desktopOnboardingContinueRuntimeOutcomeState: DesktopOnboardingContinueRuntimeOutcomeState?
    @State private var desktopPlatformSetupReceiptRuntimeOutcomeState: DesktopPlatformSetupReceiptRuntimeOutcomeState?
    @State private var desktopTermsAcceptRuntimeOutcomeState: DesktopTermsAcceptRuntimeOutcomeState?
    @State private var desktopPrimaryDeviceConfirmRuntimeOutcomeState: DesktopPrimaryDeviceConfirmRuntimeOutcomeState?
    @State private var desktopVoiceEnrollRuntimeOutcomeState: DesktopVoiceEnrollRuntimeOutcomeState?
    @State private var desktopWakeEnrollStartDraftRuntimeOutcomeState: DesktopWakeEnrollStartDraftRuntimeOutcomeState?
    @State private var desktopWakeEnrollSampleCommitRuntimeOutcomeState: DesktopWakeEnrollSampleCommitRuntimeOutcomeState?
    @State private var desktopWakeEnrollCompleteCommitRuntimeOutcomeState: DesktopWakeEnrollCompleteCommitRuntimeOutcomeState?
    @State private var desktopOnboardingContinueFieldInput: String = ""
    @State private var desktopAuthoritativeReplyRenderState: DesktopAuthoritativeReplyRenderState?
    @State private var desktopAuthoritativeReplyProvenanceRenderState: DesktopAuthoritativeReplyProvenanceRenderState?
    @State private var desktopAuthoritativeReplyPlaybackState: DesktopAuthoritativeReplyPlaybackState = .idle

    var body: some View {
        HStack(alignment: .top, spacing: 20) {
            VStack(alignment: .leading, spacing: 16) {
                posturePanel

                historyCard
            }
            .frame(width: 270, alignment: .topLeading)

            VStack(alignment: .leading, spacing: 16) {
                explicitVoiceEntryAffordanceCard

                desktopOnboardingEntryCard
                desktopOnboardingContinuePromptCard
                desktopPlatformSetupReceiptSubmissionCard
                desktopTermsAcceptCard
                desktopPrimaryDeviceConfirmCard
                desktopVoiceEnrollCard
                desktopWakeEnrollStartDraftCard
                desktopWakeEnrollSampleCommitCard
                desktopWakeEnrollCompleteCommitCard

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
        .task(id: explicitVoiceController.pendingRequest?.id) {
            await dispatchPreparedExplicitVoiceRequestIfNeeded()
        }
        .task(id: desktopOnboardingEntryContext?.id) {
            await openInviteLinkAndStartOnboardingIfNeeded()
        }
        .task(id: desktopOnboardingContinuePromptSeedID) {
            await fetchOnboardingContinuePromptIfNeeded()
        }
        .onReceive(desktopAuthoritativeReplyPlaybackController.$playbackState) { playbackState in
            desktopAuthoritativeReplyPlaybackState = playbackState
        }
        .onDisappear {
            explicitVoiceController.haltCaptureSession()
            desktopCanonicalRuntimeBridge.stopManagedAdapter()
            desktopAuthoritativeReplyPlaybackController.reset()
        }
        .onOpenURL { url in
            if let context = DesktopOnboardingEntryContext(url: url) {
                if desktopOnboardingEntryContext?.id != context.id {
                    desktopInviteOpenRuntimeOutcomeState = nil
                    desktopOnboardingContinueRuntimeOutcomeState = nil
                    desktopPlatformSetupReceiptRuntimeOutcomeState = nil
                    desktopTermsAcceptRuntimeOutcomeState = nil
                    desktopPrimaryDeviceConfirmRuntimeOutcomeState = nil
                    desktopVoiceEnrollRuntimeOutcomeState = nil
                    desktopWakeEnrollStartDraftRuntimeOutcomeState = nil
                    desktopWakeEnrollSampleCommitRuntimeOutcomeState = nil
                    desktopWakeEnrollCompleteCommitRuntimeOutcomeState = nil
                    desktopOnboardingContinueFieldInput = ""
                }
                desktopOnboardingEntryContext = context
            }

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

    private var explicitVoiceEntryAffordanceCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Explicit voice entry now produces and dispatches a bounded desktop explicit voice-turn request only after foreground user initiation. Capture, transcript preview, canonical runtime dispatch status, pending posture, and failed posture remain explicit, bounded, and cloud-authoritative.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .center, spacing: 12) {
                    Image(systemName: "mic.circle")
                        .font(.system(size: 28))
                        .foregroundStyle(.secondary)

                    VStack(alignment: .leading, spacing: 4) {
                        Text("Explicit voice entry")
                            .font(.headline)

                        Text("Bounded foreground macOS capture and non-authoritative transcript preview only. This surface dispatches only bounded canonical runtime ingress and still does not synthesize assistant output, reply playback, or wake-listener authority.")
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
                    posturePill("Desktop foreground capture")
                    posturePill("Transcript preview only")
                }

                HStack(spacing: 8) {
                    posturePill("Cloud authoritative")
                    posturePill("No wake parity")
                    posturePill("No local authority")
                }

                VStack(alignment: .leading, spacing: 10) {
                    VStack(alignment: .leading, spacing: 6) {
                        HStack(alignment: .firstTextBaseline, spacing: 12) {
                            Text("microphone_permission")
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)

                            Spacer()

                            Text(explicitVoiceController.microphonePermission.rawValue)
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)
                        }

                        Text(explicitVoiceController.microphonePermission.detail)
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .padding(12)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color(nsColor: .controlBackgroundColor))
                    .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))

                    VStack(alignment: .leading, spacing: 6) {
                        HStack(alignment: .firstTextBaseline, spacing: 12) {
                            Text("speech_recognition_permission")
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)

                            Spacer()

                            Text(explicitVoiceController.speechRecognitionPermission.rawValue)
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)
                        }

                        Text(explicitVoiceController.speechRecognitionPermission.detail)
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .padding(12)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color(nsColor: .controlBackgroundColor))
                    .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
                }

                Text("Explicit foreground user action is required before microphone capture or speech recognition starts. This shell does not begin capture on wake or background triggers.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(spacing: 12) {
                    Button("Start explicit voice turn") {
                        desktopCanonicalRuntimeOutcomeState = nil
                        desktopAuthoritativeReplyRenderState = nil
                        desktopAuthoritativeReplyProvenanceRenderState = nil
                        desktopAuthoritativeReplyPlaybackController.reset()
                        desktopAuthoritativeReplyPlaybackState = .idle
                        explicitVoiceController.startExplicitVoiceTurn()
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(explicitVoiceController.isListening || explicitVoiceController.pendingRequest != nil)

                    Button("Stop capture and prepare voice request") {
                        desktopCanonicalRuntimeOutcomeState = nil
                        desktopAuthoritativeReplyRenderState = nil
                        desktopAuthoritativeReplyProvenanceRenderState = nil
                        desktopAuthoritativeReplyPlaybackController.reset()
                        desktopAuthoritativeReplyPlaybackState = .idle
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

                if let desktopCanonicalRuntimeOutcomeState {
                    desktopCanonicalRuntimeOutcomeCard(desktopCanonicalRuntimeOutcomeState)
                }

                if desktopAuthoritativeReplyRenderState != nil {
                    desktopAuthoritativeReplyCard
                    desktopAuthoritativeReplyProvenanceCard
                    desktopAuthoritativeReplyPlaybackCard
                }

                if let failedRequest = explicitVoiceController.failedRequest {
                    VStack(alignment: .leading, spacing: 8) {
                        Text(failedRequest.title)
                            .font(.subheadline.weight(.semibold))

                        Text(failedRequest.summary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text(failedRequest.detail)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .padding(12)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color(nsColor: .controlBackgroundColor))
                    .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
                }

                Text("No wake parity claim, no proven native macOS wake-listener integration claim, no autonomous-unlock claim, and no local authority claim are introduced by this affordance.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Explicit Voice Entry Affordance")
                .font(.headline)
        }
    }

    private var desktopOnboardingContinuePromptState: DesktopOnboardingContinuePromptState? {
        if let desktopOnboardingContinueRuntimeOutcomeState,
           desktopOnboardingContinueRuntimeOutcomeState.phase == .completed,
           desktopOnboardingContinueRuntimeOutcomeState.nextStep == "ASK_MISSING",
           let onboardingSessionID = desktopOnboardingContinueRuntimeOutcomeState.onboardingSessionID {
            let fallbackRemainingFields = desktopInviteOpenRuntimeOutcomeState?.requiredFields ?? []
            let remainingMissingFields = desktopOnboardingContinueRuntimeOutcomeState.remainingMissingFields.isEmpty
                ? fallbackRemainingFields
                : desktopOnboardingContinueRuntimeOutcomeState.remainingMissingFields
            let blockingField = desktopOnboardingContinueRuntimeOutcomeState.blockingField
                ?? remainingMissingFields.first

            if let blockingField {
                return DesktopOnboardingContinuePromptState(
                    onboardingSessionID: onboardingSessionID,
                    nextStep: "ASK_MISSING",
                    blockingField: blockingField,
                    blockingQuestion: desktopOnboardingContinueRuntimeOutcomeState.blockingQuestion,
                    remainingMissingFields: remainingMissingFields.isEmpty ? [blockingField] : remainingMissingFields
                )
            }
        }

        if let desktopInviteOpenRuntimeOutcomeState,
           desktopInviteOpenRuntimeOutcomeState.phase == .completed,
           desktopInviteOpenRuntimeOutcomeState.nextStep == "ASK_MISSING",
           let onboardingSessionID = desktopInviteOpenRuntimeOutcomeState.onboardingSessionID,
           let blockingField = desktopInviteOpenRuntimeOutcomeState.requiredFields.first {
            return DesktopOnboardingContinuePromptState(
                onboardingSessionID: onboardingSessionID,
                nextStep: "ASK_MISSING",
                blockingField: blockingField,
                blockingQuestion: nil,
                remainingMissingFields: desktopInviteOpenRuntimeOutcomeState.requiredFields
            )
        }

        return nil
    }

    private var desktopOnboardingContinuePromptSeedID: String? {
        guard desktopOnboardingContinueRuntimeOutcomeState == nil,
              let desktopOnboardingContinuePromptState,
              desktopInviteOpenRuntimeOutcomeState?.phase == .completed else {
            return nil
        }

        return desktopOnboardingContinuePromptState.id
    }

    private var boundedDesktopOnboardingContinueFieldInput: String? {
        boundedOnboardingContinueFieldInput(desktopOnboardingContinueFieldInput)
    }

    private var desktopPlatformSetupReceiptPresentationState: (
        onboardingSessionID: String,
        nextStep: String,
        onboardingStatus: String?,
        remainingPlatformReceiptKinds: [String]
    )? {
        if let desktopPlatformSetupReceiptRuntimeOutcomeState {
            return (
                onboardingSessionID: desktopPlatformSetupReceiptRuntimeOutcomeState.onboardingSessionID
                    ?? desktopOnboardingContinueRuntimeOutcomeState?.onboardingSessionID
                    ?? desktopInviteOpenRuntimeOutcomeState?.onboardingSessionID
                    ?? "unavailable",
                nextStep: desktopPlatformSetupReceiptRuntimeOutcomeState.nextStep ?? "not_provided",
                onboardingStatus: desktopPlatformSetupReceiptRuntimeOutcomeState.onboardingStatus,
                remainingPlatformReceiptKinds: desktopPlatformSetupReceiptRuntimeOutcomeState.remainingPlatformReceiptKinds
            )
        }

        if let desktopOnboardingContinueRuntimeOutcomeState,
           desktopOnboardingContinueRuntimeOutcomeState.phase == .completed,
           desktopOnboardingContinueRuntimeOutcomeState.nextStep != "ASK_MISSING",
           let onboardingSessionID = desktopOnboardingContinueRuntimeOutcomeState.onboardingSessionID {
            return (
                onboardingSessionID: onboardingSessionID,
                nextStep: desktopOnboardingContinueRuntimeOutcomeState.nextStep ?? "not_provided",
                onboardingStatus: desktopOnboardingContinueRuntimeOutcomeState.onboardingStatus,
                remainingPlatformReceiptKinds: desktopOnboardingContinueRuntimeOutcomeState.remainingPlatformReceiptKinds
            )
        }

        return nil
    }

    private var desktopPlatformSetupReceiptDrafts: [DesktopPlatformSetupReceiptDraft] {
        guard let presentationState = desktopPlatformSetupReceiptPresentationState else {
            return []
        }

        var drafts: [DesktopPlatformSetupReceiptDraft] = []
        let remainingKinds = Set(presentationState.remainingPlatformReceiptKinds)

        if remainingKinds.contains("install_launch_handshake") {
            drafts.append(
                DesktopPlatformSetupReceiptDraft(
                    onboardingSessionID: presentationState.onboardingSessionID,
                    receiptKind: "install_launch_handshake",
                    proofMaterial: "onboarding_session_id=\(presentationState.onboardingSessionID)|receipt_kind=install_launch_handshake|launch_posture=desktop_shell_live|app_platform=DESKTOP",
                    proofSummary: "Locally provable from bounded live desktop shell posture only."
                )
            )
        }

        if remainingKinds.contains("mic_permission_granted"),
           case .granted = explicitVoiceController.microphonePermission {
            drafts.append(
                DesktopPlatformSetupReceiptDraft(
                    onboardingSessionID: presentationState.onboardingSessionID,
                    receiptKind: "mic_permission_granted",
                    proofMaterial: "onboarding_session_id=\(presentationState.onboardingSessionID)|receipt_kind=mic_permission_granted|microphone_permission=granted|app_platform=DESKTOP",
                    proofSummary: "Locally provable from current granted microphone permission posture only."
                )
            )
        }

        return drafts
    }

    private var desktopTermsAcceptPromptState: DesktopTermsAcceptPromptState? {
        if let desktopTermsAcceptRuntimeOutcomeState,
           desktopTermsAcceptRuntimeOutcomeState.phase == .completed,
           desktopTermsAcceptRuntimeOutcomeState.nextStep != "TERMS" {
            return nil
        }

        if let desktopPlatformSetupReceiptRuntimeOutcomeState,
           desktopPlatformSetupReceiptRuntimeOutcomeState.phase == .completed,
           desktopPlatformSetupReceiptRuntimeOutcomeState.nextStep == "TERMS",
           let onboardingSessionID = desktopPlatformSetupReceiptRuntimeOutcomeState.onboardingSessionID {
            return DesktopTermsAcceptPromptState(
                onboardingSessionID: onboardingSessionID,
                nextStep: "TERMS",
                termsVersionID: desktopCanonicalTermsVersionID
            )
        }

        if let desktopOnboardingContinueRuntimeOutcomeState,
           desktopOnboardingContinueRuntimeOutcomeState.phase == .completed,
           desktopOnboardingContinueRuntimeOutcomeState.nextStep == "TERMS",
           let onboardingSessionID = desktopOnboardingContinueRuntimeOutcomeState.onboardingSessionID {
            return DesktopTermsAcceptPromptState(
                onboardingSessionID: onboardingSessionID,
                nextStep: "TERMS",
                termsVersionID: desktopCanonicalTermsVersionID
            )
        }

        return nil
    }

    private var desktopManagedPrimaryDeviceID: String? {
        let trimmed = desktopCanonicalRuntimeBridge.managedDeviceID
            .trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty, trimmed.count <= 128, !trimmed.contains("\n"), !trimmed.contains("\r") else {
            return nil
        }

        return trimmed
    }

    private var desktopPrimaryDeviceConfirmPromptState: DesktopPrimaryDeviceConfirmPromptState? {
        if let desktopPrimaryDeviceConfirmRuntimeOutcomeState,
           desktopPrimaryDeviceConfirmRuntimeOutcomeState.phase == .completed,
           desktopPrimaryDeviceConfirmRuntimeOutcomeState.nextStep != "PRIMARY_DEVICE_CONFIRM" {
            return nil
        }

        if let desktopTermsAcceptRuntimeOutcomeState,
           desktopTermsAcceptRuntimeOutcomeState.phase == .completed,
           desktopTermsAcceptRuntimeOutcomeState.nextStep == "PRIMARY_DEVICE_CONFIRM",
           let onboardingSessionID = desktopTermsAcceptRuntimeOutcomeState.onboardingSessionID,
           let deviceID = desktopManagedPrimaryDeviceID {
            return DesktopPrimaryDeviceConfirmPromptState(
                onboardingSessionID: onboardingSessionID,
                nextStep: "PRIMARY_DEVICE_CONFIRM",
                deviceID: deviceID,
                proofOK: true
            )
        }

        return nil
    }

    private var boundedDesktopVoiceEnrollTranscriptPreview: String? {
        let trimmed = explicitVoiceController.transcriptPreview
            .trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty,
              trimmed.utf8.count <= 4_096,
              !trimmed.contains("\n"),
              !trimmed.contains("\r") else {
            return nil
        }

        return trimmed
    }

    private var desktopVoiceEnrollPromptState: DesktopVoiceEnrollPromptState? {
        if let desktopVoiceEnrollRuntimeOutcomeState,
           desktopVoiceEnrollRuntimeOutcomeState.phase == .completed,
           desktopVoiceEnrollRuntimeOutcomeState.nextStep != "VOICE_ENROLL" {
            return nil
        }

        if let desktopPrimaryDeviceConfirmRuntimeOutcomeState,
           desktopPrimaryDeviceConfirmRuntimeOutcomeState.phase == .completed,
           desktopPrimaryDeviceConfirmRuntimeOutcomeState.nextStep == "VOICE_ENROLL",
           let onboardingSessionID = desktopPrimaryDeviceConfirmRuntimeOutcomeState.onboardingSessionID,
           let deviceID = desktopManagedPrimaryDeviceID,
           let transcriptPreview = boundedDesktopVoiceEnrollTranscriptPreview {
            return DesktopVoiceEnrollPromptState(
                onboardingSessionID: onboardingSessionID,
                nextStep: "VOICE_ENROLL",
                deviceID: deviceID,
                transcriptPreview: transcriptPreview
            )
        }

        return nil
    }

    private var desktopWakeEnrollStartDraftPromptState: DesktopWakeEnrollStartDraftPromptState? {
        if let desktopWakeEnrollStartDraftRuntimeOutcomeState,
           desktopWakeEnrollStartDraftRuntimeOutcomeState.phase == .completed {
            return nil
        }

        if let desktopVoiceEnrollRuntimeOutcomeState,
           desktopVoiceEnrollRuntimeOutcomeState.phase == .completed,
           desktopVoiceEnrollRuntimeOutcomeState.nextStep == "WAKE_ENROLL",
           let onboardingSessionID = desktopVoiceEnrollRuntimeOutcomeState.onboardingSessionID,
           let deviceID = desktopManagedPrimaryDeviceID {
            return DesktopWakeEnrollStartDraftPromptState(
                onboardingSessionID: onboardingSessionID,
                nextStep: "WAKE_ENROLL",
                deviceID: deviceID,
                voiceArtifactSyncReceiptRef: desktopVoiceEnrollRuntimeOutcomeState.voiceArtifactSyncReceiptRef
            )
        }

        return nil
    }

    private var desktopWakeEnrollSampleCommitPromptState: DesktopWakeEnrollSampleCommitPromptState? {
        if let desktopWakeEnrollSampleCommitRuntimeOutcomeState,
           desktopWakeEnrollSampleCommitRuntimeOutcomeState.phase == .completed {
            guard desktopWakeEnrollSampleCommitRuntimeOutcomeState.nextStep == "WAKE_ENROLL",
                  let onboardingSessionID = desktopWakeEnrollSampleCommitRuntimeOutcomeState.onboardingSessionID,
                  let deviceID = desktopManagedPrimaryDeviceID else {
                return nil
            }

            return DesktopWakeEnrollSampleCommitPromptState(
                onboardingSessionID: onboardingSessionID,
                nextStep: "WAKE_ENROLL",
                deviceID: deviceID,
                proofOK: true,
                voiceArtifactSyncReceiptRef: desktopWakeEnrollSampleCommitRuntimeOutcomeState.voiceArtifactSyncReceiptRef
            )
        }

        if let desktopWakeEnrollStartDraftRuntimeOutcomeState,
           desktopWakeEnrollStartDraftRuntimeOutcomeState.phase == .completed,
           desktopWakeEnrollStartDraftRuntimeOutcomeState.nextStep == "WAKE_ENROLL",
           let onboardingSessionID = desktopWakeEnrollStartDraftRuntimeOutcomeState.onboardingSessionID,
           let deviceID = desktopManagedPrimaryDeviceID {
            return DesktopWakeEnrollSampleCommitPromptState(
                onboardingSessionID: onboardingSessionID,
                nextStep: "WAKE_ENROLL",
                deviceID: deviceID,
                proofOK: true,
                voiceArtifactSyncReceiptRef: desktopWakeEnrollStartDraftRuntimeOutcomeState.voiceArtifactSyncReceiptRef
            )
        }

        return nil
    }

    private var desktopWakeEnrollCompleteCommitPromptState: DesktopWakeEnrollCompleteCommitPromptState? {
        if let desktopWakeEnrollCompleteCommitRuntimeOutcomeState,
           desktopWakeEnrollCompleteCommitRuntimeOutcomeState.phase == .completed,
           desktopWakeEnrollCompleteCommitRuntimeOutcomeState.nextStep != "WAKE_ENROLL" {
            return nil
        }

        if let desktopWakeEnrollSampleCommitRuntimeOutcomeState,
           desktopWakeEnrollSampleCommitRuntimeOutcomeState.phase == .completed,
           desktopWakeEnrollSampleCommitRuntimeOutcomeState.nextStep == "WAKE_ENROLL",
           let onboardingSessionID = desktopWakeEnrollSampleCommitRuntimeOutcomeState.onboardingSessionID,
           let deviceID = desktopManagedPrimaryDeviceID {
            return DesktopWakeEnrollCompleteCommitPromptState(
                onboardingSessionID: onboardingSessionID,
                nextStep: "WAKE_ENROLL",
                deviceID: deviceID,
                voiceArtifactSyncReceiptRef: desktopWakeEnrollSampleCommitRuntimeOutcomeState.voiceArtifactSyncReceiptRef
            )
        }

        return nil
    }

    private var desktopUnsupportedPlatformSetupReceiptKinds: [String] {
        guard let presentationState = desktopPlatformSetupReceiptPresentationState else {
            return []
        }

        let supportedKinds = Set(desktopPlatformSetupReceiptDrafts.map(\.receiptKind))
        return presentationState.remainingPlatformReceiptKinds.filter { !supportedKinds.contains($0) }
    }

    private func desktopPlatformSetupReceiptReadOnlyDetail(for receiptKind: String) -> String {
        switch receiptKind {
        case "mic_permission_granted":
            if case .granted = explicitVoiceController.microphonePermission {
                return "Current microphone permission is granted and can be submitted from this bounded shell while it remains required."
            }

            return "Current microphone permission is `\(explicitVoiceController.microphonePermission.rawValue)`, so this receipt remains visible but not locally provable yet."
        default:
            return onboardingPlatformSetupReceiptDetail(for: receiptKind)
        }
    }

    @ViewBuilder
    private var desktopOnboardingContinuePromptCard: some View {
        let cardState = desktopOnboardingContinuePromptState
            ?? desktopOnboardingContinueRuntimeOutcomeState.map {
                DesktopOnboardingContinuePromptState(
                    onboardingSessionID: $0.onboardingSessionID ?? "unavailable",
                    nextStep: $0.nextStep ?? "not_provided",
                    blockingField: $0.blockingField ?? "not_provided",
                    blockingQuestion: $0.blockingQuestion,
                    remainingMissingFields: $0.remainingMissingFields
                )
            }

        if let cardState,
           desktopInviteOpenRuntimeOutcomeState?.phase == .completed || desktopOnboardingContinueRuntimeOutcomeState != nil {
            GroupBox {
                VStack(alignment: .leading, spacing: 12) {
                    Text("Bounded onboarding continue missing-field prompt-and-submit only. This shell derives prompt state from the already-live onboarding-entry outcome plus returned continue outcome, dispatches exact `ASK_MISSING_SUBMIT`, and stops when canonical runtime advances beyond `ASK_MISSING`.")
                        .frame(maxWidth: .infinity, alignment: .leading)

                    ForEach(
                        [
                            ("onboarding_session_id", cardState.onboardingSessionID),
                            ("next_step", desktopOnboardingContinueRuntimeOutcomeState?.nextStep ?? cardState.nextStep),
                            ("blocking_field", desktopOnboardingContinueRuntimeOutcomeState?.blockingField ?? cardState.blockingField),
                            ("blocking_question", desktopOnboardingContinueRuntimeOutcomeState?.blockingQuestion ?? cardState.blockingQuestion ?? "not_provided"),
                        ],
                        id: \.0
                    ) { row in
                        HStack(alignment: .top, spacing: 12) {
                            Text(row.0)
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)
                                .frame(width: 170, alignment: .leading)

                            Text(row.1)
                                .font(.body.monospaced())
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }

                    desktopOnboardingEntryListCard(
                        title: "remaining_missing_fields",
                        items: cardState.remainingMissingFields,
                        emptyText: "No remaining_missing_fields are currently visible in the bounded missing-field prompt state."
                    )

                    if cardState.nextStep == "ASK_MISSING" {
                        VStack(alignment: .leading, spacing: 8) {
                            TextField(
                                cardState.blockingQuestion
                                    ?? "Enter a bounded value for \(cardState.blockingField)",
                                text: $desktopOnboardingContinueFieldInput
                            )
                            .textFieldStyle(.roundedBorder)
                            .disabled(desktopOnboardingContinueRuntimeOutcomeState?.phase == .dispatching)

                            Button("Submit missing field") {
                                guard let boundedFieldValue = boundedDesktopOnboardingContinueFieldInput else {
                                    return
                                }

                                Task {
                                    await dispatchOnboardingContinueMissingField(
                                        promptState: cardState,
                                        fieldValue: boundedFieldValue
                                    )
                                }
                            }
                            .buttonStyle(.borderedProminent)
                            .disabled(
                                desktopOnboardingContinueRuntimeOutcomeState?.phase == .dispatching
                                    || boundedDesktopOnboardingContinueFieldInput == nil
                            )
                        }
                    }

                    if let desktopOnboardingContinueRuntimeOutcomeState {
                        Divider()

                        Text(desktopOnboardingContinueRuntimeOutcomeState.title)
                            .font(.headline)

                        ForEach(
                            [
                                ("dispatch_phase", desktopOnboardingContinueRuntimeOutcomeState.phase.rawValue),
                                ("request_id", desktopOnboardingContinueRuntimeOutcomeState.requestID),
                                ("endpoint", desktopOnboardingContinueRuntimeOutcomeState.endpoint),
                                ("outcome", desktopOnboardingContinueRuntimeOutcomeState.outcome ?? "not_available"),
                                ("reason", desktopOnboardingContinueRuntimeOutcomeState.reason ?? "not_available"),
                                ("onboarding_status", desktopOnboardingContinueRuntimeOutcomeState.onboardingStatus ?? "not_available"),
                            ],
                            id: \.0
                        ) { row in
                            HStack(alignment: .top, spacing: 12) {
                                Text(row.0)
                                    .font(.caption.monospaced())
                                    .foregroundStyle(.secondary)
                                    .frame(width: 170, alignment: .leading)

                                Text(row.1)
                                    .font(.body.monospaced())
                                    .frame(maxWidth: .infinity, alignment: .leading)
                            }
                        }

                        Text(desktopOnboardingContinueRuntimeOutcomeState.summary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text(desktopOnboardingContinueRuntimeOutcomeState.detail)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        if desktopOnboardingContinueRuntimeOutcomeState.phase == .completed,
                           desktopOnboardingContinueRuntimeOutcomeState.nextStep != "ASK_MISSING" {
                            desktopOnboardingEntryListCard(
                                title: "remaining_platform_receipt_kinds",
                                items: desktopOnboardingContinueRuntimeOutcomeState.remainingPlatformReceiptKinds,
                                emptyText: "No remaining_platform_receipt_kinds were returned after the bounded missing-field loop advanced."
                            )
                        }
                    } else {
                        Text("Awaiting bounded missing-field prompt visibility from canonical `/v1/onboarding/continue`. This surface will not expose platform receipts, terms acceptance, primary-device confirmation, voice enrollment, access provisioning, wake controls, or autonomous unlock.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }

                    Text("Only the exact `ASK_MISSING_SUBMIT` action is in scope here. No platform-receipt submission controls, no terms acceptance controls, no primary-device confirmation controls, no voice-enrollment controls, no access-provision controls, no wake-enrollment controls, no proven native macOS wake-listener integration claim, and no autonomous-unlock claim are introduced by this surface.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            } label: {
                Text("Onboarding Continue Missing Field")
                    .font(.headline)
            }
        }
    }

    @ViewBuilder
    private var desktopPlatformSetupReceiptSubmissionCard: some View {
        if let presentationState = desktopPlatformSetupReceiptPresentationState,
           desktopOnboardingContinueRuntimeOutcomeState?.phase == .completed
                || desktopPlatformSetupReceiptRuntimeOutcomeState != nil {
            GroupBox {
                VStack(alignment: .leading, spacing: 12) {
                    Text("Bounded desktop platform-setup receipt submission only. This shell derives exact locally provable drafts for `install_launch_handshake` and `mic_permission_granted`, dispatches exact `PLATFORM_SETUP_RECEIPT`, and keeps unsupported remaining receipt kinds read-only.")
                        .frame(maxWidth: .infinity, alignment: .leading)

                    ForEach(
                        [
                            ("onboarding_session_id", presentationState.onboardingSessionID),
                            ("next_step", desktopPlatformSetupReceiptRuntimeOutcomeState?.nextStep ?? presentationState.nextStep),
                            ("onboarding_status", desktopPlatformSetupReceiptRuntimeOutcomeState?.onboardingStatus ?? presentationState.onboardingStatus ?? "not_provided"),
                        ],
                        id: \.0
                    ) { row in
                        HStack(alignment: .top, spacing: 12) {
                            Text(row.0)
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)
                                .frame(width: 170, alignment: .leading)

                            Text(row.1)
                                .font(.body.monospaced())
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }

                    desktopOnboardingEntryListCard(
                        title: "remaining_platform_receipt_kinds",
                        items: presentationState.remainingPlatformReceiptKinds,
                        emptyText: "No remaining_platform_receipt_kinds are currently visible in the bounded platform-setup receipt posture."
                    )

                    if desktopPlatformSetupReceiptDrafts.isEmpty {
                        Text("No locally provable desktop platform-setup receipt drafts are currently actionable in this shell. Remaining receipts stay visible and read-only here.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    } else {
                        VStack(alignment: .leading, spacing: 10) {
                            Text("Supported local receipt drafts")
                                .font(.subheadline.weight(.semibold))

                            ForEach(desktopPlatformSetupReceiptDrafts) { draft in
                                VStack(alignment: .leading, spacing: 8) {
                                    Text(draft.receiptKind)
                                        .font(.caption.monospaced())

                                    Text(draft.proofSummary)
                                        .font(.footnote)
                                        .foregroundStyle(.secondary)
                                        .frame(maxWidth: .infinity, alignment: .leading)

                                    Button(draft.buttonTitle) {
                                        Task {
                                            await dispatchDesktopPlatformSetupReceipt(draft)
                                        }
                                    }
                                    .buttonStyle(.borderedProminent)
                                    .disabled(desktopPlatformSetupReceiptRuntimeOutcomeState?.phase == .dispatching)
                                }
                                .padding(12)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .background(Color(nsColor: .controlBackgroundColor))
                                .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
                            }
                        }
                    }

                    if !desktopUnsupportedPlatformSetupReceiptKinds.isEmpty {
                        VStack(alignment: .leading, spacing: 10) {
                            Text("Read-only remaining receipts")
                                .font(.subheadline.weight(.semibold))

                            ForEach(Array(desktopUnsupportedPlatformSetupReceiptKinds.enumerated()), id: \.offset) { _, receiptKind in
                                VStack(alignment: .leading, spacing: 4) {
                                    Text(receiptKind)
                                        .font(.caption.monospaced())

                                    Text(desktopPlatformSetupReceiptReadOnlyDetail(for: receiptKind))
                                        .font(.footnote)
                                        .foregroundStyle(.secondary)
                                        .frame(maxWidth: .infinity, alignment: .leading)
                                }
                                .padding(12)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .background(Color(nsColor: .controlBackgroundColor))
                                .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
                            }
                        }
                    }

                    if let desktopPlatformSetupReceiptRuntimeOutcomeState {
                        Divider()

                        Text(desktopPlatformSetupReceiptRuntimeOutcomeState.title)
                            .font(.headline)

                        ForEach(
                            [
                                ("dispatch_phase", desktopPlatformSetupReceiptRuntimeOutcomeState.phase.rawValue),
                                ("submitted_receipt_kind", desktopPlatformSetupReceiptRuntimeOutcomeState.receiptKind),
                                ("request_id", desktopPlatformSetupReceiptRuntimeOutcomeState.requestID),
                                ("endpoint", desktopPlatformSetupReceiptRuntimeOutcomeState.endpoint),
                                ("outcome", desktopPlatformSetupReceiptRuntimeOutcomeState.outcome ?? "not_available"),
                                ("reason", desktopPlatformSetupReceiptRuntimeOutcomeState.reason ?? "not_available"),
                            ],
                            id: \.0
                        ) { row in
                            HStack(alignment: .top, spacing: 12) {
                                Text(row.0)
                                    .font(.caption.monospaced())
                                    .foregroundStyle(.secondary)
                                    .frame(width: 170, alignment: .leading)

                                Text(row.1)
                                    .font(.body.monospaced())
                                    .frame(maxWidth: .infinity, alignment: .leading)
                            }
                        }

                        Text(desktopPlatformSetupReceiptRuntimeOutcomeState.summary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text(desktopPlatformSetupReceiptRuntimeOutcomeState.detail)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    } else {
                        Text("Awaiting user-triggered desktop platform-setup receipt submission. Unsupported remaining receipts stay read-only here, and later onboarding actions remain out of scope.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }

                    Text("Only exact `install_launch_handshake` and exact `mic_permission_granted` submission are in scope here. Exact `desktop_wakeword_configured` and exact `desktop_pairing_bound` remain read-only required receipt visibility only, and later onboarding actions remain out of scope.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            } label: {
                Text("Onboarding Platform Setup Receipts")
                    .font(.headline)
            }
        }
    }

    @ViewBuilder
    private var desktopTermsAcceptCard: some View {
        let promptState = desktopTermsAcceptPromptState
        let displayedOnboardingSessionID = promptState?.onboardingSessionID
            ?? desktopTermsAcceptRuntimeOutcomeState?.onboardingSessionID
            ?? "unavailable"
        let displayedNextStep = desktopTermsAcceptRuntimeOutcomeState?.nextStep
            ?? promptState?.nextStep
            ?? "not_provided"
        let displayedTermsVersionID = promptState?.termsVersionID
            ?? desktopTermsAcceptRuntimeOutcomeState?.termsVersionID
            ?? desktopCanonicalTermsVersionID

        if promptState != nil || desktopTermsAcceptRuntimeOutcomeState != nil {
            GroupBox {
                VStack(alignment: .leading, spacing: 12) {
                    Text("Bounded desktop terms acceptance submission only. This shell derives bounded prompt state when canonical onboarding posture has advanced to exact `TERMS`, dispatches exact `TERMS_ACCEPT`, and preserves returned next-step visibility in read-only form only.")
                        .frame(maxWidth: .infinity, alignment: .leading)

                    ForEach(
                        [
                            ("onboarding_session_id", displayedOnboardingSessionID),
                            ("next_step", displayedNextStep),
                            ("terms_version_id", displayedTermsVersionID),
                        ],
                        id: \.0
                    ) { row in
                        HStack(alignment: .top, spacing: 12) {
                            Text(row.0)
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)
                                .frame(width: 170, alignment: .leading)

                            Text(row.1)
                                .font(.body.monospaced())
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }

                    Text("This shell is dispatching canonical terms acceptance only. It does not fabricate a local terms document, a local policy summary, or local onboarding authority.")
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    if let promptState {
                        Button("Accept canonical terms") {
                            Task {
                                await acceptDesktopTerms(promptState: promptState)
                            }
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(desktopTermsAcceptRuntimeOutcomeState?.phase == .dispatching)
                    }

                    if let desktopTermsAcceptRuntimeOutcomeState {
                        Divider()

                        Text(desktopTermsAcceptRuntimeOutcomeState.title)
                            .font(.headline)

                        ForEach(
                            [
                                ("dispatch_phase", desktopTermsAcceptRuntimeOutcomeState.phase.rawValue),
                                ("request_id", desktopTermsAcceptRuntimeOutcomeState.requestID),
                                ("endpoint", desktopTermsAcceptRuntimeOutcomeState.endpoint),
                                ("accepted", desktopTermsAcceptRuntimeOutcomeState.accepted ? "true" : "false"),
                                ("outcome", desktopTermsAcceptRuntimeOutcomeState.outcome ?? "not_available"),
                                ("reason", desktopTermsAcceptRuntimeOutcomeState.reason ?? "not_available"),
                                ("onboarding_status", desktopTermsAcceptRuntimeOutcomeState.onboardingStatus ?? "not_available"),
                            ],
                            id: \.0
                        ) { row in
                            HStack(alignment: .top, spacing: 12) {
                                Text(row.0)
                                    .font(.caption.monospaced())
                                    .foregroundStyle(.secondary)
                                    .frame(width: 170, alignment: .leading)

                                Text(row.1)
                                    .font(.body.monospaced())
                                    .frame(maxWidth: .infinity, alignment: .leading)
                            }
                        }

                        Text(desktopTermsAcceptRuntimeOutcomeState.summary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text(desktopTermsAcceptRuntimeOutcomeState.detail)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    } else {
                        Text("Awaiting explicit user-triggered canonical terms acceptance. This surface does not render a local terms document and does not expose later onboarding controls.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }

                    Text("Only exact `TERMS_ACCEPT` with exact current repo-truth `terms_v1` and exact `accepted=true` is in scope here. No decline flow, no sender-verification controls, no primary-device confirmation controls, no voice-enrollment controls, no pairing-completion controls, no wake-enrollment controls, and no proven native macOS wake-listener integration claim are introduced by this surface.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            } label: {
                Text("Onboarding Terms Acceptance")
                    .font(.headline)
            }
        }
    }

    @ViewBuilder
    private var desktopPrimaryDeviceConfirmCard: some View {
        let promptState = desktopPrimaryDeviceConfirmPromptState
        let displayedOnboardingSessionID = promptState?.onboardingSessionID
            ?? desktopPrimaryDeviceConfirmRuntimeOutcomeState?.onboardingSessionID
            ?? "unavailable"
        let displayedNextStep = desktopPrimaryDeviceConfirmRuntimeOutcomeState?.nextStep
            ?? promptState?.nextStep
            ?? "not_provided"
        let displayedDeviceID = promptState?.deviceID
            ?? desktopPrimaryDeviceConfirmRuntimeOutcomeState?.deviceID
            ?? desktopManagedPrimaryDeviceID
            ?? "not_provided"
        let displayedProofOK = desktopPrimaryDeviceConfirmRuntimeOutcomeState?.proofOK
            ?? promptState?.proofOK
            ?? true

        if promptState != nil || desktopPrimaryDeviceConfirmRuntimeOutcomeState != nil {
            GroupBox {
                VStack(alignment: .leading, spacing: 12) {
                    Text("Bounded desktop primary-device confirmation submission only. This shell derives bounded prompt state when canonical onboarding posture has advanced to exact `PRIMARY_DEVICE_CONFIRM`, dispatches exact `PRIMARY_DEVICE_CONFIRM`, and preserves returned next-step visibility in read-only form only.")
                        .frame(maxWidth: .infinity, alignment: .leading)

                    ForEach(
                        [
                            ("onboarding_session_id", displayedOnboardingSessionID),
                            ("next_step", displayedNextStep),
                            ("device_id", displayedDeviceID),
                            ("proof_ok", displayedProofOK ? "true" : "false"),
                        ],
                        id: \.0
                    ) { row in
                        HStack(alignment: .top, spacing: 12) {
                            Text(row.0)
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)
                                .frame(width: 170, alignment: .leading)

                            Text(row.1)
                                .font(.body.monospaced())
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }

                    Text("This shell is dispatching canonical primary-device confirmation only. It does not add sender-verification logic, employee-photo capture, local onboarding authority, or later onboarding controls.")
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    if let promptState {
                        Button("Confirm this desktop as primary device") {
                            Task {
                                await confirmDesktopPrimaryDevice(promptState: promptState)
                            }
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(desktopPrimaryDeviceConfirmRuntimeOutcomeState?.phase == .dispatching)
                    }

                    if let desktopPrimaryDeviceConfirmRuntimeOutcomeState {
                        Divider()

                        Text(desktopPrimaryDeviceConfirmRuntimeOutcomeState.title)
                            .font(.headline)

                        ForEach(
                            [
                                ("dispatch_phase", desktopPrimaryDeviceConfirmRuntimeOutcomeState.phase.rawValue),
                                ("request_id", desktopPrimaryDeviceConfirmRuntimeOutcomeState.requestID),
                                ("endpoint", desktopPrimaryDeviceConfirmRuntimeOutcomeState.endpoint),
                                ("outcome", desktopPrimaryDeviceConfirmRuntimeOutcomeState.outcome ?? "not_available"),
                                ("reason", desktopPrimaryDeviceConfirmRuntimeOutcomeState.reason ?? "not_available"),
                                ("onboarding_status", desktopPrimaryDeviceConfirmRuntimeOutcomeState.onboardingStatus ?? "not_available"),
                            ],
                            id: \.0
                        ) { row in
                            HStack(alignment: .top, spacing: 12) {
                                Text(row.0)
                                    .font(.caption.monospaced())
                                    .foregroundStyle(.secondary)
                                    .frame(width: 170, alignment: .leading)

                                Text(row.1)
                                    .font(.body.monospaced())
                                    .frame(maxWidth: .infinity, alignment: .leading)
                            }
                        }

                        Text(desktopPrimaryDeviceConfirmRuntimeOutcomeState.summary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text(desktopPrimaryDeviceConfirmRuntimeOutcomeState.detail)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    } else {
                        Text("Awaiting explicit user-triggered canonical primary-device confirmation. This surface does not expose sender verification, employee-photo capture, voice-enrollment controls, pairing completion, wake controls, or autonomous unlock.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }

                    Text("Only exact `PRIMARY_DEVICE_CONFIRM` with the exact managed bridge `deviceID` and exact `proofOK=true` is in scope here. No sender-verification controls, no employee-photo controls, no voice-enrollment controls, no pairing-completion controls, no wake-enrollment controls, and no proven native macOS wake-listener integration claim are introduced by this surface.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            } label: {
                Text("Onboarding Primary Device Confirmation")
                    .font(.headline)
            }
        }
    }

    @ViewBuilder
    private var desktopVoiceEnrollCard: some View {
        let promptState = desktopVoiceEnrollPromptState
        let primaryDeviceVoiceContext = (
            desktopPrimaryDeviceConfirmRuntimeOutcomeState?.phase == .completed
            && desktopPrimaryDeviceConfirmRuntimeOutcomeState?.nextStep == "VOICE_ENROLL"
        ) ? desktopPrimaryDeviceConfirmRuntimeOutcomeState : nil
        let displayedOnboardingSessionID = promptState?.onboardingSessionID
            ?? desktopVoiceEnrollRuntimeOutcomeState?.onboardingSessionID
            ?? primaryDeviceVoiceContext?.onboardingSessionID
            ?? "unavailable"
        let displayedNextStep = desktopVoiceEnrollRuntimeOutcomeState?.nextStep
            ?? promptState?.nextStep
            ?? primaryDeviceVoiceContext?.nextStep
            ?? "not_provided"
        let displayedDeviceID = promptState?.deviceID
            ?? desktopVoiceEnrollRuntimeOutcomeState?.deviceID
            ?? primaryDeviceVoiceContext?.deviceID
            ?? desktopManagedPrimaryDeviceID
            ?? "not_provided"
        let sampleSeedPosture = promptState != nil
            ? "deterministic_from_explicit_voice_transcript_preview_and_onboarding_context"
            : displayedNextStep == "VOICE_ENROLL"
                ? "bounded_explicit_voice_transcript_preview_required_before_dispatch"
                : "read_only_after_voice_enroll_dispatch"

        if promptState != nil || primaryDeviceVoiceContext != nil || desktopVoiceEnrollRuntimeOutcomeState != nil {
            GroupBox {
                VStack(alignment: .leading, spacing: 12) {
                    Text("Bounded desktop voice-enroll lock submission only. This shell derives bounded prompt state when canonical onboarding posture has advanced to exact `VOICE_ENROLL`, derives one exact bounded deterministic `sample_seed` from the already-live explicit voice transcript preview plus bounded onboarding context, dispatches exact `VOICE_ENROLL_LOCK`, and keeps returned `WAKE_ENROLL` visibility read-only only.")
                        .frame(maxWidth: .infinity, alignment: .leading)

                    ForEach(
                        [
                            ("onboarding_session_id", displayedOnboardingSessionID),
                            ("next_step", displayedNextStep),
                            ("device_id", displayedDeviceID),
                            ("sample_seed_posture", sampleSeedPosture),
                        ],
                        id: \.0
                    ) { row in
                        HStack(alignment: .top, spacing: 12) {
                            Text(row.0)
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)
                                .frame(width: 170, alignment: .leading)

                            Text(row.1)
                                .font(.body.monospaced())
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }

                    Text(promptState != nil
                        ? "A deterministic bounded `sample_seed` will be derived from the current explicit voice transcript preview together with `onboarding_session_id`, exact `next_step`, and the exact managed bridge `device_id`. This shell does not introduce new voice-capture authority or any wake mutation."
                        : displayedNextStep == "VOICE_ENROLL"
                            ? "The exact seed source for this surface is the already-live bounded explicit voice transcript preview. Produce a bounded transcript preview first, then this shell can derive the deterministic `sample_seed` and dispatch exact `VOICE_ENROLL_LOCK`."
                            : "This shell now preserves returned voice-enroll completion posture in read-only form only. Any returned `WAKE_ENROLL` visibility remains unsubmitted here.")
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    if let promptState {
                        Button("Lock bounded voice enrollment") {
                            Task {
                                await submitDesktopVoiceEnrollLock(promptState: promptState)
                            }
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(desktopVoiceEnrollRuntimeOutcomeState?.phase == .dispatching)
                    }

                    if let desktopVoiceEnrollRuntimeOutcomeState {
                        Divider()

                        Text(desktopVoiceEnrollRuntimeOutcomeState.title)
                            .font(.headline)

                        ForEach(
                            [
                                ("dispatch_phase", desktopVoiceEnrollRuntimeOutcomeState.phase.rawValue),
                                ("request_id", desktopVoiceEnrollRuntimeOutcomeState.requestID),
                                ("endpoint", desktopVoiceEnrollRuntimeOutcomeState.endpoint),
                                ("outcome", desktopVoiceEnrollRuntimeOutcomeState.outcome ?? "not_available"),
                                ("reason", desktopVoiceEnrollRuntimeOutcomeState.reason ?? "not_available"),
                                ("onboarding_status", desktopVoiceEnrollRuntimeOutcomeState.onboardingStatus ?? "not_available"),
                                ("voice_artifact_sync_receipt_ref", desktopVoiceEnrollRuntimeOutcomeState.voiceArtifactSyncReceiptRef ?? "not_available"),
                            ],
                            id: \.0
                        ) { row in
                            HStack(alignment: .top, spacing: 12) {
                                Text(row.0)
                                    .font(.caption.monospaced())
                                    .foregroundStyle(.secondary)
                                    .frame(width: 170, alignment: .leading)

                                Text(row.1)
                                    .font(.body.monospaced())
                                    .frame(maxWidth: .infinity, alignment: .leading)
                            }
                        }

                        Text(desktopVoiceEnrollRuntimeOutcomeState.summary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text(desktopVoiceEnrollRuntimeOutcomeState.detail)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    } else {
                        Text(promptState != nil
                            ? "Awaiting explicit user-triggered canonical voice-enroll lock. If canonical runtime advances to `WAKE_ENROLL` for desktop, this shell will preserve that next step in read-only form only."
                            : "Canonical onboarding posture is at exact `VOICE_ENROLL`, but this shell is still waiting for an already-live bounded explicit voice transcript preview before it can lawfully derive `sample_seed` and dispatch exact `VOICE_ENROLL_LOCK`.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }

                    Text("Only exact `VOICE_ENROLL_LOCK` with the exact managed bridge `deviceID` and one exact bounded `sample_seed` derived only from already-live bounded explicit voice transcript preview is in scope here. No sender-verification controls, no employee-photo controls, no wake-enrollment controls, no emo-persona controls, no access-provision controls, no pairing-completion controls, and no proven native macOS wake-listener integration claim are introduced by this surface.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            } label: {
                Text("Onboarding Voice Enrollment Lock")
                    .font(.headline)
            }
        }
    }

    @ViewBuilder
    private var desktopWakeEnrollStartDraftCard: some View {
        let promptState = desktopWakeEnrollStartDraftPromptState
        let voiceWakeContext = (
            desktopVoiceEnrollRuntimeOutcomeState?.phase == .completed
            && desktopVoiceEnrollRuntimeOutcomeState?.nextStep == "WAKE_ENROLL"
        ) ? desktopVoiceEnrollRuntimeOutcomeState : nil
        let displayedOnboardingSessionID = promptState?.onboardingSessionID
            ?? desktopWakeEnrollStartDraftRuntimeOutcomeState?.onboardingSessionID
            ?? voiceWakeContext?.onboardingSessionID
            ?? "unavailable"
        let displayedNextStep = desktopWakeEnrollStartDraftRuntimeOutcomeState?.nextStep
            ?? promptState?.nextStep
            ?? voiceWakeContext?.nextStep
            ?? "not_provided"
        let displayedDeviceID = promptState?.deviceID
            ?? desktopWakeEnrollStartDraftRuntimeOutcomeState?.deviceID
            ?? voiceWakeContext?.deviceID
            ?? desktopManagedPrimaryDeviceID
            ?? "not_provided"
        let displayedVoiceArtifactSyncReceiptRef = desktopWakeEnrollStartDraftRuntimeOutcomeState?.voiceArtifactSyncReceiptRef
            ?? promptState?.voiceArtifactSyncReceiptRef
            ?? voiceWakeContext?.voiceArtifactSyncReceiptRef
            ?? "not_available"

        if promptState != nil || voiceWakeContext != nil || desktopWakeEnrollStartDraftRuntimeOutcomeState != nil {
            GroupBox {
                VStack(alignment: .leading, spacing: 12) {
                    Text("Bounded desktop wake-enroll start-draft submission only. This shell derives bounded prompt state when canonical onboarding posture has advanced to exact `WAKE_ENROLL`, dispatches exact `WAKE_ENROLL_START_DRAFT`, and keeps returned exact `WAKE_ENROLL` visibility plus exact `voice_artifact_sync_receipt_ref` read-only only.")
                        .frame(maxWidth: .infinity, alignment: .leading)

                    ForEach(
                        [
                            ("onboarding_session_id", displayedOnboardingSessionID),
                            ("next_step", displayedNextStep),
                            ("device_id", displayedDeviceID),
                            ("voice_artifact_sync_receipt_ref", displayedVoiceArtifactSyncReceiptRef),
                        ],
                        id: \.0
                    ) { row in
                        HStack(alignment: .top, spacing: 12) {
                            Text(row.0)
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)
                                .frame(width: 170, alignment: .leading)

                            Text(row.1)
                                .font(.body.monospaced())
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }

                    Text("This exact surface is dispatching canonical wake-enroll start draft only. Any later wake-sample and wake-complete controls are separately gated from this surface, while wake-defer controls, local wake authority, and proven native macOS wake-listener claims remain out of scope here.")
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    if let promptState {
                        Button("Start bounded wake enrollment") {
                            Task {
                                await submitDesktopWakeEnrollStartDraft(promptState: promptState)
                            }
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(desktopWakeEnrollStartDraftRuntimeOutcomeState?.phase == .dispatching)
                    }

                    if let desktopWakeEnrollStartDraftRuntimeOutcomeState {
                        Divider()

                        Text(desktopWakeEnrollStartDraftRuntimeOutcomeState.title)
                            .font(.headline)

                        ForEach(
                            [
                                ("dispatch_phase", desktopWakeEnrollStartDraftRuntimeOutcomeState.phase.rawValue),
                                ("request_id", desktopWakeEnrollStartDraftRuntimeOutcomeState.requestID),
                                ("endpoint", desktopWakeEnrollStartDraftRuntimeOutcomeState.endpoint),
                                ("outcome", desktopWakeEnrollStartDraftRuntimeOutcomeState.outcome ?? "not_available"),
                                ("reason", desktopWakeEnrollStartDraftRuntimeOutcomeState.reason ?? "not_available"),
                                ("onboarding_status", desktopWakeEnrollStartDraftRuntimeOutcomeState.onboardingStatus ?? "not_available"),
                            ],
                            id: \.0
                        ) { row in
                            HStack(alignment: .top, spacing: 12) {
                                Text(row.0)
                                    .font(.caption.monospaced())
                                    .foregroundStyle(.secondary)
                                    .frame(width: 170, alignment: .leading)

                                Text(row.1)
                                    .font(.body.monospaced())
                                    .frame(maxWidth: .infinity, alignment: .leading)
                            }
                        }

                        Text(desktopWakeEnrollStartDraftRuntimeOutcomeState.summary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text(desktopWakeEnrollStartDraftRuntimeOutcomeState.detail)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    } else {
                        Text("Awaiting explicit user-triggered canonical wake-enroll start draft. After submission, any returned exact `WAKE_ENROLL` posture and any returned exact `voice_artifact_sync_receipt_ref` remain read-only only in this shell while later wake-sample and wake-complete controls stay separately gated.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }

                    Text("Only exact `WAKE_ENROLL_START_DRAFT` with the exact managed bridge `deviceID` is in scope here. Any later wake-sample and wake-complete control is separately gated from this surface; no wake-defer controls, no sender-verification controls, no employee-photo controls, no emo-persona controls, no access-provision controls, no pairing-completion controls, and no proven native macOS wake-listener integration claim are introduced by this surface.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            } label: {
                Text("Onboarding Wake Enrollment Start Draft")
                    .font(.headline)
            }
        }
    }

    @ViewBuilder
    private var desktopWakeEnrollSampleCommitCard: some View {
        let promptState = desktopWakeEnrollSampleCommitPromptState
        let wakeStartSampleContext = (
            desktopWakeEnrollStartDraftRuntimeOutcomeState?.phase == .completed
            && desktopWakeEnrollStartDraftRuntimeOutcomeState?.nextStep == "WAKE_ENROLL"
        ) ? desktopWakeEnrollStartDraftRuntimeOutcomeState : nil
        let displayedOnboardingSessionID = promptState?.onboardingSessionID
            ?? desktopWakeEnrollSampleCommitRuntimeOutcomeState?.onboardingSessionID
            ?? wakeStartSampleContext?.onboardingSessionID
            ?? "unavailable"
        let displayedNextStep = desktopWakeEnrollSampleCommitRuntimeOutcomeState?.nextStep
            ?? promptState?.nextStep
            ?? wakeStartSampleContext?.nextStep
            ?? "not_provided"
        let displayedDeviceID = promptState?.deviceID
            ?? desktopWakeEnrollSampleCommitRuntimeOutcomeState?.deviceID
            ?? wakeStartSampleContext?.deviceID
            ?? desktopManagedPrimaryDeviceID
            ?? "not_provided"
        let displayedProofOK = desktopWakeEnrollSampleCommitRuntimeOutcomeState?.proofOK
            ?? promptState?.proofOK
            ?? false
        let displayedVoiceArtifactSyncReceiptRef = desktopWakeEnrollSampleCommitRuntimeOutcomeState?.voiceArtifactSyncReceiptRef
            ?? promptState?.voiceArtifactSyncReceiptRef
            ?? wakeStartSampleContext?.voiceArtifactSyncReceiptRef
            ?? "not_available"

        if promptState != nil || wakeStartSampleContext != nil || desktopWakeEnrollSampleCommitRuntimeOutcomeState != nil {
            GroupBox {
                VStack(alignment: .leading, spacing: 12) {
                    Text("Bounded desktop wake-enroll sample-commit submission only. This shell derives bounded prompt state only while canonical onboarding posture remains at exact `WAKE_ENROLL`, dispatches exact `WAKE_ENROLL_SAMPLE_COMMIT` with exact `proof_ok=true`, and keeps repeated sample-commit submission explicit and user-triggered only.")
                        .frame(maxWidth: .infinity, alignment: .leading)

                    ForEach(
                        [
                            ("onboarding_session_id", displayedOnboardingSessionID),
                            ("next_step", displayedNextStep),
                            ("device_id", displayedDeviceID),
                            ("proof_ok", displayedProofOK ? "true" : "false"),
                            ("voice_artifact_sync_receipt_ref", displayedVoiceArtifactSyncReceiptRef),
                        ],
                        id: \.0
                    ) { row in
                        HStack(alignment: .top, spacing: 12) {
                            Text(row.0)
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)
                                .frame(width: 170, alignment: .leading)

                            Text(row.1)
                                .font(.body.monospaced())
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }

                    Text("This exact sample-commit surface dispatches canonical wake-enroll sample commit only. It does not batch or auto-loop requests. Another explicit submit remains available only while lawful prompt state remains present and canonical `next_step` remains exact `WAKE_ENROLL`.")
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    if let promptState {
                        Button("Commit bounded wake sample") {
                            Task {
                                await submitDesktopWakeEnrollSampleCommit(promptState: promptState)
                            }
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(desktopWakeEnrollSampleCommitRuntimeOutcomeState?.phase == .dispatching)
                    }

                    if let desktopWakeEnrollSampleCommitRuntimeOutcomeState {
                        Divider()

                        Text(desktopWakeEnrollSampleCommitRuntimeOutcomeState.title)
                            .font(.headline)

                        ForEach(
                            [
                                ("dispatch_phase", desktopWakeEnrollSampleCommitRuntimeOutcomeState.phase.rawValue),
                                ("request_id", desktopWakeEnrollSampleCommitRuntimeOutcomeState.requestID),
                                ("endpoint", desktopWakeEnrollSampleCommitRuntimeOutcomeState.endpoint),
                                ("outcome", desktopWakeEnrollSampleCommitRuntimeOutcomeState.outcome ?? "not_available"),
                                ("reason", desktopWakeEnrollSampleCommitRuntimeOutcomeState.reason ?? "not_available"),
                                ("onboarding_status", desktopWakeEnrollSampleCommitRuntimeOutcomeState.onboardingStatus ?? "not_available"),
                            ],
                            id: \.0
                        ) { row in
                            HStack(alignment: .top, spacing: 12) {
                                Text(row.0)
                                    .font(.caption.monospaced())
                                    .foregroundStyle(.secondary)
                                    .frame(width: 170, alignment: .leading)

                                Text(row.1)
                                    .font(.body.monospaced())
                                    .frame(maxWidth: .infinity, alignment: .leading)
                            }
                        }

                        Text(desktopWakeEnrollSampleCommitRuntimeOutcomeState.summary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text(desktopWakeEnrollSampleCommitRuntimeOutcomeState.detail)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    } else {
                        Text(promptState != nil
                            ? "Awaiting explicit user-triggered canonical wake-enroll sample commit. If canonical runtime remains at exact `WAKE_ENROLL`, another explicit sample commit can be submitted from this same bounded surface."
                            : "Read-only wake-enrollment posture only. Another bounded wake sample commit is unavailable until lawful prompt state is present at exact `WAKE_ENROLL`.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }

                    Text("Only exact `WAKE_ENROLL_SAMPLE_COMMIT` with the exact managed bridge `deviceID` and exact `proofOK=true` is in scope here. Any wake-complete control remains separately gated from this surface; no wake-defer controls, no sender-verification controls, no employee-photo controls, no emo-persona controls, no access-provision controls, no pairing-completion controls, and no proven native macOS wake-listener integration claim are introduced by this surface.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            } label: {
                Text("Onboarding Wake Enrollment Sample Commit")
                    .font(.headline)
            }
        }
    }

    @ViewBuilder
    private var desktopWakeEnrollCompleteCommitCard: some View {
        let promptState = desktopWakeEnrollCompleteCommitPromptState
        let wakeSampleCompleteContext = desktopWakeEnrollSampleCommitRuntimeOutcomeState?.phase == .completed
            ? desktopWakeEnrollSampleCommitRuntimeOutcomeState
            : nil
        let displayedOnboardingSessionID = promptState?.onboardingSessionID
            ?? desktopWakeEnrollCompleteCommitRuntimeOutcomeState?.onboardingSessionID
            ?? wakeSampleCompleteContext?.onboardingSessionID
            ?? "unavailable"
        let displayedNextStep = desktopWakeEnrollCompleteCommitRuntimeOutcomeState?.nextStep
            ?? promptState?.nextStep
            ?? wakeSampleCompleteContext?.nextStep
            ?? "not_provided"
        let displayedDeviceID = promptState?.deviceID
            ?? desktopWakeEnrollCompleteCommitRuntimeOutcomeState?.deviceID
            ?? wakeSampleCompleteContext?.deviceID
            ?? desktopManagedPrimaryDeviceID
            ?? "not_provided"
        let displayedVoiceArtifactSyncReceiptRef = desktopWakeEnrollCompleteCommitRuntimeOutcomeState?.voiceArtifactSyncReceiptRef
            ?? promptState?.voiceArtifactSyncReceiptRef
            ?? wakeSampleCompleteContext?.voiceArtifactSyncReceiptRef
            ?? "not_available"

        if promptState != nil || wakeSampleCompleteContext != nil || desktopWakeEnrollCompleteCommitRuntimeOutcomeState != nil {
            GroupBox {
                VStack(alignment: .leading, spacing: 12) {
                    Text("Bounded desktop wake-enroll complete-commit submission only. This shell derives bounded prompt state from already-live wake-sample outcome only while canonical onboarding posture remains at exact `WAKE_ENROLL`, dispatches exact wake-enroll complete commit, and keeps returned `EMO_PERSONA_LOCK`, returned `voice_artifact_sync_receipt_ref`, and any returned `WAKE_ENROLL` visibility read-only only outside the exact wake-complete control itself.")
                        .frame(maxWidth: .infinity, alignment: .leading)

                    ForEach(
                        [
                            ("onboarding_session_id", displayedOnboardingSessionID),
                            ("next_step", displayedNextStep),
                            ("device_id", displayedDeviceID),
                            ("voice_artifact_sync_receipt_ref", displayedVoiceArtifactSyncReceiptRef),
                        ],
                        id: \.0
                    ) { row in
                        HStack(alignment: .top, spacing: 12) {
                            Text(row.0)
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)
                                .frame(width: 170, alignment: .leading)

                            Text(row.1)
                                .font(.body.monospaced())
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }

                    Text("This exact surface is dispatching canonical wake-enroll complete commit only. If canonical runtime advances to exact `EMO_PERSONA_LOCK`, that next-step visibility remains read-only only here and does not unlock emo-persona submit behavior.")
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    if let promptState {
                        Button("Complete bounded wake enrollment") {
                            Task {
                                await submitDesktopWakeEnrollCompleteCommit(promptState: promptState)
                            }
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(desktopWakeEnrollCompleteCommitRuntimeOutcomeState?.phase == .dispatching)
                    }

                    if let desktopWakeEnrollCompleteCommitRuntimeOutcomeState {
                        Divider()

                        Text(desktopWakeEnrollCompleteCommitRuntimeOutcomeState.title)
                            .font(.headline)

                        ForEach(
                            [
                                ("dispatch_phase", desktopWakeEnrollCompleteCommitRuntimeOutcomeState.phase.rawValue),
                                ("request_id", desktopWakeEnrollCompleteCommitRuntimeOutcomeState.requestID),
                                ("endpoint", desktopWakeEnrollCompleteCommitRuntimeOutcomeState.endpoint),
                                ("outcome", desktopWakeEnrollCompleteCommitRuntimeOutcomeState.outcome ?? "not_available"),
                                ("reason", desktopWakeEnrollCompleteCommitRuntimeOutcomeState.reason ?? "not_available"),
                                ("onboarding_status", desktopWakeEnrollCompleteCommitRuntimeOutcomeState.onboardingStatus ?? "not_available"),
                            ],
                            id: \.0
                        ) { row in
                            HStack(alignment: .top, spacing: 12) {
                                Text(row.0)
                                    .font(.caption.monospaced())
                                    .foregroundStyle(.secondary)
                                    .frame(width: 170, alignment: .leading)

                                Text(row.1)
                                    .font(.body.monospaced())
                                    .frame(maxWidth: .infinity, alignment: .leading)
                            }
                        }

                        Text(desktopWakeEnrollCompleteCommitRuntimeOutcomeState.summary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text(desktopWakeEnrollCompleteCommitRuntimeOutcomeState.detail)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    } else {
                        Text(promptState != nil
                            ? "Awaiting explicit user-triggered canonical wake-enroll complete commit. If canonical runtime remains at exact `WAKE_ENROLL`, this bounded complete-control can be retried explicitly from the same lawful prompt posture."
                            : "Read-only wake-enrollment posture only. A bounded wake-complete commit is unavailable until lawful prompt state remains present at exact `WAKE_ENROLL`.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }

                    Text("Only exact wake-enroll complete commit with the exact managed bridge `deviceID` is in scope here. No wake-defer controls, no emo-persona controls, no access-provision controls, no pairing-completion controls, no sender-verification controls, no employee-photo controls, and no proven native macOS wake-listener integration claim are introduced by this surface.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            } label: {
                Text("Onboarding Wake Enrollment Complete Commit")
                    .font(.headline)
            }
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

    @ViewBuilder
    private var desktopOnboardingEntryCard: some View {
        if let desktopOnboardingEntryContext {
            GroupBox {
                VStack(alignment: .leading, spacing: 12) {
                    Text(desktopOnboardingEntryContext.routeKind.title)
                        .font(.headline)

                    Text("Bounded app-open / invite-open onboarding entry only. This shell parses lawful route context, dispatches canonical `/v1/invite/click`, and renders returned onboarding-start posture in read-only form without widening into onboarding-continue mutation or local onboarding authority.")
                        .frame(maxWidth: .infinity, alignment: .leading)

                    VStack(alignment: .leading, spacing: 8) {
                        ForEach(Array(desktopOnboardingEntryContext.routeRows.enumerated()), id: \.offset) { entry in
                            let row = entry.element
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
                    }

                    if let desktopInviteOpenRuntimeOutcomeState {
                        Divider()

                        Text(desktopInviteOpenRuntimeOutcomeState.title)
                            .font(.headline)

                        ForEach(
                            [
                                ("dispatch_phase", desktopInviteOpenRuntimeOutcomeState.phase.rawValue),
                                ("request_id", desktopInviteOpenRuntimeOutcomeState.requestID),
                                ("endpoint", desktopInviteOpenRuntimeOutcomeState.endpoint),
                                ("outcome", desktopInviteOpenRuntimeOutcomeState.outcome ?? "not_available"),
                                ("reason", desktopInviteOpenRuntimeOutcomeState.reason ?? "not_available"),
                                ("onboarding_session_id", desktopInviteOpenRuntimeOutcomeState.onboardingSessionID ?? "not_available"),
                                ("next_step", desktopInviteOpenRuntimeOutcomeState.nextStep ?? "not_available"),
                            ],
                            id: \.0
                        ) { row in
                            HStack(alignment: .top, spacing: 12) {
                                Text(row.0)
                                    .font(.caption.monospaced())
                                    .foregroundStyle(.secondary)
                                    .frame(width: 170, alignment: .leading)

                                Text(row.1)
                                    .font(.body.monospaced())
                                    .frame(maxWidth: .infinity, alignment: .leading)
                            }
                        }

                        Text(desktopInviteOpenRuntimeOutcomeState.summary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text(desktopInviteOpenRuntimeOutcomeState.detail)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        desktopOnboardingEntryListCard(
                            title: "required_fields",
                            items: desktopInviteOpenRuntimeOutcomeState.requiredFields,
                            emptyText: "No required_fields were returned in the bounded onboarding-start outcome."
                        )

                        desktopOnboardingEntryListCard(
                            title: "required_verification_gates",
                            items: desktopInviteOpenRuntimeOutcomeState.requiredVerificationGates,
                            emptyText: "No required_verification_gates were returned in the bounded onboarding-start outcome."
                        )
                    } else {
                        Text("Awaiting bounded invite-open dispatch. This shell stays read-only and does not locally activate invites, continue onboarding, or bypass canonical runtime routing.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }

                    Text("Read-only onboarding-entry visibility only. No onboarding-continue action controls, no platform-receipt submission controls, no access-provision controls, no pairing-completion controls, no wake-enrollment controls, no proven native macOS wake-listener integration claim, and no autonomous-unlock claim are introduced by this surface.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            } label: {
                Text("Onboarding Entry")
                    .font(.headline)
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

                        if latestSessionActiveVisibleContext
                            .hasLawfulOnboardingPlatformSetupReceiptCarrierFamily
                        {
                            onboardingPlatformSetupReceiptCard(latestSessionActiveVisibleContext)
                        }

                        if latestSessionActiveVisibleContext.hasLawfulAuthorityStateCarrierFamily {
                            authorityStateCard(latestSessionActiveVisibleContext)
                        }

                        if latestSessionActiveVisibleContext
                            .hasLawfulWakeRuntimeEventEvidenceCarrierFamily
                        {
                            wakeRuntimeEventEvidenceCard(latestSessionActiveVisibleContext)
                        }

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

    private func onboardingPlatformSetupReceiptCard(
        _ context: DesktopSessionActiveVisibleContext
    ) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Onboarding platform-setup receipts")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored onboarding platform-setup receipt evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            metadataRow(
                label: "remaining_platform_receipt_kinds",
                value: context.remainingPlatformReceiptKinds.joined(separator: ", ")
            )

            ForEach(Array(context.remainingPlatformReceiptKinds.enumerated()), id: \.offset) { _, receiptKind in
                VStack(alignment: .leading, spacing: 4) {
                    Text(receiptKind)
                        .font(.caption.monospaced())

                    Text(onboardingPlatformSetupReceiptDetail(for: receiptKind))
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            }

            Text("Read-only platform-setup receipt visibility only. No local onboarding authority, no local wake authority, no local governance authority, no local proof authority, and no local dispatch unlock.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func onboardingPlatformSetupReceiptDetail(for receiptKind: String) -> String {
        switch receiptKind {
        case "install_launch_handshake":
            return "Canonical install / first-launch handshake receipt family rendered as bounded read-only setup evidence."
        case "mic_permission_granted":
            return "Canonical microphone-permission receipt family rendered without mutating device policy locally."
        case "desktop_wakeword_configured":
            return "Canonical desktop wake-word setup receipt family rendered as cloud-authored evidence only and not as proven wake-listener authority."
        case "desktop_pairing_bound":
            return "Canonical desktop pairing receipt family rendered as bounded setup evidence only and not as local authority adoption."
        default:
            return "Unknown receipt family."
        }
    }

    private func authorityStateCard(_ context: DesktopSessionActiveVisibleContext) -> some View {
        Group {
            if let authorityStateSimulationCertificationState =
                context.authorityStateSimulationCertificationState,
               let authorityStateOnboardingReadinessState =
                context.authorityStateOnboardingReadinessState,
               let authorityStatePolicyDecision = context.authorityStatePolicyDecision,
               let authorityStateIdentityScopeRequired =
                context.authorityStateIdentityScopeRequired,
               let authorityStateIdentityScopeSatisfied =
                context.authorityStateIdentityScopeSatisfied,
               let authorityStateMemoryScopeAllowed = context.authorityStateMemoryScopeAllowed
            {
                VStack(alignment: .leading, spacing: 10) {
                    Text("Authority state")
                        .font(.subheadline.weight(.semibold))

                    Text("Cloud-authored authority-state evidence only")
                        .font(.footnote.weight(.semibold))
                        .frame(maxWidth: .infinity, alignment: .leading)

                    Text("Authority posture")
                        .font(.footnote.weight(.semibold))
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    metadataRow(
                        label: "authority_state_policy_context_ref",
                        value: context.authorityStatePolicyContextRef ?? "not_provided"
                    )
                    metadataRow(
                        label: "authority_state_simulation_certification_state",
                        value: authorityStateSimulationCertificationState
                    )
                    metadataRow(
                        label: "authority_state_onboarding_readiness_state",
                        value: authorityStateOnboardingReadinessState
                    )
                    metadataRow(
                        label: "authority_state_policy_decision",
                        value: authorityStatePolicyDecision
                    )
                    metadataRow(
                        label: "authority_state_identity_scope_required",
                        value: booleanValue(authorityStateIdentityScopeRequired)
                    )
                    metadataRow(
                        label: "authority_state_identity_scope_satisfied",
                        value: booleanValue(authorityStateIdentityScopeSatisfied)
                    )
                    metadataRow(
                        label: "authority_state_memory_scope_allowed",
                        value: booleanValue(authorityStateMemoryScopeAllowed)
                    )
                    metadataRow(
                        label: "authority_state_reason_code",
                        value: context.authorityStateReasonCode ?? "not_provided"
                    )

                    Text("Exact cloud-authored authority-state family only")
                        .font(.footnote.weight(.semibold))
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    Text("No local authority law, no local governance authority, no local proof authority, and no local dispatch unlock.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                .padding(12)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color(nsColor: .controlBackgroundColor))
                .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
            }
        }
    }

    private func wakeRuntimeEventEvidenceCard(
        _ context: DesktopSessionActiveVisibleContext
    ) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Wake runtime event evidence")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored wake runtime event evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            ForEach(context.wakeRuntimeEventEvidenceRows, id: \.label) { row in
                metadataRow(label: row.label, value: row.value)
            }

            Text("Bounded read-only wake-entry evidence only. No local wake-listener authority, no local threshold law, no local wake runtime law, no local session-open authority, no local entry unlock, no wake parity, and no autonomous-unlock capability.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
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

                if let resumeBufferExpiresAt = context.resumeBufferExpiresAt,
                   context.hasLawfulInterruptResumeBufferExpiresAt {
                    interruptResumeBufferExpiresAtCard(resumeBufferExpiresAt)
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
                    interruptTtsResumeSnapshotResponseTextCard(ttsResumeSnapshotResponseText)
                }

                if let ttsResumeSnapshotTopicHint = context.ttsResumeSnapshotTopicHint,
                   context.hasLawfulInterruptTtsResumeSnapshotTopicHint {
                    interruptTtsResumeSnapshotTopicHintCard(ttsResumeSnapshotTopicHint)
                }

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

            if let interruptClarifyRequiresConfirmation = context.interruptClarifyRequiresConfirmation,
               context.hasLawfulInterruptClarifyRequiresConfirmation {
                interruptClarifyConfirmationCard(interruptClarifyRequiresConfirmation)
            }

            if let interruptClarifySensitivityLevel = context.interruptClarifySensitivityLevel,
               context.hasLawfulInterruptClarifySensitivityLevel {
                interruptClarifySensitivityCard(interruptClarifySensitivityLevel)
            }

            if let interruptSubjectRelationConfidence = context.interruptSubjectRelationConfidence,
               context.hasLawfulInterruptSubjectRelationConfidence {
                interruptSubjectRelationConfidenceCard(interruptSubjectRelationConfidence)
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

    private func interruptClarifyConfirmationCard(_ requiresConfirmation: Bool) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Clarify confirmation")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored confirmation evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("Confirmation posture")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            metadataRow(
                label: "interrupt_clarify_requires_confirmation",
                value: booleanValue(requiresConfirmation)
            )

            Text("Exact cloud-authored confirmation truth only")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("No local confirmation law, no local execution unlock.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func interruptClarifySensitivityCard(_ sensitivityLevel: String) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Clarify sensitivity")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored sensitivity evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("Sensitivity posture")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            metadataRow(
                label: "interrupt_clarify_sensitivity_level",
                value: sensitivityLevel
            )

            Text("Exact cloud-authored sensitivity level only")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("No local sensitivity policy, no local authority upgrade.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func interruptSubjectRelationConfidenceCard(_ confidence: String) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Subject relation confidence")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored continuity confidence evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("Confidence posture")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            metadataRow(
                label: "interrupt_subject_relation_confidence",
                value: confidence
            )

            Text("Exact cloud-authored confidence only")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("No local threshold law, no local dispatch unlock.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func interruptSubjectReferencesCard(
        activeSubjectRef: String?,
        interruptedSubjectRef: String?
    ) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Subject references")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored continuity subject evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("Subject posture")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            if let activeSubjectRef {
                metadataRow(
                    label: "active_subject_ref",
                    value: activeSubjectRef
                )
            }

            if let interruptedSubjectRef {
                metadataRow(
                    label: "interrupted_subject_ref",
                    value: interruptedSubjectRef
                )
            }

            Text("Exact cloud-authored subject refs only")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("No local subject binding, no local dispatch unlock.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func interruptReturnCheckExpiryCard(_ returnCheckExpiresAt: String) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Return-check expiry")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored return-check expiry evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("Expiry posture")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            metadataRow(
                label: "return_check_expires_at",
                value: returnCheckExpiresAt
            )

            Text("Exact cloud-authored expiry only")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("No local countdown, no local dispatch unlock.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func interruptResumeBufferLiveCard(_ resumeBufferLive: Bool) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Resume buffer")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored resume-buffer live evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("Resume posture")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            metadataRow(
                label: "resume_buffer_live",
                value: booleanValue(resumeBufferLive)
            )

            Text("Exact cloud-authored liveness truth only")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("No local resume authoring, no local dispatch unlock.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func interruptResumeBufferExpiresAtCard(_ resumeBufferExpiresAt: String) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Resume buffer expiry")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored resume-buffer expiry evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("Expiry posture")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            metadataRow(
                label: "resume_buffer_expires_at",
                value: resumeBufferExpiresAt
            )

            Text("Exact cloud-authored expiry only")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("No local countdown, no local expiry authority, no local resume authoring, and no local dispatch unlock.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func interruptResumeBufferAnswerIDCard(_ resumeBufferAnswerID: String) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Resume answer ID")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored resume-buffer answer-ID evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("Answer posture")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            metadataRow(
                label: "Resume buffer answer ID",
                value: resumeBufferAnswerID
            )

            Text("Exact cloud-authored answer ID only")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("No local resume authoring, no local topic synthesis, and no local dispatch unlock.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func interruptResumeBufferSpokenPrefixCard(
        _ resumeBufferSpokenPrefix: String
    ) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Resume spoken prefix")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored resume-buffer spoken-prefix evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("Prefix posture")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            metadataRow(
                label: "Resume buffer spoken prefix",
                value: resumeBufferSpokenPrefix
            )

            Text("Exact cloud-authored spoken prefix only")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("No local resume authoring, no local synthesis of the remaining response, and no local dispatch unlock.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func interruptResumeBufferUnsaidRemainderCard(
        _ resumeBufferUnsaidRemainder: String
    ) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Resume unsaid remainder")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored resume-buffer unsaid-remainder evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("Remainder posture")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            metadataRow(
                label: "Resume buffer unsaid remainder",
                value: resumeBufferUnsaidRemainder
            )

            Text("Exact cloud-authored unsaid remainder only")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("No local resume authoring, no local completion of the remaining response, and no local dispatch unlock.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func interruptResumeBufferTopicHintCard(_ resumeBufferTopicHint: String) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Resume topic hint")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored resume-buffer topic-hint evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("Topic posture")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            metadataRow(
                label: "Resume buffer topic hint",
                value: resumeBufferTopicHint
            )

            Text("Exact cloud-authored topic hint only")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("No local topic synthesis, no local dispatch unlock.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func interruptTtsResumeSnapshotAnswerIDCard(
        _ ttsResumeSnapshotAnswerID: String
    ) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("TTS snapshot answer ID")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored TTS resume snapshot answer-ID evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("Snapshot answer posture")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            metadataRow(
                label: "TTS resume snapshot answer ID",
                value: ttsResumeSnapshotAnswerID
            )

            Text("Exact cloud-authored snapshot answer ID only")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("No local resume authoring, no local snapshot linkage authority, no local response synthesis, and no local dispatch unlock.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func interruptTtsResumeSnapshotSpokenCursorByteCard(
        _ ttsResumeSnapshotSpokenCursorByte: String
    ) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("TTS snapshot cursor")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored TTS resume snapshot cursor evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("Cursor posture")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            metadataRow(
                label: "TTS resume snapshot cursor byte",
                value: ttsResumeSnapshotSpokenCursorByte
            )

            Text("Exact cloud-authored snapshot cursor only")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("No local resume authoring, no local playback math authority, no local response synthesis, and no local dispatch unlock.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func interruptTtsResumeSnapshotResponseTextCard(
        _ ttsResumeSnapshotResponseText: String
    ) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("TTS snapshot response text")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored TTS resume snapshot response-text evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("Snapshot response posture")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            metadataRow(
                label: "TTS resume snapshot response text",
                value: ttsResumeSnapshotResponseText
            )

            Text("Exact cloud-authored snapshot response text only")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("No local resume authoring, no local cursor math authority, no local response synthesis, no local completion authority, and no local dispatch unlock.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func interruptTtsResumeSnapshotTopicHintCard(
        _ ttsResumeSnapshotTopicHint: String
    ) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("TTS snapshot topic hint")
                .font(.subheadline.weight(.semibold))

            Text("Cloud-authored TTS resume snapshot topic-hint evidence only")
                .font(.footnote.weight(.semibold))
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("Snapshot topic posture")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            metadataRow(
                label: "TTS resume snapshot topic hint",
                value: ttsResumeSnapshotTopicHint
            )

            Text("Exact cloud-authored snapshot topic hint only")
                .font(.footnote.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            Text("No local resume authoring, no local topic synthesis, no local snapshot linkage authority, and no local dispatch unlock.")
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
                        ("request_id", request.id),
                        ("surface", "bounded_desktop_explicit_voice"),
                        ("capture_mode", "foreground_only"),
                        ("transcript_posture", "non_authoritative_preview"),
                        ("transcript_bytes", "\(request.byteCount)"),
                    ],
                    id: \.0
                ) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.0)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 170, alignment: .leading)

                        Text(row.1)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                Text(request.boundedPreview)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("This bounded explicit voice request preview remains session-bound, `EXPLICIT_ONLY`, and non-authoritative while canonical runtime dispatch and later cloud-visible response posture resolve.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Explicit Voice Turn Request")
                .font(.headline)
        }
    }

    private func desktopCanonicalRuntimeOutcomeCard(
        _ outcomeState: DesktopCanonicalRuntimeOutcomeState
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 10) {
                Text(outcomeState.title)
                    .font(.headline)

                ForEach(
                    [
                        ("dispatch_phase", outcomeState.phase.rawValue),
                        ("request_id", outcomeState.requestID),
                        ("endpoint", outcomeState.endpoint),
                        ("outcome", outcomeState.outcome ?? "not_available"),
                        ("next_move", outcomeState.nextMove ?? "not_available"),
                        ("reason_code", outcomeState.reasonCode ?? "not_available"),
                        ("failure_class", outcomeState.failureClass ?? "not_available"),
                        ("session_id", outcomeState.sessionID ?? "not_available"),
                        ("turn_id", outcomeState.turnID ?? "not_available"),
                    ],
                    id: \.0
                ) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.0)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(width: 170, alignment: .leading)

                        Text(row.1)
                            .font(.body.monospaced())
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                Text(outcomeState.summary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text(outcomeState.detail)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Canonical Runtime Dispatch")
                .font(.headline)
        }
    }

    @ViewBuilder
    private var desktopAuthoritativeReplyCard: some View {
        if let desktopAuthoritativeReplyRenderState {
            GroupBox {
                VStack(alignment: .leading, spacing: 10) {
                    Text(desktopAuthoritativeReplyRenderState.title)
                        .font(.headline)

                    Text(desktopAuthoritativeReplyRenderState.summary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    if let authoritativeResponseText = desktopAuthoritativeReplyRenderState.authoritativeResponseText {
                        Text(authoritativeResponseText)
                            .textSelection(.enabled)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    } else {
                        Text("No cloud-authored reply text is available for this completed canonical runtime outcome. This shell stays read-only and does not fabricate local answer content.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }

                    Text("Read-only cloud-authored reply visibility only. This shell remains explicitly non-authoritative, keeps playback out of scope, and does not claim wake parity, native wake-listener integration, or autonomous-unlock capability.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            } label: {
                Text("Authoritative Reply")
                    .font(.headline)
            }
        }
    }

    @ViewBuilder
    private var desktopAuthoritativeReplyProvenanceCard: some View {
        if let desktopAuthoritativeReplyProvenanceRenderState {
            GroupBox {
                VStack(alignment: .leading, spacing: 10) {
                    Text(desktopAuthoritativeReplyProvenanceRenderState.title)
                        .font(.headline)

                    Text(desktopAuthoritativeReplyProvenanceRenderState.summary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    if desktopAuthoritativeReplyProvenanceRenderState.authoritativeResponseProvenance == nil {
                        Text("No cloud-authored provenance is available for this completed canonical runtime outcome. This shell fails closed and does not fabricate local sources, retrieval timing, or cache posture.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    } else {
                        ForEach(
                            [
                                ("retrieved_at", desktopAuthoritativeReplyProvenanceRenderState.retrievedAtLabel ?? "not_available"),
                                ("cache_status", desktopAuthoritativeReplyProvenanceRenderState.cacheStatusLabel ?? "not_available"),
                            ],
                            id: \.0
                        ) { row in
                            HStack(alignment: .top, spacing: 12) {
                                Text(row.0)
                                    .font(.caption.monospaced())
                                    .foregroundStyle(.secondary)
                                    .frame(width: 120, alignment: .leading)

                                Text(row.1)
                                    .font(.body.monospaced())
                                    .frame(maxWidth: .infinity, alignment: .leading)
                            }
                        }

                        if desktopAuthoritativeReplyProvenanceRenderState.sources.isEmpty {
                            Text("Canonical runtime returned provenance posture without source rows. This shell stays read-only and does not claim that search executed locally.")
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        } else {
                            VStack(alignment: .leading, spacing: 8) {
                                Text("Cloud-authored sources")
                                    .font(.subheadline.weight(.semibold))

                                ForEach(desktopAuthoritativeReplyProvenanceRenderState.sources) { source in
                                    VStack(alignment: .leading, spacing: 4) {
                                        Text(source.title)
                                            .font(.body.weight(.medium))
                                            .frame(maxWidth: .infinity, alignment: .leading)

                                        if let sourceURL = URL(string: source.url) {
                                            Link(source.url, destination: sourceURL)
                                                .font(.footnote)
                                        } else {
                                            Text(source.url)
                                                .font(.footnote)
                                                .foregroundStyle(.secondary)
                                                .textSelection(.enabled)
                                                .frame(maxWidth: .infinity, alignment: .leading)
                                        }
                                    }
                                    .padding(.vertical, 2)
                                }
                            }
                        }
                    }

                    Text("Read-only cloud-authored provenance visibility only. This shell remains explicitly non-authoritative, does not claim that search executed locally, and does not widen into new search controls, wake posture, or autonomous-unlock behavior.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            } label: {
                Text("Authoritative Reply Provenance")
                    .font(.headline)
            }
        }
    }

    private var desktopAuthoritativeReplyPlaybackCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 10) {
                Text(desktopAuthoritativeReplyPlaybackState.title)
                    .font(.headline)

                Text(desktopAuthoritativeReplyPlaybackState.summary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text(desktopAuthoritativeReplyPlaybackState.detail)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(spacing: 12) {
                    Button("Play authoritative reply") {
                        playAuthoritativeReply()
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(
                        desktopAuthoritativeReplyRenderState?.authoritativeResponseText?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty != false
                        || desktopAuthoritativeReplyPlaybackState.phase == .speaking
                    )

                    Button("Stop reply playback") {
                        stopAuthoritativeReplyPlayback()
                    }
                    .buttonStyle(.bordered)
                    .disabled(desktopAuthoritativeReplyPlaybackState.phase != .speaking)
                }

                Text("User-triggered bounded reply playback only. No transcript mutation, no conversation-history mutation, no wake parity claim, no proven native macOS wake-listener integration claim, and no autonomous-unlock claim are introduced by this surface.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Authoritative Reply Playback")
                .font(.headline)
        }
    }

    private func playAuthoritativeReply() {
        desktopAuthoritativeReplyPlaybackState = desktopAuthoritativeReplyPlaybackController.play(
            authoritativeResponseText: desktopAuthoritativeReplyRenderState?.authoritativeResponseText
        )
    }

    private func stopAuthoritativeReplyPlayback() {
        desktopAuthoritativeReplyPlaybackState = desktopAuthoritativeReplyPlaybackController.stop()
    }

    @MainActor
    private func openInviteLinkAndStartOnboardingIfNeeded() async {
        guard let desktopOnboardingEntryContext else {
            return
        }

        desktopOnboardingContinueRuntimeOutcomeState = nil
        desktopOnboardingContinueFieldInput = ""

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopInviteClickRequestBuilder(
                desktopOnboardingEntryContext
            )
            desktopInviteOpenRuntimeOutcomeState = .dispatching(
                entryContextID: desktopOnboardingEntryContext.id,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.openInviteLinkAndStartOnboarding(
                ingressContext
            )
            guard self.desktopOnboardingEntryContext?.id == desktopOnboardingEntryContext.id else {
                return
            }

            desktopInviteOpenRuntimeOutcomeState = outcomeState
        } catch {
            desktopInviteOpenRuntimeOutcomeState = .failed(
                entryContextID: desktopOnboardingEntryContext.id,
                endpoint: desktopCanonicalRuntimeBridge.inviteClickEndpoint,
                requestID: "unavailable",
                summary: "The canonical invite-open bridge could not stage this onboarding-entry request.",
                detail: error.localizedDescription
            )
        }
    }

    @MainActor
    private func fetchOnboardingContinuePromptIfNeeded() async {
        guard desktopOnboardingContinueRuntimeOutcomeState == nil,
              let desktopOnboardingContinuePromptState else {
            return
        }

        await dispatchOnboardingContinueMissingField(
            promptState: desktopOnboardingContinuePromptState,
            fieldValue: nil
        )
    }

    @MainActor
    private func dispatchOnboardingContinueMissingField(
        promptState: DesktopOnboardingContinuePromptState,
        fieldValue: String?
    ) async {
        let activeEntryContextID = desktopOnboardingEntryContext?.id
        desktopPlatformSetupReceiptRuntimeOutcomeState = nil
        desktopTermsAcceptRuntimeOutcomeState = nil
        desktopPrimaryDeviceConfirmRuntimeOutcomeState = nil
        desktopWakeEnrollStartDraftRuntimeOutcomeState = nil
        desktopWakeEnrollSampleCommitRuntimeOutcomeState = nil
        desktopWakeEnrollCompleteCommitRuntimeOutcomeState = nil

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopOnboardingContinueMissingFieldRequestBuilder(
                promptState: promptState,
                fieldValue: fieldValue
            )
            desktopOnboardingContinueRuntimeOutcomeState = .dispatching(
                onboardingSessionID: ingressContext.onboardingSessionID,
                blockingField: ingressContext.blockingField,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                submittedFieldValue: fieldValue
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.continueOnboardingMissingField(
                ingressContext
            )
            guard desktopOnboardingEntryContext?.id == activeEntryContextID else {
                return
            }

            desktopOnboardingContinueRuntimeOutcomeState = outcomeState
            desktopOnboardingContinueFieldInput = ""
        } catch {
            desktopOnboardingContinueRuntimeOutcomeState = .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                blockingField: promptState.blockingField,
                endpoint: desktopCanonicalRuntimeBridge.onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded missing-field request.",
                detail: error.localizedDescription
            )
        }
    }

    @MainActor
    private func dispatchDesktopPlatformSetupReceipt(
        _ draft: DesktopPlatformSetupReceiptDraft
    ) async {
        let activeEntryContextID = desktopOnboardingEntryContext?.id
        desktopTermsAcceptRuntimeOutcomeState = nil
        desktopPrimaryDeviceConfirmRuntimeOutcomeState = nil
        desktopWakeEnrollStartDraftRuntimeOutcomeState = nil
        desktopWakeEnrollSampleCommitRuntimeOutcomeState = nil
        desktopWakeEnrollCompleteCommitRuntimeOutcomeState = nil

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopPlatformSetupReceiptRequestBuilder(
                draft
            )
            desktopPlatformSetupReceiptRuntimeOutcomeState = .dispatching(
                onboardingSessionID: ingressContext.onboardingSessionID,
                receiptKind: ingressContext.receiptKind,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.submitDesktopPlatformSetupReceipt(
                ingressContext
            )
            guard desktopOnboardingEntryContext?.id == activeEntryContextID else {
                return
            }

            desktopPlatformSetupReceiptRuntimeOutcomeState = outcomeState
        } catch {
            desktopPlatformSetupReceiptRuntimeOutcomeState = .failed(
                onboardingSessionID: draft.onboardingSessionID,
                receiptKind: draft.receiptKind,
                endpoint: desktopCanonicalRuntimeBridge.onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop platform-setup receipt.",
                detail: error.localizedDescription
            )
        }
    }

    @MainActor
    private func acceptDesktopTerms(
        promptState: DesktopTermsAcceptPromptState
    ) async {
        let activeEntryContextID = desktopOnboardingEntryContext?.id
        desktopPrimaryDeviceConfirmRuntimeOutcomeState = nil
        desktopWakeEnrollStartDraftRuntimeOutcomeState = nil
        desktopWakeEnrollSampleCommitRuntimeOutcomeState = nil
        desktopWakeEnrollCompleteCommitRuntimeOutcomeState = nil

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopTermsAcceptRequestBuilder(
                promptState
            )
            desktopTermsAcceptRuntimeOutcomeState = .dispatching(
                onboardingSessionID: ingressContext.onboardingSessionID,
                termsVersionID: ingressContext.termsVersionID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.acceptDesktopTerms(
                ingressContext
            )
            guard desktopOnboardingEntryContext?.id == activeEntryContextID else {
                return
            }

            desktopTermsAcceptRuntimeOutcomeState = outcomeState
        } catch {
            desktopTermsAcceptRuntimeOutcomeState = .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                termsVersionID: promptState.termsVersionID,
                endpoint: desktopCanonicalRuntimeBridge.onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop terms acceptance request.",
                detail: error.localizedDescription
            )
        }
    }

    @MainActor
    private func confirmDesktopPrimaryDevice(
        promptState: DesktopPrimaryDeviceConfirmPromptState
    ) async {
        let activeEntryContextID = desktopOnboardingEntryContext?.id
        desktopWakeEnrollStartDraftRuntimeOutcomeState = nil
        desktopWakeEnrollSampleCommitRuntimeOutcomeState = nil
        desktopWakeEnrollCompleteCommitRuntimeOutcomeState = nil

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopPrimaryDeviceConfirmRequestBuilder(
                promptState
            )
            desktopPrimaryDeviceConfirmRuntimeOutcomeState = .dispatching(
                onboardingSessionID: ingressContext.onboardingSessionID,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.confirmDesktopPrimaryDevice(
                ingressContext
            )
            guard desktopOnboardingEntryContext?.id == activeEntryContextID else {
                return
            }

            desktopPrimaryDeviceConfirmRuntimeOutcomeState = outcomeState
        } catch {
            desktopPrimaryDeviceConfirmRuntimeOutcomeState = .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                deviceID: promptState.deviceID,
                endpoint: desktopCanonicalRuntimeBridge.onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop primary-device confirmation request.",
                detail: error.localizedDescription
            )
        }
    }

    @MainActor
    private func submitDesktopVoiceEnrollLock(
        promptState: DesktopVoiceEnrollPromptState
    ) async {
        let activeEntryContextID = desktopOnboardingEntryContext?.id
        desktopWakeEnrollStartDraftRuntimeOutcomeState = nil
        desktopWakeEnrollSampleCommitRuntimeOutcomeState = nil
        desktopWakeEnrollCompleteCommitRuntimeOutcomeState = nil

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopVoiceEnrollRequestBuilder(
                promptState
            )
            desktopVoiceEnrollRuntimeOutcomeState = .dispatching(
                onboardingSessionID: ingressContext.onboardingSessionID,
                deviceID: ingressContext.deviceID,
                sampleSeed: ingressContext.sampleSeed,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.submitDesktopVoiceEnrollLock(
                ingressContext
            )
            guard desktopOnboardingEntryContext?.id == activeEntryContextID else {
                return
            }

            desktopVoiceEnrollRuntimeOutcomeState = outcomeState
        } catch {
            desktopVoiceEnrollRuntimeOutcomeState = .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                deviceID: promptState.deviceID,
                sampleSeed: nil,
                endpoint: desktopCanonicalRuntimeBridge.onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop voice-enroll lock request.",
                detail: error.localizedDescription
            )
        }
    }

    @MainActor
    private func submitDesktopWakeEnrollStartDraft(
        promptState: DesktopWakeEnrollStartDraftPromptState
    ) async {
        let activeEntryContextID = desktopOnboardingEntryContext?.id
        desktopWakeEnrollSampleCommitRuntimeOutcomeState = nil
        desktopWakeEnrollCompleteCommitRuntimeOutcomeState = nil

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopWakeEnrollStartDraftRequestBuilder(
                promptState
            )
            desktopWakeEnrollStartDraftRuntimeOutcomeState = .dispatching(
                onboardingSessionID: ingressContext.onboardingSessionID,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.submitDesktopWakeEnrollStartDraft(
                ingressContext
            )
            guard desktopOnboardingEntryContext?.id == activeEntryContextID else {
                return
            }

            desktopWakeEnrollStartDraftRuntimeOutcomeState = outcomeState
        } catch {
            desktopWakeEnrollStartDraftRuntimeOutcomeState = .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                deviceID: promptState.deviceID,
                endpoint: desktopCanonicalRuntimeBridge.onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop wake-enroll start-draft request.",
                detail: error.localizedDescription
            )
        }
    }

    @MainActor
    private func submitDesktopWakeEnrollSampleCommit(
        promptState: DesktopWakeEnrollSampleCommitPromptState
    ) async {
        let activeEntryContextID = desktopOnboardingEntryContext?.id
        desktopWakeEnrollCompleteCommitRuntimeOutcomeState = nil

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopWakeEnrollSampleCommitRequestBuilder(
                promptState
            )
            desktopWakeEnrollSampleCommitRuntimeOutcomeState = .dispatching(
                onboardingSessionID: ingressContext.onboardingSessionID,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.submitDesktopWakeEnrollSampleCommit(
                ingressContext
            )
            guard desktopOnboardingEntryContext?.id == activeEntryContextID else {
                return
            }

            desktopWakeEnrollSampleCommitRuntimeOutcomeState = outcomeState
        } catch {
            desktopWakeEnrollSampleCommitRuntimeOutcomeState = .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                deviceID: promptState.deviceID,
                endpoint: desktopCanonicalRuntimeBridge.onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop wake-enroll sample-commit request.",
                detail: error.localizedDescription
            )
        }
    }

    @MainActor
    private func submitDesktopWakeEnrollCompleteCommit(
        promptState: DesktopWakeEnrollCompleteCommitPromptState
    ) async {
        let activeEntryContextID = desktopOnboardingEntryContext?.id

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopWakeEnrollCompleteCommitRequestBuilder(
                promptState
            )
            desktopWakeEnrollCompleteCommitRuntimeOutcomeState = .dispatching(
                onboardingSessionID: ingressContext.onboardingSessionID,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.submitDesktopWakeEnrollCompleteCommit(
                ingressContext
            )
            guard desktopOnboardingEntryContext?.id == activeEntryContextID else {
                return
            }

            desktopWakeEnrollCompleteCommitRuntimeOutcomeState = outcomeState
        } catch {
            desktopWakeEnrollCompleteCommitRuntimeOutcomeState = .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                deviceID: promptState.deviceID,
                endpoint: desktopCanonicalRuntimeBridge.onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop wake-enroll complete-commit request.",
                detail: error.localizedDescription
            )
        }
    }

    @MainActor
    private func dispatchPreparedExplicitVoiceRequestIfNeeded() async {
        guard let pendingRequest = explicitVoiceController.pendingRequest else {
            return
        }

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopExplicitVoiceIngressRequestBuilder(pendingRequest)
            desktopCanonicalRuntimeOutcomeState = .dispatching(
                preparedRequestID: ingressContext.preparedRequestID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )
            desktopAuthoritativeReplyRenderState = nil
            desktopAuthoritativeReplyProvenanceRenderState = nil
            desktopAuthoritativeReplyPlaybackController.reset()
            desktopAuthoritativeReplyPlaybackState = .idle

            let outcomeState = await desktopCanonicalRuntimeBridge.dispatchPreparedExplicitVoiceRequest(ingressContext)
            guard explicitVoiceController.pendingRequest?.id == pendingRequest.id else {
                return
            }

            desktopCanonicalRuntimeOutcomeState = outcomeState
            if outcomeState.phase == .completed {
                desktopAuthoritativeReplyRenderState = DesktopAuthoritativeReplyRenderState(
                    title: "Cloud-authored authoritative reply",
                    summary: outcomeState.authoritativeResponseText == nil
                        ? "The canonical runtime completed without reply text for this bounded explicit voice turn."
                        : "Read-only canonical reply text from the completed runtime dispatch is now visible here while the shell remains explicitly non-authoritative.",
                    authoritativeResponseText: outcomeState.authoritativeResponseText
                )
                desktopAuthoritativeReplyProvenanceRenderState = DesktopAuthoritativeReplyProvenanceRenderState(
                    title: "Cloud-authored authoritative reply provenance",
                    summary: outcomeState.authoritativeResponseProvenance == nil
                        ? "The canonical runtime completed without provenance for this bounded explicit voice turn."
                        : "Read-only canonical provenance from the completed runtime dispatch is now visible here while the shell remains explicitly non-authoritative.",
                    authoritativeResponseProvenance: outcomeState.authoritativeResponseProvenance,
                    sources: outcomeState.authoritativeResponseProvenance?.sources.map {
                        DesktopAuthoritativeReplyProvenanceRenderState.Source(
                            title: $0.title,
                            url: $0.url
                        )
                    } ?? [],
                    retrievedAtLabel: formatAuthoritativeReplyRetrievedAt(
                        outcomeState.authoritativeResponseProvenance?.retrievedAt
                    ),
                    cacheStatusLabel: outcomeState.authoritativeResponseProvenance?.cacheStatus
                )
            } else {
                desktopAuthoritativeReplyRenderState = nil
                desktopAuthoritativeReplyProvenanceRenderState = nil
            }
            explicitVoiceController.clearPendingPreparedVoiceTurn()
        } catch {
            desktopCanonicalRuntimeOutcomeState = .failed(
                preparedRequestID: pendingRequest.id,
                endpoint: desktopCanonicalRuntimeBridge.voiceTurnEndpoint,
                requestID: "unavailable",
                summary: "The canonical runtime bridge could not stage the bounded explicit voice request for dispatch.",
                detail: error.localizedDescription,
                reasonCode: "desktop_runtime_bridge_failure",
                failureClass: "RetryableRuntime"
            )
            desktopAuthoritativeReplyRenderState = nil
            desktopAuthoritativeReplyProvenanceRenderState = nil
            desktopAuthoritativeReplyPlaybackController.reset()
            desktopAuthoritativeReplyPlaybackState = .idle
            explicitVoiceController.clearPendingPreparedVoiceTurn()
        }
    }

    private func clearInterruptResponseState() {
        interruptResponsePendingRequest = nil
        interruptResponseFailedRequest = nil
    }

    private func desktopOnboardingEntryListCard(
        title: String,
        items: [String],
        emptyText: String
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 8) {
                if items.isEmpty {
                    Text(emptyText)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                } else {
                    ForEach(Array(items.enumerated()), id: \.offset) { entry in
                        let item = entry.element
                        HStack(alignment: .top, spacing: 12) {
                            Text("•")
                                .foregroundStyle(.secondary)

                            Text(item)
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

    private func formatAuthoritativeReplyRetrievedAt(_ retrievedAt: UInt64?) -> String? {
        guard let retrievedAt, retrievedAt > 0 else {
            return nil
        }

        let date = Date(timeIntervalSince1970: TimeInterval(retrievedAt) / 1000)
        return Self.authoritativeReplyRetrievedAtFormatter.string(from: date)
    }

    private static let authoritativeReplyRetrievedAtFormatter: ISO8601DateFormatter = {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return formatter
    }()
}

#Preview {
    DesktopSessionShellView()
}
