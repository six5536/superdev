// Minimal HTTP/1.1 server using Network.framework (NWListener).
// Zero external dependencies — uses only Apple platform frameworks.

#if DEBUG
import Foundation
import Network

/// Minimal HTTP server built on Network.framework's NWListener.
final class HTTPServer: @unchecked Sendable {
    private let router: DebugRouter
    private var listener: NWListener?
    private let serverQueue = DispatchQueue(label: "com.nativedebugserver.http")

    init(router: DebugRouter) {
        self.router = router
    }

    func start(port: UInt16) {
        do {
            let params = NWParameters.tcp
            params.allowLocalEndpointReuse = true
            listener = try NWListener(using: params, on: NWEndpoint.Port(rawValue: port)!)
        } catch {
            print("[DebugServer] Failed to create listener: \(error)")
            return
        }

        listener?.newConnectionHandler = { [weak self] connection in
            self?.handleConnection(connection)
        }

        listener?.stateUpdateHandler = { state in
            switch state {
            case .ready:
                print("[DebugServer] Listening on port \(port)")
            case .failed(let error):
                print("[DebugServer] Listener failed: \(error)")
            default:
                break
            }
        }

        listener?.start(queue: serverQueue)
    }

    func stop() {
        listener?.cancel()
        listener = nil
    }

    // MARK: - Connection Handling

    private func handleConnection(_ connection: NWConnection) {
        connection.start(queue: serverQueue)
        receiveData(on: connection, accumulated: Data())
    }

    private func receiveData(on connection: NWConnection, accumulated: Data) {
        connection.receive(minimumIncompleteLength: 1, maximumLength: 65536) { [weak self] content, _, isComplete, error in
            guard let self = self else { return }

            if let error = error {
                print("[DebugServer] Receive error: \(error)")
                connection.cancel()
                return
            }

            var data = accumulated
            if let content = content {
                data.append(content)
            }

            // Try to parse the HTTP request from accumulated data
            if let request = self.parseHTTPRequest(data) {
                let response = self.router.route(request)
                self.sendResponse(response, on: connection)
            } else if isComplete {
                // Connection closed before full request
                connection.cancel()
            } else {
                // Need more data
                self.receiveData(on: connection, accumulated: data)
            }
        }
    }

    // MARK: - HTTP Parsing

    private func parseHTTPRequest(_ data: Data) -> HTTPRequest? {
        guard let string = String(data: data, encoding: .utf8) else { return nil }

        // Find header/body boundary
        guard let headerEnd = string.range(of: "\r\n\r\n") else { return nil }

        let headerSection = String(string[string.startIndex..<headerEnd.lowerBound])
        let bodyStart = headerEnd.upperBound

        let lines = headerSection.split(separator: "\r\n", maxSplits: 1)
        guard let requestLine = lines.first else { return nil }

        let parts = requestLine.split(separator: " ")
        guard parts.count >= 2 else { return nil }

        let method = String(parts[0])
        let rawPath = String(parts[1])

        // Check Content-Length for body
        var contentLength = 0
        let headerLines = headerSection.split(separator: "\r\n")
        for line in headerLines {
            let lower = line.lowercased()
            if lower.hasPrefix("content-length:") {
                let value = line.dropFirst("content-length:".count).trimmingCharacters(in: .whitespaces)
                contentLength = Int(value) ?? 0
            }
        }

        // Extract body if we have enough data
        let bodyString = String(string[bodyStart...])
        let bodyData = bodyString.data(using: .utf8) ?? Data()
        if contentLength > 0 && bodyData.count < contentLength {
            return nil // Need more data
        }

        // Parse path and query parameters
        let (path, queryParams) = parsePathAndQuery(rawPath)

        let body: Data? = contentLength > 0 ? bodyData.prefix(contentLength) : nil

        return HTTPRequest(
            method: method,
            path: path,
            queryParams: queryParams,
            body: body
        )
    }

    private func parsePathAndQuery(_ rawPath: String) -> (String, [String: String]) {
        let components = rawPath.split(separator: "?", maxSplits: 1)
        let path = String(components[0])

        var queryParams: [String: String] = [:]
        if components.count > 1 {
            let queryString = String(components[1])
            for pair in queryString.split(separator: "&") {
                let kv = pair.split(separator: "=", maxSplits: 1)
                if kv.count == 2 {
                    let key = String(kv[0]).removingPercentEncoding ?? String(kv[0])
                    let value = String(kv[1]).removingPercentEncoding ?? String(kv[1])
                    queryParams[key] = value
                }
            }
        }

        return (path, queryParams)
    }

    // MARK: - Response

    private func sendResponse(_ response: HTTPResponse, on connection: NWConnection) {
        let statusText = httpStatusText(response.statusCode)
        var header = "HTTP/1.1 \(response.statusCode) \(statusText)\r\n"
        header += "Content-Type: \(response.contentType)\r\n"
        header += "Content-Length: \(response.body.count)\r\n"
        header += "Connection: close\r\n"
        header += "Access-Control-Allow-Origin: *\r\n"
        header += "\r\n"

        var responseData = header.data(using: .utf8) ?? Data()
        responseData.append(response.body)

        connection.send(content: responseData, completion: .contentProcessed { _ in
            connection.cancel()
        })
    }

    private func httpStatusText(_ code: Int) -> String {
        switch code {
        case 200: return "OK"
        case 400: return "Bad Request"
        case 404: return "Not Found"
        case 500: return "Internal Server Error"
        default: return "Unknown"
        }
    }
}

#endif
