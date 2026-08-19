# Release Guide

## Version Management

All version and build information lives in [release/release.yaml](../release/release.yaml). This is the single source of truth.

### Bump Version

```bash
# Bump patch version (0.1.0 → 0.1.1)
bundle exec fastlane bump_version type:patch

# Bump minor version (0.1.0 → 0.2.0)
bundle exec fastlane bump_version type:minor

# Bump build number only
bundle exec fastlane bump_version type:build
```

`bump_version` automatically runs `prepare_release` to stamp the new version into all platform build files.

### Manual Stamp

If you edit release.yaml directly:

```bash
bundle exec fastlane prepare_release
```

This stamps version into:
- `apps/android-native/app/build.gradle.kts` (versionName, versionCode)
- `apps/ios-native/{{superdev:project-pascal}}/Info.plist` (CFBundleShortVersionString, CFBundleVersion)
- `apps/web/package.json` (version)

## GitHub Actions

### CI (ci.yml)

Triggers on push/PR. Calls the reusable checks.yml, which runs three
parallel jobs:
- Web: lint, test, build
- Android: assembleDebug, test
- iOS: swift build, swift test

release.yml calls the same checks.yml before publishing, so the release
gate cannot drift from CI.

### Release (release.yml)

Triggers on:
- Manual dispatch (choose platform and track)
- Git tags matching `v*`

Platforms: android, ios, web, or all

### Screenshots (screenshot.yml)

Manual dispatch for generating store screenshots (placeholder).

## Required GitHub Secrets

### Android
- `ANDROID_KEYSTORE_BASE64` — Base64-encoded release keystore
- `ANDROID_KEY_ALIAS` — Keystore key alias
- `ANDROID_KEY_PASSWORD` — Keystore key password
- `GOOGLE_PLAY_SERVICE_ACCOUNT_JSON` — Play Store API service account JSON

### iOS
- `MATCH_PASSWORD` — Fastlane match encryption password
- `MATCH_GIT_URL` — Git repo URL for match certificates
- `APP_STORE_CONNECT_API_KEY_PATH` — App Store Connect API key

### Web
- `DEPLOY_TOKEN` — Deployment token for web hosting provider

## Store Metadata

Marketing metadata lives in `release/metadata/{locale}/`:
- `title.txt` — App name
- `short_description.txt` — Store short description
- `full_description.txt` — Store full description
- `keywords.txt` — App Store keywords
- `privacy_url.txt` — Privacy policy URL
- `screenshots/` — Store screenshots

Used by Fastlane `supply` (Android) and `deliver` (iOS).
