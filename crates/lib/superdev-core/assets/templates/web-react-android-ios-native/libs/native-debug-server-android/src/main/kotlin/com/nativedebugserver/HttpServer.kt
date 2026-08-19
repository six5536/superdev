// Minimal HTTP/1.1 server using raw ServerSocket. Zero external dependencies.

package com.nativedebugserver

import java.io.BufferedReader
import java.io.InputStreamReader
import java.io.OutputStream
import java.net.ServerSocket
import java.net.Socket
import java.net.URLDecoder
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Minimal HTTP server using raw ServerSocket.
 * Runs on a background thread, handles connections via a thread pool.
 */
class HttpServer(private val router: DebugRouter) {
    private var serverSocket: ServerSocket? = null
    private var serverThread: Thread? = null
    private var executor: ExecutorService? = null
    private val running = AtomicBoolean(false)

    fun start(port: Int) {
        if (running.get()) return

        running.set(true)
        executor = Executors.newFixedThreadPool(4)

        serverThread = Thread({
            try {
                val socket = ServerSocket(port)
                socket.reuseAddress = true
                serverSocket = socket
                println("[DebugServer] Listening on port $port")

                while (running.get()) {
                    try {
                        val client = socket.accept()
                        executor?.execute { handleConnection(client) }
                    } catch (_: Exception) {
                        if (!running.get()) break
                    }
                }
            } catch (e: Exception) {
                println("[DebugServer] Failed to start: ${e.message}")
            }
        }, "DebugServer-Listener")
        serverThread?.isDaemon = true
        serverThread?.start()
    }

    fun stop() {
        running.set(false)
        serverSocket?.close()
        serverSocket = null
        executor?.shutdownNow()
        executor = null
        serverThread = null
    }

    private fun handleConnection(client: Socket) {
        try {
            client.soTimeout = 5000
            val reader = BufferedReader(InputStreamReader(client.getInputStream()))
            val output = client.getOutputStream()

            val request = parseRequest(reader)
            if (request != null) {
                val response = router.route(request)
                sendResponse(response, output)
            }
        } catch (_: Exception) {
            // Connection error — ignore
        } finally {
            try { client.close() } catch (_: Exception) {}
        }
    }

    // MARK: - HTTP Parsing

    private fun parseRequest(reader: BufferedReader): HttpRequest? {
        val requestLine = reader.readLine() ?: return null
        val parts = requestLine.split(" ")
        if (parts.size < 2) return null

        val method = parts[0]
        val rawPath = parts[1]

        // Read headers
        var contentLength = 0
        val headers = mutableMapOf<String, String>()
        while (true) {
            val line = reader.readLine() ?: break
            if (line.isEmpty()) break
            val colonIdx = line.indexOf(':')
            if (colonIdx > 0) {
                val key = line.substring(0, colonIdx).trim().lowercase()
                val value = line.substring(colonIdx + 1).trim()
                headers[key] = value
                if (key == "content-length") {
                    contentLength = value.toIntOrNull() ?: 0
                }
            }
        }

        // Read body
        val body = if (contentLength > 0) {
            val chars = CharArray(contentLength)
            var read = 0
            while (read < contentLength) {
                val n = reader.read(chars, read, contentLength - read)
                if (n < 0) break
                read += n
            }
            String(chars, 0, read)
        } else null

        // Parse path and query
        val (path, queryParams) = parsePathAndQuery(rawPath)

        return HttpRequest(method, path, queryParams, body)
    }

    private fun parsePathAndQuery(rawPath: String): Pair<String, Map<String, String>> {
        val qIdx = rawPath.indexOf('?')
        if (qIdx < 0) return Pair(rawPath, emptyMap())

        val path = rawPath.substring(0, qIdx)
        val queryString = rawPath.substring(qIdx + 1)
        val params = mutableMapOf<String, String>()

        queryString.split("&").forEach { pair ->
            val eqIdx = pair.indexOf('=')
            if (eqIdx > 0) {
                val key = URLDecoder.decode(pair.substring(0, eqIdx), "UTF-8")
                val value = URLDecoder.decode(pair.substring(eqIdx + 1), "UTF-8")
                params[key] = value
            }
        }

        return Pair(path, params)
    }

    // MARK: - Response

    private fun sendResponse(response: HttpResponse, output: OutputStream) {
        val statusText = httpStatusText(response.statusCode)
        val header = buildString {
            append("HTTP/1.1 ${response.statusCode} $statusText\r\n")
            append("Content-Type: ${response.contentType}\r\n")
            append("Content-Length: ${response.body.size}\r\n")
            append("Connection: close\r\n")
            append("Access-Control-Allow-Origin: *\r\n")
            append("\r\n")
        }

        output.write(header.toByteArray())
        output.write(response.body)
        output.flush()
    }

    private fun httpStatusText(code: Int): String = when (code) {
        200 -> "OK"
        400 -> "Bad Request"
        404 -> "Not Found"
        500 -> "Internal Server Error"
        else -> "Unknown"
    }
}
