// NativeDebugServer — Lightweight HTTP debug server for iOS apps.
// Uses Network.framework (NWListener) for zero-dependency HTTP serving.

#if DEBUG
import Foundation
import Network

/// Lightweight HTTP debug server for exposing app state to AI agents and tools.
/// Debug-only — all methods are no-ops in release builds.
///
/// Usage:
/// ```swift
/// DebugServer.shared.start()
/// DebugServer.shared.registerStateProvider("routes") { routeState() }
/// DebugServer.shared.registerAction("navigate") { params in navigate(params) }
/// DebugServer.shared.log(.info, tag: "App", "Started")
/// ```
public final class DebugServer: Sendable {
    public static let shared = DebugServer()

    private let router: DebugRouter
    private let logBuffer: LogBuffer
    private let server: HTTPServer

    private init() {
        self.logBuffer = LogBuffer()
        self.router = DebugRouter(logBuffer: logBuffer)
        self.server = HTTPServer(router: router)
    }

    /// Start the debug server on the given port.
    /// Automatically registers a screenshot provider using ScreenshotCapture.
    public func start(port: UInt16 = 8080, appId: String = Bundle.main.bundleIdentifier ?? "unknown") {
        router.configure(appId: appId)
        registerScreenshotProvider {
            if Thread.isMainThread {
                return MainActor.assumeIsolated { ScreenshotCapture.capturePNG() }
            }
            return DispatchQueue.main.sync {
                MainActor.assumeIsolated { ScreenshotCapture.capturePNG() }
            }
        }
        server.start(port: port)
        log(.info, tag: "DebugServer", "Started on port \(port)")
    }

    /// Stop the debug server.
    public func stop() {
        server.stop()
        log(.info, tag: "DebugServer", "Stopped")
    }

    /// Register a state provider that will be called on `GET /debug/state/<key>`.
    public func registerStateProvider(_ key: String, provider: @escaping @Sendable () -> [String: Any]) {
        router.registerStateProvider(key, provider: provider)
    }

    /// Register an action handler that will be called on `POST /debug/action` with matching type.
    public func registerAction(_ type: String, handler: @escaping @Sendable ([String: Any]) -> [String: Any]) {
        router.registerAction(type, handler: handler)
    }

    /// Register a navigate handler for `POST /debug/navigate`.
    public func registerNavigateHandler(_ handler: @escaping @Sendable (String) -> Bool) {
        router.registerNavigateHandler(handler)
    }

    /// Register a screenshot provider for `GET /debug/screenshot`.
    public func registerScreenshotProvider(_ provider: @escaping @Sendable () -> Data?) {
        router.registerScreenshotProvider(provider)
    }

    /// Add a log entry to the buffer.
    public func log(_ level: LogEntry.Level, tag: String, _ message: String) {
        logBuffer.append(level: level, tag: tag, message: message)
    }
}

#else

// Release stub — all methods are no-ops, zero overhead.
public final class DebugServer: Sendable {
    public static let shared = DebugServer()
    private init() {}

    public func start(port: UInt16 = 8080, appId: String = "") {}
    public func stop() {}
    public func registerStateProvider(_ key: String, provider: @escaping @Sendable () -> [String: Any]) {}
    public func registerAction(_ type: String, handler: @escaping @Sendable ([String: Any]) -> [String: Any]) {}
    public func registerNavigateHandler(_ handler: @escaping @Sendable (String) -> Bool) {}
    public func registerScreenshotProvider(_ provider: @escaping @Sendable () -> Data?) {}
    public func log(_ level: LogEntry.Level, tag: String, _ message: String) {}
}

#endif
