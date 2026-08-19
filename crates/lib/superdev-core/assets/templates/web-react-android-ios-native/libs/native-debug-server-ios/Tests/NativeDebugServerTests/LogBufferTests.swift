// Tests for LogBuffer — thread-safe circular buffer.

import Testing
import Foundation
@testable import NativeDebugServer

@Suite("LogBuffer Tests")
struct LogBufferTests {

    @Test("appends and retrieves entries")
    func appendAndRetrieve() {
        let buffer = LogBuffer(capacity: 10)
        buffer.append(level: .info, tag: "Test", message: "Hello")
        buffer.append(level: .warn, tag: "Test", message: "World")

        let entries = buffer.getEntries()
        #expect(entries.count == 2)
        #expect(entries[0].message == "Hello")
        #expect(entries[1].message == "World")
        #expect(entries[0].level == .info)
        #expect(entries[1].level == .warn)
    }

    @Test("enforces capacity limit")
    func capacityLimit() {
        let buffer = LogBuffer(capacity: 3)
        for i in 0..<5 {
            buffer.append(level: .debug, tag: "T", message: "msg\(i)")
        }

        let entries = buffer.getEntries()
        #expect(entries.count == 3)
        #expect(entries[0].message == "msg2")
        #expect(entries[1].message == "msg3")
        #expect(entries[2].message == "msg4")
    }

    @Test("filters by since timestamp")
    func filterBySince() throws {
        let buffer = LogBuffer(capacity: 100)
        buffer.append(level: .info, tag: "T", message: "old")

        // Small delay to ensure timestamp difference
        let midpoint = Date()
        Thread.sleep(forTimeInterval: 0.01)

        buffer.append(level: .info, tag: "T", message: "new")

        let filtered = buffer.getEntries(since: midpoint)
        #expect(filtered.count == 1)
        #expect(filtered[0].message == "new")
    }

    @Test("clears all entries")
    func clearEntries() {
        let buffer = LogBuffer(capacity: 10)
        buffer.append(level: .info, tag: "T", message: "a")
        buffer.append(level: .info, tag: "T", message: "b")
        #expect(buffer.count == 2)

        buffer.clear()
        #expect(buffer.count == 0)
        #expect(buffer.getEntries().isEmpty)
    }

    @Test("LogEntry toDict produces correct format")
    func logEntryToDict() {
        let entry = LogEntry(ts: Date(), level: .error, tag: "Audio", message: "Failed")
        let dict = entry.toDict()

        #expect(dict["level"] as? String == "error")
        #expect(dict["tag"] as? String == "Audio")
        #expect(dict["message"] as? String == "Failed")
        #expect(dict["ts"] is String)
    }
}
