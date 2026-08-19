# Build Guide

## Repository Structure

```
{{superdev:project-slug}}/
├── apps/
│   ├── web/              TypeScript + React + Vite
│   ├── android-native/   Kotlin + Jetpack Compose
│   └── ios-native/       Swift + SwiftUI
├── libs/                 Shared libraries (native debug servers, debug MCP server)
├── scripts/              CLI automation (dev loops, device automation)
├── release/              Release manifest and store metadata
└── docs/                 Documentation
```

Tool versions are pinned in `mise.toml` — `mise install` puts Node, a JDK,
Gradle and xcodegen on PATH, in the dev container and outside it alike.

## Web (apps/web)

Prerequisites: Node.js (see `mise.toml`)

```bash
# From repo root (delegates via npm workspaces)
npm install          # Install all workspace dependencies
npm run dev          # Start dev server (http://localhost:5173)
npm run build        # Production build → apps/web/dist/
npm run test         # Run Vitest test suite
npm run lint         # ESLint
npm run format       # Prettier

# Or from apps/web directly
cd apps/web
npm run dev
npm run build
npm run test
```

## Android Native (apps/android-native)

Prerequisites: JDK 21, Android SDK 35 (both from `mise install` +
`mise run android-packages` in the dev container; Android Studio works too)

One-time bootstrap: the Gradle wrapper jar is a binary, so the seeded repo
ships `gradlew` and `gradle-wrapper.properties` but not the jar. Generate and
commit it once, with the Gradle that mise pins. Nothing seeded is executable,
so the wrapper script needs its mode set too:

```bash
cd apps/android-native
gradle wrapper            # writes gradle/wrapper/gradle-wrapper.jar
chmod +x gradlew
git add gradle/wrapper/gradle-wrapper.jar
git update-index --chmod=+x gradlew
```

Then:

```bash
cd apps/android-native
./gradlew assembleDebug   # Build debug APK
./gradlew test            # Run unit tests (JVM + Robolectric)
```

Open in Android Studio: File → Open → select `apps/android-native/`

## iOS Native (apps/ios-native)

Prerequisites: Xcode 16+, macOS

The Xcode project is generated, not committed — `project.yml` is the source
of truth:

```bash
cd apps/ios-native
xcodegen generate         # writes {{superdev:project-pascal}}.xcodeproj
```

Build and test:

```bash
cd apps/ios-native
swift build               # Build via Swift Package Manager
swift test                # Run tests

# Or via Xcode
xcodebuild -scheme {{superdev:project-pascal}} -destination 'platform=iOS Simulator,name=iPhone 16' build
```

Open in Xcode: File → Open → select `apps/ios-native/`

## Device automation

`scripts/` wraps the emulator/simulator dev loop — build, install, launch,
logs, screenshots:

```bash
npm run android           # subcommands: build, install, run, logs, screenshot, ...
npm run ios
npm run dev:all test      # run a command across all three platforms
```

For debugging on a physical Android device from the dev container, see
[ANDROID_DEBUGGING.md](ANDROID_DEBUGGING.md).
