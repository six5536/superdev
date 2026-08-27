# {{superdev:project-name}}

One product, three native codebases: a React web app, a Kotlin/Jetpack
Compose Android app, and a SwiftUI iOS app. Behaviour stays in step across
the three by discipline and tests, not by a shared runtime.

## Layout

- `apps/web` — TypeScript + React + Vite
- `apps/android-native` — Kotlin + Jetpack Compose
- `apps/ios-native` — Swift + SwiftUI (Xcode project generated from
  `project.yml` by xcodegen)
- `libs/` — the native debug servers (Android, iOS), and an MCP server that
  lets agents drive a debug build on a device
- `scripts/` — the dev CLI: build/install/launch/logs/screenshot loops for
  emulator, simulator and device
- `release/` — `release.yaml`, the single source of truth for version and
  app id, plus store metadata

## Develop

Open the repo in the dev container (`.devcontainer/`) and everything is
already there — Node, the JDK, Gradle, the Android SDK, and the superdev
agent tooling. Without it, [mise](https://mise.jdx.dev) installs the same
versions from `mise.toml`:

```sh
mise install
npm install
```

Two one-time bootstrap steps, both in [docs/BUILD.md](docs/BUILD.md): the
Android Gradle wrapper jar (a binary the scaffold cannot carry) and the
generated Xcode project.

```sh
npm run dev            # web dev server
npm run dev:android    # build + install + launch on emulator/device
npm run dev:ios        # build + install + launch on simulator (macOS)
npm run dev:all test   # one command, all three platforms
```

Debugging on a physical Android device from inside the container is covered
in [docs/ANDROID_DEBUGGING.md](docs/ANDROID_DEBUGGING.md).

## Knowledge

The repo's knowledge bundle lives in `knowledge/`, seeded by `superdev init`.
Run the `bootstrap` skill after seeding to harvest these docs into it.

## Release

Version and app id live in `release/release.yaml`; fastlane stamps them into
every platform and ships to the stores. See [docs/RELEASE.md](docs/RELEASE.md).

## License

Proprietary — see [LICENSE](LICENSE).
