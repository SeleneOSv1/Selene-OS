import SwiftUI

@main
struct SeleneMacDesktopApp: App {
    var body: some Scene {
        WindowGroup {
            DesktopSessionShellView()
        }
        .defaultSize(width: 1220, height: 760)
    }
}
