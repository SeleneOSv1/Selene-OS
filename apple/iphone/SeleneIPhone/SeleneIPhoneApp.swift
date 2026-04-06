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
            // where H83 preserves the H79 recent-thread and explicit-voice surfaces,
            // advances the typed-input affordance into bounded typed-turn request
            // production, preserves the H80 history side-drawer / incremental-expansion /
            // archived-session recall slice, preserves the H81 System Activity / Pending /
            // Failed surfaces, preserves the H82 Needs Attention actionable queue, and
            // keeps the H74-H77 takeover posture cloud-authoritative and session-bound.
            .onOpenURL { url in
                explicitEntryRouter.receive(url: url)
            }
    }
}
