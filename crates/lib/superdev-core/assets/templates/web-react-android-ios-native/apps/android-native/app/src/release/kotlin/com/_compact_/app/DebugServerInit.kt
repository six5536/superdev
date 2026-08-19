// Release stub — no debug server in release builds.

package com.{{superdev:project-compact}}.app

import android.app.Activity

/**
 * No-op stub for release builds. Debug server is not available.
 */
object DebugServerInit {
    fun start(activity: Activity) { /* no-op */ }
    fun stop() { /* no-op */ }
}
