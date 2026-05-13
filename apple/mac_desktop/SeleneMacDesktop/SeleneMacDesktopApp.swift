import AppKit
import SwiftUI

final class SeleneMacDesktopAppDelegate: NSObject, NSApplicationDelegate {
    override init() {
        super.init()
        NSLog("SeleneMacDesktop app delegate initialized")
    }

    func applicationWillFinishLaunching(_ notification: Notification) {
        guard let bundleIdentifier = Bundle.main.bundleIdentifier else {
            return
        }

        let currentProcessID = ProcessInfo.processInfo.processIdentifier
        let existingApplications = NSRunningApplication
            .runningApplications(withBundleIdentifier: bundleIdentifier)
            .filter { application in
                application.processIdentifier != currentProcessID
            }

        guard !existingApplications.isEmpty else {
            return
        }

        if Self.shouldTerminateCurrentInstance(existingApplications: existingApplications) {
            NSApplication.shared.terminate(nil)
        }
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApplication.shared.setActivationPolicy(.regular)
    }

    func applicationShouldHandleReopen(
        _ sender: NSApplication,
        hasVisibleWindows flag: Bool
    ) -> Bool {
        sender.activate(ignoringOtherApps: true)
        return false
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        false
    }

    private static func shouldTerminateCurrentInstance(
        existingApplications: [NSRunningApplication]
    ) -> Bool {
        let currentExecutablePath = canonicalPath(Bundle.main.executableURL)

        if let sameExecutableApplication = existingApplications.first(where: { application in
            canonicalPath(application.executableURL) == currentExecutablePath
        }) {
            existingApplications
                .filter { canonicalPath($0.executableURL) != currentExecutablePath }
                .forEach { $0.terminate() }
            sameExecutableApplication.activate(options: [.activateAllWindows])
            return true
        }

        let currentExecutableModifiedAt = executableModifiedAt(Bundle.main.executableURL)
        let newestExistingApplication = existingApplications.max { lhs, rhs in
            executableModifiedAt(lhs.executableURL) < executableModifiedAt(rhs.executableURL)
        }

        guard let newestExistingApplication else {
            return false
        }

        if executableModifiedAt(newestExistingApplication.executableURL) > currentExecutableModifiedAt {
            newestExistingApplication.activate(options: [.activateAllWindows])
            return true
        }

        existingApplications.forEach { $0.terminate() }
        return false
    }

    private static func canonicalPath(_ url: URL?) -> String {
        url?.resolvingSymlinksInPath().standardizedFileURL.path ?? ""
    }

    private static func executableModifiedAt(_ url: URL?) -> Date {
        guard let path = url?.path,
              let attributes = try? FileManager.default.attributesOfItem(atPath: path),
              let modifiedAt = attributes[.modificationDate] as? Date else {
            return .distantPast
        }

        return modifiedAt
    }
}

@main
struct SeleneMacDesktopApp: App {
    @NSApplicationDelegateAdaptor(SeleneMacDesktopAppDelegate.self)
    private var appDelegate: SeleneMacDesktopAppDelegate

    var body: some Scene {
        Window("Selene", id: "selene-main") {
            DesktopSessionShellView()
        }
        .defaultSize(width: 1220, height: 760)
        .commands {
            CommandGroup(replacing: .newItem) {}
        }

        Settings {
            EmptyView()
        }
    }
}
