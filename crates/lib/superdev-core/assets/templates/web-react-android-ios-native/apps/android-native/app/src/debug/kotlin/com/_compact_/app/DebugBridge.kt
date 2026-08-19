// Bridges app state to debug server handlers.

package com.{{superdev:project-compact}}.app

import com.nativedebugserver.DebugServer
import com.nativedebugserver.LogEntry

/**
 * Wires the debug server up with app state. Grow this alongside the app:
 * register a state provider per subsystem, a navigate handler once there are
 * routes, and an action per thing an agent should be able to trigger.
 */
object DebugBridge {

    @Volatile private var attached = false

    /**
     * Attach debug handlers. Safe to call multiple times (idempotent after first).
     */
    fun attach() {
        if (attached) return
        attached = true

        DebugServer.registerStateProvider("app") {
            mapOf("screen" to "home")
        }

        DebugServer.log(LogEntry.Level.INFO, "DebugBridge", "Handlers registered")
    }
}
