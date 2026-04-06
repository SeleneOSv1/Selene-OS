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
    private let recentThreadPreviewEntries = [
        RecentThreadPreviewEntry(
            speaker: "You",
            posture: "explicit_recent_user_turn",
            body: "Show the latest lawful session context before any cloud-authoritative request path is opened.",
            detail: "User-side thread preview only; no typed-turn dispatch or local transcript authority is introduced here."
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
            body: "Hold this session surface until a lawful app-open or explicit voice ingress can be rendered without local production.",
            detail: "Read-only preview only; no invite activation, no onboarding mutation, and no session resurrection occur locally."
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
                boundedSurfaceCard(title: "Session", detail: "One dominant session surface remains primary. No local runtime request production occurs inside this shell.")
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

            Text("H82 preserves the H79 read-only EXPLICIT_ENTRY_READY recent thread window, typed input affordance, and explicit voice entry affordance, preserves the H80 read-only history side-drawer recall, incremental history expansion, and archived session recall, preserves the H81 read-only System Activity operational queue with separate Pending and Failed visibility, and adds a separate read-only Needs Attention actionable queue while preserving the H74, H75, H76, and H77 takeover surfaces.")
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
            VStack(alignment: .leading, spacing: 16) {
                Text("Waiting for lawful app-open / invite-open ingress.")
                    .font(.headline)

                Text("H82 keeps `EXPLICIT_ENTRY_READY` as the dominant bounded session surface. Recent thread, typed input, explicit voice, history recall, `System Activity`, and `Needs Attention` affordances remain read-only, `EXPLICIT_ONLY`, session-bound, and cloud-authoritative.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                recentThreadWindowCard
                typedInputAffordanceCard
                explicitVoiceEntryAffordanceCard
                historySideDrawerCard
                systemActivityQueueCard
                needsAttentionQueueCard

                Text("No invite activation, no onboarding mutation, no typed-turn dispatch, no explicit voice-turn dispatch, and no runtime request production occur locally.")
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
                Text("Composer-style surface only. This affordance previews where typed follow-up will live once a lawful cloud-authoritative request path exists.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .center, spacing: 12) {
                    Image(systemName: "text.cursor")
                        .foregroundStyle(.secondary)

                    TextField(
                        "Type a follow-up once cloud-authoritative ingress is available.",
                        text: .constant("")
                    )
                    .disabled(true)
                    .textFieldStyle(.roundedBorder)
                }

                HStack(spacing: 8) {
                    posturePill("Read-only composer")
                    posturePill("No typed-turn dispatch")
                    posturePill("Session-bound")
                }

                Text("No local authority, no runtime request production, and no onboarding mutation are introduced by this affordance.")
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
                Text("Explicit voice entry remains a lawful session-bound preview only. This surface does not start capture, produce a request, or activate wake behavior.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                HStack(alignment: .center, spacing: 12) {
                    Image(systemName: "mic.circle")
                        .font(.system(size: 28))
                        .foregroundStyle(.secondary)

                    VStack(alignment: .leading, spacing: 4) {
                        Text("Explicit voice entry")
                            .font(.headline)

                        Text("Read-only non-producing posture aligned to `voice_context_ios_explicit()` and cloud-authoritative session control.")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }

                    Spacer()

                    Text("Not live here")
                        .font(.caption.weight(.semibold))
                        .padding(.horizontal, 10)
                        .padding(.vertical, 6)
                        .background(Color.secondary.opacity(0.12))
                        .clipShape(Capsule())
                }

                HStack(spacing: 8) {
                    posturePill("EXPLICIT_ONLY")
                    posturePill("No voice-turn dispatch")
                    posturePill("No wake parity")
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
                Text("H81 replaces the placeholder `System Activity` posture with a bounded read-only operational queue. Persistence acknowledgement, reconciliation decision, broadcast waiting / follow-up / reminder state, sync queue posture, dead-letter posture, and recovery posture remain visible only, separate from transcript history, the recent thread window, the history side drawer, archived recall, and PH1.M memory, while H82 keeps `Needs Attention` as a separate actionable subset below.")
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
                Text("`Pending` remains a separate operational queue from history. It stays visible only, cloud-authoritative, and session-bound.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                ForEach(pendingOperationalEntries) { entry in
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
                Text("`Failed` remains a separate operational queue from history. It stays visible only, cloud-authoritative, and distinct from the current visible thread window.")
                    .frame(maxWidth: .infinity, alignment: .leading)

                ForEach(failedOperationalEntries) { entry in
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
}

#Preview {
    SessionShellView(router: ExplicitEntryRouter())
}
