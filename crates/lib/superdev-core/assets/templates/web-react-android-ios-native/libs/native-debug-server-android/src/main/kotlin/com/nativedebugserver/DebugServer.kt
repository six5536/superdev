// NativeDebugServer — Lightweight HTTP debug server for Android apps.
// Uses raw ServerSocket for zero-dependency HTTP serving.

package com.nativedebugserver

import android.app.Activity
import android.graphics.Bitmap
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.view.PixelCopy
import java.io.ByteArrayOutputStream
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

/**
 * Lightweight HTTP debug server for exposing app state to AI agents and tools.
 *
 * Usage:
 * ```kotlin
 * DebugServer.start(port = 8080)
 * DebugServer.registerStateProvider("routes") { mapOf("current" to currentRoute) }
 * DebugServer.registerAction("navigate") { params -> navigateTo(params) }
 * DebugServer.log(LogEntry.Level.INFO, "App", "Started")
 * ```
 */
object DebugServer {
    private val logBuffer = LogBuffer()
    private val router = DebugRouter(logBuffer)
    private val server = HttpServer(router)
    private var activity: Activity? = null

    /**
     * Start the debug server. Call from Activity.onCreate() in debug builds.
     * @param port TCP port to listen on (default 8080)
     * @param appId Application identifier (default from BuildConfig)
     * @param activity Activity reference for screenshot capture
     */
    fun start(port: Int = 8080, appId: String = "unknown", activity: Activity? = null) {
        this.activity = activity
        router.configure(appId = appId)

        // Register screenshot provider if activity available
        if (activity != null) {
            router.registerScreenshotProvider { captureScreenshot() }
        }

        server.start(port)
        log(LogEntry.Level.INFO, "DebugServer", "Started on port $port")
    }

    /**
     * Stop the debug server.
     */
    fun stop() {
        server.stop()
        activity = null
        log(LogEntry.Level.INFO, "DebugServer", "Stopped")
    }

    /**
     * Register a state provider called on GET /debug/state/<key>.
     */
    fun registerStateProvider(key: String, provider: () -> Map<String, Any?>) {
        router.registerStateProvider(key, provider)
    }

    /**
     * Register an action handler called on POST /debug/action with matching type.
     */
    fun registerAction(type: String, handler: (Map<String, Any?>) -> Map<String, Any?>) {
        router.registerAction(type, handler)
    }

    /**
     * Register a navigate handler for POST /debug/navigate.
     */
    fun registerNavigateHandler(handler: (String) -> Boolean) {
        router.registerNavigateHandler(handler)
    }

    /**
     * Add a log entry to the buffer.
     */
    fun log(level: LogEntry.Level, tag: String, message: String) {
        logBuffer.append(level, tag, message)
    }

    // MARK: - Screenshot

    @Suppress("DEPRECATION")
    private fun captureScreenshot(): ByteArray? {
        val act = activity ?: return null

        var bitmap: Bitmap? = null
        val latch = CountDownLatch(1)

        Handler(Looper.getMainLooper()).post {
            try {
                val window = act.window
                val view = window.decorView.rootView
                val width = view.width
                val height = view.height

                if (width <= 0 || height <= 0) {
                    latch.countDown()
                    return@post
                }

                val bmp = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)

                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                    PixelCopy.request(window, bmp, { result ->
                        if (result == PixelCopy.SUCCESS) {
                            bitmap = bmp
                        }
                        latch.countDown()
                    }, Handler(Looper.getMainLooper()))
                } else {
                    view.isDrawingCacheEnabled = true
                    view.buildDrawingCache()
                    val cache = view.drawingCache
                    if (cache != null) {
                        bitmap = cache.copy(Bitmap.Config.ARGB_8888, false)
                    }
                    view.isDrawingCacheEnabled = false
                    latch.countDown()
                }
            } catch (_: Exception) {
                latch.countDown()
            }
        }

        latch.await(3, TimeUnit.SECONDS)

        return bitmap?.let { bmp ->
            val stream = ByteArrayOutputStream()
            bmp.compress(Bitmap.CompressFormat.PNG, 100, stream)
            stream.toByteArray()
        }
    }
}
