// Tests for DebugRouter — route matching and dispatch.

package com.nativedebugserver

import org.json.JSONObject
import org.junit.Before
import org.junit.Test
import kotlin.test.assertEquals

class DebugRouterTest {

    private lateinit var router: DebugRouter

    @Before
    fun setup() {
        router = DebugRouter(LogBuffer())
        router.configure(appId = "com.test.app")
    }

    private fun get(path: String): HttpResponse {
        val (parsedPath, queryParams) = parsePath(path)
        return router.route(HttpRequest("GET", parsedPath, queryParams, null))
    }

    private fun post(path: String, body: Map<String, Any?>): HttpResponse {
        val json = JSONObject(body).toString()
        return router.route(HttpRequest("POST", path, emptyMap(), json))
    }

    private fun parsePath(raw: String): Pair<String, Map<String, String>> {
        val qIdx = raw.indexOf('?')
        if (qIdx < 0) return Pair(raw, emptyMap())
        val path = raw.substring(0, qIdx)
        val params = mutableMapOf<String, String>()
        raw.substring(qIdx + 1).split("&").forEach { pair ->
            val eqIdx = pair.indexOf('=')
            if (eqIdx > 0) {
                params[pair.substring(0, eqIdx)] = pair.substring(eqIdx + 1)
            }
        }
        return Pair(path, params)
    }

    private fun parseJSON(response: HttpResponse): JSONObject {
        return JSONObject(String(response.body))
    }

    // MARK: - Ping

    @Test
    fun `GET debug-ping returns ok`() {
        val response = get("/debug/ping")
        assertEquals(200, response.statusCode)

        val json = parseJSON(response)
        assertEquals("ok", json.getString("status"))
        assertEquals("android", json.getString("platform"))
        assertEquals("com.test.app", json.getString("appId"))
    }

    // MARK: - Logs

    @Test
    fun `GET debug-logs returns empty initially`() {
        val response = get("/debug/logs")
        assertEquals(200, response.statusCode)

        val json = parseJSON(response)
        assertEquals(0, json.getInt("count"))
    }

    @Test
    fun `POST debug-logs-clear clears buffer`() {
        val response = post("/debug/logs/clear", emptyMap())
        assertEquals(200, response.statusCode)

        val json = parseJSON(response)
        assertEquals(true, json.getBoolean("cleared"))
    }

    // MARK: - State

    @Test
    fun `GET debug-state returns registered providers`() {
        router.registerStateProvider("test") { mapOf("value" to "hello") }

        val response = get("/debug/state")
        assertEquals(200, response.statusCode)

        val json = parseJSON(response)
        val test = json.getJSONObject("test")
        assertEquals("hello", test.getString("value"))
    }

    @Test
    fun `GET debug-state-key returns specific provider`() {
        router.registerStateProvider("audio") { mapOf("playing" to false) }

        val response = get("/debug/state/audio")
        assertEquals(200, response.statusCode)

        val json = parseJSON(response)
        assertEquals(false, json.getBoolean("playing"))
    }

    @Test
    fun `GET debug-state-unknown returns 404`() {
        val response = get("/debug/state/nope")
        assertEquals(404, response.statusCode)
    }

    // MARK: - Navigate

    @Test
    fun `POST debug-navigate calls handler`() {
        router.registerNavigateHandler { route -> route == "/config" }

        val response = post("/debug/navigate", mapOf("route" to "/config"))
        assertEquals(200, response.statusCode)

        val json = parseJSON(response)
        assertEquals(true, json.getBoolean("navigated"))
        assertEquals("/config", json.getString("route"))
    }

    @Test
    fun `POST debug-navigate without route returns 400`() {
        val response = post("/debug/navigate", emptyMap())
        assertEquals(400, response.statusCode)
    }

    // MARK: - Action

    @Test
    fun `POST debug-action calls registered handler`() {
        router.registerAction("reset") { _ -> mapOf("done" to true) }

        val response = post("/debug/action", mapOf("type" to "reset", "payload" to emptyMap<String, Any>()))
        assertEquals(200, response.statusCode)

        val json = parseJSON(response)
        assertEquals(true, json.getBoolean("done"))
    }

    @Test
    fun `POST debug-action with unknown type returns 404`() {
        val response = post("/debug/action", mapOf("type" to "unknown"))
        assertEquals(404, response.statusCode)
    }

    // MARK: - 404

    @Test
    fun `unknown path returns 404`() {
        val response = get("/unknown")
        assertEquals(404, response.statusCode)
    }

    @Test
    fun `GET debug-routes aliases to state-routes`() {
        router.registerStateProvider("routes") { mapOf("current" to "/welcome") }

        val response = get("/debug/routes")
        assertEquals(200, response.statusCode)

        val json = parseJSON(response)
        assertEquals("/welcome", json.getString("current"))
    }
}
