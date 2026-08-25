//! templates/web_react_android_ios_native.rs — the web-react-android-ios-native
//! template: one product as three native codebases — a React web app, a
//! Kotlin/Jetpack Compose Android app and a SwiftUI iOS app — plus the
//! device debug tooling that lets agents drive a debug build (native debug
//! servers and an MCP wrapper), a fastlane release pipeline, and an
//! Android-capable dev container. Derived from a real three-platform app,
//! with the app code reduced to hello-world stubs that build and pass CI.
//!
//! On disk the assets follow the seeded layout with leading dots stripped
//! and tokenised segments written `_pascal_`/`_compact_` (see the module
//! docs in [`super`]); this table restores both in the target paths. Two
//! binaries cannot be seeded and are bootstrapped instead (documented in
//! the template's docs/BUILD.md): the Gradle wrapper jar (`gradle wrapper`,
//! with the Gradle that mise pins) and the generated Xcode project
//! (`xcodegen generate`).

use super::Template;

macro_rules! tpl {
    ($rel:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/projects/web-react-android-ios-native/",
            $rel
        ))
    };
}

/// (tokenised target path, embedded content), in asset-path order.
const FILES: [(&str, &str); 109] = [
    ("CHANGELOG.md", tpl!("CHANGELOG.md")),
    ("CODE_OF_CONDUCT.md", tpl!("CODE_OF_CONDUCT.md")),
    ("CONTRIBUTING.md", tpl!("CONTRIBUTING.md")),
    ("LICENSE", tpl!("LICENSE")),
    ("README.md", tpl!("README.md")),
    ("SECURITY.md", tpl!("SECURITY.md")),
    ("appium-capabilities.json", tpl!("appium-capabilities.json")),
    (
        "apps/android-native/app/build.gradle.kts",
        tpl!("apps/android-native/app/build.gradle.kts"),
    ),
    (
        "apps/android-native/app/src/debug/kotlin/com/{{superdev:project-compact}}/app/DebugBridge.kt",
        tpl!("apps/android-native/app/src/debug/kotlin/com/_compact_/app/DebugBridge.kt"),
    ),
    (
        "apps/android-native/app/src/debug/kotlin/com/{{superdev:project-compact}}/app/DebugServerInit.kt",
        tpl!("apps/android-native/app/src/debug/kotlin/com/_compact_/app/DebugServerInit.kt"),
    ),
    (
        "apps/android-native/app/src/main/AndroidManifest.xml",
        tpl!("apps/android-native/app/src/main/AndroidManifest.xml"),
    ),
    (
        "apps/android-native/app/src/main/kotlin/com/{{superdev:project-compact}}/app/MainActivity.kt",
        tpl!("apps/android-native/app/src/main/kotlin/com/_compact_/app/MainActivity.kt"),
    ),
    (
        "apps/android-native/app/src/main/res/values/strings.xml",
        tpl!("apps/android-native/app/src/main/res/values/strings.xml"),
    ),
    (
        "apps/android-native/app/src/main/res/values/themes.xml",
        tpl!("apps/android-native/app/src/main/res/values/themes.xml"),
    ),
    (
        "apps/android-native/app/src/release/kotlin/com/{{superdev:project-compact}}/app/DebugBridge.kt",
        tpl!("apps/android-native/app/src/release/kotlin/com/_compact_/app/DebugBridge.kt"),
    ),
    (
        "apps/android-native/app/src/release/kotlin/com/{{superdev:project-compact}}/app/DebugServerInit.kt",
        tpl!("apps/android-native/app/src/release/kotlin/com/_compact_/app/DebugServerInit.kt"),
    ),
    (
        "apps/android-native/app/src/test/kotlin/com/{{superdev:project-compact}}/app/GreetingTest.kt",
        tpl!("apps/android-native/app/src/test/kotlin/com/_compact_/app/GreetingTest.kt"),
    ),
    (
        "apps/android-native/build.gradle.kts",
        tpl!("apps/android-native/build.gradle.kts"),
    ),
    (
        "apps/android-native/gradle.properties",
        tpl!("apps/android-native/gradle.properties"),
    ),
    (
        "apps/android-native/gradle/wrapper/gradle-wrapper.properties",
        tpl!("apps/android-native/gradle/wrapper/gradle-wrapper.properties"),
    ),
    (
        "apps/android-native/gradlew",
        tpl!("apps/android-native/gradlew"),
    ),
    (
        "apps/android-native/gradlew.bat",
        tpl!("apps/android-native/gradlew.bat"),
    ),
    (
        "apps/android-native/settings.gradle.kts",
        tpl!("apps/android-native/settings.gradle.kts"),
    ),
    (
        "apps/ios-native/Package.swift",
        tpl!("apps/ios-native/Package.swift"),
    ),
    (
        "apps/ios-native/{{superdev:project-pascal}}/App/{{superdev:project-pascal}}App.swift",
        tpl!("apps/ios-native/_pascal_/App/_pascal_App.swift"),
    ),
    (
        "apps/ios-native/{{superdev:project-pascal}}/Sources/ContentView.swift",
        tpl!("apps/ios-native/_pascal_/Sources/ContentView.swift"),
    ),
    (
        "apps/ios-native/{{superdev:project-pascal}}/Sources/Resources/Localizable.xcstrings",
        tpl!("apps/ios-native/_pascal_/Sources/Resources/Localizable.xcstrings"),
    ),
    (
        "apps/ios-native/{{superdev:project-pascal}}Tests/{{superdev:project-pascal}}Tests.swift",
        tpl!("apps/ios-native/_pascal_Tests/_pascal_Tests.swift"),
    ),
    (
        "apps/ios-native/project.yml",
        tpl!("apps/ios-native/project.yml"),
    ),
    (
        "apps/web/eslint.config.js",
        tpl!("apps/web/eslint.config.js"),
    ),
    ("apps/web/index.html", tpl!("apps/web/index.html")),
    ("apps/web/package.json", tpl!("apps/web/package.json")),
    ("apps/web/src/App.tsx", tpl!("apps/web/src/App.tsx")),
    ("apps/web/src/index.css", tpl!("apps/web/src/index.css")),
    (
        "apps/web/src/lib/__tests__/greeting.test.ts",
        tpl!("apps/web/src/lib/__tests__/greeting.test.ts"),
    ),
    (
        "apps/web/src/lib/greeting.ts",
        tpl!("apps/web/src/lib/greeting.ts"),
    ),
    ("apps/web/src/main.tsx", tpl!("apps/web/src/main.tsx")),
    (
        "apps/web/src/vite-env.d.ts",
        tpl!("apps/web/src/vite-env.d.ts"),
    ),
    (
        "apps/web/tsconfig.app.json",
        tpl!("apps/web/tsconfig.app.json"),
    ),
    ("apps/web/tsconfig.json", tpl!("apps/web/tsconfig.json")),
    (
        "apps/web/tsconfig.node.json",
        tpl!("apps/web/tsconfig.node.json"),
    ),
    ("apps/web/vite.config.ts", tpl!("apps/web/vite.config.ts")),
    (".devcontainer/Dockerfile", tpl!("devcontainer/Dockerfile")),
    (
        ".devcontainer/devcontainer-lock.json",
        tpl!("devcontainer/devcontainer-lock.json"),
    ),
    (
        ".devcontainer/devcontainer.json",
        tpl!("devcontainer/devcontainer.json"),
    ),
    (
        ".devcontainer/scripts/install-deps.sh",
        tpl!("devcontainer/scripts/install-deps.sh"),
    ),
    (
        ".devcontainer/scripts/post-create.sh",
        tpl!("devcontainer/scripts/post-create.sh"),
    ),
    (
        ".devcontainer/scripts/post-start.sh",
        tpl!("devcontainer/scripts/post-start.sh"),
    ),
    (
        ".devcontainer/scripts/setup-amd64-multiarch.sh",
        tpl!("devcontainer/scripts/setup-amd64-multiarch.sh"),
    ),
    (".dockerignore", tpl!("dockerignore")),
    (
        "docs/ANDROID_DEBUGGING.md",
        tpl!("docs/ANDROID_DEBUGGING.md"),
    ),
    ("docs/BUILD.md", tpl!("docs/BUILD.md")),
    ("docs/RELEASE.md", tpl!("docs/RELEASE.md")),
    ("fastlane/Appfile", tpl!("fastlane/Appfile")),
    ("fastlane/Fastfile", tpl!("fastlane/Fastfile")),
    ("fastlane/Gemfile", tpl!("fastlane/Gemfile")),
    ("fastlane/Matchfile", tpl!("fastlane/Matchfile")),
    (".gitattributes", tpl!("gitattributes")),
    (
        ".github/workflows/checks.yml",
        tpl!("github/workflows/checks.yml"),
    ),
    (".github/workflows/ci.yml", tpl!("github/workflows/ci.yml")),
    (
        ".github/workflows/release.yml",
        tpl!("github/workflows/release.yml"),
    ),
    (
        ".github/workflows/screenshot.yml",
        tpl!("github/workflows/screenshot.yml"),
    ),
    (".gitignore", tpl!("gitignore")),
    (
        "libs/debug-mcp-server/package.json",
        tpl!("libs/debug-mcp-server/package.json"),
    ),
    (
        "libs/debug-mcp-server/src/DebugClient.ts",
        tpl!("libs/debug-mcp-server/src/DebugClient.ts"),
    ),
    (
        "libs/debug-mcp-server/src/index.ts",
        tpl!("libs/debug-mcp-server/src/index.ts"),
    ),
    (
        "libs/debug-mcp-server/src/tools.ts",
        tpl!("libs/debug-mcp-server/src/tools.ts"),
    ),
    (
        "libs/debug-mcp-server/src/types.ts",
        tpl!("libs/debug-mcp-server/src/types.ts"),
    ),
    (
        "libs/debug-mcp-server/tests/DebugClient.test.ts",
        tpl!("libs/debug-mcp-server/tests/DebugClient.test.ts"),
    ),
    (
        "libs/debug-mcp-server/tests/tools.test.ts",
        tpl!("libs/debug-mcp-server/tests/tools.test.ts"),
    ),
    (
        "libs/debug-mcp-server/tsconfig.json",
        tpl!("libs/debug-mcp-server/tsconfig.json"),
    ),
    (
        "libs/native-debug-server-android/build.gradle.kts",
        tpl!("libs/native-debug-server-android/build.gradle.kts"),
    ),
    (
        "libs/native-debug-server-android/src/main/AndroidManifest.xml",
        tpl!("libs/native-debug-server-android/src/main/AndroidManifest.xml"),
    ),
    (
        "libs/native-debug-server-android/src/main/kotlin/com/nativedebugserver/DebugRouter.kt",
        tpl!(
            "libs/native-debug-server-android/src/main/kotlin/com/nativedebugserver/DebugRouter.kt"
        ),
    ),
    (
        "libs/native-debug-server-android/src/main/kotlin/com/nativedebugserver/DebugServer.kt",
        tpl!(
            "libs/native-debug-server-android/src/main/kotlin/com/nativedebugserver/DebugServer.kt"
        ),
    ),
    (
        "libs/native-debug-server-android/src/main/kotlin/com/nativedebugserver/HttpServer.kt",
        tpl!(
            "libs/native-debug-server-android/src/main/kotlin/com/nativedebugserver/HttpServer.kt"
        ),
    ),
    (
        "libs/native-debug-server-android/src/main/kotlin/com/nativedebugserver/LogBuffer.kt",
        tpl!("libs/native-debug-server-android/src/main/kotlin/com/nativedebugserver/LogBuffer.kt"),
    ),
    (
        "libs/native-debug-server-android/src/test/kotlin/com/nativedebugserver/DebugRouterTest.kt",
        tpl!(
            "libs/native-debug-server-android/src/test/kotlin/com/nativedebugserver/DebugRouterTest.kt"
        ),
    ),
    (
        "libs/native-debug-server-android/src/test/kotlin/com/nativedebugserver/LogBufferTest.kt",
        tpl!(
            "libs/native-debug-server-android/src/test/kotlin/com/nativedebugserver/LogBufferTest.kt"
        ),
    ),
    (
        "libs/native-debug-server-ios/Package.swift",
        tpl!("libs/native-debug-server-ios/Package.swift"),
    ),
    (
        "libs/native-debug-server-ios/Sources/NativeDebugServer/DebugRouter.swift",
        tpl!("libs/native-debug-server-ios/Sources/NativeDebugServer/DebugRouter.swift"),
    ),
    (
        "libs/native-debug-server-ios/Sources/NativeDebugServer/DebugServer.swift",
        tpl!("libs/native-debug-server-ios/Sources/NativeDebugServer/DebugServer.swift"),
    ),
    (
        "libs/native-debug-server-ios/Sources/NativeDebugServer/HTTPServer.swift",
        tpl!("libs/native-debug-server-ios/Sources/NativeDebugServer/HTTPServer.swift"),
    ),
    (
        "libs/native-debug-server-ios/Sources/NativeDebugServer/LogBuffer.swift",
        tpl!("libs/native-debug-server-ios/Sources/NativeDebugServer/LogBuffer.swift"),
    ),
    (
        "libs/native-debug-server-ios/Sources/NativeDebugServer/ScreenshotCapture.swift",
        tpl!("libs/native-debug-server-ios/Sources/NativeDebugServer/ScreenshotCapture.swift"),
    ),
    (
        "libs/native-debug-server-ios/Tests/NativeDebugServerTests/DebugRouterTests.swift",
        tpl!("libs/native-debug-server-ios/Tests/NativeDebugServerTests/DebugRouterTests.swift"),
    ),
    (
        "libs/native-debug-server-ios/Tests/NativeDebugServerTests/LogBufferTests.swift",
        tpl!("libs/native-debug-server-ios/Tests/NativeDebugServerTests/LogBufferTests.swift"),
    ),
    (
        "libs/native-debug-server/API.md",
        tpl!("libs/native-debug-server/API.md"),
    ),
    ("mise.toml", tpl!("mise.toml")),
    ("package.json", tpl!("package.json")),
    (
        ".playwright/cli.config.json",
        tpl!("playwright/cli.config.json"),
    ),
    (".prettierignore", tpl!("prettierignore")),
    (
        "release/metadata/en-US/full_description.txt",
        tpl!("release/metadata/en-US/full_description.txt"),
    ),
    (
        "release/metadata/en-US/keywords.txt",
        tpl!("release/metadata/en-US/keywords.txt"),
    ),
    (
        "release/metadata/en-US/privacy_url.txt",
        tpl!("release/metadata/en-US/privacy_url.txt"),
    ),
    (
        "release/metadata/en-US/screenshots/.gitkeep",
        tpl!("release/metadata/en-US/screenshots/gitkeep"),
    ),
    (
        "release/metadata/en-US/short_description.txt",
        tpl!("release/metadata/en-US/short_description.txt"),
    ),
    (
        "release/metadata/en-US/title.txt",
        tpl!("release/metadata/en-US/title.txt"),
    ),
    ("release/release.yaml", tpl!("release/release.yaml")),
    ("scripts/adb-env.sh", tpl!("scripts/adb-env.sh")),
    ("scripts/android.ts", tpl!("scripts/android.ts")),
    ("scripts/dev.ts", tpl!("scripts/dev.ts")),
    (
        "scripts/host/android-debug.sh",
        tpl!("scripts/host/android-debug.sh"),
    ),
    ("scripts/ios.ts", tpl!("scripts/ios.ts")),
    ("scripts/lib.ts", tpl!("scripts/lib.ts")),
    ("scripts/tsconfig.json", tpl!("scripts/tsconfig.json")),
    (".vscode/extensions.json", tpl!("vscode/extensions.json")),
    (".vscode/mcp.json", tpl!("vscode/mcp.json")),
    (".vscode/settings.json", tpl!("vscode/settings.json")),
];

pub(super) const TEMPLATE: Template = Template {
    name: "web-react-android-ios-native",
    description: "Three native codebases — React web, Compose Android, SwiftUI iOS — with agent debug tooling",
    files: &FILES,
};
