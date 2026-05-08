import AppKit
import AVFoundation
import Foundation
import SwiftUI

struct DesktopControlledWakeModeView: View {
    @StateObject private var controller = DesktopControlledWakeModeController()
    @State private var speakerName = "JD"
    @State private var wakeText = "Selene"
    @State private var listenSeconds = 5

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            VStack(alignment: .leading, spacing: 6) {
                Text("Controlled Wake Mode")
                    .font(.headline)

                Text("Runtime authoritative. Desktop capture/render only. Wake is activation only. Protected execution closed.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }

            Toggle(isOn: controller.controlledWakeBinding) {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Controlled Wake Mode")
                        .font(.subheadline.weight(.semibold))
                    Text("Microphone active only while controlled wake mode is on.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
            .toggleStyle(.switch)

            HStack(spacing: 10) {
                TextField("Speaker name", text: $speakerName)
                    .textFieldStyle(.roundedBorder)
                TextField("Wake text", text: $wakeText)
                    .textFieldStyle(.roundedBorder)
                Stepper("Wake seconds \(listenSeconds)", value: $listenSeconds, in: 3...30)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            HStack(spacing: 10) {
                Button {
                    controller.startControlledWake(
                        wakeText: wakeText,
                        listenSeconds: listenSeconds
                    )
                } label: {
                    Label("Run live wake", systemImage: "waveform.badge.mic")
                }
                .disabled(controller.isRunning)

                Button {
                    controller.runQuietControl(
                        wakeText: wakeText,
                        listenSeconds: listenSeconds
                    )
                } label: {
                    Label("Run quiet control", systemImage: "speaker.slash")
                }
                .disabled(controller.isRunning)

                Button {
                    controller.runKnownSpeakerProof(
                        speakerName: speakerName,
                        wakeText: wakeText
                    )
                } label: {
                    Label("Known JD proof", systemImage: "person.wave.2")
                }
                .disabled(controller.isRunning)

                Button {
                    controller.runEchoGuardProof(
                        wakeText: wakeText,
                        listenSeconds: listenSeconds
                    )
                } label: {
                    Label("Echo guard", systemImage: "speaker.badge.exclamationmark")
                }
                .disabled(controller.isRunning)

                if controller.isRunning {
                    ProgressView()
                        .controlSize(.small)
                }
            }

            controlledWakeStatusBody
        }
        .padding(14)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(Color(nsColor: .controlBackgroundColor).opacity(0.55))
        .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .stroke(Color.primary.opacity(0.08), lineWidth: 1)
        )
        .onDisappear {
            controller.stopControlledWake(reason: "view disappeared")
        }
    }

    @ViewBuilder
    private var controlledWakeStatusBody: some View {
        switch controller.state {
        case .off:
            infoRows([
                ("state", "Off"),
                ("privacy", "wake listener off"),
                ("mic permission", controller.microphonePermissionLabel),
                ("resource proof", "available through bounded wake proof runner"),
            ])
        case .listening(let startedAt):
            infoRows([
                ("state", "Listening"),
                ("privacy", "listening for wake"),
                ("started", startedAt),
                ("posture", "bounded foreground listener"),
            ])
        case .stopped(let reason):
            infoRows([
                ("state", "Stopped"),
                ("privacy", "wake listener off"),
                ("reason", reason),
            ])
        case .failed(let summary, let detail):
            VStack(alignment: .leading, spacing: 8) {
                Text(summary)
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(.red)
                Text(detail)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
        case .result(let result):
            resultView(result)
        }
    }

    private func resultView(_ result: DesktopControlledWakeModeResult) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            infoRows([
                ("status", result.status),
                ("state", result.stateLabel),
                ("privacy", result.privacyLabel),
                ("wake accepted", result.boolean(result.wakeAccepted)),
                ("session opened", result.boolean(result.sessionOpened)),
                ("recognized", result.boolean(result.recognized)),
                ("posture", result.voicePosture),
                ("named greeting", result.boolean(result.namedGreeting)),
                ("source chips", String(result.sourceChips)),
                ("provider paths", String(result.providerCalls)),
                ("protected execution", result.boolean(result.protectedExecution)),
                ("memory_write", result.boolean(result.memoryWrite)),
                ("authority", result.boolean(result.authorityGrant)),
                ("cpu", result.cpuLabel),
                ("rss", result.rssLabel),
                ("timeout", result.timeoutLabel),
            ])

            if !result.greetingText.isEmpty {
                VStack(alignment: .leading, spacing: 5) {
                    Text("Greeting")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.secondary)
                    Text(result.greetingText)
                        .font(.body)
                        .fixedSize(horizontal: false, vertical: true)
                }
            }

            VStack(alignment: .leading, spacing: 6) {
                Text("Gates")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
                ForEach(result.gates) { gate in
                    HStack(alignment: .firstTextBaseline, spacing: 8) {
                        Image(systemName: gate.passed ? "checkmark.circle.fill" : "xmark.octagon.fill")
                            .foregroundStyle(gate.passed ? .green : .red)
                        Text(gate.name)
                            .font(.caption)
                        Spacer(minLength: 8)
                        Text(gate.status)
                            .font(.caption.weight(.semibold))
                    }
                }
            }
        }
    }

    private func infoRows(_ rows: [(String, String)]) -> some View {
        Grid(alignment: .leadingFirstTextBaseline, horizontalSpacing: 12, verticalSpacing: 6) {
            ForEach(rows, id: \.0) { row in
                GridRow {
                    Text(row.0)
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.secondary)
                    Text(row.1)
                        .font(.caption)
                        .textSelection(.enabled)
                }
            }
        }
    }
}

@MainActor
final class DesktopControlledWakeModeController: ObservableObject {
    enum State {
        case off
        case listening(String)
        case stopped(String)
        case failed(String, String)
        case result(DesktopControlledWakeModeResult)
    }

    @Published private(set) var state: State = .off
    @Published private(set) var isRunning = false
    @Published private(set) var isEnabled = false
    @Published private(set) var microphonePermissionLabel = DesktopControlledWakeModeRunner.microphonePermissionLabel()

    var controlledWakeBinding: Binding<Bool> {
        Binding(
            get: { self.isEnabled },
            set: { enabled in
                if enabled {
                    self.startControlledWake(wakeText: "Selene", listenSeconds: 5)
                } else {
                    self.stopControlledWake(reason: "explicit user toggle off")
                }
            }
        )
    }

    func startControlledWake(wakeText: String, listenSeconds: Int) {
        guard !isRunning else {
            return
        }
        run(mode: .liveWake, wakeText: wakeText, listenSeconds: listenSeconds)
    }

    func runQuietControl(wakeText: String, listenSeconds: Int) {
        guard !isRunning else {
            return
        }
        run(mode: .quietControl, wakeText: wakeText, listenSeconds: listenSeconds)
    }

    func runKnownSpeakerProof(speakerName: String, wakeText: String) {
        guard !isRunning else {
            return
        }
        run(mode: .knownSpeaker(speakerName: speakerName), wakeText: wakeText, listenSeconds: 5)
    }

    func runEchoGuardProof(wakeText: String, listenSeconds: Int) {
        guard !isRunning else {
            return
        }
        run(mode: .echoGuard, wakeText: wakeText, listenSeconds: listenSeconds)
    }

    func stopControlledWake(reason: String) {
        DesktopControlledWakeModeRunner.terminateActiveProcess()
        isEnabled = false
        isRunning = false
        state = .stopped(reason)
    }

    private func run(
        mode: DesktopControlledWakeModeRunner.Mode,
        wakeText: String,
        listenSeconds: Int
    ) {
        microphonePermissionLabel = DesktopControlledWakeModeRunner.microphonePermissionLabel()
        guard DesktopControlledWakeModeRunner.microphonePermissionAllowsStart() else {
            isEnabled = false
            state = .failed(
                "Controlled Wake Mode failed closed",
                "Microphone permission is \(microphonePermissionLabel). macOS microphone permission is required before controlled wake capture can start."
            )
            return
        }

        isEnabled = mode.startsListener
        isRunning = true
        state = .listening(Self.timestampLabel())

        Task {
            do {
                let result = try await DesktopControlledWakeModeRunner.run(
                    mode: mode,
                    wakeText: wakeText,
                    listenSeconds: listenSeconds
                )
                state = .result(result)
            } catch {
                state = .failed("Controlled Wake Mode failed closed", error.localizedDescription)
            }
            isEnabled = false
            isRunning = false
            microphonePermissionLabel = DesktopControlledWakeModeRunner.microphonePermissionLabel()
        }
    }

    private static func timestampLabel() -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = .none
        formatter.timeStyle = .medium
        return formatter.string(from: Date())
    }
}

enum DesktopControlledWakeModeRunner {
    enum Mode: Equatable {
        case liveWake
        case quietControl
        case knownSpeaker(speakerName: String)
        case echoGuard

        var startsListener: Bool {
            switch self {
            case .liveWake, .echoGuard:
                return true
            case .quietControl, .knownSpeaker:
                return false
            }
        }
    }

    private static let activeProcessLock = NSLock()
    private static var activeProcess: Process?

    static var repoRootURL: URL {
        let envRoot = ProcessInfo.processInfo.environment["SELENE_REPO_ROOT"]?
            .trimmingCharacters(in: .whitespacesAndNewlines)
        let root = envRoot?.isEmpty == false ? envRoot! : "/Users/selene/Documents/Selene-OS"
        return URL(fileURLWithPath: root, isDirectory: true)
    }

    static func microphonePermissionLabel() -> String {
        switch AVCaptureDevice.authorizationStatus(for: .audio) {
        case .authorized:
            return "granted"
        case .denied:
            return "denied"
        case .restricted:
            return "restricted"
        case .notDetermined:
            return "not requested"
        @unknown default:
            return "unavailable"
        }
    }

    static func microphonePermissionAllowsStart() -> Bool {
        switch AVCaptureDevice.authorizationStatus(for: .audio) {
        case .authorized:
            return true
        case .notDetermined:
            return false
        case .denied, .restricted:
            return false
        @unknown default:
            return false
        }
    }

    static func terminateActiveProcess() {
        activeProcessLock.lock()
        let process = activeProcess
        activeProcess = nil
        activeProcessLock.unlock()

        if process?.isRunning == true {
            process?.terminate()
        }
    }

    static func run(
        mode: Mode,
        wakeText: String,
        listenSeconds: Int
    ) async throws -> DesktopControlledWakeModeResult {
        let boundedWakeText = try boundedArgument(wakeText, fallback: "Selene", maxCount: 64)
        let boundedListenSeconds = min(max(listenSeconds, 3), 30)
        switch mode {
        case .liveWake:
            let output = try await runWakeLife(
                quietControl: false,
                wakeText: boundedWakeText,
                seconds: boundedListenSeconds
            )
            return try DesktopControlledWakeModeResult.fromWakeLife(
                output,
                expectedState: "Session opened",
                privacyLabel: "wake listener off after bounded window"
            )
        case .quietControl:
            let output = try await runWakeLife(
                quietControl: true,
                wakeText: boundedWakeText,
                seconds: boundedListenSeconds
            )
            return try DesktopControlledWakeModeResult.fromWakeLife(
                output,
                expectedState: "Stopped",
                privacyLabel: "wake listener off after quiet control"
            )
        case .knownSpeaker(let speakerName):
            let boundedSpeakerName = try boundedArgument(speakerName, fallback: "JD", maxCount: 32)
            let output = try await runVoiceE2E(
                speakerName: boundedSpeakerName,
                wakeText: boundedWakeText
            )
            return try DesktopControlledWakeModeResult.fromVoiceE2E(output)
        case .echoGuard:
            let output = try await runWakeLife(
                quietControl: true,
                wakeText: boundedWakeText,
                seconds: boundedListenSeconds
            )
            var result = try DesktopControlledWakeModeResult.fromWakeLife(
                output,
                expectedState: "Stopped",
                privacyLabel: "echo/self-trigger guard active; app-originated wake did not open a session"
            )
            result.gates.append(.init(name: "echo_self_trigger_closed", status: result.sessionOpened ? "FAIL" : "PASS"))
            result.stateLabel = result.sessionOpened ? "Failed closed" : "Stopped"
            return result
        }
    }

    private static func runWakeLife(
        quietControl: Bool,
        wakeText: String,
        seconds: Int
    ) async throws -> String {
        var arguments = [
            "cargo",
            "run",
            "-p",
            "selene_adapter",
            "--bin",
            "desktop_wake_life",
            "--",
            "--seconds",
            String(seconds),
        ]
        if quietControl {
            arguments.append("--quiet-control")
        } else {
            arguments.append(contentsOf: ["--controlled-wake-text", wakeText])
        }
        return try await runBoundedProcess(arguments: arguments, timeout: TimeInterval(seconds + 45))
    }

    private static func runVoiceE2E(
        speakerName: String,
        wakeText: String
    ) async throws -> String {
        let arguments = [
            "cargo",
            "run",
            "-p",
            "selene_adapter",
            "--bin",
            "desktop_voice_e2e",
            "--",
            "--mode",
            "enroll-and-recognize",
            "--speaker-name",
            speakerName,
            "--wake-text",
            wakeText,
            "--enroll-samples",
            "3",
            "--seconds-per-sample",
            "4",
            "--wake-seconds",
            "5",
            "--desktop-integration-proof",
            "--json",
        ]
        return try await runBoundedProcess(arguments: arguments, timeout: 75)
    }

    private static func runBoundedProcess(
        arguments: [String],
        timeout: TimeInterval
    ) async throws -> String {
        try await withCheckedThrowingContinuation { continuation in
            let process = Process()
            process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
            process.arguments = arguments
            process.currentDirectoryURL = repoRootURL
            process.environment = boundedEnvironment(ProcessInfo.processInfo.environment)

            let stdoutPipe = Pipe()
            let stderrPipe = Pipe()
            process.standardOutput = stdoutPipe
            process.standardError = stderrPipe

            let lock = NSLock()
            var didResume = false

            @Sendable func finish(_ result: Result<String, Error>) {
                lock.lock()
                guard !didResume else {
                    lock.unlock()
                    return
                }
                didResume = true
                activeProcessLock.lock()
                if activeProcess === process {
                    activeProcess = nil
                }
                activeProcessLock.unlock()
                lock.unlock()
                continuation.resume(with: result)
            }

            process.terminationHandler = { terminatedProcess in
                let stdout = stdoutPipe.fileHandleForReading.readDataToEndOfFile()
                let stderr = stderrPipe.fileHandleForReading.readDataToEndOfFile()
                let stdoutText = String(data: stdout, encoding: .utf8) ?? ""
                let stderrText = String(data: stderr, encoding: .utf8) ?? ""
                if terminatedProcess.terminationStatus == 0 {
                    finish(.success(stdoutText + "\n" + stderrText))
                } else {
                    let preview = (stdoutText + "\n" + stderrText)
                        .trimmingCharacters(in: .whitespacesAndNewlines)
                        .prefix(1_200)
                    finish(.failure(DesktopControlledWakeModeError.processFailed(String(preview))))
                }
            }

            do {
                try process.run()
                activeProcessLock.lock()
                activeProcess = process
                activeProcessLock.unlock()
            } catch {
                finish(.failure(error))
                return
            }

            DispatchQueue.global(qos: .utility).asyncAfter(deadline: .now() + timeout) {
                if process.isRunning {
                    process.terminate()
                    finish(.failure(DesktopControlledWakeModeError.timedOut))
                }
            }
        }
    }

    private static func boundedArgument(
        _ raw: String,
        fallback: String,
        maxCount: Int
    ) throws -> String {
        let trimmed = raw.trimmingCharacters(in: .whitespacesAndNewlines)
        let value = trimmed.isEmpty ? fallback : trimmed
        guard value.count <= maxCount,
              !value.contains("\n"),
              !value.contains("\r") else {
            throw DesktopControlledWakeModeError.invalidArgument
        }
        return value
    }

    private static func boundedEnvironment(_ environment: [String: String]) -> [String: String] {
        let blockedFragments = [
            "API_KEY",
            "BRAVE",
            "OPENAI",
            "TAVILY",
            "SERPAPI",
            "EXA",
            "BING",
            "WHISPER",
            "DEEPGRAM",
            "ASSEMBLYAI",
            "GOOGLE",
            "GEMINI",
            "PROVIDER",
        ]
        return environment.filter { key, _ in
            !blockedFragments.contains { key.uppercased().contains($0) }
        }
    }
}

struct DesktopControlledWakeModeResult: Equatable {
    struct Gate: Identifiable, Equatable {
        let name: String
        let status: String

        var id: String { name }
        var passed: Bool { status == "PASS" || status == "OPEN" }
    }

    var status: String
    var stateLabel: String
    var privacyLabel: String
    var wakeAccepted: Bool
    var sessionOpened: Bool
    var recognized: Bool
    var voicePosture: String
    var namedGreeting: Bool
    var greetingText: String
    var sourceChips: Int
    var providerCalls: Int
    var protectedExecution: Bool
    var memoryWrite: Bool
    var authorityGrant: Bool
    var cpuLabel: String
    var rssLabel: String
    var timeoutLabel: String
    var gates: [Gate]

    func boolean(_ value: Bool) -> String {
        value ? "true" : "false"
    }

    static func fromWakeLife(
        _ output: String,
        expectedState: String,
        privacyLabel: String
    ) throws -> DesktopControlledWakeModeResult {
        let keyValues = parseKeyValues(output)
        let wakeAccepted = keyValues["metric.wake_accepted"] == "true"
        let sessionOpened = keyValues["metric.session_opened"] == "true"
        let sourceChips = Int(keyValues["metric.runtime_source_chip_count"] ?? "0") ?? 0
        let gateSummary = keyValues["gate.summary"] ?? "FAIL"
        let failed = gateSummary == "FAIL"
        let gates = parseGates(output)
        return DesktopControlledWakeModeResult(
            status: failed ? "FAIL" : "PASS",
            stateLabel: sessionOpened ? expectedState : (wakeAccepted ? "Wake detected" : "Timed out"),
            privacyLabel: privacyLabel,
            wakeAccepted: wakeAccepted,
            sessionOpened: sessionOpened,
            recognized: false,
            voicePosture: sessionOpened ? "UNKNOWN_OR_UNAVAILABLE" : "NOT_ATTEMPTED",
            namedGreeting: false,
            greetingText: sessionOpened ? "Generic runtime greeting; Voice ID not authoritative in wake-only proof." : "",
            sourceChips: sourceChips,
            providerCalls: 0,
            protectedExecution: false,
            memoryWrite: false,
            authorityGrant: false,
            cpuLabel: keyValues["metric.process_cpu_percent_snapshot"] ?? "OPEN",
            rssLabel: keyValues["metric.process_rss_mb_peak"] ?? "OPEN",
            timeoutLabel: sessionOpened ? "session opened" : "no session opened",
            gates: gates
        )
    }

    static func fromVoiceE2E(_ output: String) throws -> DesktopControlledWakeModeResult {
        guard let json = jsonObjectCandidates(in: output).first,
              let data = json.data(using: .utf8),
              let summary = try? JSONDecoder().decode(VoiceSummary.self, from: data) else {
            throw DesktopControlledWakeModeError.malformedOutput
        }
        let status = summary.status
        let gates = summary.gates.map { Gate(name: $0.name, status: $0.status) }
        return DesktopControlledWakeModeResult(
            status: status,
            stateLabel: summary.wake.sessionOpened ? "Session opened" : "Failed closed",
            privacyLabel: "wake listener off after known speaker proof",
            wakeAccepted: summary.wake.accepted,
            sessionOpened: summary.wake.sessionOpened,
            recognized: summary.voiceID.recognized,
            voicePosture: summary.voiceID.posture,
            namedGreeting: summary.greeting.named,
            greetingText: summary.greeting.responseText,
            sourceChips: summary.safety.sourceChips,
            providerCalls: summary.safety.providerCalls,
            protectedExecution: summary.safety.protectedExecution,
            memoryWrite: summary.safety.memoryWrite,
            authorityGrant: summary.voiceID.authoritative
                || (summary.desktopRuntimeIntegration?.outcome.authorityGranted ?? false),
            cpuLabel: "covered by wake proof runner",
            rssLabel: "covered by wake proof runner",
            timeoutLabel: summary.wake.sessionOpened ? "session opened" : "closed",
            gates: gates
        )
    }

    private static func parseKeyValues(_ output: String) -> [String: String] {
        var values: [String: String] = [:]
        for line in output.split(separator: "\n") {
            let parts = line.split(separator: " ", maxSplits: 1)
            let first = parts.first.map(String.init) ?? ""
            guard let eqIndex = first.firstIndex(of: "=") else {
                continue
            }
            let key = String(first[..<eqIndex])
            let value = String(first[first.index(after: eqIndex)...])
            values[key] = value
        }
        return values
    }

    private static func parseGates(_ output: String) -> [Gate] {
        output
            .split(separator: "\n")
            .compactMap { line -> Gate? in
                guard line.hasPrefix("gate."),
                      let eqIndex = line.firstIndex(of: "=") else {
                    return nil
                }
                let rawName = String(line[line.index(line.startIndex, offsetBy: 5)..<eqIndex])
                let remainder = String(line[line.index(after: eqIndex)...])
                let status = remainder.split(separator: " ").first.map(String.init) ?? "FAIL"
                return Gate(name: rawName, status: status)
            }
    }

    private static func jsonObjectCandidates(in text: String) -> [String] {
        var results: [String] = []
        var depth = 0
        var start: String.Index?
        for index in text.indices {
            let character = text[index]
            if character == "{" {
                if depth == 0 {
                    start = index
                }
                depth += 1
            } else if character == "}" {
                depth -= 1
                if depth == 0, let startIndex = start {
                    results.append(String(text[startIndex...index]))
                    start = nil
                }
            }
        }
        return results.reversed()
    }

    private struct VoiceSummary: Decodable {
        let status: String
        let wake: Wake
        let voiceID: VoiceID
        let greeting: Greeting
        let safety: Safety
        let gates: [GateSummary]
        let desktopRuntimeIntegration: RuntimeIntegration?

        private enum CodingKeys: String, CodingKey {
            case status
            case wake
            case voiceID = "voice_id"
            case greeting
            case safety
            case gates
            case desktopRuntimeIntegration = "desktop_runtime_integration"
        }

        struct Wake: Decodable {
            let accepted: Bool
            let sessionOpened: Bool

            private enum CodingKeys: String, CodingKey {
                case accepted
                case sessionOpened = "session_opened"
            }
        }

        struct VoiceID: Decodable {
            let recognized: Bool
            let posture: String
            let authoritative: Bool
        }

        struct Greeting: Decodable {
            let named: Bool
            let responseText: String

            private enum CodingKeys: String, CodingKey {
                case named
                case responseText = "response_text"
            }
        }

        struct Safety: Decodable {
            let sourceChips: Int
            let providerCalls: Int
            let protectedExecution: Bool
            let memoryWrite: Bool

            private enum CodingKeys: String, CodingKey {
                case sourceChips = "source_chips"
                case providerCalls = "provider_calls"
                case protectedExecution = "protected_execution"
                case memoryWrite = "memory_write"
            }
        }

        struct GateSummary: Decodable {
            let name: String
            let status: String
        }

        struct RuntimeIntegration: Decodable {
            let outcome: Outcome
        }

        struct Outcome: Decodable {
            let authorityGranted: Bool

            private enum CodingKeys: String, CodingKey {
                case authorityGranted = "authority_granted"
            }
        }
    }
}

private enum DesktopControlledWakeModeError: LocalizedError {
    case invalidArgument
    case timedOut
    case processFailed(String)
    case malformedOutput

    var errorDescription: String? {
        switch self {
        case .invalidArgument:
            return "controlled wake argument was empty, too long, or multiline"
        case .timedOut:
            return "controlled wake process timed out and was stopped"
        case .processFailed(let detail):
            return detail
        case .malformedOutput:
            return "controlled wake runtime output was missing or malformed"
        }
    }
}
