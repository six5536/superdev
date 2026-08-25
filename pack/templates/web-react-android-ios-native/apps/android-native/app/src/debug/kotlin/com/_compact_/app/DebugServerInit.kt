// Debug-only initializer for NativeDebugServer. This file lives in the debug
// source set — it is not included in release builds.

package com.{{superdev:project-compact}}.app

import android.app.Activity
import com.nativedebugserver.DebugServer

/**
 * Initialize the debug server. Called from MainActivity.onCreate() in debug builds.
 */
object DebugServerInit {
    fun start(activity: Activity) {
        DebugServer.start(
            port = 8081,
            appId = "com.{{superdev:project-compact}}.app",
            activity = activity
        )
    }

    fun stop() {
        DebugServer.stop()
    }
}
