import Combine
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

final class DesktopCanonicalRuntimeBridge: ObservableObject {
    private enum BridgeError: LocalizedError {
        case invalidPreparedRequest(String)
        case invalidAdapterBind(String)
        case adapterStartFailed(String)
        case adapterUnavailable(String)
        case requestEncodingFailed(String)
        case responseDecodingFailed(String)
        case transportFailed(String)

        var errorDescription: String? {
            switch self {
            case .invalidPreparedRequest(let detail),
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

    struct DesktopInviteOpenIngressContext {
        let entryContextID: String
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
        let audioCaptureRef: String?
        let visualInputRef: String?
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
            audioCaptureRef: nil,
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

    var voiceTurnEndpoint: String {
        adapterBaseURL.appendingPathComponent("v1/voice/turn").absoluteString
    }

    var inviteClickEndpoint: String {
        adapterBaseURL.appendingPathComponent("v1/invite/click").absoluteString
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
