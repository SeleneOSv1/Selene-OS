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

private struct DesktopSessionHeaderContext: Equatable {
    let sessionState: String
    let sessionID: String
    let sessionAttachOutcome: String

    init(sessionState: String, sessionID: String, sessionAttachOutcome: String) {
        self.sessionState = sessionState
        self.sessionID = sessionID
        self.sessionAttachOutcome = sessionAttachOutcome
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
    }
}

struct DesktopSessionShellView: View {
    @State private var latestSessionHeaderContext: DesktopSessionHeaderContext?
    @State private var latestSessionActiveVisibleContext: DesktopSessionActiveVisibleContext?

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

                sectionCard(
                    title: "Needs Attention",
                    detail: "Bounded actionable placeholder kept separate from transcript history."
                )
            }
            .frame(width: 300, alignment: .topLeading)
        }
        .padding(24)
        .frame(minWidth: 1180, minHeight: 720, alignment: .topLeading)
        .background(Color(nsColor: .windowBackgroundColor))
        .onOpenURL { url in
            if let context = DesktopSessionActiveVisibleContext(url: url) {
                latestSessionActiveVisibleContext = context

                if let sessionAttachOutcome = context.sessionAttachOutcome {
                    latestSessionHeaderContext = DesktopSessionHeaderContext(
                        sessionState: context.sessionState,
                        sessionID: context.sessionID,
                        sessionAttachOutcome: sessionAttachOutcome
                    )
                } else if latestSessionHeaderContext?.sessionID != context.sessionID {
                    latestSessionHeaderContext = nil
                }

                return
            }

            if let context = DesktopSessionHeaderContext(url: url) {
                latestSessionHeaderContext = context
                latestSessionActiveVisibleContext = nil
            }
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

    private var sessionCard: some View {
        Group {
            if let latestSessionActiveVisibleContext {
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
            if let latestSessionActiveVisibleContext {
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
            if let latestSessionActiveVisibleContext {
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
            } else {
                sectionCard(
                    title: "System Activity",
                    detail: "Read-only operational placeholder for governed sync, recovery, and alert posture."
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
