import Foundation
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
            return "A lawful app-open / invite-open route has been parsed and is being rendered as a bounded onboarding-entry takeover surface with read-only onboarding outcome, onboarding_status, prompt-state, and remaining platform-receipt context only."
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

struct SessionShellView: View {
    @ObservedObject var router: ExplicitEntryRouter

    @State private var displayState: ShellDisplayState = .explicitEntryReady
    @State private var activeContext: ExplicitEntryContext?

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
                boundedSurfaceCard(title: "Session", detail: "One dominant session surface remains primary. No local runtime request production occurs inside this shell.")
                boundedSurfaceCard(title: "History", detail: "History remains a bounded recall surface only. No local memory or transcript authority is created here.")
                boundedSurfaceCard(title: "System Activity", detail: "System Activity remains a governed visibility surface only, separate from transcript history and local authority.")
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

            Text("H76 adds read-only blocking_field, blocking_question, and remaining missing-field visibility while preserving the H74 and H75 takeover surfaces.")
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

                Text("This shell remains session-bound, read-only over parsed explicit-entry context, and cloud-authoritative for onboarding, identity, governance, and runtime law.")
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
            Text("Waiting for lawful app-open / invite-open ingress. Canonical entry URLs are parsed and displayed only; no invite activation, no onboarding mutation, and no runtime request production occur locally.")
                .frame(maxWidth: .infinity, alignment: .leading)
        } label: {
            Text("EXPLICIT_ENTRY_READY")
                .font(.headline.monospaced())
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

                Text("This H75 surface preserves current receipt/task status only, and H76 keeps that bounded while still refusing to surface voice_artifact_sync_receipt_ref or access_engine_instance_id.")
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
}

#Preview {
    SessionShellView(router: ExplicitEntryRouter())
}
