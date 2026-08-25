// Tests for DebugRouter — route matching and dispatch.

#if DEBUG
import Testing
import Foundation
@testable import NativeDebugServer

@Suite("DebugRouter Tests")
struct DebugRouterTests {

    private func makeRouter() -> DebugRouter {
        let buffer = LogBuffer()
        let router = DebugRouter(logBuffer: buffer)
        router.configure(appId: "com.test.app")
        return router
    }

    private func get(_ path: String, router: DebugRouter) -> HTTPResponse {
        let (parsedPath, queryParams) = parsePath(path)
        let request = HTTPRequest(method: "GET", path: parsedPath, queryParams: queryParams, body: nil)
        return router.route(request)
    }

    private func post(_ path: String, body: [String: Any], router: DebugRouter) -> HTTPResponse {
        let data = try? JSONSerialization.data(withJSONObject: body)
        let request = HTTPRequest(method: "POST", path: path, queryParams: [:], body: data)
        return router.route(request)
    }

    private func parsePath(_ raw: String) -> (String, [String: String]) {
        let parts = raw.split(separator: "?", maxSplits: 1)
        let path = String(parts[0])
        var params: [String: String] = [:]
        if parts.count > 1 {
            for pair in parts[1].split(separator: "&") {
                let kv = pair.split(separator: "=", maxSplits: 1)
                if kv.count == 2 { params[String(kv[0])] = String(kv[1]) }
            }
        }
        return (path, params)
    }

    private func parseJSON(_ response: HTTPResponse) -> [String: Any]? {
        try? JSONSerialization.jsonObject(with: response.body) as? [String: Any]
    }

    // MARK: - Ping

    @Test("GET /debug/ping returns ok")
    func ping() {
        let router = makeRouter()
        let response = get("/debug/ping", router: router)

        #expect(response.statusCode == 200)
        let json = parseJSON(response)
        #expect(json?["status"] as? String == "ok")
        #expect(json?["platform"] as? String == "ios")
        #expect(json?["appId"] as? String == "com.test.app")
    }

    // MARK: - Logs

    @Test("GET /debug/logs returns empty initially")
    func logsEmpty() {
        let router = makeRouter()
        let response = get("/debug/logs", router: router)

        #expect(response.statusCode == 200)
        let json = parseJSON(response)
        #expect(json?["count"] as? Int == 0)
    }

    @Test("POST /debug/logs/clear clears buffer")
    func logsClear() {
        let router = makeRouter()
        let response = post("/debug/logs/clear", body: [:], router: router)

        #expect(response.statusCode == 200)
        let json = parseJSON(response)
        #expect(json?["cleared"] as? Bool == true)
    }

    // MARK: - State

    @Test("GET /debug/state returns registered providers")
    func stateAll() {
        let router = makeRouter()
        router.registerStateProvider("test") { ["value": "hello"] }

        let response = get("/debug/state", router: router)
        #expect(response.statusCode == 200)

        let json = parseJSON(response)
        let test = json?["test"] as? [String: Any]
        #expect(test?["value"] as? String == "hello")
    }

    @Test("GET /debug/state/key returns specific provider")
    func stateByKey() {
        let router = makeRouter()
        router.registerStateProvider("audio") { ["playing": false] }

        let response = get("/debug/state/audio", router: router)
        #expect(response.statusCode == 200)

        let json = parseJSON(response)
        #expect(json?["playing"] as? Bool == false)
    }

    @Test("GET /debug/state/unknown returns 404")
    func stateNotFound() {
        let router = makeRouter()
        let response = get("/debug/state/nope", router: router)
        #expect(response.statusCode == 404)
    }

    // MARK: - Navigate

    @Test("POST /debug/navigate calls handler")
    func navigate() {
        let router = makeRouter()
        router.registerNavigateHandler { route in
            return route == "/config"
        }

        let response = post("/debug/navigate", body: ["route": "/config"], router: router)
        #expect(response.statusCode == 200)

        let json = parseJSON(response)
        #expect(json?["navigated"] as? Bool == true)
        #expect(json?["route"] as? String == "/config")
    }

    @Test("POST /debug/navigate without route returns 400")
    func navigateMissingRoute() {
        let router = makeRouter()
        let response = post("/debug/navigate", body: [:], router: router)
        #expect(response.statusCode == 400)
    }

    // MARK: - Action

    @Test("POST /debug/action calls registered handler")
    func action() {
        let router = makeRouter()
        router.registerAction("reset") { _ in ["done": true] }

        let response = post("/debug/action", body: ["type": "reset", "payload": [:] as [String: Any]], router: router)
        #expect(response.statusCode == 200)

        let json = parseJSON(response)
        #expect(json?["done"] as? Bool == true)
    }

    @Test("POST /debug/action with unknown type returns 404")
    func actionNotFound() {
        let router = makeRouter()
        let response = post("/debug/action", body: ["type": "unknown"], router: router)
        #expect(response.statusCode == 404)
    }

    // MARK: - 404

    @Test("unknown path returns 404")
    func unknownPath() {
        let router = makeRouter()
        let response = get("/unknown", router: router)
        #expect(response.statusCode == 404)
    }

    @Test("GET /debug/routes aliases to state/routes")
    func routesAlias() {
        let router = makeRouter()
        router.registerStateProvider("routes") { ["current": "/welcome"] }

        let response = get("/debug/routes", router: router)
        #expect(response.statusCode == 200)

        let json = parseJSON(response)
        #expect(json?["current"] as? String == "/welcome")
    }
}

#endif
