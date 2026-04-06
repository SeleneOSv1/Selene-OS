import AVFoundation
import Foundation
import Speech
import SwiftUI

final class ExplicitEntryRouter: ObservableObject {
    @Published private(set) var latestContext: ExplicitEntryContext?

    func receive(url: URL) {
        guard let context = ExplicitEntryContext(url: url) else {
            return
        }

        latestContext = context
    }
}

private enum ShellDisplayState: String {
    case explicitEntryReady = "EXPLICIT_ENTRY_READY"
    case onboardingEntryActive = "ONBOARDING_ENTRY_ACTIVE"

    var title: String {
        rawValue
    }

    var detail: String {
        switch self {
        case .explicitEntryReady:
            return "The iPhone shell is waiting for lawful explicit entry through canonical app-open / invite-open ingress."
        case .onboardingEntryActive:
            return "A lawful app-open / invite-open route has been parsed and is being rendered as a bounded onboarding-entry takeover surface with read-only onboarding outcome, onboarding_status, prompt-state, artifact/access identifier, and remaining platform-receipt context only."
        }
    }
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

private struct RecentThreadPreviewEntry: Identifiable {
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
    @State private var typedTurnDraft: String = ""
    @State private var typedTurnPendingRequest: TypedTurnRequestState?
    @State private var typedTurnFailedRequest: OperationalQueueEntry?
    @State private var typedTurnRequestSequence: Int = 0
    @StateObject private var explicitVoiceController = ExplicitVoiceCaptureController()

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
                } else {
                    explicitEntryReadyCard
                }

                setupReceiptCard
                boundedSurfaceCard(
                    title: "Session",
                    detail: "One dominant session surface remains primary. Bounded typed-turn request production now lives here while authoritative transcript acceptance and response remain cloud-side."
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

            activeContext = newContext
            displayState = .onboardingEntryActive
        }
        .onChange(of: displayState) { _, newState in
            if newState != .explicitEntryReady {
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

                Text("H84 preserves the H79 recent thread window, preserves the H83 typed-turn request production posture, advances the explicit voice entry affordance into bounded explicit voice-turn request production, preserves the H80 history side-drawer recall, incremental history expansion, and archived session recall, preserves the H81 System Activity operational queue with separate Pending and Failed visibility, and preserves the H82 Needs Attention actionable queue while preserving the H74, H75, H76, and H77 takeover surfaces.")
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

                Text("This shell remains session-bound and cloud-authoritative for onboarding, identity, governance, runtime law, and authoritative transcript state while the typed and explicit voice surfaces only produce bounded explicit turn requests.")
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

                Text("H84 keeps `EXPLICIT_ENTRY_READY` as the dominant bounded session surface. Recent thread, typed input, explicit voice, history recall, `System Activity`, and `Needs Attention` remain bounded, `EXPLICIT_ONLY`, session-bound, and cloud-authoritative while typed input and explicit voice now produce bounded explicit turn requests.")
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

        if let explicitVoiceFailedRequest = explicitVoiceController.failedRequest {
            entries.insert(explicitVoiceFailedRequest, at: 0)
        }

        if let typedTurnFailedRequest {
            entries.insert(typedTurnFailedRequest, at: 0)
        }

        return entries
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
