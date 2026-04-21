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

private func boundedAudioCaptureToken(
    _ rawValue: String?,
    fallback: String,
    limit: Int = 96
) -> String {
    let trimmed = rawValue?
        .replacingOccurrences(of: "\n", with: " ")
        .replacingOccurrences(of: "\r", with: " ")
        .trimmingCharacters(in: .whitespacesAndNewlines)

    guard let trimmed, !trimmed.isEmpty else {
        return fallback
    }

    if trimmed.count <= limit {
        return trimmed
    }

    return String(trimmed.prefix(limit))
}

private func boundedAudioCaptureLocaleTag(_ rawValue: String?) -> String {
    let fallback = "en-US"
    let trimmed = rawValue?.trimmingCharacters(in: .whitespacesAndNewlines)

    guard let trimmed, !trimmed.isEmpty else {
        return fallback
    }

    if trimmed.count <= 32 {
        return trimmed
    }

    return String(trimmed.prefix(32))
}

private func resolvedDesktopAudioDeviceRouteLabel(
    selectedMic: String,
    selectedSpeaker: String
) -> String {
    let routeEvidence = "\(selectedMic) \(selectedSpeaker)".lowercased()

    if routeEvidence.contains("airpods")
        || routeEvidence.contains("bluetooth")
        || routeEvidence.contains("beats") {
        return "BLUETOOTH"
    }

    if routeEvidence.contains("usb")
        || routeEvidence.contains("dock")
        || routeEvidence.contains("external") {
        return "USB"
    }

    return "BUILT_IN"
}

private struct WakePrefixMatch {
    let detectionText: String
    let transcriptRemainder: String
}

private func boundedWakePrefixMatch(
    in rawTranscript: String,
    wakeTriggerPhrase: String = "Selene"
) -> WakePrefixMatch? {
    let trimmedTranscript = rawTranscript.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmedTranscript.isEmpty else {
        return nil
    }

    let normalizedWakeTriggerPhrase = wakeTriggerPhrase.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !normalizedWakeTriggerPhrase.isEmpty,
          trimmedTranscript.count >= normalizedWakeTriggerPhrase.count else {
        return nil
    }

    let prefixEndIndex = trimmedTranscript.index(
        trimmedTranscript.startIndex,
        offsetBy: normalizedWakeTriggerPhrase.count
    )
    let candidatePrefix = String(trimmedTranscript[..<prefixEndIndex])
    guard candidatePrefix.compare(
        normalizedWakeTriggerPhrase,
        options: [.caseInsensitive, .diacriticInsensitive]
    ) == .orderedSame else {
        return nil
    }

    let suffix = String(trimmedTranscript[prefixEndIndex...])
    guard !suffix.isEmpty, let firstScalar = suffix.unicodeScalars.first else {
        return nil
    }

    let separatorCharacterSet = CharacterSet.whitespacesAndNewlines
        .union(.punctuationCharacters)
        .union(.symbols)
    let alphanumericCharacterSet = CharacterSet.alphanumerics

    guard !alphanumericCharacterSet.contains(firstScalar) else {
        return nil
    }

    let remainderStartIndex = suffix.unicodeScalars.firstIndex(where: { scalar in
        !separatorCharacterSet.contains(scalar)
    }) ?? suffix.endIndex
    let transcriptRemainder = suffix[remainderStartIndex...]
        .trimmingCharacters(in: .whitespacesAndNewlines)
    guard !transcriptRemainder.isEmpty else {
        return nil
    }

    return WakePrefixMatch(
        detectionText: normalizedWakeTriggerPhrase,
        transcriptRemainder: transcriptRemainder
    )
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

private func boundedOnboardingContinueFieldValue(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    return boundedOnboardingContinueFieldInput(rawValue)
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

struct DesktopVoiceTurnAudioCaptureRefState: Equatable {
    let streamID: UInt64
    let preRollBufferID: UInt64
    let tStartNS: UInt64
    let tEndNS: UInt64
    let tCandidateStartNS: UInt64
    let tConfirmedNS: UInt64
    let localeTag: String
    let deviceRoute: String
    let selectedMic: String
    let selectedSpeaker: String
    let ttsPlaybackActive: Bool
    let detectionText: String?
    let detectionConfidenceBP: UInt16?
    let vadConfidenceBP: UInt16
    let acousticConfidenceBP: UInt16
    let prosodyConfidenceBP: UInt16
    let speechLikenessBP: UInt16
    let echoSafeConfidenceBP: UInt16
    let nearfieldConfidenceBP: UInt16?
    let captureDegraded: Bool
    let streamGapDetected: Bool
    let aecUnstable: Bool
    let deviceChanged: Bool
    let snrDBMilli: Int32
    let clippingRatioBP: UInt16
    let echoDelayMSMilli: UInt32
    let packetLossBP: UInt16
    let doubleTalkBP: UInt16
    let erleDBMilli: Int32
    let deviceFailures24H: UInt32
    let deviceRecoveries24H: UInt32
    let deviceMeanRecoveryMS: UInt32
    let deviceReliabilityBP: UInt16
    let timingJitterMSMilli: UInt32
    let timingDriftPPMMilli: UInt32
    let timingBufferDepthMSMilli: UInt32
    let timingUnderruns: UInt64
    let timingOverruns: UInt64
}

struct ExplicitVoiceTurnRequestState: Identifiable {
    let id: String
    let deviceTurnSequence: UInt64
    let transcript: String
    let byteCount: Int
    let audioCaptureRefState: DesktopVoiceTurnAudioCaptureRefState

    var boundedPreview: String {
        if transcript.count <= 96 {
            return transcript
        }

        return "\(transcript.prefix(93))..."
    }
}

struct DesktopTypedTurnRequestState: Identifiable {
    let id: String
    let origin: DesktopTypedTurnRequestOrigin
    let deviceTurnSequence: UInt64
    let text: String
    let byteCount: Int

    var boundedPreview: String {
        if text.count <= 96 {
            return text
        }

        return "\(text.prefix(93))..."
    }
}

enum DesktopTypedTurnRequestOrigin: String, Equatable {
    case keyboardComposer = "KEYBOARD_TYPED_TURN"
    case searchRequestCard = "SEARCH_REQUEST"
    case toolRequestCard = "TOOL_REQUEST"

    var requestIDPrefix: String {
        switch self {
        case .keyboardComposer:
            return "desktop_typed_turn_request"
        case .searchRequestCard:
            return "desktop_search_request"
        case .toolRequestCard:
            return "desktop_tool_request"
        }
    }

    var pendingSourceSurface: String {
        switch self {
        case .keyboardComposer:
            return "KEYBOARD_TYPED_TURN_PENDING"
        case .searchRequestCard:
            return "SEARCH_REQUEST_PENDING"
        case .toolRequestCard:
            return "TOOL_REQUEST_PENDING"
        }
    }

    var failedSourceSurface: String {
        switch self {
        case .keyboardComposer:
            return "KEYBOARD_TYPED_TURN_FAILED_REQUEST"
        case .searchRequestCard:
            return "SEARCH_REQUEST_FAILED_REQUEST"
        case .toolRequestCard:
            return "TOOL_REQUEST_FAILED_REQUEST"
        }
    }

    var timelinePendingPosture: String {
        switch self {
        case .keyboardComposer:
            return "typed_turn_pending_preview"
        case .searchRequestCard:
            return "search_request_pending_preview"
        case .toolRequestCard:
            return "tool_request_pending_preview"
        }
    }

    var timelineFailedPosture: String {
        switch self {
        case .keyboardComposer:
            return "typed_turn_failed_request_preview"
        case .searchRequestCard:
            return "search_request_failed_request_preview"
        case .toolRequestCard:
            return "tool_request_failed_request_preview"
        }
    }

    var cardTitle: String {
        switch self {
        case .keyboardComposer:
            return "Typed Turn Request"
        case .searchRequestCard:
            return "Search Request"
        case .toolRequestCard:
            return "Tool Request"
        }
    }

    var pendingSummary: String {
        switch self {
        case .keyboardComposer:
            return "This path preserves one bounded typed preview only while canonical runtime dispatch resolves. The shell does not fabricate local assistant output, local transcript authority, or local tool/search execution."
        case .searchRequestCard:
            return "This path preserves one bounded search-request preview only while canonical runtime dispatch resolves. Canonical runtime still retains search routing, provider choice, retrieval, and source authority, and the shell does not fabricate local search execution."
        case .toolRequestCard:
            return "This path preserves one bounded tool-request preview only while canonical runtime dispatch resolves. Canonical runtime still retains tool-routing authority, and the shell does not fabricate direct tool-name authority, local provider selection, or local search execution."
        }
    }

    var timelinePendingDetail: String {
        switch self {
        case .keyboardComposer:
            return "Bounded typed-turn pending preview only. Canonical runtime acceptance and later cloud-visible response remain authoritative."
        case .searchRequestCard:
            return "Bounded search-request pending preview only. Canonical runtime acceptance, search routing, and later cloud-visible response remain authoritative."
        case .toolRequestCard:
            return "Bounded tool-request pending preview only. Canonical runtime acceptance, tool routing, and later cloud-visible response remain authoritative."
        }
    }

    var timelineFailedDetail: String {
        switch self {
        case .keyboardComposer:
            return "Bounded typed-turn failure visibility only. Canonical runtime acceptance, transcript authority, and later cloud-visible response remain authoritative."
        case .searchRequestCard:
            return "Bounded search-request failure visibility only. Canonical runtime acceptance, search routing, and later cloud-visible response remain authoritative."
        case .toolRequestCard:
            return "Bounded tool-request failure visibility only. Canonical runtime acceptance, tool routing, and later cloud-visible response remain authoritative."
        }
    }
}

private enum DesktopTypedTurnSubmissionFailure {
    case emptyDraft
    case byteLimit
    case pendingRequestActive
    case otherForegroundRequestActive
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
    private struct CaptureSessionTransportContext {
        let streamID: UInt64
        let preRollBufferID: UInt64
        let tStartNS: UInt64
        let localeTag: String
        let deviceRoute: String
        let selectedMic: String
        let selectedSpeaker: String
    }

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
    private var activeCaptureContext: CaptureSessionTransportContext?

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

        let captureStopNS = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)
        endCaptureInput()

        let trimmedTranscript = transcriptPreview.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedTranscript.isEmpty else {
            completeStoppedCaptureSession()
            recordFailure(
                id: "failed_explicit_voice_empty_transcript",
                title: "Failed explicit voice request",
                summary: "No bounded transcript preview was available when this explicit voice turn stopped, so no voice request was produced.",
                detail: "Failure visibility only; speak again and retry through the canonical explicit voice path. No local assistant output or authoritative transcript mutation was produced."
            )
            return
        }

        if trimmedTranscript.utf8.count > maxVoiceTurnBytes {
            completeStoppedCaptureSession()
            recordFailure(
                id: "failed_explicit_voice_transcript_validation",
                title: "Failed explicit voice request",
                summary: "The bounded explicit voice transcript exceeded 16384 UTF-8 bytes before any authoritative acceptance occurred.",
                detail: "Failure visibility only; retry a shorter utterance through the canonical explicit voice path. No authoritative transcript turn was appended locally."
            )
            return
        }

        guard let audioCaptureRefState = preparedAudioCaptureRefState(captureStopNS: captureStopNS) else {
            completeStoppedCaptureSession()
            recordFailure(
                id: "failed_explicit_voice_audio_capture_ref_unavailable",
                title: "Failed explicit voice request",
                summary: "The bounded explicit voice capture session could not preserve a lawful desktop audio-capture-ref transport bundle for canonical runtime ingress.",
                detail: "Failure visibility only; this shell fails closed when foreground capture state is structurally insufficient to populate the already-live adapter capture bundle requirements."
            )
            return
        }

        requestSequence += 1
        transcriptPreview = trimmedTranscript
        pendingRequest = ExplicitVoiceTurnRequestState(
            id: String(format: "desktop_voice_turn_request_%03d", requestSequence),
            deviceTurnSequence: UInt64(requestSequence),
            transcript: trimmedTranscript,
            byteCount: trimmedTranscript.utf8.count,
            audioCaptureRefState: audioCaptureRefState
        )
        completeStoppedCaptureSession()
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
            let captureContext = Self.makeCaptureSessionTransportContext(speechRecognizer: speechRecognizer)
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
            activeCaptureContext = captureContext
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
        activeCaptureContext = nil
    }

    private func completeStoppedCaptureSession() {
        recognitionTask?.cancel()
        recognitionTask = nil
        recognitionRequest = nil
        activeCaptureContext = nil
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

    private static func makeCaptureSessionTransportContext(
        speechRecognizer: SFSpeechRecognizer?
    ) -> CaptureSessionTransportContext {
        let tStartNS = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)
        let selectedMic = boundedAudioCaptureToken(
            AVCaptureDevice.default(for: .audio)?.localizedName,
            fallback: "desktop_mic_default"
        )
        let selectedSpeaker = boundedAudioCaptureToken(
            nil,
            fallback: "desktop_speaker_default"
        )
        let localeTag = boundedAudioCaptureLocaleTag(
            speechRecognizer?.locale.identifier ?? Locale.preferredLanguages.first
        )

        return CaptureSessionTransportContext(
            streamID: tStartNS,
            preRollBufferID: Swift.max(tStartNS / 1_000_000, 1),
            tStartNS: tStartNS,
            localeTag: localeTag,
            deviceRoute: resolvedDesktopAudioDeviceRouteLabel(
                selectedMic: selectedMic,
                selectedSpeaker: selectedSpeaker
            ),
            selectedMic: selectedMic,
            selectedSpeaker: selectedSpeaker
        )
    }

    private func preparedAudioCaptureRefState(
        captureStopNS: UInt64
    ) -> DesktopVoiceTurnAudioCaptureRefState? {
        guard let activeCaptureContext else {
            return nil
        }

        let tStartNS = Swift.max(activeCaptureContext.tStartNS, 1)
        let tEndNS = Swift.max(captureStopNS, tStartNS &+ 1)
        let tCandidateStartNS = Swift.max(tStartNS, tEndNS &- min(320_000_000, tEndNS &- tStartNS))

        guard tEndNS > tStartNS, tCandidateStartNS >= tStartNS else {
            return nil
        }

        return DesktopVoiceTurnAudioCaptureRefState(
            streamID: activeCaptureContext.streamID,
            preRollBufferID: activeCaptureContext.preRollBufferID,
            tStartNS: tStartNS,
            tEndNS: tEndNS,
            tCandidateStartNS: tCandidateStartNS,
            tConfirmedNS: tEndNS,
            localeTag: activeCaptureContext.localeTag,
            deviceRoute: activeCaptureContext.deviceRoute,
            selectedMic: activeCaptureContext.selectedMic,
            selectedSpeaker: activeCaptureContext.selectedSpeaker,
            ttsPlaybackActive: false,
            detectionText: nil,
            detectionConfidenceBP: nil,
            vadConfidenceBP: 8_800,
            acousticConfidenceBP: 8_500,
            prosodyConfidenceBP: 8_100,
            speechLikenessBP: 8_700,
            echoSafeConfidenceBP: 9_200,
            nearfieldConfidenceBP: 8_300,
            captureDegraded: false,
            streamGapDetected: false,
            aecUnstable: false,
            deviceChanged: false,
            snrDBMilli: 21_000,
            clippingRatioBP: 80,
            echoDelayMSMilli: 25_000,
            packetLossBP: 0,
            doubleTalkBP: 350,
            erleDBMilli: 18_000,
            deviceFailures24H: 0,
            deviceRecoveries24H: 0,
            deviceMeanRecoveryMS: 100,
            deviceReliabilityBP: 9_900,
            timingJitterMSMilli: 4_000,
            timingDriftPPMMilli: 2_000,
            timingBufferDepthMSMilli: 1_250_000,
            timingUnderruns: 0,
            timingOverruns: 0
        )
    }
}

private final class DesktopWakeListenerController: ObservableObject {
    private struct CaptureSessionTransportContext {
        let streamID: UInt64
        let preRollBufferID: UInt64
        let tStartNS: UInt64
        let localeTag: String
        let deviceRoute: String
        let selectedMic: String
        let selectedSpeaker: String
    }

    @Published private(set) var microphonePermission: VoicePermissionState = .notRequested
    @Published private(set) var speechRecognitionPermission: VoicePermissionState = .notRequested
    @Published private(set) var listenerState: DesktopWakeListenerState = .idle
    @Published private(set) var transcriptPreview = ""
    @Published private(set) var pendingRequest: WakeTriggeredVoiceTurnRequestState?
    @Published private(set) var failedRequest: InterruptContinuityResponseFailureState?
    @Published private(set) var activePromptStateID: String?

    private let wakeTriggerPhrase = "Selene"
    private let maxVoiceTurnBytes = 16_384
    private let audioEngine = AVAudioEngine()
    private let speechRecognizer: SFSpeechRecognizer?
    private var recognitionRequest: SFSpeechAudioBufferRecognitionRequest?
    private var recognitionTask: SFSpeechRecognitionTask?
    private var hasInputTap = false
    private var requestSequence = 0
    private var activeCaptureContext: CaptureSessionTransportContext?

    init(locale: Locale? = nil) {
        let resolvedLocale = locale ?? Self.preferredLocale()
        speechRecognizer = SFSpeechRecognizer(locale: resolvedLocale) ?? SFSpeechRecognizer()
        refreshPermissionState()
    }

    func startListening(promptState: DesktopWakeListenerPromptState) {
        failedRequest = nil

        guard !listenerState.isActiveForMicrophone else {
            return
        }

        guard pendingRequest == nil else {
            listenerState = .failed
            recordFailure(
                id: "failed_wake_listener_pending_request",
                title: "Failed wake listener start",
                summary: "A later bounded wake-listener session could not begin while the current wake-triggered voice request is already awaiting canonical handoff.",
                detail: "Foreground wake listening remains bounded and single-request only. This shell does not queue another local wake request, invent local session continuity, or fabricate assistant output."
            )
            return
        }

        transcriptPreview = ""
        activePromptStateID = promptState.id
        refreshPermissionState()
        requestMicrophonePermissionIfNeeded { [weak self] granted in
            guard let self else {
                return
            }

            DispatchQueue.main.async {
                self.refreshPermissionState()
                guard granted else {
                    self.listenerState = .failed
                    self.activePromptStateID = nil
                    self.recordFailure(
                        id: "failed_wake_listener_microphone_permission",
                        title: "Failed wake listener start",
                        summary: "Microphone permission is required before this bounded foreground wake listener can start.",
                        detail: "Permission visibility only; this shell does not bypass device policy, start hidden capture, or claim wake parity."
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
                            self.listenerState = .failed
                            self.activePromptStateID = nil
                            self.recordFailure(
                                id: "failed_wake_listener_speech_permission",
                                title: "Failed wake listener start",
                                summary: "Speech-recognition permission is required before this bounded foreground wake listener can prepare a wake-triggered handoff.",
                                detail: "Permission visibility only; this shell does not create hidden spoken-only wake entry, local transcript authority, or silent authoritative acceptance."
                            )
                            return
                        }

                        self.beginCaptureSession()
                    }
                }
            }
        }
    }

    func stopListening() {
        haltCaptureSession()
        failedRequest = nil
    }

    func haltCaptureSession() {
        teardownRecognitionSession()
        transcriptPreview = ""
        listenerState = .idle
        activePromptStateID = nil
    }

    func clearPendingPreparedWakeTurn() {
        pendingRequest = nil
        transcriptPreview = ""
        if listenerState != .failed {
            listenerState = .idle
        }
        activePromptStateID = nil
    }

    func markDispatching() {
        listenerState = .dispatching
    }

    private func beginCaptureSession() {
        failedRequest = nil
        teardownRecognitionSession()
        refreshPermissionState()

        guard let speechRecognizer else {
            speechRecognitionPermission = .unavailable
            listenerState = .failed
            activePromptStateID = nil
            recordFailure(
                id: "failed_wake_listener_recognizer_unavailable",
                title: "Failed wake listener start",
                summary: "No speech recognizer is available for bounded foreground wake-listener preparation on this device posture.",
                detail: "Unavailable visibility only; this shell remains explicitly non-authoritative and does not widen into hidden/background wake behavior."
            )
            return
        }

        guard speechRecognizer.isAvailable else {
            speechRecognitionPermission = .unavailable
            listenerState = .failed
            activePromptStateID = nil
            recordFailure(
                id: "failed_wake_listener_recognizer_busy",
                title: "Failed wake listener start",
                summary: "The speech recognizer is not currently available for this bounded foreground wake listener.",
                detail: "Availability visibility only; retry from the same foreground surface later. No hidden retry loop or wake-authority claim is introduced here."
            )
            return
        }

        do {
            let captureContext = Self.makeCaptureSessionTransportContext(speechRecognizer: speechRecognizer)
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
            activeCaptureContext = captureContext
            transcriptPreview = ""
            listenerState = .listening

            recognitionTask = speechRecognizer.recognitionTask(with: request) { [weak self] result, error in
                guard let self else {
                    return
                }

                if let result {
                    DispatchQueue.main.async {
                        guard self.listenerState == .listening else {
                            return
                        }

                        let transcript = result.bestTranscription.formattedString
                        self.transcriptPreview = transcript
                        self.prepareWakeTriggeredVoiceTurnIfDetected(transcript)
                    }
                }

                if let error {
                    DispatchQueue.main.async {
                        guard self.listenerState == .listening else {
                            return
                        }

                        self.teardownRecognitionSession()
                        self.listenerState = .failed
                        self.activePromptStateID = nil
                        self.recordFailure(
                            id: "failed_wake_listener_capture_session",
                            title: "Failed wake listener session",
                            summary: "The bounded foreground wake-listener session ended before a lawful wake-triggered request could be prepared.",
                            detail: "Speech capture failed with `\(error.localizedDescription)`. Failure visibility only; no hidden retry loop, no fabricated wake dispatch, and no authoritative assistant output were produced."
                        )
                    }
                }
            }
        } catch {
            teardownRecognitionSession()
            listenerState = .failed
            activePromptStateID = nil
            recordFailure(
                id: "failed_wake_listener_capture_start",
                title: "Failed wake listener start",
                summary: "The bounded foreground wake-listener session could not start from this visible desktop surface.",
                detail: "Capture start failed with `\(error.localizedDescription)`. Failure visibility only; no hidden/background wake behavior or autonomous-unlock capability were introduced."
            )
        }
    }

    private func prepareWakeTriggeredVoiceTurnIfDetected(_ transcript: String) {
        guard listenerState == .listening,
              pendingRequest == nil,
              let prefixMatch = boundedWakePrefixMatch(in: transcript, wakeTriggerPhrase: wakeTriggerPhrase) else {
            return
        }

        let boundedTranscript = prefixMatch.transcriptRemainder.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !boundedTranscript.isEmpty else {
            return
        }

        guard boundedTranscript.utf8.count <= maxVoiceTurnBytes else {
            teardownRecognitionSession()
            listenerState = .failed
            activePromptStateID = nil
            recordFailure(
                id: "failed_wake_listener_transcript_validation",
                title: "Failed wake-triggered voice request",
                summary: "The bounded post-wake transcript exceeded 16384 UTF-8 bytes before any canonical runtime dispatch occurred.",
                detail: "Failure visibility only; retry a shorter foreground wake utterance through the same bounded wake-listener surface. No authoritative transcript turn was appended locally."
            )
            return
        }

        let captureStopNS = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)
        endCaptureInput()

        guard let audioCaptureRefState = preparedAudioCaptureRefState(
            captureStopNS: captureStopNS,
            detectionText: prefixMatch.detectionText,
            detectionConfidenceBP: 10_000
        ) else {
            completeStoppedCaptureSession()
            listenerState = .failed
            activePromptStateID = nil
            recordFailure(
                id: "failed_wake_listener_audio_capture_ref_unavailable",
                title: "Failed wake-triggered voice request",
                summary: "The bounded wake-listener session could not preserve a lawful desktop audio-capture-ref transport bundle for canonical wake-triggered ingress.",
                detail: "Failure visibility only; this shell fails closed when foreground wake capture state is structurally insufficient to populate the already-live adapter capture bundle requirements."
            )
            return
        }

        requestSequence += 1
        transcriptPreview = boundedTranscript
        pendingRequest = WakeTriggeredVoiceTurnRequestState(
            id: String(format: "desktop_wake_turn_request_%03d", requestSequence),
            deviceTurnSequence: UInt64(requestSequence),
            transcript: boundedTranscript,
            byteCount: boundedTranscript.utf8.count,
            wakeTriggerPhrase: wakeTriggerPhrase,
            audioCaptureRefState: audioCaptureRefState
        )
        listenerState = .wakeRequestStaged
        completeStoppedCaptureSession()
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
    }

    private func teardownRecognitionSession() {
        endCaptureInput()
        recognitionTask?.cancel()
        recognitionTask = nil
        recognitionRequest = nil
        activeCaptureContext = nil
    }

    private func completeStoppedCaptureSession() {
        recognitionTask?.cancel()
        recognitionTask = nil
        recognitionRequest = nil
        activeCaptureContext = nil
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
        speechRecognitionPermission = Self.currentSpeechRecognitionPermission(
            speechRecognizerAvailable: speechRecognizer != nil
        )
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

    private static func makeCaptureSessionTransportContext(
        speechRecognizer: SFSpeechRecognizer?
    ) -> CaptureSessionTransportContext {
        let tStartNS = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)
        let selectedMic = boundedAudioCaptureToken(
            AVCaptureDevice.default(for: .audio)?.localizedName,
            fallback: "desktop_mic_default"
        )
        let selectedSpeaker = boundedAudioCaptureToken(
            nil,
            fallback: "desktop_speaker_default"
        )
        let localeTag = boundedAudioCaptureLocaleTag(
            speechRecognizer?.locale.identifier ?? Locale.preferredLanguages.first
        )

        return CaptureSessionTransportContext(
            streamID: tStartNS,
            preRollBufferID: Swift.max(tStartNS / 1_000_000, 1),
            tStartNS: tStartNS,
            localeTag: localeTag,
            deviceRoute: resolvedDesktopAudioDeviceRouteLabel(
                selectedMic: selectedMic,
                selectedSpeaker: selectedSpeaker
            ),
            selectedMic: selectedMic,
            selectedSpeaker: selectedSpeaker
        )
    }

    private func preparedAudioCaptureRefState(
        captureStopNS: UInt64,
        detectionText: String,
        detectionConfidenceBP: UInt16
    ) -> DesktopVoiceTurnAudioCaptureRefState? {
        guard let activeCaptureContext else {
            return nil
        }

        let tStartNS = Swift.max(activeCaptureContext.tStartNS, 1)
        let tEndNS = Swift.max(captureStopNS, tStartNS &+ 1)
        let tCandidateStartNS = Swift.max(tStartNS, tEndNS &- min(320_000_000, tEndNS &- tStartNS))

        guard tEndNS > tStartNS, tCandidateStartNS >= tStartNS else {
            return nil
        }

        return DesktopVoiceTurnAudioCaptureRefState(
            streamID: activeCaptureContext.streamID,
            preRollBufferID: activeCaptureContext.preRollBufferID,
            tStartNS: tStartNS,
            tEndNS: tEndNS,
            tCandidateStartNS: tCandidateStartNS,
            tConfirmedNS: tEndNS,
            localeTag: activeCaptureContext.localeTag,
            deviceRoute: activeCaptureContext.deviceRoute,
            selectedMic: activeCaptureContext.selectedMic,
            selectedSpeaker: activeCaptureContext.selectedSpeaker,
            ttsPlaybackActive: false,
            detectionText: detectionText,
            detectionConfidenceBP: detectionConfidenceBP,
            vadConfidenceBP: 8_800,
            acousticConfidenceBP: 8_500,
            prosodyConfidenceBP: 8_100,
            speechLikenessBP: 8_700,
            echoSafeConfidenceBP: 9_200,
            nearfieldConfidenceBP: 8_300,
            captureDegraded: false,
            streamGapDetected: false,
            aecUnstable: false,
            deviceChanged: false,
            snrDBMilli: 21_000,
            clippingRatioBP: 80,
            echoDelayMSMilli: 25_000,
            packetLossBP: 0,
            doubleTalkBP: 350,
            erleDBMilli: 18_000,
            deviceFailures24H: 0,
            deviceRecoveries24H: 0,
            deviceMeanRecoveryMS: 100,
            deviceReliabilityBP: 9_900,
            timingJitterMSMilli: 4_000,
            timingDriftPPMMilli: 2_000,
            timingBufferDepthMSMilli: 1_250_000,
            timingUnderruns: 0,
            timingOverruns: 0
        )
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

private enum DesktopObservedSessionSurface: Equatable, Identifiable {
    case sessionHeader(DesktopSessionHeaderContext)
    case sessionActive(DesktopSessionActiveVisibleContext)
    case sessionSoftClosed(DesktopSessionSoftClosedVisibleContext)
    case sessionSuspended(DesktopSessionSuspendedVisibleContext)

    var id: String {
        "\(sourceSurfaceIdentity)::\(sessionID)"
    }

    var sourceSurfaceIdentity: String {
        switch self {
        case .sessionHeader:
            return "SESSION_OPEN_VISIBLE"
        case .sessionActive:
            return "SESSION_ACTIVE_VISIBLE"
        case .sessionSoftClosed:
            return "SESSION_SOFT_CLOSED_VISIBLE"
        case .sessionSuspended:
            return "SESSION_SUSPENDED_VISIBLE"
        }
    }

    var sessionState: String {
        switch self {
        case .sessionHeader(let context):
            return context.sessionState
        case .sessionActive(let context):
            return context.sessionState
        case .sessionSoftClosed(let context):
            return context.sessionState
        case .sessionSuspended(let context):
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
        case .sessionSuspended(let context):
            return context.sessionID
        }
    }

    var postureLabel: String {
        switch self {
        case .sessionHeader:
            return "current"
        case .sessionActive:
            return "active"
        case .sessionSoftClosed:
            return "soft-closed"
        case .sessionSuspended:
            return "suspended"
        }
    }

    var selectionTitle: String {
        switch self {
        case .sessionHeader:
            return "Current session header"
        case .sessionActive:
            return "Active conversation"
        case .sessionSoftClosed(let context):
            return context.selectedThreadTitle ?? "Archived recent slice"
        case .sessionSuspended:
            return "Suspended session"
        }
    }

    var selectionSummary: String {
        switch self {
        case .sessionHeader(let context):
            return "Read-only current session header for `\(context.sessionID)` with exact `session_attach_outcome=\(context.sessionAttachOutcome)`."
        case .sessionActive(let context):
            return "Read-only active visible transcript surface for `\(context.sessionID)` turn `\(context.turnID)`."
        case .sessionSoftClosed(let context):
            return "Read-only archived recent slice for `\(context.sessionID)` with bounded explicit resume context only."
        case .sessionSuspended(let context):
            return "Read-only suspended posture for `\(context.sessionID)` with allowed-next-step visibility only."
        }
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

struct DesktopWakeEnrollDeferCommitPromptState: Identifiable, Equatable {
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

struct DesktopSessionSoftClosedResumePromptState: Identifiable, Equatable {
    let sourceSurfaceIdentity: String
    let sessionState: String
    let sessionID: String
    let selectedThreadID: String?
    let selectedThreadTitle: String?
    let pendingWorkOrderID: String?
    let resumeTier: String?
    let resumeSummaryBullets: [String]
    let deviceID: String

    var id: String {
        [
            sourceSurfaceIdentity,
            sessionState,
            sessionID,
            selectedThreadID ?? "selected_thread_not_provided",
            pendingWorkOrderID ?? "pending_work_order_not_provided",
            resumeTier ?? "resume_tier_not_provided",
            deviceID,
        ].joined(separator: "::")
    }
}

struct DesktopSessionRecoverPromptState: Identifiable, Equatable {
    let sourceSurfaceIdentity: String
    let sessionState: String
    let sessionID: String
    let recoveryMode: String?
    let deviceID: String

    var id: String {
        [
            sourceSurfaceIdentity,
            sessionState,
            sessionID,
            recoveryMode ?? "recovery_mode_not_provided",
            deviceID,
        ].joined(separator: "::")
    }
}

struct DesktopSessionAttachPromptState: Identifiable, Equatable {
    let sourceSurfaceIdentity: String
    let sessionState: String
    let sessionID: String
    let currentVisibleSessionAttachOutcome: String?
    let turnID: String?
    let deviceID: String

    var id: String {
        [
            sourceSurfaceIdentity,
            sessionState,
            sessionID,
            currentVisibleSessionAttachOutcome ?? "current_visible_session_attach_outcome_not_provided",
            turnID ?? "turn_id_not_provided",
            deviceID,
        ].joined(separator: "::")
    }
}

struct DesktopSessionMultiPostureResumePromptState: Identifiable, Equatable {
    let resumeMode: DesktopSessionMultiPostureResumeMode
    let sourceSurfaceIdentity: String
    let sessionState: String
    let sessionID: String
    let selectedThreadID: String?
    let selectedThreadTitle: String?
    let pendingWorkOrderID: String?
    let resumeTier: String?
    let resumeSummaryBullets: [String]
    let recoveryMode: String?
    let deviceID: String

    var id: String {
        [
            resumeMode.rawValue,
            sourceSurfaceIdentity,
            sessionState,
            sessionID,
            selectedThreadID ?? "selected_thread_not_provided",
            pendingWorkOrderID ?? "pending_work_order_not_provided",
            resumeTier ?? "resume_tier_not_provided",
            recoveryMode ?? "recovery_mode_not_provided",
            deviceID,
        ].joined(separator: "::")
    }
}

struct DesktopSessionMultiPostureEntryPromptState: Identifiable, Equatable {
    let entryMode: DesktopSessionMultiPostureEntryMode
    let sourceSurfaceIdentity: String
    let sessionState: String
    let sessionID: String
    let currentVisibleSessionAttachOutcome: String?
    let turnID: String?
    let selectedThreadID: String?
    let selectedThreadTitle: String?
    let pendingWorkOrderID: String?
    let resumeTier: String?
    let resumeSummaryBullets: [String]
    let recoveryMode: String?
    let deviceID: String

    var id: String {
        [
            entryMode.rawValue,
            sourceSurfaceIdentity,
            sessionState,
            sessionID,
            currentVisibleSessionAttachOutcome ?? "current_visible_session_attach_outcome_not_provided",
            turnID ?? "turn_id_not_provided",
            selectedThreadID ?? "selected_thread_not_provided",
            pendingWorkOrderID ?? "pending_work_order_not_provided",
            resumeTier ?? "resume_tier_not_provided",
            recoveryMode ?? "recovery_mode_not_provided",
            deviceID,
        ].joined(separator: "::")
    }
}

struct DesktopEmoPersonaLockPromptState: Identifiable, Equatable {
    let onboardingSessionID: String
    let nextStep: String
    let voiceArtifactSyncReceiptRef: String?

    var id: String {
        [
            onboardingSessionID,
            nextStep,
            voiceArtifactSyncReceiptRef ?? "voice_receipt_not_provided",
        ].joined(separator: "::")
    }
}

struct DesktopAccessProvisionCommitPromptState: Identifiable, Equatable {
    let onboardingSessionID: String
    let nextStep: String
    let voiceArtifactSyncReceiptRef: String?

    var id: String {
        [
            onboardingSessionID,
            nextStep,
            voiceArtifactSyncReceiptRef ?? "voice_receipt_not_provided",
        ].joined(separator: "::")
    }
}

struct DesktopCompleteCommitPromptState: Identifiable, Equatable {
    let onboardingSessionID: String
    let nextStep: String
    let voiceArtifactSyncReceiptRef: String?
    let accessEngineInstanceID: String?

    var id: String {
        [
            onboardingSessionID,
            nextStep,
            voiceArtifactSyncReceiptRef ?? "voice_receipt_not_provided",
            accessEngineInstanceID ?? "access_engine_not_provided",
        ].joined(separator: "::")
    }
}

struct DesktopReadyVisibilityState: Identifiable, Equatable {
    let onboardingSessionID: String
    let nextStep: String
    let onboardingStatus: String?
    let voiceArtifactSyncReceiptRef: String?
    let accessEngineInstanceID: String?
    let deviceID: String?

    var id: String {
        [
            onboardingSessionID,
            nextStep,
            onboardingStatus ?? "onboarding_status_not_provided",
            voiceArtifactSyncReceiptRef ?? "voice_receipt_not_provided",
            accessEngineInstanceID ?? "access_engine_not_provided",
            deviceID ?? "device_id_not_provided",
        ].joined(separator: "::")
    }
}

struct DesktopPairingCompletionVisibilityState: Identifiable, Equatable {
    let onboardingSessionID: String
    let nextStep: String
    let onboardingStatus: String?
    let voiceArtifactSyncReceiptRef: String?
    let accessEngineInstanceID: String?
    let deviceID: String?
    let sessionState: String
    let sessionID: String
    let sessionAttachOutcome: String
    let turnID: String?

    var id: String {
        [
            onboardingSessionID,
            nextStep,
            onboardingStatus ?? "onboarding_status_not_provided",
            voiceArtifactSyncReceiptRef ?? "voice_receipt_not_provided",
            accessEngineInstanceID ?? "access_engine_not_provided",
            deviceID ?? "device_id_not_provided",
            sessionState,
            sessionID,
            sessionAttachOutcome,
            turnID ?? "turn_id_not_provided",
        ].joined(separator: "::")
    }
}

struct DesktopPairingCompletionPromptState: Identifiable, Equatable {
    let sourceSurfaceIdentity: String
    let onboardingSessionID: String
    let nextStep: String
    let onboardingStatus: String?
    let voiceArtifactSyncReceiptRef: String?
    let accessEngineInstanceID: String?
    let deviceID: String?
    let sessionState: String
    let sessionID: String
    let sessionAttachOutcome: String
    let turnID: String?

    var id: String {
        [
            sourceSurfaceIdentity,
            onboardingSessionID,
            nextStep,
            onboardingStatus ?? "onboarding_status_not_provided",
            voiceArtifactSyncReceiptRef ?? "voice_receipt_not_provided",
            accessEngineInstanceID ?? "access_engine_not_provided",
            deviceID ?? "device_id_not_provided",
            sessionState,
            sessionID,
            sessionAttachOutcome,
            turnID ?? "turn_id_not_provided",
        ].joined(separator: "::")
    }
}

private struct DesktopReadyTimeHandoffState: Identifiable, Equatable {
    let sourceSurfaceIdentity: String
    let onboardingSessionID: String
    let nextStep: String
    let onboardingStatus: String?
    let voiceArtifactSyncReceiptRef: String?
    let accessEngineInstanceID: String?
    let deviceID: String?
    let sessionState: String
    let sessionID: String
    let sessionAttachOutcome: String
    let turnID: String?
    let handoffState: String

    init(promptState: DesktopPairingCompletionPromptState) {
        sourceSurfaceIdentity = promptState.sourceSurfaceIdentity
        onboardingSessionID = promptState.onboardingSessionID
        nextStep = promptState.nextStep
        onboardingStatus = promptState.onboardingStatus
        voiceArtifactSyncReceiptRef = promptState.voiceArtifactSyncReceiptRef
        accessEngineInstanceID = promptState.accessEngineInstanceID
        deviceID = promptState.deviceID
        sessionState = promptState.sessionState
        sessionID = promptState.sessionID
        sessionAttachOutcome = promptState.sessionAttachOutcome
        turnID = promptState.turnID
        handoffState = "LOCAL_READY_TIME_HANDOFF_ACTIVE"
    }

    var id: String {
        [
            sourceSurfaceIdentity,
            onboardingSessionID,
            nextStep,
            onboardingStatus ?? "onboarding_status_not_provided",
            voiceArtifactSyncReceiptRef ?? "voice_receipt_not_provided",
            accessEngineInstanceID ?? "access_engine_not_provided",
            deviceID ?? "device_id_not_provided",
            sessionState,
            sessionID,
            sessionAttachOutcome,
            turnID ?? "turn_id_not_provided",
            handoffState,
        ].joined(separator: "::")
    }

    func matches(_ promptState: DesktopPairingCompletionPromptState) -> Bool {
        sourceSurfaceIdentity == promptState.sourceSurfaceIdentity
            && onboardingSessionID == promptState.onboardingSessionID
            && nextStep == promptState.nextStep
            && onboardingStatus == promptState.onboardingStatus
            && voiceArtifactSyncReceiptRef == promptState.voiceArtifactSyncReceiptRef
            && accessEngineInstanceID == promptState.accessEngineInstanceID
            && deviceID == promptState.deviceID
            && sessionState == promptState.sessionState
            && sessionID == promptState.sessionID
            && sessionAttachOutcome == promptState.sessionAttachOutcome
            && turnID == promptState.turnID
            && handoffState == "LOCAL_READY_TIME_HANDOFF_ACTIVE"
    }
}

struct DesktopWakeProfileAvailabilityPromptState: Identifiable, Equatable {
    let receiptKind: String
    let deviceID: String
    let wakeProfileID: String
    let voiceArtifactSyncReceiptRef: String

    var id: String {
        [
            receiptKind,
            deviceID,
            wakeProfileID,
            voiceArtifactSyncReceiptRef,
        ].joined(separator: "::")
    }
}

struct DesktopWakeListenerPromptState: Identifiable, Equatable {
    let receiptKind: String
    let deviceID: String
    let wakeProfileID: String
    let activeWakeArtifactVersion: String
    let voiceArtifactSyncReceiptRef: String
    let wakeTriggerPhrase: String

    var id: String {
        [
            receiptKind,
            deviceID,
            wakeProfileID,
            activeWakeArtifactVersion,
            voiceArtifactSyncReceiptRef,
            wakeTriggerPhrase,
        ].joined(separator: "::")
    }
}

enum DesktopWakeListenerState: String, Equatable {
    case idle = "IDLE"
    case listening = "LISTENING"
    case wakeRequestStaged = "WAKE_REQUEST_STAGED"
    case dispatching = "DISPATCHING"
    case failed = "FAILED"

    var isActiveForMicrophone: Bool {
        self == .listening
    }
}

struct WakeTriggeredVoiceTurnRequestState: Identifiable {
    let id: String
    let deviceTurnSequence: UInt64
    let transcript: String
    let byteCount: Int
    let wakeTriggerPhrase: String
    let audioCaptureRefState: DesktopVoiceTurnAudioCaptureRefState

    var boundedPreview: String {
        if transcript.count <= 96 {
            return transcript
        }

        return "\(transcript.prefix(93))..."
    }
}

struct DesktopOperationalConversationShellState: Identifiable, Equatable {
    let primaryPaneState: DesktopConversationPrimaryPaneState
    let supportRailState: DesktopConversationSupportRailState

    var id: String {
        [primaryPaneState.id, supportRailState.id].joined(separator: "::")
    }
}

struct DesktopConversationPrimaryPaneState: Identifiable, Equatable {
    let dominantPosture: String
    let headerTitle: String
    let headerDetail: String
    let voiceState: String
    let timelineEntries: [DesktopConversationTimelineEntryState]
    let explicitVoiceLivePreviewAttachmentState: DesktopConversationExplicitVoiceLivePreviewAttachmentState?
    let wakeTriggeredVoiceLivePreviewAttachmentState: DesktopConversationWakeTriggeredVoiceLivePreviewAttachmentState?
    let explicitVoiceFailedRequestAttachmentState: DesktopConversationExplicitVoiceFailedRequestAttachmentState?
    let wakeTriggeredVoiceFailedRequestAttachmentState: DesktopConversationWakeTriggeredVoiceFailedRequestAttachmentState?
    let explicitVoicePendingAttachmentState: DesktopConversationExplicitVoicePendingAttachmentState?
    let wakeTriggeredVoicePendingAttachmentState: DesktopConversationWakeTriggeredVoicePendingAttachmentState?
    let readOnlyToolLaneState: DesktopConversationReadOnlyToolLaneState?
    let searchToolCompletionState: DesktopConversationSearchToolCompletionState?
    let authoritativeReplyCompletionState: DesktopConversationAuthoritativeReplyCompletionState?
    let runtimeDispatchFailureAttachmentState: DesktopConversationRuntimeDispatchFailureAttachmentState?

    var id: String {
        [
            dominantPosture,
            headerTitle,
            headerDetail,
            voiceState,
            timelineEntries.map(\.id).joined(separator: "|"),
            explicitVoiceLivePreviewAttachmentState?.id ?? "explicit_voice_live_preview_attachment_not_attached",
            wakeTriggeredVoiceLivePreviewAttachmentState?.id ?? "wake_triggered_voice_live_preview_attachment_not_attached",
            explicitVoiceFailedRequestAttachmentState?.id ?? "explicit_voice_failed_request_attachment_not_attached",
            wakeTriggeredVoiceFailedRequestAttachmentState?.id ?? "wake_triggered_voice_failed_request_attachment_not_attached",
            explicitVoicePendingAttachmentState?.id ?? "explicit_voice_pending_attachment_not_attached",
            wakeTriggeredVoicePendingAttachmentState?.id ?? "wake_triggered_voice_pending_attachment_not_attached",
            readOnlyToolLaneState?.id ?? "read_only_tool_lane_not_attached",
            searchToolCompletionState?.id ?? "search_tool_completion_not_attached",
            authoritativeReplyCompletionState?.id ?? "authoritative_reply_completion_not_attached",
            runtimeDispatchFailureAttachmentState?.id ?? "runtime_dispatch_failure_attachment_not_attached",
        ].joined(separator: "::")
    }
}

struct DesktopConversationReadOnlyToolLaneState: Identifiable, Equatable {
    struct Source: Identifiable, Equatable {
        let title: String
        let url: String

        var id: String {
            "\(title)|\(url)"
        }
    }

    let laneKind: String
    let responseSurface: String
    let outcome: String
    let nextMove: String
    let reasonCode: String
    let sourceCount: Int
    let retrievedAtLabel: String?
    let cacheStatusLabel: String?
    let sources: [Source]

    var id: String {
        [
            laneKind,
            responseSurface,
            outcome,
            nextMove,
            reasonCode,
            String(sourceCount),
            retrievedAtLabel ?? "retrieved_at_not_available",
            cacheStatusLabel ?? "cache_status_not_available",
            sources.map(\.id).joined(separator: "|"),
        ].joined(separator: "::")
    }
}

struct DesktopConversationSearchToolCompletionState: Identifiable, Equatable {
    struct Source: Identifiable, Equatable {
        let title: String
        let url: String

        var id: String {
            "\(title)|\(url)"
        }
    }

    let dispatchPhase: String
    let requestID: String
    let endpoint: String
    let outcome: String
    let nextMove: String
    let reasonCode: String
    let sessionID: String
    let turnID: String
    let authoritativeResponseText: String
    let retrievedAtLabel: String?
    let cacheStatusLabel: String?
    let sources: [Source]
    let readOnlyToolLaneState: DesktopConversationReadOnlyToolLaneState
    let playbackPhase: String
    let playbackTitle: String
    let playbackSummary: String
    let playbackDetail: String

    var id: String {
        [
            dispatchPhase,
            requestID,
            endpoint,
            outcome,
            nextMove,
            reasonCode,
            sessionID,
            turnID,
            authoritativeResponseText,
            retrievedAtLabel ?? "retrieved_at_not_available",
            cacheStatusLabel ?? "cache_status_not_available",
            sources.map(\.id).joined(separator: "|"),
            readOnlyToolLaneState.id,
            playbackPhase,
            playbackTitle,
            playbackSummary,
            playbackDetail,
        ].joined(separator: "::")
    }
}

struct DesktopConversationAuthoritativeReplyCompletionState: Identifiable, Equatable {
    struct Source: Identifiable, Equatable {
        let title: String
        let url: String

        var id: String {
            "\(title)|\(url)"
        }
    }

    let dispatchPhase: String
    let requestID: String
    let endpoint: String
    let outcome: String
    let nextMove: String
    let reasonCode: String
    let sessionID: String
    let turnID: String
    let authoritativeResponseText: String
    let retrievedAtLabel: String?
    let cacheStatusLabel: String?
    let sources: [Source]
    let playbackPhase: String
    let playbackTitle: String
    let playbackSummary: String
    let playbackDetail: String

    var id: String {
        [
            dispatchPhase,
            requestID,
            endpoint,
            outcome,
            nextMove,
            reasonCode,
            sessionID,
            turnID,
            authoritativeResponseText,
            retrievedAtLabel ?? "retrieved_at_not_available",
            cacheStatusLabel ?? "cache_status_not_available",
            sources.map(\.id).joined(separator: "|"),
            playbackPhase,
            playbackTitle,
            playbackSummary,
            playbackDetail,
        ].joined(separator: "::")
    }
}

struct DesktopConversationRuntimeDispatchFailureAttachmentState: Identifiable, Equatable {
    let dispatchPhase: String
    let requestID: String
    let endpoint: String
    let outcome: String
    let nextMove: String
    let reasonCode: String
    let failureClass: String
    let sessionID: String
    let turnID: String
    let summary: String
    let detail: String

    var id: String {
        [
            dispatchPhase,
            requestID,
            endpoint,
            outcome,
            nextMove,
            reasonCode,
            failureClass,
            sessionID,
            turnID,
            summary,
            detail,
        ].joined(separator: "::")
    }
}

struct DesktopConversationExplicitVoiceLivePreviewAttachmentState: Identifiable, Equatable {
    let sourceSurface: String
    let captureState: String
    let captureMode: String
    let transcriptPosture: String
    let transcriptBytes: String

    var id: String {
        [
            sourceSurface,
            captureState,
            captureMode,
            transcriptPosture,
            transcriptBytes,
        ].joined(separator: "::")
    }
}

struct DesktopConversationWakeTriggeredVoiceLivePreviewAttachmentState: Identifiable, Equatable {
    let sourceSurface: String
    let listenerState: String
    let wakeTriggerPhrase: String
    let transcriptPosture: String
    let transcriptBytes: String

    var id: String {
        [
            sourceSurface,
            listenerState,
            wakeTriggerPhrase,
            transcriptPosture,
            transcriptBytes,
        ].joined(separator: "::")
    }
}

struct DesktopConversationExplicitVoiceFailedRequestAttachmentState: Identifiable, Equatable {
    let failureID: String
    let sourceSurface: String
    let failureTitle: String
    let failureSummary: String
    let failureDetail: String

    var id: String {
        [
            failureID,
            sourceSurface,
            failureTitle,
            failureSummary,
            failureDetail,
        ].joined(separator: "::")
    }
}

struct DesktopConversationWakeTriggeredVoiceFailedRequestAttachmentState: Identifiable, Equatable {
    let failureID: String
    let sourceSurface: String
    let listenerState: String
    let wakeTriggerPhrase: String
    let failureTitle: String
    let failureSummary: String
    let failureDetail: String

    var id: String {
        [
            failureID,
            sourceSurface,
            listenerState,
            wakeTriggerPhrase,
            failureTitle,
            failureSummary,
            failureDetail,
        ].joined(separator: "::")
    }
}

struct DesktopConversationExplicitVoicePendingAttachmentState: Identifiable, Equatable {
    let requestID: String
    let sourceSurface: String
    let captureMode: String
    let transcriptPosture: String
    let transcriptBytes: String
    let selectedMic: String
    let selectedSpeaker: String
    let deviceRoute: String
    let localeTag: String
    let ttsPlaybackActive: String
    let captureDegraded: String
    let streamGapDetected: String
    let deviceChanged: String

    var id: String {
        [
            requestID,
            sourceSurface,
            captureMode,
            transcriptPosture,
            transcriptBytes,
            selectedMic,
            selectedSpeaker,
            deviceRoute,
            localeTag,
            ttsPlaybackActive,
            captureDegraded,
            streamGapDetected,
            deviceChanged,
        ].joined(separator: "::")
    }
}

struct DesktopConversationWakeTriggeredVoicePendingAttachmentState: Identifiable, Equatable {
    let requestID: String
    let sourceSurface: String
    let wakeTriggerPhrase: String
    let transcriptPosture: String
    let transcriptBytes: String
    let selectedMic: String
    let selectedSpeaker: String
    let deviceRoute: String
    let localeTag: String
    let ttsPlaybackActive: String
    let captureDegraded: String
    let streamGapDetected: String
    let deviceChanged: String

    var id: String {
        [
            requestID,
            sourceSurface,
            wakeTriggerPhrase,
            transcriptPosture,
            transcriptBytes,
            selectedMic,
            selectedSpeaker,
            deviceRoute,
            localeTag,
            ttsPlaybackActive,
            captureDegraded,
            streamGapDetected,
            deviceChanged,
        ].joined(separator: "::")
    }
}

struct DesktopConversationTimelineEntryState: Identifiable, Equatable {
    let speaker: String
    let posture: String
    let body: String
    let detail: String
    let sourceSurface: String

    var id: String {
        [
            speaker,
            posture,
            body,
            detail,
            sourceSurface,
        ].joined(separator: "::")
    }

    var isUserAuthored: Bool {
        speaker == "You"
    }
}

struct DesktopConversationSupportRailState: Identifiable, Equatable {
    let title: String
    let detail: String
    let supportSurfaceLabels: [String]

    var id: String {
        ([title, detail] + supportSurfaceLabels).joined(separator: "::")
    }
}

private struct DesktopSessionRecentListRowState: Identifiable, Equatable {
    let sessionID: String
    let sessionState: String
    let lastTurnID: String?

    var id: String {
        sessionID
    }

    var title: String {
        switch sessionState {
        case "ACTIVE":
            return "Recent active session"
        case "SOFT_CLOSED":
            return "Recent archived slice"
        case "SUSPENDED":
            return "Recent suspended session"
        default:
            return "Recent session"
        }
    }

    var summary: String {
        if let lastTurnID {
            return "Read-only recent-session row for `\(sessionID)` with exact `session_state=\(sessionState)` and exact `last_turn_id=\(lastTurnID)`."
        }

        return "Read-only recent-session row for `\(sessionID)` with exact `session_state=\(sessionState)` and no visible `last_turn_id`."
    }
}

struct DesktopSessionSoftClosedVisibilityState: Identifiable, Equatable {
    let sourceSurfaceIdentity: String
    let sessionState: String
    let sessionID: String
    let selectedThreadID: String?
    let selectedThreadTitle: String?
    let pendingWorkOrderID: String?
    let resumeTier: String?
    let resumeSummaryBullets: [String]
    let archivedUserTurnText: String
    let archivedSeleneTurnText: String

    var id: String {
        [
            sourceSurfaceIdentity,
            sessionState,
            sessionID,
            selectedThreadID ?? "selected_thread_not_provided",
            selectedThreadTitle ?? "selected_thread_title_not_provided",
            pendingWorkOrderID ?? "pending_work_order_not_provided",
            resumeTier ?? "resume_tier_not_provided",
            resumeSummaryBullets.joined(separator: "|"),
            archivedUserTurnText,
            archivedSeleneTurnText,
        ].joined(separator: "::")
    }
}

struct DesktopSessionSuspendedVisibilityState: Identifiable, Equatable {
    let sourceSurfaceIdentity: String
    let sessionState: String
    let sessionID: String
    let nextAllowedActionsMaySpeak: Bool
    let nextAllowedActionsMustWait: Bool
    let nextAllowedActionsMustRewake: Bool
    let recoveryMode: String?
    let reconciliationDecision: String?

    var id: String {
        [
            sourceSurfaceIdentity,
            sessionState,
            sessionID,
            booleanValue(nextAllowedActionsMaySpeak),
            booleanValue(nextAllowedActionsMustWait),
            booleanValue(nextAllowedActionsMustRewake),
            recoveryMode ?? "recovery_mode_not_provided",
            reconciliationDecision ?? "reconciliation_decision_not_provided",
        ].joined(separator: "::")
    }
}

private struct DesktopRecoveryVisibilityState: Identifiable, Equatable {
    let displayState: DesktopRecoveryDisplayState
    let sourceSurfaceIdentity: String
    let sessionState: String
    let sessionID: String
    let recoveryMode: String?
    let reconciliationDecision: String?

    var id: String {
        [
            displayState.rawValue,
            sourceSurfaceIdentity,
            sessionState,
            sessionID,
            recoveryMode ?? "recovery_mode_not_provided",
            reconciliationDecision ?? "reconciliation_decision_not_provided",
        ].joined(separator: "::")
    }
}

private struct DesktopInterruptVisibilityState: Identifiable, Equatable {
    let sourceSurfaceIdentity: String
    let sessionState: String
    let sessionID: String
    let turnID: String
    let interruptSubjectRelation: String?
    let interruptContinuityOutcome: String?
    let interruptResumePolicy: String?
    let returnCheckPending: Bool?
    let acceptedInterruptPostureSummary: String

    var interruptContinuityRows: [(label: String, value: String)] {
        var rows: [(label: String, value: String)] = []

        if let interruptSubjectRelation {
            rows.append(("interrupt_subject_relation", interruptSubjectRelation))
        }

        if let interruptContinuityOutcome {
            rows.append(("interrupt_continuity_outcome", interruptContinuityOutcome))
        }

        if let interruptResumePolicy {
            rows.append(("interrupt_resume_policy", interruptResumePolicy))
        }

        return rows
    }

    var id: String {
        [
            sourceSurfaceIdentity,
            sessionState,
            sessionID,
            turnID,
            interruptSubjectRelation ?? "interrupt_subject_relation_not_provided",
            interruptContinuityOutcome ?? "interrupt_continuity_outcome_not_provided",
            interruptResumePolicy ?? "interrupt_resume_policy_not_provided",
            returnCheckPending.map(booleanValue) ?? "return_check_pending_not_provided",
            acceptedInterruptPostureSummary,
        ].joined(separator: "::")
    }
}

private struct DesktopInterruptResponseProductionState: Identifiable, Equatable {
    let sourceSurfaceIdentity: String
    let sessionState: String
    let sessionID: String
    let turnID: String
    let interruptSubjectRelation: String?
    let interruptContinuityOutcome: String?
    let interruptResumePolicy: String?
    let returnCheckPending: Bool?
    let hasInterruptResponseConflict: Bool
    let hasLawfulInterruptClarifyDirective: Bool
    let interruptClarifyQuestion: String?
    let interruptAcceptedAnswerFormats: [String]
    let activeContext: DesktopSessionActiveVisibleContext

    var interruptContinuityRows: [(label: String, value: String)] {
        var rows: [(label: String, value: String)] = []

        if let interruptSubjectRelation {
            rows.append(("interrupt_subject_relation", interruptSubjectRelation))
        }

        if let interruptContinuityOutcome {
            rows.append(("interrupt_continuity_outcome", interruptContinuityOutcome))
        }

        if let interruptResumePolicy {
            rows.append(("interrupt_resume_policy", interruptResumePolicy))
        }

        return rows
    }

    var id: String {
        [
            sourceSurfaceIdentity,
            sessionState,
            sessionID,
            turnID,
            interruptSubjectRelation ?? "interrupt_subject_relation_not_provided",
            interruptContinuityOutcome ?? "interrupt_continuity_outcome_not_provided",
            interruptResumePolicy ?? "interrupt_resume_policy_not_provided",
            returnCheckPending.map(booleanValue) ?? "return_check_pending_not_provided",
            booleanValue(hasInterruptResponseConflict),
            booleanValue(hasLawfulInterruptClarifyDirective),
            interruptClarifyQuestion ?? "interrupt_clarify_question_not_provided",
            interruptAcceptedAnswerFormats.joined(separator: "|"),
        ].joined(separator: "::")
    }
}

private struct DesktopInterruptSubjectReferencesVisibilityState: Identifiable, Equatable {
    let sourceSurfaceIdentity: String
    let sessionState: String
    let sessionID: String
    let turnID: String
    let interruptSubjectRelation: String?
    let activeSubjectRef: String?
    let interruptedSubjectRef: String?
    let hasLawfulInterruptSubjectReferences: Bool

    var id: String {
        [
            sourceSurfaceIdentity,
            sessionState,
            sessionID,
            turnID,
            interruptSubjectRelation ?? "interrupt_subject_relation_not_provided",
            activeSubjectRef ?? "active_subject_ref_not_provided",
            interruptedSubjectRef ?? "interrupted_subject_ref_not_provided",
            booleanValue(hasLawfulInterruptSubjectReferences),
        ].joined(separator: "::")
    }
}

private struct DesktopInterruptSubjectRelationConfidenceVisibilityState: Identifiable,
    Equatable
{
    let sourceSurfaceIdentity: String
    let sessionState: String
    let sessionID: String
    let turnID: String
    let interruptSubjectRelation: String?
    let interruptSubjectRelationConfidence: String?
    let hasLawfulInterruptSubjectRelationConfidence: Bool

    var id: String {
        [
            sourceSurfaceIdentity,
            sessionState,
            sessionID,
            turnID,
            interruptSubjectRelation ?? "interrupt_subject_relation_not_provided",
            interruptSubjectRelationConfidence ?? "interrupt_subject_relation_confidence_not_provided",
            booleanValue(hasLawfulInterruptSubjectRelationConfidence),
        ].joined(separator: "::")
    }
}

private struct DesktopInterruptReturnCheckExpiryVisibilityState: Identifiable,
    Equatable
{
    let sourceSurfaceIdentity: String
    let sessionState: String
    let sessionID: String
    let turnID: String
    let interruptSubjectRelation: String?
    let interruptContinuityOutcome: String?
    let interruptResumePolicy: String?
    let returnCheckPending: Bool?
    let returnCheckExpiresAt: String?
    let hasLawfulInterruptReturnCheckExpiry: Bool

    var interruptContinuityRows: [(label: String, value: String)] {
        var rows: [(label: String, value: String)] = []

        if let interruptSubjectRelation {
            rows.append(("interrupt_subject_relation", interruptSubjectRelation))
        }

        if let interruptContinuityOutcome {
            rows.append(("interrupt_continuity_outcome", interruptContinuityOutcome))
        }

        if let interruptResumePolicy {
            rows.append(("interrupt_resume_policy", interruptResumePolicy))
        }

        return rows
    }

    var id: String {
        [
            sourceSurfaceIdentity,
            sessionState,
            sessionID,
            turnID,
            interruptSubjectRelation ?? "interrupt_subject_relation_not_provided",
            interruptContinuityOutcome ?? "interrupt_continuity_outcome_not_provided",
            interruptResumePolicy ?? "interrupt_resume_policy_not_provided",
            returnCheckPending.map(booleanValue) ?? "return_check_pending_not_provided",
            returnCheckExpiresAt ?? "return_check_expires_at_not_provided",
            booleanValue(hasLawfulInterruptReturnCheckExpiry),
        ].joined(separator: "::")
    }
}

struct DesktopSenderVerificationVisibilityState: Identifiable, Equatable {
    let onboardingSessionID: String
    let nextStep: String
    let onboardingStatus: String?
    let requiredVerificationGates: [String]

    var id: String {
        [
            onboardingSessionID,
            nextStep,
            onboardingStatus ?? "onboarding_status_not_provided",
            requiredVerificationGates.joined(separator: "|"),
        ].joined(separator: "::")
    }
}

struct DesktopEmployeePhotoCaptureSendPromptState: Identifiable, Equatable {
    let onboardingSessionID: String
    let nextStep: String
    let onboardingStatus: String?
    let requiredVerificationGates: [String]

    var id: String {
        [
            onboardingSessionID,
            nextStep,
            onboardingStatus ?? "onboarding_status_not_provided",
            requiredVerificationGates.joined(separator: "|"),
        ].joined(separator: "::")
    }
}

struct DesktopEmployeeSenderVerifyCommitPromptState: Identifiable, Equatable {
    let onboardingSessionID: String
    let nextStep: String
    let onboardingStatus: String?
    let requiredVerificationGates: [String]
    let photoBlobRef: String

    var id: String {
        [
            onboardingSessionID,
            nextStep,
            onboardingStatus ?? "onboarding_status_not_provided",
            requiredVerificationGates.joined(separator: "|"),
            photoBlobRef,
        ].joined(separator: "::")
    }
}

struct DesktopPlatformSetupReceiptDraft: Identifiable, Equatable {
    let onboardingSessionID: String
    let receiptKind: String
    let deviceID: String?
    let proofMaterial: String
    let proofSummary: String
    let wakeRuntimeEventWakeProfileID: String?
    let wakeRuntimeEventAccepted: Bool?
    let voiceArtifactSyncReceiptRef: String?

    init(
        onboardingSessionID: String,
        receiptKind: String,
        deviceID: String?,
        proofMaterial: String,
        proofSummary: String,
        wakeRuntimeEventWakeProfileID: String? = nil,
        wakeRuntimeEventAccepted: Bool? = nil,
        voiceArtifactSyncReceiptRef: String? = nil
    ) {
        self.onboardingSessionID = onboardingSessionID
        self.receiptKind = receiptKind
        self.deviceID = deviceID
        self.proofMaterial = proofMaterial
        self.proofSummary = proofSummary
        self.wakeRuntimeEventWakeProfileID = wakeRuntimeEventWakeProfileID
        self.wakeRuntimeEventAccepted = wakeRuntimeEventAccepted
        self.voiceArtifactSyncReceiptRef = voiceArtifactSyncReceiptRef
    }

    var id: String {
        "\(onboardingSessionID)::\(receiptKind)"
    }

    var buttonTitle: String {
        switch receiptKind {
        case "install_launch_handshake":
            return "Submit install launch handshake"
        case "mic_permission_granted":
            return "Submit microphone permission receipt"
        case "desktop_pairing_bound":
            return "Submit desktop pairing-bound receipt"
        case "desktop_wakeword_configured":
            return "Submit desktop wakeword-configured receipt"
        default:
            return "Submit desktop platform receipt"
        }
    }
}

private struct DesktopSelectedSessionProjectContextState: Equatable {
    let sessionID: String
    let projectID: String?
    let pinnedContextRefs: [String]

    var hasLawfulCarrier: Bool {
        projectID != nil || !pinnedContextRefs.isEmpty
    }
}

struct DesktopSessionShellView: View {
    @Environment(\.scenePhase) private var scenePhase
    @State private var latestSessionHeaderContext: DesktopSessionHeaderContext?
    @State private var latestSessionActiveVisibleContext: DesktopSessionActiveVisibleContext?
    @State private var latestSessionSoftClosedVisibleContext: DesktopSessionSoftClosedVisibleContext?
    @State private var latestSessionSuspendedVisibleContext: DesktopSessionSuspendedVisibleContext?
    @State private var observedSessionSurfaces: [DesktopObservedSessionSurface] = []
    @State private var selectedObservedSessionSurfaceID: String?
    @State private var desktopSelectedSessionProjectContexts: [String: DesktopSelectedSessionProjectContextState] = [:]
    @State private var desktopOnboardingEntryContext: DesktopOnboardingEntryContext?
    @State private var interruptResponsePendingRequest: InterruptContinuityResponseRequestState?
    @State private var interruptResponseFailedRequest: InterruptContinuityResponseFailureState?
    @State private var interruptResponseRequestSequence: Int = 0
    @StateObject private var explicitVoiceController = ExplicitVoiceCaptureController()
    @StateObject private var desktopWakeListenerController = DesktopWakeListenerController()
    @StateObject private var desktopCanonicalRuntimeBridge = DesktopCanonicalRuntimeBridge()
    @StateObject private var desktopAuthoritativeReplyPlaybackController = DesktopAuthoritativeReplyPlaybackController()
    @State private var desktopCanonicalRuntimeOutcomeState: DesktopCanonicalRuntimeOutcomeState?
    @State private var desktopInviteOpenRuntimeOutcomeState: DesktopInviteOpenRuntimeOutcomeState?
    @State private var desktopOnboardingContinueRuntimeOutcomeState: DesktopOnboardingContinueRuntimeOutcomeState?
    @State private var desktopPlatformSetupReceiptRuntimeOutcomeState: DesktopPlatformSetupReceiptRuntimeOutcomeState?
    @State private var desktopTermsAcceptRuntimeOutcomeState: DesktopTermsAcceptRuntimeOutcomeState?
    @State private var desktopEmployeePhotoCaptureSendRuntimeOutcomeState: DesktopEmployeePhotoCaptureSendRuntimeOutcomeState?
    @State private var desktopEmployeeSenderVerifyCommitRuntimeOutcomeState: DesktopEmployeeSenderVerifyCommitRuntimeOutcomeState?
    @State private var desktopPrimaryDeviceConfirmRuntimeOutcomeState: DesktopPrimaryDeviceConfirmRuntimeOutcomeState?
    @State private var desktopVoiceEnrollRuntimeOutcomeState: DesktopVoiceEnrollRuntimeOutcomeState?
    @State private var desktopWakeEnrollStartDraftRuntimeOutcomeState: DesktopWakeEnrollStartDraftRuntimeOutcomeState?
    @State private var desktopWakeEnrollSampleCommitRuntimeOutcomeState: DesktopWakeEnrollSampleCommitRuntimeOutcomeState?
    @State private var desktopWakeEnrollCompleteCommitRuntimeOutcomeState: DesktopWakeEnrollCompleteCommitRuntimeOutcomeState?
    @State private var desktopWakeEnrollDeferCommitRuntimeOutcomeState: DesktopWakeEnrollDeferCommitRuntimeOutcomeState?
    @State private var desktopSessionAttachRuntimeOutcomeState: DesktopSessionAttachRuntimeOutcomeState?
    @State private var desktopSessionMultiPostureResumeRuntimeOutcomeState: DesktopSessionMultiPostureResumeRuntimeOutcomeState?
    @State private var desktopSessionMultiPostureEntryRuntimeOutcomeState: DesktopSessionMultiPostureEntryRuntimeOutcomeState?
    @State private var desktopPairingCompletionCommitRuntimeOutcomeState: DesktopPairingCompletionCommitRuntimeOutcomeState?
    @State private var desktopReadyTimeHandoffState: DesktopReadyTimeHandoffState?
    @State private var desktopWakeProfileAvailabilityRuntimeOutcomeState: DesktopWakeProfileAvailabilityRuntimeOutcomeState?
    @State private var desktopSessionRecentListRuntimeOutcomeState: DesktopSessionRecentListRuntimeOutcomeState?
    @State private var desktopEmoPersonaLockRuntimeOutcomeState: DesktopEmoPersonaLockRuntimeOutcomeState?
    @State private var desktopAccessProvisionCommitRuntimeOutcomeState: DesktopAccessProvisionCommitRuntimeOutcomeState?
    @State private var desktopCompleteCommitRuntimeOutcomeState: DesktopCompleteCommitRuntimeOutcomeState?
    @State private var desktopOnboardingContinueFieldInput: String = ""
    @State private var desktopEmployeePhotoCaptureSendPhotoBlobRefInput: String = ""
    @State private var desktopEmployeeSenderVerifyCommitSelectedDecision: String = "CONFIRM"
    @State private var desktopAuthoritativeReplyRenderState: DesktopAuthoritativeReplyRenderState?
    @State private var desktopAuthoritativeReplyProvenanceRenderState: DesktopAuthoritativeReplyProvenanceRenderState?
    @State private var desktopAuthoritativeReplyPlaybackState: DesktopAuthoritativeReplyPlaybackState = .idle
    @State private var desktopTypedTurnDraft: String = ""
    @State private var desktopSearchRequestDraft: String = ""
    @State private var desktopToolRequestDraft: String = ""
    @State private var desktopTypedTurnPendingRequest: DesktopTypedTurnRequestState?
    @State private var desktopTypedTurnFailedRequest: InterruptContinuityResponseFailureState?
    @State private var desktopSearchRequestFailedRequest: InterruptContinuityResponseFailureState?
    @State private var desktopToolRequestFailedRequest: InterruptContinuityResponseFailureState?
    @State private var desktopTypedTurnRequestSequence: Int = 0
    @State private var lastStagedWakeTriggeredVoiceTurnRequestState: WakeTriggeredVoiceTurnRequestState?
    @State private var desktopWakeAutoStartAttemptedPromptID: String?
    @State private var desktopWakeAutoStartSuppressedPromptID: String?
    private let maxDesktopTypedTurnBytes = 16_384

    var body: some View {
        Group {
            if let operationalConversationShellState = desktopOperationalConversationShellState {
                desktopOperationalConversationShell(operationalConversationShellState)
            } else {
                desktopEvidenceFirstOperationalShell
            }
        }
        .padding(24)
        .frame(minWidth: 1180, minHeight: 720, alignment: .topLeading)
        .background(Color(nsColor: .windowBackgroundColor))
        .task(id: explicitVoiceController.pendingRequest?.id) {
            await dispatchPreparedExplicitVoiceRequestIfNeeded()
            await synchronizeDesktopWakeListenerLifecycleState()
        }
        .task(id: desktopWakeListenerController.pendingRequest?.id) {
            await dispatchPreparedWakeTriggeredVoiceRequestIfNeeded()
            await synchronizeDesktopWakeListenerLifecycleState()
        }
        .task(id: desktopTypedTurnPendingRequest?.id) {
            await dispatchPreparedTypedTurnRequestIfNeeded()
            await synchronizeDesktopWakeListenerLifecycleState()
        }
        .task(id: desktopOnboardingEntryContext?.id) {
            await openInviteLinkAndStartOnboardingIfNeeded()
        }
        .task(id: desktopOnboardingContinuePromptSeedID) {
            await fetchOnboardingContinuePromptIfNeeded()
        }
        .task(id: desktopPairingCompletionPromptState?.id) {
            synchronizeDesktopPairingCompletionReadyTimeHandoffState()
        }
        .task(id: desktopWakeProfileAvailabilityPromptState?.id) {
            synchronizeDesktopWakeProfileAvailabilityRuntimeOutcomeState()
        }
        .task(id: desktopWakeListenerPromptState?.id) {
            await synchronizeDesktopWakeListenerLifecycleState()
        }
        .task(id: explicitVoiceController.isListening) {
            await synchronizeDesktopWakeListenerLifecycleState()
        }
        .task(id: explicitVoiceController.pendingRequest?.id) {
            await synchronizeDesktopWakeListenerLifecycleState()
        }
        .task(id: desktopOperationalConversationShellState?.id) {
            await synchronizeDesktopWakeListenerLifecycleState()
            await synchronizeDesktopSessionRecentListRuntimeOutcomeState()
        }
        .task(id: desktopSessionAttachRuntimeOutcomeState?.id) {
            await synchronizeDesktopSessionRecentListRuntimeOutcomeState()
        }
        .task(id: desktopSessionMultiPostureResumeRuntimeOutcomeState?.id) {
            await synchronizeDesktopSessionRecentListRuntimeOutcomeState()
        }
        .task(id: desktopSessionMultiPostureEntryRuntimeOutcomeState?.id) {
            await synchronizeDesktopSessionRecentListRuntimeOutcomeState()
        }
        .task(id: scenePhase) {
            await synchronizeDesktopWakeListenerLifecycleState()
        }
        .onReceive(desktopAuthoritativeReplyPlaybackController.$playbackState) { playbackState in
            desktopAuthoritativeReplyPlaybackState = playbackState
        }
        .onDisappear {
            explicitVoiceController.haltCaptureSession()
            desktopWakeListenerController.haltCaptureSession()
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
                    desktopEmployeePhotoCaptureSendRuntimeOutcomeState = nil
                    desktopEmployeeSenderVerifyCommitRuntimeOutcomeState = nil
                    desktopPrimaryDeviceConfirmRuntimeOutcomeState = nil
                    desktopVoiceEnrollRuntimeOutcomeState = nil
                    desktopWakeEnrollStartDraftRuntimeOutcomeState = nil
                    desktopWakeEnrollSampleCommitRuntimeOutcomeState = nil
                    desktopWakeEnrollCompleteCommitRuntimeOutcomeState = nil
                    desktopEmoPersonaLockRuntimeOutcomeState = nil
                    desktopAccessProvisionCommitRuntimeOutcomeState = nil
                    desktopCompleteCommitRuntimeOutcomeState = nil
                    desktopPairingCompletionCommitRuntimeOutcomeState = nil
                    desktopReadyTimeHandoffState = nil
                    desktopWakeProfileAvailabilityRuntimeOutcomeState = nil
                    desktopOnboardingContinueFieldInput = ""
                    desktopEmployeePhotoCaptureSendPhotoBlobRefInput = ""
                    desktopEmployeeSenderVerifyCommitSelectedDecision = "CONFIRM"
                }
                desktopOnboardingEntryContext = context
            }

            if let context = DesktopSessionActiveVisibleContext(url: url) {
                clearInterruptResponseState()
                latestSessionActiveVisibleContext = context
                latestSessionSoftClosedVisibleContext = nil
                latestSessionSuspendedVisibleContext = nil

                if let sessionAttachOutcome = context.sessionAttachOutcome {
                    let headerContext = DesktopSessionHeaderContext(
                        sessionState: context.sessionState,
                        sessionID: context.sessionID,
                        sessionAttachOutcome: sessionAttachOutcome,
                        recoveryMode: context.recoveryMode,
                        reconciliationDecision: context.reconciliationDecision
                    )
                    latestSessionHeaderContext = headerContext
                    recordObservedSessionSurface(.sessionHeader(headerContext))
                } else if latestSessionHeaderContext?.sessionID != context.sessionID {
                    latestSessionHeaderContext = nil
                }

                recordObservedSessionSurface(.sessionActive(context))

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

                recordObservedSessionSurface(.sessionSoftClosed(context))

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

                recordObservedSessionSurface(.sessionSuspended(context))

                return
            }

            if let context = DesktopSessionHeaderContext(url: url) {
                clearInterruptResponseState()
                latestSessionHeaderContext = context
                latestSessionActiveVisibleContext = nil
                latestSessionSoftClosedVisibleContext = nil
                latestSessionSuspendedVisibleContext = nil
                recordObservedSessionSurface(.sessionHeader(context))
            }
        }
    }

    private var desktopEvidenceFirstOperationalShell: some View {
        HStack(alignment: .top, spacing: 20) {
            VStack(alignment: .leading, spacing: 16) {
                posturePanel

                historyCard
            }
            .frame(width: 270, alignment: .topLeading)

            VStack(alignment: .leading, spacing: 16) {
                explicitVoiceEntryAffordanceCard
                desktopWakeProfileAvailabilityCard
                desktopWakeListenerControlCard

                if desktopReadyTimeHandoffIsActive {
                    desktopPairingCompletionMutationCard
                } else {
                    desktopOnboardingEntryCard
                    desktopOnboardingContinuePromptCard
                    desktopPlatformSetupReceiptSubmissionCard
                    desktopTermsAcceptCard
                    desktopSenderVerificationVisibilityCard
                    desktopEmployeePhotoCaptureSendCard
                    desktopEmployeeSenderVerifyCommitCard
                    desktopPrimaryDeviceConfirmCard
                    desktopVoiceEnrollCard
                    desktopWakeEnrollStartDraftCard
                    desktopWakeEnrollSampleCommitCard
                    desktopWakeEnrollCompleteCommitCard
                    desktopWakeEnrollDeferCommitCard
                    desktopEmoPersonaLockCard
                    desktopAccessProvisionCommitCard
                    desktopCompleteCommitCard
                    desktopReadyVisibilityCard
                    desktopPairingCompletionVisibilityCard
                    desktopPairingCompletionMutationCard
                }
                desktopSessionSoftClosedVisibilityCard
                desktopSessionSuspendedVisibilityCard
                desktopRecoveryVisibilityCard
                desktopInterruptVisibilityCard
                desktopInterruptResponseProductionCard
                desktopInterruptSubjectReferencesVisibilityCard
                desktopInterruptSubjectRelationConfidenceVisibilityCard
                desktopInterruptReturnCheckExpiryVisibilityCard

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
    }

    private var activeRecoveryVisibleSurface: DesktopRecoveryVisibleSurface? {
        guard foregroundSessionSuspendedVisibleContext == nil else {
            return nil
        }

        if let foregroundSessionActiveVisibleContext {
            return .sessionActive(foregroundSessionActiveVisibleContext)
        }

        if let foregroundSessionSoftClosedVisibleContext {
            return .sessionSoftClosed(foregroundSessionSoftClosedVisibleContext)
        }

        if let foregroundSessionHeaderContext {
            return .sessionHeader(foregroundSessionHeaderContext)
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
              foregroundSessionSuspendedVisibleContext == nil,
              let foregroundSessionActiveVisibleContext else {
            return nil
        }

        return resolvedInterruptDisplayState(
            interruptSubjectRelation: foregroundSessionActiveVisibleContext.interruptSubjectRelation,
            interruptContinuityOutcome: foregroundSessionActiveVisibleContext.interruptContinuityOutcome,
            interruptResumePolicy: foregroundSessionActiveVisibleContext.interruptResumePolicy
        )
    }

    private var currentDominantObservedSessionSurface: DesktopObservedSessionSurface? {
        if let latestSessionSuspendedVisibleContext {
            return .sessionSuspended(latestSessionSuspendedVisibleContext)
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

    private var selectedObservedSessionSurface: DesktopObservedSessionSurface? {
        guard let selectedObservedSessionSurfaceID else {
            return nil
        }

        return observedSessionSurfaces.first { $0.id == selectedObservedSessionSurfaceID }
    }

    private var foregroundObservedSessionSurface: DesktopObservedSessionSurface? {
        selectedObservedSessionSurface ?? currentDominantObservedSessionSurface
    }

    private var desktopForegroundVoiceTurnSelectedSessionProjectContext: DesktopSelectedSessionProjectContextState? {
        guard let foregroundObservedSessionSurface,
              let activeVisibleContext = latestSessionActiveVisibleContext,
              foregroundObservedSessionSurface.sessionID == activeVisibleContext.sessionID,
              let context = desktopSelectedSessionProjectContexts[foregroundObservedSessionSurface.sessionID],
              context.hasLawfulCarrier else {
            return nil
        }

        return context
    }

    private var desktopForegroundVoiceTurnSelectedProjectID: String? {
        desktopForegroundVoiceTurnSelectedSessionProjectContext?.projectID
    }

    private var desktopForegroundVoiceTurnSelectedPinnedContextRefs: [String] {
        desktopForegroundVoiceTurnSelectedSessionProjectContext?.pinnedContextRefs ?? []
    }

    private var desktopForegroundSelectionShowsCurrentDominantSurface: Bool {
        guard let selectedObservedSessionSurface else {
            return true
        }

        return selectedObservedSessionSurface.id == currentDominantObservedSessionSurface?.id
    }

    private var foregroundSessionHeaderContext: DesktopSessionHeaderContext? {
        guard let foregroundObservedSessionSurface else {
            return nil
        }

        if case .sessionHeader(let context) = foregroundObservedSessionSurface {
            return context
        }

        return nil
    }

    private var foregroundSessionActiveVisibleContext: DesktopSessionActiveVisibleContext? {
        guard let foregroundObservedSessionSurface else {
            return nil
        }

        if case .sessionActive(let context) = foregroundObservedSessionSurface {
            return context
        }

        return nil
    }

    private var foregroundSessionSoftClosedVisibleContext: DesktopSessionSoftClosedVisibleContext? {
        guard let foregroundObservedSessionSurface else {
            return nil
        }

        if case .sessionSoftClosed(let context) = foregroundObservedSessionSurface {
            return context
        }

        return nil
    }

    private var foregroundSessionSuspendedVisibleContext: DesktopSessionSuspendedVisibleContext? {
        guard let foregroundObservedSessionSurface else {
            return nil
        }

        if case .sessionSuspended(let context) = foregroundObservedSessionSurface {
            return context
        }

        return nil
    }

    private func recordObservedSessionSurface(_ surface: DesktopObservedSessionSurface) {
        observedSessionSurfaces.removeAll { $0.id == surface.id }
        observedSessionSurfaces.insert(surface, at: 0)

        if observedSessionSurfaces.count > 8 {
            observedSessionSurfaces.removeSubrange(8...)
        }

        synchronizeDesktopSelectedSessionProjectContextState()
    }

    private func selectObservedSessionSurface(_ surface: DesktopObservedSessionSurface) {
        if surface.id == currentDominantObservedSessionSurface?.id {
            selectedObservedSessionSurfaceID = nil
        } else {
            selectedObservedSessionSurfaceID = surface.id
        }
    }

    @MainActor
    private func synchronizeDesktopSelectedSessionProjectContextState() {
        func upsert(
            sessionID: String,
            projectID: String?,
            pinnedContextRefs: [String]
        ) {
            let context = DesktopSelectedSessionProjectContextState(
                sessionID: sessionID,
                projectID: projectID,
                pinnedContextRefs: pinnedContextRefs
            )

            if context.hasLawfulCarrier {
                desktopSelectedSessionProjectContexts[sessionID] = context
            } else {
                desktopSelectedSessionProjectContexts.removeValue(forKey: sessionID)
            }
        }

        if let outcomeState = desktopSessionAttachRuntimeOutcomeState,
           outcomeState.phase == .completed {
            upsert(
                sessionID: outcomeState.sessionID,
                projectID: outcomeState.projectID,
                pinnedContextRefs: outcomeState.pinnedContextRefs
            )
        }

        if let outcomeState = desktopSessionMultiPostureResumeRuntimeOutcomeState,
           outcomeState.phase == .completed {
            upsert(
                sessionID: outcomeState.sessionID,
                projectID: outcomeState.projectID,
                pinnedContextRefs: outcomeState.pinnedContextRefs
            )
        }

        if let outcomeState = desktopSessionMultiPostureEntryRuntimeOutcomeState,
           outcomeState.phase == .completed {
            upsert(
                sessionID: outcomeState.sessionID,
                projectID: outcomeState.projectID,
                pinnedContextRefs: outcomeState.pinnedContextRefs
            )
        }

        let observedSessionIDs = Set(observedSessionSurfaces.map(\.sessionID))
        desktopSelectedSessionProjectContexts = desktopSelectedSessionProjectContexts.filter { sessionID, _ in
            observedSessionIDs.contains(sessionID)
        }
    }

    private var desktopSessionRecentListRefreshTriggerID: String? {
        guard desktopOperationalConversationShellState != nil,
              let managedDeviceID = boundedOnboardingContinueFieldValue(
                desktopCanonicalRuntimeBridge.managedDeviceID
              ) else {
            return nil
        }

        let attachTrigger = (desktopSessionAttachRuntimeOutcomeState?.phase == .completed)
            ? desktopSessionAttachRuntimeOutcomeState?.id
            : nil
        let resumeTrigger = (desktopSessionMultiPostureResumeRuntimeOutcomeState?.phase == .completed)
            ? desktopSessionMultiPostureResumeRuntimeOutcomeState?.id
            : nil
        let entryTrigger = (desktopSessionMultiPostureEntryRuntimeOutcomeState?.phase == .completed)
            ? desktopSessionMultiPostureEntryRuntimeOutcomeState?.id
            : nil

        return [
            managedDeviceID,
            attachTrigger ?? "session_attach_not_completed",
            resumeTrigger ?? "session_resume_not_completed",
            entryTrigger ?? "session_entry_not_completed",
        ].joined(separator: "::")
    }

    private var desktopSessionRecentListRowStates: [DesktopSessionRecentListRowState] {
        guard let outcomeState = desktopSessionRecentListRuntimeOutcomeState,
              outcomeState.phase == .completed else {
            return []
        }

        return outcomeState.sessions.map { session in
            DesktopSessionRecentListRowState(
                sessionID: session.sessionID,
                sessionState: session.sessionState,
                lastTurnID: session.lastTurnID
            )
        }
    }

    @MainActor
    private func synchronizeDesktopSessionRecentListRuntimeOutcomeState() async {
        guard let refreshTriggerID = desktopSessionRecentListRefreshTriggerID else {
            if desktopSessionRecentListRuntimeOutcomeState?.phase != .dispatching {
                desktopSessionRecentListRuntimeOutcomeState = nil
            }
            return
        }

        if let outcomeState = desktopSessionRecentListRuntimeOutcomeState {
            if outcomeState.phase == .dispatching {
                return
            }

            if outcomeState.refreshTriggerID == refreshTriggerID {
                return
            }
        }

        await submitDesktopSessionRecentListRefresh(refreshTriggerID: refreshTriggerID)
    }

    @MainActor
    private func submitDesktopSessionRecentListRefresh(
        refreshTriggerID: String
    ) async {
        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopSessionRecentListRequestBuilder()
            desktopSessionRecentListRuntimeOutcomeState = .dispatching(
                deviceID: ingressContext.deviceID,
                refreshTriggerID: refreshTriggerID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.submitDesktopSessionRecentList(
                ingressContext,
                refreshTriggerID: refreshTriggerID
            )
            desktopSessionRecentListRuntimeOutcomeState = outcomeState
        } catch {
            let managedDeviceID = boundedOnboardingContinueFieldValue(
                desktopCanonicalRuntimeBridge.managedDeviceID
            ) ?? "not_available"
            desktopSessionRecentListRuntimeOutcomeState = .failed(
                deviceID: managedDeviceID,
                refreshTriggerID: refreshTriggerID,
                endpoint: desktopCanonicalRuntimeBridge.sessionRecentEndpoint,
                requestID: "unavailable",
                summary: "The canonical recent-session visibility bridge could not stage this bounded desktop current-device recent-session visibility request.",
                detail: error.localizedDescription
            )
        }
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
                        let wakeDispatchInFlight = desktopWakeListenerController.listenerState == .dispatching
                        desktopWakeListenerController.haltCaptureSession()
                        if !wakeDispatchInFlight {
                            desktopWakeListenerController.clearPendingPreparedWakeTurn()
                            lastStagedWakeTriggeredVoiceTurnRequestState = nil
                        }
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
                    if let operationalConversationShellState = desktopOperationalConversationShellState,
                       desktopConversationShouldAttachExplicitVoiceLivePreview(
                           operationalConversationShellState.primaryPaneState.explicitVoiceLivePreviewAttachmentState
                       ) {
                        EmptyView()
                    } else {
                        explicitVoiceTranscriptPreviewCard
                    }
                }

                if let pendingRequest = explicitVoiceController.pendingRequest {
                    if let operationalConversationShellState = desktopOperationalConversationShellState,
                       desktopConversationShouldAttachExplicitVoicePendingAttachment(
                           operationalConversationShellState.primaryPaneState.explicitVoicePendingAttachmentState
                       ) {
                        EmptyView()
                    } else {
                        explicitVoicePendingRequestCard(pendingRequest)
                    }
                }

                if desktopOperationalConversationShellState == nil {
                    if let desktopCanonicalRuntimeOutcomeState {
                        desktopCanonicalRuntimeOutcomeCard(desktopCanonicalRuntimeOutcomeState)
                    }

                    if desktopAuthoritativeReplyRenderState != nil {
                        desktopAuthoritativeReplyCard
                        desktopAuthoritativeReplyProvenanceCard
                        desktopAuthoritativeReplyPlaybackCard
                    }
                }

                if let failedRequest = explicitVoiceController.failedRequest {
                    if let operationalConversationShellState = desktopOperationalConversationShellState,
                       desktopConversationShouldAttachExplicitVoiceFailedRequest(
                           operationalConversationShellState.primaryPaneState.explicitVoiceFailedRequestAttachmentState
                       ) {
                        EmptyView()
                    } else {
                        interruptResponseFailedRequestCard(failedRequest)
                    }
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

    private var trimmedDesktopTypedTurnDraft: String {
        desktopTypedTurnDraft.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private var trimmedDesktopSearchRequestDraft: String {
        desktopSearchRequestDraft.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private var trimmedDesktopToolRequestDraft: String {
        desktopToolRequestDraft.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private var desktopTypedTurnSubmissionInterlocksActive: Bool {
        desktopTypedTurnPendingRequest != nil
            || explicitVoiceController.isListening
            || explicitVoiceController.pendingRequest != nil
            || desktopWakeListenerController.listenerState.isActiveForMicrophone
            || desktopWakeListenerController.listenerState == .dispatching
            || desktopWakeListenerController.pendingRequest != nil
            || lastStagedWakeTriggeredVoiceTurnRequestState != nil
    }

    private var keyboardComposerPendingTypedTurnRequest: DesktopTypedTurnRequestState? {
        guard let desktopTypedTurnPendingRequest,
              desktopTypedTurnPendingRequest.origin == .keyboardComposer else {
            return nil
        }

        return desktopTypedTurnPendingRequest
    }

    private var toolRequestPendingTypedTurnRequest: DesktopTypedTurnRequestState? {
        guard let desktopTypedTurnPendingRequest,
              desktopTypedTurnPendingRequest.origin == .toolRequestCard else {
            return nil
        }

        return desktopTypedTurnPendingRequest
    }

    private var searchRequestPendingTypedTurnRequest: DesktopTypedTurnRequestState? {
        guard let desktopTypedTurnPendingRequest,
              desktopTypedTurnPendingRequest.origin == .searchRequestCard else {
            return nil
        }

        return desktopTypedTurnPendingRequest
    }

    private var desktopSearchRequestCardIsExecutable: Bool {
        desktopReadyTimeHandoffIsActive
            && desktopForegroundSelectionShowsCurrentDominantSurface
            && foregroundSessionSoftClosedVisibleContext == nil
            && foregroundSessionSuspendedVisibleContext == nil
            && activeRecoveryDisplayState != .quarantinedLocalState
    }

    private var desktopSearchRequestReadOnlyDetail: String {
        if !desktopForegroundSelectionShowsCurrentDominantSurface {
            return "This bounded search-request surface stays read-only while a previously observed session surface is foregrounded. Search-request production remains bound to the current lawful dominant desktop surface only."
        }

        if foregroundSessionSoftClosedVisibleContext != nil {
            return "This bounded search-request surface stays read-only while the foregrounded session surface is soft-closed. Archived recent-slice visibility does not itself reopen, resume, or retarget canonical runtime dispatch."
        }

        if foregroundSessionSuspendedVisibleContext != nil {
            return "This bounded search-request surface stays read-only while the foregrounded session surface remains suspended. Suspended posture stays explanation-only until authoritative reread or later lawful continuation clears the suspension cloud-side."
        }

        if activeRecoveryDisplayState == .quarantinedLocalState {
            return "This bounded search-request surface stays read-only while quarantined local recovery posture withholds current active/ready production. Canonical runtime still remains authoritative over later recovery, search routing, and dispatch."
        }

        return "This bounded search-request surface stays read-only until one lawful current active/ready conversation surface becomes foregrounded."
    }

    private var desktopToolRequestCardIsExecutable: Bool {
        desktopReadyTimeHandoffIsActive
            && desktopForegroundSelectionShowsCurrentDominantSurface
            && foregroundSessionSoftClosedVisibleContext == nil
            && foregroundSessionSuspendedVisibleContext == nil
            && activeRecoveryDisplayState != .quarantinedLocalState
    }

    private var desktopToolRequestReadOnlyDetail: String {
        if !desktopForegroundSelectionShowsCurrentDominantSurface {
            return "This bounded tool-request surface stays read-only while a previously observed session surface is foregrounded. Tool-request production remains bound to the current lawful dominant desktop surface only."
        }

        if foregroundSessionSoftClosedVisibleContext != nil {
            return "This bounded tool-request surface stays read-only while the foregrounded session surface is soft-closed. Archived recent-slice visibility does not itself reopen, resume, or retarget canonical runtime dispatch."
        }

        if foregroundSessionSuspendedVisibleContext != nil {
            return "This bounded tool-request surface stays read-only while the foregrounded session surface remains suspended. Suspended posture stays explanation-only until authoritative reread or later lawful continuation clears the suspension cloud-side."
        }

        if activeRecoveryDisplayState == .quarantinedLocalState {
            return "This bounded tool-request surface stays read-only while quarantined local recovery posture withholds current active/ready production. Canonical runtime still remains authoritative over later recovery and dispatch."
        }

        return "This bounded tool-request surface stays read-only until one lawful current active/ready conversation surface becomes foregrounded."
    }

    private var desktopCurrentReadOnlyToolLaneState: DesktopConversationReadOnlyToolLaneState? {
        desktopConversationSearchToolCompletionState?.readOnlyToolLaneState
    }

    private var desktopCurrentSearchToolCompletionState: DesktopConversationSearchToolCompletionState? {
        desktopConversationSearchToolCompletionState
    }

    private var desktopTypedTurnComposerCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Conversation-first keyboard entry now produces one bounded desktop typed-turn request into the already-live canonical runtime path while transcript authority, authoritative acceptance, search/tool routing, and authoritative reply remain cloud-side.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .center, spacing: 12) {
                    Image(systemName: "text.cursor")
                        .font(.system(size: 26))
                        .foregroundStyle(.secondary)

                    VStack(alignment: .leading, spacing: 4) {
                        Text("Keyboard composer")
                            .font(.headline)

                        Text("Bounded typed request production only. This surface reuses the already-live canonical `/v1/voice/turn` carrier and does not invent a local assistant, local search execution, or local tool authority.")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }

                    Spacer()

                    Text(keyboardComposerPendingTypedTurnRequest == nil ? "Ready" : "Dispatching")
                        .font(.caption.weight(.semibold))
                        .padding(.horizontal, 10)
                        .padding(.vertical, 6)
                        .background(
                            keyboardComposerPendingTypedTurnRequest == nil
                                ? Color.secondary.opacity(0.12)
                                : Color.accentColor.opacity(0.16)
                        )
                        .clipShape(Capsule())
                }

                HStack(spacing: 8) {
                    posturePill("EXPLICIT_ONLY")
                    posturePill("Keyboard typed turn")
                    posturePill("text/plain")
                }

                HStack(spacing: 8) {
                    posturePill("Cloud authoritative")
                    posturePill("Session-bound")
                    posturePill("No local authority")
                }

                TextField(
                    "Type a follow-up for canonical text-turn ingress.",
                    text: $desktopTypedTurnDraft
                )
                .textFieldStyle(.roundedBorder)
                .disabled(desktopTypedTurnSubmissionInterlocksActive)
                .onSubmit {
                    submitDesktopTypedTurn()
                }

                HStack(spacing: 12) {
                    Button("Send typed turn") {
                        submitDesktopTypedTurn()
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(trimmedDesktopTypedTurnDraft.isEmpty || desktopTypedTurnSubmissionInterlocksActive)

                    Button("Clear draft") {
                        desktopTypedTurnDraft = ""
                        desktopTypedTurnFailedRequest = nil
                    }
                    .buttonStyle(.bordered)
                    .disabled(desktopTypedTurnDraft.isEmpty && desktopTypedTurnFailedRequest == nil)
                }

                Text("Draft validation: trimmed non-empty text only, canonical `text/plain`, \(trimmedDesktopTypedTurnDraft.utf8.count) / \(maxDesktopTypedTurnBytes) UTF-8 bytes.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                if desktopTypedTurnSubmissionInterlocksActive {
                    Text("Typed-turn production stays single-request only while another foreground capture or canonical voice-turn dispatch posture remains active.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                if let pendingRequest = keyboardComposerPendingTypedTurnRequest {
                    desktopTypedTurnPendingRequestCard(pendingRequest)
                }

                if let failedRequest = desktopTypedTurnFailedRequest {
                    interruptResponseFailedRequestCard(failedRequest)
                }

                Text("This composer itself still does not introduce local search execution, local provider selection, direct tool-name authority, shell-side `projectID` or `pinnedContextRefs` transport, hidden/background wake behavior, or autonomous unlock.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Keyboard Composer")
                .font(.headline)
        }
    }

    private var desktopToolRequestAuthoringCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Bounded tool-request authoring only. This surface stays tool-lane-adjacent, reuses the already-live canonical `/v1/voice/turn` typed-turn carrier, and still leaves tool-routing authority, provider selection, and final dispatch posture entirely cloud-side.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .center, spacing: 12) {
                    Image(systemName: "wrench.and.screwdriver")
                        .font(.system(size: 24))
                        .foregroundStyle(.secondary)

                    VStack(alignment: .leading, spacing: 4) {
                        Text("Tool-request authoring")
                            .font(.headline)

                        Text("One bounded tool-oriented request only. This surface does not create a standalone search box, direct tool-name authority, local provider picking, or local search execution.")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }

                    Spacer()

                    Text(
                        toolRequestPendingTypedTurnRequest == nil
                            ? (desktopToolRequestCardIsExecutable ? "Ready" : "Read-only")
                            : "Dispatching"
                    )
                    .font(.caption.weight(.semibold))
                    .padding(.horizontal, 10)
                    .padding(.vertical, 6)
                    .background(
                        toolRequestPendingTypedTurnRequest == nil
                            ? (desktopToolRequestCardIsExecutable
                                ? Color.secondary.opacity(0.12)
                                : Color.orange.opacity(0.16))
                            : Color.accentColor.opacity(0.16)
                    )
                    .clipShape(Capsule())
                }

                HStack(spacing: 8) {
                    posturePill("Tool-lane adjacent")
                    posturePill("Canonical /v1/voice/turn")
                    posturePill("No direct tool authority")
                }

                HStack(spacing: 8) {
                    posturePill("Cloud authoritative")
                    posturePill("Session-bound")
                    posturePill("text/plain")
                }

                if let readOnlyToolLaneState = desktopCurrentReadOnlyToolLaneState {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Current cloud-authored tool-lane context")
                            .font(.subheadline.weight(.semibold))

                        metadataRow(label: "lane_kind", value: readOnlyToolLaneState.laneKind)
                        metadataRow(label: "outcome", value: readOnlyToolLaneState.outcome)
                        metadataRow(label: "next_move", value: readOnlyToolLaneState.nextMove)
                        metadataRow(label: "reason_code", value: readOnlyToolLaneState.reasonCode)
                    }
                } else {
                    Text("No cloud-authored tool-lane attachment is currently foregrounded. You can still author one bounded tool-oriented request below so canonical runtime can decide whether tool dispatch is lawful.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                if desktopToolRequestCardIsExecutable {
                    TextField(
                        "Author one bounded tool-oriented request for canonical routing.",
                        text: $desktopToolRequestDraft
                    )
                    .textFieldStyle(.roundedBorder)
                    .disabled(desktopTypedTurnSubmissionInterlocksActive)
                    .onSubmit {
                        submitDesktopToolRequest()
                    }

                    HStack(spacing: 12) {
                        Button("Execute tool request") {
                            submitDesktopToolRequest()
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(
                            trimmedDesktopToolRequestDraft.isEmpty
                                || desktopTypedTurnSubmissionInterlocksActive
                        )

                        Button("Clear tool request") {
                            desktopToolRequestDraft = ""
                            desktopToolRequestFailedRequest = nil
                        }
                        .buttonStyle(.bordered)
                        .disabled(
                            desktopToolRequestDraft.isEmpty
                                && desktopToolRequestFailedRequest == nil
                        )
                    }

                    Text("Draft validation: trimmed non-empty tool-oriented text only, canonical `text/plain`, \(trimmedDesktopToolRequestDraft.utf8.count) / \(maxDesktopTypedTurnBytes) UTF-8 bytes.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    if desktopTypedTurnSubmissionInterlocksActive {
                        Text("Tool-request production stays single-request only while another bounded typed-turn, explicit voice capture, wake-triggered request, or canonical voice-turn dispatch posture remains active.")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } else {
                    Text(desktopToolRequestReadOnlyDetail)
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                if let pendingRequest = toolRequestPendingTypedTurnRequest {
                    desktopTypedTurnPendingRequestCard(pendingRequest)
                }

                if let failedRequest = desktopToolRequestFailedRequest {
                    interruptResponseFailedRequestCard(failedRequest)
                }

                Text("This card still does not claim standalone local search execution, local provider selection, direct tool-name authority, shell-side `projectID` or `pinnedContextRefs` transport, hidden/background wake behavior, or autonomous unlock.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Tool Request")
                .font(.headline)
        }
    }

    private var desktopSearchRequestAuthoringCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Bounded search-request authoring only. This surface stays search-lane-adjacent, reuses the already-live canonical `/v1/voice/turn` typed-turn carrier, and still leaves search routing, provider choice, retrieval, and final dispatch posture entirely cloud-side.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .center, spacing: 12) {
                    Image(systemName: "magnifyingglass")
                        .font(.system(size: 24))
                        .foregroundStyle(.secondary)

                    VStack(alignment: .leading, spacing: 4) {
                        Text("Search-request authoring")
                            .font(.headline)

                        Text("One bounded search-oriented request only. This surface does not create local search execution, local provider picking, local source authority, or direct tool-name authority.")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }

                    Spacer()

                    Text(
                        searchRequestPendingTypedTurnRequest == nil
                            ? (desktopSearchRequestCardIsExecutable ? "Ready" : "Read-only")
                            : "Dispatching"
                    )
                    .font(.caption.weight(.semibold))
                    .padding(.horizontal, 10)
                    .padding(.vertical, 6)
                    .background(
                        searchRequestPendingTypedTurnRequest == nil
                            ? (desktopSearchRequestCardIsExecutable
                                ? Color.secondary.opacity(0.12)
                                : Color.orange.opacity(0.16))
                            : Color.accentColor.opacity(0.16)
                    )
                    .clipShape(Capsule())
                }

                HStack(spacing: 8) {
                    posturePill("Search-lane adjacent")
                    posturePill("Canonical /v1/voice/turn")
                    posturePill("No local search execution")
                }

                HStack(spacing: 8) {
                    posturePill("Cloud authoritative")
                    posturePill("Session-bound")
                    posturePill("text/plain")
                }

                if let searchToolCompletionState = desktopCurrentSearchToolCompletionState {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Current cloud-authored search context")
                            .font(.subheadline.weight(.semibold))

                        metadataRow(label: "outcome", value: searchToolCompletionState.outcome)
                        metadataRow(label: "next_move", value: searchToolCompletionState.nextMove)
                        metadataRow(label: "reason_code", value: searchToolCompletionState.reasonCode)
                        metadataRow(label: "source_count", value: String(searchToolCompletionState.sources.count))
                    }
                } else {
                    Text("No cloud-authored search completion attachment is currently foregrounded. You can still author one bounded search-oriented request below so canonical runtime can decide whether search dispatch is lawful.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                if desktopSearchRequestCardIsExecutable {
                    TextField(
                        "Author one bounded search-oriented request for canonical routing.",
                        text: $desktopSearchRequestDraft
                    )
                    .textFieldStyle(.roundedBorder)
                    .disabled(desktopTypedTurnSubmissionInterlocksActive)
                    .onSubmit {
                        submitDesktopSearchRequest()
                    }

                    HStack(spacing: 12) {
                        Button("Execute search request") {
                            submitDesktopSearchRequest()
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(
                            trimmedDesktopSearchRequestDraft.isEmpty
                                || desktopTypedTurnSubmissionInterlocksActive
                        )

                        Button("Clear search request") {
                            desktopSearchRequestDraft = ""
                            desktopSearchRequestFailedRequest = nil
                        }
                        .buttonStyle(.bordered)
                        .disabled(
                            desktopSearchRequestDraft.isEmpty
                                && desktopSearchRequestFailedRequest == nil
                        )
                    }

                    Text("Draft validation: trimmed non-empty search-oriented text only, canonical `text/plain`, \(trimmedDesktopSearchRequestDraft.utf8.count) / \(maxDesktopTypedTurnBytes) UTF-8 bytes.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    if desktopTypedTurnSubmissionInterlocksActive {
                        Text("Search-request production stays single-request only while another bounded typed-turn, explicit voice capture, wake-triggered request, or canonical voice-turn dispatch posture remains active.")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } else {
                    Text(desktopSearchRequestReadOnlyDetail)
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                if let pendingRequest = searchRequestPendingTypedTurnRequest {
                    desktopTypedTurnPendingRequestCard(pendingRequest)
                }

                if let failedRequest = desktopSearchRequestFailedRequest {
                    interruptResponseFailedRequestCard(failedRequest)
                }

                Text("This card still does not claim standalone local search execution, local provider selection, direct tool-name authority, shell-side `projectID` or `pinnedContextRefs` transport, hidden/background wake behavior, or autonomous unlock.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Search Request")
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
                    deviceID: nil,
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
                    deviceID: nil,
                    proofMaterial: "onboarding_session_id=\(presentationState.onboardingSessionID)|receipt_kind=mic_permission_granted|microphone_permission=granted|app_platform=DESKTOP",
                    proofSummary: "Locally provable from current granted microphone permission posture only."
                )
            )
        }

        if remainingKinds.contains("desktop_pairing_bound"),
           let deviceID = desktopManagedPrimaryDeviceID {
            drafts.append(
                DesktopPlatformSetupReceiptDraft(
                    onboardingSessionID: presentationState.onboardingSessionID,
                    receiptKind: "desktop_pairing_bound",
                    deviceID: deviceID,
                    proofMaterial: "onboarding_session_id=\(presentationState.onboardingSessionID)|receipt_kind=desktop_pairing_bound|device_id=\(deviceID)|pairing_binding=managed_bridge_device_id|app_platform=DESKTOP",
                    proofSummary: "Locally provable from the exact managed bridge `deviceID` plus the current bounded platform-setup posture only."
                )
            )
        }

        if remainingKinds.contains("desktop_wakeword_configured"),
           let wakewordProofContext = desktopWakewordConfiguredProofContext {
            drafts.append(
                DesktopPlatformSetupReceiptDraft(
                    onboardingSessionID: presentationState.onboardingSessionID,
                    receiptKind: "desktop_wakeword_configured",
                    deviceID: wakewordProofContext.deviceID,
                    proofMaterial: "receipt_kind=desktop_wakeword_configured|wake_profile_id=\(wakewordProofContext.wakeRuntimeEventWakeProfileID)|wake_accepted=\(wakewordProofContext.wakeRuntimeEventAccepted ? "true" : "false")|voice_receipt_ref=\(wakewordProofContext.voiceArtifactSyncReceiptRef)|app_platform=DESKTOP",
                    proofSummary: "Locally provable from the exact managed bridge `deviceID`, exact lawful wake runtime event evidence carrier family, exact `wake_runtime_event_wake_profile_id`, and exact bounded wake-enroll sync receipt visibility only.",
                    wakeRuntimeEventWakeProfileID: wakewordProofContext.wakeRuntimeEventWakeProfileID,
                    wakeRuntimeEventAccepted: wakewordProofContext.wakeRuntimeEventAccepted,
                    voiceArtifactSyncReceiptRef: wakewordProofContext.voiceArtifactSyncReceiptRef
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

    private var desktopSenderVerificationVisibilityState: DesktopSenderVerificationVisibilityState? {
        guard let desktopTermsAcceptRuntimeOutcomeState,
              desktopTermsAcceptRuntimeOutcomeState.phase == .completed,
              desktopTermsAcceptRuntimeOutcomeState.nextStep == "SENDER_VERIFICATION",
              let onboardingSessionID = desktopTermsAcceptRuntimeOutcomeState.onboardingSessionID
        else {
            return nil
        }

        return DesktopSenderVerificationVisibilityState(
            onboardingSessionID: onboardingSessionID,
            nextStep: "SENDER_VERIFICATION",
            onboardingStatus: desktopTermsAcceptRuntimeOutcomeState.onboardingStatus,
            requiredVerificationGates: desktopInviteOpenRuntimeOutcomeState?.requiredVerificationGates ?? []
        )
    }

    private var desktopEmployeePhotoCaptureSendPromptState: DesktopEmployeePhotoCaptureSendPromptState? {
        if let desktopEmployeePhotoCaptureSendRuntimeOutcomeState,
           desktopEmployeePhotoCaptureSendRuntimeOutcomeState.phase == .completed,
           desktopEmployeePhotoCaptureSendRuntimeOutcomeState.nextStep != "SENDER_VERIFICATION" {
            return nil
        }

        guard let visibilityState = desktopSenderVerificationVisibilityState else {
            return nil
        }

        return DesktopEmployeePhotoCaptureSendPromptState(
            onboardingSessionID: visibilityState.onboardingSessionID,
            nextStep: visibilityState.nextStep,
            onboardingStatus: visibilityState.onboardingStatus,
            requiredVerificationGates: visibilityState.requiredVerificationGates
        )
    }

    private var desktopEmployeeSenderVerifyCommitPromptState: DesktopEmployeeSenderVerifyCommitPromptState? {
        guard let desktopEmployeePhotoCaptureSendRuntimeOutcomeState,
              desktopEmployeePhotoCaptureSendRuntimeOutcomeState.phase == .completed,
              desktopEmployeePhotoCaptureSendRuntimeOutcomeState.nextStep == "SENDER_VERIFICATION",
              let onboardingSessionID = desktopEmployeePhotoCaptureSendRuntimeOutcomeState.onboardingSessionID,
              let photoBlobRef = desktopEmployeePhotoCaptureSendRuntimeOutcomeState.photoBlobRef
        else {
            return nil
        }

        return DesktopEmployeeSenderVerifyCommitPromptState(
            onboardingSessionID: onboardingSessionID,
            nextStep: "SENDER_VERIFICATION",
            onboardingStatus: desktopEmployeePhotoCaptureSendRuntimeOutcomeState.onboardingStatus,
            requiredVerificationGates: desktopSenderVerificationVisibilityState?.requiredVerificationGates ?? [],
            photoBlobRef: photoBlobRef
        )
    }

    private var boundedWakeEnrollCompletionLineageVoiceArtifactSyncReceiptRef: String? {
        [
            desktopCompleteCommitRuntimeOutcomeState?.voiceArtifactSyncReceiptRef,
            desktopAccessProvisionCommitRuntimeOutcomeState?.voiceArtifactSyncReceiptRef,
            desktopEmoPersonaLockRuntimeOutcomeState?.voiceArtifactSyncReceiptRef,
            desktopWakeEnrollCompleteCommitRuntimeOutcomeState?.voiceArtifactSyncReceiptRef,
        ]
        .compactMap { candidate in
            let trimmed = candidate?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
            return trimmed.isEmpty ? nil : trimmed
        }
        .first
    }

    private var desktopWakewordConfiguredProofContext: (
        deviceID: String,
        wakeRuntimeEventWakeProfileID: String,
        wakeRuntimeEventAccepted: Bool,
        voiceArtifactSyncReceiptRef: String
    )? {
        guard let deviceID = desktopManagedPrimaryDeviceID,
              let activeVisibleContext = latestSessionActiveVisibleContext,
              activeVisibleContext.hasLawfulWakeRuntimeEventEvidenceCarrierFamily,
              let wakeRuntimeEventAccepted = activeVisibleContext.wakeRuntimeEventAccepted,
              let wakeRuntimeEventWakeProfileID = activeVisibleContext.wakeRuntimeEventWakeProfileID?
                .trimmingCharacters(in: .whitespacesAndNewlines),
              !wakeRuntimeEventWakeProfileID.isEmpty,
              let voiceArtifactSyncReceiptRef = boundedWakeEnrollCompletionLineageVoiceArtifactSyncReceiptRef
        else {
            return nil
        }

        return (
            deviceID: deviceID,
            wakeRuntimeEventWakeProfileID: wakeRuntimeEventWakeProfileID,
            wakeRuntimeEventAccepted: wakeRuntimeEventAccepted,
            voiceArtifactSyncReceiptRef: voiceArtifactSyncReceiptRef
        )
    }

    private var desktopWakeProfileAvailabilityPromptState: DesktopWakeProfileAvailabilityPromptState? {
        guard desktopReadyTimeHandoffIsActive,
              let wakewordProofContext = desktopWakewordConfiguredProofContext else {
            return nil
        }

        return DesktopWakeProfileAvailabilityPromptState(
            receiptKind: "desktop_wakeword_configured",
            deviceID: wakewordProofContext.deviceID,
            wakeProfileID: wakewordProofContext.wakeRuntimeEventWakeProfileID,
            voiceArtifactSyncReceiptRef: wakewordProofContext.voiceArtifactSyncReceiptRef
        )
    }

    private var desktopWakeListenerPromptState: DesktopWakeListenerPromptState? {
        guard desktopReadyTimeHandoffIsActive,
              let wakewordProofContext = desktopWakewordConfiguredProofContext,
              let desktopWakeProfileAvailabilityRuntimeOutcomeState,
              desktopWakeProfileAvailabilityRuntimeOutcomeState.phase == .completed,
              desktopWakeProfileAvailabilityRuntimeOutcomeState.receiptKind == "desktop_wakeword_configured",
              desktopWakeProfileAvailabilityRuntimeOutcomeState.deviceID == wakewordProofContext.deviceID,
              desktopWakeProfileAvailabilityRuntimeOutcomeState.wakeProfileID
                == wakewordProofContext.wakeRuntimeEventWakeProfileID,
              desktopWakeProfileAvailabilityRuntimeOutcomeState.voiceArtifactSyncReceiptRef
                == wakewordProofContext.voiceArtifactSyncReceiptRef,
              let activeWakeArtifactVersion = desktopWakeProfileAvailabilityRuntimeOutcomeState
                .activeWakeArtifactVersion else {
            return nil
        }

        return DesktopWakeListenerPromptState(
            receiptKind: "desktop_wakeword_configured",
            deviceID: wakewordProofContext.deviceID,
            wakeProfileID: wakewordProofContext.wakeRuntimeEventWakeProfileID,
            activeWakeArtifactVersion: activeWakeArtifactVersion,
            voiceArtifactSyncReceiptRef: wakewordProofContext.voiceArtifactSyncReceiptRef,
            wakeTriggerPhrase: "Selene"
        )
    }

    private var desktopWakeAutoStartEligiblePromptState: DesktopWakeListenerPromptState? {
        guard scenePhase == .active,
              desktopOperationalConversationShellState != nil,
              let promptState = desktopWakeListenerPromptState,
              !explicitVoiceController.isListening,
              explicitVoiceController.pendingRequest == nil,
              desktopTypedTurnPendingRequest == nil,
              desktopWakeListenerController.listenerState == .idle,
              desktopWakeListenerController.pendingRequest == nil,
              lastStagedWakeTriggeredVoiceTurnRequestState == nil else {
            return nil
        }

        return promptState
    }

    private var desktopOperationalConversationShellState: DesktopOperationalConversationShellState? {
        guard let primaryPaneState = desktopConversationPrimaryPaneState,
              let supportRailState = desktopConversationSupportRailState else {
            return nil
        }

        return DesktopOperationalConversationShellState(
            primaryPaneState: primaryPaneState,
            supportRailState: supportRailState
        )
    }

    private var desktopConversationPrimaryPaneState: DesktopConversationPrimaryPaneState? {
        guard desktopReadyTimeHandoffIsActive else {
            return nil
        }

        let isShowingCurrentDominantSurface = desktopForegroundSelectionShowsCurrentDominantSurface
        let dominantPosture: String
        let headerTitle: String
        let headerDetail: String

        if foregroundSessionSuspendedVisibleContext != nil {
            dominantPosture = "SESSION_SUSPENDED_VISIBLE"
            headerTitle = "Conversation unavailable while the session is suspended"
            headerDetail = "This transcript-primary pane fails closed to explanation-only content while the authoritative runtime keeps the desktop session in a hard full takeover posture."
        } else if activeRecoveryDisplayState == .quarantinedLocalState {
            dominantPosture = "QUARANTINED_LOCAL_STATE"
            headerTitle = "Conversation unavailable while local state is quarantined"
            headerDetail = "This transcript-primary pane fails closed to explanation-only content until authoritative reread clears the quarantine posture cloud-side."
        } else if let foregroundSessionActiveVisibleContext {
            dominantPosture = foregroundSessionActiveVisibleContext.sessionState
            headerTitle = "Active conversation"
            headerDetail = isShowingCurrentDominantSurface
                ? "Cloud-authored active-session transcript, bounded runtime dispatch posture, and authoritative reply surfaces remain visible here without introducing local session authority."
                : "A previously observed active-session transcript surface is foregrounded here in bounded read-only form only; local selection does not retarget runtime mutation."
        } else if let foregroundSessionSoftClosedVisibleContext {
            dominantPosture = foregroundSessionSoftClosedVisibleContext.sessionState
            headerTitle = "Archived recent slice"
            headerDetail = "Soft-closed archive truth remains distinct from PH1.M resume context while explicit resume stays bounded and non-authoritative."
        } else if let foregroundSessionHeaderContext {
            dominantPosture = foregroundSessionHeaderContext.sessionState
            headerTitle = "Current session header"
            headerDetail = "Cloud-authored session-header visibility remains foregrounded here in bounded read-only form only while local selection stays non-authoritative."
        } else if let activeRecoveryDisplayState {
            dominantPosture = activeRecoveryDisplayState.rawValue
            headerTitle = "\(activeRecoveryDisplayState.rawValue) conversation posture"
            headerDetail = "The lawful main session surface remains visible while recovery restriction posture stays cloud-authored, bounded, and non-authoritative."
        } else if desktopCanonicalRuntimeOutcomeState != nil || desktopAuthoritativeReplyRenderState != nil {
            dominantPosture = "CANONICAL_RUNTIME_VISIBLE"
            headerTitle = "Operational conversation"
            headerDetail = "Canonical runtime outcome and cloud-authored reply posture remain visible in transcript-first form only."
        } else {
            dominantPosture = "READY_FOR_OPERATION"
            headerTitle = "Conversation ready"
            headerDetail = "Use the bounded keyboard composer, explicit voice, or the bounded foreground wake listener to start the next lawful cloud-authored turn from this transcript-primary shell."
        }

        var timelineEntries: [DesktopConversationTimelineEntryState] = []

        if let foregroundSessionActiveVisibleContext {
            timelineEntries.append(
                DesktopConversationTimelineEntryState(
                    speaker: "You",
                    posture: "current_user_turn_text",
                    body: foregroundSessionActiveVisibleContext.currentUserTurnText,
                    detail: "Current user turn remains text-visible, session-bound, and cloud-authoritative for the active desktop session.",
                    sourceSurface: "SESSION_ACTIVE_VISIBLE"
                )
            )
            timelineEntries.append(
                DesktopConversationTimelineEntryState(
                    speaker: "Selene",
                    posture: "current_selene_turn_text",
                    body: foregroundSessionActiveVisibleContext.currentSeleneTurnText,
                    detail: "Current Selene turn remains text-visible and tied to the same active cloud session without a local-only transcript fork.",
                    sourceSurface: "SESSION_ACTIVE_VISIBLE"
                )
            )
        } else if let foregroundSessionSoftClosedVisibleContext {
            timelineEntries.append(
                DesktopConversationTimelineEntryState(
                    speaker: "You",
                    posture: "archived_user_turn_text",
                    body: foregroundSessionSoftClosedVisibleContext.archivedUserTurnText,
                    detail: "Archived recent slice remains durable archived conversation truth and stays distinct from bounded PH1.M resume-context output.",
                    sourceSurface: "SESSION_SOFT_CLOSED_VISIBLE"
                )
            )
            timelineEntries.append(
                DesktopConversationTimelineEntryState(
                    speaker: "Selene",
                    posture: "archived_selene_turn_text",
                    body: foregroundSessionSoftClosedVisibleContext.archivedSeleneTurnText,
                    detail: "Archived recent slice remains text-visible after visual reset without local auto-reopen, hidden spoken-only output, or local transcript authority.",
                    sourceSurface: "SESSION_SOFT_CLOSED_VISIBLE"
                )
            )
        }

        if isShowingCurrentDominantSurface {
            let trimmedExplicitVoiceTranscriptPreview = explicitVoiceController.transcriptPreview
                .trimmingCharacters(in: .whitespacesAndNewlines)
            if explicitVoiceController.isListening,
               explicitVoiceController.pendingRequest == nil,
               !trimmedExplicitVoiceTranscriptPreview.isEmpty {
                timelineEntries.append(
                    DesktopConversationTimelineEntryState(
                        speaker: "You",
                        posture: "explicit_voice_live_preview",
                        body: trimmedExplicitVoiceTranscriptPreview,
                        detail: "Bounded explicit live transcript preview only. Canonical runtime acceptance and later cloud-visible response remain authoritative.",
                        sourceSurface: "EXPLICIT_VOICE_LISTENING"
                    )
                )
            }

            let trimmedWakeTranscriptPreview = desktopWakeListenerController.transcriptPreview
                .trimmingCharacters(in: .whitespacesAndNewlines)
            if desktopWakeListenerPromptState != nil,
               desktopWakeListenerController.listenerState == .listening,
               desktopWakeListenerController.pendingRequest == nil,
               lastStagedWakeTriggeredVoiceTurnRequestState == nil,
               !trimmedWakeTranscriptPreview.isEmpty {
                timelineEntries.append(
                    DesktopConversationTimelineEntryState(
                        speaker: "You",
                        posture: "wake_voice_live_preview",
                        body: trimmedWakeTranscriptPreview,
                        detail: "Bounded wake live transcript preview only. Exact wake-prefix detection, canonical runtime acceptance, and later cloud-visible response remain authoritative.",
                        sourceSurface: "WAKE_TRIGGERED_VOICE_LISTENING"
                    )
                )
            }

            if let pendingRequest = explicitVoiceController.pendingRequest {
                timelineEntries.append(
                    DesktopConversationTimelineEntryState(
                        speaker: "You",
                        posture: "explicit_voice_pending_preview",
                        body: pendingRequest.boundedPreview,
                        detail: "Bounded explicit voice pending preview only. Canonical runtime and later cloud-visible acceptance remain authoritative.",
                        sourceSurface: "EXPLICIT_VOICE_PENDING"
                    )
                )
            }

            if let pendingWakeRequest = desktopWakeListenerController.pendingRequest
                ?? lastStagedWakeTriggeredVoiceTurnRequestState {
                timelineEntries.append(
                    DesktopConversationTimelineEntryState(
                        speaker: "You",
                        posture: "wake_voice_pending_preview",
                        body: pendingWakeRequest.boundedPreview,
                        detail: "Bounded post-wake transcript remainder only. This foreground wake-trigger preview remains non-authoritative until canonical runtime returns.",
                        sourceSurface: "WAKE_TRIGGERED_VOICE_PENDING"
                    )
                )
            }

            if let failedRequest = explicitVoiceController.failedRequest,
               !explicitVoiceController.isListening,
               explicitVoiceController.pendingRequest == nil {
                timelineEntries.append(
                    DesktopConversationTimelineEntryState(
                        speaker: "You",
                        posture: "explicit_voice_failed_request_preview",
                        body: failedRequest.summary,
                        detail: "Bounded explicit local failure visibility only. Canonical runtime acceptance, transcript authority, and later cloud-visible response remain authoritative.",
                        sourceSurface: "EXPLICIT_VOICE_FAILED_REQUEST"
                    )
                )
            }

            if desktopWakeListenerPromptState != nil,
               let failedWakeRequest = desktopWakeListenerController.failedRequest,
               desktopWakeListenerController.listenerState == .failed,
               desktopWakeListenerController.pendingRequest == nil,
               lastStagedWakeTriggeredVoiceTurnRequestState == nil {
                timelineEntries.append(
                    DesktopConversationTimelineEntryState(
                        speaker: "You",
                        posture: "wake_voice_failed_request_preview",
                        body: failedWakeRequest.summary,
                        detail: "Bounded wake local failure visibility only. Wake authority, canonical runtime acceptance, and later cloud-visible response remain authoritative.",
                        sourceSurface: "WAKE_TRIGGERED_VOICE_FAILED_REQUEST"
                    )
                )
            }

            if let pendingTypedTurnRequest = desktopTypedTurnPendingRequest {
                timelineEntries.append(
                    DesktopConversationTimelineEntryState(
                        speaker: "You",
                        posture: pendingTypedTurnRequest.origin.timelinePendingPosture,
                        body: pendingTypedTurnRequest.boundedPreview,
                        detail: pendingTypedTurnRequest.origin.timelinePendingDetail,
                        sourceSurface: pendingTypedTurnRequest.origin.pendingSourceSurface
                    )
                )
            }

            if let failedTypedTurnRequest = desktopTypedTurnFailedRequest,
               desktopTypedTurnPendingRequest == nil {
                timelineEntries.append(
                    DesktopConversationTimelineEntryState(
                        speaker: "You",
                        posture: "typed_turn_failed_request_preview",
                        body: failedTypedTurnRequest.summary,
                        detail: DesktopTypedTurnRequestOrigin.keyboardComposer.timelineFailedDetail,
                        sourceSurface: DesktopTypedTurnRequestOrigin.keyboardComposer.failedSourceSurface
                    )
                )
            }

            if let failedSearchRequest = desktopSearchRequestFailedRequest,
               desktopTypedTurnPendingRequest == nil {
                timelineEntries.append(
                    DesktopConversationTimelineEntryState(
                        speaker: "You",
                        posture: DesktopTypedTurnRequestOrigin.searchRequestCard.timelineFailedPosture,
                        body: failedSearchRequest.summary,
                        detail: DesktopTypedTurnRequestOrigin.searchRequestCard.timelineFailedDetail,
                        sourceSurface: DesktopTypedTurnRequestOrigin.searchRequestCard.failedSourceSurface
                    )
                )
            }

            if let failedToolRequest = desktopToolRequestFailedRequest,
               desktopTypedTurnPendingRequest == nil {
                timelineEntries.append(
                    DesktopConversationTimelineEntryState(
                        speaker: "You",
                        posture: DesktopTypedTurnRequestOrigin.toolRequestCard.timelineFailedPosture,
                        body: failedToolRequest.summary,
                        detail: DesktopTypedTurnRequestOrigin.toolRequestCard.timelineFailedDetail,
                        sourceSurface: DesktopTypedTurnRequestOrigin.toolRequestCard.failedSourceSurface
                    )
                )
            }
        }

        let authoritativeResponseText = isShowingCurrentDominantSurface
            ? desktopAuthoritativeReplyRenderState?.authoritativeResponseText?
                .trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
            : ""
        let searchToolCompletionState = isShowingCurrentDominantSurface
            ? desktopConversationSearchToolCompletionState
            : nil
        let readOnlyToolLaneState = searchToolCompletionState?.readOnlyToolLaneState
        let authoritativeReplyCompletionState = isShowingCurrentDominantSurface
            ? desktopConversationAuthoritativeReplyCompletionState
            : nil
        let runtimeDispatchFailureAttachmentState = isShowingCurrentDominantSurface
            ? desktopConversationRuntimeDispatchFailureAttachmentState
            : nil
        if let runtimeDispatchFailureAttachmentState {
            timelineEntries.append(
                DesktopConversationTimelineEntryState(
                    speaker: "Selene",
                    posture: "runtime_dispatch_failure_preview",
                    body: runtimeDispatchFailureAttachmentState.summary,
                    detail: "Bounded canonical runtime dispatch/failure visibility only. Already-live runtime carriers remain read-only, non-authoritative, and do not add local session or tool authority.",
                    sourceSurface: "CANONICAL_RUNTIME_DISPATCH_FAILURE"
                )
            )
        }

        if !authoritativeResponseText.isEmpty,
           !desktopConversationShouldSuppressDedicatedAuthoritativeReplyTextEntry(
               timelineEntries,
               authoritativeResponseText: authoritativeResponseText
           ) {
            timelineEntries.append(
                DesktopConversationTimelineEntryState(
                    speaker: "Selene",
                    posture: "authoritative_reply_text",
                    body: authoritativeResponseText,
                    detail: "Cloud-authored authoritative reply text only. This shell does not fabricate local answer content.",
                    sourceSurface: "CANONICAL_RUNTIME_COMPLETED"
                )
            )
        }

        if isShowingCurrentDominantSurface,
           desktopConversationShouldAttachRuntimeCompletedWithoutInlineReply(
               desktopCanonicalRuntimeOutcomeState,
               searchToolCompletionState: searchToolCompletionState,
               authoritativeReplyCompletionState: authoritativeReplyCompletionState
           ),
           let outcomeState = desktopCanonicalRuntimeOutcomeState {
            timelineEntries.append(
                DesktopConversationTimelineEntryState(
                    speaker: "Selene",
                    posture: "runtime_completed_without_inline_reply_preview",
                    body: outcomeState.summary,
                    detail: "Bounded canonical runtime completed visibility only when no lawful inline reply attachment is present. Already-live runtime carriers remain read-only, non-authoritative, and do not add local session authority, local search input or execution, local tool invocation or provider selection, wake-listener authority, hidden/background wake behavior, or autonomous unlock.",
                    sourceSurface: "CANONICAL_RUNTIME_COMPLETED_WITHOUT_INLINE_REPLY"
                )
            )
        }

        let explicitVoiceLivePreviewAttachmentState =
            desktopConversationExplicitVoiceLivePreviewAttachmentState(for: timelineEntries)
        let wakeTriggeredVoiceLivePreviewAttachmentState =
            desktopConversationWakeTriggeredVoiceLivePreviewAttachmentState(for: timelineEntries)
        let explicitVoiceFailedRequestAttachmentState =
            desktopConversationExplicitVoiceFailedRequestAttachmentState(for: timelineEntries)
        let wakeTriggeredVoiceFailedRequestAttachmentState =
            desktopConversationWakeTriggeredVoiceFailedRequestAttachmentState(for: timelineEntries)
        let explicitVoicePendingAttachmentState =
            desktopConversationExplicitVoicePendingAttachmentState(for: timelineEntries)
        let wakeTriggeredVoicePendingAttachmentState =
            desktopConversationWakeTriggeredVoicePendingAttachmentState(for: timelineEntries)

        return DesktopConversationPrimaryPaneState(
            dominantPosture: dominantPosture,
            headerTitle: headerTitle,
            headerDetail: headerDetail,
            voiceState: desktopOperationalVoiceStateLabel,
            timelineEntries: timelineEntries,
            explicitVoiceLivePreviewAttachmentState: explicitVoiceLivePreviewAttachmentState,
            wakeTriggeredVoiceLivePreviewAttachmentState: wakeTriggeredVoiceLivePreviewAttachmentState,
            explicitVoiceFailedRequestAttachmentState: explicitVoiceFailedRequestAttachmentState,
            wakeTriggeredVoiceFailedRequestAttachmentState: wakeTriggeredVoiceFailedRequestAttachmentState,
            explicitVoicePendingAttachmentState: explicitVoicePendingAttachmentState,
            wakeTriggeredVoicePendingAttachmentState: wakeTriggeredVoicePendingAttachmentState,
            readOnlyToolLaneState: readOnlyToolLaneState,
            searchToolCompletionState: searchToolCompletionState,
            authoritativeReplyCompletionState: authoritativeReplyCompletionState,
            runtimeDispatchFailureAttachmentState: runtimeDispatchFailureAttachmentState
        )
    }

    private var desktopConversationSearchToolCompletionState: DesktopConversationSearchToolCompletionState? {
        guard desktopReadyTimeHandoffIsActive,
              desktopForegroundSelectionShowsCurrentDominantSurface,
              foregroundSessionSuspendedVisibleContext == nil,
              activeRecoveryDisplayState != .quarantinedLocalState,
              let outcomeState = desktopCanonicalRuntimeOutcomeState,
              outcomeState.phase == .completed,
              let authoritativeResponseText = desktopAuthoritativeReplyRenderState?.authoritativeResponseText?
                .trimmingCharacters(in: .whitespacesAndNewlines),
              !authoritativeResponseText.isEmpty else {
            return nil
        }

        let outcome = outcomeState.outcome ?? "not_available"
        let nextMove = outcomeState.nextMove ?? "not_available"
        guard outcome == "FINAL_TOOL" || nextMove == "dispatch_tool" else {
            return nil
        }

        let provenanceSources = desktopAuthoritativeReplyProvenanceRenderState?.sources.map {
            DesktopConversationReadOnlyToolLaneState.Source(
                title: $0.title,
                url: $0.url
            )
        } ?? []
        let readOnlyToolLaneState = DesktopConversationReadOnlyToolLaneState(
            laneKind: "READ_ONLY_TOOL",
            responseSurface: "CONVERSATION_PRIMARY_PANE_TOOL_ATTACHMENT",
            outcome: outcome,
            nextMove: nextMove,
            reasonCode: outcomeState.reasonCode ?? "not_available",
            sourceCount: provenanceSources.count,
            retrievedAtLabel: desktopAuthoritativeReplyProvenanceRenderState?.retrievedAtLabel,
            cacheStatusLabel: desktopAuthoritativeReplyProvenanceRenderState?.cacheStatusLabel,
            sources: provenanceSources
        )
        let searchToolCompletionSources = provenanceSources.map {
            DesktopConversationSearchToolCompletionState.Source(
                title: $0.title,
                url: $0.url
            )
        }

        return DesktopConversationSearchToolCompletionState(
            dispatchPhase: outcomeState.phase.rawValue,
            requestID: outcomeState.requestID,
            endpoint: outcomeState.endpoint,
            outcome: outcome,
            nextMove: nextMove,
            reasonCode: outcomeState.reasonCode ?? "not_available",
            sessionID: outcomeState.sessionID ?? "not_available",
            turnID: outcomeState.turnID ?? "not_available",
            authoritativeResponseText: authoritativeResponseText,
            retrievedAtLabel: desktopAuthoritativeReplyProvenanceRenderState?.retrievedAtLabel,
            cacheStatusLabel: desktopAuthoritativeReplyProvenanceRenderState?.cacheStatusLabel,
            sources: searchToolCompletionSources,
            readOnlyToolLaneState: readOnlyToolLaneState,
            playbackPhase: desktopAuthoritativeReplyPlaybackState.phase.rawValue,
            playbackTitle: desktopAuthoritativeReplyPlaybackState.title,
            playbackSummary: desktopAuthoritativeReplyPlaybackState.summary,
            playbackDetail: desktopAuthoritativeReplyPlaybackState.detail
        )
    }

    private var desktopConversationAuthoritativeReplyCompletionState: DesktopConversationAuthoritativeReplyCompletionState? {
        guard desktopReadyTimeHandoffIsActive,
              desktopForegroundSelectionShowsCurrentDominantSurface,
              foregroundSessionSuspendedVisibleContext == nil,
              activeRecoveryDisplayState != .quarantinedLocalState,
              let outcomeState = desktopCanonicalRuntimeOutcomeState,
              outcomeState.phase == .completed,
              let authoritativeResponseText = desktopAuthoritativeReplyRenderState?.authoritativeResponseText?
                .trimmingCharacters(in: .whitespacesAndNewlines),
              !authoritativeResponseText.isEmpty,
              desktopConversationSearchToolCompletionState == nil else {
            return nil
        }

        let authoritativeReplySources = desktopAuthoritativeReplyProvenanceRenderState?.sources.map {
            DesktopConversationAuthoritativeReplyCompletionState.Source(
                title: $0.title,
                url: $0.url
            )
        } ?? []

        return DesktopConversationAuthoritativeReplyCompletionState(
            dispatchPhase: outcomeState.phase.rawValue,
            requestID: outcomeState.requestID,
            endpoint: outcomeState.endpoint,
            outcome: outcomeState.outcome ?? "not_available",
            nextMove: outcomeState.nextMove ?? "not_available",
            reasonCode: outcomeState.reasonCode ?? "not_available",
            sessionID: outcomeState.sessionID ?? "not_available",
            turnID: outcomeState.turnID ?? "not_available",
            authoritativeResponseText: authoritativeResponseText,
            retrievedAtLabel: desktopAuthoritativeReplyProvenanceRenderState?.retrievedAtLabel,
            cacheStatusLabel: desktopAuthoritativeReplyProvenanceRenderState?.cacheStatusLabel,
            sources: authoritativeReplySources,
            playbackPhase: desktopAuthoritativeReplyPlaybackState.phase.rawValue,
            playbackTitle: desktopAuthoritativeReplyPlaybackState.title,
            playbackSummary: desktopAuthoritativeReplyPlaybackState.summary,
            playbackDetail: desktopAuthoritativeReplyPlaybackState.detail
        )
    }

    private var desktopConversationRuntimeDispatchFailureAttachmentState: DesktopConversationRuntimeDispatchFailureAttachmentState? {
        guard desktopReadyTimeHandoffIsActive,
              desktopForegroundSelectionShowsCurrentDominantSurface,
              foregroundSessionSuspendedVisibleContext == nil,
              activeRecoveryDisplayState != .quarantinedLocalState,
              let outcomeState = desktopCanonicalRuntimeOutcomeState,
              outcomeState.phase == .dispatching || outcomeState.phase == .failed,
              desktopConversationSearchToolCompletionState == nil,
              desktopConversationAuthoritativeReplyCompletionState == nil else {
            return nil
        }

        return DesktopConversationRuntimeDispatchFailureAttachmentState(
            dispatchPhase: outcomeState.phase.rawValue,
            requestID: outcomeState.requestID,
            endpoint: outcomeState.endpoint,
            outcome: outcomeState.outcome ?? "not_available",
            nextMove: outcomeState.nextMove ?? "not_available",
            reasonCode: outcomeState.reasonCode ?? "not_available",
            failureClass: outcomeState.failureClass ?? "not_available",
            sessionID: outcomeState.sessionID ?? "not_available",
            turnID: outcomeState.turnID ?? "not_available",
            summary: outcomeState.summary,
            detail: outcomeState.detail
        )
    }

    private func desktopConversationExplicitVoiceLivePreviewAttachmentState(
        for timelineEntries: [DesktopConversationTimelineEntryState]
    ) -> DesktopConversationExplicitVoiceLivePreviewAttachmentState? {
        let trimmedExplicitVoiceTranscriptPreview = explicitVoiceController.transcriptPreview
            .trimmingCharacters(in: .whitespacesAndNewlines)
        guard desktopReadyTimeHandoffIsActive,
              foregroundSessionSuspendedVisibleContext == nil,
              activeRecoveryDisplayState != .quarantinedLocalState,
              explicitVoiceController.isListening,
              explicitVoiceController.pendingRequest == nil,
              !trimmedExplicitVoiceTranscriptPreview.isEmpty,
              timelineEntries.contains(where: { entry in
                  entry.posture == "explicit_voice_live_preview"
                      && entry.sourceSurface == "EXPLICIT_VOICE_LISTENING"
              }) else {
            return nil
        }

        return DesktopConversationExplicitVoiceLivePreviewAttachmentState(
            sourceSurface: "EXPLICIT_VOICE_LISTENING",
            captureState: "foreground_listening",
            captureMode: "foreground_only",
            transcriptPosture: "non_authoritative_live_preview",
            transcriptBytes: "\(trimmedExplicitVoiceTranscriptPreview.utf8.count)"
        )
    }

    private func desktopConversationWakeTriggeredVoiceLivePreviewAttachmentState(
        for timelineEntries: [DesktopConversationTimelineEntryState]
    ) -> DesktopConversationWakeTriggeredVoiceLivePreviewAttachmentState? {
        let trimmedWakeTranscriptPreview = desktopWakeListenerController.transcriptPreview
            .trimmingCharacters(in: .whitespacesAndNewlines)
        guard desktopReadyTimeHandoffIsActive,
              foregroundSessionSuspendedVisibleContext == nil,
              activeRecoveryDisplayState != .quarantinedLocalState,
              let wakeListenerPromptState = desktopWakeListenerPromptState,
              desktopWakeListenerController.listenerState == .listening,
              desktopWakeListenerController.pendingRequest == nil,
              lastStagedWakeTriggeredVoiceTurnRequestState == nil,
              !trimmedWakeTranscriptPreview.isEmpty,
              timelineEntries.contains(where: { entry in
                  entry.posture == "wake_voice_live_preview"
                      && entry.sourceSurface == "WAKE_TRIGGERED_VOICE_LISTENING"
              }) else {
            return nil
        }

        return DesktopConversationWakeTriggeredVoiceLivePreviewAttachmentState(
            sourceSurface: "WAKE_TRIGGERED_VOICE_LISTENING",
            listenerState: desktopWakeListenerController.listenerState.rawValue,
            wakeTriggerPhrase: wakeListenerPromptState.wakeTriggerPhrase,
            transcriptPosture: "non_authoritative_live_preview",
            transcriptBytes: "\(trimmedWakeTranscriptPreview.utf8.count)"
        )
    }

    private func desktopConversationExplicitVoiceFailedRequestAttachmentState(
        for timelineEntries: [DesktopConversationTimelineEntryState]
    ) -> DesktopConversationExplicitVoiceFailedRequestAttachmentState? {
        guard desktopReadyTimeHandoffIsActive,
              foregroundSessionSuspendedVisibleContext == nil,
              activeRecoveryDisplayState != .quarantinedLocalState,
              let failedRequest = explicitVoiceController.failedRequest,
              !explicitVoiceController.isListening,
              explicitVoiceController.pendingRequest == nil,
              timelineEntries.contains(where: { entry in
                  entry.posture == "explicit_voice_failed_request_preview"
                      && entry.sourceSurface == "EXPLICIT_VOICE_FAILED_REQUEST"
              }) else {
            return nil
        }

        return DesktopConversationExplicitVoiceFailedRequestAttachmentState(
            failureID: failedRequest.id,
            sourceSurface: "EXPLICIT_VOICE_FAILED_REQUEST",
            failureTitle: failedRequest.title,
            failureSummary: failedRequest.summary,
            failureDetail: failedRequest.detail
        )
    }

    private func desktopConversationWakeTriggeredVoiceFailedRequestAttachmentState(
        for timelineEntries: [DesktopConversationTimelineEntryState]
    ) -> DesktopConversationWakeTriggeredVoiceFailedRequestAttachmentState? {
        guard desktopReadyTimeHandoffIsActive,
              foregroundSessionSuspendedVisibleContext == nil,
              activeRecoveryDisplayState != .quarantinedLocalState,
              let wakeListenerPromptState = desktopWakeListenerPromptState,
              let failedRequest = desktopWakeListenerController.failedRequest,
              desktopWakeListenerController.listenerState == .failed,
              desktopWakeListenerController.pendingRequest == nil,
              lastStagedWakeTriggeredVoiceTurnRequestState == nil,
              timelineEntries.contains(where: { entry in
                  entry.posture == "wake_voice_failed_request_preview"
                      && entry.sourceSurface == "WAKE_TRIGGERED_VOICE_FAILED_REQUEST"
              }) else {
            return nil
        }

        return DesktopConversationWakeTriggeredVoiceFailedRequestAttachmentState(
            failureID: failedRequest.id,
            sourceSurface: "WAKE_TRIGGERED_VOICE_FAILED_REQUEST",
            listenerState: desktopWakeListenerController.listenerState.rawValue,
            wakeTriggerPhrase: wakeListenerPromptState.wakeTriggerPhrase,
            failureTitle: failedRequest.title,
            failureSummary: failedRequest.summary,
            failureDetail: failedRequest.detail
        )
    }

    private func desktopConversationExplicitVoicePendingAttachmentState(
        for timelineEntries: [DesktopConversationTimelineEntryState]
    ) -> DesktopConversationExplicitVoicePendingAttachmentState? {
        guard desktopReadyTimeHandoffIsActive,
              foregroundSessionSuspendedVisibleContext == nil,
              activeRecoveryDisplayState != .quarantinedLocalState,
              let pendingRequest = explicitVoiceController.pendingRequest,
              timelineEntries.contains(where: { entry in
                  entry.posture == "explicit_voice_pending_preview"
                      && entry.sourceSurface == "EXPLICIT_VOICE_PENDING"
              }) else {
            return nil
        }

        return DesktopConversationExplicitVoicePendingAttachmentState(
            requestID: pendingRequest.id,
            sourceSurface: "EXPLICIT_VOICE_PENDING",
            captureMode: "foreground_only",
            transcriptPosture: "non_authoritative_preview",
            transcriptBytes: "\(pendingRequest.byteCount)",
            selectedMic: pendingRequest.audioCaptureRefState.selectedMic,
            selectedSpeaker: pendingRequest.audioCaptureRefState.selectedSpeaker,
            deviceRoute: pendingRequest.audioCaptureRefState.deviceRoute,
            localeTag: pendingRequest.audioCaptureRefState.localeTag,
            ttsPlaybackActive: pendingRequest.audioCaptureRefState.ttsPlaybackActive ? "true" : "false",
            captureDegraded: pendingRequest.audioCaptureRefState.captureDegraded ? "true" : "false",
            streamGapDetected: pendingRequest.audioCaptureRefState.streamGapDetected ? "true" : "false",
            deviceChanged: pendingRequest.audioCaptureRefState.deviceChanged ? "true" : "false"
        )
    }

    private func desktopConversationWakeTriggeredVoicePendingAttachmentState(
        for timelineEntries: [DesktopConversationTimelineEntryState]
    ) -> DesktopConversationWakeTriggeredVoicePendingAttachmentState? {
        guard desktopReadyTimeHandoffIsActive,
              foregroundSessionSuspendedVisibleContext == nil,
              activeRecoveryDisplayState != .quarantinedLocalState,
              let pendingRequest = desktopWakeListenerController.pendingRequest
                ?? lastStagedWakeTriggeredVoiceTurnRequestState,
              timelineEntries.contains(where: { entry in
                  entry.posture == "wake_voice_pending_preview"
                      && entry.sourceSurface == "WAKE_TRIGGERED_VOICE_PENDING"
              }) else {
            return nil
        }

        return DesktopConversationWakeTriggeredVoicePendingAttachmentState(
            requestID: pendingRequest.id,
            sourceSurface: "WAKE_TRIGGERED_VOICE_PENDING",
            wakeTriggerPhrase: pendingRequest.wakeTriggerPhrase,
            transcriptPosture: "non_authoritative_preview",
            transcriptBytes: "\(pendingRequest.byteCount)",
            selectedMic: pendingRequest.audioCaptureRefState.selectedMic,
            selectedSpeaker: pendingRequest.audioCaptureRefState.selectedSpeaker,
            deviceRoute: pendingRequest.audioCaptureRefState.deviceRoute,
            localeTag: pendingRequest.audioCaptureRefState.localeTag,
            ttsPlaybackActive: pendingRequest.audioCaptureRefState.ttsPlaybackActive ? "true" : "false",
            captureDegraded: pendingRequest.audioCaptureRefState.captureDegraded ? "true" : "false",
            streamGapDetected: pendingRequest.audioCaptureRefState.streamGapDetected ? "true" : "false",
            deviceChanged: pendingRequest.audioCaptureRefState.deviceChanged ? "true" : "false"
        )
    }

    private var desktopConversationSupportRailState: DesktopConversationSupportRailState? {
        guard desktopReadyTimeHandoffIsActive else {
            return nil
        }

        return DesktopConversationSupportRailState(
            title: "Operational controls and status",
            detail: "Support surfaces remain bounded, session-bound, and non-authoritative while one local observed-session selection rail can foreground already-seen cloud-authored surfaces, one bounded current-device recent-session visibility card can render already-live upstream recent-session metadata in read-only form, one bounded search-request authoring card can reuse the already-live canonical voice-turn carrier, and one bounded tool-request authoring card remains tool-lane-adjacent.",
            supportSurfaceLabels: [
                "session_surface_selection_rail",
                "recent_session_visibility",
                "search_request_authoring",
                "tool_request_authoring",
                "posture_panel",
                "history",
                "session_multi_posture_entry",
                "explicit_voice_entry_affordance",
                "wake_profile_local_availability",
                "wake_listener_control",
                "session_soft_closed_visibility",
                "session_suspended_visibility",
                "recovery_visibility",
                "interrupt_visibility",
                "interrupt_response_production",
                "interrupt_subject_references_visibility",
                "interrupt_subject_relation_confidence_visibility",
                "interrupt_return_check_expiry_visibility",
                "session",
                "system_activity",
                "needs_attention",
            ]
        )
    }

    private var desktopOperationalVoiceStateLabel: String {
        if desktopAuthoritativeReplyPlaybackState.phase == .speaking {
            return "SPEAKING"
        }

        if desktopWakeListenerController.listenerState == .dispatching
            || desktopCanonicalRuntimeOutcomeState?.phase == .dispatching {
            return "DISPATCHING"
        }

        if desktopWakeListenerController.pendingRequest != nil
            || lastStagedWakeTriggeredVoiceTurnRequestState != nil
            || desktopWakeListenerController.listenerState == .wakeRequestStaged {
            return "WAKE_PENDING"
        }

        if desktopWakeListenerController.listenerState == .listening {
            return "WAKE_LISTENING"
        }

        if explicitVoiceController.pendingRequest != nil {
            return "EXPLICIT_PENDING"
        }

        if explicitVoiceController.isListening {
            return "EXPLICIT_LISTENING"
        }

        return "IDLE"
    }

    private var desktopPrimaryDeviceConfirmPromptState: DesktopPrimaryDeviceConfirmPromptState? {
        if let desktopPrimaryDeviceConfirmRuntimeOutcomeState,
           desktopPrimaryDeviceConfirmRuntimeOutcomeState.phase == .completed,
           desktopPrimaryDeviceConfirmRuntimeOutcomeState.nextStep != "PRIMARY_DEVICE_CONFIRM" {
            return nil
        }

        if let desktopEmployeeSenderVerifyCommitRuntimeOutcomeState,
           desktopEmployeeSenderVerifyCommitRuntimeOutcomeState.phase == .completed,
           desktopEmployeeSenderVerifyCommitRuntimeOutcomeState.nextStep == "PRIMARY_DEVICE_CONFIRM",
           let onboardingSessionID = desktopEmployeeSenderVerifyCommitRuntimeOutcomeState.onboardingSessionID,
           let deviceID = desktopManagedPrimaryDeviceID {
            return DesktopPrimaryDeviceConfirmPromptState(
                onboardingSessionID: onboardingSessionID,
                nextStep: "PRIMARY_DEVICE_CONFIRM",
                deviceID: deviceID,
                proofOK: true
            )
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

    private var desktopWakeEnrollDeferCommitPromptState: DesktopWakeEnrollDeferCommitPromptState? {
        if let desktopWakeEnrollCompleteCommitRuntimeOutcomeState,
           desktopWakeEnrollCompleteCommitRuntimeOutcomeState.phase == .completed {
            guard desktopWakeEnrollCompleteCommitRuntimeOutcomeState.nextStep == "WAKE_ENROLL",
                  let onboardingSessionID = desktopWakeEnrollCompleteCommitRuntimeOutcomeState.onboardingSessionID,
                  let deviceID = desktopManagedPrimaryDeviceID else {
                return nil
            }

            return DesktopWakeEnrollDeferCommitPromptState(
                onboardingSessionID: onboardingSessionID,
                nextStep: "WAKE_ENROLL",
                deviceID: deviceID,
                voiceArtifactSyncReceiptRef: desktopWakeEnrollCompleteCommitRuntimeOutcomeState.voiceArtifactSyncReceiptRef
            )
        }

        if let desktopWakeEnrollSampleCommitRuntimeOutcomeState,
           desktopWakeEnrollSampleCommitRuntimeOutcomeState.phase == .completed {
            guard desktopWakeEnrollSampleCommitRuntimeOutcomeState.nextStep == "WAKE_ENROLL",
                  let onboardingSessionID = desktopWakeEnrollSampleCommitRuntimeOutcomeState.onboardingSessionID,
                  let deviceID = desktopManagedPrimaryDeviceID else {
                return nil
            }

            return DesktopWakeEnrollDeferCommitPromptState(
                onboardingSessionID: onboardingSessionID,
                nextStep: "WAKE_ENROLL",
                deviceID: deviceID,
                voiceArtifactSyncReceiptRef: desktopWakeEnrollSampleCommitRuntimeOutcomeState.voiceArtifactSyncReceiptRef
            )
        }

        if let desktopWakeEnrollStartDraftRuntimeOutcomeState,
           desktopWakeEnrollStartDraftRuntimeOutcomeState.phase == .completed {
            guard desktopWakeEnrollStartDraftRuntimeOutcomeState.nextStep == "WAKE_ENROLL",
                  let onboardingSessionID = desktopWakeEnrollStartDraftRuntimeOutcomeState.onboardingSessionID,
                  let deviceID = desktopManagedPrimaryDeviceID else {
                return nil
            }

            return DesktopWakeEnrollDeferCommitPromptState(
                onboardingSessionID: onboardingSessionID,
                nextStep: "WAKE_ENROLL",
                deviceID: deviceID,
                voiceArtifactSyncReceiptRef: desktopWakeEnrollStartDraftRuntimeOutcomeState.voiceArtifactSyncReceiptRef
            )
        }

        return nil
    }

    private var desktopEmoPersonaLockPromptState: DesktopEmoPersonaLockPromptState? {
        if let desktopEmoPersonaLockRuntimeOutcomeState,
           desktopEmoPersonaLockRuntimeOutcomeState.phase == .completed {
            return nil
        }

        if let desktopWakeEnrollCompleteCommitRuntimeOutcomeState,
           desktopWakeEnrollCompleteCommitRuntimeOutcomeState.phase == .completed,
           desktopWakeEnrollCompleteCommitRuntimeOutcomeState.nextStep == "EMO_PERSONA_LOCK",
           let onboardingSessionID = desktopWakeEnrollCompleteCommitRuntimeOutcomeState.onboardingSessionID {
            return DesktopEmoPersonaLockPromptState(
                onboardingSessionID: onboardingSessionID,
                nextStep: "EMO_PERSONA_LOCK",
                voiceArtifactSyncReceiptRef: desktopWakeEnrollCompleteCommitRuntimeOutcomeState.voiceArtifactSyncReceiptRef
            )
        }

        return nil
    }

    private var desktopAccessProvisionCommitPromptState: DesktopAccessProvisionCommitPromptState? {
        if let desktopAccessProvisionCommitRuntimeOutcomeState,
           desktopAccessProvisionCommitRuntimeOutcomeState.phase == .completed {
            return nil
        }

        if let desktopEmoPersonaLockRuntimeOutcomeState,
           desktopEmoPersonaLockRuntimeOutcomeState.phase == .completed,
           desktopEmoPersonaLockRuntimeOutcomeState.nextStep == "ACCESS_PROVISION",
           let onboardingSessionID = desktopEmoPersonaLockRuntimeOutcomeState.onboardingSessionID {
            return DesktopAccessProvisionCommitPromptState(
                onboardingSessionID: onboardingSessionID,
                nextStep: "ACCESS_PROVISION",
                voiceArtifactSyncReceiptRef: desktopEmoPersonaLockRuntimeOutcomeState.voiceArtifactSyncReceiptRef
            )
        }

        return nil
    }

    private var desktopCompleteCommitPromptState: DesktopCompleteCommitPromptState? {
        if let desktopCompleteCommitRuntimeOutcomeState,
           desktopCompleteCommitRuntimeOutcomeState.phase == .completed {
            return nil
        }

        if let desktopAccessProvisionCommitRuntimeOutcomeState,
           desktopAccessProvisionCommitRuntimeOutcomeState.phase == .completed,
           desktopAccessProvisionCommitRuntimeOutcomeState.nextStep == "COMPLETE",
           let onboardingSessionID = desktopAccessProvisionCommitRuntimeOutcomeState.onboardingSessionID {
            return DesktopCompleteCommitPromptState(
                onboardingSessionID: onboardingSessionID,
                nextStep: "COMPLETE",
                voiceArtifactSyncReceiptRef: desktopAccessProvisionCommitRuntimeOutcomeState.voiceArtifactSyncReceiptRef,
                accessEngineInstanceID: desktopAccessProvisionCommitRuntimeOutcomeState.accessEngineInstanceID
            )
        }

        return nil
    }

    private var desktopReadyVisibilityState: DesktopReadyVisibilityState? {
        guard let desktopCompleteCommitRuntimeOutcomeState,
              desktopCompleteCommitRuntimeOutcomeState.phase == .completed,
              desktopCompleteCommitRuntimeOutcomeState.nextStep == "READY",
              let onboardingSessionID = desktopCompleteCommitRuntimeOutcomeState.onboardingSessionID
        else {
            return nil
        }

        return DesktopReadyVisibilityState(
            onboardingSessionID: onboardingSessionID,
            nextStep: "READY",
            onboardingStatus: desktopCompleteCommitRuntimeOutcomeState.onboardingStatus,
            voiceArtifactSyncReceiptRef: desktopCompleteCommitRuntimeOutcomeState.voiceArtifactSyncReceiptRef,
            accessEngineInstanceID: desktopCompleteCommitRuntimeOutcomeState.accessEngineInstanceID,
            deviceID: desktopManagedPrimaryDeviceID
        )
    }

    private var desktopPairingCompletionVisibilityState: DesktopPairingCompletionVisibilityState? {
        guard let readyVisibilityState = desktopReadyVisibilityState else {
            return nil
        }

        if let latestSessionActiveVisibleContext {
            guard let sessionAttachOutcome = latestSessionActiveVisibleContext.sessionAttachOutcome,
                  ["NEW_SESSION_CREATED", "EXISTING_SESSION_ATTACHED"].contains(sessionAttachOutcome) else {
                return nil
            }

            return DesktopPairingCompletionVisibilityState(
                onboardingSessionID: readyVisibilityState.onboardingSessionID,
                nextStep: readyVisibilityState.nextStep,
                onboardingStatus: readyVisibilityState.onboardingStatus,
                voiceArtifactSyncReceiptRef: readyVisibilityState.voiceArtifactSyncReceiptRef,
                accessEngineInstanceID: readyVisibilityState.accessEngineInstanceID,
                deviceID: readyVisibilityState.deviceID,
                sessionState: latestSessionActiveVisibleContext.sessionState,
                sessionID: latestSessionActiveVisibleContext.sessionID,
                sessionAttachOutcome: sessionAttachOutcome,
                turnID: latestSessionActiveVisibleContext.turnID
            )
        }

        guard ["NEW_SESSION_CREATED", "EXISTING_SESSION_ATTACHED"].contains(
            latestSessionHeaderContext?.sessionAttachOutcome ?? ""
        ),
        let latestSessionHeaderContext else {
            return nil
        }

        return DesktopPairingCompletionVisibilityState(
            onboardingSessionID: readyVisibilityState.onboardingSessionID,
            nextStep: readyVisibilityState.nextStep,
            onboardingStatus: readyVisibilityState.onboardingStatus,
            voiceArtifactSyncReceiptRef: readyVisibilityState.voiceArtifactSyncReceiptRef,
            accessEngineInstanceID: readyVisibilityState.accessEngineInstanceID,
            deviceID: readyVisibilityState.deviceID,
            sessionState: latestSessionHeaderContext.sessionState,
            sessionID: latestSessionHeaderContext.sessionID,
            sessionAttachOutcome: latestSessionHeaderContext.sessionAttachOutcome,
            turnID: nil
        )
    }

    private var desktopPairingCompletionPromptState: DesktopPairingCompletionPromptState? {
        guard let pairingCompletionVisibilityState = desktopPairingCompletionVisibilityState else {
            return nil
        }

        return DesktopPairingCompletionPromptState(
            sourceSurfaceIdentity: "PAIRING_COMPLETION_VISIBLE",
            onboardingSessionID: pairingCompletionVisibilityState.onboardingSessionID,
            nextStep: pairingCompletionVisibilityState.nextStep,
            onboardingStatus: pairingCompletionVisibilityState.onboardingStatus,
            voiceArtifactSyncReceiptRef: pairingCompletionVisibilityState.voiceArtifactSyncReceiptRef,
            accessEngineInstanceID: pairingCompletionVisibilityState.accessEngineInstanceID,
            deviceID: pairingCompletionVisibilityState.deviceID,
            sessionState: pairingCompletionVisibilityState.sessionState,
            sessionID: pairingCompletionVisibilityState.sessionID,
            sessionAttachOutcome: pairingCompletionVisibilityState.sessionAttachOutcome,
            turnID: pairingCompletionVisibilityState.turnID
        )
    }

    private var desktopReadyTimeHandoffIsActive: Bool {
        guard let desktopReadyTimeHandoffState,
              let promptState = desktopPairingCompletionPromptState else {
            return false
        }

        return desktopReadyTimeHandoffState.matches(promptState)
    }

    private var desktopSessionAttachPromptState: DesktopSessionAttachPromptState? {
        guard desktopReadyTimeHandoffIsActive,
              desktopSessionMultiPostureResumePromptState == nil,
              let activeRecoveryVisibleSurface,
              let deviceID = desktopManagedPrimaryDeviceID else {
            return nil
        }

        switch activeRecoveryVisibleSurface {
        case .sessionHeader(let context):
            return DesktopSessionAttachPromptState(
                sourceSurfaceIdentity: "SESSION_OPEN_VISIBLE",
                sessionState: context.sessionState,
                sessionID: context.sessionID,
                currentVisibleSessionAttachOutcome: context.sessionAttachOutcome,
                turnID: nil,
                deviceID: deviceID
            )

        case .sessionActive(let context):
            return DesktopSessionAttachPromptState(
                sourceSurfaceIdentity: "SESSION_ACTIVE_VISIBLE",
                sessionState: context.sessionState,
                sessionID: context.sessionID,
                currentVisibleSessionAttachOutcome: context.sessionAttachOutcome,
                turnID: context.turnID,
                deviceID: deviceID
            )

        case .sessionSoftClosed:
            return nil
        }
    }

    private var desktopSessionSoftClosedVisibilityState: DesktopSessionSoftClosedVisibilityState? {
        guard let foregroundSessionSoftClosedVisibleContext else {
            return nil
        }

        return DesktopSessionSoftClosedVisibilityState(
            sourceSurfaceIdentity: "SESSION_SOFT_CLOSED_VISIBLE",
            sessionState: foregroundSessionSoftClosedVisibleContext.sessionState,
            sessionID: foregroundSessionSoftClosedVisibleContext.sessionID,
            selectedThreadID: foregroundSessionSoftClosedVisibleContext.selectedThreadID,
            selectedThreadTitle: foregroundSessionSoftClosedVisibleContext.selectedThreadTitle,
            pendingWorkOrderID: foregroundSessionSoftClosedVisibleContext.pendingWorkOrderID,
            resumeTier: foregroundSessionSoftClosedVisibleContext.resumeTier,
            resumeSummaryBullets: foregroundSessionSoftClosedVisibleContext.resumeSummaryBullets,
            archivedUserTurnText: foregroundSessionSoftClosedVisibleContext.archivedUserTurnText,
            archivedSeleneTurnText: foregroundSessionSoftClosedVisibleContext.archivedSeleneTurnText
        )
    }

    private var desktopSessionSoftClosedResumePromptState: DesktopSessionSoftClosedResumePromptState? {
        guard let latestSessionSoftClosedVisibleContext,
              let deviceID = desktopManagedPrimaryDeviceID else {
            return nil
        }

        return DesktopSessionSoftClosedResumePromptState(
            sourceSurfaceIdentity: "SESSION_SOFT_CLOSED_VISIBLE",
            sessionState: latestSessionSoftClosedVisibleContext.sessionState,
            sessionID: latestSessionSoftClosedVisibleContext.sessionID,
            selectedThreadID: latestSessionSoftClosedVisibleContext.selectedThreadID,
            selectedThreadTitle: latestSessionSoftClosedVisibleContext.selectedThreadTitle,
            pendingWorkOrderID: latestSessionSoftClosedVisibleContext.pendingWorkOrderID,
            resumeTier: latestSessionSoftClosedVisibleContext.resumeTier,
            resumeSummaryBullets: latestSessionSoftClosedVisibleContext.resumeSummaryBullets,
            deviceID: deviceID
        )
    }

    private var desktopSessionRecoverPromptState: DesktopSessionRecoverPromptState? {
        guard desktopReadyTimeHandoffIsActive,
              let latestSessionSuspendedVisibleContext,
              let deviceID = desktopManagedPrimaryDeviceID else {
            return nil
        }

        return DesktopSessionRecoverPromptState(
            sourceSurfaceIdentity: "SESSION_SUSPENDED_VISIBLE",
            sessionState: latestSessionSuspendedVisibleContext.sessionState,
            sessionID: latestSessionSuspendedVisibleContext.sessionID,
            recoveryMode: latestSessionSuspendedVisibleContext.recoveryMode?.rawValue,
            deviceID: deviceID
        )
    }

    private var desktopSessionMultiPostureResumePromptState: DesktopSessionMultiPostureResumePromptState? {
        let softClosedPromptState = desktopSessionSoftClosedResumePromptState
        let suspendedPromptState = desktopSessionRecoverPromptState

        guard !(softClosedPromptState != nil && suspendedPromptState != nil) else {
            return nil
        }

        if let softClosedPromptState {
            return DesktopSessionMultiPostureResumePromptState(
                resumeMode: .softClosedExplicitResume,
                sourceSurfaceIdentity: softClosedPromptState.sourceSurfaceIdentity,
                sessionState: softClosedPromptState.sessionState,
                sessionID: softClosedPromptState.sessionID,
                selectedThreadID: softClosedPromptState.selectedThreadID,
                selectedThreadTitle: softClosedPromptState.selectedThreadTitle,
                pendingWorkOrderID: softClosedPromptState.pendingWorkOrderID,
                resumeTier: softClosedPromptState.resumeTier,
                resumeSummaryBullets: softClosedPromptState.resumeSummaryBullets,
                recoveryMode: nil,
                deviceID: softClosedPromptState.deviceID
            )
        }

        if let suspendedPromptState {
            return DesktopSessionMultiPostureResumePromptState(
                resumeMode: .suspendedAuthoritativeRereadRecover,
                sourceSurfaceIdentity: suspendedPromptState.sourceSurfaceIdentity,
                sessionState: suspendedPromptState.sessionState,
                sessionID: suspendedPromptState.sessionID,
                selectedThreadID: nil,
                selectedThreadTitle: nil,
                pendingWorkOrderID: nil,
                resumeTier: nil,
                resumeSummaryBullets: [],
                recoveryMode: suspendedPromptState.recoveryMode,
                deviceID: suspendedPromptState.deviceID
            )
        }

        return nil
    }

    private var desktopSessionMultiPostureEntryPromptState: DesktopSessionMultiPostureEntryPromptState? {
        let attachPromptState = desktopSessionAttachPromptState
        let resumePromptState = desktopSessionMultiPostureResumePromptState

        guard !(attachPromptState != nil && resumePromptState != nil) else {
            return nil
        }

        if let attachPromptState {
            return DesktopSessionMultiPostureEntryPromptState(
                entryMode: .currentVisibleAttach,
                sourceSurfaceIdentity: attachPromptState.sourceSurfaceIdentity,
                sessionState: attachPromptState.sessionState,
                sessionID: attachPromptState.sessionID,
                currentVisibleSessionAttachOutcome: attachPromptState.currentVisibleSessionAttachOutcome,
                turnID: attachPromptState.turnID,
                selectedThreadID: nil,
                selectedThreadTitle: nil,
                pendingWorkOrderID: nil,
                resumeTier: nil,
                resumeSummaryBullets: [],
                recoveryMode: nil,
                deviceID: attachPromptState.deviceID
            )
        }

        guard let resumePromptState else {
            return nil
        }

        switch resumePromptState.resumeMode {
        case .softClosedExplicitResume:
            return DesktopSessionMultiPostureEntryPromptState(
                entryMode: .softClosedExplicitResume,
                sourceSurfaceIdentity: resumePromptState.sourceSurfaceIdentity,
                sessionState: resumePromptState.sessionState,
                sessionID: resumePromptState.sessionID,
                currentVisibleSessionAttachOutcome: nil,
                turnID: nil,
                selectedThreadID: resumePromptState.selectedThreadID,
                selectedThreadTitle: resumePromptState.selectedThreadTitle,
                pendingWorkOrderID: resumePromptState.pendingWorkOrderID,
                resumeTier: resumePromptState.resumeTier,
                resumeSummaryBullets: resumePromptState.resumeSummaryBullets,
                recoveryMode: nil,
                deviceID: resumePromptState.deviceID
            )

        case .suspendedAuthoritativeRereadRecover:
            return DesktopSessionMultiPostureEntryPromptState(
                entryMode: .suspendedAuthoritativeRereadRecover,
                sourceSurfaceIdentity: resumePromptState.sourceSurfaceIdentity,
                sessionState: resumePromptState.sessionState,
                sessionID: resumePromptState.sessionID,
                currentVisibleSessionAttachOutcome: nil,
                turnID: nil,
                selectedThreadID: nil,
                selectedThreadTitle: nil,
                pendingWorkOrderID: nil,
                resumeTier: nil,
                resumeSummaryBullets: [],
                recoveryMode: resumePromptState.recoveryMode,
                deviceID: resumePromptState.deviceID
            )
        }
    }

    private var desktopForegroundVoiceTurnMatchingSelectedThreadKey: String? {
        guard let activeSessionID = latestSessionActiveVisibleContext?.sessionID else {
            return nil
        }

        if let completedEntryOutcome = desktopSessionMultiPostureEntryRuntimeOutcomeState,
           completedEntryOutcome.phase == .completed,
           completedEntryOutcome.entryMode == .softClosedExplicitResume,
           completedEntryOutcome.sessionID == activeSessionID,
           let selectedThreadID = completedEntryOutcome.selectedThreadID {
            return selectedThreadID
        }

        if let completedResumeOutcome = desktopSessionMultiPostureResumeRuntimeOutcomeState,
           completedResumeOutcome.phase == .completed,
           completedResumeOutcome.resumeMode == .softClosedExplicitResume,
           completedResumeOutcome.sessionID == activeSessionID,
           let selectedThreadID = completedResumeOutcome.selectedThreadID {
            return selectedThreadID
        }

        return nil
    }

    private var desktopForegroundVoiceTurnActiveAuthorityPolicyContextRef: String? {
        guard let activeVisibleContext = latestSessionActiveVisibleContext,
              activeVisibleContext.sessionState == "SessionState::Active" else {
            return nil
        }

        return activeVisibleContext.authorityStatePolicyContextRef
    }

    private var desktopSessionSuspendedVisibilityState: DesktopSessionSuspendedVisibilityState? {
        guard let foregroundSessionSuspendedVisibleContext else {
            return nil
        }

        return DesktopSessionSuspendedVisibilityState(
            sourceSurfaceIdentity: "SESSION_SUSPENDED_VISIBLE",
            sessionState: foregroundSessionSuspendedVisibleContext.sessionState,
            sessionID: foregroundSessionSuspendedVisibleContext.sessionID,
            nextAllowedActionsMaySpeak: foregroundSessionSuspendedVisibleContext.nextAllowedActionsMaySpeak,
            nextAllowedActionsMustWait: foregroundSessionSuspendedVisibleContext.nextAllowedActionsMustWait,
            nextAllowedActionsMustRewake: foregroundSessionSuspendedVisibleContext.nextAllowedActionsMustRewake,
            recoveryMode: foregroundSessionSuspendedVisibleContext.recoveryMode?.rawValue,
            reconciliationDecision: foregroundSessionSuspendedVisibleContext.reconciliationDecision?.rawValue
        )
    }

    private var desktopRecoveryVisibilityState: DesktopRecoveryVisibilityState? {
        guard let activeRecoveryVisibleSurface, let activeRecoveryDisplayState else {
            return nil
        }

        return DesktopRecoveryVisibilityState(
            displayState: activeRecoveryDisplayState,
            sourceSurfaceIdentity: activeRecoveryVisibleSurface.sourceSurfaceTitle,
            sessionState: activeRecoveryVisibleSurface.sessionState,
            sessionID: activeRecoveryVisibleSurface.sessionID,
            recoveryMode: activeRecoveryVisibleSurface.recoveryMode?.rawValue,
            reconciliationDecision: activeRecoveryVisibleSurface.reconciliationDecision?.rawValue
        )
    }

    private var desktopInterruptVisibilityState: DesktopInterruptVisibilityState? {
        guard
            activeInterruptDisplayState == .interruptVisible,
            let foregroundSessionActiveVisibleContext
        else {
            return nil
        }

        return DesktopInterruptVisibilityState(
            sourceSurfaceIdentity: "INTERRUPT_VISIBLE",
            sessionState: foregroundSessionActiveVisibleContext.sessionState,
            sessionID: foregroundSessionActiveVisibleContext.sessionID,
            turnID: foregroundSessionActiveVisibleContext.turnID,
            interruptSubjectRelation: foregroundSessionActiveVisibleContext.interruptSubjectRelation?
                .rawValue,
            interruptContinuityOutcome: foregroundSessionActiveVisibleContext
                .interruptContinuityOutcome?.rawValue,
            interruptResumePolicy: foregroundSessionActiveVisibleContext.interruptResumePolicy?.rawValue,
            returnCheckPending: foregroundSessionActiveVisibleContext.returnCheckPending,
            acceptedInterruptPostureSummary: foregroundSessionActiveVisibleContext
                .acceptedInterruptPostureSummary
        )
    }

    private var desktopInterruptResponseProductionState: DesktopInterruptResponseProductionState? {
        guard
            activeInterruptDisplayState == .interruptVisible,
            let foregroundSessionActiveVisibleContext,
            foregroundSessionActiveVisibleContext.hasInterruptResponseProductionSurface
        else {
            return nil
        }

        return DesktopInterruptResponseProductionState(
            sourceSurfaceIdentity: "INTERRUPT_VISIBLE",
            sessionState: foregroundSessionActiveVisibleContext.sessionState,
            sessionID: foregroundSessionActiveVisibleContext.sessionID,
            turnID: foregroundSessionActiveVisibleContext.turnID,
            interruptSubjectRelation: foregroundSessionActiveVisibleContext.interruptSubjectRelation?
                .rawValue,
            interruptContinuityOutcome: foregroundSessionActiveVisibleContext
                .interruptContinuityOutcome?.rawValue,
            interruptResumePolicy: foregroundSessionActiveVisibleContext.interruptResumePolicy?.rawValue,
            returnCheckPending: foregroundSessionActiveVisibleContext.returnCheckPending,
            hasInterruptResponseConflict: foregroundSessionActiveVisibleContext
                .hasInterruptResponseConflict,
            hasLawfulInterruptClarifyDirective: foregroundSessionActiveVisibleContext
                .hasLawfulInterruptClarifyDirective,
            interruptClarifyQuestion: foregroundSessionActiveVisibleContext.interruptClarifyQuestion,
            interruptAcceptedAnswerFormats: foregroundSessionActiveVisibleContext
                .interruptAcceptedAnswerFormats,
            activeContext: foregroundSessionActiveVisibleContext
        )
    }

    private var desktopInterruptSubjectReferencesVisibilityState:
        DesktopInterruptSubjectReferencesVisibilityState?
    {
        guard
            activeInterruptDisplayState == .interruptVisible,
            let foregroundSessionActiveVisibleContext,
            foregroundSessionActiveVisibleContext.hasLawfulInterruptSubjectReferences
        else {
            return nil
        }

        return DesktopInterruptSubjectReferencesVisibilityState(
            sourceSurfaceIdentity: "INTERRUPT_VISIBLE",
            sessionState: foregroundSessionActiveVisibleContext.sessionState,
            sessionID: foregroundSessionActiveVisibleContext.sessionID,
            turnID: foregroundSessionActiveVisibleContext.turnID,
            interruptSubjectRelation: foregroundSessionActiveVisibleContext.interruptSubjectRelation?
                .rawValue,
            activeSubjectRef: foregroundSessionActiveVisibleContext.activeSubjectRef,
            interruptedSubjectRef: foregroundSessionActiveVisibleContext.interruptedSubjectRef,
            hasLawfulInterruptSubjectReferences: foregroundSessionActiveVisibleContext
                .hasLawfulInterruptSubjectReferences
        )
    }

    private var desktopInterruptSubjectRelationConfidenceVisibilityState:
        DesktopInterruptSubjectRelationConfidenceVisibilityState?
    {
        guard
            activeInterruptDisplayState == .interruptVisible,
            let foregroundSessionActiveVisibleContext,
            foregroundSessionActiveVisibleContext.hasLawfulInterruptSubjectRelationConfidence
        else {
            return nil
        }

        return DesktopInterruptSubjectRelationConfidenceVisibilityState(
            sourceSurfaceIdentity: "INTERRUPT_VISIBLE",
            sessionState: foregroundSessionActiveVisibleContext.sessionState,
            sessionID: foregroundSessionActiveVisibleContext.sessionID,
            turnID: foregroundSessionActiveVisibleContext.turnID,
            interruptSubjectRelation: foregroundSessionActiveVisibleContext.interruptSubjectRelation?
                .rawValue,
            interruptSubjectRelationConfidence: foregroundSessionActiveVisibleContext
                .interruptSubjectRelationConfidence,
            hasLawfulInterruptSubjectRelationConfidence: foregroundSessionActiveVisibleContext
                .hasLawfulInterruptSubjectRelationConfidence
        )
    }

    private var desktopInterruptReturnCheckExpiryVisibilityState:
        DesktopInterruptReturnCheckExpiryVisibilityState?
    {
        guard
            activeInterruptDisplayState == .interruptVisible,
            let foregroundSessionActiveVisibleContext,
            foregroundSessionActiveVisibleContext.hasLawfulInterruptReturnCheckExpiry
        else {
            return nil
        }

        return DesktopInterruptReturnCheckExpiryVisibilityState(
            sourceSurfaceIdentity: "INTERRUPT_VISIBLE",
            sessionState: foregroundSessionActiveVisibleContext.sessionState,
            sessionID: foregroundSessionActiveVisibleContext.sessionID,
            turnID: foregroundSessionActiveVisibleContext.turnID,
            interruptSubjectRelation: foregroundSessionActiveVisibleContext.interruptSubjectRelation?
                .rawValue,
            interruptContinuityOutcome: foregroundSessionActiveVisibleContext
                .interruptContinuityOutcome?.rawValue,
            interruptResumePolicy: foregroundSessionActiveVisibleContext.interruptResumePolicy?
                .rawValue,
            returnCheckPending: foregroundSessionActiveVisibleContext.returnCheckPending,
            returnCheckExpiresAt: foregroundSessionActiveVisibleContext.returnCheckExpiresAt,
            hasLawfulInterruptReturnCheckExpiry: foregroundSessionActiveVisibleContext
                .hasLawfulInterruptReturnCheckExpiry
        )
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
        case "desktop_pairing_bound":
            if let deviceID = desktopManagedPrimaryDeviceID {
                return "The exact managed bridge `deviceID` is currently `\(deviceID)`, but this receipt remains read-only unless it is surfaced as an actionable bounded pairing-bound draft."
            }

            return "The exact managed bridge `deviceID` is currently unavailable, so this receipt remains visible but not locally provable yet."
        case "desktop_wakeword_configured":
            if desktopManagedPrimaryDeviceID == nil {
                return "The exact managed bridge `deviceID` is currently unavailable, so this receipt remains visible but not locally provable yet."
            }

            guard let activeVisibleContext = latestSessionActiveVisibleContext else {
                return "The current active visible context is unavailable, so this receipt remains visible but not locally provable yet."
            }

            guard activeVisibleContext.hasLawfulWakeRuntimeEventEvidenceCarrierFamily else {
                return "The current active visible context does not yet preserve the exact lawful wake runtime event evidence carrier family, so this receipt remains read-only."
            }

            guard let wakeRuntimeEventWakeProfileID = activeVisibleContext.wakeRuntimeEventWakeProfileID?
                .trimmingCharacters(in: .whitespacesAndNewlines),
                !wakeRuntimeEventWakeProfileID.isEmpty else {
                return "The current active visible context does not yet preserve exact `wake_runtime_event_wake_profile_id`, so this receipt remains read-only."
            }

            guard boundedWakeEnrollCompletionLineageVoiceArtifactSyncReceiptRef != nil else {
                return "The bounded wake-enroll completion lineage does not currently preserve exact `voice_artifact_sync_receipt_ref`, so this receipt remains read-only."
            }

            return "The current active visible context already preserves exact `wake_runtime_event_wake_profile_id=\(wakeRuntimeEventWakeProfileID)`, but this receipt remains read-only unless it is surfaced as an actionable bounded wakeword-configured draft."
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
                    Text("Bounded desktop platform-setup receipt submission only. This shell derives exact locally provable drafts for `install_launch_handshake`, `mic_permission_granted`, exact `desktop_pairing_bound` when the exact managed bridge `deviceID` is available, and exact `desktop_wakeword_configured` only when the exact managed bridge `deviceID`, exact lawful wake runtime event evidence carrier family, exact `wake_runtime_event_wake_profile_id`, and exact bounded wake-enroll sync receipt visibility are all present. It dispatches exact `PLATFORM_SETUP_RECEIPT` only and does not widen into local wake authority.")
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
                                    HStack(alignment: .top, spacing: 12) {
                                        Text("receipt_kind")
                                            .font(.caption.monospaced())
                                            .foregroundStyle(.secondary)
                                            .frame(width: 170, alignment: .leading)

                                        Text(draft.receiptKind)
                                            .font(.body.monospaced())
                                            .frame(maxWidth: .infinity, alignment: .leading)
                                    }

                                    if let deviceID = draft.deviceID {
                                        HStack(alignment: .top, spacing: 12) {
                                            Text("device_id")
                                                .font(.caption.monospaced())
                                                .foregroundStyle(.secondary)
                                                .frame(width: 170, alignment: .leading)

                                            Text(deviceID)
                                                .font(.body.monospaced())
                                                .frame(maxWidth: .infinity, alignment: .leading)
                                        }
                                    }

                                    if let wakeRuntimeEventWakeProfileID = draft.wakeRuntimeEventWakeProfileID {
                                        HStack(alignment: .top, spacing: 12) {
                                            Text("wake_runtime_event_wake_profile_id")
                                                .font(.caption.monospaced())
                                                .foregroundStyle(.secondary)
                                                .frame(width: 170, alignment: .leading)

                                            Text(wakeRuntimeEventWakeProfileID)
                                                .font(.body.monospaced())
                                                .frame(maxWidth: .infinity, alignment: .leading)
                                        }
                                    }

                                    if let wakeRuntimeEventAccepted = draft.wakeRuntimeEventAccepted {
                                        HStack(alignment: .top, spacing: 12) {
                                            Text("wake_runtime_event_accepted")
                                                .font(.caption.monospaced())
                                                .foregroundStyle(.secondary)
                                                .frame(width: 170, alignment: .leading)

                                            Text(wakeRuntimeEventAccepted ? "true" : "false")
                                                .font(.body.monospaced())
                                                .frame(maxWidth: .infinity, alignment: .leading)
                                        }
                                    }

                                    if let voiceArtifactSyncReceiptRef = draft.voiceArtifactSyncReceiptRef {
                                        HStack(alignment: .top, spacing: 12) {
                                            Text("voice_artifact_sync_receipt_ref")
                                                .font(.caption.monospaced())
                                                .foregroundStyle(.secondary)
                                                .frame(width: 170, alignment: .leading)

                                            Text(voiceArtifactSyncReceiptRef)
                                                .font(.body.monospaced())
                                                .frame(maxWidth: .infinity, alignment: .leading)
                                        }
                                    }

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

                    Text("Only exact `install_launch_handshake`, exact `mic_permission_granted`, exact `desktop_pairing_bound`, and exact `desktop_wakeword_configured` submission are in scope here. Exact `desktop_wakeword_configured` is derived only from already-live bounded wake evidence plus already-live bounded wake-enroll sync posture, and no local wake authority, no wake-routing handoff, no link-delivery controls, and no autonomous-unlock controls are introduced here.")
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

    private var desktopSenderVerificationVisibilityCard: some View {
        let visibilityState = desktopSenderVerificationVisibilityState

        return Group {
            if let visibilityState {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded desktop sender-verification visibility only. This shell preserves the canonical `SENDER_VERIFICATION` posture reached after exact `TERMS_ACCEPT` and renders returned verification posture in read-only form only.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        ForEach(
                            [
                                ("onboarding_session_id", visibilityState.onboardingSessionID),
                                ("next_step", visibilityState.nextStep),
                                ("onboarding_status", visibilityState.onboardingStatus ?? "not_available"),
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

                        Text("Canonical onboarding posture has advanced to exact `SENDER_VERIFICATION`. This shell is intentionally read-only at that boundary and does not add employee-photo upload, sender-decision submission, local photo capture, or local onboarding authority.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        desktopOnboardingEntryListCard(
                            title: "required_verification_gates",
                            items: visibilityState.requiredVerificationGates,
                            emptyText: "No required_verification_gates are available in the bounded sender-verification visibility posture."
                        )

                        Text("Read-only sender-verification visibility only. No employee-photo controls, no sender confirm / reject controls, no hidden or background capture, no primary-device bypass, and no proven native macOS wake-listener integration claim are introduced by this surface.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Onboarding Sender Verification")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopEmployeePhotoCaptureSendCard: some View {
        let promptState = desktopEmployeePhotoCaptureSendPromptState
        let displayedOnboardingSessionID = desktopEmployeePhotoCaptureSendRuntimeOutcomeState?.onboardingSessionID
            ?? promptState?.onboardingSessionID
            ?? "unavailable"
        let displayedNextStep = desktopEmployeePhotoCaptureSendRuntimeOutcomeState?.nextStep
            ?? promptState?.nextStep
            ?? "not_provided"
        let displayedOnboardingStatus = desktopEmployeePhotoCaptureSendRuntimeOutcomeState?.onboardingStatus
            ?? promptState?.onboardingStatus
            ?? "not_available"
        let displayedRequiredVerificationGates = promptState?.requiredVerificationGates
            ?? desktopSenderVerificationVisibilityState?.requiredVerificationGates
            ?? []

        return Group {
            if promptState != nil || desktopEmployeePhotoCaptureSendRuntimeOutcomeState != nil {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded employee photo capture send only. This shell derives bounded prompt state from the already-live sender-verification visibility posture, dispatches exact `EMPLOYEE_PHOTO_CAPTURE_SEND` with an already-existing exact `photo_blob_ref` only, and keeps returned onboarding posture read-only outside the exact control itself.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        ForEach(
                            [
                                ("onboarding_session_id", displayedOnboardingSessionID),
                                ("next_step", displayedNextStep),
                                ("onboarding_status", displayedOnboardingStatus),
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
                            title: "required_verification_gates",
                            items: displayedRequiredVerificationGates,
                            emptyText: "No required_verification_gates are available in the bounded employee-photo-send posture."
                        )

                        VStack(alignment: .leading, spacing: 8) {
                            Text("photo_blob_ref")
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)

                            TextField(
                                "Enter existing photo_blob_ref",
                                text: $desktopEmployeePhotoCaptureSendPhotoBlobRefInput
                            )
                            .textFieldStyle(.roundedBorder)
                            .disabled(desktopEmployeePhotoCaptureSendRuntimeOutcomeState?.phase == .dispatching)
                        }

                        Text("This shell is dispatching canonical `EMPLOYEE_PHOTO_CAPTURE_SEND` for an already-existing exact `photo_blob_ref` only. No local photo picker, no local camera capture, no local upload, no pasteboard blob authority, and no sender-decision mutation are introduced here.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        if let promptState {
                            Button("Submit employee photo capture send") {
                                Task {
                                    await submitDesktopEmployeePhotoCaptureSend(promptState: promptState)
                                }
                            }
                            .buttonStyle(.borderedProminent)
                            .disabled(
                                desktopEmployeePhotoCaptureSendRuntimeOutcomeState?.phase == .dispatching
                                || desktopEmployeePhotoCaptureSendPhotoBlobRefInput
                                    .trimmingCharacters(in: .whitespacesAndNewlines)
                                    .isEmpty
                            )
                        }

                        if let desktopEmployeePhotoCaptureSendRuntimeOutcomeState {
                            Divider()

                            Text(desktopEmployeePhotoCaptureSendRuntimeOutcomeState.title)
                                .font(.headline)

                            ForEach(
                                [
                                    ("dispatch_phase", desktopEmployeePhotoCaptureSendRuntimeOutcomeState.phase.rawValue),
                                    ("request_id", desktopEmployeePhotoCaptureSendRuntimeOutcomeState.requestID),
                                    ("endpoint", desktopEmployeePhotoCaptureSendRuntimeOutcomeState.endpoint),
                                    ("photo_blob_ref", desktopEmployeePhotoCaptureSendRuntimeOutcomeState.photoBlobRef ?? "not_provided"),
                                    ("outcome", desktopEmployeePhotoCaptureSendRuntimeOutcomeState.outcome ?? "not_available"),
                                    ("reason", desktopEmployeePhotoCaptureSendRuntimeOutcomeState.reason ?? "not_available"),
                                    ("onboarding_status", desktopEmployeePhotoCaptureSendRuntimeOutcomeState.onboardingStatus ?? "not_available"),
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

                            Text(desktopEmployeePhotoCaptureSendRuntimeOutcomeState.summary)
                                .frame(maxWidth: .infinity, alignment: .leading)

                            Text(desktopEmployeePhotoCaptureSendRuntimeOutcomeState.detail)
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        } else {
                            Text("Awaiting explicit user-triggered employee photo capture send for an already-existing exact `photo_blob_ref`. Sender-verification visibility remains read-only outside this exact bounded submit surface.")
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }

                        Text("Only exact `EMPLOYEE_PHOTO_CAPTURE_SEND` with exact `photo_blob_ref` is in scope here. No local photo picker controls, no local camera controls, no local upload controls, no pasteboard-import controls, no sender confirm / reject controls, no hidden or background capture, and no local onboarding authority claims are introduced by this surface.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Onboarding Employee Photo Capture Send")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopEmployeeSenderVerifyCommitCard: some View {
        let promptState = desktopEmployeeSenderVerifyCommitPromptState
        let displayedOnboardingSessionID = desktopEmployeeSenderVerifyCommitRuntimeOutcomeState?.onboardingSessionID
            ?? promptState?.onboardingSessionID
            ?? "unavailable"
        let displayedNextStep = desktopEmployeeSenderVerifyCommitRuntimeOutcomeState?.nextStep
            ?? promptState?.nextStep
            ?? "not_provided"
        let displayedOnboardingStatus = desktopEmployeeSenderVerifyCommitRuntimeOutcomeState?.onboardingStatus
            ?? promptState?.onboardingStatus
            ?? "not_available"
        let displayedRequiredVerificationGates = promptState?.requiredVerificationGates
            ?? desktopSenderVerificationVisibilityState?.requiredVerificationGates
            ?? []
        let displayedPhotoBlobRef = promptState?.photoBlobRef
            ?? desktopEmployeePhotoCaptureSendRuntimeOutcomeState?.photoBlobRef
            ?? "not_provided"

        return Group {
            if promptState != nil || desktopEmployeeSenderVerifyCommitRuntimeOutcomeState != nil {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded sender verification commit only. This shell derives bounded prompt state from already-live H273 employee-photo-send completion posture, dispatches exact `EMPLOYEE_SENDER_VERIFY_COMMIT` with exact `sender_decision` only, and keeps returned onboarding posture read-only outside the exact control itself.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        ForEach(
                            [
                                ("onboarding_session_id", displayedOnboardingSessionID),
                                ("next_step", displayedNextStep),
                                ("onboarding_status", displayedOnboardingStatus),
                                ("photo_blob_ref", displayedPhotoBlobRef),
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
                            title: "required_verification_gates",
                            items: displayedRequiredVerificationGates,
                            emptyText: "No required_verification_gates are available in the bounded sender-verification commit posture."
                        )

                        VStack(alignment: .leading, spacing: 8) {
                            Text("sender_decision")
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)

                            Picker("sender_decision", selection: $desktopEmployeeSenderVerifyCommitSelectedDecision) {
                                Text("CONFIRM").tag("CONFIRM")
                                Text("REJECT").tag("REJECT")
                            }
                            .pickerStyle(.segmented)
                            .disabled(desktopEmployeeSenderVerifyCommitRuntimeOutcomeState?.phase == .dispatching)
                        }

                        Text("This shell is dispatching canonical `EMPLOYEE_SENDER_VERIFY_COMMIT` with exact `sender_decision` only, derived from already-live bounded sender-verification visibility plus H273 photo-send completion posture. Returned `PRIMARY_DEVICE_CONFIRM`, `BLOCKED`, or later next-step posture stays bounded and read-only here except where the already-landed primary-device-confirm card becomes actionable.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        if let promptState {
                            Button("Submit sender verification commit") {
                                Task {
                                    await submitDesktopEmployeeSenderVerifyCommit(promptState: promptState)
                                }
                            }
                            .buttonStyle(.borderedProminent)
                            .disabled(
                                desktopEmployeeSenderVerifyCommitRuntimeOutcomeState?.phase == .dispatching
                                || !["CONFIRM", "REJECT"].contains(desktopEmployeeSenderVerifyCommitSelectedDecision)
                            )
                        }

                        if let desktopEmployeeSenderVerifyCommitRuntimeOutcomeState {
                            Divider()

                            Text(desktopEmployeeSenderVerifyCommitRuntimeOutcomeState.title)
                                .font(.headline)

                            ForEach(
                                [
                                    ("dispatch_phase", desktopEmployeeSenderVerifyCommitRuntimeOutcomeState.phase.rawValue),
                                    ("request_id", desktopEmployeeSenderVerifyCommitRuntimeOutcomeState.requestID),
                                    ("endpoint", desktopEmployeeSenderVerifyCommitRuntimeOutcomeState.endpoint),
                                    ("sender_decision", desktopEmployeeSenderVerifyCommitRuntimeOutcomeState.senderDecision ?? "not_provided"),
                                    ("outcome", desktopEmployeeSenderVerifyCommitRuntimeOutcomeState.outcome ?? "not_available"),
                                    ("reason", desktopEmployeeSenderVerifyCommitRuntimeOutcomeState.reason ?? "not_available"),
                                    ("onboarding_status", desktopEmployeeSenderVerifyCommitRuntimeOutcomeState.onboardingStatus ?? "not_available"),
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

                            Text(desktopEmployeeSenderVerifyCommitRuntimeOutcomeState.summary)
                                .frame(maxWidth: .infinity, alignment: .leading)

                            Text(desktopEmployeeSenderVerifyCommitRuntimeOutcomeState.detail)
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        } else {
                            Text("Awaiting explicit user-triggered sender verification commit with exact `sender_decision`, derived from already-live H273 photo-send completion posture. Sender-verification visibility remains bounded and read-only outside this exact submit surface.")
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }

                        Text("Only exact `EMPLOYEE_SENDER_VERIFY_COMMIT` with exact `sender_decision` is in scope here. No local photo picker controls, no local camera controls, no local upload controls, no pasteboard-import controls, no primary-device bypass controls, no hidden or background capture, and no local onboarding authority claims are introduced by this surface.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Onboarding Sender Verification Commit")
                        .font(.headline)
                }
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

                    Text("This exact surface is dispatching canonical wake-enroll start draft only. Any later wake-sample, wake-complete, and wake-defer controls are separately gated from this surface, while local wake authority and proven native macOS wake-listener claims remain out of scope here.")
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

                    Text("Only exact `WAKE_ENROLL_START_DRAFT` with the exact managed bridge `deviceID` is in scope here. Any later wake-sample, wake-complete, and wake-defer control is separately gated from this surface; no sender-verification controls, no employee-photo controls, no emo-persona controls, no access-provision controls, no pairing-completion controls, and no proven native macOS wake-listener integration claim are introduced by this surface.")
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

                    Text("Only exact `WAKE_ENROLL_SAMPLE_COMMIT` with the exact managed bridge `deviceID` and exact `proofOK=true` is in scope here. Any wake-complete and wake-defer control remains separately gated from this surface; no sender-verification controls, no employee-photo controls, no emo-persona controls, no access-provision controls, no pairing-completion controls, and no proven native macOS wake-listener integration claim are introduced by this surface.")
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

                    Text("Only exact wake-enroll complete commit with the exact managed bridge `deviceID` is in scope here. Any wake-defer control remains separately gated from this surface; no emo-persona controls, no access-provision controls, no pairing-completion controls, no sender-verification controls, no employee-photo controls, and no proven native macOS wake-listener integration claim are introduced by this surface.")
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

    @ViewBuilder
    private var desktopWakeEnrollDeferCommitCard: some View {
        let promptState = desktopWakeEnrollDeferCommitPromptState
        let wakeCompleteDeferContext = (
            desktopWakeEnrollCompleteCommitRuntimeOutcomeState?.phase == .completed
            && desktopWakeEnrollCompleteCommitRuntimeOutcomeState?.nextStep == "WAKE_ENROLL"
        ) ? desktopWakeEnrollCompleteCommitRuntimeOutcomeState : nil
        let wakeSampleDeferContext = (
            desktopWakeEnrollSampleCommitRuntimeOutcomeState?.phase == .completed
            && desktopWakeEnrollSampleCommitRuntimeOutcomeState?.nextStep == "WAKE_ENROLL"
        ) ? desktopWakeEnrollSampleCommitRuntimeOutcomeState : nil
        let wakeStartDeferContext = (
            desktopWakeEnrollStartDraftRuntimeOutcomeState?.phase == .completed
            && desktopWakeEnrollStartDraftRuntimeOutcomeState?.nextStep == "WAKE_ENROLL"
        ) ? desktopWakeEnrollStartDraftRuntimeOutcomeState : nil
        let displayedOnboardingSessionID = promptState?.onboardingSessionID
            ?? desktopWakeEnrollDeferCommitRuntimeOutcomeState?.onboardingSessionID
            ?? wakeCompleteDeferContext?.onboardingSessionID
            ?? wakeSampleDeferContext?.onboardingSessionID
            ?? wakeStartDeferContext?.onboardingSessionID
            ?? "unavailable"
        let displayedNextStep = desktopWakeEnrollDeferCommitRuntimeOutcomeState?.nextStep
            ?? promptState?.nextStep
            ?? wakeCompleteDeferContext?.nextStep
            ?? wakeSampleDeferContext?.nextStep
            ?? wakeStartDeferContext?.nextStep
            ?? "not_provided"
        let displayedDeviceID = promptState?.deviceID
            ?? desktopWakeEnrollDeferCommitRuntimeOutcomeState?.deviceID
            ?? wakeCompleteDeferContext?.deviceID
            ?? wakeSampleDeferContext?.deviceID
            ?? wakeStartDeferContext?.deviceID
            ?? desktopManagedPrimaryDeviceID
            ?? "not_provided"
        let displayedVoiceArtifactSyncReceiptRef = desktopWakeEnrollDeferCommitRuntimeOutcomeState?.voiceArtifactSyncReceiptRef
            ?? promptState?.voiceArtifactSyncReceiptRef
            ?? wakeCompleteDeferContext?.voiceArtifactSyncReceiptRef
            ?? wakeSampleDeferContext?.voiceArtifactSyncReceiptRef
            ?? wakeStartDeferContext?.voiceArtifactSyncReceiptRef
            ?? "not_available"

        if promptState != nil
            || wakeCompleteDeferContext != nil
            || wakeSampleDeferContext != nil
            || wakeStartDeferContext != nil
            || desktopWakeEnrollDeferCommitRuntimeOutcomeState != nil {
            GroupBox {
                VStack(alignment: .leading, spacing: 12) {
                    Text("Bounded desktop wake-enroll defer-commit submission only. This shell derives bounded prompt state from already-live completed wake-enroll outcome only while canonical onboarding posture remains at exact `WAKE_ENROLL`, dispatches exact `WAKE_ENROLL_DEFER_COMMIT`, and keeps any returned exact `WAKE_ENROLL` posture plus any returned exact `voice_artifact_sync_receipt_ref` read-only only outside the exact wake-defer control itself.")
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

                    Text("This exact surface is dispatching canonical wake-enroll defer commit only. No local `deferred_until` authoring, no wake-listener integration, no wake-to-turn handoff, no pairing completion mutation, and no autonomous unlock are introduced by this shell.")
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    if let promptState {
                        Button("Defer bounded wake enrollment") {
                            Task {
                                await submitDesktopWakeEnrollDeferCommit(promptState: promptState)
                            }
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(desktopWakeEnrollDeferCommitRuntimeOutcomeState?.phase == .dispatching)
                    }

                    if let desktopWakeEnrollDeferCommitRuntimeOutcomeState {
                        Divider()

                        Text(desktopWakeEnrollDeferCommitRuntimeOutcomeState.title)
                            .font(.headline)

                        ForEach(
                            [
                                ("dispatch_phase", desktopWakeEnrollDeferCommitRuntimeOutcomeState.phase.rawValue),
                                ("request_id", desktopWakeEnrollDeferCommitRuntimeOutcomeState.requestID),
                                ("endpoint", desktopWakeEnrollDeferCommitRuntimeOutcomeState.endpoint),
                                ("outcome", desktopWakeEnrollDeferCommitRuntimeOutcomeState.outcome ?? "not_available"),
                                ("reason", desktopWakeEnrollDeferCommitRuntimeOutcomeState.reason ?? "not_available"),
                                ("onboarding_status", desktopWakeEnrollDeferCommitRuntimeOutcomeState.onboardingStatus ?? "not_available"),
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

                        Text(desktopWakeEnrollDeferCommitRuntimeOutcomeState.summary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text(desktopWakeEnrollDeferCommitRuntimeOutcomeState.detail)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    } else {
                        Text(promptState != nil
                            ? "Awaiting explicit user-triggered canonical wake-enroll defer commit. When lawful bounded wake context remains at exact `WAKE_ENROLL`, this shell can submit one explicit defer request without authoring any local `deferred_until` value."
                            : "Read-only wake-enrollment posture only. A bounded wake-defer commit is unavailable until lawful prompt state remains present at exact `WAKE_ENROLL` from already-live wake outcome context.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }

                    Text("Only exact `WAKE_ENROLL_DEFER_COMMIT` with the exact managed bridge `deviceID` is in scope here. No local `deferred_until` authoring, no wake-listener integration, no wake-to-turn handoff, no pairing-completion mutation, no session resume / attach / reopen mutation, and no autonomous unlock claim are introduced by this surface.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            } label: {
                Text("Onboarding Wake Enrollment Defer Commit")
                    .font(.headline)
            }
        }
    }

    @ViewBuilder
    private var desktopEmoPersonaLockCard: some View {
        let promptState = desktopEmoPersonaLockPromptState
        let wakeCompleteEmoContext = (
            desktopWakeEnrollCompleteCommitRuntimeOutcomeState?.phase == .completed
            && desktopWakeEnrollCompleteCommitRuntimeOutcomeState?.nextStep == "EMO_PERSONA_LOCK"
        ) ? desktopWakeEnrollCompleteCommitRuntimeOutcomeState : nil
        let displayedOnboardingSessionID = promptState?.onboardingSessionID
            ?? desktopEmoPersonaLockRuntimeOutcomeState?.onboardingSessionID
            ?? wakeCompleteEmoContext?.onboardingSessionID
            ?? "unavailable"
        let displayedNextStep = desktopEmoPersonaLockRuntimeOutcomeState?.nextStep
            ?? promptState?.nextStep
            ?? wakeCompleteEmoContext?.nextStep
            ?? "not_provided"
        let displayedVoiceArtifactSyncReceiptRef = desktopEmoPersonaLockRuntimeOutcomeState?.voiceArtifactSyncReceiptRef
            ?? promptState?.voiceArtifactSyncReceiptRef
            ?? wakeCompleteEmoContext?.voiceArtifactSyncReceiptRef
            ?? "not_available"

        if promptState != nil || wakeCompleteEmoContext != nil || desktopEmoPersonaLockRuntimeOutcomeState != nil {
            GroupBox {
                VStack(alignment: .leading, spacing: 12) {
                    Text("Bounded desktop emo/persona-lock submission only. This shell derives bounded prompt state from already-live wake-complete outcome only when canonical onboarding posture remains at exact `EMO_PERSONA_LOCK`, dispatches exact emo/persona lock, and keeps returned `ACCESS_PROVISION`, returned `voice_artifact_sync_receipt_ref`, and any returned `EMO_PERSONA_LOCK` visibility read-only only outside the exact emo/persona-lock control itself.")
                        .frame(maxWidth: .infinity, alignment: .leading)

                    ForEach(
                        [
                            ("onboarding_session_id", displayedOnboardingSessionID),
                            ("next_step", displayedNextStep),
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

                    Text("This exact surface is dispatching canonical emo/persona lock only. No additional local device, proof, transcript, photo, or sender payload is attached to exact `EMO_PERSONA_LOCK` from this shell.")
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    if let promptState {
                        Button("Lock bounded emo/persona profile") {
                            Task {
                                await submitDesktopEmoPersonaLock(promptState: promptState)
                            }
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(desktopEmoPersonaLockRuntimeOutcomeState?.phase == .dispatching)
                    }

                    if let desktopEmoPersonaLockRuntimeOutcomeState {
                        Divider()

                        Text(desktopEmoPersonaLockRuntimeOutcomeState.title)
                            .font(.headline)

                        ForEach(
                            [
                                ("dispatch_phase", desktopEmoPersonaLockRuntimeOutcomeState.phase.rawValue),
                                ("request_id", desktopEmoPersonaLockRuntimeOutcomeState.requestID),
                                ("endpoint", desktopEmoPersonaLockRuntimeOutcomeState.endpoint),
                                ("outcome", desktopEmoPersonaLockRuntimeOutcomeState.outcome ?? "not_available"),
                                ("reason", desktopEmoPersonaLockRuntimeOutcomeState.reason ?? "not_available"),
                                ("onboarding_status", desktopEmoPersonaLockRuntimeOutcomeState.onboardingStatus ?? "not_available"),
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

                        Text(desktopEmoPersonaLockRuntimeOutcomeState.summary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text(desktopEmoPersonaLockRuntimeOutcomeState.detail)
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    } else {
                        Text(promptState != nil
                            ? "Awaiting explicit user-triggered canonical emo/persona lock. After submission, any returned exact `EMO_PERSONA_LOCK` or exact `ACCESS_PROVISION` posture plus any returned exact `voice_artifact_sync_receipt_ref` remain read-only only in this shell."
                            : "Read-only emo/persona posture only. A bounded emo/persona-lock submit is unavailable until lawful prompt state is present at exact `EMO_PERSONA_LOCK`.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }

                    Text("Only exact `EMO_PERSONA_LOCK` is in scope here. No local device/proof/transcript/photo/sender payload is attached, no access-provision controls, no complete controls, no sender-verification controls, no employee-photo controls, no wake-defer controls, and no proven native macOS wake-listener integration claim are introduced by this surface.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            } label: {
                Text("Onboarding Emo Persona Lock")
                    .font(.headline)
            }
        }
    }

    private var desktopAccessProvisionCommitCard: some View {
        let promptState = desktopAccessProvisionCommitPromptState
        let emoAccessContext = (
            desktopEmoPersonaLockRuntimeOutcomeState?.phase == .completed
            && desktopEmoPersonaLockRuntimeOutcomeState?.nextStep == "ACCESS_PROVISION"
        ) ? desktopEmoPersonaLockRuntimeOutcomeState : nil
        let displayedOnboardingSessionID = promptState?.onboardingSessionID
            ?? desktopAccessProvisionCommitRuntimeOutcomeState?.onboardingSessionID
            ?? emoAccessContext?.onboardingSessionID
            ?? "unavailable"
        let displayedNextStep = desktopAccessProvisionCommitRuntimeOutcomeState?.nextStep
            ?? promptState?.nextStep
            ?? emoAccessContext?.nextStep
            ?? "not_provided"
        let displayedVoiceArtifactSyncReceiptRef = desktopAccessProvisionCommitRuntimeOutcomeState?.voiceArtifactSyncReceiptRef
            ?? promptState?.voiceArtifactSyncReceiptRef
            ?? emoAccessContext?.voiceArtifactSyncReceiptRef
            ?? "not_available"
        let displayedAccessEngineInstanceID = desktopAccessProvisionCommitRuntimeOutcomeState?.accessEngineInstanceID
            ?? "not_available"

        return Group {
            if promptState != nil || emoAccessContext != nil || desktopAccessProvisionCommitRuntimeOutcomeState != nil {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded desktop access-provision submission only. This shell derives bounded prompt state from already-live emo/persona-lock outcome only when canonical onboarding posture remains at exact `ACCESS_PROVISION`, dispatches exact access provision commit, and keeps returned `COMPLETE`, returned `voice_artifact_sync_receipt_ref`, returned `access_engine_instance_id`, and any returned `ACCESS_PROVISION` visibility read-only only outside the exact access-provision control itself.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        ForEach(
                            [
                                ("onboarding_session_id", displayedOnboardingSessionID),
                                ("next_step", displayedNextStep),
                                ("voice_artifact_sync_receipt_ref", displayedVoiceArtifactSyncReceiptRef),
                                ("access_engine_instance_id", displayedAccessEngineInstanceID),
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

                        Text("This exact surface is dispatching canonical access provision commit only. No additional local device, proof, transcript, photo, or sender payload is attached to exact `ACCESS_PROVISION_COMMIT` from this shell.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        if let promptState {
                            Button("Commit bounded access provision") {
                                Task {
                                    await submitDesktopAccessProvisionCommit(promptState: promptState)
                                }
                            }
                            .buttonStyle(.borderedProminent)
                            .disabled(desktopAccessProvisionCommitRuntimeOutcomeState?.phase == .dispatching)
                        }

                        if let desktopAccessProvisionCommitRuntimeOutcomeState {
                            Divider()

                            Text(desktopAccessProvisionCommitRuntimeOutcomeState.title)
                                .font(.headline)

                            ForEach(
                                [
                                    ("dispatch_phase", desktopAccessProvisionCommitRuntimeOutcomeState.phase.rawValue),
                                    ("request_id", desktopAccessProvisionCommitRuntimeOutcomeState.requestID),
                                    ("endpoint", desktopAccessProvisionCommitRuntimeOutcomeState.endpoint),
                                    ("outcome", desktopAccessProvisionCommitRuntimeOutcomeState.outcome ?? "not_available"),
                                    ("reason", desktopAccessProvisionCommitRuntimeOutcomeState.reason ?? "not_available"),
                                    ("onboarding_status", desktopAccessProvisionCommitRuntimeOutcomeState.onboardingStatus ?? "not_available"),
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

                            Text(desktopAccessProvisionCommitRuntimeOutcomeState.summary)
                                .frame(maxWidth: .infinity, alignment: .leading)

                            Text(desktopAccessProvisionCommitRuntimeOutcomeState.detail)
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        } else {
                            Text(promptState != nil
                                ? "Awaiting explicit user-triggered canonical access provision commit. After submission, any returned exact `ACCESS_PROVISION` or exact `COMPLETE` posture plus any returned exact `voice_artifact_sync_receipt_ref` and exact `access_engine_instance_id` remain read-only only in this shell."
                                : "Read-only access-provision posture only. A bounded access-provision submit is unavailable until lawful prompt state is present at exact `ACCESS_PROVISION`.")
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }

                        Text("Only exact `ACCESS_PROVISION_COMMIT` is in scope here. No local device/proof/transcript/photo/sender payload is attached, no complete controls, no sender-verification controls, no employee-photo controls, no wake-defer controls, and no proven native macOS wake-listener integration claim are introduced by this surface.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Onboarding Access Provision")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopCompleteCommitCard: some View {
        let promptState = desktopCompleteCommitPromptState
        let accessCompleteContext = (
            desktopAccessProvisionCommitRuntimeOutcomeState?.phase == .completed
            && desktopAccessProvisionCommitRuntimeOutcomeState?.nextStep == "COMPLETE"
        ) ? desktopAccessProvisionCommitRuntimeOutcomeState : nil
        let displayedOnboardingSessionID = promptState?.onboardingSessionID
            ?? desktopCompleteCommitRuntimeOutcomeState?.onboardingSessionID
            ?? accessCompleteContext?.onboardingSessionID
            ?? "unavailable"
        let displayedNextStep = desktopCompleteCommitRuntimeOutcomeState?.nextStep
            ?? promptState?.nextStep
            ?? accessCompleteContext?.nextStep
            ?? "not_provided"
        let displayedVoiceArtifactSyncReceiptRef = desktopCompleteCommitRuntimeOutcomeState?.voiceArtifactSyncReceiptRef
            ?? promptState?.voiceArtifactSyncReceiptRef
            ?? accessCompleteContext?.voiceArtifactSyncReceiptRef
            ?? "not_available"
        let displayedAccessEngineInstanceID = desktopCompleteCommitRuntimeOutcomeState?.accessEngineInstanceID
            ?? promptState?.accessEngineInstanceID
            ?? accessCompleteContext?.accessEngineInstanceID
            ?? "not_available"

        return Group {
            if promptState != nil || accessCompleteContext != nil || desktopCompleteCommitRuntimeOutcomeState != nil {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded desktop complete submission only. This shell derives bounded prompt state from already-live access-provision outcome only when canonical onboarding posture remains at exact `COMPLETE`, dispatches exact complete commit, and keeps returned `READY`, returned `onboarding_status`, returned `voice_artifact_sync_receipt_ref`, returned `access_engine_instance_id`, and any returned `COMPLETE` visibility read-only only outside the exact complete control itself.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        ForEach(
                            [
                                ("onboarding_session_id", displayedOnboardingSessionID),
                                ("next_step", displayedNextStep),
                                ("voice_artifact_sync_receipt_ref", displayedVoiceArtifactSyncReceiptRef),
                                ("access_engine_instance_id", displayedAccessEngineInstanceID),
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

                        Text("This exact surface is dispatching canonical complete commit only. No additional local device, proof, transcript, photo, sender, or access payload is attached to exact `COMPLETE_COMMIT` from this shell.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        if let promptState {
                            Button("Commit bounded onboarding completion") {
                                Task {
                                    await submitDesktopCompleteCommit(promptState: promptState)
                                }
                            }
                            .buttonStyle(.borderedProminent)
                            .disabled(desktopCompleteCommitRuntimeOutcomeState?.phase == .dispatching)
                        }

                        if let desktopCompleteCommitRuntimeOutcomeState {
                            Divider()

                            Text(desktopCompleteCommitRuntimeOutcomeState.title)
                                .font(.headline)

                            ForEach(
                                [
                                    ("dispatch_phase", desktopCompleteCommitRuntimeOutcomeState.phase.rawValue),
                                    ("request_id", desktopCompleteCommitRuntimeOutcomeState.requestID),
                                    ("endpoint", desktopCompleteCommitRuntimeOutcomeState.endpoint),
                                    ("outcome", desktopCompleteCommitRuntimeOutcomeState.outcome ?? "not_available"),
                                    ("reason", desktopCompleteCommitRuntimeOutcomeState.reason ?? "not_available"),
                                    ("onboarding_status", desktopCompleteCommitRuntimeOutcomeState.onboardingStatus ?? "not_available"),
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

                            Text(desktopCompleteCommitRuntimeOutcomeState.summary)
                                .frame(maxWidth: .infinity, alignment: .leading)

                            Text(desktopCompleteCommitRuntimeOutcomeState.detail)
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        } else {
                            Text(promptState != nil
                                ? "Awaiting explicit user-triggered canonical complete commit. After submission, any returned exact `COMPLETE` or exact `READY` posture plus any returned exact `onboarding_status`, exact `voice_artifact_sync_receipt_ref`, and exact `access_engine_instance_id` remain read-only only in this shell."
                                : "Read-only completion posture only. A bounded complete submit is unavailable until lawful prompt state is present at exact `COMPLETE`.")
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }

                        Text("Only exact `COMPLETE_COMMIT` is in scope here. No local device/proof/transcript/photo/sender/access payload is attached, no pairing-completion controls, no sender-verification controls, no employee-photo controls, no wake-defer controls, and no proven native macOS wake-listener integration claim are introduced by this surface.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Onboarding Completion")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopReadyVisibilityCard: some View {
        let readyVisibilityState = desktopReadyVisibilityState

        return Group {
            if let readyVisibilityState {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded desktop ready visibility only. This shell derives bounded read-only posture from the already-live complete-commit outcome only when canonical onboarding posture has advanced to exact `READY`, enriches it with the exact managed bridge `deviceID` when available, and keeps pairing completion mutation plus any onboarding-derived ready-time behavior out of scope.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        let displayedRows: [(String, String)] = {
                            var rows: [(String, String)] = [
                                ("onboarding_session_id", readyVisibilityState.onboardingSessionID),
                                ("next_step", readyVisibilityState.nextStep),
                            ]

                            if let onboardingStatus = readyVisibilityState.onboardingStatus {
                                rows.append(("onboarding_status", onboardingStatus))
                            }

                            if let voiceArtifactSyncReceiptRef = readyVisibilityState.voiceArtifactSyncReceiptRef {
                                rows.append(("voice_artifact_sync_receipt_ref", voiceArtifactSyncReceiptRef))
                            }

                            if let accessEngineInstanceID = readyVisibilityState.accessEngineInstanceID {
                                rows.append(("access_engine_instance_id", accessEngineInstanceID))
                            }

                            if let deviceID = readyVisibilityState.deviceID {
                                rows.append(("device_id", deviceID))
                            }

                            return rows
                        }()

                        ForEach(displayedRows, id: \.0) { row in
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

                        Text("Canonical onboarding posture has advanced to exact `READY`. This card is read-only only and preserves returned exact `onboarding_status`, exact `voice_artifact_sync_receipt_ref`, exact `access_engine_instance_id`, and the exact managed bridge `deviceID` when available.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text("Pairing completion mutation, onboarding-derived ready-time local attach behavior, wake-listener integration, wake-to-turn behavior, and autonomous unlock remain out of scope here. This shell stays explicitly non-authoritative.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Onboarding Ready Visibility")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopPairingCompletionVisibilityCard: some View {
        let pairingCompletionVisibilityState = desktopPairingCompletionVisibilityState

        return Group {
            if let pairingCompletionVisibilityState {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded desktop pairing-completion visibility only. This shell derives bounded read-only pairing/session continuity posture from the already-live exact `READY` visibility plus one already-live cloud-authored session surface only when exact `session_attach_outcome` remains exact `NEW_SESSION_CREATED` or exact `EXISTING_SESSION_ATTACHED`.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        let displayedRows: [(String, String)] = {
                            var rows: [(String, String)] = [
                                ("onboarding_session_id", pairingCompletionVisibilityState.onboardingSessionID),
                                ("next_step", pairingCompletionVisibilityState.nextStep),
                            ]

                            if let onboardingStatus = pairingCompletionVisibilityState.onboardingStatus {
                                rows.append(("onboarding_status", onboardingStatus))
                            }

                            if let voiceArtifactSyncReceiptRef = pairingCompletionVisibilityState.voiceArtifactSyncReceiptRef {
                                rows.append(("voice_artifact_sync_receipt_ref", voiceArtifactSyncReceiptRef))
                            }

                            if let accessEngineInstanceID = pairingCompletionVisibilityState.accessEngineInstanceID {
                                rows.append(("access_engine_instance_id", accessEngineInstanceID))
                            }

                            if let deviceID = pairingCompletionVisibilityState.deviceID {
                                rows.append(("device_id", deviceID))
                            }

                            rows.append(("session_state", pairingCompletionVisibilityState.sessionState))
                            rows.append(("session_id", pairingCompletionVisibilityState.sessionID))
                            rows.append(("session_attach_outcome", pairingCompletionVisibilityState.sessionAttachOutcome))

                            if let turnID = pairingCompletionVisibilityState.turnID {
                                rows.append(("turn_id", turnID))
                            }

                            return rows
                        }()

                        ForEach(displayedRows, id: \.0) { row in
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

                        Text(continuityLabel(for: pairingCompletionVisibilityState.sessionAttachOutcome))
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text("Canonical onboarding posture has advanced to exact `READY`, and cloud-authoritative pairing/session continuity is now visible in bounded read-only form only through this surface. Exact `RETRY_REUSED_RESULT` remains in the broader generic session continuity surface only and is not reclassified here.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text("Pairing completion mutation, local session attach or reopen authority, onboarding-derived ready-time local handoff behavior, wake-listener integration, wake-to-turn behavior, and autonomous unlock remain out of scope here. This shell stays explicitly non-authoritative.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Onboarding Pairing Completion Visibility")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopPairingCompletionMutationCard: some View {
        let promptState = desktopPairingCompletionPromptState
        let runtimeOutcomeState = desktopPairingCompletionCommitRuntimeOutcomeState
        let displayedSourceSurface = promptState?.sourceSurfaceIdentity
            ?? runtimeOutcomeState?.sourceSurfaceIdentity
            ?? desktopReadyTimeHandoffState?.sourceSurfaceIdentity
            ?? "PAIRING_COMPLETION_VISIBLE"
        let displayedOnboardingSessionID = promptState?.onboardingSessionID
            ?? runtimeOutcomeState?.onboardingSessionID
            ?? desktopReadyTimeHandoffState?.onboardingSessionID
            ?? "not_provided"
        let displayedNextStep = promptState?.nextStep
            ?? runtimeOutcomeState?.nextStep
            ?? desktopReadyTimeHandoffState?.nextStep
            ?? "not_provided"
        let displayedOnboardingStatus = promptState?.onboardingStatus
            ?? runtimeOutcomeState?.onboardingStatus
            ?? desktopReadyTimeHandoffState?.onboardingStatus
        let displayedVoiceArtifactSyncReceiptRef = promptState?.voiceArtifactSyncReceiptRef
            ?? runtimeOutcomeState?.voiceArtifactSyncReceiptRef
            ?? desktopReadyTimeHandoffState?.voiceArtifactSyncReceiptRef
        let displayedAccessEngineInstanceID = promptState?.accessEngineInstanceID
            ?? runtimeOutcomeState?.accessEngineInstanceID
            ?? desktopReadyTimeHandoffState?.accessEngineInstanceID
        let displayedDeviceID = promptState?.deviceID
            ?? runtimeOutcomeState?.deviceID
            ?? desktopReadyTimeHandoffState?.deviceID
        let displayedSessionState = promptState?.sessionState
            ?? runtimeOutcomeState?.sessionState
            ?? desktopReadyTimeHandoffState?.sessionState
            ?? "not_provided"
        let displayedSessionID = promptState?.sessionID
            ?? runtimeOutcomeState?.sessionID
            ?? desktopReadyTimeHandoffState?.sessionID
            ?? "not_provided"
        let displayedSessionAttachOutcome = promptState?.sessionAttachOutcome
            ?? runtimeOutcomeState?.sessionAttachOutcome
            ?? desktopReadyTimeHandoffState?.sessionAttachOutcome
            ?? "not_provided"
        let displayedTurnID = promptState?.turnID
            ?? runtimeOutcomeState?.turnID
            ?? desktopReadyTimeHandoffState?.turnID

        return Group {
            if promptState != nil || runtimeOutcomeState != nil {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded desktop pairing-completion commit plus onboarding-derived ready-time handoff only. This shell derives one bounded prompt state from the already-live exact pairing-completion visibility surface only, dispatches only exact `/v1/onboarding/continue` with exact `PAIRING_COMPLETION_COMMIT`, and foregrounds the already-live operational desktop surfaces only after canonical success still matches the visible lawful pairing prompt.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        metadataRow(label: "source_surface", value: displayedSourceSurface)
                        metadataRow(label: "onboarding_session_id", value: displayedOnboardingSessionID)
                        metadataRow(label: "next_step", value: displayedNextStep)

                        if let displayedOnboardingStatus {
                            metadataRow(label: "onboarding_status", value: displayedOnboardingStatus)
                        }

                        if let displayedVoiceArtifactSyncReceiptRef {
                            metadataRow(
                                label: "voice_artifact_sync_receipt_ref",
                                value: displayedVoiceArtifactSyncReceiptRef
                            )
                        }

                        if let displayedAccessEngineInstanceID {
                            metadataRow(label: "access_engine_instance_id", value: displayedAccessEngineInstanceID)
                        }

                        if let displayedDeviceID {
                            metadataRow(label: "device_id", value: displayedDeviceID)
                        }

                        metadataRow(label: "session_state", value: displayedSessionState)
                        metadataRow(label: "session_id", value: displayedSessionID)
                        metadataRow(label: "session_attach_outcome", value: displayedSessionAttachOutcome)

                        if let displayedTurnID {
                            metadataRow(label: "turn_id", value: displayedTurnID)
                        }

                        Text("Pairing/session continuity remains cloud-authored and non-authoritative here. This exact canonical commit only foregrounds already-live desktop operational surfaces while keeping ready and pairing continuity details bounded and read-only outside the explicit control itself.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text("This path does not add generic reopen authority, broader session-list fetch authority, search input, tool controls, hidden/background wake auto-start, wake parity, or autonomous unlock.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        if let promptState, !desktopReadyTimeHandoffIsActive {
                            Button("Submit canonical pairing completion commit") {
                                Task {
                                    await submitDesktopPairingCompletionCommit(promptState: promptState)
                                }
                            }
                            .buttonStyle(.borderedProminent)
                            .disabled(runtimeOutcomeState?.phase == .dispatching)
                        }

                        if let runtimeOutcomeState {
                            Divider()

                            Text(runtimeOutcomeState.title)
                                .font(.headline)

                            metadataRow(
                                label: "mutation_phase",
                                value: runtimeOutcomeState.phase.rawValue
                            )
                            metadataRow(
                                label: "request_id",
                                value: runtimeOutcomeState.requestID
                            )
                            metadataRow(
                                label: "endpoint",
                                value: runtimeOutcomeState.endpoint
                            )
                            metadataRow(
                                label: "outcome",
                                value: runtimeOutcomeState.outcome ?? "not_available"
                            )
                            metadataRow(
                                label: "reason",
                                value: runtimeOutcomeState.reason ?? "not_available"
                            )

                            if let returnedOnboardingStatus = runtimeOutcomeState.onboardingStatus {
                                metadataRow(label: "returned_onboarding_status", value: returnedOnboardingStatus)
                            }

                            Text(runtimeOutcomeState.summary)
                                .frame(maxWidth: .infinity, alignment: .leading)

                            Text(runtimeOutcomeState.detail)
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        } else {
                            Text(promptState != nil
                                ? "Awaiting one explicit user-triggered canonical pairing-completion commit. After canonical success still matches the visible lawful pairing prompt, the shell will foreground the already-live session, history, system-activity, needs-attention, and explicit-voice desktop surfaces while keeping this path bounded and non-authoritative."
                                : "Read-only pairing posture only. A bounded canonical pairing-completion commit is unavailable until lawful prompt state is present with exact `READY` posture plus exact `NEW_SESSION_CREATED` or exact `EXISTING_SESSION_ATTACHED` continuity.")
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }

                        Text("Cloud-authored and non-authoritative only. No text field, no hidden/background behavior, no wake parity claim, and no proven native macOS wake-listener integration claim are introduced by this surface.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Onboarding Pairing Completion Commit")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopWakeProfileAvailabilityCard: some View {
        let promptState = desktopWakeProfileAvailabilityPromptState
        let displayedReceiptKind = promptState?.receiptKind
            ?? desktopWakeProfileAvailabilityRuntimeOutcomeState?.receiptKind
            ?? "desktop_wakeword_configured"
        let displayedDeviceID = promptState?.deviceID
            ?? desktopWakeProfileAvailabilityRuntimeOutcomeState?.deviceID
            ?? "not_provided"
        let displayedWakeProfileID = promptState?.wakeProfileID
            ?? desktopWakeProfileAvailabilityRuntimeOutcomeState?.wakeProfileID
            ?? "not_provided"
        let displayedVoiceArtifactSyncReceiptRef = promptState?.voiceArtifactSyncReceiptRef
            ?? desktopWakeProfileAvailabilityRuntimeOutcomeState?.voiceArtifactSyncReceiptRef
            ?? "not_provided"

        return Group {
            if promptState != nil || desktopWakeProfileAvailabilityRuntimeOutcomeState != nil {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded desktop wake-profile local-availability refresh only. This shell derives one prompt from the already-live exact `desktop_wakeword_configured` proof carrier family only while exact local ready-time handoff remains active, dispatches one explicit refresh into canonical `/v1/wake-profile/availability`, and keeps returned local wake-artifact availability read-only and non-authoritative.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        metadataRow(label: "receipt_kind", value: displayedReceiptKind)
                        metadataRow(label: "device_id", value: displayedDeviceID)
                        metadataRow(label: "wake_profile_id", value: displayedWakeProfileID)
                        metadataRow(
                            label: "voice_artifact_sync_receipt_ref",
                            value: displayedVoiceArtifactSyncReceiptRef
                        )

                        Text("Wake configuration proof, wake runtime evidence, and any later wake entry remain cloud-authored and non-authoritative here.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text("This path refreshes local wake-artifact availability only and does not add native wake-listener start or stop, wake detection, wake-to-turn dispatch, hidden/background auto-start, or autonomous unlock.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        if let promptState {
                            Button("Refresh local wake-profile availability") {
                                Task {
                                    await submitDesktopWakeProfileAvailabilityRefresh(
                                        promptState: promptState
                                    )
                                }
                            }
                            .buttonStyle(.borderedProminent)
                            .disabled(
                                desktopWakeProfileAvailabilityRuntimeOutcomeState?.phase
                                    == .dispatching
                            )
                        }

                        if let desktopWakeProfileAvailabilityRuntimeOutcomeState {
                            Divider()

                            Text(desktopWakeProfileAvailabilityRuntimeOutcomeState.title)
                                .font(.headline)

                            ForEach(
                                [
                                    ("outcome", desktopWakeProfileAvailabilityRuntimeOutcomeState.outcome ?? "not_available"),
                                    ("reason", desktopWakeProfileAvailabilityRuntimeOutcomeState.reason ?? "not_available"),
                                    ("wake_profile_id", desktopWakeProfileAvailabilityRuntimeOutcomeState.wakeProfileID),
                                    ("active_wake_artifact_version", desktopWakeProfileAvailabilityRuntimeOutcomeState.activeWakeArtifactVersion ?? "not_available"),
                                    ("activated_count", "\(desktopWakeProfileAvailabilityRuntimeOutcomeState.activatedCount)"),
                                    ("noop_count", "\(desktopWakeProfileAvailabilityRuntimeOutcomeState.noopCount)"),
                                    ("pull_error_count", "\(desktopWakeProfileAvailabilityRuntimeOutcomeState.pullErrorCount)"),
                                ],
                                id: \.0
                            ) { row in
                                metadataRow(label: row.0, value: row.1)
                            }

                            Text(desktopWakeProfileAvailabilityRuntimeOutcomeState.summary)
                                .frame(maxWidth: .infinity, alignment: .leading)

                            Text(desktopWakeProfileAvailabilityRuntimeOutcomeState.detail)
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        } else {
                            Text(promptState != nil
                                ? "Awaiting explicit user-triggered local wake-profile availability refresh. After submission, any returned exact `wake_profile_id`, exact `active_wake_artifact_version`, and bounded worker-pass refresh outcome remain read-only only in this shell."
                                : "Read-only wakeword-configured proof posture only. A bounded local wake-profile availability refresh is unavailable until exact ready-time handoff is active and the exact proof carrier family remains lawful.")
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }

                        Text("No text field, no hidden/background behavior, no wake parity claim, and no autonomous unlock claim are introduced by this surface.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Wake Profile Availability Refresh")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopWakeListenerControlCard: some View {
        let promptState = desktopWakeListenerPromptState
        let displayedRequestState = desktopWakeListenerController.pendingRequest
            ?? lastStagedWakeTriggeredVoiceTurnRequestState
        let shouldAttachWakeTriggeredVoicePendingAttachment =
            desktopOperationalConversationShellState.map { operationalConversationShellState in
                desktopConversationShouldAttachWakeTriggeredVoicePendingAttachment(
                    operationalConversationShellState.primaryPaneState.wakeTriggeredVoicePendingAttachmentState
                )
            } ?? false
        let shouldAttachWakeTriggeredVoiceFailedRequest =
            desktopOperationalConversationShellState.map { operationalConversationShellState in
                desktopConversationShouldAttachWakeTriggeredVoiceFailedRequest(
                    operationalConversationShellState.primaryPaneState.wakeTriggeredVoiceFailedRequestAttachmentState
                )
            } ?? false

        return Group {
            if promptState != nil
                || desktopWakeListenerController.listenerState != .idle
                || displayedRequestState != nil
                || desktopWakeListenerController.failedRequest != nil {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded native macOS foreground wake-listener integration only. This shell derives one lawful prompt from already-live local wake-profile availability and exact wakeword-configured proof, may auto-start one already-live foreground wake listener only while the exact visible shell is active, still allows explicit user start/stop, stages one bounded post-wake transcript remainder only after exact `Selene` prefix detection, and hands that request into canonical `/v1/voice/turn` as exact `WAKE_WORD` while remaining non-authoritative.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        metadataRow(
                            label: "source_surface",
                            value: "WAKE_PROFILE_LOCAL_AVAILABILITY_VISIBLE"
                        )
                        metadataRow(
                            label: "receipt_kind",
                            value: promptState?.receiptKind ?? "desktop_wakeword_configured"
                        )
                        metadataRow(
                            label: "device_id",
                            value: promptState?.deviceID ?? "not_provided"
                        )
                        metadataRow(
                            label: "wake_profile_id",
                            value: promptState?.wakeProfileID ?? "not_provided"
                        )
                        metadataRow(
                            label: "active_wake_artifact_version",
                            value: promptState?.activeWakeArtifactVersion ?? "not_provided"
                        )
                        metadataRow(
                            label: "voice_artifact_sync_receipt_ref",
                            value: promptState?.voiceArtifactSyncReceiptRef ?? "not_provided"
                        )
                        metadataRow(
                            label: "wake_trigger_phrase",
                            value: promptState?.wakeTriggerPhrase ?? "Selene"
                        )
                        metadataRow(
                            label: "listener_state",
                            value: desktopWakeListenerController.listenerState.rawValue
                        )
                        if !shouldAttachWakeTriggeredVoicePendingAttachment {
                            metadataRow(
                                label: "wake_request_id",
                                value: displayedRequestState?.id ?? "not_available"
                            )
                            metadataRow(
                                label: "wake_transcript_preview",
                                value: displayedRequestState?.boundedPreview ?? "not_available"
                            )
                        }

                        Text("Canonical runtime response posture, session continuity, wake runtime evidence, and local wake-artifact availability remain cloud-authored or bridge-bounded and non-authoritative here.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text("This path reuses existing exact `/v1/voice/turn` and exact H288 structured `audioCaptureRef`, and does not add backend mutation, thread authoring, pinned-context authoring, device-turn-sequence authoring, hidden/background auto-start, or autonomous unlock.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        HStack(spacing: 12) {
                            if let promptState {
                                Button("Start foreground wake listener") {
                                    startDesktopWakeListener(promptState: promptState)
                                }
                                .buttonStyle(.borderedProminent)
                                .disabled(
                                    desktopWakeListenerController.listenerState.isActiveForMicrophone
                                        || desktopWakeListenerController.pendingRequest != nil
                                        || explicitVoiceController.isListening
                                        || explicitVoiceController.pendingRequest != nil
                                )
                            }

                            if desktopWakeListenerController.listenerState.isActiveForMicrophone {
                                Button("Stop wake listener") {
                                    stopDesktopWakeListenerAndSuppressAutoStart(promptState: promptState)
                                }
                                .buttonStyle(.bordered)
                            }
                        }

                        if let failedRequest = desktopWakeListenerController.failedRequest {
                            if shouldAttachWakeTriggeredVoiceFailedRequest {
                                EmptyView()
                            } else {
                                interruptResponseFailedRequestCard(failedRequest)
                            }
                        } else if promptState != nil
                            && desktopWakeListenerController.listenerState == .idle
                            && displayedRequestState == nil {
                            Text("Awaiting one bounded foreground wake-listener start from the active visible shell, either through the local visible-shell auto-start seam or explicit user start. After exact local wake-prefix detection and one non-empty post-wake transcript remainder, this shell stages one canonical wake-triggered handoff only and remains non-authoritative.")
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }

                        Text("No text field, no threshold editor, no true hidden/background behavior, no wake parity claim, and no autonomous unlock claim are introduced by this surface.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Wake Listener Control")
                        .font(.headline)
                }
            }
        }
    }

    private func desktopOperationalConversationShell(
        _ state: DesktopOperationalConversationShellState
    ) -> some View {
        HStack(alignment: .top, spacing: 20) {
            ScrollView {
                desktopConversationPrimaryPane(state.primaryPaneState)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)

            ScrollView {
                desktopConversationSupportRail(state.supportRailState)
            }
            .frame(width: 340)
            .frame(maxHeight: .infinity, alignment: .topLeading)
        }
    }

    private func desktopConversationPrimaryPane(
        _ state: DesktopConversationPrimaryPaneState
    ) -> some View {
        let shouldAttachSearchToolCompletion = desktopConversationShouldAttachSearchToolCompletion(
            state.searchToolCompletionState
        )
        let shouldAttachAuthoritativeReplyCompletion =
            desktopConversationShouldAttachAuthoritativeReplyCompletion(
                state.authoritativeReplyCompletionState
            )
        let shouldAttachRuntimeDispatchFailure =
            desktopConversationShouldAttachRuntimeDispatchFailure(
                state.runtimeDispatchFailureAttachmentState
            )
        let shouldAttachRuntimeDispatchFailureInline = shouldAttachRuntimeDispatchFailure
            && state.timelineEntries.contains(where: { entry in
                desktopConversationShouldAttachRuntimeDispatchFailureInline(
                    to: entry,
                    runtimeDispatchFailureAttachmentState: state.runtimeDispatchFailureAttachmentState
                )
            })
        let shouldAttachRuntimeCompletedWithoutInlineReply =
            desktopConversationShouldAttachRuntimeCompletedWithoutInlineReply(
                desktopCanonicalRuntimeOutcomeState,
                searchToolCompletionState: state.searchToolCompletionState,
                authoritativeReplyCompletionState: state.authoritativeReplyCompletionState
            )
        let shouldAttachRuntimeCompletedWithoutInlineReplyInline =
            shouldAttachRuntimeCompletedWithoutInlineReply
            && state.timelineEntries.contains(where: { entry in
                entry.posture == "runtime_completed_without_inline_reply_preview"
                    && entry.sourceSurface == "CANONICAL_RUNTIME_COMPLETED_WITHOUT_INLINE_REPLY"
            })

        return VStack(alignment: .leading, spacing: 18) {
            VStack(alignment: .leading, spacing: 12) {
                HStack(alignment: .top, spacing: 16) {
                    VStack(alignment: .leading, spacing: 6) {
                        Text(state.headerTitle)
                            .font(.largeTitle.weight(.semibold))

                        Text(state.headerDetail)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }

                    Spacer(minLength: 16)

                    desktopConversationVoiceStateBadge(state.voiceState)
                }
            }
            .padding(20)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Color(nsColor: .controlBackgroundColor))
            .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))

            if state.dominantPosture == "SESSION_SUSPENDED_VISIBLE" {
                sectionCard(
                    title: "Conversation Status",
                    detail: "Suspended posture remains explanation-only on macOS in this run. No transcript authority, no archive fabrication, no hidden continuation, and no local unsuspend path are introduced while the authoritative runtime keeps the session suspended."
                )
            } else if state.dominantPosture == "QUARANTINED_LOCAL_STATE" {
                sectionCard(
                    title: "Conversation Status",
                    detail: "QUARANTINED_LOCAL_STATE withholds live dual transcript and archived recent-slice visibility from this transcript-primary pane until authoritative reread clears the recovery posture cloud-side."
                )
            } else {
                if let desktopCanonicalRuntimeOutcomeState,
                   !shouldAttachSearchToolCompletion,
                   !shouldAttachAuthoritativeReplyCompletion,
                   !shouldAttachRuntimeDispatchFailure,
                   !shouldAttachRuntimeCompletedWithoutInlineReplyInline {
                    desktopCanonicalRuntimeOutcomeCard(desktopCanonicalRuntimeOutcomeState)
                }

                if state.timelineEntries.isEmpty {
                    sectionCard(
                        title: "Conversation Timeline",
                        detail: desktopForegroundSelectionShowsCurrentDominantSurface
                            ? "Awaiting the next lawful cloud-authored turn. Use the bounded keyboard composer, the bounded search-request authoring card, the bounded tool-request authoring card, explicit voice, or the bounded foreground wake listener to start runtime dispatch from this transcript-primary shell without introducing hidden/background wake behavior or fake local authority."
                            : "Awaiting read-only foreground visibility for the selected observed session surface. Local selection does not itself attach, resume, recover, reopen, or retarget canonical runtime mutation."
                    )
                } else {
                    VStack(alignment: .leading, spacing: 14) {
                        ForEach(state.timelineEntries) { entry in
                            desktopConversationTimelineEntryCard(
                                entry,
                                explicitVoiceLivePreviewAttachmentState: state.explicitVoiceLivePreviewAttachmentState,
                                wakeTriggeredVoiceLivePreviewAttachmentState: state.wakeTriggeredVoiceLivePreviewAttachmentState,
                                explicitVoiceFailedRequestAttachmentState: state.explicitVoiceFailedRequestAttachmentState,
                                wakeTriggeredVoiceFailedRequestAttachmentState: state.wakeTriggeredVoiceFailedRequestAttachmentState,
                                explicitVoicePendingAttachmentState: state.explicitVoicePendingAttachmentState,
                                wakeTriggeredVoicePendingAttachmentState: state.wakeTriggeredVoicePendingAttachmentState,
                                readOnlyToolLaneState: state.readOnlyToolLaneState,
                                searchToolCompletionState: state.searchToolCompletionState,
                                authoritativeReplyCompletionState: state.authoritativeReplyCompletionState,
                                runtimeDispatchFailureAttachmentState: state.runtimeDispatchFailureAttachmentState
                            )
                        }

                        if shouldAttachRuntimeDispatchFailure,
                           !shouldAttachRuntimeDispatchFailureInline,
                           let runtimeDispatchFailureAttachmentState = state.runtimeDispatchFailureAttachmentState {
                            desktopConversationRuntimeDispatchFailureAttachment(
                                runtimeDispatchFailureAttachmentState
                            )
                        }
                    }
                }

                if desktopForegroundSelectionShowsCurrentDominantSurface {
                    desktopTypedTurnComposerCard
                } else {
                    sectionCard(
                        title: "Selected Surface Status",
                        detail: "This previously observed session surface is foregrounded in bounded read-only form only. Existing attach / resume / recover controls remain separate, and bounded keyboard typed-turn plus bounded search-request and tool-request production stay bound to the current lawful dominant desktop surface."
                    )
                }
            }
        }
        .frame(maxWidth: .infinity, alignment: .topLeading)
    }

    private func desktopConversationSupportRail(
        _ state: DesktopConversationSupportRailState
    ) -> some View {
        VStack(alignment: .leading, spacing: 16) {
            GroupBox {
                VStack(alignment: .leading, spacing: 10) {
                    Text(state.detail)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    ForEach(state.supportSurfaceLabels, id: \.self) { surfaceLabel in
                        Text(surfaceLabel)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }

                    Text("Controls and evidence-adjacent support remain bounded, session-bound, and non-authoritative here. One bounded local observed-session selection rail plus one bounded current-device recent-session visibility card, one bounded search-request authoring card, and one bounded tool-request authoring card are now available, but they still do not introduce local attach or reopen authority, standalone local search execution, or hidden/background wake behavior.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            } label: {
                Text(state.title)
                    .font(.headline)
            }

            desktopSessionSurfaceSelectionRailCard
            desktopSessionRecentListVisibilityCard
            desktopSearchRequestAuthoringCard
            desktopToolRequestAuthoringCard
            explicitVoiceEntryAffordanceCard
            desktopWakeProfileAvailabilityCard
            desktopWakeListenerControlCard
            posturePanel
            historyCard
            desktopSessionMultiPostureEntryCard
            desktopSessionSoftClosedVisibilityCard
            desktopSessionSuspendedVisibilityCard
            desktopRecoveryVisibilityCard
            desktopInterruptVisibilityCard
            desktopInterruptResponseProductionCard
            desktopInterruptSubjectReferencesVisibilityCard
            desktopInterruptSubjectRelationConfidenceVisibilityCard
            desktopInterruptReturnCheckExpiryVisibilityCard
            sessionCard
            systemActivityCard
            needsAttentionCard
        }
        .frame(maxWidth: .infinity, alignment: .topLeading)
    }

    private var desktopSessionSurfaceSelectionRailCard: some View {
        let foregroundSurfaceID = foregroundObservedSessionSurface?.id

        return Group {
            if !observedSessionSurfaces.isEmpty {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded local observed-session selection only. This rail foregrounds already-observed cloud-authored session surfaces from the current shell lifetime and changes visibility only; it does not itself attach, resume, recover, reopen, or grant local authority.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        ForEach(observedSessionSurfaces) { surface in
                            let isForegrounded = surface.id == foregroundSurfaceID
                            let isCurrentDominant = surface.id == currentDominantObservedSessionSurface?.id

                            Button {
                                selectObservedSessionSurface(surface)
                            } label: {
                                VStack(alignment: .leading, spacing: 8) {
                                    HStack(alignment: .center, spacing: 8) {
                                        Text(surface.selectionTitle)
                                            .font(.subheadline.weight(.semibold))
                                            .frame(maxWidth: .infinity, alignment: .leading)

                                        if isCurrentDominant {
                                            posturePill("CURRENT")
                                        }

                                        posturePill(surface.postureLabel.uppercased())
                                    }

                                    Text(surface.selectionSummary)
                                        .font(.footnote)
                                        .foregroundStyle(.secondary)
                                        .frame(maxWidth: .infinity, alignment: .leading)

                                    HStack(alignment: .firstTextBaseline, spacing: 10) {
                                        Text("session_id")
                                            .font(.caption.monospaced())
                                            .foregroundStyle(.secondary)

                                        Text(surface.sessionID)
                                            .font(.caption.monospaced())
                                            .foregroundStyle(.secondary)
                                    }
                                }
                                .padding(12)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .background(
                                    isForegrounded
                                        ? Color.accentColor.opacity(0.12)
                                        : Color(nsColor: .controlBackgroundColor)
                                )
                                .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
                            }
                            .buttonStyle(.plain)
                        }

                        Text("Foreground selection fails closed to the current lawful dominant session surface whenever no selected observed surface remains available.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Session Surface Selection")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopSessionRecentListVisibilityCard: some View {
        let rowStates = desktopSessionRecentListRowStates

        return GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Bounded current-device recent-session visibility only. This shell consumes exact `/v1/session/recent`, preserves exact returned row order only, and keeps these recent-session rows separate from the already-observed local session-surface rail.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                if let outcomeState = desktopSessionRecentListRuntimeOutcomeState {
                    Text(outcomeState.title)
                        .font(.subheadline.weight(.semibold))

                    ForEach(
                        [
                            ("dispatch_phase", outcomeState.phase.rawValue),
                            ("request_id", outcomeState.requestID),
                            ("endpoint", outcomeState.endpoint),
                            ("outcome", outcomeState.outcome ?? "not_available"),
                            ("reason", outcomeState.reason ?? "not_available"),
                            ("device_id", outcomeState.deviceID),
                        ],
                        id: \.0
                    ) { row in
                        metadataRow(label: row.0, value: row.1)
                    }

                    Text(outcomeState.summary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    Text(outcomeState.detail)
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                    if outcomeState.phase == .completed {
                        if rowStates.isEmpty {
                            Text("No current-device recent-session rows are currently visible from exact `/v1/session/recent` for this managed desktop device.")
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        } else {
                            ForEach(rowStates) { rowState in
                                VStack(alignment: .leading, spacing: 8) {
                                    HStack(alignment: .center, spacing: 8) {
                                        Text(rowState.title)
                                            .font(.subheadline.weight(.semibold))
                                            .frame(maxWidth: .infinity, alignment: .leading)

                                        posturePill(rowState.sessionState)
                                    }

                                    Text(rowState.summary)
                                        .font(.footnote)
                                        .foregroundStyle(.secondary)
                                        .frame(maxWidth: .infinity, alignment: .leading)

                                    metadataRow(label: "session_state", value: rowState.sessionState)
                                    metadataRow(label: "session_id", value: rowState.sessionID)
                                    metadataRow(
                                        label: "last_turn_id",
                                        value: rowState.lastTurnID ?? "not_available"
                                    )
                                }
                                .padding(12)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .background(Color(nsColor: .controlBackgroundColor))
                                .clipShape(
                                    RoundedRectangle(cornerRadius: 14, style: .continuous)
                                )
                            }
                        }
                    }
                } else {
                    Text("Awaiting bounded current-device recent-session visibility refresh. This shell fails closed until exact `/v1/session/recent` returns lawful current-device recent-session rows in read-only form.")
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("Recent-session rows remain evidence-only here: no synthetic transcript surface, no merge into `observedSessionSurfaces`, and no recent-row-driven attach / resume / recover or reopen authority are introduced by this card.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Current-Device Recent Sessions")
                .font(.headline)
        }
    }

    private func desktopConversationTimelineEntryCard(
        _ entry: DesktopConversationTimelineEntryState,
        explicitVoiceLivePreviewAttachmentState: DesktopConversationExplicitVoiceLivePreviewAttachmentState?,
        wakeTriggeredVoiceLivePreviewAttachmentState: DesktopConversationWakeTriggeredVoiceLivePreviewAttachmentState?,
        explicitVoiceFailedRequestAttachmentState: DesktopConversationExplicitVoiceFailedRequestAttachmentState?,
        wakeTriggeredVoiceFailedRequestAttachmentState: DesktopConversationWakeTriggeredVoiceFailedRequestAttachmentState?,
        explicitVoicePendingAttachmentState: DesktopConversationExplicitVoicePendingAttachmentState?,
        wakeTriggeredVoicePendingAttachmentState: DesktopConversationWakeTriggeredVoicePendingAttachmentState?,
        readOnlyToolLaneState: DesktopConversationReadOnlyToolLaneState?,
        searchToolCompletionState: DesktopConversationSearchToolCompletionState?,
        authoritativeReplyCompletionState: DesktopConversationAuthoritativeReplyCompletionState?,
        runtimeDispatchFailureAttachmentState: DesktopConversationRuntimeDispatchFailureAttachmentState?
    ) -> some View {
        let bubbleColor = entry.isUserAuthored
            ? Color.accentColor.opacity(0.12)
            : Color(nsColor: .controlBackgroundColor)

        return HStack(alignment: .top, spacing: 0) {
            if entry.isUserAuthored {
                Spacer(minLength: 72)
            }

            VStack(alignment: .leading, spacing: 10) {
                HStack(alignment: .firstTextBaseline, spacing: 10) {
                    Text(entry.speaker)
                        .font(.subheadline.weight(.semibold))

                    Text(entry.sourceSurface)
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                }

                Text(entry.body)
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("\(entry.posture): \(entry.detail)")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                if entry.posture == "explicit_voice_live_preview",
                   entry.sourceSurface == "EXPLICIT_VOICE_LISTENING",
                   desktopConversationShouldAttachExplicitVoiceLivePreview(
                       explicitVoiceLivePreviewAttachmentState
                   ),
                   let explicitVoiceLivePreviewAttachmentState {
                    desktopConversationExplicitVoiceLivePreviewAttachment(
                        explicitVoiceLivePreviewAttachmentState
                    )
                }

                if entry.posture == "wake_voice_live_preview",
                   entry.sourceSurface == "WAKE_TRIGGERED_VOICE_LISTENING",
                   desktopConversationShouldAttachWakeTriggeredVoiceLivePreview(
                       wakeTriggeredVoiceLivePreviewAttachmentState
                   ),
                   let wakeTriggeredVoiceLivePreviewAttachmentState {
                    desktopConversationWakeTriggeredVoiceLivePreviewAttachment(
                        wakeTriggeredVoiceLivePreviewAttachmentState
                    )
                }

                if entry.posture == "explicit_voice_failed_request_preview",
                   entry.sourceSurface == "EXPLICIT_VOICE_FAILED_REQUEST",
                   desktopConversationShouldAttachExplicitVoiceFailedRequest(
                       explicitVoiceFailedRequestAttachmentState
                   ),
                   let explicitVoiceFailedRequestAttachmentState {
                    desktopConversationExplicitVoiceFailedRequestAttachment(
                        explicitVoiceFailedRequestAttachmentState
                    )
                }

                if entry.posture == "wake_voice_failed_request_preview",
                   entry.sourceSurface == "WAKE_TRIGGERED_VOICE_FAILED_REQUEST",
                   desktopConversationShouldAttachWakeTriggeredVoiceFailedRequest(
                       wakeTriggeredVoiceFailedRequestAttachmentState
                   ),
                   let wakeTriggeredVoiceFailedRequestAttachmentState {
                    desktopConversationWakeTriggeredVoiceFailedRequestAttachment(
                        wakeTriggeredVoiceFailedRequestAttachmentState
                    )
                }

                if entry.posture == "explicit_voice_pending_preview",
                   entry.sourceSurface == "EXPLICIT_VOICE_PENDING",
                   desktopConversationShouldAttachExplicitVoicePendingAttachment(
                       explicitVoicePendingAttachmentState
                   ),
                   let explicitVoicePendingAttachmentState {
                    desktopConversationExplicitVoicePendingAttachment(
                        explicitVoicePendingAttachmentState
                    )
                }

                if entry.posture == "wake_voice_pending_preview",
                   entry.sourceSurface == "WAKE_TRIGGERED_VOICE_PENDING",
                   desktopConversationShouldAttachWakeTriggeredVoicePendingAttachment(
                       wakeTriggeredVoicePendingAttachmentState
                   ),
                   let wakeTriggeredVoicePendingAttachmentState {
                    desktopConversationWakeTriggeredVoicePendingAttachment(
                        wakeTriggeredVoicePendingAttachmentState
                    )
                }

                if desktopConversationShouldAttachRuntimeDispatchFailureInline(
                    to: entry,
                    runtimeDispatchFailureAttachmentState: runtimeDispatchFailureAttachmentState
                ),
                   let runtimeDispatchFailureAttachmentState {
                    desktopConversationRuntimeDispatchFailureAttachment(
                        runtimeDispatchFailureAttachmentState
                    )
                }

                if entry.posture == "runtime_completed_without_inline_reply_preview",
                   entry.sourceSurface == "CANONICAL_RUNTIME_COMPLETED_WITHOUT_INLINE_REPLY",
                   desktopConversationShouldAttachRuntimeCompletedWithoutInlineReply(
                       desktopCanonicalRuntimeOutcomeState,
                       searchToolCompletionState: searchToolCompletionState,
                       authoritativeReplyCompletionState: authoritativeReplyCompletionState
                   ),
                   let outcomeState = desktopCanonicalRuntimeOutcomeState {
                    desktopCanonicalRuntimeOutcomeCard(outcomeState)
                }

                if desktopConversationShouldAttachAuthoritativeReplyArtifacts(to: entry) {
                    desktopConversationAuthoritativeReplyAttachment(
                        readOnlyToolLaneState: readOnlyToolLaneState,
                        searchToolCompletionState: searchToolCompletionState,
                        authoritativeReplyCompletionState: authoritativeReplyCompletionState
                    )
                }
            }
            .padding(16)
            .frame(maxWidth: 720, alignment: .leading)
            .background(bubbleColor)
            .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))

            if !entry.isUserAuthored {
                Spacer(minLength: 72)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    private func desktopConversationVoiceStateBadge(_ voiceState: String) -> some View {
        Text(voiceState)
            .font(.caption.weight(.semibold))
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .background(Color.accentColor.opacity(0.14))
            .clipShape(Capsule())
    }

    private func desktopConversationShouldSuppressDedicatedAuthoritativeReplyTextEntry(
        _ timelineEntries: [DesktopConversationTimelineEntryState],
        authoritativeResponseText: String
    ) -> Bool {
        timelineEntries.contains { entry in
            let normalizedEntryBody = entry.body.trimmingCharacters(in: .whitespacesAndNewlines)
            guard normalizedEntryBody == authoritativeResponseText else {
                return false
            }

            return entry.posture == "current_selene_turn_text"
                || entry.posture == "archived_selene_turn_text"
        }
    }

    private func desktopConversationShouldSuppressSupportRailArchivedRecentSliceTranscript() -> Bool {
        guard desktopReadyTimeHandoffIsActive,
              foregroundSessionSuspendedVisibleContext == nil,
              activeRecoveryDisplayState != .quarantinedLocalState,
              foregroundSessionSoftClosedVisibleContext != nil,
              let timelineEntries = desktopOperationalConversationShellState?.primaryPaneState.timelineEntries else {
            return false
        }

        return timelineEntries.contains(where: { $0.posture == "archived_user_turn_text" })
            && timelineEntries.contains(where: { $0.posture == "archived_selene_turn_text" })
    }

    private func desktopConversationShouldSuppressSupportRailCurrentSessionTranscript() -> Bool {
        guard desktopReadyTimeHandoffIsActive,
              foregroundSessionSuspendedVisibleContext == nil,
              activeRecoveryDisplayState != .quarantinedLocalState,
              foregroundSessionActiveVisibleContext != nil,
              let timelineEntries = desktopOperationalConversationShellState?.primaryPaneState.timelineEntries else {
            return false
        }

        return timelineEntries.contains(where: { $0.posture == "current_user_turn_text" })
            && timelineEntries.contains(where: { $0.posture == "current_selene_turn_text" })
    }

    private func desktopConversationShouldAttachAuthoritativeReplyArtifacts(
        to entry: DesktopConversationTimelineEntryState
    ) -> Bool {
        guard let authoritativeResponseText = desktopAuthoritativeReplyRenderState?.authoritativeResponseText?
            .trimmingCharacters(in: .whitespacesAndNewlines),
            !authoritativeResponseText.isEmpty else {
            return false
        }

        let normalizedEntryBody = entry.body.trimmingCharacters(in: .whitespacesAndNewlines)
        return entry.posture == "authoritative_reply_text"
            || (entry.posture == "current_selene_turn_text"
                && normalizedEntryBody == authoritativeResponseText)
            || (entry.posture == "archived_selene_turn_text"
                && normalizedEntryBody == authoritativeResponseText)
    }

    private func desktopConversationShouldAttachReadOnlyToolLaneCluster(
        _ readOnlyToolLaneState: DesktopConversationReadOnlyToolLaneState?
    ) -> Bool {
        readOnlyToolLaneState != nil
    }

    private func desktopConversationShouldAttachExplicitVoiceLivePreview(
        _ explicitVoiceLivePreviewAttachmentState: DesktopConversationExplicitVoiceLivePreviewAttachmentState?
    ) -> Bool {
        explicitVoiceLivePreviewAttachmentState != nil
    }

    private func desktopConversationShouldAttachWakeTriggeredVoiceLivePreview(
        _ wakeTriggeredVoiceLivePreviewAttachmentState: DesktopConversationWakeTriggeredVoiceLivePreviewAttachmentState?
    ) -> Bool {
        wakeTriggeredVoiceLivePreviewAttachmentState != nil
    }

    private func desktopConversationShouldAttachExplicitVoiceFailedRequest(
        _ explicitVoiceFailedRequestAttachmentState: DesktopConversationExplicitVoiceFailedRequestAttachmentState?
    ) -> Bool {
        explicitVoiceFailedRequestAttachmentState != nil
    }

    private func desktopConversationShouldAttachWakeTriggeredVoiceFailedRequest(
        _ wakeTriggeredVoiceFailedRequestAttachmentState: DesktopConversationWakeTriggeredVoiceFailedRequestAttachmentState?
    ) -> Bool {
        wakeTriggeredVoiceFailedRequestAttachmentState != nil
    }

    private func desktopConversationShouldAttachExplicitVoicePendingAttachment(
        _ explicitVoicePendingAttachmentState: DesktopConversationExplicitVoicePendingAttachmentState?
    ) -> Bool {
        explicitVoicePendingAttachmentState != nil
    }

    private func desktopConversationShouldAttachWakeTriggeredVoicePendingAttachment(
        _ wakeTriggeredVoicePendingAttachmentState: DesktopConversationWakeTriggeredVoicePendingAttachmentState?
    ) -> Bool {
        wakeTriggeredVoicePendingAttachmentState != nil
    }

    private func desktopConversationShouldAttachSearchToolCompletion(
        _ searchToolCompletionState: DesktopConversationSearchToolCompletionState?
    ) -> Bool {
        searchToolCompletionState != nil
    }

    private func desktopConversationShouldAttachAuthoritativeReplyCompletion(
        _ authoritativeReplyCompletionState: DesktopConversationAuthoritativeReplyCompletionState?
    ) -> Bool {
        authoritativeReplyCompletionState != nil
    }

    private func desktopConversationShouldAttachRuntimeDispatchFailure(
        _ runtimeDispatchFailureAttachmentState: DesktopConversationRuntimeDispatchFailureAttachmentState?
    ) -> Bool {
        runtimeDispatchFailureAttachmentState != nil
    }

    private func desktopConversationShouldAttachRuntimeCompletedWithoutInlineReply(
        _ outcomeState: DesktopCanonicalRuntimeOutcomeState?,
        searchToolCompletionState: DesktopConversationSearchToolCompletionState?,
        authoritativeReplyCompletionState: DesktopConversationAuthoritativeReplyCompletionState?
    ) -> Bool {
        guard desktopReadyTimeHandoffIsActive,
              foregroundSessionSuspendedVisibleContext == nil,
              activeRecoveryDisplayState != .quarantinedLocalState,
              let outcomeState,
              outcomeState.phase == .completed,
              searchToolCompletionState == nil,
              authoritativeReplyCompletionState == nil else {
            return false
        }

        let authoritativeResponseText = desktopAuthoritativeReplyRenderState?.authoritativeResponseText?
            .trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        return authoritativeResponseText.isEmpty
    }

    private func desktopConversationShouldAttachRuntimeDispatchFailureInline(
        to entry: DesktopConversationTimelineEntryState,
        runtimeDispatchFailureAttachmentState: DesktopConversationRuntimeDispatchFailureAttachmentState?
    ) -> Bool {
        guard runtimeDispatchFailureAttachmentState != nil else {
            return false
        }

        return entry.posture == "runtime_dispatch_failure_preview"
            && entry.sourceSurface == "CANONICAL_RUNTIME_DISPATCH_FAILURE"
    }

    private func desktopConversationRuntimeDispatchFailureAttachment(
        _ state: DesktopConversationRuntimeDispatchFailureAttachmentState
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Runtime dispatch/failure attachment")
                    .font(.subheadline.weight(.semibold))

                metadataRow(label: "dispatch_phase", value: state.dispatchPhase)
                metadataRow(label: "request_id", value: state.requestID)
                metadataRow(label: "endpoint", value: state.endpoint)
                metadataRow(label: "outcome", value: state.outcome)
                metadataRow(label: "next_move", value: state.nextMove)
                metadataRow(label: "reason_code", value: state.reasonCode)
                metadataRow(label: "failure_class", value: state.failureClass)
                metadataRow(label: "session_id", value: state.sessionID)
                metadataRow(label: "turn_id", value: state.turnID)

                VStack(alignment: .leading, spacing: 4) {
                    Text("summary")
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)

                    Text(state.summary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                VStack(alignment: .leading, spacing: 4) {
                    Text("detail")
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)

                    Text(state.detail)
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                Text("This path composes already-live canonical runtime outcome carriers only. It does not add local session authority, local search input, local search execution, local tool invocation controls, local provider selection, interrupt-inline authority, hidden/background wake behavior, or autonomous unlock.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    private func desktopConversationExplicitVoiceLivePreviewAttachment(
        _ state: DesktopConversationExplicitVoiceLivePreviewAttachmentState
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Explicit voice live-preview attachment")
                    .font(.subheadline.weight(.semibold))

                metadataRow(label: "source_surface", value: state.sourceSurface)
                metadataRow(label: "capture_state", value: state.captureState)
                metadataRow(label: "capture_mode", value: state.captureMode)
                metadataRow(label: "transcript_posture", value: state.transcriptPosture)
                metadataRow(label: "transcript_bytes", value: state.transcriptBytes)

                Text("This path composes already-live exact explicit voice live transcript preview carriers only.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("This path does not add canonical runtime acceptance, local session authority, local search input, local search execution, local tool invocation controls, local provider selection, wake-listener authority, hidden/background wake behavior, or autonomous unlock.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    private func desktopConversationWakeTriggeredVoiceLivePreviewAttachment(
        _ state: DesktopConversationWakeTriggeredVoiceLivePreviewAttachmentState
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Wake live-preview attachment")
                    .font(.subheadline.weight(.semibold))

                metadataRow(label: "source_surface", value: state.sourceSurface)
                metadataRow(label: "listener_state", value: state.listenerState)
                metadataRow(label: "wake_trigger_phrase", value: state.wakeTriggerPhrase)
                metadataRow(label: "transcript_posture", value: state.transcriptPosture)
                metadataRow(label: "transcript_bytes", value: state.transcriptBytes)

                Text("This path composes already-live exact foreground wake-listener transcript preview carriers only.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("This path does not add canonical runtime acceptance, wake-listener control authority inline, local session authority, local search input, local search execution, local tool invocation controls, local provider selection, hidden/background wake behavior, or autonomous unlock.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    private func desktopConversationExplicitVoiceFailedRequestAttachment(
        _ state: DesktopConversationExplicitVoiceFailedRequestAttachmentState
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Explicit voice failed-request attachment")
                    .font(.subheadline.weight(.semibold))

                metadataRow(label: "failure_id", value: state.failureID)
                metadataRow(label: "source_surface", value: state.sourceSurface)
                metadataRow(label: "failure_title", value: state.failureTitle)
                metadataRow(label: "failure_summary", value: state.failureSummary)
                metadataRow(label: "failure_detail", value: state.failureDetail)

                Text("This path composes already-live exact explicit local failed-request carriers only.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("This path does not add canonical runtime acceptance, local session authority, local search input, local search execution, local tool invocation controls, local provider selection, wake-listener authority, hidden/background wake behavior, or autonomous unlock.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    private func desktopConversationWakeTriggeredVoiceFailedRequestAttachment(
        _ state: DesktopConversationWakeTriggeredVoiceFailedRequestAttachmentState
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Wake failed-request attachment")
                    .font(.subheadline.weight(.semibold))

                metadataRow(label: "failure_id", value: state.failureID)
                metadataRow(label: "source_surface", value: state.sourceSurface)
                metadataRow(label: "listener_state", value: state.listenerState)
                metadataRow(label: "wake_trigger_phrase", value: state.wakeTriggerPhrase)
                metadataRow(label: "failure_title", value: state.failureTitle)
                metadataRow(label: "failure_summary", value: state.failureSummary)
                metadataRow(label: "failure_detail", value: state.failureDetail)

                Text("This path composes already-live exact wake local failed-request carriers only.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("This path does not add canonical runtime acceptance, wake-listener control authority inline, local session authority, local search input, local search execution, local tool invocation controls, local provider selection, hidden/background wake behavior, or autonomous unlock.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    private func desktopConversationExplicitVoicePendingAttachment(
        _ state: DesktopConversationExplicitVoicePendingAttachmentState
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Explicit voice pending attachment")
                    .font(.subheadline.weight(.semibold))

                metadataRow(label: "request_id", value: state.requestID)
                metadataRow(label: "source_surface", value: state.sourceSurface)
                metadataRow(label: "capture_mode", value: state.captureMode)
                metadataRow(label: "transcript_posture", value: state.transcriptPosture)
                metadataRow(label: "transcript_bytes", value: state.transcriptBytes)
                metadataRow(label: "selected_mic", value: state.selectedMic)
                metadataRow(label: "selected_speaker", value: state.selectedSpeaker)
                metadataRow(label: "device_route", value: state.deviceRoute)
                metadataRow(label: "locale_tag", value: state.localeTag)
                metadataRow(label: "tts_playback_active", value: state.ttsPlaybackActive)
                metadataRow(label: "capture_degraded", value: state.captureDegraded)
                metadataRow(label: "stream_gap_detected", value: state.streamGapDetected)
                metadataRow(label: "device_changed", value: state.deviceChanged)

                Text("This path composes already-live exact prepared explicit voice request carriers and exact H288 structured `audioCaptureRef` rows only.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("This path does not add canonical runtime acceptance, local session authority, local search input, local search execution, local tool invocation controls, local provider selection, hidden/background wake behavior, or autonomous unlock.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    private func desktopConversationWakeTriggeredVoicePendingAttachment(
        _ state: DesktopConversationWakeTriggeredVoicePendingAttachmentState
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Wake-triggered pending attachment")
                    .font(.subheadline.weight(.semibold))

                metadataRow(label: "request_id", value: state.requestID)
                metadataRow(label: "source_surface", value: state.sourceSurface)
                metadataRow(label: "wake_trigger_phrase", value: state.wakeTriggerPhrase)
                metadataRow(label: "transcript_posture", value: state.transcriptPosture)
                metadataRow(label: "transcript_bytes", value: state.transcriptBytes)
                metadataRow(label: "selected_mic", value: state.selectedMic)
                metadataRow(label: "selected_speaker", value: state.selectedSpeaker)
                metadataRow(label: "device_route", value: state.deviceRoute)
                metadataRow(label: "locale_tag", value: state.localeTag)
                metadataRow(label: "tts_playback_active", value: state.ttsPlaybackActive)
                metadataRow(label: "capture_degraded", value: state.captureDegraded)
                metadataRow(label: "stream_gap_detected", value: state.streamGapDetected)
                metadataRow(label: "device_changed", value: state.deviceChanged)

                Text("This path composes already-live exact prepared wake-triggered voice request carriers and exact H288 structured `audioCaptureRef` rows only.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("This path does not add canonical runtime acceptance, wake-listener control authority inline, local session authority, local search input, local search execution, local tool invocation controls, local provider selection, hidden/background wake behavior, or autonomous unlock.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    private func desktopConversationAuthoritativeReplyAttachment(
        readOnlyToolLaneState: DesktopConversationReadOnlyToolLaneState?,
        searchToolCompletionState: DesktopConversationSearchToolCompletionState?,
        authoritativeReplyCompletionState: DesktopConversationAuthoritativeReplyCompletionState?
    ) -> some View {
        VStack(alignment: .leading, spacing: 12) {
                Divider()
            if desktopConversationShouldAttachSearchToolCompletion(searchToolCompletionState),
               let searchToolCompletionState {
                desktopConversationSearchToolCompletionAttachment(searchToolCompletionState)
            } else if desktopConversationShouldAttachAuthoritativeReplyCompletion(authoritativeReplyCompletionState),
                      let authoritativeReplyCompletionState {
                desktopConversationAuthoritativeReplyCompletionAttachment(authoritativeReplyCompletionState)
            } else {
                if let desktopAuthoritativeReplyProvenanceRenderState {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Attached provenance")
                            .font(.subheadline.weight(.semibold))

                        Text(desktopAuthoritativeReplyProvenanceRenderState.summary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        if let retrievedAtLabel = desktopAuthoritativeReplyProvenanceRenderState.retrievedAtLabel {
                            metadataRow(label: "retrieved_at", value: retrievedAtLabel)
                        }

                        if let cacheStatusLabel = desktopAuthoritativeReplyProvenanceRenderState.cacheStatusLabel {
                            metadataRow(label: "cache_status", value: cacheStatusLabel)
                        }

                        if desktopAuthoritativeReplyProvenanceRenderState.sources.isEmpty {
                            Text("No cloud-authored source rows were returned for this authoritative reply posture.")
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        } else {
                            ForEach(desktopAuthoritativeReplyProvenanceRenderState.sources) { source in
                                VStack(alignment: .leading, spacing: 4) {
                                    Text(source.title)
                                        .font(.subheadline.weight(.medium))

                                    if let sourceURL = URL(string: source.url) {
                                        Link(source.url, destination: sourceURL)
                                            .font(.footnote)
                                    } else {
                                        Text(source.url)
                                            .font(.footnote)
                                            .foregroundStyle(.secondary)
                                            .textSelection(.enabled)
                                    }
                                }
                                .frame(maxWidth: .infinity, alignment: .leading)
                            }
                        }

                        Text("Read-only cloud-authored provenance only. This shell does not claim that search executed locally and does not widen into new search controls, wake posture, or autonomous-unlock behavior.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }

                if desktopConversationShouldAttachReadOnlyToolLaneCluster(readOnlyToolLaneState),
                   let readOnlyToolLaneState {
                    Divider()
                    desktopConversationReadOnlyToolLaneCluster(readOnlyToolLaneState)
                }

                Divider()

                VStack(alignment: .leading, spacing: 10) {
                    Text(desktopAuthoritativeReplyPlaybackState.title)
                        .font(.subheadline.weight(.semibold))

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
                            desktopAuthoritativeReplyRenderState?.authoritativeResponseText?
                                .trimmingCharacters(in: .whitespacesAndNewlines).isEmpty != false
                                || desktopAuthoritativeReplyPlaybackState.phase == .speaking
                        )

                        Button("Stop reply playback") {
                            stopAuthoritativeReplyPlayback()
                        }
                        .buttonStyle(.bordered)
                        .disabled(desktopAuthoritativeReplyPlaybackState.phase != .speaking)
                    }

                    Text("User-triggered bounded reply playback only. No transcript mutation, no conversation-history mutation, no wake parity claim, and no autonomous-unlock claim are introduced by this surface.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            }
        }
    }

    private func desktopConversationAuthoritativeReplyCompletionAttachment(
        _ state: DesktopConversationAuthoritativeReplyCompletionState
    ) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            VStack(alignment: .leading, spacing: 8) {
                Text("Authoritative reply completion attachment")
                    .font(.subheadline.weight(.semibold))

                metadataRow(label: "dispatch_phase", value: state.dispatchPhase)
                metadataRow(label: "request_id", value: state.requestID)
                metadataRow(label: "endpoint", value: state.endpoint)
                metadataRow(label: "outcome", value: state.outcome)
                metadataRow(label: "next_move", value: state.nextMove)
                metadataRow(label: "reason_code", value: state.reasonCode)
                metadataRow(label: "session_id", value: state.sessionID)
                metadataRow(label: "turn_id", value: state.turnID)

                Text(state.authoritativeResponseText)
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("This path composes already-live cloud-authored canonical runtime outcome, authoritative reply, provenance, and playback carriers only. It does not add local search input, local search execution, local tool invocation controls, local provider selection, local session authority, hidden/background wake behavior, or autonomous unlock.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }

            Divider()

            VStack(alignment: .leading, spacing: 8) {
                Text("Cloud-authored provenance")
                    .font(.subheadline.weight(.semibold))

                if let retrievedAtLabel = state.retrievedAtLabel {
                    metadataRow(label: "retrieved_at", value: retrievedAtLabel)
                }

                if let cacheStatusLabel = state.cacheStatusLabel {
                    metadataRow(label: "cache_status", value: cacheStatusLabel)
                }

                if state.sources.isEmpty {
                    Text("No cloud-authored source rows were returned for this completed authoritative-reply posture.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                } else {
                    ForEach(state.sources) { source in
                        VStack(alignment: .leading, spacing: 4) {
                            Text(source.title)
                                .font(.subheadline.weight(.medium))

                            if let sourceURL = URL(string: source.url) {
                                Link(source.url, destination: sourceURL)
                                    .font(.footnote)
                            } else {
                                Text(source.url)
                                    .font(.footnote)
                                    .foregroundStyle(.secondary)
                                    .textSelection(.enabled)
                            }
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
            }

            Divider()

            VStack(alignment: .leading, spacing: 10) {
                Text(state.playbackTitle)
                    .font(.subheadline.weight(.semibold))

                Text(state.playbackSummary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text(state.playbackDetail)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(spacing: 12) {
                    Button("Play authoritative reply") {
                        playAuthoritativeReply()
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(
                        state.authoritativeResponseText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                            || state.playbackPhase == DesktopAuthoritativeReplyPlaybackState.Phase.speaking.rawValue
                    )

                    Button("Stop reply playback") {
                        stopAuthoritativeReplyPlayback()
                    }
                    .buttonStyle(.bordered)
                    .disabled(
                        state.playbackPhase != DesktopAuthoritativeReplyPlaybackState.Phase.speaking.rawValue
                    )
                }

                Text("User-triggered bounded reply playback only. No transcript mutation, no conversation-history mutation, no wake parity claim, and no autonomous-unlock claim are introduced by this surface.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    private func desktopConversationSearchToolCompletionAttachment(
        _ state: DesktopConversationSearchToolCompletionState
    ) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            VStack(alignment: .leading, spacing: 8) {
                Text("Search/tool completion attachment")
                    .font(.subheadline.weight(.semibold))

                metadataRow(label: "dispatch_phase", value: state.dispatchPhase)
                metadataRow(label: "request_id", value: state.requestID)
                metadataRow(label: "endpoint", value: state.endpoint)
                metadataRow(label: "outcome", value: state.outcome)
                metadataRow(label: "next_move", value: state.nextMove)
                metadataRow(label: "reason_code", value: state.reasonCode)
                metadataRow(label: "session_id", value: state.sessionID)
                metadataRow(label: "turn_id", value: state.turnID)

                Text(state.authoritativeResponseText)
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("This path composes already-live cloud-authored canonical runtime outcome, authoritative reply, provenance, and read-only tool-lane carriers only. It does not itself execute search locally, invoke tools locally, pick providers locally, add local session authority, introduce hidden/background wake behavior, or claim autonomous unlock.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }

            Divider()

            VStack(alignment: .leading, spacing: 8) {
                Text("Cloud-authored provenance")
                    .font(.subheadline.weight(.semibold))

                if let retrievedAtLabel = state.retrievedAtLabel {
                    metadataRow(label: "retrieved_at", value: retrievedAtLabel)
                }

                if let cacheStatusLabel = state.cacheStatusLabel {
                    metadataRow(label: "cache_status", value: cacheStatusLabel)
                }

                if state.sources.isEmpty {
                    Text("No cloud-authored source rows were returned for this completed search/tool posture.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                } else {
                    ForEach(state.sources) { source in
                        VStack(alignment: .leading, spacing: 4) {
                            Text(source.title)
                                .font(.subheadline.weight(.medium))

                            if let sourceURL = URL(string: source.url) {
                                Link(source.url, destination: sourceURL)
                                    .font(.footnote)
                            } else {
                                Text(source.url)
                                    .font(.footnote)
                                    .foregroundStyle(.secondary)
                                    .textSelection(.enabled)
                            }
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
            }

            Divider()
            desktopConversationReadOnlyToolLaneCluster(state.readOnlyToolLaneState)

            Divider()

            VStack(alignment: .leading, spacing: 10) {
                Text(state.playbackTitle)
                    .font(.subheadline.weight(.semibold))

                Text(state.playbackSummary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text(state.playbackDetail)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(spacing: 12) {
                    Button("Play authoritative reply") {
                        playAuthoritativeReply()
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(
                        state.authoritativeResponseText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                            || state.playbackPhase == DesktopAuthoritativeReplyPlaybackState.Phase.speaking.rawValue
                    )

                    Button("Stop reply playback") {
                        stopAuthoritativeReplyPlayback()
                    }
                    .buttonStyle(.bordered)
                    .disabled(
                        state.playbackPhase != DesktopAuthoritativeReplyPlaybackState.Phase.speaking.rawValue
                    )
                }

                Text("User-triggered bounded reply playback only. No transcript mutation, no conversation-history mutation, no wake parity claim, and no autonomous-unlock claim are introduced by this surface.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    private func desktopConversationReadOnlyToolLaneCluster(
        _ state: DesktopConversationReadOnlyToolLaneState
    ) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Read-only tool lane attachment")
                .font(.subheadline.weight(.semibold))

            metadataRow(label: "lane_kind", value: state.laneKind)
            metadataRow(label: "response_surface", value: state.responseSurface)
            metadataRow(label: "outcome", value: state.outcome)
            metadataRow(label: "next_move", value: state.nextMove)
            metadataRow(label: "reason_code", value: state.reasonCode)
            metadataRow(label: "source_count", value: String(state.sourceCount))

            if let retrievedAtLabel = state.retrievedAtLabel {
                metadataRow(label: "retrieved_at", value: retrievedAtLabel)
            }

            if let cacheStatusLabel = state.cacheStatusLabel {
                metadataRow(label: "cache_status", value: cacheStatusLabel)
            }

            if state.sources.isEmpty {
                Text("Canonical runtime returned a read-only tool-lane posture without cloud-authored source rows, retrieval timing, or cache posture. This shell fails closed to explanation-only metadata here and does not fabricate local sources, provider identity, query identity, or tool-name authority.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            } else {
                ForEach(state.sources) { source in
                    VStack(alignment: .leading, spacing: 4) {
                        Text(source.title)
                            .font(.subheadline.weight(.medium))

                        if let sourceURL = URL(string: source.url) {
                            Link(source.url, destination: sourceURL)
                                .font(.footnote)
                        } else {
                            Text(source.url)
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                                .textSelection(.enabled)
                        }
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
            }

            Text("Search and tool execution remain cloud-authored and read-only here. This path does not add local search execution, local tool invocation controls, local provider selection, local session attach or reopen authority, hidden/background wake behavior, or autonomous unlock.")
                .font(.footnote)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
    }

    private var desktopSessionSoftClosedVisibilityCard: some View {
        let sessionSoftClosedVisibilityState = desktopSessionSoftClosedVisibilityState
        let shouldSuppressSupportRailArchivedRecentSliceTranscript =
            desktopConversationShouldSuppressSupportRailArchivedRecentSliceTranscript()

        return Group {
            if let sessionSoftClosedVisibilityState {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded desktop soft-closed visibility only. This shell derives one dedicated read-only exact `SESSION_SOFT_CLOSED_VISIBLE` surface from the already-live soft-closed route context only, preserves the exact disabled non-producing explicit resume affordance, preserves the exact archived recent slice, and preserves bounded PH1.M `resume context` without introducing any local resume, attach, reopen, or thread-selection authority.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        ForEach(
                            [
                                ("source_surface", sessionSoftClosedVisibilityState.sourceSurfaceIdentity),
                                ("session_state", sessionSoftClosedVisibilityState.sessionState),
                                ("session_id", sessionSoftClosedVisibilityState.sessionID),
                            ],
                            id: \.0
                        ) { row in
                            metadataRow(label: row.0, value: row.1)
                        }

                        Button("Resume the selected thread explicitly") {}
                            .buttonStyle(.borderedProminent)
                            .disabled(true)

                        if !shouldSuppressSupportRailArchivedRecentSliceTranscript {
                            transcriptEntry(
                                speaker: "You",
                                posture: "archived_user_turn_text",
                                body: sessionSoftClosedVisibilityState.archivedUserTurnText,
                                detail: "Archived recent slice remains durable archived conversation truth and stays distinct from bounded PH1.M resume-context output."
                            )

                            transcriptEntry(
                                speaker: "Selene",
                                posture: "archived_selene_turn_text",
                                body: sessionSoftClosedVisibilityState.archivedSeleneTurnText,
                                detail: "Archived recent slice remains text-visible after visual reset without local auto-reopen, hidden spoken-only output, or local transcript authority."
                            )
                        }

                        ForEach(
                            [
                                ("selected_thread_id", sessionSoftClosedVisibilityState.selectedThreadID ?? "not_provided"),
                                ("selected_thread_title", sessionSoftClosedVisibilityState.selectedThreadTitle ?? "not_provided"),
                                ("pending_work_order_id", sessionSoftClosedVisibilityState.pendingWorkOrderID ?? "not_provided"),
                                ("resume_tier", sessionSoftClosedVisibilityState.resumeTier ?? "not_provided"),
                            ],
                            id: \.0
                        ) { row in
                            metadataRow(label: row.0, value: row.1)
                        }

                        if sessionSoftClosedVisibilityState.resumeSummaryBullets.isEmpty {
                            Text("No bounded `resume_summary_bullets` were provided for this soft-closed preview.")
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                        } else {
                            ForEach(
                                Array(sessionSoftClosedVisibilityState.resumeSummaryBullets.prefix(3).enumerated()),
                                id: \.offset
                            ) { index, bullet in
                                HStack(alignment: .firstTextBaseline, spacing: 10) {
                                    Text("\(index + 1).")
                                        .font(.caption.weight(.semibold))
                                        .foregroundStyle(.secondary)

                                    Text(bullet)
                                        .frame(maxWidth: .infinity, alignment: .leading)
                                }
                            }
                        }

                        Text("Visual reset may clear the screen while archive truth remains durable. Archived recent slice remains distinct from bounded PH1.M `resume context`, and the explicit resume affordance remains non-producing here.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text("No local session resume controls, no local session attach controls, no local reopen controls, no local thread-selection controls, no local PH1.M synthesis, and no local onboarding authority claims are introduced by this bounded surface. This shell stays explicitly non-authoritative.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Session Soft-Closed Visibility")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopSessionMultiPostureEntryCard: some View {
        let promptState = desktopSessionMultiPostureEntryPromptState
        let activeRuntimeOutcomeState: DesktopSessionMultiPostureEntryRuntimeOutcomeState? = {
            guard let desktopSessionMultiPostureEntryRuntimeOutcomeState else {
                return nil
            }

            guard let promptState else {
                return desktopSessionMultiPostureEntryRuntimeOutcomeState
            }

            return desktopSessionMultiPostureEntryRuntimeOutcomeState.entryMode == promptState.entryMode
                && desktopSessionMultiPostureEntryRuntimeOutcomeState.sourceSurfaceIdentity == promptState.sourceSurfaceIdentity
                && desktopSessionMultiPostureEntryRuntimeOutcomeState.sessionID == promptState.sessionID
                && desktopSessionMultiPostureEntryRuntimeOutcomeState.deviceID == promptState.deviceID
                && desktopSessionMultiPostureEntryRuntimeOutcomeState.turnID == promptState.turnID
                && desktopSessionMultiPostureEntryRuntimeOutcomeState.selectedThreadID == promptState.selectedThreadID
                && desktopSessionMultiPostureEntryRuntimeOutcomeState.recoveryMode == promptState.recoveryMode
                ? desktopSessionMultiPostureEntryRuntimeOutcomeState
                : nil
        }()
        let displayedEntryMode = activeRuntimeOutcomeState?.entryMode ?? promptState?.entryMode
        let displayedSourceSurface = activeRuntimeOutcomeState?.sourceSurfaceIdentity
            ?? promptState?.sourceSurfaceIdentity
            ?? "not_provided"
        let displayedSessionState = activeRuntimeOutcomeState?.sessionState
            ?? promptState?.sessionState
            ?? "not_provided"
        let displayedSessionID = activeRuntimeOutcomeState?.sessionID
            ?? promptState?.sessionID
            ?? "not_provided"
        let displayedDeviceID = activeRuntimeOutcomeState?.deviceID
            ?? promptState?.deviceID
            ?? desktopManagedPrimaryDeviceID
            ?? "not_provided"
        let displayedCurrentVisibleSessionAttachOutcome = activeRuntimeOutcomeState?.currentVisibleSessionAttachOutcome
            ?? promptState?.currentVisibleSessionAttachOutcome
        let displayedTurnID = activeRuntimeOutcomeState?.turnID ?? promptState?.turnID
        let displayedSelectedThreadID = activeRuntimeOutcomeState?.selectedThreadID
            ?? promptState?.selectedThreadID
        let displayedSelectedThreadTitle = activeRuntimeOutcomeState?.selectedThreadTitle
            ?? promptState?.selectedThreadTitle
        let displayedPendingWorkOrderID = activeRuntimeOutcomeState?.pendingWorkOrderID
            ?? promptState?.pendingWorkOrderID
        let displayedResumeTier = activeRuntimeOutcomeState?.resumeTier
            ?? promptState?.resumeTier
        let displayedResumeSummaryBullets = activeRuntimeOutcomeState?.resumeSummaryBullets
            ?? promptState?.resumeSummaryBullets
            ?? []
        let displayedRecoveryMode = activeRuntimeOutcomeState?.recoveryMode
            ?? promptState?.recoveryMode
        let entryButtonTitle: String? = {
            guard let promptState else {
                return nil
            }

            switch promptState.entryMode {
            case .currentVisibleAttach:
                return "Attach to the visible session"
            case .softClosedExplicitResume:
                return "Resume the selected thread explicitly"
            case .suspendedAuthoritativeRereadRecover:
                return "Recover the suspended session authoritatively"
            }
        }()

        return Group {
            if promptState != nil || activeRuntimeOutcomeState != nil {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded desktop multi-posture session-entry submission only. This shell derives one bounded prompt state from the already-live exact current-visible attach prompt or exact resumable session-entry prompt, dispatches only exact `/v1/session/attach`, exact `/v1/session/resume`, or exact `/v1/session/recover`, and keeps returned exact `session_state` plus exact `session_attach_outcome` read-only only outside the exact multi-posture control itself.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        if let displayedEntryMode {
                            ForEach(
                                [
                                    ("source_surface", displayedSourceSurface),
                                    ("entry_mode", displayedEntryMode.rawValue),
                                    ("session_state", displayedSessionState),
                                    ("session_id", displayedSessionID),
                                    ("device_id", displayedDeviceID),
                                ],
                                id: \.0
                            ) { row in
                                metadataRow(label: row.0, value: row.1)
                            }

                            switch displayedEntryMode {
                            case .currentVisibleAttach:
                                if let displayedCurrentVisibleSessionAttachOutcome {
                                    metadataRow(
                                        label: "current_visible_session_attach_outcome",
                                        value: displayedCurrentVisibleSessionAttachOutcome
                                    )
                                }

                                if displayedSourceSurface == "SESSION_ACTIVE_VISIBLE",
                                   let displayedTurnID {
                                    metadataRow(label: "turn_id", value: displayedTurnID)
                                }

                            case .softClosedExplicitResume:
                                metadataRow(
                                    label: "selected_thread_id",
                                    value: displayedSelectedThreadID ?? "not_provided"
                                )
                                metadataRow(
                                    label: "selected_thread_title",
                                    value: displayedSelectedThreadTitle ?? "not_provided"
                                )
                                metadataRow(
                                    label: "pending_work_order_id",
                                    value: displayedPendingWorkOrderID ?? "not_provided"
                                )
                                metadataRow(
                                    label: "resume_tier",
                                    value: displayedResumeTier ?? "not_provided"
                                )

                                if displayedResumeSummaryBullets.isEmpty {
                                    Text("No bounded `resume_summary_bullets` were provided for this multi-posture soft-closed explicit resume surface.")
                                        .font(.footnote)
                                        .foregroundStyle(.secondary)
                                } else {
                                    ForEach(Array(displayedResumeSummaryBullets.prefix(3).enumerated()), id: \.offset) { index, bullet in
                                        HStack(alignment: .firstTextBaseline, spacing: 10) {
                                            Text("\(index + 1).")
                                                .font(.caption.weight(.semibold))
                                                .foregroundStyle(.secondary)

                                            Text(bullet)
                                                .frame(maxWidth: .infinity, alignment: .leading)
                                        }
                                    }
                                }

                            case .suspendedAuthoritativeRereadRecover:
                                metadataRow(
                                    label: "recovery_mode",
                                    value: displayedRecoveryMode ?? "not_provided"
                                )
                            }
                        }

                        Text("This exact surface performs one bounded session-entry submission only through already-live exact route-specific seams. No local generic reopen authority, no broader session-list fetch authority, no local search controls, no local tool invocation controls, no hidden/background wake behavior, and no autonomous unlock are introduced by this shell.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        if let promptState, let entryButtonTitle {
                            Button(entryButtonTitle) {
                                Task {
                                    await submitDesktopSessionMultiPostureEntry(promptState: promptState)
                                }
                            }
                            .buttonStyle(.borderedProminent)
                            .disabled(activeRuntimeOutcomeState?.phase == .dispatching)
                        }

                        if let activeRuntimeOutcomeState {
                            Divider()

                            Text(activeRuntimeOutcomeState.title)
                                .font(.headline)

                            ForEach(
                                [
                                    ("dispatch_phase", activeRuntimeOutcomeState.phase.rawValue),
                                    ("request_id", activeRuntimeOutcomeState.requestID),
                                    ("endpoint", activeRuntimeOutcomeState.endpoint),
                                    ("outcome", activeRuntimeOutcomeState.outcome ?? "not_available"),
                                    ("reason", activeRuntimeOutcomeState.reason ?? "not_available"),
                                    ("session_id", activeRuntimeOutcomeState.sessionID),
                                    ("session_state", activeRuntimeOutcomeState.sessionState),
                                    ("session_attach_outcome", activeRuntimeOutcomeState.sessionAttachOutcome ?? "not_available"),
                                ],
                                id: \.0
                            ) { row in
                                metadataRow(label: row.0, value: row.1)
                            }

                            Text(activeRuntimeOutcomeState.summary)
                                .frame(maxWidth: .infinity, alignment: .leading)

                            Text(activeRuntimeOutcomeState.detail)
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        } else if promptState != nil {
                            Text("Awaiting explicit user-triggered canonical session entry through the bounded multi-posture control. After submission, any returned exact `session_state` and exact `session_attach_outcome` remain bounded read-only only in this shell.")
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }

                        Text("Cloud-authored, session-bound, and non-authoritative only. This path does not add local generic reopen authority, conversation-list selection, keyboard text entry, local search controls, local tool controls, hidden/background wake behavior, wake parity claims, or autonomous unlock.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Multi-Posture Session Entry")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopSessionAttachCard: some View {
        let promptState = desktopSessionAttachPromptState
        let activeRuntimeOutcomeState: DesktopSessionAttachRuntimeOutcomeState? = {
            guard let desktopSessionAttachRuntimeOutcomeState else {
                return nil
            }

            guard let promptState else {
                return desktopSessionAttachRuntimeOutcomeState
            }

            return desktopSessionAttachRuntimeOutcomeState.sourceSurfaceIdentity == promptState.sourceSurfaceIdentity
                && desktopSessionAttachRuntimeOutcomeState.sessionID == promptState.sessionID
                && desktopSessionAttachRuntimeOutcomeState.deviceID == promptState.deviceID
                && desktopSessionAttachRuntimeOutcomeState.turnID == promptState.turnID
                ? desktopSessionAttachRuntimeOutcomeState
                : nil
        }()
        let displayedSourceSurface = activeRuntimeOutcomeState?.sourceSurfaceIdentity
            ?? promptState?.sourceSurfaceIdentity
            ?? "not_provided"
        let displayedSessionState = activeRuntimeOutcomeState?.sessionState
            ?? promptState?.sessionState
            ?? "not_provided"
        let displayedSessionID = activeRuntimeOutcomeState?.sessionID
            ?? promptState?.sessionID
            ?? "not_provided"
        let displayedDeviceID = activeRuntimeOutcomeState?.deviceID
            ?? promptState?.deviceID
            ?? desktopManagedPrimaryDeviceID
            ?? "not_provided"
        let displayedCurrentVisibleSessionAttachOutcome = activeRuntimeOutcomeState?.currentVisibleSessionAttachOutcome
            ?? promptState?.currentVisibleSessionAttachOutcome
        let displayedTurnID = activeRuntimeOutcomeState?.turnID ?? promptState?.turnID

        return Group {
            if promptState != nil || activeRuntimeOutcomeState != nil {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded desktop current-visible session attach submission only. This shell derives one bounded prompt state from the already-live exact current visible session surface only, dispatches only exact `/v1/session/attach`, and keeps returned exact `session_state` plus exact `session_attach_outcome` read-only only outside the exact attach control itself.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        ForEach(
                            [
                                ("source_surface", displayedSourceSurface),
                                ("session_state", displayedSessionState),
                                ("session_id", displayedSessionID),
                                ("device_id", displayedDeviceID),
                            ],
                            id: \.0
                        ) { row in
                            metadataRow(label: row.0, value: row.1)
                        }

                        if let displayedCurrentVisibleSessionAttachOutcome {
                            metadataRow(
                                label: "current_visible_session_attach_outcome",
                                value: displayedCurrentVisibleSessionAttachOutcome
                            )
                        }

                        if displayedSourceSurface == "SESSION_ACTIVE_VISIBLE",
                           let displayedTurnID {
                            metadataRow(label: "turn_id", value: displayedTurnID)
                        }

                        Text("This exact surface performs one bounded current-visible session attach submission only. No local reopen authority, no broader session-list fetch authority, no local search controls, no local tool invocation controls, no hidden/background wake behavior, and no autonomous unlock are introduced by this shell.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        if let promptState {
                            Button("Attach to the visible session") {
                                Task {
                                    await submitDesktopSessionAttach(promptState: promptState)
                                }
                            }
                            .buttonStyle(.borderedProminent)
                            .disabled(activeRuntimeOutcomeState?.phase == .dispatching)
                        }

                        if let activeRuntimeOutcomeState {
                            Divider()

                            Text(activeRuntimeOutcomeState.title)
                                .font(.headline)

                            ForEach(
                                [
                                    ("dispatch_phase", activeRuntimeOutcomeState.phase.rawValue),
                                    ("request_id", activeRuntimeOutcomeState.requestID),
                                    ("endpoint", activeRuntimeOutcomeState.endpoint),
                                    ("outcome", activeRuntimeOutcomeState.outcome ?? "not_available"),
                                    ("reason", activeRuntimeOutcomeState.reason ?? "not_available"),
                                    ("session_id", activeRuntimeOutcomeState.sessionID),
                                    ("session_state", activeRuntimeOutcomeState.sessionState),
                                    ("session_attach_outcome", activeRuntimeOutcomeState.sessionAttachOutcome ?? "not_available"),
                                ],
                                id: \.0
                            ) { row in
                                metadataRow(label: row.0, value: row.1)
                            }

                            Text(activeRuntimeOutcomeState.summary)
                                .frame(maxWidth: .infinity, alignment: .leading)

                            Text(activeRuntimeOutcomeState.detail)
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        } else if promptState != nil {
                            Text("Awaiting explicit user-triggered canonical current-visible session attach through the bounded attach control. After submission, any returned exact `session_state` and exact `session_attach_outcome` remain bounded read-only only in this shell.")
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }

                        Text("Cloud-authored, session-bound, and non-authoritative only. This path does not add local reopen authority, conversation-list selection, keyboard text entry, local search controls, local tool controls, hidden/background wake behavior, wake parity claims, or autonomous unlock.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Current-Visible Session Attach")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopSessionMultiPostureResumeCard: some View {
        let promptState = desktopSessionMultiPostureResumePromptState
        let activeRuntimeOutcomeState: DesktopSessionMultiPostureResumeRuntimeOutcomeState? = {
            guard let desktopSessionMultiPostureResumeRuntimeOutcomeState else {
                return nil
            }

            guard let promptState else {
                return desktopSessionMultiPostureResumeRuntimeOutcomeState
            }

            return desktopSessionMultiPostureResumeRuntimeOutcomeState.resumeMode == promptState.resumeMode
                ? desktopSessionMultiPostureResumeRuntimeOutcomeState
                : nil
        }()
        let displayedResumeMode = activeRuntimeOutcomeState?.resumeMode ?? promptState?.resumeMode
        let displayedSourceSurface = activeRuntimeOutcomeState?.sourceSurfaceIdentity
            ?? promptState?.sourceSurfaceIdentity
            ?? "not_provided"
        let displayedSessionState = activeRuntimeOutcomeState?.sessionState
            ?? promptState?.sessionState
            ?? "not_provided"
        let displayedSessionID = activeRuntimeOutcomeState?.sessionID
            ?? promptState?.sessionID
            ?? "not_provided"
        let displayedDeviceID = activeRuntimeOutcomeState?.deviceID
            ?? promptState?.deviceID
            ?? desktopManagedPrimaryDeviceID
            ?? "not_provided"
        let displayedSelectedThreadID = activeRuntimeOutcomeState?.selectedThreadID
            ?? promptState?.selectedThreadID
        let displayedSelectedThreadTitle = activeRuntimeOutcomeState?.selectedThreadTitle
            ?? promptState?.selectedThreadTitle
        let displayedPendingWorkOrderID = activeRuntimeOutcomeState?.pendingWorkOrderID
            ?? promptState?.pendingWorkOrderID
        let displayedResumeTier = activeRuntimeOutcomeState?.resumeTier
            ?? promptState?.resumeTier
        let displayedResumeSummaryBullets = activeRuntimeOutcomeState?.resumeSummaryBullets
            ?? promptState?.resumeSummaryBullets
            ?? []
        let displayedRecoveryMode = activeRuntimeOutcomeState?.recoveryMode
            ?? promptState?.recoveryMode

        return Group {
            if promptState != nil || activeRuntimeOutcomeState != nil {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded desktop multi-posture session-resume submission only. This shell derives one bounded prompt state from the already-live exact soft-closed explicit resume or exact suspended-session recover prompt, selects exactly one lawful route, dispatches only exact `/v1/session/resume` or exact `/v1/session/recover`, and keeps returned exact `session_state` plus exact `session_attach_outcome` read-only only outside the exact multi-posture control itself.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        if let displayedResumeMode {
                            ForEach(
                                [
                                    ("source_surface", displayedSourceSurface),
                                    ("resume_mode", displayedResumeMode.rawValue),
                                    ("session_state", displayedSessionState),
                                    ("session_id", displayedSessionID),
                                    ("device_id", displayedDeviceID),
                                ],
                                id: \.0
                            ) { row in
                                metadataRow(label: row.0, value: row.1)
                            }

                            switch displayedResumeMode {
                            case .softClosedExplicitResume:
                                metadataRow(
                                    label: "selected_thread_id",
                                    value: displayedSelectedThreadID ?? "not_provided"
                                )
                                metadataRow(
                                    label: "selected_thread_title",
                                    value: displayedSelectedThreadTitle ?? "not_provided"
                                )
                                metadataRow(
                                    label: "pending_work_order_id",
                                    value: displayedPendingWorkOrderID ?? "not_provided"
                                )
                                metadataRow(
                                    label: "resume_tier",
                                    value: displayedResumeTier ?? "not_provided"
                                )

                                if displayedResumeSummaryBullets.isEmpty {
                                    Text("No bounded `resume_summary_bullets` were provided for this multi-posture soft-closed explicit resume surface.")
                                        .font(.footnote)
                                        .foregroundStyle(.secondary)
                                } else {
                                    ForEach(Array(displayedResumeSummaryBullets.prefix(3).enumerated()), id: \.offset) { index, bullet in
                                        HStack(alignment: .firstTextBaseline, spacing: 10) {
                                            Text("\(index + 1).")
                                                .font(.caption.weight(.semibold))
                                                .foregroundStyle(.secondary)

                                            Text(bullet)
                                                .frame(maxWidth: .infinity, alignment: .leading)
                                        }
                                    }
                                }

                            case .suspendedAuthoritativeRereadRecover:
                                if let displayedRecoveryMode {
                                    metadataRow(label: "recovery_mode", value: displayedRecoveryMode)
                                }
                            }
                        }

                        Text("This exact surface performs one bounded session-resume submission only through already-live exact route-specific seams. No local attach or reopen authority, no broader session-list fetch authority, no local search controls, no local tool invocation controls, no hidden/background wake behavior, and no autonomous unlock are introduced by this shell.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        if let promptState {
                            Button(
                                promptState.resumeMode == .softClosedExplicitResume
                                    ? "Resume the selected thread explicitly"
                                    : "Recover the suspended session authoritatively"
                            ) {
                                Task {
                                    await submitDesktopSessionMultiPostureResume(promptState: promptState)
                                }
                            }
                            .buttonStyle(.borderedProminent)
                            .disabled(activeRuntimeOutcomeState?.phase == .dispatching)
                        }

                        if let activeRuntimeOutcomeState {
                            Divider()

                            Text(activeRuntimeOutcomeState.title)
                                .font(.headline)

                            ForEach(
                                [
                                    ("dispatch_phase", activeRuntimeOutcomeState.phase.rawValue),
                                    ("request_id", activeRuntimeOutcomeState.requestID),
                                    ("endpoint", activeRuntimeOutcomeState.endpoint),
                                    ("outcome", activeRuntimeOutcomeState.outcome ?? "not_available"),
                                    ("reason", activeRuntimeOutcomeState.reason ?? "not_available"),
                                    ("session_id", activeRuntimeOutcomeState.sessionID),
                                    ("session_state", activeRuntimeOutcomeState.sessionState),
                                    ("session_attach_outcome", activeRuntimeOutcomeState.sessionAttachOutcome ?? "not_available"),
                                ],
                                id: \.0
                            ) { row in
                                metadataRow(label: row.0, value: row.1)
                            }

                            Text(activeRuntimeOutcomeState.summary)
                                .frame(maxWidth: .infinity, alignment: .leading)

                            Text(activeRuntimeOutcomeState.detail)
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        } else if let promptState {
                            Text(
                                promptState.resumeMode == .softClosedExplicitResume
                                    ? "Awaiting explicit user-triggered canonical soft-closed explicit resume through the bounded multi-posture control. After submission, any returned exact `session_state` and exact `session_attach_outcome` remain bounded read-only only in this shell."
                                    : "Awaiting explicit user-triggered canonical suspended-session recover through the bounded multi-posture control. After submission, any returned exact `session_state` and exact `session_attach_outcome` remain bounded read-only only in this shell."
                            )
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                        }

                        Text("Cloud-authored, session-bound, and non-authoritative only. This path does not add local attach or reopen authority, broader session-list fetch authority, keyboard text entry beyond the already-landed bounded composer, local search controls, local tool controls, hidden/background wake behavior, wake parity claims, or autonomous unlock.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Multi-Posture Session Resume")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopSessionSuspendedVisibilityCard: some View {
        let sessionSuspendedVisibilityState = desktopSessionSuspendedVisibilityState

        return Group {
            if let sessionSuspendedVisibilityState {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded desktop suspended visibility only. This shell derives one dedicated read-only exact `SESSION_SUSPENDED_VISIBLE` surface from the already-live suspended route context only, preserves hard full takeover explanation, and preserves allowed-next-step-only posture without introducing any local suspended-session release, fresh-read, repeated-attempt, wake-renewal, attach, or reopen authority.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        ForEach(
                            [
                                ("source_surface", sessionSuspendedVisibilityState.sourceSurfaceIdentity),
                                ("session_state", sessionSuspendedVisibilityState.sessionState),
                                ("session_id", sessionSuspendedVisibilityState.sessionID),
                                (
                                    "next_allowed_actions_may_speak",
                                    booleanValue(sessionSuspendedVisibilityState.nextAllowedActionsMaySpeak)
                                ),
                                (
                                    "next_allowed_actions_must_wait",
                                    booleanValue(sessionSuspendedVisibilityState.nextAllowedActionsMustWait)
                                ),
                                (
                                    "next_allowed_actions_must_" +
                                        "re" +
                                        "wake",
                                    booleanValue(sessionSuspendedVisibilityState.nextAllowedActionsMustRewake)
                                ),
                            ],
                            id: \.0
                        ) { row in
                            metadataRow(label: row.0, value: row.1)
                        }

                        if let recoveryMode = sessionSuspendedVisibilityState.recoveryMode {
                            metadataRow(label: "recovery_mode", value: recoveryMode)
                        }

                        if let reconciliationDecision = sessionSuspendedVisibilityState.reconciliationDecision {
                            metadataRow(label: "reconciliation_decision", value: reconciliationDecision)
                        }

                        Text("This suspended posture remains a hard full takeover, and the shell preserves only allowed-next-step visibility here in bounded read-only form.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text("No local suspended-session release, no local fresh-read, no local repeated-attempt, no local wake-renewal, no local session attach, and no local reopen production authority exist here. This shell stays explicitly non-authoritative.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Session Suspended Visibility")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopRecoveryVisibilityCard: some View {
        let recoveryVisibilityState = desktopRecoveryVisibilityState

        return Group {
            if let recoveryVisibilityState {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded desktop recovery visibility only. This shell derives one dedicated read-only recovery surface from the already-live recovery-visible carriers only, preserves exact `RECOVERING`, exact `DEGRADED_RECOVERY`, and exact `QUARANTINED_LOCAL_STATE`, and preserves reread-authoritative-state explanation without introducing any local reread, retry, queue-repair, attach, reopen, transcript, or archive authority.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        ForEach(
                            [
                                ("recovery_posture", recoveryVisibilityState.displayState.rawValue),
                                ("source_surface", recoveryVisibilityState.sourceSurfaceIdentity),
                                ("session_state", recoveryVisibilityState.sessionState),
                                ("session_id", recoveryVisibilityState.sessionID),
                            ],
                            id: \.0
                        ) { row in
                            metadataRow(label: row.0, value: row.1)
                        }

                        if let recoveryMode = recoveryVisibilityState.recoveryMode {
                            metadataRow(label: "recovery_mode", value: recoveryMode)
                        }

                        if let reconciliationDecision = recoveryVisibilityState.reconciliationDecision {
                            metadataRow(label: "reconciliation_decision", value: reconciliationDecision)
                        }

                        Text(
                            recoveryVisibilityState.displayState == .quarantinedLocalState
                                ? "QUARANTINED_LOCAL_STATE remains a hard full takeover and authoritative reread remains required before any normal interaction can be reconsidered."
                                : "\(recoveryVisibilityState.displayState.rawValue) remains inline restriction while the lawful main session surface remains visible in bounded read-only form."
                        )
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)

                        Text(recoveryRestrictionSummary(for: recoveryVisibilityState.displayState))
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text("No local reread, no local retry, no local queue repair, no local transcript or archive fabrication, no local session attach, and no local reopen production authority exist here. This shell stays explicitly non-authoritative.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Recovery Visibility")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopInterruptVisibilityCard: some View {
        let interruptVisibilityState = desktopInterruptVisibilityState

        return Group {
            if let interruptVisibilityState {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded desktop interrupt visibility only. This shell derives one dedicated read-only exact `INTERRUPT_VISIBLE` surface from the already-live active-session interrupt context only, preserves accepted interrupt posture explanation, and preserves one exact disabled non-producing clarify / continue previous topic / switch topic / resume later affordance set without introducing new interrupt response production, local interrupt law, or local routing / threshold authority.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        ForEach(
                            [
                                ("source_surface", interruptVisibilityState.sourceSurfaceIdentity),
                                ("session_state", interruptVisibilityState.sessionState),
                                ("session_id", interruptVisibilityState.sessionID),
                                ("turn_id", interruptVisibilityState.turnID),
                            ],
                            id: \.0
                        ) { row in
                            metadataRow(label: row.0, value: row.1)
                        }

                        ForEach(interruptVisibilityState.interruptContinuityRows, id: \.label) { row in
                            metadataRow(label: row.label, value: row.value)
                        }

                        if let returnCheckPending = interruptVisibilityState.returnCheckPending {
                            metadataRow(
                                label: "return_check_pending",
                                value: booleanValue(returnCheckPending)
                            )
                        }

                        Text("Accepted interrupt posture remains cloud-authored and active-session-bound here, so this shell renders continuity truth without introducing local interrupt execution authority.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text(interruptVisibilityState.acceptedInterruptPostureSummary)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        VStack(alignment: .leading, spacing: 8) {
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

                        Text("No new interrupt response production, no local interrupt law, no local routing guidance, no local threshold law, no local clarify / continue / switch-topic / resume-later authority, and no local dispatch unlock authority are introduced here. This shell stays explicitly non-authoritative.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Interrupt Visibility")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopInterruptResponseProductionCard: some View {
        let interruptResponseProductionState = desktopInterruptResponseProductionState

        return Group {
            if let interruptResponseProductionState {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded desktop interrupt response production only. This shell derives one dedicated exact `INTERRUPT_VISIBLE` response-production surface from the already-live active-session interrupt context only, preserves the cloud-authored clarify-directive and return-check submission path already live in source, and keeps broader interrupt detail branches outside this selected implementation seam.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        ForEach(
                            [
                                (
                                    "source_surface",
                                    interruptResponseProductionState.sourceSurfaceIdentity
                                ),
                                ("session_state", interruptResponseProductionState.sessionState),
                                ("session_id", interruptResponseProductionState.sessionID),
                                ("turn_id", interruptResponseProductionState.turnID),
                            ],
                            id: \.0
                        ) { row in
                            metadataRow(label: row.0, value: row.1)
                        }

                        ForEach(
                            interruptResponseProductionState.interruptContinuityRows,
                            id: \.label
                        ) { row in
                            metadataRow(label: row.label, value: row.value)
                        }

                        if let returnCheckPending = interruptResponseProductionState.returnCheckPending {
                            metadataRow(
                                label: "return_check_pending",
                                value: booleanValue(returnCheckPending)
                            )
                        }

                        Text("Continuity-response posture remains cloud-authored and active-session-bound here, so this dedicated surface reuses only the already-live lawful clarify-directive and return-check production path without widening into local interrupt execution authority.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        if interruptResponseProductionState.hasInterruptResponseConflict {
                            Text("Authoritative interruption truth exposed both clarify-directive detail and a return check, so this dedicated response-production surface fails closed until the cloud narrows to one lawful path.")
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        } else if interruptResponseProductionState.hasLawfulInterruptClarifyDirective {
                            Text("Cloud-authored clarify directive currently defines the bounded response-production posture here.")
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        } else if interruptResponseProductionState.returnCheckPending == true {
                            Text("Cloud-authored return check currently defines the bounded response-production posture here.")
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }

                        interruptResponseProductionSection(
                            interruptResponseProductionState.activeContext
                        )

                        Text("No local interrupt law, no local ambiguity inference, no local field inference, no local routing guidance, no local confirmation law, no local sensitivity policy, no local threshold law, and no local identity-resolution authority are introduced here. This shell stays explicitly non-authoritative.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Interrupt Response Production")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopInterruptSubjectReferencesVisibilityCard: some View {
        let interruptSubjectReferencesVisibilityState = desktopInterruptSubjectReferencesVisibilityState

        return Group {
            if let interruptSubjectReferencesVisibilityState {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded desktop interrupt subject-references visibility only. This shell derives one dedicated exact `INTERRUPT_VISIBLE` subject-reference surface from the already-live active-session interrupt context only, preserves cloud-authored continuity subject evidence in read-only form, and keeps broader interrupt detail and response-production branches outside this selected implementation seam.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        ForEach(
                            [
                                (
                                    "source_surface",
                                    interruptSubjectReferencesVisibilityState.sourceSurfaceIdentity
                                ),
                                ("session_state", interruptSubjectReferencesVisibilityState.sessionState),
                                ("session_id", interruptSubjectReferencesVisibilityState.sessionID),
                                ("turn_id", interruptSubjectReferencesVisibilityState.turnID),
                            ],
                            id: \.0
                        ) { row in
                            metadataRow(label: row.0, value: row.1)
                        }

                        if let interruptSubjectRelation = interruptSubjectReferencesVisibilityState
                            .interruptSubjectRelation
                        {
                            metadataRow(
                                label: "interrupt_subject_relation",
                                value: interruptSubjectRelation
                            )
                        }

                        Text("Subject-reference posture remains cloud-authored and active-session-bound here, so this dedicated surface renders continuity subject evidence without widening into local subject execution authority.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        if interruptSubjectReferencesVisibilityState.hasLawfulInterruptSubjectReferences {
                            interruptSubjectReferencesCard(
                                activeSubjectRef: interruptSubjectReferencesVisibilityState
                                    .activeSubjectRef,
                                interruptedSubjectRef: interruptSubjectReferencesVisibilityState
                                    .interruptedSubjectRef
                            )
                        }

                        Text("Subject-reference evidence remains evidence-only here and does not grant local subject binding, local subject reconciliation, local identity resolution, or local dispatch unlock authority. This shell stays explicitly non-authoritative.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Interrupt Subject References")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopInterruptSubjectRelationConfidenceVisibilityCard: some View {
        let interruptSubjectRelationConfidenceVisibilityState =
            desktopInterruptSubjectRelationConfidenceVisibilityState

        return Group {
            if let interruptSubjectRelationConfidenceVisibilityState {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded desktop interrupt subject-relation-confidence visibility only. This shell derives one dedicated exact `INTERRUPT_VISIBLE` confidence surface from the already-live active-session interrupt context only, preserves cloud-authored continuity confidence evidence in read-only form, and keeps broader interrupt detail, response-production, and subject-reference branches outside this selected implementation seam.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        ForEach(
                            [
                                (
                                    "source_surface",
                                    interruptSubjectRelationConfidenceVisibilityState
                                        .sourceSurfaceIdentity
                                ),
                                (
                                    "session_state",
                                    interruptSubjectRelationConfidenceVisibilityState.sessionState
                                ),
                                (
                                    "session_id",
                                    interruptSubjectRelationConfidenceVisibilityState.sessionID
                                ),
                                (
                                    "turn_id",
                                    interruptSubjectRelationConfidenceVisibilityState.turnID
                                ),
                            ],
                            id: \.0
                        ) { row in
                            metadataRow(label: row.0, value: row.1)
                        }

                        if let interruptSubjectRelation =
                            interruptSubjectRelationConfidenceVisibilityState
                            .interruptSubjectRelation
                        {
                            metadataRow(
                                label: "interrupt_subject_relation",
                                value: interruptSubjectRelation
                            )
                        }

                        Text("Subject-relation-confidence posture remains cloud-authored and active-session-bound here, so this dedicated surface renders continuity confidence evidence without widening into local dispatch or threshold authority.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        if
                            interruptSubjectRelationConfidenceVisibilityState
                            .hasLawfulInterruptSubjectRelationConfidence,
                            let interruptSubjectRelationConfidence =
                                interruptSubjectRelationConfidenceVisibilityState
                                .interruptSubjectRelationConfidence
                        {
                            interruptSubjectRelationConfidenceCard(
                                interruptSubjectRelationConfidence
                            )
                        }

                        Text("Confidence evidence remains evidence-only here and does not grant local threshold law, local dispatch unlock, local subject binding, or local identity-resolution authority. This shell stays explicitly non-authoritative.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Interrupt Subject Relation Confidence")
                        .font(.headline)
                }
            }
        }
    }

    private var desktopInterruptReturnCheckExpiryVisibilityCard: some View {
        let interruptReturnCheckExpiryVisibilityState =
            desktopInterruptReturnCheckExpiryVisibilityState

        return Group {
            if let interruptReturnCheckExpiryVisibilityState {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Bounded desktop interrupt return-check-expiry visibility only. This shell derives one dedicated exact `INTERRUPT_VISIBLE` expiry surface from the already-live active-session interrupt context only, preserves cloud-authored return-check expiry evidence in read-only form, and keeps broader interrupt detail, response-production, subject-reference, subject-relation-confidence, resume-buffer, and TTS-resume-snapshot branches outside this selected implementation seam.")
                            .frame(maxWidth: .infinity, alignment: .leading)

                        ForEach(
                            [
                                (
                                    "source_surface",
                                    interruptReturnCheckExpiryVisibilityState
                                        .sourceSurfaceIdentity
                                ),
                                (
                                    "session_state",
                                    interruptReturnCheckExpiryVisibilityState.sessionState
                                ),
                                (
                                    "session_id",
                                    interruptReturnCheckExpiryVisibilityState.sessionID
                                ),
                                (
                                    "turn_id",
                                    interruptReturnCheckExpiryVisibilityState.turnID
                                ),
                            ],
                            id: \.0
                        ) { row in
                            metadataRow(label: row.0, value: row.1)
                        }

                        ForEach(
                            interruptReturnCheckExpiryVisibilityState.interruptContinuityRows,
                            id: \.label
                        ) { row in
                            metadataRow(label: row.label, value: row.value)
                        }

                        metadataRow(
                            label: "return_check_pending",
                            value: interruptReturnCheckExpiryVisibilityState.returnCheckPending
                                .map(booleanValue) ?? "not_provided"
                        )

                        Text("Return-check-expiry posture remains cloud-authored and active-session-bound here, so this dedicated surface renders expiry evidence without widening into local timer law, local dispatch, or local subject authority.")
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        if
                            interruptReturnCheckExpiryVisibilityState
                            .hasLawfulInterruptReturnCheckExpiry,
                            let returnCheckExpiresAt =
                                interruptReturnCheckExpiryVisibilityState.returnCheckExpiresAt
                        {
                            interruptReturnCheckExpiryCard(returnCheckExpiresAt)
                        }

                        Text("Expiry evidence remains evidence-only here and does not grant local timer law, local dispatch unlock, local subject binding, or local identity-resolution authority. This shell stays explicitly non-authoritative.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                } label: {
                    Text("Interrupt Return Check Expiry")
                        .font(.headline)
                }
            }
        }
    }

    private var sessionCard: some View {
        Group {
            if let foregroundSessionSuspendedVisibleContext {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored desktop suspended-session evidence only.")
                            .font(.subheadline.weight(.semibold))

                        Text("Bounded read-only suspended posture for the cloud-authoritative desktop session surface.")
                            .foregroundStyle(.secondary)

                        ForEach(foregroundSessionSuspendedVisibleContext.suspendedStatusRows, id: \.label) { row in
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
            } else if let foregroundSessionSoftClosedVisibleContext {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored desktop soft-closed evidence only.")
                            .font(.subheadline.weight(.semibold))

                        Text("Bounded read-only soft-closed session posture for the cloud-authoritative desktop session surface.")
                            .foregroundStyle(.secondary)

                        metadataRow(label: "session_state", value: foregroundSessionSoftClosedVisibleContext.sessionState)
                        metadataRow(label: "session_id", value: foregroundSessionSoftClosedVisibleContext.sessionID)

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
            } else if let foregroundSessionActiveVisibleContext {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored desktop active-session evidence only.")
                            .font(.subheadline.weight(.semibold))

                        Text("Bounded read-only current session and current turn posture for the cloud-authoritative desktop session surface.")
                            .foregroundStyle(.secondary)

                        metadataRow(label: "session_state", value: foregroundSessionActiveVisibleContext.sessionState)
                        metadataRow(label: "session_id", value: foregroundSessionActiveVisibleContext.sessionID)
                        metadataRow(label: "turn_id", value: foregroundSessionActiveVisibleContext.turnID)

                        if let sessionAttachOutcome = foregroundSessionActiveVisibleContext.sessionAttachOutcome {
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
            } else if let foregroundSessionHeaderContext {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored desktop session-header evidence only.")
                            .font(.subheadline.weight(.semibold))

                        Text("Bounded read-only session posture for the current cloud-authoritative desktop session surface.")
                            .foregroundStyle(.secondary)

                        metadataRow(label: "session_state", value: foregroundSessionHeaderContext.sessionState)
                        metadataRow(label: "session_id", value: foregroundSessionHeaderContext.sessionID)
                        metadataRow(
                            label: "session_attach_outcome",
                            value: foregroundSessionHeaderContext.sessionAttachOutcome
                        )

                        Text(continuityLabel(for: foregroundSessionHeaderContext.sessionAttachOutcome))
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
        let shouldSuppressSupportRailCurrentSessionTranscript =
            desktopConversationShouldSuppressSupportRailCurrentSessionTranscript()
        let shouldSuppressSupportRailArchivedRecentSliceTranscript =
            desktopConversationShouldSuppressSupportRailArchivedRecentSliceTranscript()

        return Group {
            if foregroundSessionSuspendedVisibleContext != nil {
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
            } else if let foregroundSessionActiveVisibleContext {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored live dual-transcript evidence only.")
                            .font(.subheadline.weight(.semibold))

                        if !shouldSuppressSupportRailCurrentSessionTranscript {
                            transcriptEntry(
                                speaker: "You",
                                posture: "current_user_turn_text",
                                body: foregroundSessionActiveVisibleContext.currentUserTurnText,
                                detail: "Current user turn remains text-visible, session-bound, and cloud-authoritative for this active desktop session."
                            )

                            transcriptEntry(
                                speaker: "Selene",
                                posture: "current_selene_turn_text",
                                body: foregroundSessionActiveVisibleContext.currentSeleneTurnText,
                                detail: "Current Selene turn remains text-visible and tied to the same active cloud session without a local-only transcript fork."
                            )
                        }

                        Text("No local transcript authority, no local turn synthesis, and no local dispatch unlock are introduced by this bounded desktop surface.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                } label: {
                    Text("History")
                        .font(.headline)
                }
            } else if let foregroundSessionSoftClosedVisibleContext {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored archived recent-slice evidence only.")
                            .font(.subheadline.weight(.semibold))

                        if !shouldSuppressSupportRailArchivedRecentSliceTranscript {
                            transcriptEntry(
                                speaker: "You",
                                posture: "archived_user_turn_text",
                                body: foregroundSessionSoftClosedVisibleContext.archivedUserTurnText,
                                detail: "Archived recent slice remains durable archived conversation truth and stays distinct from bounded PH1.M resume-context output."
                            )

                            transcriptEntry(
                                speaker: "Selene",
                                posture: "archived_selene_turn_text",
                                body: foregroundSessionSoftClosedVisibleContext.archivedSeleneTurnText,
                                detail: "Archived recent slice remains text-visible after visual reset without local auto-reopen, hidden spoken-only output, or local transcript authority."
                            )
                        }

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
            if let foregroundSessionSuspendedVisibleContext {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored allowed-next-step evidence only.")
                            .font(.subheadline.weight(.semibold))

                        ForEach(foregroundSessionSuspendedVisibleContext.allowedNextStepRows, id: \.label) { row in
                            metadataRow(label: row.label, value: row.value)
                        }

                        Text(foregroundSessionSuspendedVisibleContext.allowedNextStepSummary)
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
            } else if let foregroundSessionActiveVisibleContext {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored governed-output summary evidence only.")
                            .font(.subheadline.weight(.semibold))

                        metadataRow(
                            label: "current_governed_output_summary",
                            value: foregroundSessionActiveVisibleContext.currentGovernedOutputSummary
                        )

                        if foregroundSessionActiveVisibleContext
                            .hasLawfulOnboardingPlatformSetupReceiptCarrierFamily
                        {
                            onboardingPlatformSetupReceiptCard(foregroundSessionActiveVisibleContext)
                        }

                        if foregroundSessionActiveVisibleContext.hasLawfulAuthorityStateCarrierFamily {
                            authorityStateCard(foregroundSessionActiveVisibleContext)
                        }

                        if foregroundSessionActiveVisibleContext
                            .hasLawfulWakeRuntimeEventEvidenceCarrierFamily
                        {
                            wakeRuntimeEventEvidenceCard(foregroundSessionActiveVisibleContext)
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
            } else if let foregroundSessionSoftClosedVisibleContext {
                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Cloud-authored PH1.M resume-context evidence only.")
                            .font(.subheadline.weight(.semibold))

                        ForEach([
                            ("selected_thread_id", foregroundSessionSoftClosedVisibleContext.selectedThreadID ?? "not_provided"),
                            ("selected_thread_title", foregroundSessionSoftClosedVisibleContext.selectedThreadTitle ?? "not_provided"),
                            ("pending_work_order_id", foregroundSessionSoftClosedVisibleContext.pendingWorkOrderID ?? "not_provided"),
                            ("resume_tier", foregroundSessionSoftClosedVisibleContext.resumeTier ?? "not_provided"),
                        ], id: \.0) { row in
                            metadataRow(label: row.0, value: row.1)
                        }

                        if foregroundSessionSoftClosedVisibleContext.resumeSummaryBullets.isEmpty {
                            Text("No bounded `resume_summary_bullets` were provided for this soft-closed preview.")
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                        } else {
                            ForEach(Array(foregroundSessionSoftClosedVisibleContext.resumeSummaryBullets.prefix(3).enumerated()), id: \.offset) { index, bullet in
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
                      let foregroundSessionActiveVisibleContext
            {
                interruptVisibleCard(foregroundSessionActiveVisibleContext)
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
                        ("source_surface", "EXPLICIT_VOICE_PENDING"),
                        ("capture_mode", "foreground_only"),
                        ("transcript_posture", "non_authoritative_preview"),
                        ("transcript_bytes", "\(request.byteCount)"),
                        ("selected_mic", request.audioCaptureRefState.selectedMic),
                        ("selected_speaker", request.audioCaptureRefState.selectedSpeaker),
                        ("device_route", request.audioCaptureRefState.deviceRoute),
                        ("locale_tag", request.audioCaptureRefState.localeTag),
                        ("tts_playback_active", request.audioCaptureRefState.ttsPlaybackActive ? "true" : "false"),
                        ("capture_degraded", request.audioCaptureRefState.captureDegraded ? "true" : "false"),
                        ("stream_gap_detected", request.audioCaptureRefState.streamGapDetected ? "true" : "false"),
                        ("device_changed", request.audioCaptureRefState.deviceChanged ? "true" : "false"),
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

                Text("The exact structured `audioCaptureRef` shown here is transport-only, session-bound, and non-authoritative while canonical runtime dispatch and later cloud-visible response posture resolve.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)

                Text("This path reuses existing exact `/v1/voice/turn` for explicit voice only and does not add wake-listener integration, wake-to-turn handoff, backend mutation, thread authoring, pinned-context authoring, device-turn-sequence authoring, hidden/background auto-start, or autonomous unlock.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text("Explicit Voice Turn Request")
                .font(.headline)
        }
    }

    private func desktopTypedTurnPendingRequestCard(
        _ request: DesktopTypedTurnRequestState
    ) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 10) {
                Text("Awaiting authoritative response")
                    .font(.headline)

                ForEach(
                    [
                        ("request_id", request.id),
                        ("source_surface", request.origin.pendingSourceSurface),
                        ("trigger", "EXPLICIT"),
                        ("content_type", "text/plain"),
                        ("text_posture", "non_authoritative_preview"),
                        ("text_bytes", "\(request.byteCount)"),
                        ("audio_capture_ref", "nil"),
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

                Text(request.origin.pendingSummary)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        } label: {
            Text(request.origin.cardTitle)
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
        desktopEmployeePhotoCaptureSendRuntimeOutcomeState = nil
        desktopEmployeeSenderVerifyCommitRuntimeOutcomeState = nil
        desktopPrimaryDeviceConfirmRuntimeOutcomeState = nil
        desktopWakeEnrollStartDraftRuntimeOutcomeState = nil
        desktopWakeEnrollSampleCommitRuntimeOutcomeState = nil
        desktopWakeEnrollCompleteCommitRuntimeOutcomeState = nil
        desktopEmoPersonaLockRuntimeOutcomeState = nil
        desktopAccessProvisionCommitRuntimeOutcomeState = nil
        desktopCompleteCommitRuntimeOutcomeState = nil
        desktopPairingCompletionCommitRuntimeOutcomeState = nil
        desktopEmployeeSenderVerifyCommitSelectedDecision = "CONFIRM"

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
        desktopEmployeePhotoCaptureSendRuntimeOutcomeState = nil
        desktopEmployeeSenderVerifyCommitRuntimeOutcomeState = nil
        desktopPrimaryDeviceConfirmRuntimeOutcomeState = nil
        desktopWakeEnrollStartDraftRuntimeOutcomeState = nil
        desktopWakeEnrollSampleCommitRuntimeOutcomeState = nil
        desktopWakeEnrollCompleteCommitRuntimeOutcomeState = nil
        desktopEmoPersonaLockRuntimeOutcomeState = nil
        desktopAccessProvisionCommitRuntimeOutcomeState = nil
        desktopCompleteCommitRuntimeOutcomeState = nil
        desktopPairingCompletionCommitRuntimeOutcomeState = nil
        desktopEmployeeSenderVerifyCommitSelectedDecision = "CONFIRM"

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
        desktopEmployeePhotoCaptureSendRuntimeOutcomeState = nil
        desktopEmployeeSenderVerifyCommitRuntimeOutcomeState = nil
        desktopPrimaryDeviceConfirmRuntimeOutcomeState = nil
        desktopWakeEnrollStartDraftRuntimeOutcomeState = nil
        desktopWakeEnrollSampleCommitRuntimeOutcomeState = nil
        desktopWakeEnrollCompleteCommitRuntimeOutcomeState = nil
        desktopEmoPersonaLockRuntimeOutcomeState = nil
        desktopAccessProvisionCommitRuntimeOutcomeState = nil
        desktopCompleteCommitRuntimeOutcomeState = nil
        desktopPairingCompletionCommitRuntimeOutcomeState = nil
        desktopEmployeeSenderVerifyCommitSelectedDecision = "CONFIRM"

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
    private func submitDesktopEmployeePhotoCaptureSend(
        promptState: DesktopEmployeePhotoCaptureSendPromptState
    ) async {
        let activeEntryContextID = desktopOnboardingEntryContext?.id
        desktopEmployeeSenderVerifyCommitRuntimeOutcomeState = nil
        desktopPrimaryDeviceConfirmRuntimeOutcomeState = nil
        desktopWakeEnrollStartDraftRuntimeOutcomeState = nil
        desktopWakeEnrollSampleCommitRuntimeOutcomeState = nil
        desktopWakeEnrollCompleteCommitRuntimeOutcomeState = nil
        desktopEmoPersonaLockRuntimeOutcomeState = nil
        desktopAccessProvisionCommitRuntimeOutcomeState = nil
        desktopCompleteCommitRuntimeOutcomeState = nil
        desktopPairingCompletionCommitRuntimeOutcomeState = nil
        desktopEmployeeSenderVerifyCommitSelectedDecision = "CONFIRM"

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopEmployeePhotoCaptureSendRequestBuilder(
                promptState: promptState,
                photoBlobRef: desktopEmployeePhotoCaptureSendPhotoBlobRefInput
            )
            desktopEmployeePhotoCaptureSendRuntimeOutcomeState = .dispatching(
                onboardingSessionID: ingressContext.onboardingSessionID,
                photoBlobRef: ingressContext.photoBlobRef,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.submitDesktopEmployeePhotoCaptureSend(
                ingressContext
            )
            guard desktopOnboardingEntryContext?.id == activeEntryContextID else {
                return
            }

            desktopEmployeePhotoCaptureSendRuntimeOutcomeState = outcomeState
            desktopEmployeeSenderVerifyCommitSelectedDecision = "CONFIRM"
        } catch {
            desktopEmployeePhotoCaptureSendRuntimeOutcomeState = .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                photoBlobRef: {
                    let trimmed = desktopEmployeePhotoCaptureSendPhotoBlobRefInput
                        .trimmingCharacters(in: .whitespacesAndNewlines)
                    return trimmed.isEmpty ? nil : trimmed
                }(),
                endpoint: desktopCanonicalRuntimeBridge.onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded employee photo capture send request.",
                detail: error.localizedDescription
            )
        }
    }

    @MainActor
    private func submitDesktopEmployeeSenderVerifyCommit(
        promptState: DesktopEmployeeSenderVerifyCommitPromptState
    ) async {
        let activeEntryContextID = desktopOnboardingEntryContext?.id
        desktopPrimaryDeviceConfirmRuntimeOutcomeState = nil
        desktopWakeEnrollStartDraftRuntimeOutcomeState = nil
        desktopWakeEnrollSampleCommitRuntimeOutcomeState = nil
        desktopWakeEnrollCompleteCommitRuntimeOutcomeState = nil
        desktopEmoPersonaLockRuntimeOutcomeState = nil
        desktopAccessProvisionCommitRuntimeOutcomeState = nil
        desktopCompleteCommitRuntimeOutcomeState = nil
        desktopPairingCompletionCommitRuntimeOutcomeState = nil

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopEmployeeSenderVerifyCommitRequestBuilder(
                promptState: promptState,
                senderDecision: desktopEmployeeSenderVerifyCommitSelectedDecision
            )
            desktopEmployeeSenderVerifyCommitRuntimeOutcomeState = .dispatching(
                onboardingSessionID: ingressContext.onboardingSessionID,
                senderDecision: ingressContext.senderDecision,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.submitDesktopEmployeeSenderVerifyCommit(
                ingressContext
            )
            guard desktopOnboardingEntryContext?.id == activeEntryContextID else {
                return
            }

            desktopEmployeeSenderVerifyCommitRuntimeOutcomeState = outcomeState
        } catch {
            desktopEmployeeSenderVerifyCommitRuntimeOutcomeState = .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                senderDecision: boundedOnboardingContinueFieldInput(desktopEmployeeSenderVerifyCommitSelectedDecision),
                endpoint: desktopCanonicalRuntimeBridge.onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded sender verification commit request.",
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
        desktopEmoPersonaLockRuntimeOutcomeState = nil
        desktopAccessProvisionCommitRuntimeOutcomeState = nil
        desktopCompleteCommitRuntimeOutcomeState = nil
        desktopPairingCompletionCommitRuntimeOutcomeState = nil

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
        desktopEmoPersonaLockRuntimeOutcomeState = nil
        desktopAccessProvisionCommitRuntimeOutcomeState = nil
        desktopCompleteCommitRuntimeOutcomeState = nil
        desktopPairingCompletionCommitRuntimeOutcomeState = nil

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
        desktopWakeEnrollDeferCommitRuntimeOutcomeState = nil
        desktopEmoPersonaLockRuntimeOutcomeState = nil
        desktopAccessProvisionCommitRuntimeOutcomeState = nil
        desktopCompleteCommitRuntimeOutcomeState = nil
        desktopPairingCompletionCommitRuntimeOutcomeState = nil

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
        desktopWakeEnrollDeferCommitRuntimeOutcomeState = nil
        desktopEmoPersonaLockRuntimeOutcomeState = nil
        desktopAccessProvisionCommitRuntimeOutcomeState = nil
        desktopCompleteCommitRuntimeOutcomeState = nil
        desktopPairingCompletionCommitRuntimeOutcomeState = nil

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
        desktopWakeEnrollDeferCommitRuntimeOutcomeState = nil
        desktopEmoPersonaLockRuntimeOutcomeState = nil
        desktopAccessProvisionCommitRuntimeOutcomeState = nil
        desktopCompleteCommitRuntimeOutcomeState = nil
        desktopPairingCompletionCommitRuntimeOutcomeState = nil

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
    private func submitDesktopWakeEnrollDeferCommit(
        promptState: DesktopWakeEnrollDeferCommitPromptState
    ) async {
        let activeEntryContextID = desktopOnboardingEntryContext?.id
        desktopEmoPersonaLockRuntimeOutcomeState = nil
        desktopAccessProvisionCommitRuntimeOutcomeState = nil
        desktopCompleteCommitRuntimeOutcomeState = nil
        desktopPairingCompletionCommitRuntimeOutcomeState = nil

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopWakeEnrollDeferCommitRequestBuilder(
                promptState
            )
            desktopWakeEnrollDeferCommitRuntimeOutcomeState = .dispatching(
                onboardingSessionID: ingressContext.onboardingSessionID,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.submitDesktopWakeEnrollDeferCommit(
                ingressContext
            )
            guard desktopOnboardingEntryContext?.id == activeEntryContextID else {
                return
            }

            desktopWakeEnrollDeferCommitRuntimeOutcomeState = outcomeState
        } catch {
            desktopWakeEnrollDeferCommitRuntimeOutcomeState = .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                deviceID: promptState.deviceID,
                endpoint: desktopCanonicalRuntimeBridge.onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop wake-enroll defer-commit request.",
                detail: error.localizedDescription
            )
        }
    }

    @MainActor
    private func submitDesktopPairingCompletionCommit(
        promptState: DesktopPairingCompletionPromptState
    ) async {
        desktopReadyTimeHandoffState = nil

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopPairingCompletionCommitRequestBuilder(
                promptState
            )
            guard desktopPairingCompletionPromptState?.id == promptState.id else {
                desktopPairingCompletionCommitRuntimeOutcomeState = .failed(
                    ingressContext: ingressContext,
                    summary: "The pairing-completion prompt disappeared before dispatch.",
                    detail: "This shell fails closed when the exact lawful pairing-completion prompt no longer remains available before canonical dispatch can begin."
                )
                return
            }

            desktopPairingCompletionCommitRuntimeOutcomeState = .dispatching(
                ingressContext: ingressContext
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.submitDesktopPairingCompletionCommit(
                ingressContext
            )
            guard let currentPromptState = desktopPairingCompletionPromptState,
                  currentPromptState.id == promptState.id else {
                desktopPairingCompletionCommitRuntimeOutcomeState = .failed(
                    ingressContext: ingressContext,
                    summary: "The pairing-completion prompt disappeared before completion.",
                    detail: "This shell fails closed when the exact lawful pairing-completion prompt no longer remains visible by the time canonical pairing-completion commit returns."
                )
                return
            }

            desktopPairingCompletionCommitRuntimeOutcomeState = outcomeState
            guard outcomeState.phase == .completed,
                  outcomeState.nextStep == "READY" else {
                return
            }

            desktopReadyTimeHandoffState = DesktopReadyTimeHandoffState(promptState: currentPromptState)
        } catch {
            desktopPairingCompletionCommitRuntimeOutcomeState = DesktopPairingCompletionCommitRuntimeOutcomeState(
                id: "desktop_pairing_completion_commit_request_unavailable",
                phase: .failed,
                title: "Desktop pairing completion commit failed",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop pairing-completion request.",
                detail: error.localizedDescription,
                endpoint: desktopCanonicalRuntimeBridge.onboardingContinueEndpoint,
                requestID: "unavailable",
                outcome: nil,
                reason: nil,
                sourceSurfaceIdentity: promptState.sourceSurfaceIdentity,
                onboardingSessionID: promptState.onboardingSessionID,
                nextStep: promptState.nextStep,
                onboardingStatus: promptState.onboardingStatus,
                voiceArtifactSyncReceiptRef: promptState.voiceArtifactSyncReceiptRef,
                accessEngineInstanceID: promptState.accessEngineInstanceID,
                deviceID: promptState.deviceID ?? "not_provided",
                sessionState: promptState.sessionState,
                sessionID: promptState.sessionID,
                sessionAttachOutcome: promptState.sessionAttachOutcome,
                turnID: promptState.turnID
            )
        }
    }

    @MainActor
    private func synchronizeDesktopPairingCompletionReadyTimeHandoffState() {
        guard let desktopReadyTimeHandoffState else {
            return
        }

        guard let promptState = desktopPairingCompletionPromptState,
              desktopReadyTimeHandoffState.matches(promptState) else {
            self.desktopReadyTimeHandoffState = nil
            return
        }
    }

    @MainActor
    private func synchronizeDesktopWakeProfileAvailabilityRuntimeOutcomeState() {
        guard let promptState = desktopWakeProfileAvailabilityPromptState else {
            if desktopWakeProfileAvailabilityRuntimeOutcomeState?.phase != .dispatching {
                desktopWakeProfileAvailabilityRuntimeOutcomeState = nil
            }
            return
        }

        guard let desktopWakeProfileAvailabilityRuntimeOutcomeState,
              desktopWakeProfileAvailabilityRuntimeOutcomeState.phase != .dispatching else {
            return
        }

        guard desktopWakeProfileAvailabilityRuntimeOutcomeState.receiptKind == promptState.receiptKind,
              desktopWakeProfileAvailabilityRuntimeOutcomeState.deviceID == promptState.deviceID,
              desktopWakeProfileAvailabilityRuntimeOutcomeState.wakeProfileID == promptState.wakeProfileID,
              desktopWakeProfileAvailabilityRuntimeOutcomeState.voiceArtifactSyncReceiptRef == promptState.voiceArtifactSyncReceiptRef else {
            self.desktopWakeProfileAvailabilityRuntimeOutcomeState = nil
            return
        }
    }

    @MainActor
    private func synchronizeDesktopWakeListenerControllerState() {
        let explicitVoiceOwnsMicrophone = explicitVoiceController.isListening
            || explicitVoiceController.pendingRequest != nil
        let wakeDispatchInFlight = desktopWakeListenerController.listenerState == .dispatching

        guard scenePhase == .active,
              desktopOperationalConversationShellState != nil else {
            if desktopWakeListenerController.listenerState.isActiveForMicrophone
                || (desktopWakeListenerController.pendingRequest != nil && !wakeDispatchInFlight)
                || (lastStagedWakeTriggeredVoiceTurnRequestState != nil && !wakeDispatchInFlight) {
                desktopWakeListenerController.haltCaptureSession()
                if !wakeDispatchInFlight {
                    desktopWakeListenerController.clearPendingPreparedWakeTurn()
                    lastStagedWakeTriggeredVoiceTurnRequestState = nil
                }
            }
            return
        }

        if explicitVoiceOwnsMicrophone {
            if desktopWakeListenerController.listenerState.isActiveForMicrophone
                || (desktopWakeListenerController.pendingRequest != nil && !wakeDispatchInFlight) {
                desktopWakeListenerController.haltCaptureSession()
                if !wakeDispatchInFlight {
                    desktopWakeListenerController.clearPendingPreparedWakeTurn()
                    lastStagedWakeTriggeredVoiceTurnRequestState = nil
                }
            }
            return
        }

        guard let promptState = desktopWakeListenerPromptState else {
            if desktopWakeListenerController.listenerState.isActiveForMicrophone
                || (desktopWakeListenerController.pendingRequest != nil && !wakeDispatchInFlight)
                || (lastStagedWakeTriggeredVoiceTurnRequestState != nil && !wakeDispatchInFlight) {
                desktopWakeListenerController.haltCaptureSession()
                if !wakeDispatchInFlight {
                    desktopWakeListenerController.clearPendingPreparedWakeTurn()
                    lastStagedWakeTriggeredVoiceTurnRequestState = nil
                }
            }
            return
        }

        guard let activePromptStateID = desktopWakeListenerController.activePromptStateID else {
            return
        }

        guard activePromptStateID == promptState.id else {
            if !wakeDispatchInFlight {
                desktopWakeListenerController.haltCaptureSession()
                desktopWakeListenerController.clearPendingPreparedWakeTurn()
                lastStagedWakeTriggeredVoiceTurnRequestState = nil
            }
            return
        }
    }

    @MainActor
    private func synchronizeDesktopWakeAutoStartState() {
        guard scenePhase == .active,
              let promptState = desktopWakeListenerPromptState else {
            desktopWakeAutoStartAttemptedPromptID = nil
            desktopWakeAutoStartSuppressedPromptID = nil
            return
        }

        if desktopWakeAutoStartAttemptedPromptID != promptState.id {
            desktopWakeAutoStartAttemptedPromptID = nil
        }

        if desktopWakeAutoStartSuppressedPromptID != promptState.id {
            desktopWakeAutoStartSuppressedPromptID = nil
        }

        guard let eligiblePromptState = desktopWakeAutoStartEligiblePromptState,
              eligiblePromptState.id == promptState.id,
              desktopWakeAutoStartAttemptedPromptID != promptState.id,
              desktopWakeAutoStartSuppressedPromptID != promptState.id else {
            return
        }

        desktopWakeAutoStartAttemptedPromptID = promptState.id
        startDesktopWakeListener(promptState: eligiblePromptState)
    }

    @MainActor
    private func synchronizeDesktopWakeListenerLifecycleState() async {
        synchronizeDesktopWakeListenerControllerState()
        synchronizeDesktopWakeAutoStartState()
    }

    @MainActor
    private func submitDesktopWakeProfileAvailabilityRefresh(
        promptState: DesktopWakeProfileAvailabilityPromptState
    ) async {
        do {
            let ingressContext = try desktopCanonicalRuntimeBridge
                .desktopWakeProfileAvailabilityRequestBuilder(promptState)
            desktopWakeProfileAvailabilityRuntimeOutcomeState = .dispatching(
                receiptKind: ingressContext.receiptKind,
                deviceID: ingressContext.deviceID,
                wakeProfileID: ingressContext.wakeProfileID,
                voiceArtifactSyncReceiptRef: ingressContext.voiceArtifactSyncReceiptRef,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )

            let outcomeState = await desktopCanonicalRuntimeBridge
                .submitDesktopWakeProfileAvailabilityRefresh(ingressContext)
            desktopWakeProfileAvailabilityRuntimeOutcomeState = outcomeState
        } catch {
            desktopWakeProfileAvailabilityRuntimeOutcomeState = .failed(
                receiptKind: promptState.receiptKind,
                deviceID: promptState.deviceID,
                wakeProfileID: promptState.wakeProfileID,
                voiceArtifactSyncReceiptRef: promptState.voiceArtifactSyncReceiptRef,
                endpoint: desktopCanonicalRuntimeBridge.wakeProfileAvailabilityEndpoint,
                requestID: "unavailable",
                summary: "The canonical wake-profile availability bridge could not stage this bounded desktop wake-profile local-availability refresh request.",
                detail: error.localizedDescription
            )
        }
    }

    @MainActor
    private func submitDesktopSessionAttach(
        promptState: DesktopSessionAttachPromptState
    ) async {
        guard desktopSessionAttachPromptState?.id == promptState.id else {
            desktopSessionAttachRuntimeOutcomeState = .failed(
                sourceSurfaceIdentity: promptState.sourceSurfaceIdentity,
                sessionState: promptState.sessionState,
                sessionID: promptState.sessionID,
                currentVisibleSessionAttachOutcome: promptState.currentVisibleSessionAttachOutcome,
                turnID: promptState.turnID,
                deviceID: promptState.deviceID,
                endpoint: desktopCanonicalRuntimeBridge.sessionAttachEndpoint,
                requestID: "unavailable",
                summary: "The current-visible session-attach prompt disappeared before dispatch.",
                detail: "This shell fails closed when the exact lawful current-visible session-attach prompt no longer remains available before canonical dispatch can begin."
            )
            return
        }

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopSessionAttachRequestBuilder(
                promptState
            )
            desktopSessionAttachRuntimeOutcomeState = .dispatching(
                ingressContext: ingressContext
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.submitDesktopSessionAttach(
                ingressContext
            )
            desktopSessionAttachRuntimeOutcomeState = outcomeState
            synchronizeDesktopSelectedSessionProjectContextState()
        } catch {
            desktopSessionAttachRuntimeOutcomeState = .failed(
                sourceSurfaceIdentity: promptState.sourceSurfaceIdentity,
                sessionState: promptState.sessionState,
                sessionID: promptState.sessionID,
                currentVisibleSessionAttachOutcome: promptState.currentVisibleSessionAttachOutcome,
                turnID: promptState.turnID,
                deviceID: promptState.deviceID,
                endpoint: desktopCanonicalRuntimeBridge.sessionAttachEndpoint,
                requestID: "unavailable",
                summary: "The canonical session-attach bridge could not stage this bounded desktop current-visible session attach request.",
                detail: error.localizedDescription
            )
        }
    }

    @MainActor
    private func submitDesktopSessionMultiPostureResume(
        promptState: DesktopSessionMultiPostureResumePromptState
    ) async {
        guard desktopSessionMultiPostureResumePromptState?.id == promptState.id else {
            desktopSessionMultiPostureResumeRuntimeOutcomeState = .failed(
                resumeMode: promptState.resumeMode,
                sourceSurfaceIdentity: promptState.sourceSurfaceIdentity,
                sessionState: promptState.sessionState,
                sessionID: promptState.sessionID,
                selectedThreadID: promptState.selectedThreadID,
                selectedThreadTitle: promptState.selectedThreadTitle,
                pendingWorkOrderID: promptState.pendingWorkOrderID,
                resumeTier: promptState.resumeTier,
                resumeSummaryBullets: promptState.resumeSummaryBullets,
                recoveryMode: promptState.recoveryMode,
                deviceID: promptState.deviceID,
                endpoint: promptState.resumeMode == .softClosedExplicitResume
                    ? desktopCanonicalRuntimeBridge.sessionResumeEndpoint
                    : desktopCanonicalRuntimeBridge.sessionRecoverEndpoint,
                requestID: "unavailable",
                summary: "The multi-posture session-resume prompt disappeared before dispatch.",
                detail: "This shell fails closed when the exact lawful posture-specific session-resume prompt no longer remains uniquely available before canonical dispatch can begin."
            )
            return
        }

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopSessionMultiPostureResumeRequestBuilder(
                promptState
            )
            desktopSessionMultiPostureResumeRuntimeOutcomeState = .dispatching(
                ingressContext: ingressContext
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.submitDesktopSessionMultiPostureResume(
                ingressContext
            )
            desktopSessionMultiPostureResumeRuntimeOutcomeState = outcomeState
            synchronizeDesktopSelectedSessionProjectContextState()
        } catch {
            desktopSessionMultiPostureResumeRuntimeOutcomeState = .failed(
                resumeMode: promptState.resumeMode,
                sourceSurfaceIdentity: promptState.sourceSurfaceIdentity,
                sessionState: promptState.sessionState,
                sessionID: promptState.sessionID,
                selectedThreadID: promptState.selectedThreadID,
                selectedThreadTitle: promptState.selectedThreadTitle,
                pendingWorkOrderID: promptState.pendingWorkOrderID,
                resumeTier: promptState.resumeTier,
                resumeSummaryBullets: promptState.resumeSummaryBullets,
                recoveryMode: promptState.recoveryMode,
                deviceID: promptState.deviceID,
                endpoint: promptState.resumeMode == .softClosedExplicitResume
                    ? desktopCanonicalRuntimeBridge.sessionResumeEndpoint
                    : desktopCanonicalRuntimeBridge.sessionRecoverEndpoint,
                requestID: "unavailable",
                summary: "The canonical multi-posture session-resume bridge could not stage this bounded desktop session-resume request.",
                detail: error.localizedDescription
            )
        }
    }

    @MainActor
    private func submitDesktopSessionMultiPostureEntry(
        promptState: DesktopSessionMultiPostureEntryPromptState
    ) async {
        guard desktopSessionMultiPostureEntryPromptState?.id == promptState.id else {
            let endpoint: String
            switch promptState.entryMode {
            case .currentVisibleAttach:
                endpoint = desktopCanonicalRuntimeBridge.sessionAttachEndpoint
            case .softClosedExplicitResume:
                endpoint = desktopCanonicalRuntimeBridge.sessionResumeEndpoint
            case .suspendedAuthoritativeRereadRecover:
                endpoint = desktopCanonicalRuntimeBridge.sessionRecoverEndpoint
            }

            desktopSessionMultiPostureEntryRuntimeOutcomeState = .failed(
                entryMode: promptState.entryMode,
                sourceSurfaceIdentity: promptState.sourceSurfaceIdentity,
                sessionState: promptState.sessionState,
                sessionID: promptState.sessionID,
                currentVisibleSessionAttachOutcome: promptState.currentVisibleSessionAttachOutcome,
                turnID: promptState.turnID,
                selectedThreadID: promptState.selectedThreadID,
                selectedThreadTitle: promptState.selectedThreadTitle,
                pendingWorkOrderID: promptState.pendingWorkOrderID,
                resumeTier: promptState.resumeTier,
                resumeSummaryBullets: promptState.resumeSummaryBullets,
                recoveryMode: promptState.recoveryMode,
                deviceID: promptState.deviceID,
                endpoint: endpoint,
                requestID: "unavailable",
                summary: "The multi-posture session-entry prompt disappeared before dispatch.",
                detail: "This shell fails closed when the exact lawful route-specific session-entry prompt no longer remains uniquely available before canonical dispatch can begin."
            )
            return
        }

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopSessionMultiPostureEntryRequestBuilder(
                promptState
            )
            desktopSessionMultiPostureEntryRuntimeOutcomeState = .dispatching(
                ingressContext: ingressContext
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.submitDesktopSessionMultiPostureEntry(
                ingressContext
            )
            desktopSessionMultiPostureEntryRuntimeOutcomeState = outcomeState
            synchronizeDesktopSelectedSessionProjectContextState()
        } catch {
            let endpoint: String
            switch promptState.entryMode {
            case .currentVisibleAttach:
                endpoint = desktopCanonicalRuntimeBridge.sessionAttachEndpoint
            case .softClosedExplicitResume:
                endpoint = desktopCanonicalRuntimeBridge.sessionResumeEndpoint
            case .suspendedAuthoritativeRereadRecover:
                endpoint = desktopCanonicalRuntimeBridge.sessionRecoverEndpoint
            }

            desktopSessionMultiPostureEntryRuntimeOutcomeState = .failed(
                entryMode: promptState.entryMode,
                sourceSurfaceIdentity: promptState.sourceSurfaceIdentity,
                sessionState: promptState.sessionState,
                sessionID: promptState.sessionID,
                currentVisibleSessionAttachOutcome: promptState.currentVisibleSessionAttachOutcome,
                turnID: promptState.turnID,
                selectedThreadID: promptState.selectedThreadID,
                selectedThreadTitle: promptState.selectedThreadTitle,
                pendingWorkOrderID: promptState.pendingWorkOrderID,
                resumeTier: promptState.resumeTier,
                resumeSummaryBullets: promptState.resumeSummaryBullets,
                recoveryMode: promptState.recoveryMode,
                deviceID: promptState.deviceID,
                endpoint: endpoint,
                requestID: "unavailable",
                summary: "The canonical multi-posture session-entry bridge could not stage this bounded desktop session-entry request.",
                detail: error.localizedDescription
            )
        }
    }

    @MainActor
    private func submitDesktopEmoPersonaLock(
        promptState: DesktopEmoPersonaLockPromptState
    ) async {
        let activeEntryContextID = desktopOnboardingEntryContext?.id
        desktopAccessProvisionCommitRuntimeOutcomeState = nil
        desktopCompleteCommitRuntimeOutcomeState = nil
        desktopPairingCompletionCommitRuntimeOutcomeState = nil

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopEmoPersonaLockRequestBuilder(
                promptState
            )
            desktopEmoPersonaLockRuntimeOutcomeState = .dispatching(
                onboardingSessionID: ingressContext.onboardingSessionID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.submitDesktopEmoPersonaLock(
                ingressContext
            )
            guard desktopOnboardingEntryContext?.id == activeEntryContextID else {
                return
            }

            desktopEmoPersonaLockRuntimeOutcomeState = outcomeState
        } catch {
            desktopEmoPersonaLockRuntimeOutcomeState = .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                endpoint: desktopCanonicalRuntimeBridge.onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop emo/persona-lock request.",
                detail: error.localizedDescription
            )
        }
    }

    @MainActor
    private func submitDesktopAccessProvisionCommit(
        promptState: DesktopAccessProvisionCommitPromptState
    ) async {
        let activeEntryContextID = desktopOnboardingEntryContext?.id
        desktopCompleteCommitRuntimeOutcomeState = nil
        desktopPairingCompletionCommitRuntimeOutcomeState = nil

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopAccessProvisionCommitRequestBuilder(
                promptState
            )
            desktopAccessProvisionCommitRuntimeOutcomeState = .dispatching(
                onboardingSessionID: ingressContext.onboardingSessionID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.submitDesktopAccessProvisionCommit(
                ingressContext
            )
            guard desktopOnboardingEntryContext?.id == activeEntryContextID else {
                return
            }

            desktopAccessProvisionCommitRuntimeOutcomeState = outcomeState
        } catch {
            desktopAccessProvisionCommitRuntimeOutcomeState = .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                endpoint: desktopCanonicalRuntimeBridge.onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop access-provision request.",
                detail: error.localizedDescription
            )
        }
    }

    @MainActor
    private func submitDesktopCompleteCommit(
        promptState: DesktopCompleteCommitPromptState
    ) async {
        let activeEntryContextID = desktopOnboardingEntryContext?.id

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopCompleteCommitRequestBuilder(
                promptState
            )
            desktopCompleteCommitRuntimeOutcomeState = .dispatching(
                onboardingSessionID: ingressContext.onboardingSessionID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )

            let outcomeState = await desktopCanonicalRuntimeBridge.submitDesktopCompleteCommit(
                ingressContext
            )
            guard desktopOnboardingEntryContext?.id == activeEntryContextID else {
                return
            }

            desktopCompleteCommitRuntimeOutcomeState = outcomeState
        } catch {
            desktopCompleteCommitRuntimeOutcomeState = .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                endpoint: desktopCanonicalRuntimeBridge.onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop complete request.",
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
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopExplicitVoiceIngressRequestBuilder(
                pendingRequest,
                threadKey: desktopForegroundVoiceTurnMatchingSelectedThreadKey,
                authorityStatePolicyContextRef: desktopForegroundVoiceTurnActiveAuthorityPolicyContextRef,
                projectID: desktopForegroundVoiceTurnSelectedProjectID,
                pinnedContextRefs: desktopForegroundVoiceTurnSelectedPinnedContextRefs
            )
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

    @MainActor
    private func dispatchPreparedWakeTriggeredVoiceRequestIfNeeded() async {
        guard let pendingRequest = desktopWakeListenerController.pendingRequest else {
            return
        }

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge
                .desktopWakeTriggeredVoiceIngressRequestBuilder(
                    pendingRequest,
                    threadKey: desktopForegroundVoiceTurnMatchingSelectedThreadKey,
                    authorityStatePolicyContextRef: desktopForegroundVoiceTurnActiveAuthorityPolicyContextRef,
                    projectID: desktopForegroundVoiceTurnSelectedProjectID,
                    pinnedContextRefs: desktopForegroundVoiceTurnSelectedPinnedContextRefs
                )
            lastStagedWakeTriggeredVoiceTurnRequestState = pendingRequest
            desktopWakeListenerController.markDispatching()
            desktopCanonicalRuntimeOutcomeState = .dispatchingWake(
                preparedRequestID: ingressContext.preparedRequestID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )
            desktopAuthoritativeReplyRenderState = nil
            desktopAuthoritativeReplyProvenanceRenderState = nil
            desktopAuthoritativeReplyPlaybackController.reset()
            desktopAuthoritativeReplyPlaybackState = .idle

            let outcomeState = await desktopCanonicalRuntimeBridge
                .dispatchPreparedWakeTriggeredVoiceRequest(ingressContext)
            guard desktopWakeListenerController.pendingRequest?.id == pendingRequest.id else {
                return
            }

            desktopCanonicalRuntimeOutcomeState = outcomeState
            if outcomeState.phase == .completed {
                desktopAuthoritativeReplyRenderState = DesktopAuthoritativeReplyRenderState(
                    title: "Cloud-authored authoritative reply",
                    summary: outcomeState.authoritativeResponseText == nil
                        ? "The canonical runtime completed without reply text for this bounded wake-triggered voice turn."
                        : "Read-only canonical reply text from the completed wake-triggered runtime dispatch is now visible here while the shell remains explicitly non-authoritative.",
                    authoritativeResponseText: outcomeState.authoritativeResponseText
                )
                desktopAuthoritativeReplyProvenanceRenderState = DesktopAuthoritativeReplyProvenanceRenderState(
                    title: "Cloud-authored authoritative reply provenance",
                    summary: outcomeState.authoritativeResponseProvenance == nil
                        ? "The canonical runtime completed without provenance for this bounded wake-triggered voice turn."
                        : "Read-only canonical provenance from the completed wake-triggered runtime dispatch is now visible here while the shell remains explicitly non-authoritative.",
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
            desktopWakeListenerController.clearPendingPreparedWakeTurn()
        } catch {
            desktopCanonicalRuntimeOutcomeState = .failedWake(
                preparedRequestID: pendingRequest.id,
                endpoint: desktopCanonicalRuntimeBridge.voiceTurnEndpoint,
                requestID: "unavailable",
                summary: "The canonical runtime bridge could not stage the bounded wake-triggered voice request for dispatch.",
                detail: error.localizedDescription,
                reasonCode: "desktop_runtime_bridge_failure",
                failureClass: "RetryableRuntime"
            )
            desktopAuthoritativeReplyRenderState = nil
            desktopAuthoritativeReplyProvenanceRenderState = nil
            desktopAuthoritativeReplyPlaybackController.reset()
            desktopAuthoritativeReplyPlaybackState = .idle
            desktopWakeListenerController.clearPendingPreparedWakeTurn()
        }
    }

    @MainActor
    private func dispatchPreparedTypedTurnRequestIfNeeded() async {
        guard let pendingRequest = desktopTypedTurnPendingRequest else {
            return
        }

        do {
            let ingressContext = try desktopCanonicalRuntimeBridge.desktopTypedTurnIngressRequestBuilder(
                pendingRequest,
                threadKey: desktopForegroundVoiceTurnMatchingSelectedThreadKey,
                authorityStatePolicyContextRef: desktopForegroundVoiceTurnActiveAuthorityPolicyContextRef,
                projectID: desktopForegroundVoiceTurnSelectedProjectID,
                pinnedContextRefs: desktopForegroundVoiceTurnSelectedPinnedContextRefs
            )
            desktopCanonicalRuntimeOutcomeState = .dispatchingTyped(
                preparedRequestID: ingressContext.preparedRequestID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID
            )
            desktopAuthoritativeReplyRenderState = nil
            desktopAuthoritativeReplyProvenanceRenderState = nil
            desktopAuthoritativeReplyPlaybackController.reset()
            desktopAuthoritativeReplyPlaybackState = .idle

            let outcomeState = await desktopCanonicalRuntimeBridge.dispatchPreparedTypedTurnRequest(
                ingressContext
            )
            guard desktopTypedTurnPendingRequest?.id == pendingRequest.id else {
                return
            }

            desktopCanonicalRuntimeOutcomeState = outcomeState
            if outcomeState.phase == .completed {
                desktopAuthoritativeReplyRenderState = DesktopAuthoritativeReplyRenderState(
                    title: "Cloud-authored authoritative reply",
                    summary: outcomeState.authoritativeResponseText == nil
                        ? "The canonical runtime completed without reply text for this bounded typed turn."
                        : "Read-only canonical reply text from the completed typed-turn runtime dispatch is now visible here while the shell remains explicitly non-authoritative.",
                    authoritativeResponseText: outcomeState.authoritativeResponseText
                )
                desktopAuthoritativeReplyProvenanceRenderState = DesktopAuthoritativeReplyProvenanceRenderState(
                    title: "Cloud-authored authoritative reply provenance",
                    summary: outcomeState.authoritativeResponseProvenance == nil
                        ? "The canonical runtime completed without provenance for this bounded typed turn."
                        : "Read-only canonical provenance from the completed typed-turn runtime dispatch is now visible here while the shell remains explicitly non-authoritative.",
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
            desktopTypedTurnPendingRequest = nil
        } catch {
            desktopCanonicalRuntimeOutcomeState = .failedTyped(
                preparedRequestID: pendingRequest.id,
                endpoint: desktopCanonicalRuntimeBridge.voiceTurnEndpoint,
                requestID: "unavailable",
                summary: "The canonical runtime bridge could not stage the bounded typed-turn request for dispatch.",
                detail: error.localizedDescription,
                reasonCode: "desktop_runtime_bridge_failure",
                failureClass: "RetryableRuntime"
            )
            desktopAuthoritativeReplyRenderState = nil
            desktopAuthoritativeReplyProvenanceRenderState = nil
            desktopAuthoritativeReplyPlaybackController.reset()
            desktopAuthoritativeReplyPlaybackState = .idle
            desktopTypedTurnPendingRequest = nil
        }
    }

    private func stageDesktopTypedTurnRequest(
        trimmedDraft: String,
        origin: DesktopTypedTurnRequestOrigin
    ) -> DesktopTypedTurnSubmissionFailure? {
        guard !trimmedDraft.isEmpty else {
            return .emptyDraft
        }

        if trimmedDraft.utf8.count > maxDesktopTypedTurnBytes {
            return .byteLimit
        }

        guard desktopTypedTurnPendingRequest == nil else {
            return .pendingRequestActive
        }

        guard !explicitVoiceController.isListening,
              explicitVoiceController.pendingRequest == nil,
              !desktopWakeListenerController.listenerState.isActiveForMicrophone,
              desktopWakeListenerController.listenerState != .dispatching,
              desktopWakeListenerController.pendingRequest == nil,
              lastStagedWakeTriggeredVoiceTurnRequestState == nil else {
            return .otherForegroundRequestActive
        }

        desktopTypedTurnRequestSequence += 1
        desktopTypedTurnFailedRequest = nil
        desktopSearchRequestFailedRequest = nil
        desktopToolRequestFailedRequest = nil
        desktopCanonicalRuntimeOutcomeState = nil
        desktopAuthoritativeReplyRenderState = nil
        desktopAuthoritativeReplyProvenanceRenderState = nil
        desktopAuthoritativeReplyPlaybackController.reset()
        desktopAuthoritativeReplyPlaybackState = .idle
        desktopTypedTurnPendingRequest = DesktopTypedTurnRequestState(
            id: "\(origin.requestIDPrefix)_\(String(format: "%03d", desktopTypedTurnRequestSequence))",
            origin: origin,
            deviceTurnSequence: UInt64(desktopTypedTurnRequestSequence),
            text: trimmedDraft,
            byteCount: trimmedDraft.utf8.count
        )

        return nil
    }

    private func startDesktopWakeListener(promptState: DesktopWakeListenerPromptState) {
        desktopCanonicalRuntimeOutcomeState = nil
        desktopAuthoritativeReplyRenderState = nil
        desktopAuthoritativeReplyProvenanceRenderState = nil
        desktopAuthoritativeReplyPlaybackController.reset()
        desktopAuthoritativeReplyPlaybackState = .idle
        lastStagedWakeTriggeredVoiceTurnRequestState = nil
        desktopWakeListenerController.startListening(promptState: promptState)
    }

    private func stopDesktopWakeListenerAndSuppressAutoStart(
        promptState: DesktopWakeListenerPromptState?
    ) {
        if let promptState {
            desktopWakeAutoStartAttemptedPromptID = promptState.id
            desktopWakeAutoStartSuppressedPromptID = promptState.id
        }
        desktopWakeListenerController.stopListening()
    }

    private func submitDesktopTypedTurn() {
        let trimmedDraft = trimmedDesktopTypedTurnDraft

        switch stageDesktopTypedTurnRequest(
            trimmedDraft: trimmedDraft,
            origin: .keyboardComposer
        ) {
        case .none:
            desktopTypedTurnDraft = ""
        case .emptyDraft:
            return
        case .byteLimit:
            desktopTypedTurnFailedRequest = InterruptContinuityResponseFailureState(
                id: "failed_desktop_typed_turn_text_plain_validation",
                title: "Failed typed turn request",
                summary: "Canonical text-turn validation held this request because the bounded `text/plain` payload exceeded 16384 UTF-8 bytes before any authoritative acceptance occurred.",
                detail: "Failure visibility only; shorten the draft and retry through the canonical desktop typed-turn path. No local assistant output or authoritative transcript mutation was produced."
            )
        case .pendingRequestActive:
            desktopTypedTurnFailedRequest = InterruptContinuityResponseFailureState(
                id: "failed_desktop_typed_turn_awaiting_authoritative_response",
                title: "Failed typed turn request",
                summary: "A later typed-turn request could not be produced while the current bounded typed turn is already awaiting authoritative response.",
                detail: "The shell keeps bounded pending / failed posture only; it does not queue a second typed request locally, repair transport, or fabricate local assistant output."
            )
        case .otherForegroundRequestActive:
            desktopTypedTurnFailedRequest = InterruptContinuityResponseFailureState(
                id: "failed_desktop_typed_turn_other_voice_request_active",
                title: "Failed typed turn request",
                summary: "A bounded typed turn could not be produced while another foreground voice capture or voice-turn dispatch posture was still active.",
                detail: "This shell stays single-request only and does not merge typed and voice production locally, bypass canonical runtime sequencing, or invent local authority."
            )
        }
    }

    private func submitDesktopSearchRequest() {
        let trimmedDraft = trimmedDesktopSearchRequestDraft

        switch stageDesktopTypedTurnRequest(
            trimmedDraft: trimmedDraft,
            origin: .searchRequestCard
        ) {
        case .none:
            desktopSearchRequestDraft = ""
        case .emptyDraft:
            return
        case .byteLimit:
            desktopSearchRequestFailedRequest = InterruptContinuityResponseFailureState(
                id: "failed_desktop_search_request_text_plain_validation",
                title: "Failed search request",
                summary: "Canonical text-turn validation held this search-oriented request because the bounded `text/plain` payload exceeded 16384 UTF-8 bytes before any authoritative acceptance occurred.",
                detail: "Failure visibility only; shorten the request and retry through the bounded desktop search-request surface. Canonical runtime still retains search routing, provider choice, and retrieval authority, and no local search execution was produced."
            )
        case .pendingRequestActive:
            desktopSearchRequestFailedRequest = InterruptContinuityResponseFailureState(
                id: "failed_desktop_search_request_awaiting_authoritative_response",
                title: "Failed search request",
                summary: "A later search-oriented request could not be produced while the current bounded typed, search, or tool request is already awaiting authoritative response.",
                detail: "The shell keeps bounded pending / failed posture only; it does not queue a second request locally, bypass canonical runtime sequencing, or fabricate local search execution."
            )
        case .otherForegroundRequestActive:
            desktopSearchRequestFailedRequest = InterruptContinuityResponseFailureState(
                id: "failed_desktop_search_request_other_voice_request_active",
                title: "Failed search request",
                summary: "A bounded search-oriented request could not be produced while another foreground voice capture or voice-turn dispatch posture was still active.",
                detail: "This shell stays single-request only and does not merge search-request production with voice capture locally, bypass canonical runtime sequencing, or fabricate local search execution."
            )
        }
    }

    private func submitDesktopToolRequest() {
        let trimmedDraft = trimmedDesktopToolRequestDraft

        switch stageDesktopTypedTurnRequest(
            trimmedDraft: trimmedDraft,
            origin: .toolRequestCard
        ) {
        case .none:
            desktopToolRequestDraft = ""
        case .emptyDraft:
            return
        case .byteLimit:
            desktopToolRequestFailedRequest = InterruptContinuityResponseFailureState(
                id: "failed_desktop_tool_request_text_plain_validation",
                title: "Failed tool request",
                summary: "Canonical text-turn validation held this tool-oriented request because the bounded `text/plain` payload exceeded 16384 UTF-8 bytes before any authoritative acceptance occurred.",
                detail: "Failure visibility only; shorten the request and retry through the bounded desktop tool-request surface. Canonical runtime still retains tool-routing authority and no local tool execution was produced."
            )
        case .pendingRequestActive:
            desktopToolRequestFailedRequest = InterruptContinuityResponseFailureState(
                id: "failed_desktop_tool_request_awaiting_authoritative_response",
                title: "Failed tool request",
                summary: "A later tool-oriented request could not be produced while the current bounded typed or tool request is already awaiting authoritative response.",
                detail: "The shell keeps bounded pending / failed posture only; it does not queue a second request locally, bypass canonical runtime sequencing, or fabricate direct tool authority."
            )
        case .otherForegroundRequestActive:
            desktopToolRequestFailedRequest = InterruptContinuityResponseFailureState(
                id: "failed_desktop_tool_request_other_voice_request_active",
                title: "Failed tool request",
                summary: "A bounded tool-oriented request could not be produced while another foreground voice capture or voice-turn dispatch posture was still active.",
                detail: "This shell stays single-request only and does not merge tool-request production with voice capture locally, bypass canonical runtime sequencing, or fabricate direct tool authority."
            )
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
