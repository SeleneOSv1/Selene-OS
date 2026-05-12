import AppKit
import SwiftUI

final class SeleneMacDesktopAppDelegate: NSObject, NSApplicationDelegate {
    func applicationWillFinishLaunching(_ notification: Notification) {
        guard let bundleIdentifier = Bundle.main.bundleIdentifier else {
            return
        }

        let currentProcessID = ProcessInfo.processInfo.processIdentifier
        let existingApplication = NSRunningApplication
            .runningApplications(withBundleIdentifier: bundleIdentifier)
            .first { application in
                application.processIdentifier != currentProcessID
            }

        guard let existingApplication else {
            return
        }

        existingApplication.activate(options: [.activateAllWindows])
        NSApplication.shared.terminate(nil)
    }

    func applicationShouldHandleReopen(
        _ sender: NSApplication,
        hasVisibleWindows flag: Bool
    ) -> Bool {
        if !flag {
            sender.windows.first?.makeKeyAndOrderFront(nil)
        }
        return true
    }
}

@main
struct SeleneMacDesktopApp: App {
    @NSApplicationDelegateAdaptor(SeleneMacDesktopAppDelegate.self)
    private var appDelegate

    var body: some Scene {
        Window("Selene", id: "selene-main") {
            DesktopSessionShellView()
        }
        .defaultSize(width: 1220, height: 760)
        .commands {
            CommandGroup(replacing: .newItem) {}
        }
    }
}
