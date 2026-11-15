import AppKit
import Foundation

let shutdownTimeoutSeconds: TimeInterval = 10
let pollIntervalSeconds: TimeInterval = 0.1
guard CommandLine.arguments.count == 2,
      let pid = pid_t(CommandLine.arguments[1]),
      let application = NSRunningApplication(processIdentifier: pid),
      application.terminate()
else {
    FileHandle.standardError.write(Data("Could not ask Muzo to quit.\n".utf8))
    exit(1)
}

let deadline = ProcessInfo.processInfo.systemUptime + shutdownTimeoutSeconds
while !application.isTerminated && ProcessInfo.processInfo.systemUptime < deadline {
    RunLoop.current.run(until: Date(timeIntervalSinceNow: pollIntervalSeconds))
}
if !application.isTerminated {
    FileHandle.standardError.write(Data("Muzo did not quit within the profile test timeout.\n".utf8))
    exit(1)
}
