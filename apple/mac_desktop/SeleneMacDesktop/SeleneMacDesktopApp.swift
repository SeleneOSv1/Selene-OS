import AppKit
import SwiftUI

final class SeleneMacDesktopAppDelegate: NSObject, NSApplicationDelegate, NSWindowDelegate {
    private var mainWindow: NSWindow?

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
        DispatchQueue.main.async { [weak self] in
            self?.showMainWindow()
        }
    }

    func applicationShouldHandleReopen(
        _ sender: NSApplication,
        hasVisibleWindows flag: Bool
    ) -> Bool {
        showMainWindow()
        return true
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        true
    }

    func windowWillClose(_ notification: Notification) {
        guard let window = notification.object as? NSWindow,
              window === mainWindow else {
            return
        }
        mainWindow = nil
    }

    private func showMainWindow() {
        if let mainWindow {
            mainWindow.makeKeyAndOrderFront(nil)
            NSApplication.shared.activate(ignoringOtherApps: true)
            return
        }

        if let existingWindow = NSApplication.shared.windows.first(where: { window in
            window.identifier?.rawValue == "selene-main" || window.title == "Selene"
        }) {
            existingWindow.makeKeyAndOrderFront(nil)
            mainWindow = existingWindow
            NSApplication.shared.activate(ignoringOtherApps: true)
            return
        }

        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 1220, height: 760),
            styleMask: [.titled, .closable, .miniaturizable, .resizable],
            backing: .buffered,
            defer: false
        )
        window.title = "Selene"
        window.identifier = NSUserInterfaceItemIdentifier("selene-main")
        window.contentView = NSHostingView(rootView: DesktopSessionShellView())
        window.delegate = self
        window.center()
        window.makeKeyAndOrderFront(nil)
        mainWindow = window
        NSApplication.shared.activate(ignoringOtherApps: true)
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
    private var appDelegate

    var body: some Scene {
        WindowGroup("Selene", id: "selene-main") {
            DesktopSessionShellView()
        }
        .defaultSize(width: 1220, height: 760)
        .commands {
            CommandGroup(replacing: .newItem) {}
        }
    }
}
