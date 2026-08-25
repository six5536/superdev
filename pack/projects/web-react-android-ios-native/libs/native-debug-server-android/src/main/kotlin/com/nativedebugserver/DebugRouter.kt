// Route matching and dispatch for debug server endpoints.

package com.nativedebugserver

import org.json.JSONArray
import org.json.JSONObject
import java.time.Instant
import java.time.format.DateTimeFormatter

/**
 * Parsed HTTP request.
 */
data class HttpRequest(
    val method: String,
    val path: String,
    val queryParams: Map<String, String>,
    val body: String?
) {
    val bodyJSON: JSONObject?
        get() = body?.takeIf { it.isNotBlank() }?.let {
            try { JSONObject(it) } catch (_: Exception) { null }
        }
}

/**
 * HTTP response to send.
 */
data class HttpResponse(
    val statusCode: Int,
    val contentType: String,
    val body: ByteArray
) {
    companion object {
        fun json(obj: JSONObject, status: Int = 200): HttpResponse {
            return HttpResponse(status, "application/json", obj.toString().toByteArray())
        }

        fun json(map: Map<String, Any?>, status: Int = 200): HttpResponse {
            return json(JSONObject(map), status)
        }

        fun png(data: ByteArray): HttpResponse {
            return HttpResponse(200, "image/png", data)
        }

        fun error(message: String, status: Int = 400): HttpResponse {
            return json(mapOf("error" to message), status)
        }

        val notFound = error("Not found", 404)
    }
}

/**
 * Routes incoming HTTP requests to appropriate handlers.
 */
class DebugRouter(private val logBuffer: LogBuffer) {
    private var appId: String = "unknown"

    private val lock = Any()
    private val stateProviders = mutableMapOf<String, () -> Map<String, Any?>>()
    private val actionHandlers = mutableMapOf<String, (Map<String, Any?>) -> Map<String, Any?>>()
    private var navigateHandler: ((String) -> Boolean)? = null
    private var screenshotProvider: (() -> ByteArray?)? = null

    fun configure(appId: String) {
        this.appId = appId
    }

    fun registerStateProvider(key: String, provider: () -> Map<String, Any?>) {
        synchronized(lock) { stateProviders[key] = provider }
    }

    fun registerAction(type: String, handler: (Map<String, Any?>) -> Map<String, Any?>) {
        synchronized(lock) { actionHandlers[type] = handler }
    }

    fun registerNavigateHandler(handler: (String) -> Boolean) {
        synchronized(lock) { navigateHandler = handler }
    }

    fun registerScreenshotProvider(provider: () -> ByteArray?) {
        synchronized(lock) { screenshotProvider = provider }
    }

    fun route(request: HttpRequest): HttpResponse {
        val parts = request.path.trimStart('/').split("/")
        if (parts.firstOrNull() != "debug") return HttpResponse.notFound

        val subPath = parts.drop(1)

        return when {
            request.method == "GET" && subPath == listOf("ping") -> handlePing()
            request.method == "GET" && subPath == listOf("logs") -> handleGetLogs(request.queryParams["since"])
            request.method == "POST" && subPath == listOf("logs", "clear") -> handleClearLogs()
            request.method == "GET" && subPath == listOf("screenshot") -> handleScreenshot()
            request.method == "GET" && subPath == listOf("state") -> handleGetAllState()
            request.method == "GET" && subPath.size == 2 && subPath[0] == "state" -> handleGetState(subPath[1])
            request.method == "GET" && subPath == listOf("routes") -> handleGetState("routes")
            request.method == "POST" && subPath == listOf("navigate") -> handleNavigate(request)
            request.method == "POST" && subPath == listOf("action") -> handleAction(request)
            else -> HttpResponse.notFound
        }
    }

    // MARK: - Core Handlers

    private fun handlePing(): HttpResponse {
        return HttpResponse.json(mapOf(
            "status" to "ok",
            "platform" to "android",
            "appId" to appId,
            "timestamp" to DateTimeFormatter.ISO_INSTANT.format(Instant.now())
        ))
    }

    private fun handleGetLogs(since: String?): HttpResponse {
        val sinceInstant = since?.let {
            try { Instant.parse(it) } catch (_: Exception) { null }
        }
        val entries = logBuffer.getEntries(sinceInstant)
        val entriesArray = JSONArray()
        entries.forEach { entriesArray.put(JSONObject(it.toMap())) }

        return HttpResponse.json(JSONObject().apply {
            put("entries", entriesArray)
            put("count", entries.size)
        })
    }

    private fun handleClearLogs(): HttpResponse {
        logBuffer.clear()
        return HttpResponse.json(mapOf("cleared" to true))
    }

    private fun handleScreenshot(): HttpResponse {
        val provider = synchronized(lock) { screenshotProvider }
        val data = provider?.invoke()
            ?: return HttpResponse.error("Screenshot capture not available", 500)
        return HttpResponse.png(data)
    }

    // MARK: - State Handlers

    private fun handleGetAllState(): HttpResponse {
        val providers = synchronized(lock) { stateProviders.toMap() }
        val state = mutableMapOf<String, Any?>()
        providers.forEach { (key, provider) -> state[key] = provider() }
        return HttpResponse.json(JSONObject(state as Map<String, Any?>))
    }

    private fun handleGetState(key: String): HttpResponse {
        val provider = synchronized(lock) { stateProviders[key] }
            ?: return HttpResponse.error("Unknown state key: $key", 404)
        return HttpResponse.json(JSONObject(provider()))
    }

    // MARK: - Action Handlers

    private fun handleNavigate(request: HttpRequest): HttpResponse {
        val body = request.bodyJSON ?: return HttpResponse.error("Missing 'route' in request body")
        val route = body.optString("route", "").takeIf { it.isNotEmpty() }
            ?: return HttpResponse.error("Missing 'route' in request body")

        val handler = synchronized(lock) { navigateHandler }
            ?: return HttpResponse.error("No navigate handler registered", 500)

        val success = handler(route)
        return HttpResponse.json(mapOf("navigated" to success, "route" to route))
    }

    private fun handleAction(request: HttpRequest): HttpResponse {
        val body = request.bodyJSON ?: return HttpResponse.error("Missing 'type' in request body")
        val type = body.optString("type", "").takeIf { it.isNotEmpty() }
            ?: return HttpResponse.error("Missing 'type' in request body")

        val payload = body.optJSONObject("payload")?.let { jsonToMap(it) } ?: emptyMap()

        val handler = synchronized(lock) { actionHandlers[type] }
            ?: return HttpResponse.error("Unknown action: $type", 404)

        val result = handler(payload)
        return HttpResponse.json(JSONObject(result as Map<String, Any?>))
    }

    private fun jsonToMap(json: JSONObject): Map<String, Any?> {
        val map = mutableMapOf<String, Any?>()
        json.keys().forEach { key -> map[key] = json.get(key) }
        return map
    }
}
