import SwiftUI

@main
struct SeleneIPhoneApp: App {
    @StateObject private var explicitEntryRouter = ExplicitEntryRouter()

    var body: some Scene {
        WindowGroup {
            rootShell
        }
    }

    private var rootShell: some View {
        SessionShellView(router: explicitEntryRouter)
            // Canonical app-open / invite-open URLs are handed into the bounded shell only,
            // where H87 preserves the H79 recent-thread surface, the H83
            // typed-turn request production slice, the H84 explicit voice-turn
            // request production slice, the H80 history side-drawer /
            // incremental-expansion / archived-session recall slice, the H81
            // System Activity / Pending / Failed surfaces, the H82 Needs
            // Attention actionable queue, the H74-H77 takeover posture,
            // preserves the H85 `SESSION_OPEN_VISIBLE` current session banner
            // plus attach-outcome continuity labeling path, preserves the H86
            // `SESSION_ACTIVE_VISIBLE` live dual transcript plus current
            // governed-output summary path, and now also allows bounded
            // `SESSION_SOFT_CLOSED_VISIBLE` explicit resume affordance,
            // archived recent slice, and bounded PH1.M `resume context`,
            // plus bounded `SESSION_SUSPENDED_VISIBLE` hard full takeover,
            // suspended-status explanation, and allowed next step only,
            // while H89 now also preserves bounded `RECOVERING` /
            // `DEGRADED_RECOVERY` inline restriction posture and
            // `QUARANTINED_LOCAL_STATE` hard takeover posture, and H90 now
            // also preserves bounded `INTERRUPT_VISIBLE` interruption
            // continuity posture with clarify / continue previous topic /
            // switch topic / resume later visibility only through the same
            // cloud-authored session route, all while remaining
            // cloud-authoritative and session-bound.
            .onOpenURL { url in
                explicitEntryRouter.receive(url: url)
            }
    }
}
