// Platform-specific screenshot capture utilities.

#if DEBUG
import Foundation
#if canImport(UIKit)
import UIKit

/// Capture the current key window as PNG data.
/// Must be called on the main thread.
public enum ScreenshotCapture {
    @MainActor
    public static func capturePNG() -> Data? {
        guard let scene = UIApplication.shared.connectedScenes
            .compactMap({ $0 as? UIWindowScene })
            .first,
              let window = scene.windows.first(where: { $0.isKeyWindow }) else {
            return nil
        }

        let renderer = UIGraphicsImageRenderer(bounds: window.bounds)
        let image = renderer.image { ctx in
            window.drawHierarchy(in: window.bounds, afterScreenUpdates: true)
        }
        return image.pngData()
    }
}

#else

/// macOS stub — screenshot not supported outside UIKit.
public enum ScreenshotCapture {
    public static func capturePNG() -> Data? {
        nil
    }
}

#endif
#endif
