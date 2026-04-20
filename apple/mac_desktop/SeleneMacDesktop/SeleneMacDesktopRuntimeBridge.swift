import Combine
import CryptoKit
import Foundation

struct AuthoritativeResponseProvenance: Equatable {
    struct Source: Identifiable, Equatable {
        let title: String
        let url: String

        var id: String {
            "\(title)|\(url)"
        }
    }

    let sources: [Source]
    let retrievedAt: UInt64?
    let cacheStatus: String?
}

struct DesktopCanonicalRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let outcome: String?
    let nextMove: String?
    let reasonCode: String?
    let sessionID: String?
    let turnID: String?
    let failureClass: String?
    let authoritativeResponseText: String?
    let authoritativeResponseProvenance: AuthoritativeResponseProvenance?

    static func dispatching(
        preparedRequestID: String,
        endpoint: String,
        requestID: String
    ) -> DesktopCanonicalRuntimeOutcomeState {
        DesktopCanonicalRuntimeOutcomeState(
            id: preparedRequestID,
            phase: .dispatching,
            title: "Dispatching prepared explicit voice request",
            summary: "The bounded explicit voice request is now being handed into the canonical runtime bridge.",
            detail: "Bridge dispatch only. This shell remains non-authoritative and does not fabricate local assistant output, reply text, or playback while canonical runtime execution is in flight.",
            endpoint: endpoint,
            requestID: requestID,
            outcome: nil,
            nextMove: nil,
            reasonCode: nil,
            sessionID: nil,
            turnID: nil,
            failureClass: nil,
            authoritativeResponseText: nil,
            authoritativeResponseProvenance: nil
        )
    }

    static func dispatchingWake(
        preparedRequestID: String,
        endpoint: String,
        requestID: String
    ) -> DesktopCanonicalRuntimeOutcomeState {
        DesktopCanonicalRuntimeOutcomeState(
            id: preparedRequestID,
            phase: .dispatching,
            title: "Dispatching prepared wake-triggered voice request",
            summary: "The bounded wake-triggered voice request is now being handed into the canonical runtime bridge.",
            detail: "Bridge dispatch only. This shell remains non-authoritative and does not fabricate local assistant output, reply text, playback, or hidden/background wake behavior while canonical runtime execution is in flight.",
            endpoint: endpoint,
            requestID: requestID,
            outcome: nil,
            nextMove: nil,
            reasonCode: nil,
            sessionID: nil,
            turnID: nil,
            failureClass: nil,
            authoritativeResponseText: nil,
            authoritativeResponseProvenance: nil
        )
    }

    static func completed(
        preparedRequestID: String,
        endpoint: String,
        requestID: String,
        response: DesktopCanonicalRuntimeBridge.VoiceTurnAdapterResponsePayload
    ) -> DesktopCanonicalRuntimeOutcomeState {
        DesktopCanonicalRuntimeOutcomeState(
            id: preparedRequestID,
            phase: .completed,
            title: "Canonical runtime dispatch completed",
            summary: "The bounded explicit voice request reached the canonical runtime and returned a cloud-authored outcome posture.",
            detail: "Outcome visibility plus bounded read-only reply and provenance rendering only. This bridge preserves cloud-authored reply text and provenance for shell-local display without mutating transcript preview surfaces or performing local playback.",
            endpoint: endpoint,
            requestID: requestID,
            outcome: response.outcome,
            nextMove: response.nextMove,
            reasonCode: response.reasonCode,
            sessionID: response.sessionID,
            turnID: response.turnID.map(String.init),
            failureClass: response.failureClass,
            authoritativeResponseText: boundedAuthoritativeResponseText(response.responseText),
            authoritativeResponseProvenance: boundedAuthoritativeResponseProvenance(response.provenance)
        )
    }

    static func completedWake(
        preparedRequestID: String,
        endpoint: String,
        requestID: String,
        response: DesktopCanonicalRuntimeBridge.VoiceTurnAdapterResponsePayload
    ) -> DesktopCanonicalRuntimeOutcomeState {
        DesktopCanonicalRuntimeOutcomeState(
            id: preparedRequestID,
            phase: .completed,
            title: "Canonical wake-triggered runtime dispatch completed",
            summary: "The bounded wake-triggered voice request reached the canonical runtime and returned a cloud-authored outcome posture.",
            detail: "Outcome visibility plus bounded read-only reply and provenance rendering only. This bridge preserves cloud-authored reply text and provenance for shell-local display without mutating wake transcript preview surfaces or performing local playback.",
            endpoint: endpoint,
            requestID: requestID,
            outcome: response.outcome,
            nextMove: response.nextMove,
            reasonCode: response.reasonCode,
            sessionID: response.sessionID,
            turnID: response.turnID.map(String.init),
            failureClass: response.failureClass,
            authoritativeResponseText: boundedAuthoritativeResponseText(response.responseText),
            authoritativeResponseProvenance: boundedAuthoritativeResponseProvenance(response.provenance)
        )
    }

    static func failed(
        preparedRequestID: String,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reasonCode: String? = nil,
        failureClass: String? = nil,
        sessionID: String? = nil,
        turnID: String? = nil
    ) -> DesktopCanonicalRuntimeOutcomeState {
        DesktopCanonicalRuntimeOutcomeState(
            id: preparedRequestID,
            phase: .failed,
            title: "Canonical runtime dispatch failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            outcome: nil,
            nextMove: nil,
            reasonCode: reasonCode,
            sessionID: sessionID,
            turnID: turnID,
            failureClass: failureClass,
            authoritativeResponseText: nil,
            authoritativeResponseProvenance: nil
        )
    }

    static func failedWake(
        preparedRequestID: String,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reasonCode: String? = nil,
        failureClass: String? = nil,
        sessionID: String? = nil,
        turnID: String? = nil
    ) -> DesktopCanonicalRuntimeOutcomeState {
        DesktopCanonicalRuntimeOutcomeState(
            id: preparedRequestID,
            phase: .failed,
            title: "Canonical wake-triggered runtime dispatch failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            outcome: nil,
            nextMove: nil,
            reasonCode: reasonCode,
            sessionID: sessionID,
            turnID: turnID,
            failureClass: failureClass,
            authoritativeResponseText: nil,
            authoritativeResponseProvenance: nil
        )
    }
}

struct DesktopInviteOpenRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let outcome: String?
    let reason: String?
    let onboardingSessionID: String?
    let nextStep: String?
    let requiredFields: [String]
    let requiredVerificationGates: [String]

    static func dispatching(
        entryContextID: String,
        endpoint: String,
        requestID: String
    ) -> DesktopInviteOpenRuntimeOutcomeState {
        DesktopInviteOpenRuntimeOutcomeState(
            id: entryContextID,
            phase: .dispatching,
            title: "Dispatching invite-open onboarding entry",
            summary: "The bounded invite-open context is now being handed into canonical `/v1/invite/click` onboarding-start routing.",
            detail: "Bridge dispatch only. This shell remains non-authoritative, read-only after onboarding start, and does not widen into onboarding-continue mutation, access provisioning, pairing completion, wake behavior, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            outcome: nil,
            reason: nil,
            onboardingSessionID: nil,
            nextStep: nil,
            requiredFields: [],
            requiredVerificationGates: []
        )
    }

    static func completed(
        entryContextID: String,
        endpoint: String,
        requestID: String,
        outcome: String,
        reason: String?,
        onboardingSessionID: String?,
        nextStep: String?,
        requiredFields: [String],
        requiredVerificationGates: [String]
    ) -> DesktopInviteOpenRuntimeOutcomeState {
        DesktopInviteOpenRuntimeOutcomeState(
            id: entryContextID,
            phase: .completed,
            title: "Invite-open onboarding entry completed",
            summary: "Canonical `/v1/invite/click` routing returned a bounded onboarding-start outcome posture for this invite-open context.",
            detail: "Read-only onboarding-start visibility only. This shell preserves returned session and next-step posture without exposing onboarding-continue mutation, receipt submission, wake controls, or local onboarding authority.",
            endpoint: endpoint,
            requestID: requestID,
            outcome: outcome,
            reason: reason,
            onboardingSessionID: onboardingSessionID,
            nextStep: nextStep,
            requiredFields: requiredFields,
            requiredVerificationGates: requiredVerificationGates
        )
    }

    static func failed(
        entryContextID: String,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopInviteOpenRuntimeOutcomeState {
        DesktopInviteOpenRuntimeOutcomeState(
            id: entryContextID,
            phase: .failed,
            title: "Invite-open onboarding entry failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            outcome: nil,
            reason: reason,
            onboardingSessionID: nil,
            nextStep: nil,
            requiredFields: [],
            requiredVerificationGates: []
        )
    }
}

struct DesktopOnboardingContinueRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let outcome: String?
    let reason: String?
    let onboardingSessionID: String?
    let nextStep: String?
    let blockingField: String?
    let blockingQuestion: String?
    let remainingMissingFields: [String]
    let remainingPlatformReceiptKinds: [String]
    let onboardingStatus: String?

    static func dispatching(
        onboardingSessionID: String,
        blockingField: String,
        endpoint: String,
        requestID: String,
        submittedFieldValue: String?
    ) -> DesktopOnboardingContinueRuntimeOutcomeState {
        DesktopOnboardingContinueRuntimeOutcomeState(
            id: requestID,
            phase: .dispatching,
            title: "Dispatching onboarding continue missing-field request",
            summary: submittedFieldValue == nil
                ? "The bounded missing-field prompt request is now being handed into canonical `/v1/onboarding/continue`."
                : "The bounded missing-field submission is now being handed into canonical `/v1/onboarding/continue`.",
            detail: "Only the exact `ASK_MISSING_SUBMIT` loop is in scope here. This shell remains explicitly non-authoritative and does not expose platform-receipt submission, terms acceptance, primary-device confirmation, voice enrollment, access provisioning, wake controls, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            outcome: nil,
            reason: nil,
            onboardingSessionID: onboardingSessionID,
            nextStep: "ASK_MISSING",
            blockingField: blockingField,
            blockingQuestion: nil,
            remainingMissingFields: [blockingField],
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil
        )
    }

    static func completed(
        requestID: String,
        endpoint: String,
        response: DesktopCanonicalRuntimeBridge.OnboardingContinueAdapterResponsePayload,
        fallbackOnboardingSessionID: String,
        fallbackBlockingField: String
    ) -> DesktopOnboardingContinueRuntimeOutcomeState {
        let boundedNextStep = boundedOnboardingContinueField(response.nextStep)
        return DesktopOnboardingContinueRuntimeOutcomeState(
            id: requestID,
            phase: .completed,
            title: "Onboarding continue missing-field request completed",
            summary: boundedNextStep == "ASK_MISSING"
                ? "Canonical `/v1/onboarding/continue` returned the next bounded missing-field prompt posture for this onboarding session."
                : "Canonical `/v1/onboarding/continue` advanced beyond `ASK_MISSING`; later onboarding actions remain read-only and out of scope in this shell.",
            detail: boundedNextStep == "ASK_MISSING"
                ? "Prompt-and-submit visibility only. This shell exposes the returned blocking field, blocking question, and remaining missing fields without introducing local onboarding authority."
                : "Read-only next-step visibility only. This shell preserves the advanced step and any returned `remaining_platform_receipt_kinds` without adding controls for receipts, terms, device confirmation, voice enrollment, access provisioning, wake behavior, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            outcome: boundedOnboardingContinueField(response.outcome) ?? "ONBOARDING_CONTINUED",
            reason: boundedOnboardingContinueField(response.reason),
            onboardingSessionID: boundedOnboardingContinueField(response.onboardingSessionID) ?? fallbackOnboardingSessionID,
            nextStep: boundedNextStep,
            blockingField: boundedOnboardingContinueField(response.blockingField) ?? fallbackBlockingField,
            blockingQuestion: boundedOnboardingContinueField(response.blockingQuestion),
            remainingMissingFields: boundedOnboardingContinueList(response.remainingMissingFields),
            remainingPlatformReceiptKinds: boundedOnboardingContinueList(response.remainingPlatformReceiptKinds),
            onboardingStatus: boundedOnboardingContinueField(response.onboardingStatus)
        )
    }

    static func failed(
        onboardingSessionID: String,
        blockingField: String?,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopOnboardingContinueRuntimeOutcomeState {
        DesktopOnboardingContinueRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Onboarding continue missing-field request failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            outcome: nil,
            reason: reason,
            onboardingSessionID: onboardingSessionID,
            nextStep: nil,
            blockingField: blockingField,
            blockingQuestion: nil,
            remainingMissingFields: [],
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil
        )
    }
}

struct DesktopPlatformSetupReceiptRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let receiptKind: String
    let outcome: String?
    let reason: String?
    let onboardingSessionID: String?
    let nextStep: String?
    let remainingPlatformReceiptKinds: [String]
    let onboardingStatus: String?

    static func dispatching(
        onboardingSessionID: String,
        receiptKind: String,
        endpoint: String,
        requestID: String
    ) -> DesktopPlatformSetupReceiptRuntimeOutcomeState {
        DesktopPlatformSetupReceiptRuntimeOutcomeState(
            id: requestID,
            phase: .dispatching,
            title: "Dispatching desktop platform-setup receipt",
            summary: "The bounded desktop platform-setup receipt is now being handed into canonical `/v1/onboarding/continue`.",
            detail: "Only exact locally provable desktop receipt submission is in scope here. This shell remains explicitly non-authoritative and does not introduce later onboarding actions, wake controls, pairing completion, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            receiptKind: receiptKind,
            outcome: nil,
            reason: nil,
            onboardingSessionID: onboardingSessionID,
            nextStep: "PLATFORM_SETUP",
            remainingPlatformReceiptKinds: [receiptKind],
            onboardingStatus: nil
        )
    }

    static func completed(
        requestID: String,
        endpoint: String,
        response: DesktopCanonicalRuntimeBridge.OnboardingContinueAdapterResponsePayload,
        fallbackOnboardingSessionID: String,
        fallbackReceiptKind: String
    ) -> DesktopPlatformSetupReceiptRuntimeOutcomeState {
        let boundedNextStep = boundedOnboardingContinueField(response.nextStep)
        let boundedRemainingPlatformReceiptKinds = boundedOnboardingContinueList(response.remainingPlatformReceiptKinds)
        let advancedBeyondPlatformSetup = boundedNextStep != nil
            && boundedNextStep != "PLATFORM_SETUP"
            && boundedRemainingPlatformReceiptKinds.isEmpty

        return DesktopPlatformSetupReceiptRuntimeOutcomeState(
            id: requestID,
            phase: .completed,
            title: "Desktop platform-setup receipt completed",
            summary: advancedBeyondPlatformSetup
                ? "Canonical `/v1/onboarding/continue` advanced beyond `PLATFORM_SETUP`; later onboarding actions remain read-only and out of scope in this shell."
                : "Canonical `/v1/onboarding/continue` accepted the bounded desktop platform-setup receipt and returned updated remaining receipt posture.",
            detail: advancedBeyondPlatformSetup
                ? "Read-only next-step visibility only. This shell preserves the advanced step without adding controls for later onboarding mutation, pairing completion, wake behavior, or autonomous unlock."
                : "Only exact locally provable desktop receipt submission is in scope here. Unsupported remaining receipt kinds stay read-only and unsubmitted in this shell.",
            endpoint: endpoint,
            requestID: requestID,
            receiptKind: fallbackReceiptKind,
            outcome: boundedOnboardingContinueField(response.outcome) ?? "ONBOARDING_CONTINUED",
            reason: boundedOnboardingContinueField(response.reason),
            onboardingSessionID: boundedOnboardingContinueField(response.onboardingSessionID) ?? fallbackOnboardingSessionID,
            nextStep: boundedNextStep,
            remainingPlatformReceiptKinds: boundedRemainingPlatformReceiptKinds,
            onboardingStatus: boundedOnboardingContinueField(response.onboardingStatus)
        )
    }

    static func failed(
        onboardingSessionID: String,
        receiptKind: String,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopPlatformSetupReceiptRuntimeOutcomeState {
        DesktopPlatformSetupReceiptRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Desktop platform-setup receipt failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            receiptKind: receiptKind,
            outcome: nil,
            reason: reason,
            onboardingSessionID: onboardingSessionID,
            nextStep: nil,
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil
        )
    }
}

struct DesktopTermsAcceptRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let termsVersionID: String
    let accepted: Bool
    let outcome: String?
    let reason: String?
    let onboardingSessionID: String?
    let nextStep: String?
    let remainingPlatformReceiptKinds: [String]
    let onboardingStatus: String?

    static func dispatching(
        onboardingSessionID: String,
        termsVersionID: String,
        endpoint: String,
        requestID: String
    ) -> DesktopTermsAcceptRuntimeOutcomeState {
        DesktopTermsAcceptRuntimeOutcomeState(
            id: requestID,
            phase: .dispatching,
            title: "Dispatching desktop terms acceptance",
            summary: "The bounded desktop terms acceptance request is now being handed into canonical `/v1/onboarding/continue`.",
            detail: "Only exact `TERMS_ACCEPT` is in scope here. This shell remains explicitly non-authoritative and does not introduce local terms prose, sender verification, primary-device confirmation, voice enrollment, access provisioning, pairing completion, wake behavior, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            termsVersionID: termsVersionID,
            accepted: true,
            outcome: nil,
            reason: nil,
            onboardingSessionID: onboardingSessionID,
            nextStep: "TERMS",
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil
        )
    }

    static func completed(
        requestID: String,
        endpoint: String,
        response: DesktopCanonicalRuntimeBridge.OnboardingContinueAdapterResponsePayload,
        fallbackOnboardingSessionID: String,
        fallbackTermsVersionID: String
    ) -> DesktopTermsAcceptRuntimeOutcomeState {
        let boundedNextStep = boundedOnboardingContinueField(response.nextStep)
        let advancedBeyondTerms = boundedNextStep != nil && boundedNextStep != "TERMS"

        return DesktopTermsAcceptRuntimeOutcomeState(
            id: requestID,
            phase: .completed,
            title: "Desktop terms acceptance completed",
            summary: advancedBeyondTerms
                ? "Canonical `/v1/onboarding/continue` advanced beyond `TERMS`; later onboarding actions remain read-only and out of scope in this shell."
                : "Canonical `/v1/onboarding/continue` accepted the bounded desktop terms submission and returned updated onboarding posture.",
            detail: advancedBeyondTerms
                ? "Read-only next-step visibility only. This shell preserves the advanced step and onboarding status without adding controls for sender verification, primary-device confirmation, voice enrollment, access provisioning, pairing completion, wake behavior, or autonomous unlock."
                : "Canonical terms acceptance only. This shell does not fabricate a local terms document, local policy summary, or local onboarding authority.",
            endpoint: endpoint,
            requestID: requestID,
            termsVersionID: fallbackTermsVersionID,
            accepted: true,
            outcome: boundedOnboardingContinueField(response.outcome) ?? "ONBOARDING_CONTINUED",
            reason: boundedOnboardingContinueField(response.reason),
            onboardingSessionID: boundedOnboardingContinueField(response.onboardingSessionID) ?? fallbackOnboardingSessionID,
            nextStep: boundedNextStep,
            remainingPlatformReceiptKinds: boundedOnboardingContinueList(response.remainingPlatformReceiptKinds),
            onboardingStatus: boundedOnboardingContinueField(response.onboardingStatus)
        )
    }

    static func failed(
        onboardingSessionID: String,
        termsVersionID: String,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopTermsAcceptRuntimeOutcomeState {
        DesktopTermsAcceptRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Desktop terms acceptance failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            termsVersionID: termsVersionID,
            accepted: true,
            outcome: nil,
            reason: reason,
            onboardingSessionID: onboardingSessionID,
            nextStep: nil,
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil
        )
    }
}

struct DesktopPrimaryDeviceConfirmRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let deviceID: String
    let proofOK: Bool
    let outcome: String?
    let reason: String?
    let onboardingSessionID: String?
    let nextStep: String?
    let remainingPlatformReceiptKinds: [String]
    let onboardingStatus: String?

    static func dispatching(
        onboardingSessionID: String,
        deviceID: String,
        endpoint: String,
        requestID: String
    ) -> DesktopPrimaryDeviceConfirmRuntimeOutcomeState {
        DesktopPrimaryDeviceConfirmRuntimeOutcomeState(
            id: requestID,
            phase: .dispatching,
            title: "Dispatching desktop primary-device confirmation",
            summary: "The bounded desktop primary-device confirmation request is now being handed into canonical `/v1/onboarding/continue`.",
            detail: "Only exact `PRIMARY_DEVICE_CONFIRM` is in scope here. This shell remains explicitly non-authoritative and does not introduce sender verification, employee photo capture, voice enrollment, access provisioning, pairing completion, wake behavior, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            deviceID: deviceID,
            proofOK: true,
            outcome: nil,
            reason: nil,
            onboardingSessionID: onboardingSessionID,
            nextStep: "PRIMARY_DEVICE_CONFIRM",
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil
        )
    }

    static func completed(
        requestID: String,
        endpoint: String,
        response: DesktopCanonicalRuntimeBridge.OnboardingContinueAdapterResponsePayload,
        fallbackOnboardingSessionID: String,
        fallbackDeviceID: String
    ) -> DesktopPrimaryDeviceConfirmRuntimeOutcomeState {
        let boundedNextStep = boundedOnboardingContinueField(response.nextStep)
        let advancedBeyondPrimaryDevice = boundedNextStep != nil && boundedNextStep != "PRIMARY_DEVICE_CONFIRM"

        return DesktopPrimaryDeviceConfirmRuntimeOutcomeState(
            id: requestID,
            phase: .completed,
            title: "Desktop primary-device confirmation completed",
            summary: advancedBeyondPrimaryDevice
                ? "Canonical `/v1/onboarding/continue` advanced beyond `PRIMARY_DEVICE_CONFIRM`; later onboarding actions remain read-only and out of scope in this shell."
                : "Canonical `/v1/onboarding/continue` accepted the bounded desktop primary-device confirmation and returned updated onboarding posture.",
            detail: advancedBeyondPrimaryDevice
                ? "Read-only next-step visibility only. This shell preserves the advanced step and onboarding status without adding sender verification, employee photo capture, voice enrollment, access provisioning, pairing completion, wake behavior, or autonomous unlock."
                : "Canonical primary-device confirmation only. This shell does not introduce local onboarding authority or any later onboarding controls.",
            endpoint: endpoint,
            requestID: requestID,
            deviceID: fallbackDeviceID,
            proofOK: true,
            outcome: boundedOnboardingContinueField(response.outcome) ?? "ONBOARDING_CONTINUED",
            reason: boundedOnboardingContinueField(response.reason),
            onboardingSessionID: boundedOnboardingContinueField(response.onboardingSessionID) ?? fallbackOnboardingSessionID,
            nextStep: boundedNextStep,
            remainingPlatformReceiptKinds: boundedOnboardingContinueList(response.remainingPlatformReceiptKinds),
            onboardingStatus: boundedOnboardingContinueField(response.onboardingStatus)
        )
    }

    static func failed(
        onboardingSessionID: String,
        deviceID: String,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopPrimaryDeviceConfirmRuntimeOutcomeState {
        DesktopPrimaryDeviceConfirmRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Desktop primary-device confirmation failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            deviceID: deviceID,
            proofOK: true,
            outcome: nil,
            reason: reason,
            onboardingSessionID: onboardingSessionID,
            nextStep: nil,
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil
        )
    }
}

struct DesktopEmployeePhotoCaptureSendRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let photoBlobRef: String?
    let outcome: String?
    let reason: String?
    let onboardingSessionID: String?
    let nextStep: String?
    let remainingPlatformReceiptKinds: [String]
    let onboardingStatus: String?

    static func dispatching(
        onboardingSessionID: String,
        photoBlobRef: String,
        endpoint: String,
        requestID: String
    ) -> DesktopEmployeePhotoCaptureSendRuntimeOutcomeState {
        DesktopEmployeePhotoCaptureSendRuntimeOutcomeState(
            id: requestID,
            phase: .dispatching,
            title: "Dispatching employee photo capture send",
            summary: "The bounded employee photo capture send request is now being handed into canonical `/v1/onboarding/continue`.",
            detail: "Only exact `EMPLOYEE_PHOTO_CAPTURE_SEND` with an already-existing exact `photo_blob_ref` is in scope here. This shell remains explicitly non-authoritative and does not introduce local photo picker, local capture, local upload, sender-decision mutation, pairing completion, wake behavior, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            photoBlobRef: photoBlobRef,
            outcome: nil,
            reason: nil,
            onboardingSessionID: onboardingSessionID,
            nextStep: "SENDER_VERIFICATION",
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil
        )
    }

    static func completed(
        requestID: String,
        endpoint: String,
        response: DesktopCanonicalRuntimeBridge.OnboardingContinueAdapterResponsePayload,
        fallbackOnboardingSessionID: String,
        fallbackPhotoBlobRef: String
    ) -> DesktopEmployeePhotoCaptureSendRuntimeOutcomeState {
        let boundedNextStep = boundedOnboardingContinueField(response.nextStep)
        let advancedBeyondSenderVerification = boundedNextStep != nil
            && boundedNextStep != "SENDER_VERIFICATION"

        return DesktopEmployeePhotoCaptureSendRuntimeOutcomeState(
            id: requestID,
            phase: .completed,
            title: "Employee photo capture send completed",
            summary: advancedBeyondSenderVerification
                ? "Canonical `/v1/onboarding/continue` advanced beyond `SENDER_VERIFICATION`; later onboarding actions remain read-only and out of scope in this shell."
                : "Canonical `/v1/onboarding/continue` accepted the bounded employee photo capture send and returned updated sender-verification posture.",
            detail: advancedBeyondSenderVerification
                ? "Read-only next-step visibility only. This shell preserves the advanced step and onboarding status without adding local photo authority, sender-decision mutation, primary-device bypass, pairing completion, wake behavior, or autonomous unlock."
                : "Canonical employee photo capture send only. This shell preserves exact `photo_blob_ref` dispatch posture without adding local picker, local capture, local upload, pasteboard blob authority, or sender-decision controls.",
            endpoint: endpoint,
            requestID: requestID,
            photoBlobRef: fallbackPhotoBlobRef,
            outcome: boundedOnboardingContinueField(response.outcome) ?? "ONBOARDING_CONTINUED",
            reason: boundedOnboardingContinueField(response.reason),
            onboardingSessionID: boundedOnboardingContinueField(response.onboardingSessionID) ?? fallbackOnboardingSessionID,
            nextStep: boundedNextStep,
            remainingPlatformReceiptKinds: boundedOnboardingContinueList(response.remainingPlatformReceiptKinds),
            onboardingStatus: boundedOnboardingContinueField(response.onboardingStatus)
        )
    }

    static func failed(
        onboardingSessionID: String,
        photoBlobRef: String?,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopEmployeePhotoCaptureSendRuntimeOutcomeState {
        DesktopEmployeePhotoCaptureSendRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Employee photo capture send failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            photoBlobRef: photoBlobRef,
            outcome: nil,
            reason: reason,
            onboardingSessionID: onboardingSessionID,
            nextStep: nil,
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil
        )
    }
}

struct DesktopEmployeeSenderVerifyCommitRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let senderDecision: String?
    let outcome: String?
    let reason: String?
    let onboardingSessionID: String?
    let nextStep: String?
    let remainingPlatformReceiptKinds: [String]
    let onboardingStatus: String?

    static func dispatching(
        onboardingSessionID: String,
        senderDecision: String,
        endpoint: String,
        requestID: String
    ) -> DesktopEmployeeSenderVerifyCommitRuntimeOutcomeState {
        DesktopEmployeeSenderVerifyCommitRuntimeOutcomeState(
            id: requestID,
            phase: .dispatching,
            title: "Dispatching sender verification commit",
            summary: "The bounded sender verification commit request is now being handed into canonical `/v1/onboarding/continue`.",
            detail: "Only exact `EMPLOYEE_SENDER_VERIFY_COMMIT` with exact `sender_decision` is in scope here. This shell remains explicitly non-authoritative and does not introduce local photo picker, local capture, local upload, broader sender workflow recovery, pairing completion, wake behavior, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            senderDecision: senderDecision,
            outcome: nil,
            reason: nil,
            onboardingSessionID: onboardingSessionID,
            nextStep: "SENDER_VERIFICATION",
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil
        )
    }

    static func completed(
        requestID: String,
        endpoint: String,
        response: DesktopCanonicalRuntimeBridge.OnboardingContinueAdapterResponsePayload,
        fallbackOnboardingSessionID: String,
        fallbackSenderDecision: String
    ) -> DesktopEmployeeSenderVerifyCommitRuntimeOutcomeState {
        let boundedNextStep = boundedOnboardingContinueField(response.nextStep)
        let advancedBeyondSenderVerification = boundedNextStep != nil
            && boundedNextStep != "SENDER_VERIFICATION"

        return DesktopEmployeeSenderVerifyCommitRuntimeOutcomeState(
            id: requestID,
            phase: .completed,
            title: "Sender verification commit completed",
            summary: boundedNextStep == "PRIMARY_DEVICE_CONFIRM"
                ? "Canonical `/v1/onboarding/continue` advanced to exact `PRIMARY_DEVICE_CONFIRM`; the already-landed primary-device confirmation submit remains separately gated in this shell."
                : boundedNextStep == "BLOCKED"
                    ? "Canonical `/v1/onboarding/continue` fail-closed to exact `BLOCKED`; this shell preserves that returned posture in read-only form only."
                    : advancedBeyondSenderVerification
                        ? "Canonical `/v1/onboarding/continue` advanced beyond `SENDER_VERIFICATION`; later onboarding actions remain read-only and out of scope in this shell."
                        : "Canonical `/v1/onboarding/continue` accepted the bounded sender verification commit and returned updated onboarding posture.",
            detail: boundedNextStep == "PRIMARY_DEVICE_CONFIRM"
                ? "Read-only next-step visibility only. This exact sender verification commit surface preserves returned exact `PRIMARY_DEVICE_CONFIRM` and onboarding status while the already-landed primary-device-confirm submit remains separately gated and non-authoritative."
                : boundedNextStep == "BLOCKED"
                    ? "Read-only blocked visibility only. This exact sender verification commit surface preserves returned exact `BLOCKED` and onboarding status without adding broader sender-verification recovery, local authority, pairing completion, wake behavior, or autonomous unlock."
                    : boundedNextStep == "SENDER_VERIFICATION"
                        ? "Read-only next-step visibility only. This exact sender verification commit surface preserves returned exact `SENDER_VERIFICATION` and onboarding status; broader sender-verification recovery does not widen here."
                        : advancedBeyondSenderVerification
                            ? "Read-only next-step visibility only. This shell preserves the advanced step and onboarding status without adding local photo authority, broader sender workflow mutation, pairing completion, wake behavior, or autonomous unlock."
                            : "Canonical sender verification commit only. This shell preserves returned onboarding posture without local onboarding authority or later onboarding controls.",
            endpoint: endpoint,
            requestID: requestID,
            senderDecision: fallbackSenderDecision,
            outcome: boundedOnboardingContinueField(response.outcome) ?? "ONBOARDING_CONTINUED",
            reason: boundedOnboardingContinueField(response.reason),
            onboardingSessionID: boundedOnboardingContinueField(response.onboardingSessionID) ?? fallbackOnboardingSessionID,
            nextStep: boundedNextStep,
            remainingPlatformReceiptKinds: boundedOnboardingContinueList(response.remainingPlatformReceiptKinds),
            onboardingStatus: boundedOnboardingContinueField(response.onboardingStatus)
        )
    }

    static func failed(
        onboardingSessionID: String,
        senderDecision: String?,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopEmployeeSenderVerifyCommitRuntimeOutcomeState {
        DesktopEmployeeSenderVerifyCommitRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Sender verification commit failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            senderDecision: senderDecision,
            outcome: nil,
            reason: reason,
            onboardingSessionID: onboardingSessionID,
            nextStep: nil,
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil
        )
    }
}

struct DesktopVoiceEnrollRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let deviceID: String
    let sampleSeed: String?
    let outcome: String?
    let reason: String?
    let onboardingSessionID: String?
    let nextStep: String?
    let remainingPlatformReceiptKinds: [String]
    let onboardingStatus: String?
    let voiceArtifactSyncReceiptRef: String?

    static func dispatching(
        onboardingSessionID: String,
        deviceID: String,
        sampleSeed: String,
        endpoint: String,
        requestID: String
    ) -> DesktopVoiceEnrollRuntimeOutcomeState {
        DesktopVoiceEnrollRuntimeOutcomeState(
            id: requestID,
            phase: .dispatching,
            title: "Dispatching desktop voice-enroll lock",
            summary: "The bounded desktop voice-enroll lock request is now being handed into canonical `/v1/onboarding/continue`.",
            detail: "Only exact `VOICE_ENROLL_LOCK` is in scope here. This shell remains explicitly non-authoritative and does not introduce sender verification, employee photo capture, wake-enrollment actions, emo-persona lock, access provisioning, pairing completion, wake-listener behavior, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            deviceID: deviceID,
            sampleSeed: sampleSeed,
            outcome: nil,
            reason: nil,
            onboardingSessionID: onboardingSessionID,
            nextStep: "VOICE_ENROLL",
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil,
            voiceArtifactSyncReceiptRef: nil
        )
    }

    static func completed(
        requestID: String,
        endpoint: String,
        response: DesktopCanonicalRuntimeBridge.OnboardingContinueAdapterResponsePayload,
        fallbackOnboardingSessionID: String,
        fallbackDeviceID: String,
        fallbackSampleSeed: String
    ) -> DesktopVoiceEnrollRuntimeOutcomeState {
        let boundedNextStep = boundedOnboardingContinueField(response.nextStep)
        let advancedBeyondVoiceEnroll = boundedNextStep != nil && boundedNextStep != "VOICE_ENROLL"
        let returnedVoiceArtifactSyncReceiptRef = boundedOnboardingContinueField(response.voiceArtifactSyncReceiptRef)

        return DesktopVoiceEnrollRuntimeOutcomeState(
            id: requestID,
            phase: .completed,
            title: "Desktop voice-enroll lock completed",
            summary: advancedBeyondVoiceEnroll
                ? "Canonical `/v1/onboarding/continue` advanced beyond `VOICE_ENROLL`; later onboarding actions remain read-only and out of scope in this shell."
                : "Canonical `/v1/onboarding/continue` accepted the bounded desktop voice-enroll lock and returned updated onboarding posture.",
            detail: boundedNextStep == "WAKE_ENROLL"
                ? "Read-only next-step visibility only. This exact voice-enroll surface preserves returned `voice_artifact_sync_receipt_ref` plus exact `WAKE_ENROLL` posture while any later bounded wake-enrollment actions remain separately gated and non-authoritative."
                : advancedBeyondVoiceEnroll
                    ? "Read-only next-step visibility only. This shell preserves the advanced step and any returned voice-artifact sync receipt without adding sender verification, wake mutation, emo-persona lock, access provisioning, pairing completion, or autonomous unlock."
                    : "Canonical voice-enroll lock only. This shell preserves returned onboarding posture without adding local voice authority, wake mutation, or later onboarding controls.",
            endpoint: endpoint,
            requestID: requestID,
            deviceID: fallbackDeviceID,
            sampleSeed: fallbackSampleSeed,
            outcome: boundedOnboardingContinueField(response.outcome) ?? "ONBOARDING_CONTINUED",
            reason: boundedOnboardingContinueField(response.reason),
            onboardingSessionID: boundedOnboardingContinueField(response.onboardingSessionID) ?? fallbackOnboardingSessionID,
            nextStep: boundedNextStep,
            remainingPlatformReceiptKinds: boundedOnboardingContinueList(response.remainingPlatformReceiptKinds),
            onboardingStatus: boundedOnboardingContinueField(response.onboardingStatus),
            voiceArtifactSyncReceiptRef: returnedVoiceArtifactSyncReceiptRef
        )
    }

    static func failed(
        onboardingSessionID: String,
        deviceID: String,
        sampleSeed: String?,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopVoiceEnrollRuntimeOutcomeState {
        DesktopVoiceEnrollRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Desktop voice-enroll lock failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            deviceID: deviceID,
            sampleSeed: sampleSeed,
            outcome: nil,
            reason: reason,
            onboardingSessionID: onboardingSessionID,
            nextStep: nil,
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil,
            voiceArtifactSyncReceiptRef: nil
        )
    }
}

struct DesktopWakeEnrollStartDraftRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let deviceID: String
    let outcome: String?
    let reason: String?
    let onboardingSessionID: String?
    let nextStep: String?
    let remainingPlatformReceiptKinds: [String]
    let onboardingStatus: String?
    let voiceArtifactSyncReceiptRef: String?

    static func dispatching(
        onboardingSessionID: String,
        deviceID: String,
        endpoint: String,
        requestID: String
    ) -> DesktopWakeEnrollStartDraftRuntimeOutcomeState {
        DesktopWakeEnrollStartDraftRuntimeOutcomeState(
            id: requestID,
            phase: .dispatching,
            title: "Dispatching desktop wake-enroll start draft",
            summary: "The bounded desktop wake-enroll start-draft request is now being handed into canonical `/v1/onboarding/continue`.",
            detail: "Only exact `WAKE_ENROLL_START_DRAFT` is in scope here. This exact wake-start path remains explicitly non-authoritative; any later wake-sample, wake-complete, and wake-defer controls are separately gated, while sender verification, employee photo capture, emo-persona lock, access provisioning, pairing completion, wake-listener behavior, and autonomous unlock remain out of scope here.",
            endpoint: endpoint,
            requestID: requestID,
            deviceID: deviceID,
            outcome: nil,
            reason: nil,
            onboardingSessionID: onboardingSessionID,
            nextStep: "WAKE_ENROLL",
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil,
            voiceArtifactSyncReceiptRef: nil
        )
    }

    static func completed(
        requestID: String,
        endpoint: String,
        response: DesktopCanonicalRuntimeBridge.OnboardingContinueAdapterResponsePayload,
        fallbackOnboardingSessionID: String,
        fallbackDeviceID: String
    ) -> DesktopWakeEnrollStartDraftRuntimeOutcomeState {
        let boundedNextStep = boundedOnboardingContinueField(response.nextStep)
        let advancedBeyondWakeEnroll = boundedNextStep != nil && boundedNextStep != "WAKE_ENROLL"
        let returnedVoiceArtifactSyncReceiptRef = boundedOnboardingContinueField(response.voiceArtifactSyncReceiptRef)

        return DesktopWakeEnrollStartDraftRuntimeOutcomeState(
            id: requestID,
            phase: .completed,
            title: "Desktop wake-enroll start draft completed",
            summary: advancedBeyondWakeEnroll
                ? "Canonical `/v1/onboarding/continue` advanced beyond `WAKE_ENROLL`; later onboarding actions remain read-only and out of scope in this shell."
                : "Canonical `/v1/onboarding/continue` accepted the bounded desktop wake-enroll start draft and returned updated onboarding posture.",
            detail: boundedNextStep == "WAKE_ENROLL"
                ? "Read-only next-step visibility only. This exact wake-start surface preserves returned exact `WAKE_ENROLL` posture plus exact `voice_artifact_sync_receipt_ref`; any later wake-sample, wake-complete, and wake-defer submit remain separately gated."
                : advancedBeyondWakeEnroll
                    ? "Read-only next-step visibility only. This shell preserves the advanced step and any returned voice-artifact sync receipt without adding later wake mutation, sender verification, emo-persona lock, access provisioning, pairing completion, or autonomous unlock."
                    : "Canonical wake-enroll start draft only. This shell preserves returned wake posture without adding local wake authority, wake-listener behavior, or later onboarding controls.",
            endpoint: endpoint,
            requestID: requestID,
            deviceID: fallbackDeviceID,
            outcome: boundedOnboardingContinueField(response.outcome) ?? "ONBOARDING_CONTINUED",
            reason: boundedOnboardingContinueField(response.reason),
            onboardingSessionID: boundedOnboardingContinueField(response.onboardingSessionID) ?? fallbackOnboardingSessionID,
            nextStep: boundedNextStep,
            remainingPlatformReceiptKinds: boundedOnboardingContinueList(response.remainingPlatformReceiptKinds),
            onboardingStatus: boundedOnboardingContinueField(response.onboardingStatus),
            voiceArtifactSyncReceiptRef: returnedVoiceArtifactSyncReceiptRef
        )
    }

    static func failed(
        onboardingSessionID: String,
        deviceID: String,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopWakeEnrollStartDraftRuntimeOutcomeState {
        DesktopWakeEnrollStartDraftRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Desktop wake-enroll start draft failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            deviceID: deviceID,
            outcome: nil,
            reason: reason,
            onboardingSessionID: onboardingSessionID,
            nextStep: nil,
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil,
            voiceArtifactSyncReceiptRef: nil
        )
    }
}

struct DesktopWakeEnrollSampleCommitRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let deviceID: String
    let proofOK: Bool
    let outcome: String?
    let reason: String?
    let onboardingSessionID: String?
    let nextStep: String?
    let remainingPlatformReceiptKinds: [String]
    let onboardingStatus: String?
    let voiceArtifactSyncReceiptRef: String?

    static func dispatching(
        onboardingSessionID: String,
        deviceID: String,
        endpoint: String,
        requestID: String
    ) -> DesktopWakeEnrollSampleCommitRuntimeOutcomeState {
        DesktopWakeEnrollSampleCommitRuntimeOutcomeState(
            id: requestID,
            phase: .dispatching,
            title: "Dispatching desktop wake-enroll sample commit",
            summary: "The bounded desktop wake-enroll sample-commit request is now being handed into canonical `/v1/onboarding/continue`.",
            detail: "Only exact `WAKE_ENROLL_SAMPLE_COMMIT` is in scope here. This exact sample-commit path remains explicitly non-authoritative; any later wake-complete and wake-defer submit remain separately gated, while sender verification, employee photo capture, emo-persona lock, access provisioning, pairing completion, wake-listener behavior, and autonomous unlock remain out of scope here.",
            endpoint: endpoint,
            requestID: requestID,
            deviceID: deviceID,
            proofOK: true,
            outcome: nil,
            reason: nil,
            onboardingSessionID: onboardingSessionID,
            nextStep: "WAKE_ENROLL",
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil,
            voiceArtifactSyncReceiptRef: nil
        )
    }

    static func completed(
        requestID: String,
        endpoint: String,
        response: DesktopCanonicalRuntimeBridge.OnboardingContinueAdapterResponsePayload,
        fallbackOnboardingSessionID: String,
        fallbackDeviceID: String
    ) -> DesktopWakeEnrollSampleCommitRuntimeOutcomeState {
        let boundedNextStep = boundedOnboardingContinueField(response.nextStep)
        let advancedBeyondWakeEnroll = boundedNextStep != nil && boundedNextStep != "WAKE_ENROLL"
        let returnedVoiceArtifactSyncReceiptRef = boundedOnboardingContinueField(response.voiceArtifactSyncReceiptRef)

        return DesktopWakeEnrollSampleCommitRuntimeOutcomeState(
            id: requestID,
            phase: .completed,
            title: "Desktop wake-enroll sample commit completed",
            summary: advancedBeyondWakeEnroll
                ? "Canonical `/v1/onboarding/continue` advanced beyond `WAKE_ENROLL`; later onboarding actions remain read-only and out of scope in this shell."
                : "Canonical `/v1/onboarding/continue` accepted the bounded desktop wake-enroll sample commit and returned updated onboarding posture.",
            detail: boundedNextStep == "WAKE_ENROLL"
                ? "Read-only next-step visibility only. This exact sample-commit surface preserves returned exact `WAKE_ENROLL` posture plus exact `voice_artifact_sync_receipt_ref` while keeping another explicit sample-commit submit available only when lawful bounded prompt state remains present; any wake-complete submit remains separately gated."
                : advancedBeyondWakeEnroll
                    ? "Read-only next-step visibility only. This shell preserves the advanced step and any returned voice-artifact sync receipt without auto-dispatching wake-complete or wake-defer, and without adding sender verification, emo-persona lock submit behavior, access provisioning, pairing completion, or autonomous unlock."
                    : "Canonical wake-enroll sample commit only. This shell preserves returned wake posture without batching, auto-looping, local wake authority, or later onboarding controls.",
            endpoint: endpoint,
            requestID: requestID,
            deviceID: fallbackDeviceID,
            proofOK: true,
            outcome: boundedOnboardingContinueField(response.outcome) ?? "ONBOARDING_CONTINUED",
            reason: boundedOnboardingContinueField(response.reason),
            onboardingSessionID: boundedOnboardingContinueField(response.onboardingSessionID) ?? fallbackOnboardingSessionID,
            nextStep: boundedNextStep,
            remainingPlatformReceiptKinds: boundedOnboardingContinueList(response.remainingPlatformReceiptKinds),
            onboardingStatus: boundedOnboardingContinueField(response.onboardingStatus),
            voiceArtifactSyncReceiptRef: returnedVoiceArtifactSyncReceiptRef
        )
    }

    static func failed(
        onboardingSessionID: String,
        deviceID: String,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopWakeEnrollSampleCommitRuntimeOutcomeState {
        DesktopWakeEnrollSampleCommitRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Desktop wake-enroll sample commit failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            deviceID: deviceID,
            proofOK: true,
            outcome: nil,
            reason: reason,
            onboardingSessionID: onboardingSessionID,
            nextStep: nil,
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil,
            voiceArtifactSyncReceiptRef: nil
        )
    }
}

struct DesktopWakeEnrollCompleteCommitRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let deviceID: String
    let outcome: String?
    let reason: String?
    let onboardingSessionID: String?
    let nextStep: String?
    let remainingPlatformReceiptKinds: [String]
    let onboardingStatus: String?
    let voiceArtifactSyncReceiptRef: String?

    static func dispatching(
        onboardingSessionID: String,
        deviceID: String,
        endpoint: String,
        requestID: String
    ) -> DesktopWakeEnrollCompleteCommitRuntimeOutcomeState {
        DesktopWakeEnrollCompleteCommitRuntimeOutcomeState(
            id: requestID,
            phase: .dispatching,
            title: "Dispatching desktop wake-enroll complete commit",
            summary: "The bounded desktop wake-enroll complete-commit request is now being handed into canonical `/v1/onboarding/continue`.",
            detail: "Only exact wake-enroll complete commit is in scope here. This exact wake-complete path remains explicitly non-authoritative, keeps any wake-defer submit separately gated, and does not introduce emo-persona lock submit behavior, access provisioning, pairing completion, wake-listener behavior, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            deviceID: deviceID,
            outcome: nil,
            reason: nil,
            onboardingSessionID: onboardingSessionID,
            nextStep: "WAKE_ENROLL",
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil,
            voiceArtifactSyncReceiptRef: nil
        )
    }

    static func completed(
        requestID: String,
        endpoint: String,
        response: DesktopCanonicalRuntimeBridge.OnboardingContinueAdapterResponsePayload,
        fallbackOnboardingSessionID: String,
        fallbackDeviceID: String
    ) -> DesktopWakeEnrollCompleteCommitRuntimeOutcomeState {
        let boundedNextStep = boundedOnboardingContinueField(response.nextStep)
        let advancedBeyondWakeEnroll = boundedNextStep != nil && boundedNextStep != "WAKE_ENROLL"
        let returnedVoiceArtifactSyncReceiptRef = boundedOnboardingContinueField(response.voiceArtifactSyncReceiptRef)

        return DesktopWakeEnrollCompleteCommitRuntimeOutcomeState(
            id: requestID,
            phase: .completed,
            title: "Desktop wake-enroll complete commit completed",
            summary: advancedBeyondWakeEnroll
                ? "Canonical `/v1/onboarding/continue` advanced beyond `WAKE_ENROLL`; later onboarding actions remain read-only and out of scope in this shell."
                : "Canonical `/v1/onboarding/continue` accepted the bounded desktop wake-enroll complete commit and returned updated onboarding posture.",
            detail: boundedNextStep == "EMO_PERSONA_LOCK"
                ? "Read-only next-step visibility only. This exact wake-complete surface preserves returned exact `EMO_PERSONA_LOCK` plus any returned exact `voice_artifact_sync_receipt_ref`; emo-persona submit remains separately gated and non-authoritative."
                : boundedNextStep == "WAKE_ENROLL"
                    ? "Read-only next-step visibility only. This exact wake-complete surface preserves returned exact `WAKE_ENROLL` posture plus exact `voice_artifact_sync_receipt_ref`; any wake-defer submit remains separately gated."
                    : advancedBeyondWakeEnroll
                        ? "Read-only next-step visibility only. This shell preserves the advanced step and any returned voice-artifact sync receipt without adding emo-persona submit behavior, access provisioning, pairing completion, wake-listener behavior, or autonomous unlock."
                        : "Canonical wake-enroll complete commit only. This shell preserves returned wake posture without local wake authority, emo-persona submit behavior, or later onboarding controls.",
            endpoint: endpoint,
            requestID: requestID,
            deviceID: fallbackDeviceID,
            outcome: boundedOnboardingContinueField(response.outcome) ?? "ONBOARDING_CONTINUED",
            reason: boundedOnboardingContinueField(response.reason),
            onboardingSessionID: boundedOnboardingContinueField(response.onboardingSessionID) ?? fallbackOnboardingSessionID,
            nextStep: boundedNextStep,
            remainingPlatformReceiptKinds: boundedOnboardingContinueList(response.remainingPlatformReceiptKinds),
            onboardingStatus: boundedOnboardingContinueField(response.onboardingStatus),
            voiceArtifactSyncReceiptRef: returnedVoiceArtifactSyncReceiptRef
        )
    }

    static func failed(
        onboardingSessionID: String,
        deviceID: String,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopWakeEnrollCompleteCommitRuntimeOutcomeState {
        DesktopWakeEnrollCompleteCommitRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Desktop wake-enroll complete commit failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            deviceID: deviceID,
            outcome: nil,
            reason: reason,
            onboardingSessionID: onboardingSessionID,
            nextStep: nil,
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil,
            voiceArtifactSyncReceiptRef: nil
        )
    }
}

struct DesktopWakeEnrollDeferCommitRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let deviceID: String
    let outcome: String?
    let reason: String?
    let onboardingSessionID: String?
    let nextStep: String?
    let remainingPlatformReceiptKinds: [String]
    let onboardingStatus: String?
    let voiceArtifactSyncReceiptRef: String?

    static func dispatching(
        onboardingSessionID: String,
        deviceID: String,
        endpoint: String,
        requestID: String
    ) -> DesktopWakeEnrollDeferCommitRuntimeOutcomeState {
        DesktopWakeEnrollDeferCommitRuntimeOutcomeState(
            id: requestID,
            phase: .dispatching,
            title: "Dispatching desktop wake-enroll defer commit",
            summary: "The bounded desktop wake-enroll defer-commit request is now being handed into canonical `/v1/onboarding/continue`.",
            detail: "Only exact `WAKE_ENROLL_DEFER_COMMIT` is in scope here. This exact wake-defer path remains explicitly non-authoritative and does not introduce local `deferred_until` authoring, wake-listener behavior, pairing completion mutation, session resume / attach / reopen mutation, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            deviceID: deviceID,
            outcome: nil,
            reason: nil,
            onboardingSessionID: onboardingSessionID,
            nextStep: "WAKE_ENROLL",
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil,
            voiceArtifactSyncReceiptRef: nil
        )
    }

    static func completed(
        requestID: String,
        endpoint: String,
        response: DesktopCanonicalRuntimeBridge.OnboardingContinueAdapterResponsePayload,
        fallbackOnboardingSessionID: String,
        fallbackDeviceID: String
    ) -> DesktopWakeEnrollDeferCommitRuntimeOutcomeState {
        let boundedNextStep = boundedOnboardingContinueField(response.nextStep)
        let advancedBeyondWakeEnroll = boundedNextStep != nil && boundedNextStep != "WAKE_ENROLL"
        let returnedVoiceArtifactSyncReceiptRef = boundedOnboardingContinueField(response.voiceArtifactSyncReceiptRef)

        return DesktopWakeEnrollDeferCommitRuntimeOutcomeState(
            id: requestID,
            phase: .completed,
            title: "Desktop wake-enroll defer commit completed",
            summary: advancedBeyondWakeEnroll
                ? "Canonical `/v1/onboarding/continue` advanced beyond `WAKE_ENROLL`; later onboarding actions remain read-only and out of scope in this shell."
                : "Canonical `/v1/onboarding/continue` accepted the bounded desktop wake-enroll defer commit and returned updated onboarding posture.",
            detail: boundedNextStep == "WAKE_ENROLL"
                ? "Read-only next-step visibility only. This exact wake-defer surface preserves returned exact `WAKE_ENROLL` posture plus exact `voice_artifact_sync_receipt_ref` without adding local `deferred_until` authoring, wake-listener integration, pairing completion mutation, or autonomous unlock."
                : advancedBeyondWakeEnroll
                    ? "Read-only next-step visibility only. This shell preserves the advanced step and any returned voice-artifact sync receipt without adding ready-time handoff, session resume / attach / reopen mutation, wake-listener behavior, or autonomous unlock."
                    : "Canonical wake-enroll defer commit only. This shell preserves returned wake posture without local wake authority, local scheduling authority, or later onboarding mutation.",
            endpoint: endpoint,
            requestID: requestID,
            deviceID: fallbackDeviceID,
            outcome: boundedOnboardingContinueField(response.outcome) ?? "ONBOARDING_CONTINUED",
            reason: boundedOnboardingContinueField(response.reason),
            onboardingSessionID: boundedOnboardingContinueField(response.onboardingSessionID) ?? fallbackOnboardingSessionID,
            nextStep: boundedNextStep,
            remainingPlatformReceiptKinds: boundedOnboardingContinueList(response.remainingPlatformReceiptKinds),
            onboardingStatus: boundedOnboardingContinueField(response.onboardingStatus),
            voiceArtifactSyncReceiptRef: returnedVoiceArtifactSyncReceiptRef
        )
    }

    static func failed(
        onboardingSessionID: String,
        deviceID: String,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopWakeEnrollDeferCommitRuntimeOutcomeState {
        DesktopWakeEnrollDeferCommitRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Desktop wake-enroll defer commit failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            deviceID: deviceID,
            outcome: nil,
            reason: reason,
            onboardingSessionID: onboardingSessionID,
            nextStep: nil,
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil,
            voiceArtifactSyncReceiptRef: nil
        )
    }
}

struct DesktopSessionAttachRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let sourceSurfaceIdentity: String
    let sessionState: String
    let sessionID: String
    let currentVisibleSessionAttachOutcome: String?
    let turnID: String?
    let deviceID: String
    let outcome: String?
    let reason: String?
    let sessionAttachOutcome: String?

    static func dispatching(
        ingressContext: DesktopCanonicalRuntimeBridge.DesktopSessionAttachIngressContext
    ) -> DesktopSessionAttachRuntimeOutcomeState {
        DesktopSessionAttachRuntimeOutcomeState(
            id: ingressContext.requestID,
            phase: .dispatching,
            title: "Dispatching desktop current-visible session attach",
            summary: "The bounded desktop current-visible session attach request is now being handed into canonical `/v1/session/attach`.",
            detail: "Only exact current-visible session attach is in scope here. This shell remains explicitly non-authoritative and does not introduce local reopen authority, conversation selection, search or tool controls, hidden/background wake behavior, or autonomous unlock.",
            endpoint: ingressContext.endpoint,
            requestID: ingressContext.requestID,
            sourceSurfaceIdentity: ingressContext.sourceSurfaceIdentity,
            sessionState: ingressContext.sessionState,
            sessionID: ingressContext.sessionID,
            currentVisibleSessionAttachOutcome: ingressContext.currentVisibleSessionAttachOutcome,
            turnID: ingressContext.turnID,
            deviceID: ingressContext.deviceID,
            outcome: nil,
            reason: nil,
            sessionAttachOutcome: nil
        )
    }

    static func completed(
        requestID: String,
        endpoint: String,
        response: DesktopCanonicalRuntimeBridge.SessionAttachAdapterResponsePayload,
        fallbackSourceSurfaceIdentity: String,
        fallbackSessionState: String,
        fallbackSessionID: String,
        fallbackCurrentVisibleSessionAttachOutcome: String?,
        fallbackTurnID: String?,
        fallbackDeviceID: String
    ) -> DesktopSessionAttachRuntimeOutcomeState {
        let boundedSessionState = boundedOnboardingContinueField(response.sessionState)
        let boundedAttachOutcome = boundedOnboardingContinueField(response.sessionAttachOutcome)

        return DesktopSessionAttachRuntimeOutcomeState(
            id: requestID,
            phase: .completed,
            title: "Desktop current-visible session attach completed",
            summary: "Canonical `/v1/session/attach` accepted the bounded desktop current-visible session attach request and returned updated session posture.",
            detail: "Read-only returned posture only. This shell preserves exact returned `session_state` and exact `session_attach_outcome` without introducing local reopen authority, conversation selection, search or tool controls, hidden/background wake behavior, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            sourceSurfaceIdentity: fallbackSourceSurfaceIdentity,
            sessionState: boundedSessionState ?? fallbackSessionState,
            sessionID: boundedOnboardingContinueField(response.sessionID) ?? fallbackSessionID,
            currentVisibleSessionAttachOutcome: fallbackCurrentVisibleSessionAttachOutcome,
            turnID: fallbackTurnID,
            deviceID: fallbackDeviceID,
            outcome: boundedOnboardingContinueField(response.outcome) ?? "SESSION_ATTACHED",
            reason: boundedOnboardingContinueField(response.reason),
            sessionAttachOutcome: boundedAttachOutcome
        )
    }

    static func failed(
        sourceSurfaceIdentity: String,
        sessionState: String,
        sessionID: String,
        currentVisibleSessionAttachOutcome: String?,
        turnID: String?,
        deviceID: String,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopSessionAttachRuntimeOutcomeState {
        DesktopSessionAttachRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Desktop current-visible session attach failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            sourceSurfaceIdentity: sourceSurfaceIdentity,
            sessionState: sessionState,
            sessionID: sessionID,
            currentVisibleSessionAttachOutcome: currentVisibleSessionAttachOutcome,
            turnID: turnID,
            deviceID: deviceID,
            outcome: nil,
            reason: reason,
            sessionAttachOutcome: nil
        )
    }
}

struct DesktopSessionSoftClosedResumeRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let sourceSurfaceIdentity: String
    let sessionState: String
    let sessionID: String
    let selectedThreadID: String?
    let selectedThreadTitle: String?
    let pendingWorkOrderID: String?
    let resumeTier: String?
    let resumeSummaryBullets: [String]
    let deviceID: String
    let outcome: String?
    let reason: String?
    let sessionAttachOutcome: String?

    static func dispatching(
        sourceSurfaceIdentity: String,
        sessionState: String,
        sessionID: String,
        selectedThreadID: String?,
        selectedThreadTitle: String?,
        pendingWorkOrderID: String?,
        resumeTier: String?,
        resumeSummaryBullets: [String],
        deviceID: String,
        endpoint: String,
        requestID: String
    ) -> DesktopSessionSoftClosedResumeRuntimeOutcomeState {
        DesktopSessionSoftClosedResumeRuntimeOutcomeState(
            id: requestID,
            phase: .dispatching,
            title: "Dispatching desktop soft-closed explicit resume",
            summary: "The bounded desktop soft-closed explicit resume request is now being handed into canonical `/v1/session/resume`.",
            detail: "Only exact soft-closed explicit resume is in scope here. This shell remains explicitly non-authoritative and does not introduce local thread reselection, local PH1.M synthesis, pairing completion mutation, ready-time handoff, wake-listener behavior, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            sourceSurfaceIdentity: sourceSurfaceIdentity,
            sessionState: sessionState,
            sessionID: sessionID,
            selectedThreadID: selectedThreadID,
            selectedThreadTitle: selectedThreadTitle,
            pendingWorkOrderID: pendingWorkOrderID,
            resumeTier: resumeTier,
            resumeSummaryBullets: resumeSummaryBullets,
            deviceID: deviceID,
            outcome: nil,
            reason: nil,
            sessionAttachOutcome: nil
        )
    }

    static func completed(
        requestID: String,
        endpoint: String,
        response: DesktopCanonicalRuntimeBridge.SessionResumeAdapterResponsePayload,
        fallbackSourceSurfaceIdentity: String,
        fallbackSessionState: String,
        fallbackSessionID: String,
        fallbackSelectedThreadID: String?,
        fallbackSelectedThreadTitle: String?,
        fallbackPendingWorkOrderID: String?,
        fallbackResumeTier: String?,
        fallbackResumeSummaryBullets: [String],
        fallbackDeviceID: String
    ) -> DesktopSessionSoftClosedResumeRuntimeOutcomeState {
        let boundedSessionState = boundedOnboardingContinueField(response.sessionState)
        let boundedAttachOutcome = boundedOnboardingContinueField(response.sessionAttachOutcome)

        return DesktopSessionSoftClosedResumeRuntimeOutcomeState(
            id: requestID,
            phase: .completed,
            title: "Desktop soft-closed explicit resume completed",
            summary: "Canonical `/v1/session/resume` accepted the bounded desktop soft-closed explicit resume request and returned updated session posture.",
            detail: "Read-only returned posture only. This shell preserves exact returned `session_state` and exact `session_attach_outcome` without introducing broader attach/reopen mutation, local thread reselection, local PH1.M synthesis, pairing completion mutation, ready-time handoff, wake-listener behavior, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            sourceSurfaceIdentity: fallbackSourceSurfaceIdentity,
            sessionState: boundedSessionState ?? fallbackSessionState,
            sessionID: boundedOnboardingContinueField(response.sessionID) ?? fallbackSessionID,
            selectedThreadID: fallbackSelectedThreadID,
            selectedThreadTitle: fallbackSelectedThreadTitle,
            pendingWorkOrderID: fallbackPendingWorkOrderID,
            resumeTier: fallbackResumeTier,
            resumeSummaryBullets: fallbackResumeSummaryBullets,
            deviceID: fallbackDeviceID,
            outcome: boundedOnboardingContinueField(response.outcome) ?? "SESSION_RESUMED",
            reason: boundedOnboardingContinueField(response.reason),
            sessionAttachOutcome: boundedAttachOutcome
        )
    }

    static func failed(
        sourceSurfaceIdentity: String,
        sessionState: String,
        sessionID: String,
        selectedThreadID: String?,
        selectedThreadTitle: String?,
        pendingWorkOrderID: String?,
        resumeTier: String?,
        resumeSummaryBullets: [String],
        deviceID: String,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopSessionSoftClosedResumeRuntimeOutcomeState {
        DesktopSessionSoftClosedResumeRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Desktop soft-closed explicit resume failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            sourceSurfaceIdentity: sourceSurfaceIdentity,
            sessionState: sessionState,
            sessionID: sessionID,
            selectedThreadID: selectedThreadID,
            selectedThreadTitle: selectedThreadTitle,
            pendingWorkOrderID: pendingWorkOrderID,
            resumeTier: resumeTier,
            resumeSummaryBullets: resumeSummaryBullets,
            deviceID: deviceID,
            outcome: nil,
            reason: reason,
            sessionAttachOutcome: nil
        )
    }
}

struct DesktopSessionRecoverRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let sourceSurfaceIdentity: String
    let sessionState: String
    let sessionID: String
    let recoveryMode: String?
    let deviceID: String
    let outcome: String?
    let reason: String?
    let sessionAttachOutcome: String?

    static func dispatching(
        sourceSurfaceIdentity: String,
        sessionState: String,
        sessionID: String,
        recoveryMode: String?,
        deviceID: String,
        endpoint: String,
        requestID: String
    ) -> DesktopSessionRecoverRuntimeOutcomeState {
        DesktopSessionRecoverRuntimeOutcomeState(
            id: requestID,
            phase: .dispatching,
            title: "Dispatching desktop suspended-session recover submission",
            summary: "The bounded desktop suspended-session authoritative-reread recovery request is now being handed into canonical `/v1/session/recover`.",
            detail: "Only exact suspended-session recover submission is in scope here. This shell remains explicitly non-authoritative and does not introduce local unsuspend authority, local attach/reopen authority, search or tool controls, hidden/background wake behavior, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            sourceSurfaceIdentity: sourceSurfaceIdentity,
            sessionState: sessionState,
            sessionID: sessionID,
            recoveryMode: recoveryMode,
            deviceID: deviceID,
            outcome: nil,
            reason: nil,
            sessionAttachOutcome: nil
        )
    }

    static func completed(
        requestID: String,
        endpoint: String,
        response: DesktopCanonicalRuntimeBridge.SessionRecoverAdapterResponsePayload,
        fallbackSourceSurfaceIdentity: String,
        fallbackSessionState: String,
        fallbackSessionID: String,
        fallbackRecoveryMode: String?,
        fallbackDeviceID: String
    ) -> DesktopSessionRecoverRuntimeOutcomeState {
        let boundedSessionState = boundedOnboardingContinueField(response.sessionState)
        let boundedAttachOutcome = boundedOnboardingContinueField(response.sessionAttachOutcome)

        return DesktopSessionRecoverRuntimeOutcomeState(
            id: requestID,
            phase: .completed,
            title: "Desktop suspended-session recover submission completed",
            summary: "Canonical `/v1/session/recover` accepted the bounded desktop suspended-session recover request and returned updated session posture.",
            detail: "Read-only returned posture only. This shell preserves exact returned `session_state` and exact `session_attach_outcome` without introducing local unsuspend authority, broader attach/reopen mutation, search or tool controls, hidden/background wake behavior, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            sourceSurfaceIdentity: fallbackSourceSurfaceIdentity,
            sessionState: boundedSessionState ?? fallbackSessionState,
            sessionID: boundedOnboardingContinueField(response.sessionID) ?? fallbackSessionID,
            recoveryMode: fallbackRecoveryMode,
            deviceID: fallbackDeviceID,
            outcome: boundedOnboardingContinueField(response.outcome) ?? "SESSION_RECOVERED",
            reason: boundedOnboardingContinueField(response.reason),
            sessionAttachOutcome: boundedAttachOutcome
        )
    }

    static func failed(
        sourceSurfaceIdentity: String,
        sessionState: String,
        sessionID: String,
        recoveryMode: String?,
        deviceID: String,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopSessionRecoverRuntimeOutcomeState {
        DesktopSessionRecoverRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Desktop suspended-session recover submission failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            sourceSurfaceIdentity: sourceSurfaceIdentity,
            sessionState: sessionState,
            sessionID: sessionID,
            recoveryMode: recoveryMode,
            deviceID: deviceID,
            outcome: nil,
            reason: reason,
            sessionAttachOutcome: nil
        )
    }
}

enum DesktopSessionMultiPostureResumeMode: String, Equatable {
    case softClosedExplicitResume = "SOFT_CLOSED_EXPLICIT_RESUME"
    case suspendedAuthoritativeRereadRecover = "SUSPENDED_AUTHORITATIVE_REREAD_RECOVER"
}

struct DesktopSessionMultiPostureResumeRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
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
    let outcome: String?
    let reason: String?
    let sessionAttachOutcome: String?

    static func dispatching(
        ingressContext: DesktopCanonicalRuntimeBridge.DesktopSessionMultiPostureResumeIngressContext
    ) -> DesktopSessionMultiPostureResumeRuntimeOutcomeState {
        let summary: String
        let detail: String

        switch ingressContext.resumeMode {
        case .softClosedExplicitResume:
            summary = "The bounded desktop multi-posture session-resume control selected exact soft-closed explicit resume and is now handing that request into canonical `/v1/session/resume`."
            detail = "Only exact soft-closed explicit resume is in scope for this dispatch. This shell remains explicitly non-authoritative and does not introduce broader attach/reopen mutation, local conversation selection, search or tool controls, hidden/background wake behavior, or autonomous unlock."
        case .suspendedAuthoritativeRereadRecover:
            summary = "The bounded desktop multi-posture session-resume control selected exact suspended-session authoritative-reread recover and is now handing that request into canonical `/v1/session/recover`."
            detail = "Only exact suspended-session authoritative-reread recover is in scope for this dispatch. This shell remains explicitly non-authoritative and does not introduce local unsuspend authority, broader attach/reopen mutation, local conversation selection, search or tool controls, hidden/background wake behavior, or autonomous unlock."
        }

        return DesktopSessionMultiPostureResumeRuntimeOutcomeState(
            id: ingressContext.requestID,
            phase: .dispatching,
            title: "Dispatching desktop multi-posture session resume",
            summary: summary,
            detail: detail,
            endpoint: ingressContext.endpoint,
            requestID: ingressContext.requestID,
            resumeMode: ingressContext.resumeMode,
            sourceSurfaceIdentity: ingressContext.sourceSurfaceIdentity,
            sessionState: ingressContext.sessionState,
            sessionID: ingressContext.sessionID,
            selectedThreadID: ingressContext.selectedThreadID,
            selectedThreadTitle: ingressContext.selectedThreadTitle,
            pendingWorkOrderID: ingressContext.pendingWorkOrderID,
            resumeTier: ingressContext.resumeTier,
            resumeSummaryBullets: ingressContext.resumeSummaryBullets,
            recoveryMode: ingressContext.recoveryMode,
            deviceID: ingressContext.deviceID,
            outcome: nil,
            reason: nil,
            sessionAttachOutcome: nil
        )
    }

    static func fromSoftClosedRoute(
        _ routeOutcome: DesktopSessionSoftClosedResumeRuntimeOutcomeState
    ) -> DesktopSessionMultiPostureResumeRuntimeOutcomeState {
        let phase: Phase
        switch routeOutcome.phase {
        case .dispatching:
            phase = .dispatching
        case .completed:
            phase = .completed
        case .failed:
            phase = .failed
        }

        return DesktopSessionMultiPostureResumeRuntimeOutcomeState(
            id: routeOutcome.id,
            phase: phase,
            title: routeOutcome.title,
            summary: routeOutcome.summary,
            detail: routeOutcome.detail,
            endpoint: routeOutcome.endpoint,
            requestID: routeOutcome.requestID,
            resumeMode: .softClosedExplicitResume,
            sourceSurfaceIdentity: routeOutcome.sourceSurfaceIdentity,
            sessionState: routeOutcome.sessionState,
            sessionID: routeOutcome.sessionID,
            selectedThreadID: routeOutcome.selectedThreadID,
            selectedThreadTitle: routeOutcome.selectedThreadTitle,
            pendingWorkOrderID: routeOutcome.pendingWorkOrderID,
            resumeTier: routeOutcome.resumeTier,
            resumeSummaryBullets: routeOutcome.resumeSummaryBullets,
            recoveryMode: nil,
            deviceID: routeOutcome.deviceID,
            outcome: routeOutcome.outcome,
            reason: routeOutcome.reason,
            sessionAttachOutcome: routeOutcome.sessionAttachOutcome
        )
    }

    static func fromRecoverRoute(
        _ routeOutcome: DesktopSessionRecoverRuntimeOutcomeState
    ) -> DesktopSessionMultiPostureResumeRuntimeOutcomeState {
        let phase: Phase
        switch routeOutcome.phase {
        case .dispatching:
            phase = .dispatching
        case .completed:
            phase = .completed
        case .failed:
            phase = .failed
        }

        return DesktopSessionMultiPostureResumeRuntimeOutcomeState(
            id: routeOutcome.id,
            phase: phase,
            title: routeOutcome.title,
            summary: routeOutcome.summary,
            detail: routeOutcome.detail,
            endpoint: routeOutcome.endpoint,
            requestID: routeOutcome.requestID,
            resumeMode: .suspendedAuthoritativeRereadRecover,
            sourceSurfaceIdentity: routeOutcome.sourceSurfaceIdentity,
            sessionState: routeOutcome.sessionState,
            sessionID: routeOutcome.sessionID,
            selectedThreadID: nil,
            selectedThreadTitle: nil,
            pendingWorkOrderID: nil,
            resumeTier: nil,
            resumeSummaryBullets: [],
            recoveryMode: routeOutcome.recoveryMode,
            deviceID: routeOutcome.deviceID,
            outcome: routeOutcome.outcome,
            reason: routeOutcome.reason,
            sessionAttachOutcome: routeOutcome.sessionAttachOutcome
        )
    }

    static func failed(
        resumeMode: DesktopSessionMultiPostureResumeMode,
        sourceSurfaceIdentity: String,
        sessionState: String,
        sessionID: String,
        selectedThreadID: String?,
        selectedThreadTitle: String?,
        pendingWorkOrderID: String?,
        resumeTier: String?,
        resumeSummaryBullets: [String],
        recoveryMode: String?,
        deviceID: String,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopSessionMultiPostureResumeRuntimeOutcomeState {
        DesktopSessionMultiPostureResumeRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Desktop multi-posture session resume failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            resumeMode: resumeMode,
            sourceSurfaceIdentity: sourceSurfaceIdentity,
            sessionState: sessionState,
            sessionID: sessionID,
            selectedThreadID: selectedThreadID,
            selectedThreadTitle: selectedThreadTitle,
            pendingWorkOrderID: pendingWorkOrderID,
            resumeTier: resumeTier,
            resumeSummaryBullets: resumeSummaryBullets,
            recoveryMode: recoveryMode,
            deviceID: deviceID,
            outcome: nil,
            reason: reason,
            sessionAttachOutcome: nil
        )
    }
}

struct DesktopWakeProfileAvailabilityRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let receiptKind: String
    let deviceID: String
    let wakeProfileID: String
    let voiceArtifactSyncReceiptRef: String
    let outcome: String?
    let reason: String?
    let activeWakeArtifactVersion: String?
    let activatedCount: UInt64
    let noopCount: UInt64
    let pullErrorCount: UInt64

    static func dispatching(
        receiptKind: String,
        deviceID: String,
        wakeProfileID: String,
        voiceArtifactSyncReceiptRef: String,
        endpoint: String,
        requestID: String
    ) -> DesktopWakeProfileAvailabilityRuntimeOutcomeState {
        DesktopWakeProfileAvailabilityRuntimeOutcomeState(
            id: requestID,
            phase: .dispatching,
            title: "Dispatching desktop wake-profile availability refresh",
            summary: "The bounded desktop wake-profile local-availability refresh is now being handed into canonical `/v1/wake-profile/availability`.",
            detail: "Only exact wake-profile local-availability refresh is in scope here. This shell remains explicitly non-authoritative and does not introduce native wake-listener start or stop, wake detection, wake-to-turn dispatch, hidden/background auto-start, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            receiptKind: receiptKind,
            deviceID: deviceID,
            wakeProfileID: wakeProfileID,
            voiceArtifactSyncReceiptRef: voiceArtifactSyncReceiptRef,
            outcome: nil,
            reason: nil,
            activeWakeArtifactVersion: nil,
            activatedCount: 0,
            noopCount: 0,
            pullErrorCount: 0
        )
    }

    static func completed(
        requestID: String,
        endpoint: String,
        response: DesktopCanonicalRuntimeBridge.WakeProfileAvailabilityRefreshAdapterResponsePayload,
        fallbackReceiptKind: String,
        fallbackDeviceID: String,
        fallbackWakeProfileID: String,
        fallbackVoiceArtifactSyncReceiptRef: String
    ) -> DesktopWakeProfileAvailabilityRuntimeOutcomeState {
        let boundedOutcome = boundedOnboardingContinueField(response.outcome)
        let boundedReason = boundedOnboardingContinueField(response.reason)
        let boundedWakeProfileID = boundedOnboardingContinueField(response.wakeProfileID)
            ?? fallbackWakeProfileID
        let boundedActiveWakeArtifactVersion = boundedOnboardingContinueField(
            response.activeWakeArtifactVersion
        )

        return DesktopWakeProfileAvailabilityRuntimeOutcomeState(
            id: requestID,
            phase: .completed,
            title: "Desktop wake-profile availability refresh completed",
            summary: boundedActiveWakeArtifactVersion != nil
                ? "Canonical `/v1/wake-profile/availability` returned a bounded local active wake-artifact version for the current desktop wake profile."
                : "Canonical `/v1/wake-profile/availability` completed without a visible local active wake-artifact version for the current desktop wake profile.",
            detail: "Read-only returned wake-profile local-availability posture only. This shell preserves exact `wake_profile_id`, exact `active_wake_artifact_version`, and bounded worker-pass counters without introducing native wake-listener start or stop, wake detection, wake-to-turn dispatch, hidden/background auto-start, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            receiptKind: fallbackReceiptKind,
            deviceID: boundedOnboardingContinueField(response.deviceID) ?? fallbackDeviceID,
            wakeProfileID: boundedWakeProfileID,
            voiceArtifactSyncReceiptRef: fallbackVoiceArtifactSyncReceiptRef,
            outcome: boundedOutcome ?? "ACTIVE_VERSION_VISIBLE",
            reason: boundedReason,
            activeWakeArtifactVersion: boundedActiveWakeArtifactVersion,
            activatedCount: response.activatedCount,
            noopCount: response.noopCount,
            pullErrorCount: response.pullErrorCount
        )
    }

    static func failed(
        receiptKind: String,
        deviceID: String,
        wakeProfileID: String,
        voiceArtifactSyncReceiptRef: String,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopWakeProfileAvailabilityRuntimeOutcomeState {
        DesktopWakeProfileAvailabilityRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Desktop wake-profile availability refresh failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            receiptKind: receiptKind,
            deviceID: deviceID,
            wakeProfileID: wakeProfileID,
            voiceArtifactSyncReceiptRef: voiceArtifactSyncReceiptRef,
            outcome: nil,
            reason: reason,
            activeWakeArtifactVersion: nil,
            activatedCount: 0,
            noopCount: 0,
            pullErrorCount: 0
        )
    }
}

struct DesktopEmoPersonaLockRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let outcome: String?
    let reason: String?
    let onboardingSessionID: String?
    let nextStep: String?
    let remainingPlatformReceiptKinds: [String]
    let onboardingStatus: String?
    let voiceArtifactSyncReceiptRef: String?

    static func dispatching(
        onboardingSessionID: String,
        endpoint: String,
        requestID: String
    ) -> DesktopEmoPersonaLockRuntimeOutcomeState {
        DesktopEmoPersonaLockRuntimeOutcomeState(
            id: requestID,
            phase: .dispatching,
            title: "Dispatching desktop emo/persona lock",
            summary: "The bounded desktop emo/persona-lock request is now being handed into canonical `/v1/onboarding/continue`.",
            detail: "Only exact emo/persona lock is in scope here. This exact surface remains explicitly non-authoritative and does not introduce sender verification, employee photo capture, wake defer, access provisioning, pairing completion, wake-listener behavior, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            outcome: nil,
            reason: nil,
            onboardingSessionID: onboardingSessionID,
            nextStep: "EMO_PERSONA_LOCK",
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil,
            voiceArtifactSyncReceiptRef: nil
        )
    }

    static func completed(
        requestID: String,
        endpoint: String,
        response: DesktopCanonicalRuntimeBridge.OnboardingContinueAdapterResponsePayload,
        fallbackOnboardingSessionID: String
    ) -> DesktopEmoPersonaLockRuntimeOutcomeState {
        let boundedNextStep = boundedOnboardingContinueField(response.nextStep)
        let advancedBeyondEmoPersonaLock = boundedNextStep != nil && boundedNextStep != "EMO_PERSONA_LOCK"
        let returnedVoiceArtifactSyncReceiptRef = boundedOnboardingContinueField(response.voiceArtifactSyncReceiptRef)

        return DesktopEmoPersonaLockRuntimeOutcomeState(
            id: requestID,
            phase: .completed,
            title: "Desktop emo/persona lock completed",
            summary: advancedBeyondEmoPersonaLock
                ? "Canonical `/v1/onboarding/continue` advanced beyond `EMO_PERSONA_LOCK`; later onboarding actions remain read-only and out of scope in this shell."
                : "Canonical `/v1/onboarding/continue` accepted the bounded desktop emo/persona lock and returned updated onboarding posture.",
            detail: boundedNextStep == "ACCESS_PROVISION"
                ? "Read-only next-step visibility only. This exact emo/persona-lock surface preserves returned exact `ACCESS_PROVISION` plus any returned exact `voice_artifact_sync_receipt_ref`; access-provision submit remains separately gated and non-authoritative."
                : boundedNextStep == "EMO_PERSONA_LOCK"
                    ? "Read-only next-step visibility only. This exact emo/persona-lock surface preserves returned exact `EMO_PERSONA_LOCK` plus exact `voice_artifact_sync_receipt_ref`; repeated emo/persona lock submit does not widen here."
                    : advancedBeyondEmoPersonaLock
                        ? "Read-only next-step visibility only. This shell preserves the advanced step and any returned voice-artifact sync receipt without adding access provisioning, completion, sender verification, employee photo capture, wake defer, pairing completion, wake-listener behavior, or autonomous unlock."
                        : "Canonical emo/persona lock only. This shell preserves returned onboarding posture without local emo/persona authority or later onboarding controls.",
            endpoint: endpoint,
            requestID: requestID,
            outcome: boundedOnboardingContinueField(response.outcome) ?? "ONBOARDING_CONTINUED",
            reason: boundedOnboardingContinueField(response.reason),
            onboardingSessionID: boundedOnboardingContinueField(response.onboardingSessionID) ?? fallbackOnboardingSessionID,
            nextStep: boundedNextStep,
            remainingPlatformReceiptKinds: boundedOnboardingContinueList(response.remainingPlatformReceiptKinds),
            onboardingStatus: boundedOnboardingContinueField(response.onboardingStatus),
            voiceArtifactSyncReceiptRef: returnedVoiceArtifactSyncReceiptRef
        )
    }

    static func failed(
        onboardingSessionID: String,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopEmoPersonaLockRuntimeOutcomeState {
        DesktopEmoPersonaLockRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Desktop emo/persona lock failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            outcome: nil,
            reason: reason,
            onboardingSessionID: onboardingSessionID,
            nextStep: nil,
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil,
            voiceArtifactSyncReceiptRef: nil
        )
    }
}

struct DesktopAccessProvisionCommitRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let outcome: String?
    let reason: String?
    let onboardingSessionID: String?
    let nextStep: String?
    let remainingPlatformReceiptKinds: [String]
    let onboardingStatus: String?
    let voiceArtifactSyncReceiptRef: String?
    let accessEngineInstanceID: String?

    static func dispatching(
        onboardingSessionID: String,
        endpoint: String,
        requestID: String
    ) -> DesktopAccessProvisionCommitRuntimeOutcomeState {
        DesktopAccessProvisionCommitRuntimeOutcomeState(
            id: requestID,
            phase: .dispatching,
            title: "Dispatching desktop access provision commit",
            summary: "The bounded desktop access-provision-commit request is now being handed into canonical `/v1/onboarding/continue`.",
            detail: "Only exact access provision commit is in scope here. This exact surface remains explicitly non-authoritative and does not introduce completion, sender verification, employee photo capture, wake defer, pairing completion, wake-listener behavior, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            outcome: nil,
            reason: nil,
            onboardingSessionID: onboardingSessionID,
            nextStep: "ACCESS_PROVISION",
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil,
            voiceArtifactSyncReceiptRef: nil,
            accessEngineInstanceID: nil
        )
    }

    static func completed(
        requestID: String,
        endpoint: String,
        response: DesktopCanonicalRuntimeBridge.OnboardingContinueAdapterResponsePayload,
        fallbackOnboardingSessionID: String
    ) -> DesktopAccessProvisionCommitRuntimeOutcomeState {
        let boundedNextStep = boundedOnboardingContinueField(response.nextStep)
        let advancedBeyondAccessProvision = boundedNextStep != nil && boundedNextStep != "ACCESS_PROVISION"
        let returnedVoiceArtifactSyncReceiptRef = boundedOnboardingContinueField(response.voiceArtifactSyncReceiptRef)
        let returnedAccessEngineInstanceID = boundedOnboardingContinueField(response.accessEngineInstanceID)

        return DesktopAccessProvisionCommitRuntimeOutcomeState(
            id: requestID,
            phase: .completed,
            title: "Desktop access provision commit completed",
            summary: advancedBeyondAccessProvision
                ? "Canonical `/v1/onboarding/continue` advanced beyond `ACCESS_PROVISION`; later onboarding actions remain read-only and out of scope in this shell."
                : "Canonical `/v1/onboarding/continue` accepted the bounded desktop access provision commit and returned updated onboarding posture.",
            detail: boundedNextStep == "COMPLETE"
                ? "Read-only next-step visibility only. This exact access-provision surface preserves returned exact `COMPLETE`, exact `voice_artifact_sync_receipt_ref`, and exact `access_engine_instance_id`; complete submit remains separately gated and non-authoritative."
                : boundedNextStep == "ACCESS_PROVISION"
                    ? "Read-only next-step visibility only. This exact access-provision surface preserves returned exact `ACCESS_PROVISION`, exact `voice_artifact_sync_receipt_ref`, and any returned exact `access_engine_instance_id`; repeated access-provision submit does not widen here."
                    : advancedBeyondAccessProvision
                        ? "Read-only next-step visibility only. This shell preserves the advanced step, any returned voice-artifact sync receipt, and any returned access-engine instance identifier without adding completion, sender verification, employee photo capture, wake defer, pairing completion, wake-listener behavior, or autonomous unlock."
                        : "Canonical access provision only. This shell preserves returned onboarding posture without local access authority or later onboarding controls.",
            endpoint: endpoint,
            requestID: requestID,
            outcome: boundedOnboardingContinueField(response.outcome) ?? "ONBOARDING_CONTINUED",
            reason: boundedOnboardingContinueField(response.reason),
            onboardingSessionID: boundedOnboardingContinueField(response.onboardingSessionID) ?? fallbackOnboardingSessionID,
            nextStep: boundedNextStep,
            remainingPlatformReceiptKinds: boundedOnboardingContinueList(response.remainingPlatformReceiptKinds),
            onboardingStatus: boundedOnboardingContinueField(response.onboardingStatus),
            voiceArtifactSyncReceiptRef: returnedVoiceArtifactSyncReceiptRef,
            accessEngineInstanceID: returnedAccessEngineInstanceID
        )
    }

    static func failed(
        onboardingSessionID: String,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopAccessProvisionCommitRuntimeOutcomeState {
        DesktopAccessProvisionCommitRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Desktop access provision commit failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            outcome: nil,
            reason: reason,
            onboardingSessionID: onboardingSessionID,
            nextStep: nil,
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil,
            voiceArtifactSyncReceiptRef: nil,
            accessEngineInstanceID: nil
        )
    }
}

struct DesktopCompleteCommitRuntimeOutcomeState: Identifiable, Equatable {
    enum Phase: String, Equatable {
        case dispatching = "dispatching"
        case completed = "completed"
        case failed = "failed"
    }

    let id: String
    let phase: Phase
    let title: String
    let summary: String
    let detail: String
    let endpoint: String
    let requestID: String
    let outcome: String?
    let reason: String?
    let onboardingSessionID: String?
    let nextStep: String?
    let remainingPlatformReceiptKinds: [String]
    let onboardingStatus: String?
    let voiceArtifactSyncReceiptRef: String?
    let accessEngineInstanceID: String?

    static func dispatching(
        onboardingSessionID: String,
        endpoint: String,
        requestID: String
    ) -> DesktopCompleteCommitRuntimeOutcomeState {
        DesktopCompleteCommitRuntimeOutcomeState(
            id: requestID,
            phase: .dispatching,
            title: "Dispatching desktop complete commit",
            summary: "The bounded desktop complete-commit request is now being handed into canonical `/v1/onboarding/continue`.",
            detail: "Only exact complete commit is in scope here. This exact surface remains explicitly non-authoritative and does not introduce sender verification, employee photo capture, wake defer, pairing completion, wake-listener behavior, or autonomous unlock.",
            endpoint: endpoint,
            requestID: requestID,
            outcome: nil,
            reason: nil,
            onboardingSessionID: onboardingSessionID,
            nextStep: "COMPLETE",
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil,
            voiceArtifactSyncReceiptRef: nil,
            accessEngineInstanceID: nil
        )
    }

    static func completed(
        requestID: String,
        endpoint: String,
        response: DesktopCanonicalRuntimeBridge.OnboardingContinueAdapterResponsePayload,
        fallbackOnboardingSessionID: String
    ) -> DesktopCompleteCommitRuntimeOutcomeState {
        let boundedNextStep = boundedOnboardingContinueField(response.nextStep)
        let advancedBeyondComplete = boundedNextStep != nil && boundedNextStep != "COMPLETE"
        let returnedVoiceArtifactSyncReceiptRef = boundedOnboardingContinueField(response.voiceArtifactSyncReceiptRef)
        let returnedAccessEngineInstanceID = boundedOnboardingContinueField(response.accessEngineInstanceID)

        return DesktopCompleteCommitRuntimeOutcomeState(
            id: requestID,
            phase: .completed,
            title: "Desktop complete commit completed",
            summary: advancedBeyondComplete
                ? "Canonical `/v1/onboarding/continue` advanced beyond `COMPLETE`; later readiness or post-onboarding actions remain read-only and out of scope in this shell."
                : "Canonical `/v1/onboarding/continue` accepted the bounded desktop complete commit and returned updated onboarding posture.",
            detail: boundedNextStep == "READY"
                ? "Read-only next-step visibility only. This exact complete surface preserves returned exact `READY`, exact `onboarding_status`, exact `voice_artifact_sync_receipt_ref`, and exact `access_engine_instance_id`; later pairing completion or ready-time behavior remains separately gated and non-authoritative."
                : boundedNextStep == "COMPLETE"
                    ? "Read-only next-step visibility only. This exact complete surface preserves returned exact `COMPLETE`, exact `onboarding_status`, exact `voice_artifact_sync_receipt_ref`, and exact `access_engine_instance_id`; repeated complete submit does not widen here."
                    : advancedBeyondComplete
                        ? "Read-only next-step visibility only. This shell preserves the advanced step, returned onboarding status, any returned voice-artifact sync receipt, and any returned access-engine instance identifier without adding pairing completion, sender verification, employee photo capture, wake defer, wake-listener behavior, or autonomous unlock."
                        : "Canonical complete commit only. This shell preserves returned onboarding posture without local ready authority or broader onboarding authority.",
            endpoint: endpoint,
            requestID: requestID,
            outcome: boundedOnboardingContinueField(response.outcome) ?? "ONBOARDING_CONTINUED",
            reason: boundedOnboardingContinueField(response.reason),
            onboardingSessionID: boundedOnboardingContinueField(response.onboardingSessionID) ?? fallbackOnboardingSessionID,
            nextStep: boundedNextStep,
            remainingPlatformReceiptKinds: boundedOnboardingContinueList(response.remainingPlatformReceiptKinds),
            onboardingStatus: boundedOnboardingContinueField(response.onboardingStatus),
            voiceArtifactSyncReceiptRef: returnedVoiceArtifactSyncReceiptRef,
            accessEngineInstanceID: returnedAccessEngineInstanceID
        )
    }

    static func failed(
        onboardingSessionID: String,
        endpoint: String,
        requestID: String,
        summary: String,
        detail: String,
        reason: String? = nil
    ) -> DesktopCompleteCommitRuntimeOutcomeState {
        DesktopCompleteCommitRuntimeOutcomeState(
            id: requestID,
            phase: .failed,
            title: "Desktop complete commit failed",
            summary: summary,
            detail: detail,
            endpoint: endpoint,
            requestID: requestID,
            outcome: nil,
            reason: reason,
            onboardingSessionID: onboardingSessionID,
            nextStep: nil,
            remainingPlatformReceiptKinds: [],
            onboardingStatus: nil,
            voiceArtifactSyncReceiptRef: nil,
            accessEngineInstanceID: nil
        )
    }
}

private func boundedAuthoritativeResponseText(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty else {
        return nil
    }

    return trimmed
}

private func boundedAuthoritativeResponseProvenance(
    _ provenance: DesktopCanonicalRuntimeBridge.VoiceTurnProvenancePayload?
) -> AuthoritativeResponseProvenance? {
    guard let provenance else {
        return nil
    }

    let boundedSources = provenance.sources.compactMap { source -> AuthoritativeResponseProvenance.Source? in
        let title = source.title.trimmingCharacters(in: .whitespacesAndNewlines)
        let url = source.url.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !title.isEmpty, !url.isEmpty else {
            return nil
        }

        return AuthoritativeResponseProvenance.Source(title: title, url: url)
    }

    let trimmedCacheStatus = provenance.cacheStatus?.trimmingCharacters(in: .whitespacesAndNewlines)

    guard !boundedSources.isEmpty || provenance.retrievedAt != nil || !(trimmedCacheStatus ?? "").isEmpty else {
        return nil
    }

    return AuthoritativeResponseProvenance(
        sources: boundedSources,
        retrievedAt: provenance.retrievedAt,
        cacheStatus: (trimmedCacheStatus?.isEmpty == false) ? trimmedCacheStatus : nil
    )
}

private func boundedInviteOpenField(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty, trimmed.count <= 128, !trimmed.contains("\n"), !trimmed.contains("\r") else {
        return nil
    }

    return trimmed
}

private func boundedInviteOpenList(_ rawValues: [String]) -> [String] {
    Array(
        rawValues
            .compactMap { boundedInviteOpenField($0) }
            .prefix(12)
    )
}

private func boundedOnboardingContinueField(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty, trimmed.count <= 256, !trimmed.contains("\n"), !trimmed.contains("\r") else {
        return nil
    }

    return trimmed
}

private func boundedOnboardingContinueList(_ rawValues: [String]) -> [String] {
    Array(
        rawValues
            .compactMap { boundedOnboardingContinueField($0) }
            .prefix(12)
    )
}

private func boundedOnboardingContinueFieldInput(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty, trimmed.count <= 256, !trimmed.contains("\n"), !trimmed.contains("\r") else {
        return nil
    }

    return trimmed
}

private func boundedDesktopVoiceEnrollTranscriptPreview(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty,
          trimmed.utf8.count <= 4_096,
          !trimmed.contains("\n"),
          !trimmed.contains("\r") else {
        return nil
    }

    return trimmed
}

let desktopCanonicalTermsVersionID = "terms_v1"
let desktopWakeEnrollDeferCommitAction = ["WAKE", "ENROLL", "DEFER", "COMMIT"].joined(separator: "_")
let desktopWakeEnrollCompleteCommitAction = ["WAKE", "ENROLL", "COMPLETE", "COMMIT"].joined(separator: "_")
let desktopEmoPersonaLockAction = ["EMO", "PERSONA", "LOCK"].joined(separator: "_")
let desktopAccessProvisionCommitAction = ["ACCESS", "PROVISION", "COMMIT"].joined(separator: "_")
let desktopCompleteCommitAction = ["COMPLETE", "COMMIT"].joined(separator: "_")

private let supportedDesktopPlatformSetupReceiptKinds: Set<String> = [
    "install_launch_handshake",
    "mic_permission_granted",
    "desktop_pairing_bound",
    "desktop_wakeword_configured",
]

private func boundedSupportedDesktopPlatformSetupReceiptKind(_ rawValue: String?) -> String? {
    guard let boundedReceiptKind = boundedOnboardingContinueField(rawValue),
          supportedDesktopPlatformSetupReceiptKinds.contains(boundedReceiptKind) else {
        return nil
    }

    return boundedReceiptKind
}

private func boundedDesktopPlatformSetupReceiptProofMaterial(_ rawValue: String?) -> String? {
    guard let rawValue else {
        return nil
    }

    let trimmed = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty,
          trimmed.count <= 512,
          trimmed.unicodeScalars.allSatisfy(\.isASCII),
          !trimmed.contains("\n"),
          !trimmed.contains("\r") else {
        return nil
    }

    return trimmed
}

private func desktopPlatformSetupReceiptPayloadHash(_ seed: String) -> String {
    SHA256.hash(data: Data(seed.utf8)).map { String(format: "%02x", $0) }.joined()
}

private func desktopPlatformSetupReceiptRef(receiptKind: String, payloadHash: String) -> String {
    "receipt:desktop-local:\(receiptKind):\(payloadHash.prefix(16))"
}

final class DesktopCanonicalRuntimeBridge: ObservableObject {
    private enum BridgeError: LocalizedError {
        case invalidPreparedRequest(String)
        case invalidOnboardingContinueRequest(String)
        case invalidPlatformSetupReceiptRequest(String)
        case invalidTermsAcceptRequest(String)
        case invalidEmployeePhotoCaptureSendRequest(String)
        case invalidEmployeeSenderVerifyCommitRequest(String)
        case invalidPrimaryDeviceConfirmRequest(String)
        case invalidVoiceEnrollRequest(String)
        case invalidWakeEnrollStartDraftRequest(String)
        case invalidWakeEnrollSampleCommitRequest(String)
        case invalidWakeEnrollCompleteCommitRequest(String)
        case invalidWakeEnrollDeferCommitRequest(String)
        case invalidSessionAttachRequest(String)
        case invalidSessionMultiPostureResumeRequest(String)
        case invalidSessionSoftClosedResumeRequest(String)
        case invalidSessionRecoverRequest(String)
        case invalidWakeProfileAvailabilityRequest(String)
        case invalidEmoPersonaLockRequest(String)
        case invalidAccessProvisionCommitRequest(String)
        case invalidCompleteCommitRequest(String)
        case invalidAdapterBind(String)
        case adapterStartFailed(String)
        case adapterUnavailable(String)
        case requestEncodingFailed(String)
        case responseDecodingFailed(String)
        case transportFailed(String)

        var errorDescription: String? {
            switch self {
            case .invalidPreparedRequest(let detail),
                 .invalidOnboardingContinueRequest(let detail),
                 .invalidPlatformSetupReceiptRequest(let detail),
                 .invalidTermsAcceptRequest(let detail),
                 .invalidEmployeePhotoCaptureSendRequest(let detail),
                 .invalidEmployeeSenderVerifyCommitRequest(let detail),
                 .invalidPrimaryDeviceConfirmRequest(let detail),
                 .invalidVoiceEnrollRequest(let detail),
                 .invalidWakeEnrollStartDraftRequest(let detail),
                 .invalidWakeEnrollSampleCommitRequest(let detail),
                 .invalidWakeEnrollCompleteCommitRequest(let detail),
                 .invalidWakeEnrollDeferCommitRequest(let detail),
                 .invalidSessionAttachRequest(let detail),
                 .invalidSessionMultiPostureResumeRequest(let detail),
                 .invalidSessionSoftClosedResumeRequest(let detail),
                 .invalidSessionRecoverRequest(let detail),
                 .invalidWakeProfileAvailabilityRequest(let detail),
                 .invalidEmoPersonaLockRequest(let detail),
                 .invalidAccessProvisionCommitRequest(let detail),
                 .invalidCompleteCommitRequest(let detail),
                 .invalidAdapterBind(let detail),
                 .adapterStartFailed(let detail),
                 .adapterUnavailable(let detail),
                 .requestEncodingFailed(let detail),
                 .responseDecodingFailed(let detail),
                 .transportFailed(let detail):
                return detail
            }
        }
    }

    struct DesktopExplicitVoiceIngressContext {
        let preparedRequestID: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopWakeTriggeredVoiceIngressContext {
        let preparedRequestID: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopInviteOpenIngressContext {
        let entryContextID: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopOnboardingContinueIngressContext {
        let onboardingSessionID: String
        let blockingField: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopPlatformSetupReceiptIngressContext {
        let onboardingSessionID: String
        let receiptKind: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopTermsAcceptIngressContext {
        let onboardingSessionID: String
        let termsVersionID: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopEmployeePhotoCaptureSendIngressContext {
        let onboardingSessionID: String
        let photoBlobRef: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopEmployeeSenderVerifyCommitIngressContext {
        let onboardingSessionID: String
        let senderDecision: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopPrimaryDeviceConfirmIngressContext {
        let onboardingSessionID: String
        let deviceID: String
        let proofOK: Bool
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopVoiceEnrollIngressContext {
        let onboardingSessionID: String
        let deviceID: String
        let sampleSeed: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopWakeEnrollStartDraftIngressContext {
        let onboardingSessionID: String
        let deviceID: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopWakeEnrollSampleCommitIngressContext {
        let onboardingSessionID: String
        let deviceID: String
        let proofOK: Bool
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopWakeEnrollCompleteCommitIngressContext {
        let onboardingSessionID: String
        let deviceID: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopWakeEnrollDeferCommitIngressContext {
        let onboardingSessionID: String
        let deviceID: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopSessionAttachIngressContext {
        let sourceSurfaceIdentity: String
        let sessionState: String
        let sessionID: String
        let currentVisibleSessionAttachOutcome: String?
        let turnID: String?
        let deviceID: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopSessionSoftClosedResumeIngressContext {
        let sourceSurfaceIdentity: String
        let sessionState: String
        let sessionID: String
        let selectedThreadID: String?
        let selectedThreadTitle: String?
        let pendingWorkOrderID: String?
        let resumeTier: String?
        let resumeSummaryBullets: [String]
        let deviceID: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopSessionMultiPostureResumeIngressContext {
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
        let requestID: String
        let endpoint: String
        fileprivate let softClosedResumeIngressContext: DesktopSessionSoftClosedResumeIngressContext?
        fileprivate let sessionRecoverIngressContext: DesktopSessionRecoverIngressContext?
    }

    struct DesktopSessionRecoverIngressContext {
        let sourceSurfaceIdentity: String
        let sessionState: String
        let sessionID: String
        let recoveryMode: String?
        let deviceID: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopWakeProfileAvailabilityIngressContext {
        let receiptKind: String
        let deviceID: String
        let wakeProfileID: String
        let voiceArtifactSyncReceiptRef: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopEmoPersonaLockIngressContext {
        let onboardingSessionID: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopAccessProvisionCommitIngressContext {
        let onboardingSessionID: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct DesktopCompleteCommitIngressContext {
        let onboardingSessionID: String
        let requestID: String
        let endpoint: String
        let urlRequest: URLRequest
    }

    struct VoiceTurnProvenanceSourcePayload: Decodable {
        let title: String
        let url: String
    }

    struct VoiceTurnProvenancePayload: Decodable {
        let sources: [VoiceTurnProvenanceSourcePayload]
        let retrievedAt: UInt64?
        let cacheStatus: String?
    }

    struct VoiceTurnAdapterResponsePayload: Decodable {
        let status: String
        let outcome: String
        let sessionID: String?
        let turnID: UInt64?
        let sessionState: String?
        let sessionAttachOutcome: String?
        let failureClass: String?
        let reason: String?
        let nextMove: String
        let responseText: String
        let reasonCode: String
        let provenance: VoiceTurnProvenancePayload?
    }

    struct InviteLinkOpenAdapterResponsePayload: Decodable {
        let status: String
        let outcome: String
        let reason: String?
        let onboardingSessionID: String?
        let nextStep: String?
        let requiredFields: [String]
        let requiredVerificationGates: [String]
    }

    struct OnboardingContinueAdapterResponsePayload: Decodable {
        let status: String
        let outcome: String
        let reason: String?
        let onboardingSessionID: String?
        let nextStep: String?
        let blockingField: String?
        let blockingQuestion: String?
        let remainingMissingFields: [String]
        let remainingPlatformReceiptKinds: [String]
        let voiceArtifactSyncReceiptRef: String?
        let accessEngineInstanceID: String?
        let onboardingStatus: String?
    }

    struct SessionResumeAdapterResponsePayload: Decodable {
        let status: String
        let outcome: String
        let reason: String?
        let sessionID: String?
        let sessionState: String?
        let sessionAttachOutcome: String?
    }

    struct SessionAttachAdapterResponsePayload: Decodable {
        let status: String
        let outcome: String
        let reason: String?
        let sessionID: String?
        let sessionState: String?
        let sessionAttachOutcome: String?
    }

    struct SessionRecoverAdapterResponsePayload: Decodable {
        let status: String
        let outcome: String
        let reason: String?
        let sessionID: String?
        let sessionState: String?
        let sessionAttachOutcome: String?
    }

    struct WakeProfileAvailabilityRefreshAdapterResponsePayload: Decodable {
        let status: String
        let outcome: String
        let reason: String?
        let deviceID: String?
        let wakeProfileID: String?
        let activeWakeArtifactVersion: String?
        let activatedCount: UInt64
        let noopCount: UInt64
        let pullErrorCount: UInt64
    }

    private struct VoiceTurnIngressErrorPayload: Decodable {
        let failureClass: String
        let reasonCode: String
        let reason: String?
        let sessionID: String?
        let turnID: UInt64?
        let sessionState: String?
    }

    private struct VoiceTurnAdapterRequestPayload: Encodable {
        let correlationID: UInt64
        let turnID: UInt64
        let deviceTurnSequence: UInt64?
        let appPlatform: String
        let platformVersion: String?
        let deviceClass: String?
        let runtimeClientVersion: String?
        let hardwareCapabilityProfile: String?
        let networkProfile: String?
        let claimedCapabilities: [String]?
        let integrityStatus: String?
        let attestationRef: String?
        let trigger: String
        let actorUserID: String
        let tenantID: String?
        let deviceID: String?
        let nowNS: UInt64?
        let threadKey: String?
        let projectID: String?
        let pinnedContextRefs: [String]?
        let threadPolicyFlags: VoiceTurnThreadPolicyFlagsPayload?
        let userTextPartial: String?
        let userTextFinal: String?
        let seleneTextPartial: String?
        let seleneTextFinal: String?
        let audioCaptureRef: DesktopVoiceTurnAudioCaptureRefPayload?
        let visualInputRef: String?
    }

    struct DesktopVoiceTurnAudioCaptureRefPayload: Encodable {
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

        enum CodingKeys: String, CodingKey {
            case streamID = "stream_id"
            case preRollBufferID = "pre_roll_buffer_id"
            case tStartNS = "t_start_ns"
            case tEndNS = "t_end_ns"
            case tCandidateStartNS = "t_candidate_start_ns"
            case tConfirmedNS = "t_confirmed_ns"
            case localeTag = "locale_tag"
            case deviceRoute = "device_route"
            case selectedMic = "selected_mic"
            case selectedSpeaker = "selected_speaker"
            case ttsPlaybackActive = "tts_playback_active"
            case detectionText = "detection_text"
            case detectionConfidenceBP = "detection_confidence_bp"
            case vadConfidenceBP = "vad_confidence_bp"
            case acousticConfidenceBP = "acoustic_confidence_bp"
            case prosodyConfidenceBP = "prosody_confidence_bp"
            case speechLikenessBP = "speech_likeness_bp"
            case echoSafeConfidenceBP = "echo_safe_confidence_bp"
            case nearfieldConfidenceBP = "nearfield_confidence_bp"
            case captureDegraded = "capture_degraded"
            case streamGapDetected = "stream_gap_detected"
            case aecUnstable = "aec_unstable"
            case deviceChanged = "device_changed"
            case snrDBMilli = "snr_db_milli"
            case clippingRatioBP = "clipping_ratio_bp"
            case echoDelayMSMilli = "echo_delay_ms_milli"
            case packetLossBP = "packet_loss_bp"
            case doubleTalkBP = "double_talk_bp"
            case erleDBMilli = "erle_db_milli"
            case deviceFailures24H = "device_failures_24h"
            case deviceRecoveries24H = "device_recoveries_24h"
            case deviceMeanRecoveryMS = "device_mean_recovery_ms"
            case deviceReliabilityBP = "device_reliability_bp"
            case timingJitterMSMilli = "timing_jitter_ms_milli"
            case timingDriftPPMMilli = "timing_drift_ppm_milli"
            case timingBufferDepthMSMilli = "timing_buffer_depth_ms_milli"
            case timingUnderruns = "timing_underruns"
            case timingOverruns = "timing_overruns"
        }
    }

    private struct InviteLinkOpenAdapterRequestPayload: Encodable {
        let correlationID: UInt64
        let idempotencyKey: String
        let tokenID: String
        let tokenSignature: String
        let tenantID: String?
        let appPlatform: String
        let deviceFingerprint: String
        let appInstanceID: String
        let deepLinkNonce: String
    }

    private struct OnboardingContinueAdapterRequestPayload: Encodable {
        let correlationID: UInt64
        let onboardingSessionID: String
        let idempotencyKey: String
        let tenantID: String?
        let action: String
        let fieldValue: String?
        let receiptKind: String?
        let receiptRef: String?
        let signer: String?
        let payloadHash: String?
        let termsVersionID: String?
        let accepted: Bool?
        let deviceID: String?
        let proofOK: Bool?
        let sampleSeed: String?
        let photoBlobRef: String?
        let senderDecision: String?
    }

    private struct SessionResumeAdapterRequestPayload: Encodable {
        let correlationID: UInt64
        let idempotencyKey: String
        let sessionID: String
        let deviceID: String
    }

    private struct SessionAttachAdapterRequestPayload: Encodable {
        let correlationID: UInt64
        let idempotencyKey: String
        let sessionID: String
        let deviceID: String
    }

    private struct SessionRecoverAdapterRequestPayload: Encodable {
        let correlationID: UInt64
        let idempotencyKey: String
        let sessionID: String
        let deviceID: String
    }

    private struct WakeProfileAvailabilityRefreshAdapterRequestPayload: Encodable {
        let correlationID: UInt64
        let idempotencyKey: String
        let deviceID: String
        let expectedWakeProfileID: String
        let voiceArtifactSyncReceiptRef: String
    }

    private struct VoiceTurnThreadPolicyFlagsPayload: Encodable {
        let privacyMode: Bool
        let doNotDisturb: Bool
        let strictSafety: Bool
    }

    private let adapterBaseURL: URL
    private let repoRootURL: URL
    private let actorUserID: String
    private let tenantID: String?
    private let deviceID: String
    private let urlSession: URLSession
    private var managedAdapterProcess: Process?

    init(processInfo: ProcessInfo = .processInfo) {
        self.repoRootURL = Self.resolveRepoRoot(processInfo: processInfo)
        self.adapterBaseURL = Self.resolveAdapterBaseURL(processInfo: processInfo)
        self.actorUserID = Self.resolveActorUserID(processInfo: processInfo)
        self.tenantID = Self.resolveTenantID(processInfo: processInfo)
        self.deviceID = Self.resolveDeviceID(processInfo: processInfo)

        let configuration = URLSessionConfiguration.ephemeral
        configuration.timeoutIntervalForRequest = 15
        configuration.timeoutIntervalForResource = 15
        self.urlSession = URLSession(configuration: configuration)
    }

    deinit {
        stopManagedAdapter()
    }

    func stopManagedAdapter() {
        guard let managedAdapterProcess else {
            return
        }

        if managedAdapterProcess.isRunning {
            managedAdapterProcess.terminate()
        }

        self.managedAdapterProcess = nil
    }

    func dispatchPreparedExplicitVoiceRequest(
        _ preparedRequest: ExplicitVoiceTurnRequestState
    ) async -> DesktopCanonicalRuntimeOutcomeState {
        do {
            let ingressContext = try desktopExplicitVoiceIngressRequestBuilder(preparedRequest)
            return await dispatchPreparedExplicitVoiceRequest(ingressContext)
        } catch {
            return .failed(
                preparedRequestID: preparedRequest.id,
                endpoint: voiceTurnEndpoint,
                requestID: "unavailable",
                summary: "The canonical runtime bridge could not deliver the bounded explicit voice request.",
                detail: error.localizedDescription,
                reasonCode: "desktop_runtime_bridge_failure",
                failureClass: "RetryableRuntime"
            )
        }
    }

    func dispatchPreparedWakeTriggeredVoiceRequest(
        _ preparedRequest: WakeTriggeredVoiceTurnRequestState
    ) async -> DesktopCanonicalRuntimeOutcomeState {
        do {
            let ingressContext = try desktopWakeTriggeredVoiceIngressRequestBuilder(preparedRequest)
            return await dispatchPreparedWakeTriggeredVoiceRequest(ingressContext)
        } catch {
            return .failedWake(
                preparedRequestID: preparedRequest.id,
                endpoint: voiceTurnEndpoint,
                requestID: "unavailable",
                summary: "The canonical runtime bridge could not deliver the bounded wake-triggered voice request.",
                detail: error.localizedDescription,
                reasonCode: "desktop_runtime_bridge_failure",
                failureClass: "RetryableRuntime"
            )
        }
    }

    func openInviteLinkAndStartOnboarding(
        _ onboardingEntryContext: DesktopOnboardingEntryContext
    ) async -> DesktopInviteOpenRuntimeOutcomeState {
        do {
            let ingressContext = try desktopInviteClickRequestBuilder(onboardingEntryContext)
            return await openInviteLinkAndStartOnboarding(ingressContext)
        } catch {
            return .failed(
                entryContextID: onboardingEntryContext.id,
                endpoint: inviteClickEndpoint,
                requestID: "unavailable",
                summary: "The canonical invite-open bridge could not stage this onboarding-entry request.",
                detail: error.localizedDescription
            )
        }
    }

    func continueOnboardingMissingField(
        promptState: DesktopOnboardingContinuePromptState,
        fieldValue: String?
    ) async -> DesktopOnboardingContinueRuntimeOutcomeState {
        do {
            let ingressContext = try desktopOnboardingContinueMissingFieldRequestBuilder(
                promptState: promptState,
                fieldValue: fieldValue
            )
            return await continueOnboardingMissingField(ingressContext)
        } catch {
            return .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                blockingField: promptState.blockingField,
                endpoint: onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded missing-field request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopPlatformSetupReceipt(
        _ draft: DesktopPlatformSetupReceiptDraft
    ) async -> DesktopPlatformSetupReceiptRuntimeOutcomeState {
        do {
            let ingressContext = try desktopPlatformSetupReceiptRequestBuilder(draft)
            return await submitDesktopPlatformSetupReceipt(ingressContext)
        } catch {
            return .failed(
                onboardingSessionID: draft.onboardingSessionID,
                receiptKind: draft.receiptKind,
                endpoint: onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop platform-setup receipt.",
                detail: error.localizedDescription
            )
        }
    }

    func acceptDesktopTerms(
        _ promptState: DesktopTermsAcceptPromptState
    ) async -> DesktopTermsAcceptRuntimeOutcomeState {
        do {
            let ingressContext = try desktopTermsAcceptRequestBuilder(promptState)
            return await acceptDesktopTerms(ingressContext)
        } catch {
            return .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                termsVersionID: promptState.termsVersionID,
                endpoint: onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop terms acceptance request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopEmployeePhotoCaptureSend(
        promptState: DesktopEmployeePhotoCaptureSendPromptState,
        photoBlobRef: String
    ) async -> DesktopEmployeePhotoCaptureSendRuntimeOutcomeState {
        do {
            let ingressContext = try desktopEmployeePhotoCaptureSendRequestBuilder(
                promptState: promptState,
                photoBlobRef: photoBlobRef
            )
            return await submitDesktopEmployeePhotoCaptureSend(ingressContext)
        } catch {
            return .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                photoBlobRef: boundedOnboardingContinueFieldInput(photoBlobRef),
                endpoint: onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded employee photo capture send request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopEmployeeSenderVerifyCommit(
        promptState: DesktopEmployeeSenderVerifyCommitPromptState,
        senderDecision: String
    ) async -> DesktopEmployeeSenderVerifyCommitRuntimeOutcomeState {
        do {
            let ingressContext = try desktopEmployeeSenderVerifyCommitRequestBuilder(
                promptState: promptState,
                senderDecision: senderDecision
            )
            return await submitDesktopEmployeeSenderVerifyCommit(ingressContext)
        } catch {
            return .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                senderDecision: boundedOnboardingContinueFieldInput(senderDecision),
                endpoint: onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded sender verification commit request.",
                detail: error.localizedDescription
            )
        }
    }

    func confirmDesktopPrimaryDevice(
        _ promptState: DesktopPrimaryDeviceConfirmPromptState
    ) async -> DesktopPrimaryDeviceConfirmRuntimeOutcomeState {
        do {
            let ingressContext = try desktopPrimaryDeviceConfirmRequestBuilder(promptState)
            return await confirmDesktopPrimaryDevice(ingressContext)
        } catch {
            return .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                deviceID: promptState.deviceID,
                endpoint: onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop primary-device confirmation request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopVoiceEnrollLock(
        _ promptState: DesktopVoiceEnrollPromptState
    ) async -> DesktopVoiceEnrollRuntimeOutcomeState {
        do {
            let ingressContext = try desktopVoiceEnrollRequestBuilder(promptState)
            return await submitDesktopVoiceEnrollLock(ingressContext)
        } catch {
            return .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                deviceID: promptState.deviceID,
                sampleSeed: nil,
                endpoint: onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop voice-enroll lock request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopWakeEnrollStartDraft(
        _ promptState: DesktopWakeEnrollStartDraftPromptState
    ) async -> DesktopWakeEnrollStartDraftRuntimeOutcomeState {
        do {
            let ingressContext = try desktopWakeEnrollStartDraftRequestBuilder(promptState)
            return await submitDesktopWakeEnrollStartDraft(ingressContext)
        } catch {
            return .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                deviceID: promptState.deviceID,
                endpoint: onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop wake-enroll start-draft request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopWakeEnrollSampleCommit(
        _ promptState: DesktopWakeEnrollSampleCommitPromptState
    ) async -> DesktopWakeEnrollSampleCommitRuntimeOutcomeState {
        do {
            let ingressContext = try desktopWakeEnrollSampleCommitRequestBuilder(promptState)
            return await submitDesktopWakeEnrollSampleCommit(ingressContext)
        } catch {
            return .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                deviceID: promptState.deviceID,
                endpoint: onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop wake-enroll sample-commit request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopWakeEnrollCompleteCommit(
        _ promptState: DesktopWakeEnrollCompleteCommitPromptState
    ) async -> DesktopWakeEnrollCompleteCommitRuntimeOutcomeState {
        do {
            let ingressContext = try desktopWakeEnrollCompleteCommitRequestBuilder(promptState)
            return await submitDesktopWakeEnrollCompleteCommit(ingressContext)
        } catch {
            return .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                deviceID: promptState.deviceID,
                endpoint: onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop wake-enroll complete-commit request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopWakeEnrollDeferCommit(
        _ promptState: DesktopWakeEnrollDeferCommitPromptState
    ) async -> DesktopWakeEnrollDeferCommitRuntimeOutcomeState {
        do {
            let ingressContext = try desktopWakeEnrollDeferCommitRequestBuilder(promptState)
            return await submitDesktopWakeEnrollDeferCommit(ingressContext)
        } catch {
            return .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                deviceID: promptState.deviceID,
                endpoint: onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop wake-enroll defer-commit request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopSessionAttach(
        _ promptState: DesktopSessionAttachPromptState
    ) async -> DesktopSessionAttachRuntimeOutcomeState {
        do {
            let ingressContext = try desktopSessionAttachRequestBuilder(promptState)
            return await submitDesktopSessionAttach(ingressContext)
        } catch {
            return .failed(
                sourceSurfaceIdentity: promptState.sourceSurfaceIdentity,
                sessionState: promptState.sessionState,
                sessionID: promptState.sessionID,
                currentVisibleSessionAttachOutcome: promptState.currentVisibleSessionAttachOutcome,
                turnID: promptState.turnID,
                deviceID: promptState.deviceID,
                endpoint: sessionAttachEndpoint,
                requestID: "unavailable",
                summary: "The canonical session-attach bridge could not stage this bounded desktop current-visible session attach request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopSessionSoftClosedResume(
        _ promptState: DesktopSessionSoftClosedResumePromptState
    ) async -> DesktopSessionSoftClosedResumeRuntimeOutcomeState {
        do {
            let ingressContext = try desktopSessionSoftClosedResumeRequestBuilder(promptState)
            return await submitDesktopSessionSoftClosedResume(ingressContext)
        } catch {
            return .failed(
                sourceSurfaceIdentity: promptState.sourceSurfaceIdentity,
                sessionState: promptState.sessionState,
                sessionID: promptState.sessionID,
                selectedThreadID: promptState.selectedThreadID,
                selectedThreadTitle: promptState.selectedThreadTitle,
                pendingWorkOrderID: promptState.pendingWorkOrderID,
                resumeTier: promptState.resumeTier,
                resumeSummaryBullets: promptState.resumeSummaryBullets,
                deviceID: promptState.deviceID,
                endpoint: sessionResumeEndpoint,
                requestID: "unavailable",
                summary: "The canonical session-resume bridge could not stage this bounded desktop soft-closed explicit resume request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopSessionRecover(
        _ promptState: DesktopSessionRecoverPromptState
    ) async -> DesktopSessionRecoverRuntimeOutcomeState {
        do {
            let ingressContext = try desktopSessionRecoverRequestBuilder(promptState)
            return await submitDesktopSessionRecover(ingressContext)
        } catch {
            return .failed(
                sourceSurfaceIdentity: promptState.sourceSurfaceIdentity,
                sessionState: promptState.sessionState,
                sessionID: promptState.sessionID,
                recoveryMode: promptState.recoveryMode,
                deviceID: promptState.deviceID,
                endpoint: sessionRecoverEndpoint,
                requestID: "unavailable",
                summary: "The canonical session-recover bridge could not stage this bounded desktop suspended-session recover request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopSessionMultiPostureResume(
        _ promptState: DesktopSessionMultiPostureResumePromptState
    ) async -> DesktopSessionMultiPostureResumeRuntimeOutcomeState {
        do {
            let ingressContext = try desktopSessionMultiPostureResumeRequestBuilder(promptState)
            return await submitDesktopSessionMultiPostureResume(ingressContext)
        } catch {
            return .failed(
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
                    ? sessionResumeEndpoint
                    : sessionRecoverEndpoint,
                requestID: "unavailable",
                summary: "The canonical multi-posture session-resume bridge could not stage this bounded desktop session-resume request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopWakeProfileAvailabilityRefresh(
        _ promptState: DesktopWakeProfileAvailabilityPromptState
    ) async -> DesktopWakeProfileAvailabilityRuntimeOutcomeState {
        do {
            let ingressContext = try desktopWakeProfileAvailabilityRequestBuilder(promptState)
            return await submitDesktopWakeProfileAvailabilityRefresh(ingressContext)
        } catch {
            return .failed(
                receiptKind: promptState.receiptKind,
                deviceID: promptState.deviceID,
                wakeProfileID: promptState.wakeProfileID,
                voiceArtifactSyncReceiptRef: promptState.voiceArtifactSyncReceiptRef,
                endpoint: wakeProfileAvailabilityEndpoint,
                requestID: "unavailable",
                summary: "The canonical wake-profile availability bridge could not stage this bounded desktop wake-profile local-availability refresh request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopEmoPersonaLock(
        _ promptState: DesktopEmoPersonaLockPromptState
    ) async -> DesktopEmoPersonaLockRuntimeOutcomeState {
        do {
            let ingressContext = try desktopEmoPersonaLockRequestBuilder(promptState)
            return await submitDesktopEmoPersonaLock(ingressContext)
        } catch {
            return .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                endpoint: onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop emo/persona-lock request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopAccessProvisionCommit(
        _ promptState: DesktopAccessProvisionCommitPromptState
    ) async -> DesktopAccessProvisionCommitRuntimeOutcomeState {
        do {
            let ingressContext = try desktopAccessProvisionCommitRequestBuilder(promptState)
            return await submitDesktopAccessProvisionCommit(ingressContext)
        } catch {
            return .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                endpoint: onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop access-provision request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopCompleteCommit(
        _ promptState: DesktopCompleteCommitPromptState
    ) async -> DesktopCompleteCommitRuntimeOutcomeState {
        do {
            let ingressContext = try desktopCompleteCommitRequestBuilder(promptState)
            return await submitDesktopCompleteCommit(ingressContext)
        } catch {
            return .failed(
                onboardingSessionID: promptState.onboardingSessionID,
                endpoint: onboardingContinueEndpoint,
                requestID: "unavailable",
                summary: "The canonical onboarding-continue bridge could not stage this bounded desktop complete request.",
                detail: error.localizedDescription
            )
        }
    }

    func dispatchPreparedExplicitVoiceRequest(
        _ ingressContext: DesktopExplicitVoiceIngressContext
    ) async -> DesktopCanonicalRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0

            if statusCode == 200 {
                let payload = try decoder.decode(VoiceTurnAdapterResponsePayload.self, from: data)
                return .completed(
                    preparedRequestID: ingressContext.preparedRequestID,
                    endpoint: ingressContext.endpoint,
                    requestID: ingressContext.requestID,
                    response: payload
                )
            }

            if let payload = try? decoder.decode(VoiceTurnIngressErrorPayload.self, from: data) {
                return .failed(
                    preparedRequestID: ingressContext.preparedRequestID,
                    endpoint: ingressContext.endpoint,
                    requestID: ingressContext.requestID,
                    summary: "The canonical runtime rejected or failed the bounded explicit voice request before reply rendering was allowed.",
                    detail: "Canonical dispatch failed closed with reason code `\(payload.reasonCode)` and failure class `\(payload.failureClass)`. This shell does not fabricate local assistant output or bypass runtime law.",
                    reasonCode: payload.reasonCode,
                    failureClass: payload.failureClass,
                    sessionID: payload.sessionID,
                    turnID: payload.turnID.map(String.init)
                )
            }

            throw BridgeError.responseDecodingFailed(
                "canonical runtime bridge returned status \(statusCode) with an unreadable response payload"
            )
        } catch {
            return .failed(
                preparedRequestID: ingressContext.preparedRequestID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical runtime bridge could not deliver the bounded explicit voice request.",
                detail: error.localizedDescription,
                reasonCode: "desktop_runtime_bridge_failure",
                failureClass: "RetryableRuntime"
            )
        }
    }

    func dispatchPreparedWakeTriggeredVoiceRequest(
        _ ingressContext: DesktopWakeTriggeredVoiceIngressContext
    ) async -> DesktopCanonicalRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0

            if statusCode == 200 {
                let payload = try decoder.decode(VoiceTurnAdapterResponsePayload.self, from: data)
                return .completedWake(
                    preparedRequestID: ingressContext.preparedRequestID,
                    endpoint: ingressContext.endpoint,
                    requestID: ingressContext.requestID,
                    response: payload
                )
            }

            if let payload = try? decoder.decode(VoiceTurnIngressErrorPayload.self, from: data) {
                return .failedWake(
                    preparedRequestID: ingressContext.preparedRequestID,
                    endpoint: ingressContext.endpoint,
                    requestID: ingressContext.requestID,
                    summary: "The canonical runtime rejected or failed the bounded wake-triggered voice request before reply rendering was allowed.",
                    detail: "Canonical wake-triggered dispatch failed closed with reason code `\(payload.reasonCode)` and failure class `\(payload.failureClass)`. This shell does not fabricate local assistant output or bypass runtime law.",
                    reasonCode: payload.reasonCode,
                    failureClass: payload.failureClass,
                    sessionID: payload.sessionID,
                    turnID: payload.turnID.map(String.init)
                )
            }

            throw BridgeError.responseDecodingFailed(
                "canonical runtime bridge returned status \(statusCode) with an unreadable response payload"
            )
        } catch {
            return .failedWake(
                preparedRequestID: ingressContext.preparedRequestID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical runtime bridge could not deliver the bounded wake-triggered voice request.",
                detail: error.localizedDescription,
                reasonCode: "desktop_runtime_bridge_failure",
                failureClass: "RetryableRuntime"
            )
        }
    }

    func openInviteLinkAndStartOnboarding(
        _ ingressContext: DesktopInviteOpenIngressContext
    ) async -> DesktopInviteOpenRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(InviteLinkOpenAdapterResponsePayload.self, from: data)

            if statusCode == 200, payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    entryContextID: ingressContext.entryContextID,
                    endpoint: ingressContext.endpoint,
                    requestID: ingressContext.requestID,
                    outcome: boundedInviteOpenField(payload.outcome) ?? "ONBOARDING_STARTED",
                    reason: boundedInviteOpenField(payload.reason),
                    onboardingSessionID: boundedInviteOpenField(payload.onboardingSessionID),
                    nextStep: boundedInviteOpenField(payload.nextStep),
                    requiredFields: boundedInviteOpenList(payload.requiredFields),
                    requiredVerificationGates: boundedInviteOpenList(payload.requiredVerificationGates)
                )
            }

            return .failed(
                entryContextID: ingressContext.entryContextID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical invite-open bridge rejected or failed this onboarding-entry request before onboarding-continue mutation was allowed.",
                detail: "Canonical `/v1/invite/click` failed closed with outcome `\(payload.outcome)` and reason `\(boundedInviteOpenField(payload.reason) ?? "not_provided")`. This shell remains read-only and does not bypass onboarding law.",
                reason: boundedInviteOpenField(payload.reason)
            )
        } catch {
            return .failed(
                entryContextID: ingressContext.entryContextID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical invite-open bridge could not deliver this onboarding-entry request.",
                detail: error.localizedDescription
            )
        }
    }

    func continueOnboardingMissingField(
        _ ingressContext: DesktopOnboardingContinueIngressContext
    ) async -> DesktopOnboardingContinueRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(OnboardingContinueAdapterResponsePayload.self, from: data)

            if statusCode == 200,
               payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    requestID: ingressContext.requestID,
                    endpoint: ingressContext.endpoint,
                    response: payload,
                    fallbackOnboardingSessionID: ingressContext.onboardingSessionID,
                    fallbackBlockingField: ingressContext.blockingField
                )
            }

            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                blockingField: ingressContext.blockingField,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge rejected or failed this bounded missing-field request before later onboarding actions were allowed.",
                detail: "Canonical `/v1/onboarding/continue` failed closed with outcome `\(payload.outcome)` and reason `\(boundedOnboardingContinueField(payload.reason) ?? "not_provided")`. This shell remains limited to the exact `ASK_MISSING_SUBMIT` slice and does not bypass onboarding law.",
                reason: boundedOnboardingContinueField(payload.reason)
            )
        } catch {
            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                blockingField: ingressContext.blockingField,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge could not deliver this bounded missing-field request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopPlatformSetupReceipt(
        _ ingressContext: DesktopPlatformSetupReceiptIngressContext
    ) async -> DesktopPlatformSetupReceiptRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(OnboardingContinueAdapterResponsePayload.self, from: data)

            if statusCode == 200,
               payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    requestID: ingressContext.requestID,
                    endpoint: ingressContext.endpoint,
                    response: payload,
                    fallbackOnboardingSessionID: ingressContext.onboardingSessionID,
                    fallbackReceiptKind: ingressContext.receiptKind
                )
            }

            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                receiptKind: ingressContext.receiptKind,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge rejected or failed this bounded desktop platform-setup receipt.",
                detail: "Canonical `/v1/onboarding/continue` failed closed with outcome `\(payload.outcome)` and reason `\(boundedOnboardingContinueField(payload.reason) ?? "not_provided")`. This shell remains limited to exact locally provable desktop receipt submission and does not bypass later onboarding law.",
                reason: boundedOnboardingContinueField(payload.reason)
            )
        } catch {
            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                receiptKind: ingressContext.receiptKind,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge could not deliver this bounded desktop platform-setup receipt.",
                detail: error.localizedDescription
            )
        }
    }

    func acceptDesktopTerms(
        _ ingressContext: DesktopTermsAcceptIngressContext
    ) async -> DesktopTermsAcceptRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(OnboardingContinueAdapterResponsePayload.self, from: data)

            if statusCode == 200,
               payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    requestID: ingressContext.requestID,
                    endpoint: ingressContext.endpoint,
                    response: payload,
                    fallbackOnboardingSessionID: ingressContext.onboardingSessionID,
                    fallbackTermsVersionID: ingressContext.termsVersionID
                )
            }

            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                termsVersionID: ingressContext.termsVersionID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge rejected or failed this bounded desktop terms acceptance request.",
                detail: "Canonical `/v1/onboarding/continue` failed closed with outcome `\(payload.outcome)` and reason `\(boundedOnboardingContinueField(payload.reason) ?? "not_provided")`. This shell remains limited to exact `TERMS_ACCEPT` and does not bypass later onboarding law.",
                reason: boundedOnboardingContinueField(payload.reason)
            )
        } catch {
            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                termsVersionID: ingressContext.termsVersionID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge could not deliver this bounded desktop terms acceptance request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopEmployeePhotoCaptureSend(
        _ ingressContext: DesktopEmployeePhotoCaptureSendIngressContext
    ) async -> DesktopEmployeePhotoCaptureSendRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(OnboardingContinueAdapterResponsePayload.self, from: data)

            if statusCode == 200,
               payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    requestID: ingressContext.requestID,
                    endpoint: ingressContext.endpoint,
                    response: payload,
                    fallbackOnboardingSessionID: ingressContext.onboardingSessionID,
                    fallbackPhotoBlobRef: ingressContext.photoBlobRef
                )
            }

            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                photoBlobRef: ingressContext.photoBlobRef,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge rejected or failed this bounded employee photo capture send request.",
                detail: "Canonical `/v1/onboarding/continue` failed closed with outcome `\(payload.outcome)` and reason `\(boundedOnboardingContinueField(payload.reason) ?? "not_provided")`. This shell remains limited to exact `EMPLOYEE_PHOTO_CAPTURE_SEND` with an already-existing exact `photo_blob_ref` and does not bypass sender-verification law.",
                reason: boundedOnboardingContinueField(payload.reason)
            )
        } catch {
            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                photoBlobRef: ingressContext.photoBlobRef,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge could not deliver this bounded employee photo capture send request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopEmployeeSenderVerifyCommit(
        _ ingressContext: DesktopEmployeeSenderVerifyCommitIngressContext
    ) async -> DesktopEmployeeSenderVerifyCommitRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(OnboardingContinueAdapterResponsePayload.self, from: data)

            if statusCode == 200,
               payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    requestID: ingressContext.requestID,
                    endpoint: ingressContext.endpoint,
                    response: payload,
                    fallbackOnboardingSessionID: ingressContext.onboardingSessionID,
                    fallbackSenderDecision: ingressContext.senderDecision
                )
            }

            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                senderDecision: ingressContext.senderDecision,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge rejected or failed this bounded sender verification commit request.",
                detail: "Canonical `/v1/onboarding/continue` failed closed with outcome `\(payload.outcome)` and reason `\(boundedOnboardingContinueField(payload.reason) ?? "not_provided")`. This shell remains limited to exact `EMPLOYEE_SENDER_VERIFY_COMMIT` with exact `sender_decision` only and does not widen sender workflow authority.",
                reason: boundedOnboardingContinueField(payload.reason)
            )
        } catch {
            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                senderDecision: ingressContext.senderDecision,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge could not deliver this bounded sender verification commit request.",
                detail: error.localizedDescription
            )
        }
    }

    func confirmDesktopPrimaryDevice(
        _ ingressContext: DesktopPrimaryDeviceConfirmIngressContext
    ) async -> DesktopPrimaryDeviceConfirmRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(OnboardingContinueAdapterResponsePayload.self, from: data)

            if statusCode == 200,
               payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    requestID: ingressContext.requestID,
                    endpoint: ingressContext.endpoint,
                    response: payload,
                    fallbackOnboardingSessionID: ingressContext.onboardingSessionID,
                    fallbackDeviceID: ingressContext.deviceID
                )
            }

            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge rejected or failed this bounded desktop primary-device confirmation request.",
                detail: "Canonical `/v1/onboarding/continue` failed closed with outcome `\(payload.outcome)` and reason `\(boundedOnboardingContinueField(payload.reason) ?? "not_provided")`. This shell remains limited to exact `PRIMARY_DEVICE_CONFIRM` and does not bypass later onboarding law.",
                reason: boundedOnboardingContinueField(payload.reason)
            )
        } catch {
            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge could not deliver this bounded desktop primary-device confirmation request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopVoiceEnrollLock(
        _ ingressContext: DesktopVoiceEnrollIngressContext
    ) async -> DesktopVoiceEnrollRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(OnboardingContinueAdapterResponsePayload.self, from: data)

            if statusCode == 200,
               payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    requestID: ingressContext.requestID,
                    endpoint: ingressContext.endpoint,
                    response: payload,
                    fallbackOnboardingSessionID: ingressContext.onboardingSessionID,
                    fallbackDeviceID: ingressContext.deviceID,
                    fallbackSampleSeed: ingressContext.sampleSeed
                )
            }

            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                deviceID: ingressContext.deviceID,
                sampleSeed: ingressContext.sampleSeed,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge rejected or failed this bounded desktop voice-enroll lock request.",
                detail: "Canonical `/v1/onboarding/continue` failed closed with outcome `\(payload.outcome)` and reason `\(boundedOnboardingContinueField(payload.reason) ?? "not_provided")`. This shell remains limited to exact `VOICE_ENROLL_LOCK`, preserves any returned `WAKE_ENROLL` visibility as read-only only, and does not bypass later onboarding law.",
                reason: boundedOnboardingContinueField(payload.reason)
            )
        } catch {
            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                deviceID: ingressContext.deviceID,
                sampleSeed: ingressContext.sampleSeed,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge could not deliver this bounded desktop voice-enroll lock request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopWakeEnrollStartDraft(
        _ ingressContext: DesktopWakeEnrollStartDraftIngressContext
    ) async -> DesktopWakeEnrollStartDraftRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(OnboardingContinueAdapterResponsePayload.self, from: data)

            if statusCode == 200,
               payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    requestID: ingressContext.requestID,
                    endpoint: ingressContext.endpoint,
                    response: payload,
                    fallbackOnboardingSessionID: ingressContext.onboardingSessionID,
                    fallbackDeviceID: ingressContext.deviceID
                )
            }

            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge rejected or failed this bounded desktop wake-enroll start-draft request.",
                detail: "Canonical `/v1/onboarding/continue` failed closed with outcome `\(payload.outcome)` and reason `\(boundedOnboardingContinueField(payload.reason) ?? "not_provided")`. This shell remains limited to exact `WAKE_ENROLL_START_DRAFT`, preserves returned `WAKE_ENROLL` and `voice_artifact_sync_receipt_ref` visibility in read-only form only, keeps later wake-sample and wake-complete submit separately gated, and does not bypass later onboarding law.",
                reason: boundedOnboardingContinueField(payload.reason)
            )
        } catch {
            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge could not deliver this bounded desktop wake-enroll start-draft request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopWakeEnrollSampleCommit(
        _ ingressContext: DesktopWakeEnrollSampleCommitIngressContext
    ) async -> DesktopWakeEnrollSampleCommitRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(OnboardingContinueAdapterResponsePayload.self, from: data)

            if statusCode == 200,
               payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    requestID: ingressContext.requestID,
                    endpoint: ingressContext.endpoint,
                    response: payload,
                    fallbackOnboardingSessionID: ingressContext.onboardingSessionID,
                    fallbackDeviceID: ingressContext.deviceID
                )
            }

            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge rejected or failed this bounded desktop wake-enroll sample-commit request.",
                detail: "Canonical `/v1/onboarding/continue` failed closed with outcome `\(payload.outcome)` and reason `\(boundedOnboardingContinueField(payload.reason) ?? "not_provided")`. This shell remains limited to exact `WAKE_ENROLL_SAMPLE_COMMIT`, keeps wake-complete and wake-defer submit separately gated, and does not bypass later onboarding law.",
                reason: boundedOnboardingContinueField(payload.reason)
            )
        } catch {
            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge could not deliver this bounded desktop wake-enroll sample-commit request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopWakeEnrollCompleteCommit(
        _ ingressContext: DesktopWakeEnrollCompleteCommitIngressContext
    ) async -> DesktopWakeEnrollCompleteCommitRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(OnboardingContinueAdapterResponsePayload.self, from: data)

            if statusCode == 200,
               payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    requestID: ingressContext.requestID,
                    endpoint: ingressContext.endpoint,
                    response: payload,
                    fallbackOnboardingSessionID: ingressContext.onboardingSessionID,
                    fallbackDeviceID: ingressContext.deviceID
                )
            }

            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge rejected or failed this bounded desktop wake-enroll complete-commit request.",
                detail: "Canonical `/v1/onboarding/continue` failed closed with outcome `\(payload.outcome)` and reason `\(boundedOnboardingContinueField(payload.reason) ?? "not_provided")`. This shell remains limited to exact wake-enroll complete commit, preserves returned `EMO_PERSONA_LOCK`, `voice_artifact_sync_receipt_ref`, and any returned `WAKE_ENROLL` visibility in read-only form only, and does not bypass later onboarding law.",
                reason: boundedOnboardingContinueField(payload.reason)
            )
        } catch {
            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge could not deliver this bounded desktop wake-enroll complete-commit request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopWakeEnrollDeferCommit(
        _ ingressContext: DesktopWakeEnrollDeferCommitIngressContext
    ) async -> DesktopWakeEnrollDeferCommitRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(OnboardingContinueAdapterResponsePayload.self, from: data)

            if statusCode == 200,
               payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    requestID: ingressContext.requestID,
                    endpoint: ingressContext.endpoint,
                    response: payload,
                    fallbackOnboardingSessionID: ingressContext.onboardingSessionID,
                    fallbackDeviceID: ingressContext.deviceID
                )
            }

            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge rejected or failed this bounded desktop wake-enroll defer-commit request.",
                detail: "Canonical `/v1/onboarding/continue` failed closed with outcome `\(payload.outcome)` and reason `\(boundedOnboardingContinueField(payload.reason) ?? "not_provided")`. This shell remains limited to exact `WAKE_ENROLL_DEFER_COMMIT`, preserves returned `WAKE_ENROLL` and `voice_artifact_sync_receipt_ref` visibility in read-only form only, and does not add local `deferred_until` authoring, wake-listener behavior, pairing completion mutation, or autonomous unlock.",
                reason: boundedOnboardingContinueField(payload.reason)
            )
        } catch {
            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge could not deliver this bounded desktop wake-enroll defer-commit request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopSessionAttach(
        _ ingressContext: DesktopSessionAttachIngressContext
    ) async -> DesktopSessionAttachRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(SessionAttachAdapterResponsePayload.self, from: data)

            if statusCode == 200,
               payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    requestID: ingressContext.requestID,
                    endpoint: ingressContext.endpoint,
                    response: payload,
                    fallbackSourceSurfaceIdentity: ingressContext.sourceSurfaceIdentity,
                    fallbackSessionState: ingressContext.sessionState,
                    fallbackSessionID: ingressContext.sessionID,
                    fallbackCurrentVisibleSessionAttachOutcome: ingressContext.currentVisibleSessionAttachOutcome,
                    fallbackTurnID: ingressContext.turnID,
                    fallbackDeviceID: ingressContext.deviceID
                )
            }

            return .failed(
                sourceSurfaceIdentity: ingressContext.sourceSurfaceIdentity,
                sessionState: ingressContext.sessionState,
                sessionID: ingressContext.sessionID,
                currentVisibleSessionAttachOutcome: ingressContext.currentVisibleSessionAttachOutcome,
                turnID: ingressContext.turnID,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical session-attach bridge rejected or failed this bounded desktop current-visible session attach request.",
                detail: "Canonical `/v1/session/attach` failed closed with outcome `\(payload.outcome)` and reason `\(boundedOnboardingContinueField(payload.reason) ?? "not_provided")`. This shell remains limited to exact current-visible session attach submission and does not widen into local reopen authority, conversation selection, search or tool controls, hidden/background wake behavior, or autonomous unlock.",
                reason: boundedOnboardingContinueField(payload.reason)
            )
        } catch {
            return .failed(
                sourceSurfaceIdentity: ingressContext.sourceSurfaceIdentity,
                sessionState: ingressContext.sessionState,
                sessionID: ingressContext.sessionID,
                currentVisibleSessionAttachOutcome: ingressContext.currentVisibleSessionAttachOutcome,
                turnID: ingressContext.turnID,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical session-attach bridge could not deliver this bounded desktop current-visible session attach request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopSessionSoftClosedResume(
        _ ingressContext: DesktopSessionSoftClosedResumeIngressContext
    ) async -> DesktopSessionSoftClosedResumeRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(SessionResumeAdapterResponsePayload.self, from: data)

            if statusCode == 200,
               payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    requestID: ingressContext.requestID,
                    endpoint: ingressContext.endpoint,
                    response: payload,
                    fallbackSourceSurfaceIdentity: ingressContext.sourceSurfaceIdentity,
                    fallbackSessionState: ingressContext.sessionState,
                    fallbackSessionID: ingressContext.sessionID,
                    fallbackSelectedThreadID: ingressContext.selectedThreadID,
                    fallbackSelectedThreadTitle: ingressContext.selectedThreadTitle,
                    fallbackPendingWorkOrderID: ingressContext.pendingWorkOrderID,
                    fallbackResumeTier: ingressContext.resumeTier,
                    fallbackResumeSummaryBullets: ingressContext.resumeSummaryBullets,
                    fallbackDeviceID: ingressContext.deviceID
                )
            }

            return .failed(
                sourceSurfaceIdentity: ingressContext.sourceSurfaceIdentity,
                sessionState: ingressContext.sessionState,
                sessionID: ingressContext.sessionID,
                selectedThreadID: ingressContext.selectedThreadID,
                selectedThreadTitle: ingressContext.selectedThreadTitle,
                pendingWorkOrderID: ingressContext.pendingWorkOrderID,
                resumeTier: ingressContext.resumeTier,
                resumeSummaryBullets: ingressContext.resumeSummaryBullets,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical session-resume bridge rejected or failed this bounded desktop soft-closed explicit resume request.",
                detail: "Canonical `/v1/session/resume` failed closed with outcome `\(payload.outcome)` and reason `\(boundedOnboardingContinueField(payload.reason) ?? "not_provided")`. This shell remains limited to exact soft-closed explicit resume and does not widen into broader attach/reopen mutation, local thread reselection, local PH1.M synthesis, pairing completion mutation, wake-listener behavior, or autonomous unlock.",
                reason: boundedOnboardingContinueField(payload.reason)
            )
        } catch {
            return .failed(
                sourceSurfaceIdentity: ingressContext.sourceSurfaceIdentity,
                sessionState: ingressContext.sessionState,
                sessionID: ingressContext.sessionID,
                selectedThreadID: ingressContext.selectedThreadID,
                selectedThreadTitle: ingressContext.selectedThreadTitle,
                pendingWorkOrderID: ingressContext.pendingWorkOrderID,
                resumeTier: ingressContext.resumeTier,
                resumeSummaryBullets: ingressContext.resumeSummaryBullets,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical session-resume bridge could not deliver this bounded desktop soft-closed explicit resume request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopSessionRecover(
        _ ingressContext: DesktopSessionRecoverIngressContext
    ) async -> DesktopSessionRecoverRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(SessionRecoverAdapterResponsePayload.self, from: data)

            if statusCode == 200,
               payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    requestID: ingressContext.requestID,
                    endpoint: ingressContext.endpoint,
                    response: payload,
                    fallbackSourceSurfaceIdentity: ingressContext.sourceSurfaceIdentity,
                    fallbackSessionState: ingressContext.sessionState,
                    fallbackSessionID: ingressContext.sessionID,
                    fallbackRecoveryMode: ingressContext.recoveryMode,
                    fallbackDeviceID: ingressContext.deviceID
                )
            }

            return .failed(
                sourceSurfaceIdentity: ingressContext.sourceSurfaceIdentity,
                sessionState: ingressContext.sessionState,
                sessionID: ingressContext.sessionID,
                recoveryMode: ingressContext.recoveryMode,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical session-recover bridge rejected or failed this bounded desktop suspended-session recover request.",
                detail: "Canonical `/v1/session/recover` failed closed with outcome `\(payload.outcome)` and reason `\(boundedOnboardingContinueField(payload.reason) ?? "not_provided")`. This shell remains limited to exact suspended-session authoritative-reread recovery submission and does not widen into local unsuspend authority, broader attach/reopen mutation, search or tool controls, hidden/background wake behavior, or autonomous unlock.",
                reason: boundedOnboardingContinueField(payload.reason)
            )
        } catch {
            return .failed(
                sourceSurfaceIdentity: ingressContext.sourceSurfaceIdentity,
                sessionState: ingressContext.sessionState,
                sessionID: ingressContext.sessionID,
                recoveryMode: ingressContext.recoveryMode,
                deviceID: ingressContext.deviceID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical session-recover bridge could not deliver this bounded desktop suspended-session recover request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopSessionMultiPostureResume(
        _ ingressContext: DesktopSessionMultiPostureResumeIngressContext
    ) async -> DesktopSessionMultiPostureResumeRuntimeOutcomeState {
        switch ingressContext.resumeMode {
        case .softClosedExplicitResume:
            guard let softClosedResumeIngressContext = ingressContext.softClosedResumeIngressContext else {
                return .failed(
                    resumeMode: ingressContext.resumeMode,
                    sourceSurfaceIdentity: ingressContext.sourceSurfaceIdentity,
                    sessionState: ingressContext.sessionState,
                    sessionID: ingressContext.sessionID,
                    selectedThreadID: ingressContext.selectedThreadID,
                    selectedThreadTitle: ingressContext.selectedThreadTitle,
                    pendingWorkOrderID: ingressContext.pendingWorkOrderID,
                    resumeTier: ingressContext.resumeTier,
                    resumeSummaryBullets: ingressContext.resumeSummaryBullets,
                    recoveryMode: ingressContext.recoveryMode,
                    deviceID: ingressContext.deviceID,
                    endpoint: ingressContext.endpoint,
                    requestID: ingressContext.requestID,
                    summary: "The canonical multi-posture session-resume bridge could not select a lawful soft-closed explicit resume route.",
                    detail: "This bounded desktop session-resume request failed closed because the exact soft-closed route context was unavailable after route selection."
                )
            }

            let routeOutcome = await submitDesktopSessionSoftClosedResume(softClosedResumeIngressContext)
            return .fromSoftClosedRoute(routeOutcome)

        case .suspendedAuthoritativeRereadRecover:
            guard let sessionRecoverIngressContext = ingressContext.sessionRecoverIngressContext else {
                return .failed(
                    resumeMode: ingressContext.resumeMode,
                    sourceSurfaceIdentity: ingressContext.sourceSurfaceIdentity,
                    sessionState: ingressContext.sessionState,
                    sessionID: ingressContext.sessionID,
                    selectedThreadID: ingressContext.selectedThreadID,
                    selectedThreadTitle: ingressContext.selectedThreadTitle,
                    pendingWorkOrderID: ingressContext.pendingWorkOrderID,
                    resumeTier: ingressContext.resumeTier,
                    resumeSummaryBullets: ingressContext.resumeSummaryBullets,
                    recoveryMode: ingressContext.recoveryMode,
                    deviceID: ingressContext.deviceID,
                    endpoint: ingressContext.endpoint,
                    requestID: ingressContext.requestID,
                    summary: "The canonical multi-posture session-resume bridge could not select a lawful suspended-session recover route.",
                    detail: "This bounded desktop session-resume request failed closed because the exact suspended-session recover route context was unavailable after route selection."
                )
            }

            let routeOutcome = await submitDesktopSessionRecover(sessionRecoverIngressContext)
            return .fromRecoverRoute(routeOutcome)
        }
    }

    func submitDesktopWakeProfileAvailabilityRefresh(
        _ ingressContext: DesktopWakeProfileAvailabilityIngressContext
    ) async -> DesktopWakeProfileAvailabilityRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(
                WakeProfileAvailabilityRefreshAdapterResponsePayload.self,
                from: data
            )

            if statusCode == 200,
               payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    requestID: ingressContext.requestID,
                    endpoint: ingressContext.endpoint,
                    response: payload,
                    fallbackReceiptKind: ingressContext.receiptKind,
                    fallbackDeviceID: ingressContext.deviceID,
                    fallbackWakeProfileID: ingressContext.wakeProfileID,
                    fallbackVoiceArtifactSyncReceiptRef: ingressContext.voiceArtifactSyncReceiptRef
                )
            }

            return .failed(
                receiptKind: ingressContext.receiptKind,
                deviceID: ingressContext.deviceID,
                wakeProfileID: ingressContext.wakeProfileID,
                voiceArtifactSyncReceiptRef: ingressContext.voiceArtifactSyncReceiptRef,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical wake-profile availability bridge rejected or failed this bounded desktop wake-profile local-availability refresh request.",
                detail: "Canonical `/v1/wake-profile/availability` failed closed with outcome `\(payload.outcome)` and reason `\(boundedOnboardingContinueField(payload.reason) ?? "not_provided")`. This shell remains limited to exact wake-profile local-availability refresh and does not add native wake-listener start or stop, wake detection, wake-to-turn dispatch, hidden/background auto-start, or autonomous unlock.",
                reason: boundedOnboardingContinueField(payload.reason)
            )
        } catch {
            return .failed(
                receiptKind: ingressContext.receiptKind,
                deviceID: ingressContext.deviceID,
                wakeProfileID: ingressContext.wakeProfileID,
                voiceArtifactSyncReceiptRef: ingressContext.voiceArtifactSyncReceiptRef,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical wake-profile availability bridge could not deliver this bounded desktop wake-profile local-availability refresh request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopEmoPersonaLock(
        _ ingressContext: DesktopEmoPersonaLockIngressContext
    ) async -> DesktopEmoPersonaLockRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(OnboardingContinueAdapterResponsePayload.self, from: data)

            if statusCode == 200,
               payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    requestID: ingressContext.requestID,
                    endpoint: ingressContext.endpoint,
                    response: payload,
                    fallbackOnboardingSessionID: ingressContext.onboardingSessionID
                )
            }

            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge rejected or failed this bounded desktop emo/persona-lock request.",
                detail: "Canonical `/v1/onboarding/continue` failed closed with outcome `\(payload.outcome)` and reason `\(boundedOnboardingContinueField(payload.reason) ?? "not_provided")`. This shell remains limited to exact emo/persona lock, preserves returned `ACCESS_PROVISION`, `voice_artifact_sync_receipt_ref`, and any returned `EMO_PERSONA_LOCK` visibility in read-only form only, and does not bypass later onboarding law.",
                reason: boundedOnboardingContinueField(payload.reason)
            )
        } catch {
            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge could not deliver this bounded desktop emo/persona-lock request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopAccessProvisionCommit(
        _ ingressContext: DesktopAccessProvisionCommitIngressContext
    ) async -> DesktopAccessProvisionCommitRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(OnboardingContinueAdapterResponsePayload.self, from: data)

            if statusCode == 200,
               payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    requestID: ingressContext.requestID,
                    endpoint: ingressContext.endpoint,
                    response: payload,
                    fallbackOnboardingSessionID: ingressContext.onboardingSessionID
                )
            }

            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge rejected or failed this bounded desktop access-provision request.",
                detail: "Canonical `/v1/onboarding/continue` failed closed with outcome `\(payload.outcome)` and reason `\(boundedOnboardingContinueField(payload.reason) ?? "not_provided")`. This shell remains limited to exact access provision commit, preserves returned `COMPLETE`, `voice_artifact_sync_receipt_ref`, `access_engine_instance_id`, and any returned `ACCESS_PROVISION` visibility in read-only form only, and does not bypass later onboarding law.",
                reason: boundedOnboardingContinueField(payload.reason)
            )
        } catch {
            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge could not deliver this bounded desktop access-provision request.",
                detail: error.localizedDescription
            )
        }
    }

    func submitDesktopCompleteCommit(
        _ ingressContext: DesktopCompleteCommitIngressContext
    ) async -> DesktopCompleteCommitRuntimeOutcomeState {
        do {
            try await ensureAdapterAvailable()

            let (data, response) = try await urlSession.data(for: ingressContext.urlRequest)
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let httpResponse = response as? HTTPURLResponse
            let statusCode = httpResponse?.statusCode ?? 0
            let payload = try decoder.decode(OnboardingContinueAdapterResponsePayload.self, from: data)

            if statusCode == 200,
               payload.status.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() == "ok" {
                return .completed(
                    requestID: ingressContext.requestID,
                    endpoint: ingressContext.endpoint,
                    response: payload,
                    fallbackOnboardingSessionID: ingressContext.onboardingSessionID
                )
            }

            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge rejected or failed this bounded desktop complete request.",
                detail: "Canonical `/v1/onboarding/continue` failed closed with outcome `\(payload.outcome)` and reason `\(boundedOnboardingContinueField(payload.reason) ?? "not_provided")`. This shell remains limited to exact complete commit, preserves returned `READY`, `onboarding_status`, `voice_artifact_sync_receipt_ref`, `access_engine_instance_id`, and any returned `COMPLETE` visibility in read-only form only, and does not bypass later onboarding law.",
                reason: boundedOnboardingContinueField(payload.reason)
            )
        } catch {
            return .failed(
                onboardingSessionID: ingressContext.onboardingSessionID,
                endpoint: ingressContext.endpoint,
                requestID: ingressContext.requestID,
                summary: "The canonical onboarding-continue bridge could not deliver this bounded desktop complete request.",
                detail: error.localizedDescription
            )
        }
    }

    func desktopExplicitVoiceIngressRequestBuilder(
        _ preparedRequest: ExplicitVoiceTurnRequestState
    ) throws -> DesktopExplicitVoiceIngressContext {
        let transcript = preparedRequest.transcript.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !transcript.isEmpty else {
            throw BridgeError.invalidPreparedRequest(
                "the prepared explicit voice request contained no transcript after bounded shell validation"
            )
        }

        let timestampMS = Self.systemTimeNowMS()
        let monotonicNowNS = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)
        let correlationID = monotonicNowNS
        let turnID = monotonicNowNS &+ 1
        let requestID = "desktop_runtime_request_\(preparedRequest.id)_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_runtime_idempotency_\(preparedRequest.id)"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let audioCaptureRef = try desktopVoiceTurnAudioCaptureRefBuilder(preparedRequest.audioCaptureRefState)

        let payload = VoiceTurnAdapterRequestPayload(
            correlationID: correlationID,
            turnID: turnID,
            deviceTurnSequence: nil,
            appPlatform: "DESKTOP",
            platformVersion: nil,
            deviceClass: nil,
            runtimeClientVersion: nil,
            hardwareCapabilityProfile: nil,
            networkProfile: nil,
            claimedCapabilities: nil,
            integrityStatus: nil,
            attestationRef: nil,
            trigger: "EXPLICIT",
            actorUserID: actorUserID,
            tenantID: tenantID,
            deviceID: deviceID,
            nowNS: monotonicNowNS,
            threadKey: nil,
            projectID: nil,
            pinnedContextRefs: nil,
            threadPolicyFlags: nil,
            userTextPartial: nil,
            userTextFinal: transcript,
            seleneTextPartial: nil,
            seleneTextFinal: nil,
            audioCaptureRef: audioCaptureRef,
            visualInputRef: nil
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/voice/turn")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: actorUserID, device: deviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopExplicitVoiceIngressContext(
            preparedRequestID: preparedRequest.id,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopWakeTriggeredVoiceIngressRequestBuilder(
        _ preparedRequest: WakeTriggeredVoiceTurnRequestState
    ) throws -> DesktopWakeTriggeredVoiceIngressContext {
        let transcript = preparedRequest.transcript.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !transcript.isEmpty else {
            throw BridgeError.invalidPreparedRequest(
                "the prepared wake-triggered voice request contained no post-wake transcript after bounded shell validation"
            )
        }

        let detectionText = Self.nonEmpty(preparedRequest.audioCaptureRefState.detectionText)
        guard detectionText == "Selene" else {
            throw BridgeError.invalidPreparedRequest(
                "the prepared wake-triggered voice request did not preserve the exact local wake detection text carrier"
            )
        }

        guard preparedRequest.audioCaptureRefState.detectionConfidenceBP != nil else {
            throw BridgeError.invalidPreparedRequest(
                "the prepared wake-triggered voice request did not preserve a bounded local wake detection confidence carrier"
            )
        }

        let timestampMS = Self.systemTimeNowMS()
        let monotonicNowNS = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)
        let correlationID = monotonicNowNS
        let turnID = monotonicNowNS &+ 1
        let requestID = "desktop_runtime_request_\(preparedRequest.id)_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_runtime_idempotency_\(preparedRequest.id)"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let audioCaptureRef = try desktopVoiceTurnAudioCaptureRefBuilder(preparedRequest.audioCaptureRefState)

        let payload = VoiceTurnAdapterRequestPayload(
            correlationID: correlationID,
            turnID: turnID,
            deviceTurnSequence: nil,
            appPlatform: "DESKTOP",
            platformVersion: nil,
            deviceClass: nil,
            runtimeClientVersion: nil,
            hardwareCapabilityProfile: nil,
            networkProfile: nil,
            claimedCapabilities: nil,
            integrityStatus: nil,
            attestationRef: nil,
            trigger: "WAKE_WORD",
            actorUserID: actorUserID,
            tenantID: tenantID,
            deviceID: deviceID,
            nowNS: monotonicNowNS,
            threadKey: nil,
            projectID: nil,
            pinnedContextRefs: nil,
            threadPolicyFlags: nil,
            userTextPartial: nil,
            userTextFinal: transcript,
            seleneTextPartial: nil,
            seleneTextFinal: nil,
            audioCaptureRef: audioCaptureRef,
            visualInputRef: nil
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/voice/turn")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: actorUserID, device: deviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopWakeTriggeredVoiceIngressContext(
            preparedRequestID: preparedRequest.id,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopVoiceTurnAudioCaptureRefBuilder(
        _ captureState: DesktopVoiceTurnAudioCaptureRefState
    ) throws -> DesktopVoiceTurnAudioCaptureRefPayload {
        guard let localeTag = Self.nonEmpty(captureState.localeTag) else {
            throw BridgeError.invalidPreparedRequest(
                "the prepared explicit voice request contained no locale_tag for audio_capture_ref transport"
            )
        }

        guard let deviceRoute = Self.nonEmpty(captureState.deviceRoute) else {
            throw BridgeError.invalidPreparedRequest(
                "the prepared explicit voice request contained no device_route for audio_capture_ref transport"
            )
        }

        guard ["BUILT_IN", "BLUETOOTH", "USB"].contains(deviceRoute) else {
            throw BridgeError.invalidPreparedRequest(
                "the prepared explicit voice request contained an unsupported device_route `\(deviceRoute)` for audio_capture_ref transport"
            )
        }

        guard let selectedMic = Self.nonEmpty(captureState.selectedMic) else {
            throw BridgeError.invalidPreparedRequest(
                "the prepared explicit voice request contained no selected_mic for audio_capture_ref transport"
            )
        }

        guard let selectedSpeaker = Self.nonEmpty(captureState.selectedSpeaker) else {
            throw BridgeError.invalidPreparedRequest(
                "the prepared explicit voice request contained no selected_speaker for audio_capture_ref transport"
            )
        }

        let tStartNS = Swift.max(captureState.tStartNS, 1)
        let tEndNS = Swift.max(captureState.tEndNS, tStartNS &+ 1)
        let tCandidateStartNS = Swift.max(captureState.tCandidateStartNS, tStartNS)
        let tConfirmedNS = Swift.max(captureState.tConfirmedNS, tCandidateStartNS)
        let detectionText = Self.nonEmpty(captureState.detectionText)

        return DesktopVoiceTurnAudioCaptureRefPayload(
            streamID: Swift.max(captureState.streamID, 1),
            preRollBufferID: Swift.max(captureState.preRollBufferID, 1),
            tStartNS: tStartNS,
            tEndNS: tEndNS,
            tCandidateStartNS: tCandidateStartNS,
            tConfirmedNS: tConfirmedNS,
            localeTag: localeTag,
            deviceRoute: deviceRoute,
            selectedMic: selectedMic,
            selectedSpeaker: selectedSpeaker,
            ttsPlaybackActive: captureState.ttsPlaybackActive,
            detectionText: detectionText,
            detectionConfidenceBP: detectionText == nil ? nil : captureState.detectionConfidenceBP,
            vadConfidenceBP: captureState.vadConfidenceBP,
            acousticConfidenceBP: captureState.acousticConfidenceBP,
            prosodyConfidenceBP: captureState.prosodyConfidenceBP,
            speechLikenessBP: captureState.speechLikenessBP,
            echoSafeConfidenceBP: captureState.echoSafeConfidenceBP,
            nearfieldConfidenceBP: captureState.nearfieldConfidenceBP,
            captureDegraded: captureState.captureDegraded,
            streamGapDetected: captureState.streamGapDetected,
            aecUnstable: captureState.aecUnstable,
            deviceChanged: captureState.deviceChanged,
            snrDBMilli: captureState.snrDBMilli,
            clippingRatioBP: captureState.clippingRatioBP,
            echoDelayMSMilli: captureState.echoDelayMSMilli,
            packetLossBP: captureState.packetLossBP,
            doubleTalkBP: captureState.doubleTalkBP,
            erleDBMilli: captureState.erleDBMilli,
            deviceFailures24H: captureState.deviceFailures24H,
            deviceRecoveries24H: captureState.deviceRecoveries24H,
            deviceMeanRecoveryMS: captureState.deviceMeanRecoveryMS,
            deviceReliabilityBP: captureState.deviceReliabilityBP,
            timingJitterMSMilli: captureState.timingJitterMSMilli,
            timingDriftPPMMilli: captureState.timingDriftPPMMilli,
            timingBufferDepthMSMilli: captureState.timingBufferDepthMSMilli,
            timingUnderruns: captureState.timingUnderruns,
            timingOverruns: captureState.timingOverruns
        )
    }

    func desktopInviteClickRequestBuilder(
        _ onboardingEntryContext: DesktopOnboardingEntryContext
    ) throws -> DesktopInviteOpenIngressContext {
        let monotonicNowNS = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)
        let correlationID = monotonicNowNS
        let requestID = "desktop_invite_click_request_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_invite_click_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()

        let payload = InviteLinkOpenAdapterRequestPayload(
            correlationID: correlationID,
            idempotencyKey: idempotencyKey,
            tokenID: onboardingEntryContext.tokenID,
            tokenSignature: onboardingEntryContext.tokenSignature,
            tenantID: onboardingEntryContext.tenantID,
            appPlatform: "DESKTOP",
            deviceFingerprint: onboardingEntryContext.deviceFingerprint,
            appInstanceID: onboardingEntryContext.appInstanceID,
            deepLinkNonce: onboardingEntryContext.deepLinkNonce
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/invite/click")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: onboardingEntryContext.tokenID, device: onboardingEntryContext.appInstanceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopInviteOpenIngressContext(
            entryContextID: onboardingEntryContext.id,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopOnboardingContinueMissingFieldRequestBuilder(
        promptState: DesktopOnboardingContinuePromptState,
        fieldValue: String?
    ) throws -> DesktopOnboardingContinueIngressContext {
        guard let onboardingSessionID = boundedOnboardingContinueField(promptState.onboardingSessionID) else {
            throw BridgeError.invalidOnboardingContinueRequest(
                "the bounded onboarding-continue prompt state did not preserve a lawful onboarding_session_id"
            )
        }

        guard let blockingField = boundedOnboardingContinueField(promptState.blockingField) else {
            throw BridgeError.invalidOnboardingContinueRequest(
                "the bounded onboarding-continue prompt state did not preserve a lawful blocking_field"
            )
        }

        let boundedFieldValue: String?
        if let fieldValue {
            guard let normalizedFieldValue = boundedOnboardingContinueFieldInput(fieldValue) else {
                throw BridgeError.invalidOnboardingContinueRequest(
                    "the bounded onboarding-continue field input was missing or invalid for `ASK_MISSING_SUBMIT`"
                )
            }
            boundedFieldValue = normalizedFieldValue
        } else {
            boundedFieldValue = nil
        }

        let monotonicNowNS = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)
        let correlationID = monotonicNowNS
        let requestID = "desktop_onboarding_continue_request_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_onboarding_continue_\(onboardingSessionID)_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()

        let payload = OnboardingContinueAdapterRequestPayload(
            correlationID: correlationID,
            onboardingSessionID: onboardingSessionID,
            idempotencyKey: idempotencyKey,
            tenantID: tenantID,
            action: "ASK_MISSING_SUBMIT",
            fieldValue: boundedFieldValue,
            receiptKind: nil,
            receiptRef: nil,
            signer: nil,
            payloadHash: nil,
            termsVersionID: nil,
            accepted: nil,
            deviceID: deviceID,
            proofOK: nil,
            sampleSeed: nil,
            photoBlobRef: nil,
            senderDecision: nil
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/onboarding/continue")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: actorUserID, device: deviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopOnboardingContinueIngressContext(
            onboardingSessionID: onboardingSessionID,
            blockingField: blockingField,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopPlatformSetupReceiptRequestBuilder(
        _ draft: DesktopPlatformSetupReceiptDraft
    ) throws -> DesktopPlatformSetupReceiptIngressContext {
        guard let onboardingSessionID = boundedOnboardingContinueField(draft.onboardingSessionID) else {
            throw BridgeError.invalidPlatformSetupReceiptRequest(
                "the bounded desktop platform-setup receipt draft did not preserve a lawful onboarding_session_id"
            )
        }

        guard let receiptKind = boundedSupportedDesktopPlatformSetupReceiptKind(draft.receiptKind) else {
            throw BridgeError.invalidPlatformSetupReceiptRequest(
                "only exact `install_launch_handshake`, exact `mic_permission_granted`, exact `desktop_pairing_bound`, and exact `desktop_wakeword_configured` are in scope for bounded desktop local receipt submission"
            )
        }

        guard let proofMaterial = boundedDesktopPlatformSetupReceiptProofMaterial(draft.proofMaterial) else {
            throw BridgeError.invalidPlatformSetupReceiptRequest(
                "the bounded desktop platform-setup receipt draft did not preserve lawful local proof material"
            )
        }

        let payloadHashSeed = "\(onboardingSessionID)|\(receiptKind)|\(deviceID)|\(proofMaterial)"
        let payloadHash = desktopPlatformSetupReceiptPayloadHash(payloadHashSeed)
        let receiptRef = desktopPlatformSetupReceiptRef(receiptKind: receiptKind, payloadHash: payloadHash)
        let requestID = "desktop_platform_setup_receipt_request_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_platform_setup_receipt_\(onboardingSessionID)_\(receiptKind)_\(payloadHash.prefix(12))"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()
        let correlationID = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)

        let payload = OnboardingContinueAdapterRequestPayload(
            correlationID: correlationID,
            onboardingSessionID: onboardingSessionID,
            idempotencyKey: idempotencyKey,
            tenantID: tenantID,
            action: "PLATFORM_SETUP_RECEIPT",
            fieldValue: nil,
            receiptKind: receiptKind,
            receiptRef: receiptRef,
            signer: "selene_desktop_app",
            payloadHash: payloadHash,
            termsVersionID: nil,
            accepted: nil,
            deviceID: deviceID,
            proofOK: nil,
            sampleSeed: nil,
            photoBlobRef: nil,
            senderDecision: nil
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/onboarding/continue")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: actorUserID, device: deviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopPlatformSetupReceiptIngressContext(
            onboardingSessionID: onboardingSessionID,
            receiptKind: receiptKind,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopTermsAcceptRequestBuilder(
        _ promptState: DesktopTermsAcceptPromptState
    ) throws -> DesktopTermsAcceptIngressContext {
        guard let onboardingSessionID = boundedOnboardingContinueField(promptState.onboardingSessionID) else {
            throw BridgeError.invalidTermsAcceptRequest(
                "the bounded desktop terms prompt state did not preserve a lawful onboarding_session_id"
            )
        }

        guard let nextStep = boundedOnboardingContinueField(promptState.nextStep),
              nextStep == "TERMS" else {
            throw BridgeError.invalidTermsAcceptRequest(
                "bounded desktop terms acceptance is only lawful when canonical onboarding posture has advanced to exact `TERMS`"
            )
        }

        guard promptState.termsVersionID == desktopCanonicalTermsVersionID else {
            throw BridgeError.invalidTermsAcceptRequest(
                "bounded desktop terms acceptance must preserve exact current repo-truth `terms_v1` only"
            )
        }

        let requestID = "desktop_terms_accept_request_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_terms_accept_\(onboardingSessionID)_\(desktopCanonicalTermsVersionID)"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()
        let correlationID = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)

        let payload = OnboardingContinueAdapterRequestPayload(
            correlationID: correlationID,
            onboardingSessionID: onboardingSessionID,
            idempotencyKey: idempotencyKey,
            tenantID: tenantID,
            action: "TERMS_ACCEPT",
            fieldValue: nil,
            receiptKind: nil,
            receiptRef: nil,
            signer: nil,
            payloadHash: nil,
            termsVersionID: desktopCanonicalTermsVersionID,
            accepted: true,
            deviceID: deviceID,
            proofOK: nil,
            sampleSeed: nil,
            photoBlobRef: nil,
            senderDecision: nil
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/onboarding/continue")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: actorUserID, device: deviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopTermsAcceptIngressContext(
            onboardingSessionID: onboardingSessionID,
            termsVersionID: desktopCanonicalTermsVersionID,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopEmployeePhotoCaptureSendRequestBuilder(
        promptState: DesktopEmployeePhotoCaptureSendPromptState,
        photoBlobRef: String
    ) throws -> DesktopEmployeePhotoCaptureSendIngressContext {
        guard let onboardingSessionID = boundedOnboardingContinueField(promptState.onboardingSessionID) else {
            throw BridgeError.invalidEmployeePhotoCaptureSendRequest(
                "the bounded employee photo capture send prompt state did not preserve a lawful onboarding_session_id"
            )
        }

        guard let nextStep = boundedOnboardingContinueField(promptState.nextStep),
              nextStep == "SENDER_VERIFICATION" else {
            throw BridgeError.invalidEmployeePhotoCaptureSendRequest(
                "bounded employee photo capture send is only lawful when canonical onboarding posture remains at exact `SENDER_VERIFICATION`"
            )
        }

        guard let boundedPhotoBlobRef = boundedOnboardingContinueFieldInput(photoBlobRef) else {
            throw BridgeError.invalidEmployeePhotoCaptureSendRequest(
                "bounded employee photo capture send requires one exact existing `photo_blob_ref` and does not mint local blob authority"
            )
        }

        let requestID = "desktop_employee_photo_capture_send_request_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_employee_photo_capture_send_\(onboardingSessionID)_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()
        let correlationID = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)

        struct EmployeePhotoCaptureSendRequestPayload: Encodable {
            let correlationID: UInt64
            let onboardingSessionID: String
            let idempotencyKey: String
            let tenantID: String?
            let action: String
            let fieldValue: String?
            let receiptKind: String?
            let receiptRef: String?
            let signer: String?
            let payloadHash: String?
            let termsVersionID: String?
            let accepted: Bool?
            let deviceID: String?
            let proofOK: Bool?
            let sampleSeed: String?
            let photoBlobRef: String?
        }

        let payload = EmployeePhotoCaptureSendRequestPayload(
            correlationID: correlationID,
            onboardingSessionID: onboardingSessionID,
            idempotencyKey: idempotencyKey,
            tenantID: tenantID,
            action: "EMPLOYEE_PHOTO_CAPTURE_SEND",
            fieldValue: nil,
            receiptKind: nil,
            receiptRef: nil,
            signer: nil,
            payloadHash: nil,
            termsVersionID: nil,
            accepted: nil,
            deviceID: nil,
            proofOK: nil,
            sampleSeed: nil,
            photoBlobRef: boundedPhotoBlobRef
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/onboarding/continue")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: actorUserID, device: deviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopEmployeePhotoCaptureSendIngressContext(
            onboardingSessionID: onboardingSessionID,
            photoBlobRef: boundedPhotoBlobRef,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopEmployeeSenderVerifyCommitRequestBuilder(
        promptState: DesktopEmployeeSenderVerifyCommitPromptState,
        senderDecision: String
    ) throws -> DesktopEmployeeSenderVerifyCommitIngressContext {
        guard let onboardingSessionID = boundedOnboardingContinueField(promptState.onboardingSessionID) else {
            throw BridgeError.invalidEmployeeSenderVerifyCommitRequest(
                "the bounded sender verification commit prompt state did not preserve a lawful onboarding_session_id"
            )
        }

        guard let nextStep = boundedOnboardingContinueField(promptState.nextStep),
              nextStep == "SENDER_VERIFICATION" else {
            throw BridgeError.invalidEmployeeSenderVerifyCommitRequest(
                "bounded sender verification commit is only lawful when canonical onboarding posture remains at exact `SENDER_VERIFICATION`"
            )
        }

        guard let boundedPhotoBlobRef = boundedOnboardingContinueField(promptState.photoBlobRef) else {
            throw BridgeError.invalidEmployeeSenderVerifyCommitRequest(
                "bounded sender verification commit must derive from already-live H273 completed posture preserving exact `photo_blob_ref`"
            )
        }

        let allowedSenderDecisions = Set(["CONFIRM", "REJECT"])
        guard let boundedSenderDecision = boundedOnboardingContinueFieldInput(senderDecision),
              allowedSenderDecisions.contains(boundedSenderDecision) else {
            throw BridgeError.invalidEmployeeSenderVerifyCommitRequest(
                "bounded sender verification commit requires exact `sender_decision` of `CONFIRM` or `REJECT` only"
            )
        }

        let requestID = "desktop_employee_sender_verify_commit_request_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_employee_sender_verify_commit_\(onboardingSessionID)_\(boundedSenderDecision)_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()
        let correlationID = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)

        let payload = OnboardingContinueAdapterRequestPayload(
            correlationID: correlationID,
            onboardingSessionID: onboardingSessionID,
            idempotencyKey: idempotencyKey,
            tenantID: tenantID,
            action: "EMPLOYEE_SENDER_VERIFY_COMMIT",
            fieldValue: nil,
            receiptKind: nil,
            receiptRef: nil,
            signer: nil,
            payloadHash: nil,
            termsVersionID: nil,
            accepted: nil,
            deviceID: nil,
            proofOK: nil,
            sampleSeed: nil,
            photoBlobRef: nil,
            senderDecision: boundedSenderDecision
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/onboarding/continue")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: actorUserID, device: deviceID),
            forHTTPHeaderField: "Authorization"
        )

        _ = boundedPhotoBlobRef

        return DesktopEmployeeSenderVerifyCommitIngressContext(
            onboardingSessionID: onboardingSessionID,
            senderDecision: boundedSenderDecision,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopPrimaryDeviceConfirmRequestBuilder(
        _ promptState: DesktopPrimaryDeviceConfirmPromptState
    ) throws -> DesktopPrimaryDeviceConfirmIngressContext {
        guard let onboardingSessionID = boundedOnboardingContinueField(promptState.onboardingSessionID) else {
            throw BridgeError.invalidPrimaryDeviceConfirmRequest(
                "the bounded desktop primary-device prompt state did not preserve a lawful onboarding_session_id"
            )
        }

        guard let nextStep = boundedOnboardingContinueField(promptState.nextStep),
              nextStep == "PRIMARY_DEVICE_CONFIRM" else {
            throw BridgeError.invalidPrimaryDeviceConfirmRequest(
                "bounded desktop primary-device confirmation is only lawful when canonical onboarding posture has advanced to exact `PRIMARY_DEVICE_CONFIRM`"
            )
        }

        guard promptState.proofOK else {
            throw BridgeError.invalidPrimaryDeviceConfirmRequest(
                "bounded desktop primary-device confirmation must preserve exact `proofOK=true` only"
            )
        }

        guard let managedDeviceID = boundedOnboardingContinueField(deviceID),
              let promptDeviceID = boundedOnboardingContinueField(promptState.deviceID),
              promptDeviceID == managedDeviceID else {
            throw BridgeError.invalidPrimaryDeviceConfirmRequest(
                "bounded desktop primary-device confirmation must preserve the exact managed bridge `deviceID` only"
            )
        }

        let requestID = "desktop_primary_device_confirm_request_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_primary_device_confirm_\(onboardingSessionID)_\(managedDeviceID)"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()
        let correlationID = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)

        let payload = OnboardingContinueAdapterRequestPayload(
            correlationID: correlationID,
            onboardingSessionID: onboardingSessionID,
            idempotencyKey: idempotencyKey,
            tenantID: tenantID,
            action: "PRIMARY_DEVICE_CONFIRM",
            fieldValue: nil,
            receiptKind: nil,
            receiptRef: nil,
            signer: nil,
            payloadHash: nil,
            termsVersionID: nil,
            accepted: nil,
            deviceID: managedDeviceID,
            proofOK: true,
            sampleSeed: nil,
            photoBlobRef: nil,
            senderDecision: nil
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/onboarding/continue")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: actorUserID, device: managedDeviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopPrimaryDeviceConfirmIngressContext(
            onboardingSessionID: onboardingSessionID,
            deviceID: managedDeviceID,
            proofOK: true,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopVoiceEnrollSampleSeedBuilder(
        _ promptState: DesktopVoiceEnrollPromptState
    ) throws -> String {
        guard let onboardingSessionID = boundedOnboardingContinueField(promptState.onboardingSessionID) else {
            throw BridgeError.invalidVoiceEnrollRequest(
                "the bounded desktop voice-enroll prompt state did not preserve a lawful onboarding_session_id"
            )
        }

        guard let nextStep = boundedOnboardingContinueField(promptState.nextStep),
              nextStep == "VOICE_ENROLL" else {
            throw BridgeError.invalidVoiceEnrollRequest(
                "bounded desktop voice-enroll lock is only lawful when canonical onboarding posture has advanced to exact `VOICE_ENROLL`"
            )
        }

        guard let managedDeviceID = boundedOnboardingContinueField(deviceID),
              let promptDeviceID = boundedOnboardingContinueField(promptState.deviceID),
              promptDeviceID == managedDeviceID else {
            throw BridgeError.invalidVoiceEnrollRequest(
                "bounded desktop voice-enroll lock must preserve the exact managed bridge `deviceID` only"
            )
        }

        guard let transcriptPreview = boundedDesktopVoiceEnrollTranscriptPreview(promptState.transcriptPreview) else {
            throw BridgeError.invalidVoiceEnrollRequest(
                "bounded explicit voice transcript preview is missing or invalid for exact desktop voice-enroll sample-seed derivation"
            )
        }

        let seedMaterial = [
            onboardingSessionID,
            nextStep,
            managedDeviceID,
            transcriptPreview,
        ].joined(separator: "|")
        let seedDigest = SHA256.hash(data: Data(seedMaterial.utf8))
            .map { String(format: "%02x", $0) }
            .joined()

        return "desktop_voice_seed_\(seedDigest.prefix(24))"
    }

    func desktopVoiceEnrollRequestBuilder(
        _ promptState: DesktopVoiceEnrollPromptState
    ) throws -> DesktopVoiceEnrollIngressContext {
        guard let onboardingSessionID = boundedOnboardingContinueField(promptState.onboardingSessionID) else {
            throw BridgeError.invalidVoiceEnrollRequest(
                "the bounded desktop voice-enroll prompt state did not preserve a lawful onboarding_session_id"
            )
        }

        guard let nextStep = boundedOnboardingContinueField(promptState.nextStep),
              nextStep == "VOICE_ENROLL" else {
            throw BridgeError.invalidVoiceEnrollRequest(
                "bounded desktop voice-enroll lock is only lawful when canonical onboarding posture has advanced to exact `VOICE_ENROLL`"
            )
        }

        guard let managedDeviceID = boundedOnboardingContinueField(deviceID),
              let promptDeviceID = boundedOnboardingContinueField(promptState.deviceID),
              promptDeviceID == managedDeviceID else {
            throw BridgeError.invalidVoiceEnrollRequest(
                "bounded desktop voice-enroll lock must preserve the exact managed bridge `deviceID` only"
            )
        }

        let sampleSeed = try desktopVoiceEnrollSampleSeedBuilder(promptState)
        guard let boundedSampleSeed = boundedOnboardingContinueField(sampleSeed) else {
            throw BridgeError.invalidVoiceEnrollRequest(
                "the bounded desktop voice-enroll sample seed became invalid before canonical dispatch"
            )
        }

        let requestID = "desktop_voice_enroll_lock_request_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_voice_enroll_lock_\(onboardingSessionID)_\(managedDeviceID)_\(boundedSampleSeed)"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()
        let correlationID = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)

        let payload = OnboardingContinueAdapterRequestPayload(
            correlationID: correlationID,
            onboardingSessionID: onboardingSessionID,
            idempotencyKey: idempotencyKey,
            tenantID: tenantID,
            action: "VOICE_ENROLL_LOCK",
            fieldValue: nil,
            receiptKind: nil,
            receiptRef: nil,
            signer: nil,
            payloadHash: nil,
            termsVersionID: nil,
            accepted: nil,
            deviceID: managedDeviceID,
            proofOK: nil,
            sampleSeed: boundedSampleSeed,
            photoBlobRef: nil,
            senderDecision: nil
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/onboarding/continue")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: actorUserID, device: managedDeviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopVoiceEnrollIngressContext(
            onboardingSessionID: onboardingSessionID,
            deviceID: managedDeviceID,
            sampleSeed: boundedSampleSeed,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopWakeEnrollStartDraftRequestBuilder(
        _ promptState: DesktopWakeEnrollStartDraftPromptState
    ) throws -> DesktopWakeEnrollStartDraftIngressContext {
        guard let onboardingSessionID = boundedOnboardingContinueField(promptState.onboardingSessionID) else {
            throw BridgeError.invalidWakeEnrollStartDraftRequest(
                "the bounded desktop wake-enroll start-draft prompt state did not preserve a lawful onboarding_session_id"
            )
        }

        guard let nextStep = boundedOnboardingContinueField(promptState.nextStep),
              nextStep == "WAKE_ENROLL" else {
            throw BridgeError.invalidWakeEnrollStartDraftRequest(
                "bounded desktop wake-enroll start draft is only lawful when canonical onboarding posture has advanced to exact `WAKE_ENROLL`"
            )
        }

        guard let managedDeviceID = boundedOnboardingContinueField(deviceID),
              let promptDeviceID = boundedOnboardingContinueField(promptState.deviceID),
              promptDeviceID == managedDeviceID else {
            throw BridgeError.invalidWakeEnrollStartDraftRequest(
                "bounded desktop wake-enroll start draft must preserve the exact managed bridge `deviceID` only"
            )
        }

        let requestID = "desktop_wake_enroll_start_draft_request_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_wake_enroll_start_draft_\(onboardingSessionID)_\(managedDeviceID)"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()
        let correlationID = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)

        let payload = OnboardingContinueAdapterRequestPayload(
            correlationID: correlationID,
            onboardingSessionID: onboardingSessionID,
            idempotencyKey: idempotencyKey,
            tenantID: tenantID,
            action: "WAKE_ENROLL_START_DRAFT",
            fieldValue: nil,
            receiptKind: nil,
            receiptRef: nil,
            signer: nil,
            payloadHash: nil,
            termsVersionID: nil,
            accepted: nil,
            deviceID: managedDeviceID,
            proofOK: nil,
            sampleSeed: nil,
            photoBlobRef: nil,
            senderDecision: nil
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/onboarding/continue")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: actorUserID, device: managedDeviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopWakeEnrollStartDraftIngressContext(
            onboardingSessionID: onboardingSessionID,
            deviceID: managedDeviceID,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopWakeEnrollSampleCommitRequestBuilder(
        _ promptState: DesktopWakeEnrollSampleCommitPromptState
    ) throws -> DesktopWakeEnrollSampleCommitIngressContext {
        guard let onboardingSessionID = boundedOnboardingContinueField(promptState.onboardingSessionID) else {
            throw BridgeError.invalidWakeEnrollSampleCommitRequest(
                "the bounded desktop wake-enroll sample-commit prompt state did not preserve a lawful onboarding_session_id"
            )
        }

        guard let nextStep = boundedOnboardingContinueField(promptState.nextStep),
              nextStep == "WAKE_ENROLL" else {
            throw BridgeError.invalidWakeEnrollSampleCommitRequest(
                "bounded desktop wake-enroll sample commit is only lawful when canonical onboarding posture remains at exact `WAKE_ENROLL`"
            )
        }

        guard promptState.proofOK else {
            throw BridgeError.invalidWakeEnrollSampleCommitRequest(
                "bounded desktop wake-enroll sample commit must preserve exact `proofOK=true` only"
            )
        }

        guard let managedDeviceID = boundedOnboardingContinueField(deviceID),
              let promptDeviceID = boundedOnboardingContinueField(promptState.deviceID),
              promptDeviceID == managedDeviceID else {
            throw BridgeError.invalidWakeEnrollSampleCommitRequest(
                "bounded desktop wake-enroll sample commit must preserve the exact managed bridge `deviceID` only"
            )
        }

        let requestSuffix = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let requestID = "desktop_wake_enroll_sample_commit_request_\(requestSuffix)"
        let idempotencyKey = "desktop_wake_enroll_sample_commit_\(onboardingSessionID)_\(managedDeviceID)_\(requestSuffix)"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()
        let correlationID = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)

        let payload = OnboardingContinueAdapterRequestPayload(
            correlationID: correlationID,
            onboardingSessionID: onboardingSessionID,
            idempotencyKey: idempotencyKey,
            tenantID: tenantID,
            action: "WAKE_ENROLL_SAMPLE_COMMIT",
            fieldValue: nil,
            receiptKind: nil,
            receiptRef: nil,
            signer: nil,
            payloadHash: nil,
            termsVersionID: nil,
            accepted: nil,
            deviceID: managedDeviceID,
            proofOK: true,
            sampleSeed: nil,
            photoBlobRef: nil,
            senderDecision: nil
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/onboarding/continue")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: actorUserID, device: managedDeviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopWakeEnrollSampleCommitIngressContext(
            onboardingSessionID: onboardingSessionID,
            deviceID: managedDeviceID,
            proofOK: true,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopWakeEnrollCompleteCommitRequestBuilder(
        _ promptState: DesktopWakeEnrollCompleteCommitPromptState
    ) throws -> DesktopWakeEnrollCompleteCommitIngressContext {
        guard let onboardingSessionID = boundedOnboardingContinueField(promptState.onboardingSessionID) else {
            throw BridgeError.invalidWakeEnrollCompleteCommitRequest(
                "the bounded desktop wake-enroll complete-commit prompt state did not preserve a lawful onboarding_session_id"
            )
        }

        guard let nextStep = boundedOnboardingContinueField(promptState.nextStep),
              nextStep == "WAKE_ENROLL" else {
            throw BridgeError.invalidWakeEnrollCompleteCommitRequest(
                "bounded desktop wake-enroll complete commit is only lawful when canonical onboarding posture remains at exact `WAKE_ENROLL`"
            )
        }

        guard let managedDeviceID = boundedOnboardingContinueField(deviceID),
              let promptDeviceID = boundedOnboardingContinueField(promptState.deviceID),
              promptDeviceID == managedDeviceID else {
            throw BridgeError.invalidWakeEnrollCompleteCommitRequest(
                "bounded desktop wake-enroll complete commit must preserve the exact managed bridge `deviceID` only"
            )
        }

        let requestID = "desktop_wake_enroll_complete_commit_request_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_wake_enroll_complete_commit_\(onboardingSessionID)_\(managedDeviceID)"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()
        let correlationID = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)

        let payload = OnboardingContinueAdapterRequestPayload(
            correlationID: correlationID,
            onboardingSessionID: onboardingSessionID,
            idempotencyKey: idempotencyKey,
            tenantID: tenantID,
            action: desktopWakeEnrollCompleteCommitAction,
            fieldValue: nil,
            receiptKind: nil,
            receiptRef: nil,
            signer: nil,
            payloadHash: nil,
            termsVersionID: nil,
            accepted: nil,
            deviceID: managedDeviceID,
            proofOK: nil,
            sampleSeed: nil,
            photoBlobRef: nil,
            senderDecision: nil
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/onboarding/continue")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: actorUserID, device: managedDeviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopWakeEnrollCompleteCommitIngressContext(
            onboardingSessionID: onboardingSessionID,
            deviceID: managedDeviceID,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopWakeEnrollDeferCommitRequestBuilder(
        _ promptState: DesktopWakeEnrollDeferCommitPromptState
    ) throws -> DesktopWakeEnrollDeferCommitIngressContext {
        guard let onboardingSessionID = boundedOnboardingContinueField(promptState.onboardingSessionID) else {
            throw BridgeError.invalidWakeEnrollDeferCommitRequest(
                "the bounded desktop wake-enroll defer-commit prompt state did not preserve a lawful onboarding_session_id"
            )
        }

        guard let nextStep = boundedOnboardingContinueField(promptState.nextStep),
              nextStep == "WAKE_ENROLL" else {
            throw BridgeError.invalidWakeEnrollDeferCommitRequest(
                "bounded desktop wake-enroll defer commit is only lawful when canonical onboarding posture remains at exact `WAKE_ENROLL`"
            )
        }

        guard let managedDeviceID = boundedOnboardingContinueField(deviceID),
              let promptDeviceID = boundedOnboardingContinueField(promptState.deviceID),
              promptDeviceID == managedDeviceID else {
            throw BridgeError.invalidWakeEnrollDeferCommitRequest(
                "bounded desktop wake-enroll defer commit must preserve the exact managed bridge `deviceID` only"
            )
        }

        let requestID = "desktop_wake_enroll_defer_commit_request_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_wake_enroll_defer_commit_\(onboardingSessionID)_\(managedDeviceID)"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()
        let correlationID = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)

        let payload = OnboardingContinueAdapterRequestPayload(
            correlationID: correlationID,
            onboardingSessionID: onboardingSessionID,
            idempotencyKey: idempotencyKey,
            tenantID: tenantID,
            action: desktopWakeEnrollDeferCommitAction,
            fieldValue: nil,
            receiptKind: nil,
            receiptRef: nil,
            signer: nil,
            payloadHash: nil,
            termsVersionID: nil,
            accepted: nil,
            deviceID: managedDeviceID,
            proofOK: nil,
            sampleSeed: nil,
            photoBlobRef: nil,
            senderDecision: nil
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/onboarding/continue")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: actorUserID, device: managedDeviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopWakeEnrollDeferCommitIngressContext(
            onboardingSessionID: onboardingSessionID,
            deviceID: managedDeviceID,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopSessionAttachRequestBuilder(
        _ promptState: DesktopSessionAttachPromptState
    ) throws -> DesktopSessionAttachIngressContext {
        guard let sourceSurfaceIdentity = boundedOnboardingContinueField(promptState.sourceSurfaceIdentity),
              ["SESSION_OPEN_VISIBLE", "SESSION_ACTIVE_VISIBLE"].contains(sourceSurfaceIdentity) else {
            throw BridgeError.invalidSessionAttachRequest(
                "bounded desktop current-visible session attach must preserve exact `SESSION_OPEN_VISIBLE` or exact `SESSION_ACTIVE_VISIBLE` source surface identity"
            )
        }

        guard let sessionState = boundedOnboardingContinueField(promptState.sessionState),
              ["OPEN", "ACTIVE"].contains(sessionState) else {
            throw BridgeError.invalidSessionAttachRequest(
                "bounded desktop current-visible session attach is only lawful when canonical session posture remains exact `OPEN` or exact `ACTIVE`"
            )
        }

        guard let sessionID = boundedOnboardingContinueField(promptState.sessionID) else {
            throw BridgeError.invalidSessionAttachRequest(
                "the bounded desktop current-visible session attach prompt state did not preserve a lawful session_id"
            )
        }

        guard let managedDeviceID = boundedOnboardingContinueField(deviceID),
              let promptDeviceID = boundedOnboardingContinueField(promptState.deviceID),
              promptDeviceID == managedDeviceID else {
            throw BridgeError.invalidSessionAttachRequest(
                "bounded desktop current-visible session attach must preserve the exact managed bridge `deviceID` only"
            )
        }

        let boundedTurnID = boundedOnboardingContinueField(promptState.turnID)
        if sourceSurfaceIdentity == "SESSION_OPEN_VISIBLE", boundedTurnID != nil {
            throw BridgeError.invalidSessionAttachRequest(
                "bounded desktop current-visible session attach cannot preserve `turn_id` while exact `SESSION_OPEN_VISIBLE` is selected"
            )
        }
        if sourceSurfaceIdentity == "SESSION_ACTIVE_VISIBLE", boundedTurnID == nil {
            throw BridgeError.invalidSessionAttachRequest(
                "bounded desktop current-visible session attach must preserve exact `turn_id` while exact `SESSION_ACTIVE_VISIBLE` is selected"
            )
        }

        let requestID = "desktop_session_attach_request_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_session_attach_\(sessionID)_\(managedDeviceID)"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()
        let correlationID = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)

        let payload = SessionAttachAdapterRequestPayload(
            correlationID: correlationID,
            idempotencyKey: idempotencyKey,
            sessionID: sessionID,
            deviceID: managedDeviceID
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/session/attach")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: sessionID, device: managedDeviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopSessionAttachIngressContext(
            sourceSurfaceIdentity: sourceSurfaceIdentity,
            sessionState: sessionState,
            sessionID: sessionID,
            currentVisibleSessionAttachOutcome: boundedOnboardingContinueField(
                promptState.currentVisibleSessionAttachOutcome
            ),
            turnID: boundedTurnID,
            deviceID: managedDeviceID,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopSessionSoftClosedResumeRequestBuilder(
        _ promptState: DesktopSessionSoftClosedResumePromptState
    ) throws -> DesktopSessionSoftClosedResumeIngressContext {
        guard let sourceSurfaceIdentity = boundedOnboardingContinueField(promptState.sourceSurfaceIdentity),
              sourceSurfaceIdentity == "SESSION_SOFT_CLOSED_VISIBLE" else {
            throw BridgeError.invalidSessionSoftClosedResumeRequest(
                "bounded desktop soft-closed explicit resume must preserve exact `SESSION_SOFT_CLOSED_VISIBLE` source surface identity"
            )
        }

        guard let sessionState = boundedOnboardingContinueField(promptState.sessionState),
              sessionState == "SOFT_CLOSED" else {
            throw BridgeError.invalidSessionSoftClosedResumeRequest(
                "bounded desktop soft-closed explicit resume is only lawful when canonical session posture remains exact `SOFT_CLOSED`"
            )
        }

        guard let sessionID = boundedOnboardingContinueField(promptState.sessionID) else {
            throw BridgeError.invalidSessionSoftClosedResumeRequest(
                "the bounded desktop soft-closed explicit resume prompt state did not preserve a lawful session_id"
            )
        }

        guard let managedDeviceID = boundedOnboardingContinueField(deviceID),
              let promptDeviceID = boundedOnboardingContinueField(promptState.deviceID),
              promptDeviceID == managedDeviceID else {
            throw BridgeError.invalidSessionSoftClosedResumeRequest(
                "bounded desktop soft-closed explicit resume must preserve the exact managed bridge `deviceID` only"
            )
        }

        let requestID = "desktop_session_soft_closed_resume_request_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_session_soft_closed_resume_\(sessionID)_\(managedDeviceID)"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()
        let correlationID = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)

        let payload = SessionResumeAdapterRequestPayload(
            correlationID: correlationID,
            idempotencyKey: idempotencyKey,
            sessionID: sessionID,
            deviceID: managedDeviceID
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/session/resume")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: sessionID, device: managedDeviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopSessionSoftClosedResumeIngressContext(
            sourceSurfaceIdentity: sourceSurfaceIdentity,
            sessionState: sessionState,
            sessionID: sessionID,
            selectedThreadID: boundedOnboardingContinueField(promptState.selectedThreadID),
            selectedThreadTitle: boundedOnboardingContinueField(promptState.selectedThreadTitle),
            pendingWorkOrderID: boundedOnboardingContinueField(promptState.pendingWorkOrderID),
            resumeTier: boundedOnboardingContinueField(promptState.resumeTier),
            resumeSummaryBullets: boundedOnboardingContinueList(promptState.resumeSummaryBullets),
            deviceID: managedDeviceID,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopSessionRecoverRequestBuilder(
        _ promptState: DesktopSessionRecoverPromptState
    ) throws -> DesktopSessionRecoverIngressContext {
        guard let sourceSurfaceIdentity = boundedOnboardingContinueField(promptState.sourceSurfaceIdentity),
              sourceSurfaceIdentity == "SESSION_SUSPENDED_VISIBLE" else {
            throw BridgeError.invalidSessionRecoverRequest(
                "bounded desktop suspended-session recover submission must preserve exact `SESSION_SUSPENDED_VISIBLE` source surface identity"
            )
        }

        guard let sessionState = boundedOnboardingContinueField(promptState.sessionState),
              sessionState == "SUSPENDED" else {
            throw BridgeError.invalidSessionRecoverRequest(
                "bounded desktop suspended-session recover submission is only lawful when canonical session posture remains exact `SUSPENDED`"
            )
        }

        guard let sessionID = boundedOnboardingContinueField(promptState.sessionID) else {
            throw BridgeError.invalidSessionRecoverRequest(
                "the bounded desktop suspended-session recover prompt state did not preserve a lawful session_id"
            )
        }

        guard let managedDeviceID = boundedOnboardingContinueField(deviceID),
              let promptDeviceID = boundedOnboardingContinueField(promptState.deviceID),
              promptDeviceID == managedDeviceID else {
            throw BridgeError.invalidSessionRecoverRequest(
                "bounded desktop suspended-session recover submission must preserve the exact managed bridge `deviceID` only"
            )
        }

        let requestID = "desktop_session_recover_request_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_session_recover_\(sessionID)_\(managedDeviceID)"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()
        let correlationID = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)

        let payload = SessionRecoverAdapterRequestPayload(
            correlationID: correlationID,
            idempotencyKey: idempotencyKey,
            sessionID: sessionID,
            deviceID: managedDeviceID
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/session/recover")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: actorUserID, device: managedDeviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopSessionRecoverIngressContext(
            sourceSurfaceIdentity: sourceSurfaceIdentity,
            sessionState: sessionState,
            sessionID: sessionID,
            recoveryMode: boundedOnboardingContinueField(promptState.recoveryMode),
            deviceID: managedDeviceID,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopSessionMultiPostureResumeRequestBuilder(
        _ promptState: DesktopSessionMultiPostureResumePromptState
    ) throws -> DesktopSessionMultiPostureResumeIngressContext {
        switch promptState.resumeMode {
        case .softClosedExplicitResume:
            guard boundedOnboardingContinueField(promptState.recoveryMode) == nil else {
                throw BridgeError.invalidSessionMultiPostureResumeRequest(
                    "bounded desktop multi-posture session-resume prompt state cannot preserve suspended-session `recovery_mode` while exact `SOFT_CLOSED_EXPLICIT_RESUME` is selected"
                )
            }

            let softClosedPromptState = DesktopSessionSoftClosedResumePromptState(
                sourceSurfaceIdentity: promptState.sourceSurfaceIdentity,
                sessionState: promptState.sessionState,
                sessionID: promptState.sessionID,
                selectedThreadID: promptState.selectedThreadID,
                selectedThreadTitle: promptState.selectedThreadTitle,
                pendingWorkOrderID: promptState.pendingWorkOrderID,
                resumeTier: promptState.resumeTier,
                resumeSummaryBullets: promptState.resumeSummaryBullets,
                deviceID: promptState.deviceID
            )
            let ingressContext = try desktopSessionSoftClosedResumeRequestBuilder(softClosedPromptState)

            return DesktopSessionMultiPostureResumeIngressContext(
                resumeMode: .softClosedExplicitResume,
                sourceSurfaceIdentity: ingressContext.sourceSurfaceIdentity,
                sessionState: ingressContext.sessionState,
                sessionID: ingressContext.sessionID,
                selectedThreadID: ingressContext.selectedThreadID,
                selectedThreadTitle: ingressContext.selectedThreadTitle,
                pendingWorkOrderID: ingressContext.pendingWorkOrderID,
                resumeTier: ingressContext.resumeTier,
                resumeSummaryBullets: ingressContext.resumeSummaryBullets,
                recoveryMode: nil,
                deviceID: ingressContext.deviceID,
                requestID: ingressContext.requestID,
                endpoint: ingressContext.endpoint,
                softClosedResumeIngressContext: ingressContext,
                sessionRecoverIngressContext: nil
            )

        case .suspendedAuthoritativeRereadRecover:
            guard boundedOnboardingContinueField(promptState.selectedThreadID) == nil,
                  boundedOnboardingContinueField(promptState.selectedThreadTitle) == nil,
                  boundedOnboardingContinueField(promptState.pendingWorkOrderID) == nil,
                  boundedOnboardingContinueField(promptState.resumeTier) == nil,
                  boundedOnboardingContinueList(promptState.resumeSummaryBullets).isEmpty else {
                throw BridgeError.invalidSessionMultiPostureResumeRequest(
                    "bounded desktop multi-posture session-resume prompt state cannot preserve soft-closed explicit resume fields while exact `SUSPENDED_AUTHORITATIVE_REREAD_RECOVER` is selected"
                )
            }

            let sessionRecoverPromptState = DesktopSessionRecoverPromptState(
                sourceSurfaceIdentity: promptState.sourceSurfaceIdentity,
                sessionState: promptState.sessionState,
                sessionID: promptState.sessionID,
                recoveryMode: promptState.recoveryMode,
                deviceID: promptState.deviceID
            )
            let ingressContext = try desktopSessionRecoverRequestBuilder(sessionRecoverPromptState)

            return DesktopSessionMultiPostureResumeIngressContext(
                resumeMode: .suspendedAuthoritativeRereadRecover,
                sourceSurfaceIdentity: ingressContext.sourceSurfaceIdentity,
                sessionState: ingressContext.sessionState,
                sessionID: ingressContext.sessionID,
                selectedThreadID: nil,
                selectedThreadTitle: nil,
                pendingWorkOrderID: nil,
                resumeTier: nil,
                resumeSummaryBullets: [],
                recoveryMode: ingressContext.recoveryMode,
                deviceID: ingressContext.deviceID,
                requestID: ingressContext.requestID,
                endpoint: ingressContext.endpoint,
                softClosedResumeIngressContext: nil,
                sessionRecoverIngressContext: ingressContext
            )
        }
    }

    func desktopWakeProfileAvailabilityRequestBuilder(
        _ promptState: DesktopWakeProfileAvailabilityPromptState
    ) throws -> DesktopWakeProfileAvailabilityIngressContext {
        guard let receiptKind = boundedOnboardingContinueField(promptState.receiptKind),
              receiptKind == "desktop_wakeword_configured" else {
            throw BridgeError.invalidWakeProfileAvailabilityRequest(
                "bounded desktop wake-profile local-availability refresh must preserve exact `desktop_wakeword_configured` receipt posture"
            )
        }

        guard let managedDeviceID = boundedOnboardingContinueField(deviceID),
              let promptDeviceID = boundedOnboardingContinueField(promptState.deviceID),
              promptDeviceID == managedDeviceID else {
            throw BridgeError.invalidWakeProfileAvailabilityRequest(
                "bounded desktop wake-profile local-availability refresh must preserve the exact managed bridge `deviceID` only"
            )
        }

        guard let wakeProfileID = boundedOnboardingContinueField(promptState.wakeProfileID) else {
            throw BridgeError.invalidWakeProfileAvailabilityRequest(
                "the bounded desktop wake-profile local-availability refresh prompt state did not preserve a lawful wake_profile_id"
            )
        }

        guard let voiceArtifactSyncReceiptRef = boundedOnboardingContinueField(
            promptState.voiceArtifactSyncReceiptRef
        ) else {
            throw BridgeError.invalidWakeProfileAvailabilityRequest(
                "the bounded desktop wake-profile local-availability refresh prompt state did not preserve a lawful voice_artifact_sync_receipt_ref"
            )
        }

        let requestID = "desktop_wake_profile_availability_request_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_wake_profile_availability_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()
        let correlationID = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)

        let payload = WakeProfileAvailabilityRefreshAdapterRequestPayload(
            correlationID: correlationID,
            idempotencyKey: idempotencyKey,
            deviceID: managedDeviceID,
            expectedWakeProfileID: wakeProfileID,
            voiceArtifactSyncReceiptRef: voiceArtifactSyncReceiptRef
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/wake-profile/availability")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: wakeProfileID, device: managedDeviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopWakeProfileAvailabilityIngressContext(
            receiptKind: receiptKind,
            deviceID: managedDeviceID,
            wakeProfileID: wakeProfileID,
            voiceArtifactSyncReceiptRef: voiceArtifactSyncReceiptRef,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopEmoPersonaLockRequestBuilder(
        _ promptState: DesktopEmoPersonaLockPromptState
    ) throws -> DesktopEmoPersonaLockIngressContext {
        guard let onboardingSessionID = boundedOnboardingContinueField(promptState.onboardingSessionID) else {
            throw BridgeError.invalidEmoPersonaLockRequest(
                "the bounded desktop emo/persona-lock prompt state did not preserve a lawful onboarding_session_id"
            )
        }

        guard let nextStep = boundedOnboardingContinueField(promptState.nextStep),
              nextStep == "EMO_PERSONA_LOCK" else {
            throw BridgeError.invalidEmoPersonaLockRequest(
                "bounded desktop emo/persona lock is only lawful when canonical onboarding posture remains at exact `EMO_PERSONA_LOCK`"
            )
        }

        let requestID = "desktop_emo_persona_lock_request_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_emo_persona_lock_\(onboardingSessionID)"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()
        let correlationID = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)

        let payload = OnboardingContinueAdapterRequestPayload(
            correlationID: correlationID,
            onboardingSessionID: onboardingSessionID,
            idempotencyKey: idempotencyKey,
            tenantID: tenantID,
            action: desktopEmoPersonaLockAction,
            fieldValue: nil,
            receiptKind: nil,
            receiptRef: nil,
            signer: nil,
            payloadHash: nil,
            termsVersionID: nil,
            accepted: nil,
            deviceID: nil,
            proofOK: nil,
            sampleSeed: nil,
            photoBlobRef: nil,
            senderDecision: nil
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/onboarding/continue")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: actorUserID, device: deviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopEmoPersonaLockIngressContext(
            onboardingSessionID: onboardingSessionID,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopAccessProvisionCommitRequestBuilder(
        _ promptState: DesktopAccessProvisionCommitPromptState
    ) throws -> DesktopAccessProvisionCommitIngressContext {
        guard let onboardingSessionID = boundedOnboardingContinueField(promptState.onboardingSessionID) else {
            throw BridgeError.invalidAccessProvisionCommitRequest(
                "the bounded desktop access-provision prompt state did not preserve a lawful onboarding_session_id"
            )
        }

        guard let nextStep = boundedOnboardingContinueField(promptState.nextStep),
              nextStep == "ACCESS_PROVISION" else {
            throw BridgeError.invalidAccessProvisionCommitRequest(
                "bounded desktop access provision commit is only lawful when canonical onboarding posture remains at exact `ACCESS_PROVISION`"
            )
        }

        let requestID = "desktop_access_provision_commit_request_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_access_provision_commit_\(onboardingSessionID)"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()
        let correlationID = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)

        let payload = OnboardingContinueAdapterRequestPayload(
            correlationID: correlationID,
            onboardingSessionID: onboardingSessionID,
            idempotencyKey: idempotencyKey,
            tenantID: tenantID,
            action: desktopAccessProvisionCommitAction,
            fieldValue: nil,
            receiptKind: nil,
            receiptRef: nil,
            signer: nil,
            payloadHash: nil,
            termsVersionID: nil,
            accepted: nil,
            deviceID: nil,
            proofOK: nil,
            sampleSeed: nil,
            photoBlobRef: nil,
            senderDecision: nil
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/onboarding/continue")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: actorUserID, device: deviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopAccessProvisionCommitIngressContext(
            onboardingSessionID: onboardingSessionID,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    func desktopCompleteCommitRequestBuilder(
        _ promptState: DesktopCompleteCommitPromptState
    ) throws -> DesktopCompleteCommitIngressContext {
        guard let onboardingSessionID = boundedOnboardingContinueField(promptState.onboardingSessionID) else {
            throw BridgeError.invalidCompleteCommitRequest(
                "the bounded desktop complete prompt state did not preserve a lawful onboarding_session_id"
            )
        }

        guard let nextStep = boundedOnboardingContinueField(promptState.nextStep),
              nextStep == "COMPLETE" else {
            throw BridgeError.invalidCompleteCommitRequest(
                "bounded desktop complete commit is only lawful when canonical onboarding posture remains at exact `COMPLETE`"
            )
        }

        let requestID = "desktop_complete_commit_request_\(UUID().uuidString.replacingOccurrences(of: "-", with: ""))"
        let idempotencyKey = "desktop_complete_commit_\(onboardingSessionID)"
        let nonce = UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let timestampMS = Self.systemTimeNowMS()
        let correlationID = Swift.max(DispatchTime.now().uptimeNanoseconds, 1)

        let payload = OnboardingContinueAdapterRequestPayload(
            correlationID: correlationID,
            onboardingSessionID: onboardingSessionID,
            idempotencyKey: idempotencyKey,
            tenantID: tenantID,
            action: desktopCompleteCommitAction,
            fieldValue: nil,
            receiptKind: nil,
            receiptRef: nil,
            signer: nil,
            payloadHash: nil,
            termsVersionID: nil,
            accepted: nil,
            deviceID: nil,
            proofOK: nil,
            sampleSeed: nil,
            photoBlobRef: nil,
            senderDecision: nil
        )

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let body = try encoder.encode(payload)
        let endpointURL = adapterBaseURL.appendingPathComponent("v1/onboarding/continue")
        var urlRequest = URLRequest(url: endpointURL)
        urlRequest.httpMethod = "POST"
        urlRequest.httpBody = body
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(requestID, forHTTPHeaderField: "x-request-id")
        urlRequest.setValue(idempotencyKey, forHTTPHeaderField: "idempotency-key")
        urlRequest.setValue(String(timestampMS), forHTTPHeaderField: "x-selene-timestamp-ms")
        urlRequest.setValue(nonce, forHTTPHeaderField: "x-selene-nonce")
        urlRequest.setValue(
            Self.bearerToken(subject: actorUserID, device: deviceID),
            forHTTPHeaderField: "Authorization"
        )

        return DesktopCompleteCommitIngressContext(
            onboardingSessionID: onboardingSessionID,
            requestID: requestID,
            endpoint: endpointURL.absoluteString,
            urlRequest: urlRequest
        )
    }

    var voiceTurnEndpoint: String {
        adapterBaseURL.appendingPathComponent("v1/voice/turn").absoluteString
    }

    var inviteClickEndpoint: String {
        adapterBaseURL.appendingPathComponent("v1/invite/click").absoluteString
    }

    var onboardingContinueEndpoint: String {
        adapterBaseURL.appendingPathComponent("v1/onboarding/continue").absoluteString
    }

    var sessionAttachEndpoint: String {
        adapterBaseURL.appendingPathComponent("v1/session/attach").absoluteString
    }

    var sessionResumeEndpoint: String {
        adapterBaseURL.appendingPathComponent("v1/session/resume").absoluteString
    }

    var sessionRecoverEndpoint: String {
        adapterBaseURL.appendingPathComponent("v1/session/recover").absoluteString
    }

    var wakeProfileAvailabilityEndpoint: String {
        adapterBaseURL.appendingPathComponent("v1/wake-profile/availability").absoluteString
    }

    var managedDeviceID: String {
        deviceID
    }

    private func ensureAdapterAvailable() async throws {
        if await adapterHealthCheck() {
            return
        }

        try startManagedAdapterIfNeeded()

        for _ in 0..<120 {
            if await adapterHealthCheck() {
                return
            }

            if let managedAdapterProcess, !managedAdapterProcess.isRunning {
                throw BridgeError.adapterStartFailed(
                    "the managed selene_adapter_http process exited before the canonical runtime bridge became healthy"
                )
            }

            try await Task.sleep(nanoseconds: 500_000_000)
        }

        throw BridgeError.adapterUnavailable(
            "the canonical runtime bridge did not become healthy at \(adapterBaseURL.absoluteString) within the bounded startup window"
        )
    }

    private func startManagedAdapterIfNeeded() throws {
        if let managedAdapterProcess, managedAdapterProcess.isRunning {
            return
        }

        let cargoExecutable = URL(fileURLWithPath: "/usr/bin/env")
        let process = Process()
        process.executableURL = cargoExecutable
        process.currentDirectoryURL = repoRootURL
        process.arguments = ["cargo", "run", "--quiet", "-p", "selene_adapter", "--bin", "selene_adapter_http"]

        var environment = ProcessInfo.processInfo.environment
        let bindValue = Self.bindValue(for: adapterBaseURL)
        environment["SELENE_HTTP_BIND"] = bindValue
        environment["SELENE_ADAPTER_SYNC_WORKER_ENABLED"] = "false"
        process.environment = environment
        process.standardOutput = FileHandle(forWritingAtPath: "/dev/null")
        process.standardError = FileHandle(forWritingAtPath: "/dev/null")

        do {
            try process.run()
            managedAdapterProcess = process
        } catch {
            throw BridgeError.adapterStartFailed(
                "failed to launch the managed selene_adapter_http process from \(repoRootURL.path): \(error.localizedDescription)"
            )
        }
    }

    private func adapterHealthCheck() async -> Bool {
        let healthURL = adapterBaseURL.appendingPathComponent("healthz")
        var request = URLRequest(url: healthURL)
        request.httpMethod = "GET"
        request.timeoutInterval = 2

        do {
            let (_, response) = try await urlSession.data(for: request)
            let statusCode = (response as? HTTPURLResponse)?.statusCode ?? 0
            return statusCode == 200
        } catch {
            return false
        }
    }

    private static func resolveActorUserID(processInfo: ProcessInfo) -> String {
        nonEmpty(processInfo.environment["SELENE_DESKTOP_ACTOR_USER_ID"]) ?? "tenant_a:user_ingress_test"
    }

    private static func resolveTenantID(processInfo: ProcessInfo) -> String? {
        nonEmpty(processInfo.environment["SELENE_DESKTOP_TENANT_ID"]) ?? "tenant_a"
    }

    private static func resolveDeviceID(processInfo: ProcessInfo) -> String {
        nonEmpty(processInfo.environment["SELENE_DESKTOP_DEVICE_ID"]) ?? "ingress_device_01"
    }

    private static func resolveAdapterBaseURL(processInfo: ProcessInfo) -> URL {
        if let override = nonEmpty(processInfo.environment["SELENE_DESKTOP_BRIDGE_BASE_URL"]),
           let url = URL(string: override),
           url.scheme?.hasPrefix("http") == true {
            return url
        }

        return URL(string: "http://127.0.0.1:18765/")!
    }

    private static func bindValue(for baseURL: URL) -> String {
        let host = baseURL.host ?? "127.0.0.1"
        let port = baseURL.port ?? 18765
        return "\(host):\(port)"
    }

    private static func resolveRepoRoot(processInfo: ProcessInfo) -> URL {
        if let override = nonEmpty(processInfo.environment["SELENE_REPO_ROOT"]) {
            return URL(fileURLWithPath: override, isDirectory: true)
        }

        let sourceURL = URL(fileURLWithPath: #filePath)
        return sourceURL
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
    }

    private static func systemTimeNowMS() -> UInt64 {
        UInt64(Date().timeIntervalSince1970 * 1_000)
    }

    private static func nonEmpty(_ value: String?) -> String? {
        guard let value else {
            return nil
        }

        let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? nil : trimmed
    }

    private static func bearerToken(subject: String, device: String) -> String {
        let keyID = "ingress_kid_v1"
        let secret = ProcessInfo.processInfo.environment["SELENE_INGRESS_AUTH_SIGNING_KEYS"]
            .flatMap { parseKeySecret(raw: $0, keyID: keyID) } ?? "selene_ingress_local_dev_secret_v1"
        let digest = deterministicBearerDigest(subject: subject, device: device, keyID: keyID, secret: secret)
        return "Bearer v1.\(keyID).\(subject).\(device).\(digest)"
    }

    private static func parseKeySecret(raw: String, keyID: String) -> String? {
        raw
            .split(separator: ",")
            .compactMap { entry -> String? in
                let parts = entry.split(separator: ":", maxSplits: 1)
                guard parts.count == 2 else {
                    return nil
                }

                let id = parts[0].trimmingCharacters(in: .whitespacesAndNewlines)
                let secret = parts[1].trimmingCharacters(in: .whitespacesAndNewlines)
                return id == keyID ? secret : nil
            }
            .first
    }

    private static func deterministicBearerDigest(
        subject: String,
        device: String,
        keyID: String,
        secret: String
    ) -> String {
        hashHex64(bytes: Array("v1|\(keyID)|\(subject)|\(device)|\(secret)".utf8))
    }

    private static func hashHex64(bytes: [UInt8]) -> String {
        var hash = fnv1a64(bytes: bytes)
        if hash == 0 {
            hash = 1
        }

        return String(format: "%016llx", hash)
    }

    private static func fnv1a64(bytes: [UInt8]) -> UInt64 {
        let offset: UInt64 = 0xcbf29ce484222325
        let prime: UInt64 = 0x100000001b3
        return bytes.reduce(offset) { partial, byte in
            (partial ^ UInt64(byte)) &* prime
        }
    }
}
