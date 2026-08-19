// Tests for LogBuffer — thread-safe circular buffer.

package com.nativedebugserver

import org.junit.Test
import java.time.Instant
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class LogBufferTest {

    @Test
    fun `appends and retrieves entries`() {
        val buffer = LogBuffer(capacity = 10)
        buffer.append(LogEntry.Level.INFO, "Test", "Hello")
        buffer.append(LogEntry.Level.WARN, "Test", "World")

        val entries = buffer.getEntries()
        assertEquals(2, entries.size)
        assertEquals("Hello", entries[0].message)
        assertEquals("World", entries[1].message)
        assertEquals(LogEntry.Level.INFO, entries[0].level)
        assertEquals(LogEntry.Level.WARN, entries[1].level)
    }

    @Test
    fun `enforces capacity limit`() {
        val buffer = LogBuffer(capacity = 3)
        for (i in 0 until 5) {
            buffer.append(LogEntry.Level.DEBUG, "T", "msg$i")
        }

        val entries = buffer.getEntries()
        assertEquals(3, entries.size)
        assertEquals("msg2", entries[0].message)
        assertEquals("msg3", entries[1].message)
        assertEquals("msg4", entries[2].message)
    }

    @Test
    fun `filters by since timestamp`() {
        val buffer = LogBuffer(capacity = 100)
        buffer.append(LogEntry.Level.INFO, "T", "old")

        val midpoint = Instant.now()
        Thread.sleep(10)

        buffer.append(LogEntry.Level.INFO, "T", "new")

        val filtered = buffer.getEntries(since = midpoint)
        assertEquals(1, filtered.size)
        assertEquals("new", filtered[0].message)
    }

    @Test
    fun `clears all entries`() {
        val buffer = LogBuffer(capacity = 10)
        buffer.append(LogEntry.Level.INFO, "T", "a")
        buffer.append(LogEntry.Level.INFO, "T", "b")
        assertEquals(2, buffer.count)

        buffer.clear()
        assertEquals(0, buffer.count)
        assertTrue(buffer.getEntries().isEmpty())
    }

    @Test
    fun `LogEntry toMap produces correct format`() {
        val entry = LogEntry(
            ts = Instant.parse("2026-02-27T12:00:00Z"),
            level = LogEntry.Level.ERROR,
            tag = "Audio",
            message = "Failed"
        )
        val map = entry.toMap()

        assertEquals("error", map["level"])
        assertEquals("Audio", map["tag"])
        assertEquals("Failed", map["message"])
        assertEquals("2026-02-27T12:00:00Z", map["ts"])
    }
}
