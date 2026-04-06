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
            // where H82 preserves the H79 recent-thread / typed-input / explicit-voice
            // surfaces, preserves the H80 history side-drawer / incremental-expansion /
            // archived-session recall slice, preserves the H81 read-only System Activity /
            // Pending / Failed surfaces, and adds a separate read-only Needs Attention
            // actionable queue while preserving the H74-H77 takeover posture.
            .onOpenURL { url in
                explicitEntryRouter.receive(url: url)
            }
    }
}
