import Foundation
import SwiftUI

struct DesktopLiveVoiceE2EProofView: View {
    @StateObject private var controller = DesktopLiveVoiceE2EProofController()
    @State private var speakerName = "JD"
    @State private var wakeText = "Selene"
    @State private var enrollSamples = 3
    @State private var secondsPerSample = 4
    @State private var wakeSeconds = 5

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            VStack(alignment: .leading, spacing: 6) {
                Text("Live Voice E2E Proof")
                    .font(.headline)

                Text("Runtime authoritative. Desktop capture/render only. Voice ID posture only. Protected execution closed.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }

            VStack(alignment: .leading, spacing: 10) {
                HStack(spacing: 10) {
                    TextField("Speaker name", text: $speakerName)
                        .textFieldStyle(.roundedBorder)

                    TextField("Wake text", text: $wakeText)
                        .textFieldStyle(.roundedBorder)
                }

                HStack(spacing: 14) {
                    Stepper("Enroll samples \(enrollSamples)", value: $enrollSamples, in: 3...8)
                    Stepper("Sample seconds \(secondsPerSample)", value: $secondsPerSample, in: 1...15)
                    Stepper("Wake seconds \(wakeSeconds)", value: $wakeSeconds, in: 1...30)
                }
                .font(.caption)
                .foregroundStyle(.secondary)
            }

            HStack(spacing: 10) {
                Button {
                    controller.runLiveProof(
                        speakerName: speakerName,
                        wakeText: wakeText,
                        enrollSamples: enrollSamples,
                        secondsPerSample: secondsPerSample,
                        wakeSeconds: wakeSeconds
                    )
                } label: {
                    Label("Run live proof", systemImage: "waveform")
                }
                .disabled(controller.isRunning)

                Button {
                    controller.runQuietControl(
                        speakerName: speakerName,
                        wakeText: wakeText,
                        wakeSeconds: wakeSeconds
                    )
                } label: {
                    Label("Run quiet control", systemImage: "speaker.slash")
                }
                .disabled(controller.isRunning)

                if controller.isRunning {
                    ProgressView()
                        .controlSize(.small)
                }
            }

            proofStatusBody
        }
        .padding(14)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(Color(nsColor: .controlBackgroundColor).opacity(0.55))
        .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .stroke(Color.primary.opacity(0.08), lineWidth: 1)
        )
    }

    @ViewBuilder
    private var proofStatusBody: some View {
        switch controller.state {
        case .idle:
            proofInfoRows([
                ("status", "ready"),
                ("posture", "bounded proof, not production listener"),
                ("runner", controller.repoRootPath),
            ])
        case .running(let label):
            proofInfoRows([
                ("status", label),
                ("posture", "foreground proof process"),
            ])
        case .passed(let result):
            proofResultView(result)
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
        }
    }

    private func proofResultView(_ result: DesktopLiveVoiceE2EProofDisplayResult) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            proofInfoRows([
                ("status", result.status),
                ("wake accepted", result.boolean(result.wakeAccepted)),
                ("session opened", result.boolean(result.sessionOpened)),
                ("voice enrollment", result.boolean(result.voiceEnrollmentCompleted)),
                ("voice profile", result.voiceProfileID ?? "none"),
                ("recognized", result.boolean(result.recognized)),
                ("posture", result.voicePosture),
                ("named greeting", result.boolean(result.namedGreeting)),
                ("bridge compatible", result.boolean(result.bridgeCompatible)),
                ("source chips", String(result.sourceChips)),
                ("provider paths", String(result.providerCalls)),
                ("protected execution", result.boolean(result.protectedExecution)),
                ("memory_write", result.boolean(result.memoryWrite)),
                ("authority", result.boolean(result.authorityGrant)),
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

    private func proofInfoRows(_ rows: [(String, String)]) -> some View {
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
final class DesktopLiveVoiceE2EProofController: ObservableObject {
    enum State {
        case idle
        case running(String)
        case passed(DesktopLiveVoiceE2EProofDisplayResult)
        case failed(String, String)
    }

    @Published private(set) var state: State = .idle
    @Published private(set) var isRunning = false

    var repoRootPath: String {
        DesktopLiveVoiceE2EProofRunner.repoRootURL.path
    }

    func runLiveProof(
        speakerName: String,
        wakeText: String,
        enrollSamples: Int,
        secondsPerSample: Int,
        wakeSeconds: Int
    ) {
        run(
            mode: .enrollAndRecognize,
            speakerName: speakerName,
            wakeText: wakeText,
            enrollSamples: enrollSamples,
            secondsPerSample: secondsPerSample,
            wakeSeconds: wakeSeconds
        )
    }

    func runQuietControl(
        speakerName: String,
        wakeText: String,
        wakeSeconds: Int
    ) {
        run(
            mode: .quietControl,
            speakerName: speakerName,
            wakeText: wakeText,
            enrollSamples: 3,
            secondsPerSample: 1,
            wakeSeconds: wakeSeconds
        )
    }

    private func run(
        mode: DesktopLiveVoiceE2EProofRunner.Mode,
        speakerName: String,
        wakeText: String,
        enrollSamples: Int,
        secondsPerSample: Int,
        wakeSeconds: Int
    ) {
        guard !isRunning else {
            return
        }

        isRunning = true
        state = .running(mode == .quietControl ? "running quiet control" : "running live proof")

        Task {
            do {
                let result = try await DesktopLiveVoiceE2EProofRunner.run(
                    mode: mode,
                    speakerName: speakerName,
                    wakeText: wakeText,
                    enrollSamples: enrollSamples,
                    secondsPerSample: secondsPerSample,
                    wakeSeconds: wakeSeconds
                )
                state = .passed(result)
            } catch {
                state = .failed("Live Voice E2E Proof failed closed", error.localizedDescription)
            }
            isRunning = false
        }
    }
}

enum DesktopLiveVoiceE2EProofRunner {
    enum Mode: String {
        case enrollAndRecognize = "enroll-and-recognize"
        case quietControl = "quiet-control"
    }

    static var repoRootURL: URL {
        let envRoot = ProcessInfo.processInfo.environment["SELENE_REPO_ROOT"]?
            .trimmingCharacters(in: .whitespacesAndNewlines)
        let root = envRoot?.isEmpty == false ? envRoot! : "/Users/selene/Documents/Selene-OS"
        return URL(fileURLWithPath: root, isDirectory: true)
    }

    static func run(
        mode: Mode,
        speakerName: String,
        wakeText: String,
        enrollSamples: Int,
        secondsPerSample: Int,
        wakeSeconds: Int
    ) async throws -> DesktopLiveVoiceE2EProofDisplayResult {
        let boundedSpeakerName = try boundedArgument(speakerName, fallback: "JD", maxCount: 32)
        let boundedWakeText = try boundedArgument(wakeText, fallback: "Selene", maxCount: 64)
        let boundedEnrollSamples = min(max(enrollSamples, 3), 8)
        let boundedSecondsPerSample = min(max(secondsPerSample, 1), 15)
        let boundedWakeSeconds = min(max(wakeSeconds, 1), 30)

        var arguments = [
            "cargo",
            "run",
            "-p",
            "selene_adapter",
            "--bin",
            "desktop_voice_e2e",
            "--",
            "--mode",
            mode.rawValue,
            "--speaker-name",
            boundedSpeakerName,
            "--wake-text",
            boundedWakeText,
            "--wake-seconds",
            String(boundedWakeSeconds),
            "--desktop-integration-proof",
            "--json",
        ]

        if mode == .enrollAndRecognize {
            arguments.append(contentsOf: [
                "--enroll-samples",
                String(boundedEnrollSamples),
                "--seconds-per-sample",
                String(boundedSecondsPerSample),
            ])
        }

        let timeout = TimeInterval(
            max(
                20,
                boundedWakeSeconds
                    + (mode == .enrollAndRecognize
                        ? boundedEnrollSamples * boundedSecondsPerSample + 45
                        : 12)
            )
        )
        let processResult = try await runBoundedProcess(arguments: arguments, timeout: timeout)
        guard processResult.exitCode == 0 else {
            throw DesktopLiveVoiceE2EProofError.processFailed(processResult.stderrPreview)
        }

        let summary = try DesktopLiveVoiceE2EProofSummary.decode(from: processResult.stdout)
        return try summary.validatedDisplayResult()
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
            throw DesktopLiveVoiceE2EProofError.invalidArgument("proof argument is empty or too long")
        }
        return value
    }

    private static func runBoundedProcess(
        arguments: [String],
        timeout: TimeInterval
    ) async throws -> DesktopLiveVoiceE2EProofProcessResult {
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

            func finish(_ result: Result<DesktopLiveVoiceE2EProofProcessResult, Error>) {
                lock.lock()
                guard !didResume else {
                    lock.unlock()
                    return
                }
                didResume = true
                lock.unlock()
                continuation.resume(with: result)
            }

            process.terminationHandler = { terminatedProcess in
                let stdout = stdoutPipe.fileHandleForReading.readDataToEndOfFile()
                let stderr = stderrPipe.fileHandleForReading.readDataToEndOfFile()
                finish(.success(
                    DesktopLiveVoiceE2EProofProcessResult(
                        stdout: stdout,
                        stderr: stderr,
                        exitCode: terminatedProcess.terminationStatus
                    )
                ))
            }

            do {
                try process.run()
            } catch {
                finish(.failure(error))
                return
            }

            DispatchQueue.global(qos: .utility).asyncAfter(deadline: .now() + timeout) {
                if process.isRunning {
                    process.terminate()
                    finish(.failure(DesktopLiveVoiceE2EProofError.timedOut))
                }
            }
        }
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

struct DesktopLiveVoiceE2EProofDisplayResult: Equatable {
    struct Gate: Identifiable, Equatable {
        let name: String
        let status: String

        var id: String { name }
        var passed: Bool { status == "PASS" }
    }

    let status: String
    let wakeAccepted: Bool
    let sessionOpened: Bool
    let voiceEnrollmentCompleted: Bool
    let voiceProfileID: String?
    let recognized: Bool
    let voicePosture: String
    let namedGreeting: Bool
    let greetingText: String
    let sourceChips: Int
    let providerCalls: Int
    let protectedExecution: Bool
    let memoryWrite: Bool
    let authorityGrant: Bool
    let bridgeCompatible: Bool
    let gates: [Gate]

    func boolean(_ value: Bool) -> String {
        value ? "true" : "false"
    }
}

private struct DesktopLiveVoiceE2EProofProcessResult {
    let stdout: Data
    let stderr: Data
    let exitCode: Int32

    var stderrPreview: String {
        let text = String(data: stderr, encoding: .utf8) ?? "unreadable stderr"
        let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            return "process exited with code \(exitCode)"
        }
        return String(trimmed.prefix(800))
    }
}

private enum DesktopLiveVoiceE2EProofError: LocalizedError {
    case invalidArgument(String)
    case timedOut
    case processFailed(String)
    case jsonMissing
    case unsafeSummary(String)

    var errorDescription: String? {
        switch self {
        case .invalidArgument(let detail):
            return detail
        case .timedOut:
            return "bounded desktop voice proof process timed out"
        case .processFailed(let detail):
            return detail
        case .jsonMissing:
            return "runtime proof JSON was missing or malformed"
        case .unsafeSummary(let detail):
            return detail
        }
    }
}

private struct DesktopLiveVoiceE2EProofSummary: Decodable {
    let status: String
    let speakerName: String
    let wake: Wake
    let voiceID: VoiceID
    let greeting: Greeting
    let safety: Safety
    let gates: [Gate]
    let desktopRuntimeIntegration: RuntimeIntegration

    private enum CodingKeys: String, CodingKey {
        case status
        case speakerName = "speaker_name"
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
        let nextMove: String

        private enum CodingKeys: String, CodingKey {
            case accepted
            case sessionOpened = "session_opened"
            case nextMove = "next_move"
        }
    }

    struct VoiceID: Decodable {
        let enrollmentCompleted: Bool
        let voiceProfileID: String?
        let recognized: Bool
        let posture: String
        let authoritative: Bool

        private enum CodingKeys: String, CodingKey {
            case enrollmentCompleted = "enrollment_completed"
            case voiceProfileID = "voice_profile_id"
            case recognized
            case posture
            case authoritative
        }
    }

    struct Greeting: Decodable {
        let named: Bool
        let responseText: String
        let ttsText: String

        private enum CodingKeys: String, CodingKey {
            case named
            case responseText = "response_text"
            case ttsText = "tts_text"
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

    struct Gate: Decodable {
        let name: String
        let status: String
    }

    struct RuntimeIntegration: Decodable {
        let bridgeCompatible: Bool
        let nativeClientUntouched: Bool
        let outcome: Outcome
        let gateSummary: GateSummary

        private enum CodingKeys: String, CodingKey {
            case bridgeCompatible = "bridge_compatible"
            case nativeClientUntouched = "native_client_untouched"
            case outcome
            case gateSummary = "gate_summary"
        }
    }

    struct Outcome: Decodable {
        let authorityGranted: Bool

        private enum CodingKeys: String, CodingKey {
            case authorityGranted = "authority_granted"
        }
    }

    struct GateSummary: Decodable {
        let status: String
        let pass: Int
        let fail: Int
    }

    static func decode(from data: Data) throws -> DesktopLiveVoiceE2EProofSummary {
        guard let text = String(data: data, encoding: .utf8) else {
            throw DesktopLiveVoiceE2EProofError.jsonMissing
        }
        let decoder = JSONDecoder()
        for jsonObject in jsonObjectCandidates(in: text) {
            if let objectData = jsonObject.data(using: .utf8),
               let summary = try? decoder.decode(DesktopLiveVoiceE2EProofSummary.self, from: objectData) {
                return summary
            }
        }
        throw DesktopLiveVoiceE2EProofError.jsonMissing
    }

    func validatedDisplayResult() throws -> DesktopLiveVoiceE2EProofDisplayResult {
        guard status == "PASS" else {
            throw DesktopLiveVoiceE2EProofError.unsafeSummary("runtime proof did not pass")
        }
        guard desktopRuntimeIntegration.bridgeCompatible else {
            throw DesktopLiveVoiceE2EProofError.unsafeSummary("bridge compatibility was false")
        }
        guard desktopRuntimeIntegration.nativeClientUntouched else {
            throw DesktopLiveVoiceE2EProofError.unsafeSummary("CLI proof depended on native authority")
        }
        guard !voiceID.authoritative,
              !desktopRuntimeIntegration.outcome.authorityGranted else {
            throw DesktopLiveVoiceE2EProofError.unsafeSummary("voice proof attempted authority")
        }
        guard safety.providerCalls == 0,
              safety.sourceChips == 0,
              !safety.protectedExecution,
              !safety.memoryWrite else {
            throw DesktopLiveVoiceE2EProofError.unsafeSummary("runtime safety closure failed")
        }
        guard !greeting.named || (voiceID.recognized && voiceID.posture == "KNOWN_HIGH_CONFIDENCE") else {
            throw DesktopLiveVoiceE2EProofError.unsafeSummary("named greeting lacked known high-confidence posture")
        }
        guard gates.allSatisfy({ $0.status == "PASS" }),
              desktopRuntimeIntegration.gateSummary.status == "PASS",
              desktopRuntimeIntegration.gateSummary.fail == 0 else {
            throw DesktopLiveVoiceE2EProofError.unsafeSummary("one or more proof gates failed")
        }

        return DesktopLiveVoiceE2EProofDisplayResult(
            status: status,
            wakeAccepted: wake.accepted,
            sessionOpened: wake.sessionOpened,
            voiceEnrollmentCompleted: voiceID.enrollmentCompleted,
            voiceProfileID: voiceID.voiceProfileID,
            recognized: voiceID.recognized,
            voicePosture: voiceID.posture,
            namedGreeting: greeting.named,
            greetingText: greeting.responseText,
            sourceChips: safety.sourceChips,
            providerCalls: safety.providerCalls,
            protectedExecution: safety.protectedExecution,
            memoryWrite: safety.memoryWrite,
            authorityGrant: voiceID.authoritative || desktopRuntimeIntegration.outcome.authorityGranted,
            bridgeCompatible: desktopRuntimeIntegration.bridgeCompatible,
            gates: gates.map {
                DesktopLiveVoiceE2EProofDisplayResult.Gate(name: $0.name, status: $0.status)
            }
        )
    }

    private static func jsonObjectCandidates(in text: String) -> [String] {
        let characters = Array(text)
        var candidates: [String] = []
        for start in characters.indices where characters[start] == "{" {
            var depth = 0
            var inString = false
            var escaped = false
            for index in start..<characters.count {
                let character = characters[index]
                if inString {
                    if escaped {
                        escaped = false
                    } else if character == "\\" {
                        escaped = true
                    } else if character == "\"" {
                        inString = false
                    }
                    continue
                }
                if character == "\"" {
                    inString = true
                } else if character == "{" {
                    depth += 1
                } else if character == "}" {
                    depth -= 1
                    if depth == 0 {
                        candidates.append(String(characters[start...index]))
                        break
                    }
                }
            }
        }
        return candidates
    }
}

#Preview {
    DesktopLiveVoiceE2EProofView()
        .padding()
        .frame(width: 620)
}
