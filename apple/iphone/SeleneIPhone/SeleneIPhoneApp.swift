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
            // where H79 adds read-only EXPLICIT_ENTRY_READY recent-thread, typed-input,
            // and explicit-voice affordances while preserving the H74-H77 takeover posture.
            .onOpenURL { url in
                explicitEntryRouter.receive(url: url)
            }
    }
}
