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

    func hasSize(width: CGFloat, height: CGFloat) -> Bool {
        abs(frame.width - width) <= frameTolerance && abs(frame.height - height) <= frameTolerance
    }
}

enum Shortcut: String {
    case moveLibrary
    case mini
    case focusQueue
    case activate
    case toggleQueue
    case restore
}

let windowTimeout: TimeInterval = 15
let windowPollInterval: TimeInterval = 0.15
let redrawInterval: TimeInterval = 0.35
let compactWidth: CGFloat = 420
let expandedWidth: CGFloat = 740
let miniContentHeight: CGFloat = 560
let frameTolerance: CGFloat = 1
let libraryMoveDistance: CGFloat = 40
let captionCenterY: CGFloat = 16
let closeInset: CGFloat = 16
let maximizeInset: CGFloat = 48
let minimizeInset: CGFloat = 80
let pointerSettleInterval: TimeInterval = 0.08

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

func send(_ shortcut: Shortcut, to app: Process, arguments: [String] = []) throws {
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
            if shortcutName is "moveLibrary" then
                set targetX to (item 3 of argv) as integer
                set targetY to (item 4 of argv) as integer
                set position of first window of targetApp to {targetX, targetY}
            else if shortcutName is "mini" then
                keystroke "m" using command down
            else if shortcutName is "focusQueue" then
                key code tabKey
                delay tabSettleSeconds
                key code tabKey
            else if shortcutName is "activate" then
                key code returnKey
            else if shortcutName is "toggleQueue" then
                keystroke "l" using {command down, shift down}
            else if shortcutName is "restore" then
                key code escapeKey
            end if
        end tell
    end run
    """
    try runTool(
        "/usr/bin/osascript", ["-e", script, "--", String(app.processIdentifier), shortcut.rawValue] + arguments
    )
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
func clickCaption(_ window: Window, of app: Process, inset: CGFloat) throws {
    guard isFrontmost(app) else {
        throw SmokeError(description: "Muzo lost focus before clicking the title bar.")
    }
    let position = CGPoint(x: window.frame.maxX - inset, y: window.frame.minY + captionCenterY)
    let events: [CGEventType] = [.mouseMoved, .leftMouseDown, .leftMouseUp]
    for kind in events {
        guard let event = CGEvent(
            mouseEventSource: nil, mouseType: kind, mouseCursorPosition: position, mouseButton: .left
        ) else {
            throw SmokeError(description: "Could not create a title bar mouse event.")
        }
        event.post(tap: .cghidEventTap)
        RunLoop.current.run(until: Date(timeIntervalSinceNow: pointerSettleInterval))
    }
}

func verifyMinimized(_ window: Window, of app: Process) throws {
    let script = """
    on run argv
        set targetPID to (item 1 of argv) as integer
        set windowSize to {(item 2 of argv) as integer, (item 3 of argv) as integer}
        tell application "System Events"
            set targetApp to first application process whose unix id is targetPID
            repeat with candidate in windows of targetApp
                if size of candidate is windowSize then
                    if value of attribute "AXMinimized" of candidate is true then return
                end if
            end repeat
        end tell
        error "Muzo's compact window was not minimized."
    end run
    """
    try runTool(
        "/usr/bin/osascript",
        [
            "-e", script, "--", String(app.processIdentifier),
            String(Int(window.frame.width.rounded())), String(Int(window.frame.height.rounded())),
        ]
    )
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

    let startup = try waitForWindow("startup library", of: app) { _ in true }
    let x = startup.frame.minX
    let movedX = x >= libraryMoveDistance ? x - libraryMoveDistance : x + libraryMoveDistance
    try send(
        .moveLibrary, to: app,
        arguments: [String(Int(movedX.rounded())), String(Int(startup.frame.minY.rounded()))]
    )
    let library = try waitForWindow("moved library", of: app) { window in
        window.id == startup.id && abs(window.frame.minX - movedX) <= frameTolerance
            && window.frame.size == startup.frame.size
    }
    try capture(library, of: app, named: "macos-library", in: screenshots)

    try send(.mini, to: app)
    let compact = try waitForWindow("compact mini player", of: app) { window in
        window.id != library.id && window.hasSize(width: compactWidth, height: miniContentHeight)
    }
    try capture(compact, of: app, named: "macos-mini", in: screenshots)

    try clickCaption(compact, of: app, inset: maximizeInset)
    let maximized = try waitForWindow("maximized mini player", of: app) { window in
        window.id == compact.id && window.frame.width > compactWidth + frameTolerance
    }
    try clickCaption(maximized, of: app, inset: maximizeInset)
    _ = try waitForWindow("restored mini player", of: app) { window in
        window.id == compact.id && window.hasSize(width: compactWidth, height: miniContentHeight)
    }

    try send(.focusQueue, to: app)
    try send(.activate, to: app)
    let expanded = try waitForWindow("expanded mini player queue", of: app) { window in
        window.id == compact.id && window.hasSize(width: expandedWidth, height: miniContentHeight)
    }
    try capture(expanded, of: app, named: "macos-mini-queue", in: screenshots)

    try send(.toggleQueue, to: app)
    let collapsed = try waitForWindow("mini player queue collapsed with Command-Shift-L", of: app) { window in
        window.id == compact.id && window.hasSize(width: compactWidth, height: miniContentHeight)
    }
    try capture(collapsed, of: app, named: "macos-mini-collapsed", in: screenshots)

    try send(.restore, to: app)
    let restored = try waitForWindow("restored library", of: app) { window in
        window.id == library.id && window.frame == library.frame
    }
    try capture(restored, of: app, named: "macos-library-restored", in: screenshots)

    try send(.mini, to: app)
    let closeTarget = try waitForWindow("reopened mini player", of: app) { window in
        window.id != library.id && window.hasSize(width: compactWidth, height: miniContentHeight)
    }
    try clickCaption(closeTarget, of: app, inset: closeInset)
    _ = try waitForWindow("library after closing the mini player", of: app) { window in
        window.id == library.id && window.frame == library.frame
    }

    try send(.mini, to: app)
    let minimizeTarget = try waitForWindow("mini player before minimizing", of: app) { window in
        window.id != library.id && window.hasSize(width: compactWidth, height: miniContentHeight)
    }
    try clickCaption(minimizeTarget, of: app, inset: minimizeInset)
    let deadline = ProcessInfo.processInfo.systemUptime + windowTimeout
    repeat {
        guard app.isRunning else {
            throw SmokeError(description: "Muzo exited while minimizing the mini player.")
        }
        if try visibleWindows(ownedBy: app.processIdentifier).isEmpty {
            try verifyMinimized(minimizeTarget, of: app)
            print("Verified the macOS title bar controls, mini player queue and restored library geometry.")
            return
        }
        RunLoop.current.run(until: Date(timeIntervalSinceNow: windowPollInterval))
    } while ProcessInfo.processInfo.systemUptime < deadline
    throw SmokeError(description: "Muzo still had a visible window after clicking Minimize.")
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
