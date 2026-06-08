import AppKit
import SwiftUI

extension Notification.Name {
    static let seleneDesktopScreenLifecycleAction = Notification.Name("seleneDesktopScreenLifecycleAction")
}

final class SeleneMacDesktopAppDelegate: NSObject, NSApplicationDelegate {
    private var mainWindow: NSWindow?
    private var mainWindowDelegate: SeleneMacDesktopMainWindowDelegate?

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
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(handleApprovedScreenLifecycleAction(_:)),
            name: .seleneDesktopScreenLifecycleAction,
            object: nil
        )
        showMainWindow()
        DispatchQueue.main.async { [weak self] in
            self?.showMainWindowIfNeededAfterLaunch()
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) { [weak self] in
            self?.showMainWindowIfNeededAfterLaunch()
        }
    }

    func applicationShouldHandleReopen(
        _ sender: NSApplication,
        hasVisibleWindows flag: Bool
    ) -> Bool {
        toggleMainWindowVisibilityFromAppIcon()
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
            // Prefer the newly launched binary so live acceptance cannot keep a stale rebuilt app alive.
            sameExecutableApplication.terminate()
            existingApplications
                .filter { canonicalPath($0.executableURL) != currentExecutablePath }
                .forEach { $0.terminate() }
            return false
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

    deinit {
        NotificationCenter.default.removeObserver(self)
    }

    private func showMainWindow() {
        let window: NSWindow
        if let existingWindow = mainWindow {
            window = existingWindow
        } else {
            let hostingController = NSHostingController(rootView: DesktopSessionShellView())
            let mainWindowDelegate = SeleneMacDesktopMainWindowDelegate()
            let createdWindow = NSWindow(
                contentRect: NSRect(x: 0, y: 0, width: 1220, height: 760),
                styleMask: [.titled, .closable, .miniaturizable, .resizable],
                backing: .buffered,
                defer: false
            )
            createdWindow.title = "Selene"
            createdWindow.contentViewController = hostingController
            createdWindow.delegate = mainWindowDelegate
            createdWindow.collectionBehavior.insert(.fullScreenPrimary)
            createdWindow.minSize = NSSize(width: 640, height: 520)
            createdWindow.setFrameAutosaveName("SeleneMainWindow")
            createdWindow.center()
            self.mainWindow = createdWindow
            self.mainWindowDelegate = mainWindowDelegate
            window = createdWindow
        }

        if window.isMiniaturized {
            window.deminiaturize(nil)
        }
        NSApplication.shared.unhide(nil)
        NSApplication.shared.activate(ignoringOtherApps: true)
        window.makeKeyAndOrderFront(nil)
    }

    private func showMainWindowIfNeededAfterLaunch() {
        guard let window = mainWindow else {
            showMainWindow()
            return
        }

        if !window.isVisible || window.isMiniaturized {
            showMainWindow()
        }
    }

    private func hideMainWindow() {
        guard let window = mainWindow else {
            return
        }

        if !window.isMiniaturized {
            window.miniaturize(nil)
        }
    }

    private func toggleMainWindowVisibilityFromAppIcon() {
        guard let window = mainWindow else {
            showMainWindow()
            return
        }

        if window.isVisible && !window.isMiniaturized {
            hideMainWindow()
        } else {
            showMainWindow()
        }
    }

    @objc private func handleApprovedScreenLifecycleAction(_ notification: Notification) {
        guard let action = notification.userInfo?["action"] as? String else {
            return
        }

        switch action {
        case "show":
            showMainWindow()
        case "hide":
            hideMainWindow()
        default:
            break
        }
    }
}

private final class SeleneMacDesktopMainWindowDelegate: NSObject, NSWindowDelegate {
    func windowShouldClose(_ sender: NSWindow) -> Bool {
        sender.miniaturize(nil)
        return false
    }
}

@main
struct SeleneMacDesktopApp: App {
    @NSApplicationDelegateAdaptor(SeleneMacDesktopAppDelegate.self)
    private var appDelegate: SeleneMacDesktopAppDelegate

    var body: some Scene {
        Settings {
            EmptyView()
        }
        .commands {
            CommandGroup(replacing: .newItem) {}
        }
    }
}
