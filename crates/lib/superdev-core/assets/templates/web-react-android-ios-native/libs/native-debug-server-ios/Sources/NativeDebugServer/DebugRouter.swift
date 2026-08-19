// Route matching and dispatch for debug server endpoints.

#if DEBUG
import Foundation

/// Parsed HTTP request.
struct HTTPRequest {
    let method: String
    let path: String
    let queryParams: [String: String]
    let body: Data?

    var bodyJSON: [String: Any]? {
        guard let body = body else { return nil }
        return try? JSONSerialization.jsonObject(with: body) as? [String: Any]
    }
}

/// HTTP response to send back.
struct HTTPResponse {
    let statusCode: Int
    let contentType: String
    let body: Data

    static func json(_ dict: [String: Any], status: Int = 200) -> HTTPResponse {
        let data = (try? JSONSerialization.data(withJSONObject: dict, options: [.sortedKeys])) ?? Data()
        return HTTPResponse(statusCode: status, contentType: "application/json", body: data)
    }

    static func png(_ data: Data) -> HTTPResponse {
        HTTPResponse(statusCode: 200, contentType: "image/png", body: data)
    }

    static func error(_ message: String, status: Int = 400) -> HTTPResponse {
        json(["error": message], status: status)
    }

    static let notFound = error("Not found", status: 404)
}

/// Routes incoming HTTP requests to appropriate handlers.
final class DebugRouter: @unchecked Sendable {
    private let logBuffer: LogBuffer
    private var appId: String = "unknown"

    private let lock = NSLock()
    private var stateProviders: [String: @Sendable () -> [String: Any]] = [:]
    private var actionHandlers: [String: @Sendable ([String: Any]) -> [String: Any]] = [:]
    private var navigateHandler: (@Sendable (String) -> Bool)?
    private var screenshotProvider: (@Sendable () -> Data?)?

    init(logBuffer: LogBuffer) {
        self.logBuffer = logBuffer
    }

    func configure(appId: String) {
        self.appId = appId
    }

    func registerStateProvider(_ key: String, provider: @escaping @Sendable () -> [String: Any]) {
        lock.lock()
        stateProviders[key] = provider
        lock.unlock()
    }

    func registerAction(_ type: String, handler: @escaping @Sendable ([String: Any]) -> [String: Any]) {
        lock.lock()
        actionHandlers[type] = handler
        lock.unlock()
    }

    func registerNavigateHandler(_ handler: @escaping @Sendable (String) -> Bool) {
        lock.lock()
        navigateHandler = handler
        lock.unlock()
    }

    func registerScreenshotProvider(_ provider: @escaping @Sendable () -> Data?) {
        lock.lock()
        screenshotProvider = provider
        lock.unlock()
    }

    /// Route the request and return a response.
    func route(_ request: HTTPRequest) -> HTTPResponse {
        let pathComponents = request.path.split(separator: "/").map(String.init)

        // All routes are under /debug
        guard pathComponents.first == "debug" else {
            return .notFound
        }

        let subPath = Array(pathComponents.dropFirst())

        switch (request.method, subPath) {
        case ("GET", ["ping"]):
            return handlePing()

        case ("GET", ["logs"]):
            return handleGetLogs(since: request.queryParams["since"])

        case ("POST", ["logs", "clear"]):
            return handleClearLogs()

        case ("GET", ["screenshot"]):
            return handleScreenshot()

        case ("GET", ["state"]):
            return handleGetAllState()

        case ("GET", _) where subPath.count == 2 && subPath[0] == "state":
            return handleGetState(key: subPath[1])

        case ("GET", ["routes"]):
            return handleGetState(key: "routes")

        case ("POST", ["navigate"]):
            return handleNavigate(request)

        case ("POST", ["action"]):
            return handleAction(request)

        default:
            return .notFound
        }
    }

    // MARK: - Core Handlers

    private func handlePing() -> HTTPResponse {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return .json([
            "status": "ok",
            "platform": "ios",
            "appId": appId,
            "timestamp": formatter.string(from: Date())
        ])
    }

    private func handleGetLogs(since: String?) -> HTTPResponse {
        var sinceDate: Date? = nil
        if let since = since {
            let formatter = ISO8601DateFormatter()
            formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
            sinceDate = formatter.date(from: since)
        }
        let entries = logBuffer.getEntries(since: sinceDate)
        return .json([
            "entries": entries.map { $0.toDict() },
            "count": entries.count
        ])
    }

    private func handleClearLogs() -> HTTPResponse {
        logBuffer.clear()
        return .json(["cleared": true])
    }

    private func handleScreenshot() -> HTTPResponse {
        lock.lock()
        let provider = screenshotProvider
        lock.unlock()

        guard let provider = provider, let data = provider() else {
            return .error("Screenshot capture not available", status: 500)
        }
        return .png(data)
    }

    // MARK: - State Handlers

    private func handleGetAllState() -> HTTPResponse {
        lock.lock()
        let providers = stateProviders
        lock.unlock()

        var state: [String: Any] = [:]
        for (key, provider) in providers {
            state[key] = provider()
        }
        return .json(state)
    }

    private func handleGetState(key: String) -> HTTPResponse {
        lock.lock()
        let provider = stateProviders[key]
        lock.unlock()

        guard let provider = provider else {
            return .error("Unknown state key: \(key)", status: 404)
        }
        return .json(provider())
    }

    // MARK: - Action Handlers

    private func handleNavigate(_ request: HTTPRequest) -> HTTPResponse {
        guard let body = request.bodyJSON, let route = body["route"] as? String else {
            return .error("Missing 'route' in request body")
        }

        lock.lock()
        let handler = navigateHandler
        lock.unlock()

        guard let handler = handler else {
            return .error("No navigate handler registered", status: 500)
        }

        let success = handler(route)
        return .json(["navigated": success, "route": route])
    }

    private func handleAction(_ request: HTTPRequest) -> HTTPResponse {
        guard let body = request.bodyJSON, let type = body["type"] as? String else {
            return .error("Missing 'type' in request body")
        }

        let payload = body["payload"] as? [String: Any] ?? [:]

        lock.lock()
        let handler = actionHandlers[type]
        lock.unlock()

        guard let handler = handler else {
            return .error("Unknown action: \(type)", status: 404)
        }

        let result = handler(payload)
        return .json(result)
    }
}

#endif
