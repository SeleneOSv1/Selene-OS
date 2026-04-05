import SwiftUI

@main
struct SeleneIPhoneApp: App {
    @StateObject private var explicitEntryRouter = ExplicitEntryRouter()

    var body: some Scene {
        WindowGroup {
            SessionShellView(router: explicitEntryRouter)
                .onOpenURL { url in
                    explicitEntryRouter.receive(url: url)
                }
        }
    }
}
