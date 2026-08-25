// Thread-safe circular log buffer for debug log entries.

package com.nativedebugserver

import java.time.Instant
import java.time.format.DateTimeFormatter

/**
 * A single log entry.
 */
data class LogEntry(
    val ts: Instant,
    val level: Level,
    val tag: String,
    val message: String
) {
    enum class Level { DEBUG, INFO, WARN, ERROR }

    fun toMap(): Map<String, Any> = mapOf(
        "ts" to DateTimeFormatter.ISO_INSTANT.format(ts),
        "level" to level.name.lowercase(),
        "tag" to tag,
        "message" to message
    )
}

/**
 * Thread-safe circular buffer for log entries.
 */
class LogBuffer(private val capacity: Int = 1000) {
    private val entries = mutableListOf<LogEntry>()
    private val lock = Any()

    fun append(level: LogEntry.Level, tag: String, message: String) {
        val entry = LogEntry(
            ts = Instant.now(),
            level = level,
            tag = tag,
            message = message
        )
        synchronized(lock) {
            entries.add(entry)
            if (entries.size > capacity) {
                entries.removeAt(0)
            }
        }
    }

    fun getEntries(since: Instant? = null): List<LogEntry> {
        synchronized(lock) {
            return if (since != null) {
                entries.filter { it.ts.isAfter(since) }
            } else {
                entries.toList()
            }
        }
    }

    fun clear() {
        synchronized(lock) {
            entries.clear()
        }
    }

    val count: Int
        get() = synchronized(lock) { entries.size }
}
