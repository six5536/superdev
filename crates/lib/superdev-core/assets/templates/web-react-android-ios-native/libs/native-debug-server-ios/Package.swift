// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "NativeDebugServer",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(
            name: "NativeDebugServer",
            targets: ["NativeDebugServer"]
        )
    ],
    targets: [
        .target(
            name: "NativeDebugServer",
            path: "Sources/NativeDebugServer"
        ),
        .testTarget(
            name: "NativeDebugServerTests",
            dependencies: ["NativeDebugServer"],
            path: "Tests/NativeDebugServerTests"
        )
    ]
)
