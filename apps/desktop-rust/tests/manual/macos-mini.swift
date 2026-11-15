import AppKit
import CoreFoundation
import CoreGraphics
import Darwin
import Foundation

struct SmokeError: Error, CustomStringConvertible {
    let description: String
}

struct Window {
    let id: CGWindowID
    let frame: CGRect

    @MainActor
    func hasContentSize(width: CGFloat, height: CGFloat) -> Bool {
        let style: NSWindow.StyleMask = [.titled, .closable, .miniaturizable, .resizable]
        let content = NSWindow.contentRect(forFrameRect: frame, styleMask: style)
        return abs(content.width - width) <= frameTolerance && abs(content.height - height) <= frameTolerance
    }
}

enum Shortcut: String {
    case mini
    case focusQueue
    case activate
    case restore
}

let windowTimeout: TimeInterval = 15
let windowPollInterval: TimeInterval = 0.15
let redrawInterval: TimeInterval = 0.35
let compactWidth: CGFloat = 420
let expandedWidth: CGFloat = 740
let miniContentHeight: CGFloat = 560
let frameTolerance: CGFloat = 1

func visibleWindows(ownedBy pid: pid_t) throws -> [Window] {
    let options: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
    guard let entries = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]] else {
        throw SmokeError(description: "The runner has no macOS window server session.")
    }
    return entries.compactMap { entry in
        guard entry[kCGWindowOwnerPID as String] as? pid_t == pid,
              entry[kCGWindowLayer as String] as? Int == 0,
              let id = entry[kCGWindowNumber as String] as? CGWindowID,
              let bounds = entry[kCGWindowBounds as String] as? [String: Any],
              let frame = CGRect(dictionaryRepresentation: bounds as CFDictionary)
        else { return nil }
        return Window(id: id, frame: frame)
    }
}

@MainActor
func isFrontmost(_ app: Process) -> Bool {
    NSWorkspace.shared.frontmostApplication?.processIdentifier == app.processIdentifier
}

@MainActor
func waitForWindow(
    _ stage: String, of app: Process, matching: (Window) -> Bool
) throws -> Window {
    let deadline = ProcessInfo.processInfo.systemUptime + windowTimeout
    var windows: [Window] = []
    repeat {
        guard app.isRunning else {
            throw SmokeError(description: "Muzo exited with status \(app.terminationStatus) during \(stage).")
        }
        windows = try visibleWindows(ownedBy: app.processIdentifier)
        if isFrontmost(app), windows.count == 1, let window = windows.first, matching(window) {
            print("\(stage) window \(window.id), frame \(window.frame)")
            return window
        }
        // AppKit refreshes foreground application state as the run loop advances.
        RunLoop.current.run(until: Date(timeIntervalSinceNow: windowPollInterval))
    } while ProcessInfo.processInfo.systemUptime < deadline
    throw SmokeError(description: "Timed out waiting for \(stage). Visible Muzo windows: \(windows).")
}

func runTool(_ executable: String, _ arguments: [String]) throws {
    let tool = Process()
    tool.executableURL = URL(fileURLWithPath: executable)
    tool.arguments = arguments
    try tool.run()
    tool.waitUntilExit()
    guard tool.terminationReason == .exit, tool.terminationStatus == 0 else {
        throw SmokeError(description: "\(executable) failed with status \(tool.terminationStatus).")
    }
}

func send(_ shortcut: Shortcut, to app: Process) throws {
    let script = """
    on run argv
        set targetPID to (item 1 of argv) as integer
        set shortcutName to item 2 of argv
        set escapeKey to 53
        set tabKey to 48
        set returnKey to 36
        set tabSettleSeconds to 0.15
        tell application "System Events"
            set targetApp to first application process whose unix id is targetPID
            if not (frontmost of targetApp) then error "Muzo lost focus before sending a shortcut."
            if shortcutName is "mini" then
                keystroke "m" using command down
            else if shortcutName is "focusQueue" then
                key code tabKey
                delay tabSettleSeconds
                key code tabKey
            else if shortcutName is "activate" then
                key code returnKey
            else if shortcutName is "restore" then
                key code escapeKey
            end if
        end tell
    end run
    """
    try runTool("/usr/bin/osascript", ["-e", script, "--", String(app.processIdentifier), shortcut.rawValue])
}

@MainActor
func capture(_ window: Window, of app: Process, named name: String, in folder: URL) throws {
    RunLoop.current.run(until: Date(timeIntervalSinceNow: redrawInterval))
    guard isFrontmost(app) else {
        throw SmokeError(description: "Muzo lost focus before capturing \(name).")
    }
    let path = folder.appendingPathComponent("\(name).png").path
    try runTool("/usr/sbin/screencapture", ["-x", "-o", "-l", String(window.id), path])
    guard FileManager.default.fileExists(atPath: path) else {
        throw SmokeError(description: "The runner did not capture \(name).")
    }
}

@MainActor
func verifyMiniPlayer(bundle: URL, screenshots: URL) throws {
    let files = FileManager.default
    guard let executable = Bundle(url: bundle)?.executableURL, files.isExecutableFile(atPath: executable.path)
    else {
        throw SmokeError(description: "Muzo.app has no executable at \(bundle.path).")
    }

    let profile = files.temporaryDirectory.appendingPathComponent("muzo-mini-\(UUID().uuidString)")
    try files.createDirectory(at: profile, withIntermediateDirectories: false)
    defer { try? files.removeItem(at: profile) }
    try files.createDirectory(at: screenshots, withIntermediateDirectories: true)
    let fixture = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent().deletingLastPathComponent()
        .appendingPathComponent("fixtures/tone.mp3")
    let recording = profile.appendingPathComponent("Local track.mp3")
    try files.copyItem(at: fixture, to: recording)

    let app = Process()
    app.executableURL = executable
    app.arguments = [recording.path]
    app.currentDirectoryURL = profile
    var environment = ProcessInfo.processInfo.environment
    environment["MUZO_DATA_DIR"] = profile.path
    app.environment = environment
    try app.run()
    defer {
        if app.isRunning {
            app.terminate()
            app.waitUntilExit()
        }
    }

    let library = try waitForWindow("library", of: app) { _ in true }
    try capture(library, of: app, named: "macos-library", in: screenshots)

    try send(.mini, to: app)
    let compact = try waitForWindow("compact mini player", of: app) { window in
        window.id != library.id && window.hasContentSize(width: compactWidth, height: miniContentHeight)
    }
    try capture(compact, of: app, named: "macos-mini", in: screenshots)

    try send(.focusQueue, to: app)
    try send(.activate, to: app)
    let expanded = try waitForWindow("expanded mini player queue", of: app) { window in
        window.id == compact.id && window.hasContentSize(width: expandedWidth, height: miniContentHeight)
    }
    try capture(expanded, of: app, named: "macos-mini-queue", in: screenshots)

    try send(.activate, to: app)
    let collapsed = try waitForWindow("collapsed mini player queue", of: app) { window in
        window.id == compact.id && window.hasContentSize(width: compactWidth, height: miniContentHeight)
    }
    try capture(collapsed, of: app, named: "macos-mini-collapsed", in: screenshots)

    try send(.restore, to: app)
    let restored = try waitForWindow("restored library", of: app) { window in
        window.id == library.id && window.frame == library.frame
    }
    try capture(restored, of: app, named: "macos-library-restored", in: screenshots)
    print("Verified the native macOS mini player and restored library geometry.")
}

do {
    let arguments = CommandLine.arguments
    guard arguments.count == 3 else {
        throw SmokeError(description: "Usage: swift macos-mini.swift <Muzo.app> <screenshot directory>")
    }
    try MainActor.assumeIsolated {
        try verifyMiniPlayer(
            bundle: URL(fileURLWithPath: arguments[1], isDirectory: true),
            screenshots: URL(fileURLWithPath: arguments[2], isDirectory: true)
        )
    }
} catch {
    FileHandle.standardError.write(Data("macOS mini-player smoke test: \(error)\n".utf8))
    exit(EXIT_FAILURE)
}
