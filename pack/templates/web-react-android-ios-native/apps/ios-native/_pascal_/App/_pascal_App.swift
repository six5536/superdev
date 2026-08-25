import SwiftUI
#if canImport({{superdev:project-pascal}})
import {{superdev:project-pascal}}
#endif
#if DEBUG
import NativeDebugServer
#endif

@main
struct {{superdev:project-pascal}}App: App {
    init() {
        #if DEBUG
        DebugServer.shared.start(appId: "com.{{superdev:project-compact}}.app")
        #endif
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}
