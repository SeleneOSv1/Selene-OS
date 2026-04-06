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
            // where H80 preserves the H79 recent-thread / typed-input / explicit-voice
            // surfaces and adds read-only history side-drawer, incremental expansion,
            // and archived-session recall while preserving the H74-H77 takeover posture.
            .onOpenURL { url in
                explicitEntryRouter.receive(url: url)
            }
    }
}
