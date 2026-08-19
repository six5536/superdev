// Thread-safe circular log buffer.

import Foundation

/// A single log entry.
public struct LogEntry: Sendable {
    public enum Level: String, Sendable {
        case debug, info, warn, error
    }

    public let ts: Date
    public let level: Level
    public let tag: String
    public let message: String

    func toDict() -> [String: Any] {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return [
            "ts": formatter.string(from: ts),
            "level": level.rawValue,
            "tag": tag,
            "message": message
        ]
    }
}

/// Thread-safe circular buffer for log entries.
public final class LogBuffer: Sendable {
    private let queue = DispatchQueue(label: "com.nativedebugserver.logbuffer")
    private nonisolated(unsafe) var entries: [LogEntry] = []
    private let capacity: Int

    public init(capacity: Int = 1000) {
        self.capacity = capacity
    }

    /// Append a new log entry.
    public func append(level: LogEntry.Level, tag: String, message: String) {
        let entry = LogEntry(ts: Date(), level: level, tag: tag, message: message)
        queue.sync {
            entries.append(entry)
            if entries.count > capacity {
                entries.removeFirst(entries.count - capacity)
            }
        }
    }

    /// Get all entries, optionally filtered by timestamp.
    public func getEntries(since: Date? = nil) -> [LogEntry] {
        queue.sync {
            guard let since = since else { return entries }
            return entries.filter { $0.ts > since }
        }
    }

    /// Clear all entries.
    public func clear() {
        queue.sync {
            entries.removeAll()
        }
    }

    /// Current count.
    public var count: Int {
        queue.sync { entries.count }
    }
}
