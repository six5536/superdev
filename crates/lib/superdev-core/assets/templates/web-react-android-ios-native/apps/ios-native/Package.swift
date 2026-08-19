// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "{{superdev:project-pascal}}",
    defaultLocalization: "en",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(
            name: "{{superdev:project-pascal}}",
            targets: ["{{superdev:project-pascal}}"]
        )
    ],
    dependencies: [
        .package(path: "../../libs/native-debug-server-ios"),
    ],
    targets: [
        .target(
            name: "{{superdev:project-pascal}}",
            path: "{{superdev:project-pascal}}/Sources",
            resources: [
                .process("Resources")
            ]
        ),
        .executableTarget(
            name: "{{superdev:project-pascal}}App",
            dependencies: [
                "{{superdev:project-pascal}}",
                .product(name: "NativeDebugServer", package: "native-debug-server-ios"),
            ],
            path: "{{superdev:project-pascal}}/App"
        ),
        .testTarget(
            name: "{{superdev:project-pascal}}Tests",
            dependencies: ["{{superdev:project-pascal}}"],
            path: "{{superdev:project-pascal}}Tests"
        )
    ]
)
